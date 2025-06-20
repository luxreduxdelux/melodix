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

use crate::layout::*;
use crate::library::*;
use crate::script::*;
use crate::setting::*;

//================================================================

use eframe::CreationContext;
use eframe::egui::ImageSource;
use eframe::egui::Shape;
use eframe::egui::TextureOptions;
use eframe::egui::{self, Slider, Vec2};
use mlua::prelude::*;
use notify_rust::Hint;
use notify_rust::Notification;
use rodio::OutputStream;
use rodio::OutputStreamHandle;
use rodio::Sink;
use rodio::Source;
use serde::{Deserialize, Serialize};
use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, PlatformConfig};
use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;
use tray_icon::TrayIconEvent;
use tray_icon::menu::MenuEvent;
use tray_icon::menu::MenuItem;
use tray_icon::menu::MenuItemBuilder;
use tray_icon::menu::PredefinedMenuItem;
use tray_icon::{TrayIconBuilder, menu::Menu};

//================================================================

// TO-DO minimize/hide window, tray icon, welcome menu, configuration menu, plug-in menu, plug-in setting data, add Artist/Album/Song header

pub struct State {
    pub library: Library,
    pub setting: Setting,
    pub filter_group: Vec<usize>,
    pub filter_album: Vec<usize>,
    pub filter_track: Vec<usize>,
}

impl State {
    fn new() -> (Self, bool) {
        let (library, new_library) = Library::new();
        let (setting, _) = Setting::new();

        (
            Self {
                filter_group: (0..library.list_artist.len()).collect(),
                filter_album: Vec::new(),
                filter_track: Vec::new(),
                library,
                setting,
            },
            new_library,
        )
    }
}

pub struct App {
    pub state: State,
    pub search_state: (String, String, String),
    pub select_state: (Option<usize>, Option<usize>, Option<usize>),
    pub active_state: Option<(usize, usize, usize, u64)>,
    pub layout: Layout,
    pub replay: bool,
    pub random: bool,
    pub sink: Sink,
    pub script: Script,
    stream: OutputStream,
    handle: OutputStreamHandle,
    media: MediaControls,
    ctx: egui::Context,
    click_tx: Sender<String>,
    click_rx: Receiver<String>,
    event_rx: Receiver<MediaControlEvent>,
}

impl App {
    // get the currently active artist, album, and song.
    pub fn get_play_state(&self) -> (&Artist, &Album, &Song) {
        let artist = self
            .state
            .library
            .list_artist
            .get(self.active_state.as_ref().unwrap().0)
            .unwrap();
        let album = artist
            .list_album
            .get(self.active_state.as_ref().unwrap().1)
            .unwrap();
        let song = album
            .list_song
            .get(self.active_state.as_ref().unwrap().2)
            .unwrap();

        (artist, album, song)
    }

    pub fn song_add(&mut self) {
        let clone = self.active_state;

        self.active_state = Some((
            self.select_state
                .0
                .expect("song_add(): Incorrect unwrap on member 0."),
            self.select_state
                .1
                .expect("song_add(): Incorrect unwrap on member 1."),
            self.select_state
                .2
                .expect("song_add(): Incorrect unwrap on member 2."),
            0,
        ));

        let path = {
            let (_, _, song) = self.get_play_state();
            song.path.clone()
        };

        if let Ok(file) = std::fs::File::open(path)
            && let Ok(source) = rodio::Decoder::new(BufReader::new(file))
        {
            let time = source.total_duration().unwrap_or_default().as_secs();
            let state = self.active_state.unwrap();
            self.active_state = Some((state.0, state.1, state.2, time));

            self.sink.stop();
            self.sink.append(source);
            self.sink.play();

            let (artist, album, song) = self.get_play_state();

            let t_0 = artist.name.clone();
            let t_1 = album.name.clone();
            let t_2 = song.name.clone();

            self.script.call(Script::CALL_PLAY, (t_0, t_1, t_2, time));

            let t_0 = artist.name.clone();
            let t_1 = album.name.clone();
            let t_2 = song.name.clone();
            let t_3 = album.icon.clone();
            let cx = self.ctx.clone();
            let tx = self.click_tx.clone();

            /*std::thread::spawn(move || {
                Notification::new()
                    .summary("Melodix")
                    .image_path("/home/think/Pictures/trent.jpg")
                    .icon("/home/think/Pictures/trent.jpg")
                    .body(&format!("{t_0}\n{t_1}\n{t_2}"))
                    .action("skip-a", "Skip - 1") // IDENTIFIER, LABEL
                    .action("skip-b", "Skip + 1") // IDENTIFIER, LABEL
                    .show()
                    .unwrap()
                    .wait_for_action(move |action| {
                        cx.request_repaint();
                        tx.send(action.to_string()).unwrap();
                    });
            });*/

            self.media
                .set_metadata(MediaMetadata {
                    title: Some(&song.name.clone()),
                    album: Some(&album.name.clone()),
                    artist: Some(&artist.name.clone()),
                    cover_url: album.icon.clone().as_deref(),
                    duration: None,
                })
                .unwrap();
        } else {
            self.active_state = clone;
        }
    }

    pub fn song_toggle(&self) {
        if self.sink.is_paused() {
            self.sink.play();

            self.script.call(Script::CALL_PLAY, ());
        } else {
            self.sink.pause();

            self.script.call(Script::CALL_PAUSE, ());
        }
    }

    pub fn song_seek(&self, seek: i64, delta: bool) {
        let seek = {
            if delta {
                seek + self.sink.get_pos().as_secs() as i64
            } else {
                seek
            }
        };

        self.sink
            .try_seek(Duration::from_secs(seek as u64))
            .unwrap();
    }

    pub fn song_play(&self) {
        self.sink.play();
    }

    pub fn song_pause(&self) {
        self.sink.pause();
    }

    pub fn song_set_volume(&self, volume: f32) {
        self.sink.set_volume(volume);
    }

    pub fn song_stop(&mut self) {
        self.active_state = None;
        self.sink.stop();
    }

    pub fn song_skip_a(&mut self) {
        /*
        let track = {
            let (_, _, song) = self.active_state.unwrap();
            let (_, _, song) = self.active_state.unwrap();

            album.list_song.get(song - 1)
        };

        if let Some(track) = track {
            self.select_state.2 = Some(track);
            self.song_add();
        }
        */
    }

    pub fn song_skip_b(&mut self) {
        /*
        let track = {
            let (_, album, song) = self.get_play_state();

            // TO-DO not a good way to find the next track. will crash on under/over-flow.
            album.list_song.get(song.track - 1).map(|get| get.track)
        };

        if let Some(track) = track {
            self.select_state.2 = Some(track);
            self.song_add();
        }
        */
    }

    pub fn error(message: &str) {
        rfd::MessageDialog::new()
            .set_level(rfd::MessageLevel::Error)
            .set_title("Error")
            .set_description(message)
            .show();
    }

    pub fn new(cc: &CreationContext) -> Self {
        let (stream, handle) = rodio::OutputStream::try_default().unwrap();
        let sink = rodio::Sink::try_new(&handle).unwrap();
        let (state, default) = State::new();
        let script = Script::new(&state.setting);

        let config = PlatformConfig {
            dbus_name: "melodix",
            display_name: "Melodix",
            hwnd: None,
        };

        cc.egui_ctx.set_zoom_factor(state.setting.window_scale);

        if state.setting.window_theme {
            cc.egui_ctx.set_theme(egui::Theme::Light);
        } else {
            cc.egui_ctx.set_theme(egui::Theme::Dark);
        }

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
            state,
            script,
            search_state: (String::default(), String::default(), String::default()),
            select_state: (None, None, None),
            active_state: None,
            replay: false,
            random: false,
            stream,
            handle,
            sink,
            media,
            layout: Layout::new(default),
            ctx: cc.egui_ctx.clone(),
            click_tx,
            click_rx,
            event_rx,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, context: &egui::Context, _: &mut eframe::Frame) {
        // self.handle_media_event();
        // self.handle_tray_event();
        // self.handle_menu_event(context);

        if let Ok(event) = TrayIconEvent::receiver().try_recv() {
            println!("tray event: {:?}", event);
        }

        if let Ok(event) = MenuEvent::receiver().try_recv() {
            match event.id.0.as_str() {
                "1" => self.song_toggle(),
                "2" => self.song_skip_a(),
                "3" => self.song_skip_b(),
                "4" => context.send_viewport_cmd(egui::ViewportCommand::Close),
                _ => {}
            }
        }

        if let Ok(click) = self.click_rx.try_recv() {
            match click.as_str() {
                "skip-a" => self.song_skip_a(),
                "skip-b" => self.song_skip_b(),
                _ => {}
            }
        }

        if let Ok(event) = self.event_rx.try_recv() {
            match event {
                MediaControlEvent::Play => self.song_play(),
                MediaControlEvent::Pause => self.song_pause(),
                MediaControlEvent::Toggle => self.song_toggle(),
                MediaControlEvent::Next => self.song_skip_b(),
                MediaControlEvent::Previous => self.song_skip_a(),
                MediaControlEvent::Stop => self.song_stop(),
                MediaControlEvent::Seek(seek_direction) => match seek_direction {
                    souvlaki::SeekDirection::Forward => self.song_seek(10, true),
                    souvlaki::SeekDirection::Backward => self.song_seek(-10, true),
                },
                MediaControlEvent::SeekBy(seek_direction, duration) => match seek_direction {
                    souvlaki::SeekDirection::Forward => {
                        self.song_seek(duration.as_secs() as i64, true)
                    }
                    souvlaki::SeekDirection::Backward => {
                        self.song_seek(-(duration.as_secs() as i64), true)
                    }
                },
                MediaControlEvent::SetPosition(media_position) => {
                    self.song_seek(media_position.0.as_secs() as i64, false)
                }
                MediaControlEvent::SetVolume(volume) => self.song_set_volume(volume as f32),
                MediaControlEvent::OpenUri(_) => todo!(),
                MediaControlEvent::Raise => context.send_viewport_cmd(egui::ViewportCommand::Focus),
                MediaControlEvent::Quit => context.send_viewport_cmd(egui::ViewportCommand::Close),
            }
        }

        Layout::draw(self, context);
    }
}
