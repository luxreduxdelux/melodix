/*
* Copyright (c) 2025 luxreduxdelux
*
* Redistribution and use in source and binary forms, with or without
* modification, are permitted provided that the following conditions are met:
*
* 1. Redistributions of source code must retain the above copyright notice,
* this list of conditions and the following disclaimer.
*
* 2. Redistributions in binary form must reproduce the above copyright notice,
* this list of conditions and the following disclaimer in the documentation
* and/or other materials provided with the distribution.
*
* Subject to the terms and conditions of this license, each copyright holder
* and contributor hereby grants to those receiving rights under this license
* a perpetual, worldwide, non-exclusive, no-charge, royalty-free, irrevocable
* (except for failure to satisfy the conditions of this license) patent license
* to make, have made, use, offer to sell, sell, import, and otherwise transfer
* this software, where such license applies only to those patent claims, already
* acquired or hereafter acquired, licensable by such copyright holder or
* contributor that are necessarily infringed by:
*
* (a) their Contribution(s) (the licensed copyrights of copyright holders and
* non-copyrightable additions of contributors, in source or binary form) alone;
* or
*
* (b) combination of their Contribution(s) with the work of authorship to which
* such Contribution(s) was added by such copyright holder or contributor, if,
* at the time the Contribution is added, such addition causes such combination
* to be necessarily infringed. The patent license shall not apply to any other
* combinations which include the Contribution.
*
* Except as expressly stated above, no rights or licenses from any copyright
* holder or contributor is granted under this license, whether expressly, by
* implication, estoppel or otherwise.
*
* DISCLAIMER
*
* THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
* AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
* IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
* DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDERS OR CONTRIBUTORS BE LIABLE
* FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
* DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
* SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
* CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
* OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
* OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
*/

use crate::app::*;

//================================================================

use eframe::CreationContext;
use eframe::egui;
use rodio::OutputStream;
use rodio::OutputStreamHandle;
use rodio::Sink;
use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, PlatformConfig};
use std::sync::mpsc::{Receiver, Sender};
use tray_icon::TrayIconBuilder;
use tray_icon::TrayIconEvent;
use tray_icon::menu::MenuEvent;
use tray_icon::menu::MenuItemBuilder;

//================================================================

pub struct System {
    sink: Sink,
    stream: OutputStream,
    handle: OutputStreamHandle,
    media: MediaControls,
    click_tx: Sender<String>,
    click_rx: Receiver<String>,
    event_rx: Receiver<MediaControlEvent>,
}

impl System {
    pub fn new(cc: &CreationContext) -> Self {
        let (stream, handle) = rodio::OutputStream::try_default().unwrap();
        let sink = rodio::Sink::try_new(&handle).unwrap();

        let config = PlatformConfig {
            dbus_name: "melodix",
            display_name: "Melodix",
            hwnd: None,
        };

        let mut media = MediaControls::new(config).unwrap();

        let (event_tx, event_rx) = std::sync::mpsc::channel();
        let (click_tx, click_rx) = std::sync::mpsc::channel();

        let clone = cc.egui_ctx.clone();
        media
            .attach(move |event: MediaControlEvent| {
                clone.request_repaint();
                event_tx.send(event).unwrap();
            })
            .unwrap();

        // Since egui uses winit under the hood and doesn't use gtk on Linux, and we need gtk for
        // the tray icon to show up, we need to spawn a thread
        // where we initialize gtk and create the tray_icon
        #[cfg(target_os = "linux")]
        std::thread::spawn(|| {
            use tray_icon::menu::Menu;

            gtk::init().unwrap();

            let tray_menu = tray_icon::menu::Menu::with_items(&[
                &MenuItemBuilder::new().text("Play").enabled(true).build(),
                &MenuItemBuilder::new().text("Skip -").enabled(true).build(),
                &MenuItemBuilder::new().text("Skip +").enabled(true).build(),
                &MenuItemBuilder::new().text("Exit").enabled(true).build(),
            ])
            .unwrap();
            let tray_icon = TrayIconBuilder::new()
                .with_menu(Box::new(tray_menu))
                .with_tooltip("system-tray - tray icon library!")
                .build()
                .unwrap();

            gtk::main();
        });

        Self {
            sink,
            stream,
            handle,
            media,
            click_tx,
            click_rx,
            event_rx,
        }
    }

    pub fn poll_event(&mut self) -> Option<MediaControlEvent> {
        if let Ok(event) = TrayIconEvent::receiver().try_recv() {
            println!("tray event: {:?}", event);
        }

        if let Ok(event) = MenuEvent::receiver().try_recv() {
            match event.id.0.as_str() {
                "1" => return Some(MediaControlEvent::Toggle),
                "2" => return Some(MediaControlEvent::Previous),
                "3" => return Some(MediaControlEvent::Next),
                "4" => return Some(MediaControlEvent::Quit),
                _ => return None,
            }
        }

        if let Ok(click) = self.click_rx.try_recv() {
            match click.as_str() {
                "skip-a" => return Some(MediaControlEvent::Previous),
                "skip-b" => return Some(MediaControlEvent::Next),
                _ => return None,
            }
        }

        if let Ok(event) = self.event_rx.try_recv() {
            return Some(event);
        }

        None
    }

    pub fn make_event(event: MediaControlEvent, app: &mut App, context: &egui::Context) {
        match event {
            MediaControlEvent::Play => app.song_play(),
            MediaControlEvent::Pause => app.song_pause(),
            MediaControlEvent::Toggle => app.song_toggle(),
            MediaControlEvent::Next => app.song_skip_b(),
            MediaControlEvent::Previous => app.song_skip_a(),
            MediaControlEvent::Stop => app.song_stop(),
            MediaControlEvent::Seek(seek_direction) => match seek_direction {
                souvlaki::SeekDirection::Forward => app.song_seek(10, true),
                souvlaki::SeekDirection::Backward => app.song_seek(-10, true),
            },
            MediaControlEvent::SeekBy(seek_direction, duration) => match seek_direction {
                souvlaki::SeekDirection::Forward => app.song_seek(duration.as_secs() as i64, true),
                souvlaki::SeekDirection::Backward => {
                    app.song_seek(-(duration.as_secs() as i64), true)
                }
            },
            MediaControlEvent::SetPosition(media_position) => {
                app.song_seek(media_position.0.as_secs() as i64, false)
            }
            MediaControlEvent::SetVolume(volume) => app.song_set_volume(volume as f32),
            MediaControlEvent::OpenUri(_) => todo!(),
            MediaControlEvent::Raise => context.send_viewport_cmd(egui::ViewportCommand::Focus),
            MediaControlEvent::Quit => context.send_viewport_cmd(egui::ViewportCommand::Close),
        }
    }
}
