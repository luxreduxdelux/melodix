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

use crate::{app::*, library::*, setting::*};

//================================================================

use eframe::{CreationContext, egui};
use notify_rust::{Image, Notification};
use rodio::{OutputStream, OutputStreamHandle, Sink};
use souvlaki::{MediaControlEvent, MediaControls, PlatformConfig};
use std::sync::mpsc::{Receiver, Sender};
use tray_icon::{
    TrayIconBuilder,
    menu::{MenuEvent, MenuItemBuilder},
};

//================================================================

#[allow(dead_code)]
pub struct System {
    /// media sink, for audio play-back.
    pub sink: Sink,
    /// multi-media key event handler.
    pub media: Option<(MediaControls, Receiver<MediaControlEvent>)>,
    /// push notification event handler.
    push: Option<(Sender<String>, Receiver<String>)>,
    /// media sink stream and handle.
    stream: OutputStream,
    /// media sink stream and handle.
    handle: OutputStreamHandle,
}

impl System {
    const TRAY_ICON: &[u8] = include_bytes!("../data/tray.png");
    const TRAY_COMMAND_TOGGLE: &str = "1";
    const TRAY_COMMAND_SKIP_A: &str = "2";
    const TRAY_COMMAND_SKIP_B: &str = "3";
    const TRAY_COMMAND_EXIT: &str = "4";
    const PUSH_COMMAND_SKIP_A: &str = "skip_a";
    const PUSH_COMMAND_SKIP_B: &str = "skip_b";

    pub fn new(setting: &Setting, context: &CreationContext) -> anyhow::Result<Self> {
        let (stream, handle) = rodio::OutputStream::try_default()?;
        let sink = rodio::Sink::try_new(&handle)?;

        let config = PlatformConfig {
            dbus_name: "melodix",
            display_name: "Melodix",
            hwnd: None,
        };

        let media = {
            if setting.window_media {
                let mut media = MediaControls::new(config)?;

                let clone = context.egui_ctx.clone();
                let (event_tx, media_rx) = std::sync::mpsc::channel();

                media.attach(move |event: MediaControlEvent| {
                    clone.request_repaint();
                    event_tx
                        .send(event)
                        .expect("System::new(): Couldn't send media event.");
                })?;

                Some((media, media_rx))
            } else {
                None
            }
        };

        if setting.window_tray {
            std::thread::spawn(|| {
                gtk::init().expect("System::new(): Couldn't create GTK instance.");

                let tray_menu = tray_icon::menu::Menu::with_items(&[
                    &MenuItemBuilder::new().text("Play").enabled(true).build(),
                    &MenuItemBuilder::new().text("Skip -").enabled(true).build(),
                    &MenuItemBuilder::new().text("Skip +").enabled(true).build(),
                    &MenuItemBuilder::new().text("Exit").enabled(true).build(),
                ])
                .expect("System::new(): Couldn't create tray menu.");

                let image = image::load_from_memory(Self::TRAY_ICON)
                    .expect("System::new(): Couldn't load tray icon image.")
                    .into_bytes();
                let _tray = TrayIconBuilder::new()
                    .with_menu(Box::new(tray_menu))
                    .with_icon(
                        tray_icon::Icon::from_rgba(image, 32, 32)
                            .expect("System::new(): Couldn't use tray icon image."),
                    )
                    .build()
                    .expect("System::new(): Couldn't create tray icon.");

                gtk::main();
            });
        }

        let push = {
            if setting.window_push {
                Some(std::sync::mpsc::channel())
            } else {
                None
            }
        };

        Ok(Self {
            sink,
            stream,
            handle,
            media,
            push,
        })
    }

    pub fn poll_event(&mut self) -> Option<MediaControlEvent> {
        // if multi-media key event handler is present, try reading event.
        if let Some((_, media_rx)) = self.media.as_ref() {
            if let Ok(event) = media_rx.try_recv() {
                return Some(event);
            }
        }

        // if tray notification event handler is present, try reading event.
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            match event.id.0.as_str() {
                Self::TRAY_COMMAND_TOGGLE => return Some(MediaControlEvent::Toggle),
                Self::TRAY_COMMAND_SKIP_A => return Some(MediaControlEvent::Previous),
                Self::TRAY_COMMAND_SKIP_B => return Some(MediaControlEvent::Next),
                Self::TRAY_COMMAND_EXIT => return Some(MediaControlEvent::Quit),
                _ => return None,
            }
        }

        // if push notification event handler is present, try reading event.
        if let Some((_, push_rx)) = self.push.as_ref() {
            if let Ok(click) = push_rx.try_recv() {
                match click.as_str() {
                    Self::PUSH_COMMAND_SKIP_A => return Some(MediaControlEvent::Previous),
                    Self::PUSH_COMMAND_SKIP_B => return Some(MediaControlEvent::Next),
                    _ => return None,
                }
            }
        }

        None
    }

    #[rustfmt::skip]
    pub fn make_event(
        event: MediaControlEvent,
        app: &mut App,
        context: &egui::Context,
    ) -> anyhow::Result<()> {
        match event {
            MediaControlEvent::Play                 => app.track_play(),
            MediaControlEvent::Pause                => app.track_pause(),
            MediaControlEvent::Toggle               => app.track_toggle(),
            MediaControlEvent::Next                 => app.track_skip_b(context)?,
            MediaControlEvent::Previous             => app.track_skip_a(context)?,
            MediaControlEvent::Stop                 => app.track_stop(),
            MediaControlEvent::Seek(seek_direction) => match seek_direction {
                souvlaki::SeekDirection::Forward  => app.track_seek( 10, true),
                souvlaki::SeekDirection::Backward => app.track_seek(-10, true),
            },
            MediaControlEvent::SeekBy(seek_direction, duration) => match seek_direction {
                souvlaki::SeekDirection::Forward  => app.track_seek(duration.as_secs() as i64, true),
                souvlaki::SeekDirection::Backward => {
                    app.track_seek(-(duration.as_secs() as i64), true)
                }
            },
            MediaControlEvent::SetPosition(media_position) => {
                app.track_seek(media_position.0.as_secs() as i64, false)
            }
            MediaControlEvent::SetVolume(volume) => app.track_set_volume(volume as f32),
            MediaControlEvent::Raise             => context.send_viewport_cmd(egui::ViewportCommand::Focus),
            MediaControlEvent::Quit              => context.send_viewport_cmd(egui::ViewportCommand::Close),
            _ => {}
        }

        Ok(())
    }

    #[rustfmt::skip]
    pub fn push_notification(&self, context: &egui::Context, state: (&Group, &Album, &Track)) -> anyhow::Result<()> {
        // if push notification event handler is present, send push notification.
        if let Some((push_tx, _)) = self.push.as_ref() {
            let mut notification = Notification::new();

            // build notification body.
            notification
                .summary("Melodix")
                .auto_icon()
                .body(&format!(
                    "{}\n{}\n{}",
                    state.0.name, state.1.name, state.2.name
                ))
                .action(Self::PUSH_COMMAND_SKIP_A, "Skip - 1")
                .action(Self::PUSH_COMMAND_SKIP_B, "Skip + 1");

            let mut use_icon = false;

            // if the current track has an icon and dimension for the icon...
            if let Some(icon) = &state.2.icon.0 && let Some(size) = state.2.icon.1 {
                // try loading the icon, either as an RGBA or RGB image.
                let icon = {
                    if let Ok(icon) = Image::from_rgba(size.0 as i32, size.1 as i32, icon.to_vec()) {
                        Some(icon)
                    } else if let Ok(icon) = Image::from_rgb(size.0 as i32, size.1 as i32, icon.to_vec()) {
                        Some(icon)
                    } else {
                        None
                    }
                };

                // if we could load the icon, set it as the notification icon.
                if let Some(icon) = icon {
                    use_icon = true;
                    notification.image_data(icon);
                }
            }

            // track icon isn't present or couldn't be set, but we have an icon for the album.
            if let Some(icon) = &state.1.icon && !use_icon {
                // set it as the notification icon.
                notification.image_path(icon);
            }

            // clone context, push sender.
            let context = context.clone();
            let push_tx = push_tx.clone();

            let notification = notification.show()?;

            // send notification, await action from user.
            std::thread::spawn(move || {
                notification.wait_for_action(move |action| {
                    context.request_repaint();
                    push_tx.send(action.to_string()).expect("System::push_notification(): Couldn't send push event.");
                });
            });
        }

        Ok(())
    }
}
