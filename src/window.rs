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
use crate::library::Library;
use crate::script::*;

//================================================================

use eframe::egui::{self, OpenUrl, Slider, Vec2};
use eframe::egui::{Color32, TextureOptions};
use egui_extras::{Column, TableBuilder};
use mlua::prelude::*;
use serde::Deserialize;

//================================================================

pub struct Window {
    pub layout: Layout,
    pub filter_group: Vec<usize>,
    pub filter_album: Vec<usize>,
    pub filter_track: Vec<usize>,
    pub search_state: (String, String, String),
    pub select_state: (Option<usize>, Option<usize>, Option<usize>),
    pub active_state: Option<(usize, usize, usize)>,
    pub track_queue: Vec<(usize, usize, usize)>,
    pub replay: bool,
    pub random: bool,
}

#[derive(PartialEq)]
pub enum Layout {
    Welcome,
    Library,
    Queue,
    Setup,
    About,
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

    pub fn new(default: bool) -> Self {
        if default {
            println!("welcome");
            Layout::Welcome
        } else {
            Layout::Library
        }
    }

    pub fn draw(app: &mut App, context: &egui::Context) {
        context.request_repaint_after_secs(1.0);

        if app.sink.empty() && app.active_state.is_some() {
            println!("{:?}", app.track_queue);

            if app.replay {
                app.song_add(context);
            } else {
                if !app.track_queue.is_empty() {
                    let pop = app.track_queue.remove(0);
                    app.select_state = (Some(pop.0), Some(pop.1), Some(pop.2));
                    app.song_add(context);
                }
            }
        }

        match app.layout {
            Layout::Welcome => Self::draw_welcome(app, context),
            Layout::Library => Self::draw_library(app, context),
            Layout::Queue => Self::draw_queue(app, context),
            Layout::Setup => Self::draw_setup(app, context),
            Layout::About => Self::draw_about(app, context),
        }
    }

    //================================================================
    // utility.
    //================================================================

    fn draw_button_image(
        ui: &mut egui::Ui,
        image: egui::ImageSource,
        select: bool,
        invert: bool,
    ) -> bool {
        ui.add(
            egui::Button::image(
                egui::Image::new(image)
                    .fit_to_exact_size(Vec2::new(32.0, 32.0))
                    .tint(if invert {
                        Color32::BLACK
                    } else {
                        Color32::WHITE
                    }),
            )
            .selected(select),
        )
        .clicked()
    }

    // draw the top song status bar. hidden if no song is available.
    fn draw_panel_layout(app: &mut App, context: &egui::Context) {
        egui::TopBottomPanel::top("layout").show(context, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut app.layout, Layout::Library, "Library");
                ui.selectable_value(&mut app.layout, Layout::Queue, "Queue");
                ui.selectable_value(&mut app.layout, Layout::Setup, "Setup");
                ui.selectable_value(&mut app.layout, Layout::About, "About");
            });
        });
    }

    //================================================================
    // queue layout.
    //================================================================

    fn draw_queue(app: &mut App, context: &egui::Context) {
        Self::draw_panel_layout(app, context);

        egui::CentralPanel::default().show(context, |ui| {
            let table = TableBuilder::new(ui)
                .striped(true)
                .sense(egui::Sense::click())
                .column(Column::auto())
                .column(Column::remainder())
                .column(Column::remainder())
                .column(Column::remainder())
                .header(16.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("Number");
                    });
                    header.col(|ui| {
                        ui.strong("Group");
                    });
                    header.col(|ui| {
                        ui.strong("Album");
                    });
                    header.col(|ui| {
                        ui.strong("Track");
                    });
                });

            table.body(|ui| {
                ui.rows(16.0, app.track_queue.len(), |mut row| {
                    let i = row.index();
                    let queue = app.track_queue.get(i).unwrap();
                    let group = app.state.library.list_artist.get(queue.0).unwrap();
                    let album = group.list_album.get(queue.1).unwrap();
                    let track = album.list_song.get(queue.2).unwrap();

                    row.col(|ui| {
                        ui.add(egui::Label::new((i + 1).to_string()).selectable(false));
                    });

                    row.col(|ui| {
                        ui.add(egui::Label::new(&group.name).selectable(false));
                    });

                    row.col(|ui| {
                        ui.add(egui::Label::new(&album.name).selectable(false));
                    });

                    row.col(|ui| {
                        ui.add(egui::Label::new(&track.name).selectable(false));
                    });
                })
            });
        });
    }

    //================================================================
    // about layout.
    //================================================================

    fn draw_about(app: &mut App, context: &egui::Context) {
        Self::draw_panel_layout(app, context);

        egui::CentralPanel::default().show(context, |ui| {
            ui.heading("Melodix (1.0.0)");
            ui.label("Made by luxreduxdelux.");
            ui.label("Additional help by:");
            ui.label("* agus-balles");
        });
    }

    //================================================================
    // welcome layout.
    //================================================================

    fn draw_welcome(app: &mut App, context: &egui::Context) {
        egui::CentralPanel::default().show(context, |ui| {
            ui.label("Welcome to Melodix!");
            ui.separator();
            if ui.button("Select Library Folder").clicked() {
                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                    app.state.library = Library::scan(&folder.as_path().display().to_string());
                    app.layout = Layout::Library;
                }
            }
        });
    }

    //================================================================
    // setting layout.
    //================================================================

    fn draw_setup(app: &mut App, context: &egui::Context) {
        Self::draw_panel_layout(app, context);

        egui::CentralPanel::default().show(context, |ui| {
            ui.heading("Melodix Configuration");
            if ui.button("Scan Folder").clicked() {
                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                    app.state.library = Library::scan(&folder.as_path().display().to_string());
                    app.layout = Layout::Library;
                }
            }
            if ui
                .add(
                    egui::Slider::new(&mut app.state.setting.window_scale, 1.0..=2.0)
                        .text("Window scale factor"),
                )
                .drag_stopped()
            {
                context.set_zoom_factor(app.state.setting.window_scale);
            };
            if ui
                .checkbox(
                    &mut app.state.setting.window_theme,
                    "Use alternate window theme",
                )
                .clicked()
            {
                if app.state.setting.window_theme {
                    context.set_theme(egui::Theme::Light);
                } else {
                    context.set_theme(egui::Theme::Dark);
                }
            };
            ui.checkbox(
                &mut app.state.setting.window_style,
                "Use alternate window style",
            );
            ui.checkbox(
                &mut app.state.setting.window_media,
                "Allow multi-media key usage",
            );
            ui.checkbox(&mut app.state.setting.window_tray, "Show tray icon");
            ui.checkbox(&mut app.state.setting.window_push, "Show song notification");
            ui.checkbox(
                &mut app.state.setting.library_find,
                "Allow automatic library scan",
            );
            ui.checkbox(
                &mut app.state.setting.script_allow,
                "Allow Lua plug-in scripting",
            );
            ui.checkbox(
                &mut app.state.setting.update_check,
                "Allow automatic update check",
            );

            if app.state.setting.script_allow {
                ui.separator();
                ui.heading("Lua Plug-In Configuration");

                for script in &mut app.script.script_list {
                    if let Some(setting) = &mut script.0.setting {
                        ui.collapsing(&script.0.name, |ui| {
                            ui.group(|ui| {
                                ui.label(format!("Info: {}", &script.0.info));
                                ui.label(format!("From: {}", &script.0.from));
                                ui.label(format!("Version: {}", &script.0.version));
                            });

                            let table: mlua::Table = script.1.get("setting").unwrap();

                            for (key, value) in setting.iter_mut() {
                                let table: mlua::Table = table.get(&**key).unwrap();

                                match value {
                                    /*
                                    Widget::Label { name } => {
                                        ui.label(name);
                                    }
                                    Widget::Button { name, call } => {
                                        if ui.button(name).clicked() {
                                            let call: mlua::Function =
                                                script.1.get(&**call).unwrap();
                                            call.call::<()>(()).unwrap();
                                        }
                                    }
                                    */
                                    SettingData::String {
                                        data,
                                        name,
                                        info,
                                        call,
                                    } => {
                                        let widget = ui.label(&*name).id;
                                        let widget =
                                            ui.text_edit_singleline(data).labelled_by(widget);

                                        if widget.on_hover_text(&*info).changed() {
                                            table.set("data", &**data).unwrap();

                                            if let Some(call) = call {
                                                let call: mlua::Function =
                                                    script.1.get(&**call).unwrap();
                                                call.call::<()>(&script.1).unwrap();
                                            }
                                        }
                                    }
                                    SettingData::Number {
                                        data,
                                        name,
                                        info,
                                        bind,
                                        call,
                                    } => {
                                        let widget = ui.add(
                                            egui::Slider::new(data, bind.0..=bind.1).text(&*name),
                                        );

                                        if widget.on_hover_text(&*info).drag_stopped() {
                                            table.set("data", *data).unwrap();

                                            if let Some(call) = call {
                                                let call: mlua::Function =
                                                    script.1.get(&**call).unwrap();
                                                call.call::<()>(&script.1).unwrap();
                                            }
                                        }
                                    }
                                    SettingData::Boolean {
                                        data,
                                        name,
                                        info,
                                        call,
                                    } => {
                                        let widget = ui.checkbox(data, &*name);

                                        if widget.on_hover_text(&*info).clicked() {
                                            table.set("data", *data).unwrap();

                                            if let Some(call) = call {
                                                let call: mlua::Function =
                                                    script.1.get(&**call).unwrap();
                                                call.call::<()>(&script.1).unwrap();
                                            }
                                        }
                                    }
                                };
                            }
                        });
                    }
                }
            }
        });
    }

    //================================================================
    // library layout.
    //================================================================

    fn draw_library(app: &mut App, context: &egui::Context) {
        Self::draw_panel_layout(app, context);

        Self::draw_panel_song(app, context);
        Self::draw_panel_track(app, context);
        Self::draw_panel_group(app, context);
        Self::draw_panel_album(app, context);
    }

    // draw the top song status bar. hidden if no song is available.
    fn draw_panel_song(app: &mut App, context: &egui::Context) {
        if let Some(active) = app.active_state {
            egui::TopBottomPanel::top("status").show(context, |ui| {
                egui::ScrollArea::horizontal().show(ui, |ui| {
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

                        if Self::draw_button_image(
                            ui,
                            Self::IMAGE_SKIP_A,
                            false,
                            app.state.setting.window_theme,
                        ) {
                            app.song_skip_a();
                        }

                        let image = if app.sink.is_paused() {
                            Self::IMAGE_PLAY
                        } else {
                            Self::IMAGE_PAUSE
                        };

                        if Self::draw_button_image(ui, image, false, app.state.setting.window_theme)
                        {
                            app.song_toggle();
                        }

                        if Self::draw_button_image(
                            ui,
                            Self::IMAGE_SKIP_B,
                            false,
                            app.state.setting.window_theme,
                        ) {
                            app.song_skip_b();
                        }

                        if Self::draw_button_image(
                            ui,
                            Self::IMAGE_REPLAY,
                            app.replay,
                            app.state.setting.window_theme,
                        ) {
                            app.replay = !app.replay;
                        }

                        if Self::draw_button_image(
                            ui,
                            Self::IMAGE_RANDOM,
                            app.random,
                            app.state.setting.window_theme,
                        ) {
                            app.random = !app.random;
                        }

                        //================================================================

                        ui.separator();

                        let (_, _, song) = app.get_play_state();

                        let play_time = Self::format_time(app.sink.get_pos().as_secs() as usize);
                        let song_time = Self::format_time(song.time as usize);

                        ui.label(format!("{play_time}/{song_time}"));

                        let mut seek = app.sink.get_pos().as_secs();

                        if ui
                            .add(
                                Slider::new(&mut seek, 0..=song.time)
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
                })
            });
        }
    }

    fn draw_panel_track(app: &mut App, context: &egui::Context) {
        let rect = context.available_rect();

        let mut click = false;

        egui::TopBottomPanel::bottom("panel_track")
            .resizable(false)
            .exact_height(rect.max.y / 2.0)
            .show(context, |ui| {
                ui.add_space(6.0);
                ui.text_edit_singleline(&mut app.search_state.2);
                ui.separator();

                if let Some(group) = app.select_state.0
                    && let Some(album) = app.select_state.1
                {
                    let group = app.state.library.list_artist.get(group).unwrap();
                    let album = group.list_album.get(album).unwrap();

                    let table = TableBuilder::new(ui)
                        .striped(true)
                        .sense(egui::Sense::click())
                        .column(Column::auto())
                        .column(Column::remainder())
                        .column(Column::remainder())
                        .column(Column::remainder())
                        .column(Column::auto())
                        .header(16.0, |mut header| {
                            header.col(|ui| {
                                ui.strong("Track");
                            });
                            header.col(|ui| {
                                ui.strong("Title");
                            });
                            header.col(|ui| {
                                ui.strong("Genre");
                            });
                            header.col(|ui| {
                                ui.strong("Date");
                            });
                            header.col(|ui| {
                                ui.strong("Time");
                            });
                        });

                    table.body(|ui| {
                        ui.rows(16.0, album.list_song.len(), |mut row| {
                            let i = row.index();
                            if let Some(select) = app.select_state.2 {
                                row.set_selected(i == select);
                            }
                            let track = album.list_song.get(i).unwrap();

                            row.col(|ui| {
                                let order = {
                                    if let Some(order) = track.track {
                                        order.to_string()
                                    } else {
                                        "".to_string()
                                    }
                                };
                                ui.add(egui::Label::new(&order).selectable(false));
                            });

                            row.col(|ui| {
                                ui.add(egui::Label::new(&track.name).selectable(false));
                            });

                            row.col(|ui| {
                                ui.add(
                                    egui::Label::new(if let Some(kind) = &track.kind {
                                        kind.as_str()
                                    } else {
                                        ""
                                    })
                                    .selectable(false),
                                );
                            });

                            row.col(|ui| {
                                ui.add(
                                    egui::Label::new(if let Some(date) = &track.date {
                                        date.as_str()
                                    } else {
                                        ""
                                    })
                                    .selectable(false),
                                );
                            });

                            row.col(|ui| {
                                ui.add(
                                    egui::Label::new(Self::format_time(track.time as usize))
                                        .selectable(false),
                                );
                            });

                            if row.response().clicked() {
                                app.select_state.2 = Some(i);
                                click = true;
                            }
                        });
                    });
                }
            });

        if click {
            app.track_queue.clear();
            app.song_add(context);
        }
    }

    fn draw_panel_group(app: &mut App, context: &egui::Context) {
        let rect = context.available_rect();

        egui::SidePanel::left("panel_group")
            .resizable(false)
            .exact_width(rect.max.x / 2.0)
            .show(context, |ui| {
                let mut sort = false;

                ui.add_space(6.0);

                if ui.text_edit_singleline(&mut app.search_state.0).changed() {
                    app.state.filter_group.clear();
                    app.state.filter_album.clear();
                    app.state.filter_track.clear();

                    for (i, group) in app.state.library.list_artist.iter().enumerate() {
                        if group
                            .name
                            .to_lowercase()
                            .trim()
                            .contains(&app.search_state.0.to_lowercase().trim())
                        {
                            app.state.filter_group.push(i);
                        }
                    }
                };

                ui.separator();

                let table = TableBuilder::new(ui)
                    .striped(true)
                    .sense(egui::Sense::click())
                    .column(Column::remainder())
                    .header(16.0, |mut header| {
                        header.col(|ui| {
                            ui.horizontal(|ui| {
                                ui.strong("Group");
                                if ui.button("⬆/⬇").clicked() {
                                    sort = true;
                                }
                            });
                        });
                    });

                table.body(|ui| {
                    ui.rows(16.0, app.state.filter_group.len(), |mut row| {
                        let i = row.index();
                        if let Some(select) = app.select_state.0 {
                            row.set_selected(i == select);
                        }

                        let group = app.state.filter_group.get(i).unwrap();
                        let group = app.state.library.list_artist.get(*group).unwrap();

                        row.col(|ui| {
                            ui.add(egui::Label::new(&group.name).selectable(false));
                        });

                        if row.response().clicked() {
                            println!("foo");
                            app.select_state.0 = Some(i);
                            app.state.filter_album = (0..group.list_album.len()).collect();
                            app.select_state.1 = None;
                        }
                    });
                });

                if sort {
                    app.state.filter_group.reverse();
                }
            });
    }

    fn draw_panel_album(app: &mut App, context: &egui::Context) {
        let rect = context.available_rect();

        egui::SidePanel::right("panel_album")
            .resizable(false)
            .exact_width(rect.max.x / 2.0)
            .show(context, |ui| {
                if let Some(select) = app.select_state.0 {
                    let mut sort = false;
                    let mut click = None;

                    ui.add_space(6.0);

                    let artist = app.state.library.list_artist.get(select).unwrap();

                    if ui.text_edit_singleline(&mut app.search_state.1).changed() {
                        app.state.filter_album.clear();

                        for (i, album) in artist.list_album.iter().enumerate() {
                            if album
                                .name
                                .to_lowercase()
                                .trim()
                                .contains(&app.search_state.1.to_lowercase().trim())
                            {
                                app.state.filter_album.push(i);
                            }
                        }
                    };

                    ui.separator();

                    let table = TableBuilder::new(ui)
                        .striped(true)
                        .sense(egui::Sense::click())
                        .column(Column::remainder())
                        .header(16.0, |mut header| {
                            header.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.strong("Album");
                                    if ui.button("⬆/⬇").clicked() {
                                        sort = true;
                                    }
                                });
                            });
                        });

                    table.body(|ui| {
                        ui.rows(16.0, app.state.filter_album.len(), |mut row| {
                            let i = row.index();
                            if let Some(select) = app.select_state.1 {
                                row.set_selected(i == select);
                            }

                            let album = app.state.filter_album.get(i).unwrap();
                            let album = artist.list_album.get(*album).unwrap();

                            row.col(|ui| {
                                ui.add(egui::Label::new(&album.name).selectable(false));
                            });

                            if row.response().clicked() {
                                app.select_state.1 = Some(i);
                                app.state.filter_track = (0..album.list_song.len()).collect();
                                app.select_state.2 = None;
                            }

                            if row.response().double_clicked() {
                                click = Some(app.state.filter_album.get(i).cloned().unwrap());
                            }
                        });
                    });

                    if sort {
                        app.state.filter_album.reverse();
                    }

                    if let Some(click) = click {
                        let i_group = app.select_state.0.unwrap();
                        let i_album = app.select_state.1.unwrap();
                        let album = artist.list_album.get(click).unwrap();
                        app.select_state.2 = Some(0);
                        app.track_queue.clear();

                        for x in 1..album.list_song.len() {
                            app.track_queue.push((i_group, i_album, x));
                        }

                        println!("{:?}", app.track_queue);

                        app.song_add(context);
                    }
                }
            });
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
