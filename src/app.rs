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
use crate::setting::*;

//================================================================

use eframe::egui::{self, Slider, Vec2};
use mlua::prelude::*;
use rodio::OutputStream;
use rodio::OutputStreamHandle;
use rodio::Sink;
use serde::{Deserialize, Serialize};
use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, PlatformConfig};
use std::fs::File;
use std::io::BufReader;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

//================================================================

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
    lua: Lua,
    table: mlua::Table,
    select_artist: String,
    select_album: String,
    select_song: usize,
    search_artist: String,
    search_album: String,
    search_song: String,
    current_artist: Option<String>,
    current_album: Option<String>,
    current_song: Option<(String, usize)>,
    current_time: Option<usize>,
    stream: OutputStream,
    handle: OutputStreamHandle,
    sink: Sink,
    song: Option<Song>,
    media: MediaControls,
    event: Receiver<MediaControlEvent>,
}

impl App {
    const IMAGE_SKIP_A: eframe::egui::ImageSource<'_> = egui::include_image!("../data/skip_a.png");
    const IMAGE_SKIP_B: eframe::egui::ImageSource<'_> = egui::include_image!("../data/skip_b.png");
    const IMAGE_PLAY: eframe::egui::ImageSource<'_> = egui::include_image!("../data/play.png");
    const IMAGE_PAUSE: eframe::egui::ImageSource<'_> = egui::include_image!("../data/pause.png");
}

impl Default for App {
    fn default() -> Self {
        let (stream, handle) = rodio::OutputStream::try_default().unwrap();
        let sink = rodio::Sink::try_new(&handle).unwrap();
        let lua = Lua::new();
        let table: mlua::Table = lua
            .load(std::fs::read_to_string("test/script.lua").unwrap())
            .eval()
            .unwrap();

        table
            .get::<mlua::Function>("main")
            .unwrap()
            .call::<()>(())
            .unwrap();

        let config = PlatformConfig {
            dbus_name: "melodix",
            display_name: "Melodix",
            hwnd: None,
        };

        let mut media = MediaControls::new(config).unwrap();

        let (tx, event) = channel();

        // The closure must be Send and have a static lifetime.
        media
            .attach(move |event: MediaControlEvent| {
                tx.send(event).unwrap();
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
            lua,
            table,
            select_artist: String::default(),
            select_album: String::default(),
            select_song: usize::default(),
            search_artist: String::default(),
            search_album: String::default(),
            search_song: String::default(),
            current_artist: None,
            current_album: None,
            current_song: None,
            current_time: None,
            stream,
            handle,
            sink,
            song: None,
            media,
            event,
        }
    }
}

impl eframe::App for App {
    fn raw_input_hook(&mut self, _ctx: &egui::Context, _raw_input: &mut egui::RawInput) {
        if let Ok(event) = self.event.try_recv() {
            match event {
                MediaControlEvent::Toggle => {
                    if self.sink.is_paused() {
                        self.sink.play();
                    } else {
                        self.sink.pause();
                    }
                }
                _ => {}
            }
        }
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after_secs(1.0);

        egui::TopBottomPanel::top("status")
            .min_height(64.0)
            .max_height(64.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    //ui.add(
                    //    egui::Image::new(Self::IMAGE_PLAY).fit_to_exact_size(Vec2::new(64.0, 64.0)),
                    //);

                    if self.current_artist.is_some() {
                        ui.vertical(|ui| {
                            ui.add_space(8.0);
                            ui.label(self.current_artist.as_ref().unwrap());
                            ui.label(self.current_album.as_ref().unwrap());
                            ui.label(self.current_song.as_ref().unwrap().0.clone());
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

                        if self.sink.is_paused() {
                            if ui
                                .add(egui::Button::opt_image_and_text(
                                    Some(
                                        egui::Image::new(Self::IMAGE_PLAY)
                                            .fit_to_exact_size(Vec2::new(32.0, 32.0)),
                                    ),
                                    None,
                                ))
                                .clicked()
                            {
                                self.sink.play();

                                self.table
                                    .get::<mlua::Function>("play")
                                    .unwrap()
                                    .call::<()>((
                                        self.current_artist.clone(),
                                        self.current_album.clone(),
                                        self.current_song.clone().unwrap().0,
                                    ))
                                    .unwrap();
                            }
                        } else {
                            if ui
                                .add(egui::Button::opt_image_and_text(
                                    Some(
                                        egui::Image::new(Self::IMAGE_PAUSE)
                                            .fit_to_exact_size(Vec2::new(32.0, 32.0)),
                                    ),
                                    None,
                                ))
                                .clicked()
                            {
                                self.sink.pause();

                                self.table
                                    .get::<mlua::Function>("pause")
                                    .unwrap()
                                    .call::<()>((
                                        self.current_artist.clone(),
                                        self.current_album.clone(),
                                        self.current_song.clone().unwrap(),
                                    ))
                                    .unwrap();
                            }
                        }

                        ui.add(egui::Button::opt_image_and_text(
                            Some(
                                egui::Image::new(Self::IMAGE_SKIP_B)
                                    .fit_to_exact_size(Vec2::new(32.0, 32.0)),
                            ),
                            None,
                        ))
                        .clicked();

                        let mut seek = self.sink.get_pos().as_secs();

                        if ui
                            .add(
                                Slider::new(
                                    &mut seek,
                                    0..=self.current_time.unwrap().try_into().unwrap(),
                                )
                                .trailing_fill(true)
                                .show_value(false),
                            )
                            .changed()
                        {
                            self.sink.try_seek(Duration::from_secs(seek)).unwrap();
                        }

                        if self.sink.len() == 0 {
                            let c_artist = self
                                .state
                                .library
                                .map_artist
                                .get(self.current_artist.as_ref().unwrap())
                                .unwrap();
                            let c_album = c_artist
                                .map_album
                                .get(self.current_album.as_ref().unwrap())
                                .unwrap();
                            if let Some(song) = c_album
                                .list_song
                                .get(self.current_song.as_ref().unwrap().1 + 1)
                            {
                                self.sink.stop();
                                let file = std::fs::File::open(&song.path).unwrap();
                                self.sink
                                    .append(rodio::Decoder::new(BufReader::new(file)).unwrap());
                            }
                        }

                        let time_a = self.sink.get_pos().as_secs() / 60;
                        let time_b = self.sink.get_pos().as_secs() % 60;

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

                        let song_time_a = self.current_time.unwrap() / 60;
                        let song_time_b = self.current_time.unwrap() % 60;

                        let song_time_a = {
                            if song_time_a < 10 {
                                format!("0{song_time_a}")
                            } else {
                                song_time_a.to_string()
                            }
                        };

                        let song_time_b = {
                            if song_time_b < 10 {
                                format!("0{song_time_b}")
                            } else {
                                song_time_b.to_string()
                            }
                        };

                        ui.label(format!(
                            "{}:{}/{}:{}",
                            time_a, time_b, song_time_a, song_time_b
                        ));
                    }
                });
            });

        egui::SidePanel::left("panel_0")
            .resizable(true)
            .show(ctx, |ui| {
                ui.add_space(6.0);
                ui.text_edit_singleline(&mut self.search_artist);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for i in self.state.library.map_artist.keys() {
                        if i.to_lowercase()
                            .contains(&self.search_artist.to_lowercase().trim())
                        {
                            ui.selectable_value(&mut self.select_artist, i.to_string(), i);
                        }
                    }
                });
            });

        egui::SidePanel::right("panel_1")
            .resizable(true)
            .show(ctx, |ui| {
                ui.add_space(6.0);
                ui.text_edit_singleline(&mut self.search_song);

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if let Some(artist) = self.state.library.map_artist.get(&self.select_artist)
                        {
                            if let Some(album) = artist.map_album.get(&self.select_album) {
                                for (i, song) in album.list_song.iter().enumerate() {
                                    if song
                                        .name
                                        .to_lowercase()
                                        .contains(&self.search_song.to_lowercase().trim())
                                    {
                                        if ui
                                            .selectable_value(&mut self.select_song, i, &song.name)
                                            .clicked()
                                        {
                                            self.sink.stop();
                                            let file = std::fs::File::open(&song.path).unwrap();
                                            self.sink.append(
                                                rodio::Decoder::new(BufReader::new(file)).unwrap(),
                                            );

                                            self.current_artist = Some(self.select_artist.clone());
                                            self.current_album = Some(self.select_album.clone());
                                            self.current_song = Some((song.name.clone(), i));
                                            self.current_time = Some(song.time);

                                            self.table
                                                .get::<mlua::Function>("play")
                                                .unwrap()
                                                .call::<()>((
                                                    self.current_artist.clone(),
                                                    self.current_album.clone(),
                                                    self.current_song.clone().unwrap().0,
                                                ))
                                                .unwrap();
                                        }
                                    }
                                }
                            }
                        }
                    });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.text_edit_singleline(&mut self.search_album);

            egui::ScrollArea::vertical().show(ui, |ui| {
                if let Some(artist) = self.state.library.map_artist.get(&self.select_artist) {
                    for i in artist.map_album.keys() {
                        if i.to_lowercase()
                            .contains(&self.search_album.to_lowercase().trim())
                        {
                            ui.selectable_value(&mut self.select_album, i.to_string(), i);
                        }
                    }
                }
            });
        });
    }
}
