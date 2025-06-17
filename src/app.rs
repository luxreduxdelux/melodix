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

use crate::library::*;
use crate::script::*;
use crate::setting::*;
use std::sync::Arc;
use std::sync::Mutex;

//================================================================

use eframe::egui::ImageSource;
use eframe::egui::Shape;
use eframe::egui::TextureOptions;
use eframe::egui::{self, Slider, Vec2};
use mlua::prelude::*;
use rodio::OutputStream;
use rodio::OutputStreamHandle;
use rodio::Sink;
use serde::{Deserialize, Serialize};
use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, PlatformConfig};
use std::fs::File;
use std::io::BufReader;
use std::thread;
use std::time::Duration;

//================================================================

// TO-DO minimize/hide window, tray icon, welcome menu, configuration menu, plug-in menu, plug-in setting data, add Artist/Album/Song header

#[derive(Serialize, Deserialize)]
pub struct State {
    pub library: Library,
    pub setting: Setting,
}

impl Default for State {
    fn default() -> Self {
        if let Ok(file) = std::fs::read("melodix.data") {
            println!("Reading from melodix.data...");
            let app = postcard::from_bytes(&file).unwrap();
            return app;
        }

        Self {
            library: Library::default(),
            setting: Setting::default(),
        }
    }
}

impl Drop for State {
    fn drop(&mut self) {
        let serialize: Vec<u8> = postcard::to_allocvec(&self).unwrap();
        std::fs::write("melodix.data", serialize).unwrap();
    }
}

pub struct App {
    pub state: State,
    script: Arc<Mutex<Script>>,
    search_state: (String, String, String),
    select_state: (Option<String>, Option<String>, Option<usize>),
    active_state: Option<(String, String, usize)>,
    replay: bool,
    random: bool,
    stream: OutputStream,
    handle: OutputStreamHandle,
    sink: Arc<Mutex<Sink>>,
    media: MediaControls,
}

impl App {
    const IMAGE_SKIP_A: eframe::egui::ImageSource<'_> = egui::include_image!("../data/skip_a.png");
    const IMAGE_SKIP_B: eframe::egui::ImageSource<'_> = egui::include_image!("../data/skip_b.png");
    const IMAGE_PLAY: eframe::egui::ImageSource<'_> = egui::include_image!("../data/play.png");
    const IMAGE_PAUSE: eframe::egui::ImageSource<'_> = egui::include_image!("../data/pause.png");
    //const IMAGE_REPLAY: eframe::egui::ImageSource<'_> = egui::include_image!("../data/pause.png");
    //const IMAGE_RANDOM: eframe::egui::ImageSource<'_> = egui::include_image!("../data/pause.png");
    //const IMAGE_VOLUME: eframe::egui::ImageSource<'_> = egui::include_image!("../data/pause.png");

    //================================================================

    // get the currently active artist, album, and song.
    fn get_play_state(&self) -> (&Artist, &Album, &Song) {
        let artist = self
            .state
            .library
            .map_artist
            .get(&self.active_state.as_ref().unwrap().0)
            .unwrap();
        let album = artist
            .map_album
            .get(&self.active_state.as_ref().unwrap().1)
            .unwrap();
        let song = album
            .list_song
            .get(self.active_state.as_ref().unwrap().2)
            .unwrap();

        (artist, album, song)
    }

    fn song_play(&mut self) {
        self.active_state = Some((
            self.select_state
                .0
                .clone()
                .expect("song_play(): Incorrect unwrap on member 0."),
            self.select_state
                .1
                .clone()
                .expect("song_play(): Incorrect unwrap on member 1."),
            self.select_state
                .2
                .expect("song_play(): Incorrect unwrap on member 2."),
        ));

        let (_, _, song) = self.get_play_state();

        if let Ok(sink) = self.sink.lock() {
            sink.stop();
            let file = std::fs::File::open(&song.path).unwrap();
            sink.append(rodio::Decoder::new(BufReader::new(file)).unwrap());
            sink.play();
        }
    }

    fn song_toggle(&self) {
        if let Ok(sink) = self.sink.lock() {
            if sink.is_paused() {
                sink.play();

                self.script.lock().unwrap().call(Script::CALL_PLAY, ());
            } else {
                sink.pause();

                self.script.lock().unwrap().call(Script::CALL_PAUSE, ());
            }
        }
    }

    fn song_seek(&self, seek: u64) {
        if let Ok(sink) = self.sink.lock() {
            sink.try_seek(Duration::from_secs(seek)).unwrap();
        }
    }

    fn song_stop(&self) {
        if let Ok(sink) = self.sink.lock() {
            sink.stop();
        }
    }

    fn song_skip_a(&mut self) {
        let track = {
            let (_, album, song) = self.get_play_state();

            album.list_song.get(song.track - 2).map(|get| get.track)
        };

        if let Some(track) = track {
            self.select_state.2 = Some(track);
            self.song_play();
        }
    }

    fn song_skip_b(&mut self) {
        let track = {
            let (_, album, song) = self.get_play_state();

            // TO-DO not a good way to find the next track.
            album.list_song.get(song.track - 1).map(|get| get.track)
        };

        if let Some(track) = track {
            self.select_state.2 = Some(track);
            self.song_play();
        }
    }

    fn format_time(time: usize) -> String {
        let time_a = time / 60;
        let time_b = time % 60;

        let time_a = {
            if time_a < 10 {
                format!("0{time_a}")
            } else {
                time_a.to_string()
            }
        };

        let time_b = {
            if time_b < 10 {
                format!("0{time_b}")
            } else {
                time_b.to_string()
            }
        };

        format!("{time_a}:{time_b}")
    }

    pub fn error(message: &str) {
        rfd::MessageDialog::new()
            .set_level(rfd::MessageLevel::Error)
            .set_title("Error")
            .set_description(message)
            .show();
    }

    // draw the top-most tool-bar.
    fn draw_panel_tool(&mut self, _: &egui::Context) {}

    // draw the top song status bar. hidden if no song is available.
    fn draw_panel_song(&mut self, context: &egui::Context) {
        let mut toggle = false;
        let mut seek = None;
        let mut skip_a = false;
        let mut skip_b = false;

        if let Some(active) = &self.active_state {
            egui::TopBottomPanel::top("status")
                .min_height(96.0)
                .max_height(96.0)
                .show(context, |ui| {
                    ui.horizontal(|ui| {
                        let (_, album, song) = self.get_play_state();

                        if let Some(icon) = &album.icon {
                            let image = egui::Image::new(format!("file://{icon}"))
                                .texture_options(
                                    TextureOptions::default()
                                        .with_mipmap_mode(Some(egui::TextureFilter::Nearest)),
                                )
                                .fit_to_exact_size(Vec2::new(96.0, 96.0));

                            ui.add(image);
                        }

                        ui.vertical(|ui| {
                            ui.add_space(8.0);
                            ui.label(&active.0);
                            ui.label(&active.1);
                            ui.label(&song.name);
                        });

                        ui.separator();

                        ui.add(egui::Button::opt_image_and_text(
                            Some(
                                egui::Image::new(Self::IMAGE_SKIP_A)
                                    .fit_to_exact_size(Vec2::new(32.0, 32.0)),
                            ),
                            None,
                        ))
                        .clicked();

                        let image = if self.sink.lock().unwrap().is_paused() {
                            Self::IMAGE_PLAY
                        } else {
                            Self::IMAGE_PAUSE
                        };

                        if ui
                            .add(egui::Button::opt_image_and_text(
                                Some(
                                    egui::Image::new(image)
                                        .fit_to_exact_size(Vec2::new(32.0, 32.0)),
                                ),
                                None,
                            ))
                            .clicked()
                        {
                            toggle = true;
                        }

                        if ui
                            .add(egui::Button::opt_image_and_text(
                                Some(
                                    egui::Image::new(Self::IMAGE_SKIP_B)
                                        .fit_to_exact_size(Vec2::new(32.0, 32.0)),
                                ),
                                None,
                            ))
                            .clicked()
                        {
                            skip_b = true;
                        }

                        if let Ok(sink) = self.sink.lock() {
                            let mut time = sink.get_pos().as_secs();

                            if ui
                                .add(
                                    Slider::new(&mut time, 0..=song.time as u64)
                                        .trailing_fill(true)
                                        .show_value(false),
                                )
                                .changed()
                            {
                                seek = Some(time);
                            }

                            let play_time = Self::format_time(sink.get_pos().as_secs() as usize);
                            let song_time = Self::format_time(song.time);

                            ui.label(format!("{}/{}", play_time, song_time));
                        }
                    });
                });
        }

        if toggle {
            self.song_toggle();
        }

        if let Some(seek) = seek {
            self.song_seek(seek);
        }

        if skip_a {
            self.song_skip_a();
        }

        if skip_b {
            self.song_skip_b();
        }
    }

    // draw the L-most panel.
    fn draw_panel_side_a(&mut self, context: &egui::Context) {
        egui::SidePanel::left("panel_0")
            .resizable(true)
            .show(context, |ui| {
                ui.add_space(6.0);
                ui.text_edit_singleline(&mut self.search_state.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for i in self.state.library.map_artist.keys() {
                        if i.to_lowercase()
                            .contains(&self.search_state.0.to_lowercase().trim())
                        {
                            if ui
                                .selectable_value(&mut self.select_state.0, Some(i.to_string()), i)
                                .clicked()
                            {
                                self.select_state.1 = None;
                            }
                        }
                    }
                });
            });
    }

    // draw the center panel.
    fn draw_panel_center(&mut self, context: &egui::Context) {
        egui::CentralPanel::default().show(context, |ui| {
            ui.text_edit_singleline(&mut self.search_state.1);

            egui::ScrollArea::vertical().show(ui, |ui| {
                if let Some(select_0) = &self.select_state.0 {
                    let artist = self
                        .state
                        .library
                        .map_artist
                        .get(select_0)
                        .expect("draw_panel_center(): Incorrect unwrap.");

                    for i in artist.map_album.keys() {
                        if i.to_lowercase()
                            .contains(&self.search_state.1.to_lowercase().trim())
                        {
                            if ui
                                .selectable_value(&mut self.select_state.1, Some(i.to_string()), i)
                                .clicked()
                            {
                                self.select_state.2 = None;
                            }
                        }
                    }
                }
            });
        });
    }

    // draw the R-most panel.
    fn draw_panel_side_b(&mut self, context: &egui::Context) {
        let mut click = false;

        egui::SidePanel::right("panel_1")
            .resizable(true)
            .show(context, |ui| {
                ui.add_space(6.0);
                ui.text_edit_singleline(&mut self.search_state.2);

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if let Some(select_1) = &self.select_state.1 {
                            let artist = self
                                .state
                                .library
                                .map_artist
                                .get(self.select_state.0.as_ref().unwrap())
                                .expect("draw_panel_side_b(): Incorrect unwrap (artist).");
                            let album = artist
                                .map_album
                                .get(select_1)
                                .expect("draw_panel_side_b(): Incorrect unwrap (album).");

                            for (i, song) in album.list_song.iter().enumerate() {
                                if song
                                    .name
                                    .to_lowercase()
                                    .contains(&self.search_state.2.to_lowercase().trim())
                                {
                                    if ui
                                        .selectable_value(
                                            &mut self.select_state.2,
                                            Some(i),
                                            format!("{} | {}", song.track, &song.name),
                                        )
                                        .clicked()
                                    {
                                        click = true;
                                    }
                                }
                            }
                        }
                    });
            });

        if click {
            self.song_play();
        }
    }
}

impl Default for App {
    fn default() -> Self {
        let (stream, handle) = rodio::OutputStream::try_default().unwrap();
        let sink = Arc::new(Mutex::new(rodio::Sink::try_new(&handle).unwrap()));
        let state = State::default();
        let script = Arc::new(Mutex::new(Script::new(&state.setting)));

        let config = PlatformConfig {
            dbus_name: "melodix",
            display_name: "Melodix",
            hwnd: None,
        };

        let mut media = MediaControls::new(config).unwrap();

        let sink_clone = sink.clone();
        let scri_clone = script.clone();
        // The closure must be Send and have a static lifetime.
        media
            .attach(move |event: MediaControlEvent| match event {
                MediaControlEvent::Toggle => {
                    let sink = sink_clone.lock().unwrap();

                    if sink.is_paused() {
                        sink.play();
                        scri_clone.lock().unwrap().call(Script::CALL_PLAY, ());
                    } else {
                        sink.pause();
                    }
                }
                _ => {}
            })
            .unwrap();

        // Update the media metadata.
        media
            .set_metadata(MediaMetadata {
                title: Some("Souvlaki Space Station"),
                artist: Some("Slowdive"),
                album: Some("Souvlaki"),
                ..Default::default()
            })
            .unwrap();

        Self {
            state: State::default(),
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
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, context: &egui::Context, _: &mut eframe::Frame) {
        context.request_repaint_after_secs(1.0);

        self.draw_panel_tool(context);
        self.draw_panel_song(context);
        self.draw_panel_side_a(context);
        self.draw_panel_center(context);
        self.draw_panel_side_b(context);
    }
}
