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

use eframe::egui::TextureOptions;
use eframe::egui::{self, Slider, Vec2};
use mlua::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(tag = "kind")]
enum Widget {
    Label { name: String },
    Button { name: String, call: String },
}

#[derive(Default, PartialEq)]
pub enum Layout {
    #[default]
    Library,
    Setting,
}

impl Layout {
    const IMAGE_SKIP_A: eframe::egui::ImageSource<'_> = egui::include_image!("../data/skip_a.png");
    const IMAGE_SKIP_B: eframe::egui::ImageSource<'_> = egui::include_image!("../data/skip_b.png");
    const IMAGE_PLAY: eframe::egui::ImageSource<'_> = egui::include_image!("../data/play.png");
    const IMAGE_PAUSE: eframe::egui::ImageSource<'_> = egui::include_image!("../data/pause.png");
    const IMAGE_REPLAY: eframe::egui::ImageSource<'_> = egui::include_image!("../data/replay.png");
    const IMAGE_RANDOM: eframe::egui::ImageSource<'_> = egui::include_image!("../data/random.png");
    const IMAGE_VOLUME_A: eframe::egui::ImageSource<'_> =
        egui::include_image!("../data/volume_a.png");
    const IMAGE_VOLUME_B: eframe::egui::ImageSource<'_> =
        egui::include_image!("../data/volume_b.png");
    const IMAGE_VOLUME_C: eframe::egui::ImageSource<'_> =
        egui::include_image!("../data/volume_c.png");
    const IMAGE_VOLUME_D: eframe::egui::ImageSource<'_> =
        egui::include_image!("../data/volume_d.png");

    //================================================================

    pub fn draw(app: &mut App, context: &egui::Context) {
        Self::draw_panel_layout(app, context);

        match app.layout {
            Layout::Library => Self::draw_library(app, context),
            Layout::Setting => Self::draw_setting(app, context),
        }
    }

    //================================================================
    // utility.
    //================================================================

    fn draw_button_image(ui: &mut egui::Ui, image: egui::ImageSource, select: bool) -> bool {
        ui.add(
            egui::Button::image(egui::Image::new(image).fit_to_exact_size(Vec2::new(32.0, 32.0)))
                .selected(select),
        )
        .clicked()
    }

    // draw the top song status bar. hidden if no song is available.
    fn draw_panel_layout(app: &mut App, context: &egui::Context) {
        egui::TopBottomPanel::top("layout").show(context, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut app.layout, Layout::Library, "Library");
                ui.selectable_value(&mut app.layout, Layout::Setting, "Setting");
            });
        });
    }

    //================================================================
    // setting layout.
    //================================================================

    fn draw_setting(app: &mut App, context: &egui::Context) {
        egui::CentralPanel::default().show(context, |ui| {
            ui.label("Setting");

            for script in &app.script.script_list {
                ui.separator();

                if let Ok(layout) = script.get("layout") {
                    let layout: Vec<Widget> = app.script.lua.from_value(layout).unwrap();

                    for widget in layout {
                        match widget {
                            Widget::Label { name } => {
                                ui.label(&name);
                            }
                            Widget::Button { name, call } => {
                                if ui.button(&name).clicked() {
                                    let call: mlua::Function = script.get(call).unwrap();
                                    call.call::<()>(()).unwrap();
                                }
                            }
                        };
                    }
                }
            }
        });
    }

    //================================================================
    // library layout.
    //================================================================

    fn draw_library(app: &mut App, context: &egui::Context) {
        Self::draw_panel_song(app, context);
        Self::draw_panel_side_a(app, context);
        Self::draw_panel_center(app, context);
        Self::draw_panel_side_b(app, context);
    }

    // draw the top song status bar. hidden if no song is available.
    fn draw_panel_song(app: &mut App, context: &egui::Context) {
        if let Some(_) = &app.active_state {
            context.request_repaint_after_secs(1.0);

            egui::TopBottomPanel::top("status").show(context, |ui| {
                ui.horizontal(|ui| {
                    {
                        let (artist, album, song) = app.get_play_state();

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
                            ui.label(&artist.name);
                            ui.label(&album.name);
                            ui.label(&song.name);
                        });
                    }

                    ui.separator();

                    if Self::draw_button_image(ui, Self::IMAGE_SKIP_A, false) {
                        app.song_skip_a();
                    }

                    let image = if app.sink.is_paused() {
                        Self::IMAGE_PLAY
                    } else {
                        Self::IMAGE_PAUSE
                    };

                    if Self::draw_button_image(ui, image, false) {
                        app.song_toggle();
                    }

                    if Self::draw_button_image(ui, Self::IMAGE_SKIP_B, false) {
                        app.song_skip_b();
                    }

                    if Self::draw_button_image(ui, Self::IMAGE_REPLAY, app.replay) {
                        app.replay = !app.replay;
                    }

                    if Self::draw_button_image(ui, Self::IMAGE_RANDOM, app.random) {
                        app.random = !app.random;
                    }

                    //================================================================

                    ui.separator();

                    let (_, _, song) = app.get_play_state();

                    let play_time = Self::format_time(app.sink.get_pos().as_secs() as usize);
                    let song_time = Self::format_time(song.time);

                    ui.label(format!("{play_time}/{song_time}"));

                    let mut seek = app.sink.get_pos().as_secs();

                    if ui
                        .add(
                            Slider::new(&mut seek, 0..=song.time as u64)
                                .trailing_fill(true)
                                .show_value(false),
                        )
                        .changed()
                    {
                        app.song_seek(seek as i64, false);
                    }

                    //================================================================

                    ui.separator();

                    let image = match app.sink.volume() {
                        0.00 => Self::IMAGE_VOLUME_A,
                        0.00..0.33 => Self::IMAGE_VOLUME_B,
                        0.33..0.66 => Self::IMAGE_VOLUME_C,
                        _ => Self::IMAGE_VOLUME_D,
                    };

                    ui.add(egui::Image::new(image).fit_to_exact_size(Vec2::new(32.0, 32.0)));

                    let mut volume = app.sink.volume();

                    if ui
                        .add(
                            Slider::new(&mut volume, 0.0..=1.0)
                                .trailing_fill(true)
                                .show_value(false),
                        )
                        .changed()
                    {
                        app.song_set_volume(volume);
                    }
                });
            });
        }
    }

    // draw the L-most panel.
    fn draw_panel_side_a(app: &mut App, context: &egui::Context) {
        egui::SidePanel::left("panel_0")
            .resizable(true)
            .show(context, |ui| {
                ui.add_space(6.0);
                ui.text_edit_singleline(&mut app.search_state.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, artist) in app.state.library.list_artist.iter().enumerate() {
                        if ui
                            .selectable_value(&mut app.select_state.0, Some(i), &artist.name)
                            .clicked()
                        {
                            app.select_state.1 = None;
                        }
                    }
                });
            });
    }

    // draw the center panel.
    fn draw_panel_center(app: &mut App, context: &egui::Context) {
        egui::CentralPanel::default().show(context, |ui| {
            ui.text_edit_singleline(&mut app.search_state.1);

            egui::ScrollArea::vertical().show(ui, |ui| {
                if let Some(select_0) = &app.select_state.0 {
                    let artist = app
                        .state
                        .library
                        .list_artist
                        .get(*select_0)
                        .expect("draw_panel_center(): Incorrect unwrap.");

                    for (i, album) in artist.list_album.iter().enumerate() {
                        if album
                            .name
                            .to_lowercase()
                            .contains(&app.search_state.1.to_lowercase().trim())
                        {
                            if ui
                                .selectable_value(&mut app.select_state.1, Some(i), &album.name)
                                .clicked()
                            {
                                app.select_state.2 = None;
                            }
                        }
                    }
                }
            });
        });
    }

    // draw the R-most panel.
    fn draw_panel_side_b(app: &mut App, context: &egui::Context) {
        let mut click = false;

        egui::SidePanel::right("panel_1")
            .resizable(true)
            .show(context, |ui| {
                ui.add_space(6.0);
                ui.text_edit_singleline(&mut app.search_state.2);

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if let Some(select_1) = &app.select_state.1 {
                            let artist = app
                                .state
                                .library
                                .list_artist
                                .get(*app.select_state.0.as_ref().unwrap())
                                .expect("draw_panel_side_b(): Incorrect unwrap (artist).");
                            let album = artist
                                .list_album
                                .get(*select_1)
                                .expect("draw_panel_side_b(): Incorrect unwrap (album).");

                            for (i, song) in album.list_song.iter().enumerate() {
                                if song
                                    .name
                                    .to_lowercase()
                                    .contains(&app.search_state.2.to_lowercase().trim())
                                {
                                    if ui
                                        .selectable_value(
                                            &mut app.select_state.2,
                                            Some(i),
                                            &song.name,
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
            app.song_add();
        }
    }

    //================================================================

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
}
