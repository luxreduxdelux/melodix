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

use eframe::egui::{self, ImageSource, OpenUrl, Slider, Vec2};
use eframe::egui::{Color32, TextureOptions};
use egui_extras::{Column, TableBuilder};
use mlua::prelude::*;
use serde::Deserialize;

//================================================================

pub struct Window {
    pub layout: Layout,
    pub replay: bool,
    pub random: bool,
    pub filter: (Vec<usize>, Vec<usize>, Vec<usize>),
    pub search: (String, String, String),
    pub select: (
        (Option<usize>, Option<usize>),
        (Option<usize>, Option<usize>),
        (Option<usize>, Option<usize>),
    ),
    pub state: Option<(usize, usize, usize)>,
    pub queue: (Vec<(usize, usize, usize)>, usize),
}

#[derive(PartialEq)]
pub enum Layout {
    Welcome,
    Library,
    Queue,
    Setup,
    About,
}

impl Window {
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
    const IMAGE_LOGO: eframe::egui::ImageSource<'_> = egui::include_image!("../data/logo.png");

    //================================================================

    pub fn new(library: &Library) -> Self {
        Self {
            layout: if library.list_group.is_empty() {
                Layout::Welcome
            } else {
                Layout::Library
            },
            replay: false,
            random: false,
            filter: (
                (0..library.list_group.len()).collect(),
                Vec::default(),
                Vec::default(),
            ),
            search: (String::default(), String::default(), String::default()),
            select: ((None, None), (None, None), (None, None)),
            state: None,
            queue: (Vec::default(), 0),
        }
    }

    pub fn draw(app: &mut App, context: &egui::Context) -> anyhow::Result<()> {
        context.request_repaint_after_secs(1.0);

        if app.system.sink.empty()
            && let Some(active) = app.window.state
        {
            if app.window.replay {
                app.track_add(active, context)?;
            } else if let Some(track) = app.window.queue.0.get(app.window.queue.1 + 1) {
                app.window.queue.1 += 1;
                app.track_add(*track, context)?;
            }
        }

        match app.window.layout {
            Layout::Welcome => Self::draw_welcome(app, context),
            Layout::Library => Self::draw_library(app, context),
            Layout::Queue => Self::draw_queue(app, context),
            Layout::Setup => Self::draw_setup(app, context),
            Layout::About => Self::draw_about(app, context),
        }

        // remove later
        Ok(())
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

    // draw the top track status bar. hidden if no track is available.
    fn draw_panel_layout(app: &mut App, context: &egui::Context) {
        egui::TopBottomPanel::top("layout").show(context, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut app.window.layout, Layout::Library, "Library");
                ui.selectable_value(&mut app.window.layout, Layout::Queue, "Queue");
                ui.selectable_value(&mut app.window.layout, Layout::Setup, "Setup");
                ui.selectable_value(&mut app.window.layout, Layout::About, "About");
            });
        });
    }

    //================================================================
    // queue layout.
    //================================================================

    #[rustfmt::skip]
    fn draw_queue(app: &mut App, context: &egui::Context) {
        Self::draw_panel_layout(app, context);
        Self::draw_panel_status(app, context);

        egui::CentralPanel::default().show(context, |ui| {
            let table = TableBuilder::new(ui)
                .striped(true)
                .sense(egui::Sense::click())
                .column(Column::auto())
                .column(Column::remainder())
                .column(Column::remainder())
                .column(Column::remainder())
                .column(Column::auto())
                .header(16.0, |mut header| {
                    header.col(|ui| { ui.strong("Number"); });
                    header.col(|ui| { ui.strong("Group");  });
                    header.col(|ui| { ui.strong("Album");  });
                    header.col(|ui| { ui.strong("Track");  });
                    header.col(|ui| { ui.strong("Time");   });
                });

            table.body(|ui| {
                ui.rows(16.0, app.window.queue.0.len(), |mut row| {
                    let index = row.index();
                    let queue = app.window.queue.0.get(index).unwrap();
                    let group = app.library.list_group.get(queue.0).unwrap();
                    let album = group.list_album.get(queue.1).unwrap();
                    let track = album.list_track.get(queue.2).unwrap();

                    row.set_selected(index == app.window.queue.1);

                    row.col(|ui| { ui.add(egui::Label::new((index + 1).to_string()).selectable(false));                });
                    row.col(|ui| { ui.add(egui::Label::new(&group.name).selectable(false));                            });
                    row.col(|ui| { ui.add(egui::Label::new(&album.name).selectable(false));                            });
                    row.col(|ui| { ui.add(egui::Label::new(&track.name).selectable(false));                            });
                    row.col(|ui| { ui.add(egui::Label::new(Self::format_time(track.time as usize)).selectable(false)); });

                    if row.response().clicked() {
                        app.window.queue.1 = index;
                        let _ = app.track_add(*queue, context);
                    }
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
            ui.separator();

            ui.label("Made by luxreduxdelux.");
            ui.hyperlink_to("GitHub", "https://github.com/luxreduxdelux/melodix");

            ui.label("Additional help by:");
            ui.hyperlink_to("agus-balles", "https://github.com/agus-balles");
        });
    }

    //================================================================
    // welcome layout.
    //================================================================

    fn draw_welcome(app: &mut App, context: &egui::Context) {
        egui::CentralPanel::default().show(context, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Welcome to Melodix!");

                ui.add(
                    egui::Image::new(Self::IMAGE_LOGO).fit_to_exact_size(Vec2::new(128.0, 77.0)),
                );

                ui.separator();

                if ui.button("Select Library Folder").clicked() {
                    if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                        app.library = Library::scan(&folder.as_path().display().to_string());
                        app.window.layout = Layout::Library;
                    }
                }
            });
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
                    app.library = Library::scan(&folder.as_path().display().to_string());
                    app.window.layout = Layout::Library;
                }
            }
            if ui
                .add(
                    egui::Slider::new(&mut app.setting.window_scale, 1.0..=2.0)
                        .text("Window scale factor"),
                )
                .drag_stopped()
            {
                context.set_zoom_factor(app.setting.window_scale);
            };
            if ui
                .checkbox(&mut app.setting.window_theme, "Use alternate window theme")
                .clicked()
            {
                if app.setting.window_theme {
                    context.set_theme(egui::Theme::Light);
                } else {
                    context.set_theme(egui::Theme::Dark);
                }
            };
            ui.checkbox(&mut app.setting.window_style, "Use alternate window style");
            ui.checkbox(&mut app.setting.window_media, "Allow multi-media key usage");
            ui.checkbox(&mut app.setting.window_tray, "Show tray icon");
            ui.checkbox(&mut app.setting.window_push, "Show track notification");
            ui.checkbox(
                &mut app.setting.library_find,
                "Allow automatic library scan",
            );
            ui.checkbox(&mut app.setting.script_allow, "Allow Lua plug-in scripting");
            ui.checkbox(
                &mut app.setting.update_check,
                "Allow automatic update check",
            );

            if app.setting.script_allow {
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

                            for (key, value) in setting.iter() {
                                let table: mlua::Table = table.get(&**key).unwrap();

                                match value {
                                    SettingData::String {
                                        data,
                                        name,
                                        info,
                                        call,
                                    } => {
                                        let mut data: String = table.get("data").unwrap();
                                        let widget = ui.label(&*name).id;
                                        let widget =
                                            ui.text_edit_singleline(&mut data).labelled_by(widget);

                                        if widget.on_hover_text(&*info).changed() {
                                            table.set("data", data).unwrap();

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
                                        let mut data: f32 = table.get("data").unwrap();
                                        let widget = ui.add(
                                            egui::Slider::new(&mut data, bind.0..=bind.1)
                                                .text(&*name),
                                        );

                                        if widget.on_hover_text(&*info).drag_stopped() {
                                            table.set("data", data).unwrap();

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
                                        let mut data: bool = table.get("data").unwrap();
                                        let widget = ui.checkbox(&mut data, &*name);

                                        if widget.on_hover_text(&*info).clicked() {
                                            table.set("data", data).unwrap();

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
        Self::draw_panel_status(app, context);

        Self::draw_panel_track(app, context);
        Self::draw_panel_group(app, context);
        Self::draw_panel_album(app, context);
    }

    // draw the top track status bar. hidden if no track is available.
    fn draw_panel_status(app: &mut App, context: &egui::Context) {
        if let Some(active) = app.window.state {
            egui::TopBottomPanel::top("status").show(context, |ui| {
                egui::ScrollArea::horizontal().show(ui, |ui| {
                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        if Self::draw_button_image(
                            ui,
                            Self::IMAGE_SKIP_A,
                            false,
                            app.setting.window_theme,
                        ) {
                            app.track_skip_a(context);
                        }

                        let image = if app.system.sink.is_paused() {
                            Self::IMAGE_PLAY
                        } else {
                            Self::IMAGE_PAUSE
                        };

                        if Self::draw_button_image(ui, image, false, app.setting.window_theme) {
                            app.track_toggle();
                        }

                        if Self::draw_button_image(
                            ui,
                            Self::IMAGE_SKIP_B,
                            false,
                            app.setting.window_theme,
                        ) {
                            app.track_skip_b(context);
                        }

                        if Self::draw_button_image(
                            ui,
                            Self::IMAGE_REPLAY,
                            app.window.replay,
                            app.setting.window_theme,
                        ) {
                            app.window.replay = !app.window.replay;
                        }

                        if Self::draw_button_image(
                            ui,
                            Self::IMAGE_RANDOM,
                            app.window.random,
                            app.setting.window_theme,
                        ) {
                            app.window.random = !app.window.random;
                        }

                        let image = match app.system.sink.volume() {
                            0.00 => Self::IMAGE_VOLUME_A,
                            0.00..0.33 => Self::IMAGE_VOLUME_B,
                            0.33..0.66 => Self::IMAGE_VOLUME_C,
                            _ => Self::IMAGE_VOLUME_D,
                        };

                        ui.menu_image_button(image, |ui| {
                            ui.horizontal(|ui| {
                                let mut volume = app.system.sink.volume();

                                if ui
                                    .add(
                                        Slider::new(&mut volume, 0.0..=1.0)
                                            .trailing_fill(true)
                                            .show_value(false),
                                    )
                                    .changed()
                                {
                                    app.track_set_volume(volume);
                                }
                            });
                        });

                        //================================================================

                        ui.separator();

                        let (_, _, track) = app.get_play_state();

                        let play_time =
                            Self::format_time(app.system.sink.get_pos().as_secs() as usize);
                        let track_time = Self::format_time(track.time as usize);

                        ui.label(format!("{play_time}/{track_time}"));

                        let mut seek = app.system.sink.get_pos().as_secs();

                        if ui
                            .add(
                                Slider::new(&mut seek, 0..=track.time)
                                    .trailing_fill(true)
                                    .show_value(false),
                            )
                            .changed()
                        {
                            app.track_seek(seek as i64, false);
                        }

                        //================================================================

                        ui.separator();

                        let (group, album, track) = app.get_play_state();

                        if let Some(icon) = &album.icon {
                            let image = egui::Image::new(format!("file://{icon}"))
                                .texture_options(
                                    TextureOptions::default()
                                        .with_mipmap_mode(Some(egui::TextureFilter::Nearest)),
                                )
                                .fit_to_exact_size(Vec2::new(48.0, 48.0));

                            ui.add(image);
                        }

                        if let Some(icon) = &track.icon {
                            let path = format!("bytes://{}", track.name);

                            let image = match context.try_load_bytes(&path) {
                                Ok(_) => egui::Image::new(path),
                                Err(_) => egui::Image::from_bytes(path.clone(), icon.clone()),
                            };

                            ui.add(
                                image
                                    .texture_options(
                                        TextureOptions::default()
                                            .with_mipmap_mode(Some(egui::TextureFilter::Nearest)),
                                    )
                                    .fit_to_exact_size(Vec2::new(48.0, 48.0)),
                            );
                        }

                        ui.vertical(|ui| {
                            ui.add_space(-2.0);
                            ui.label(&group.name);
                            ui.label(&album.name);
                            ui.label(&track.name);
                        });
                    });
                })
            });
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

                if ui.text_edit_singleline(&mut app.window.search.0).changed() {
                    app.window.filter.0.clear();
                    app.window.filter.1.clear();
                    app.window.filter.2.clear();
                    app.window.select.0 = (None, None);
                    app.window.select.1 = (None, None);
                    app.window.select.2 = (None, None);

                    for (i, group) in app.library.list_group.iter().enumerate() {
                        if group
                            .name
                            .to_lowercase()
                            .trim()
                            .contains(app.window.search.0.to_lowercase().trim())
                        {
                            app.window.filter.0.push(i);
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
                    ui.rows(16.0, app.window.filter.0.len(), |mut row| {
                        let i = row.index();
                        if let Some(select) = app.window.select.0.1 {
                            row.set_selected(i == select);
                        }

                        let index = app.window.filter.0.get(i).unwrap();
                        let group = app.library.list_group.get(*index).unwrap();

                        row.col(|ui| {
                            ui.add(egui::Label::new(&group.name).selectable(false));
                        });

                        if row.response().clicked() {
                            app.window.select.0 = (Some(*index), Some(i));
                            app.window.select.1 = (None, None);
                            app.window.select.2 = (None, None);
                            app.window.filter.1 = (0..group.list_album.len()).collect();
                        }
                    });
                });

                if sort {
                    app.window.filter.0.reverse();
                }
            });
    }

    fn draw_panel_album(app: &mut App, context: &egui::Context) {
        let rect = context.available_rect();

        egui::SidePanel::right("panel_album")
            .resizable(false)
            .exact_width(rect.max.x / 2.0)
            .show(context, |ui| {
                if let Some(select) = app.window.select.0.0 {
                    let mut sort = false;
                    let mut click = None;

                    ui.add_space(6.0);

                    let group = app.library.list_group.get(select).unwrap();

                    if ui.text_edit_singleline(&mut app.window.search.1).changed() {
                        app.window.filter.1.clear();
                        app.window.filter.2.clear();
                        app.window.select.1 = (None, None);
                        app.window.select.2 = (None, None);

                        for (i, album) in group.list_album.iter().enumerate() {
                            if album
                                .name
                                .to_lowercase()
                                .trim()
                                .contains(&app.window.search.1.to_lowercase().trim())
                            {
                                app.window.filter.1.push(i);
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
                        ui.rows(16.0, app.window.filter.1.len(), |mut row| {
                            let i = row.index();
                            if let Some(select) = app.window.select.1.1 {
                                row.set_selected(i == select);
                            }

                            let index = app.window.filter.1.get(i).unwrap();
                            let album = group.list_album.get(*index).unwrap();

                            row.col(|ui| {
                                ui.add(egui::Label::new(&album.name).selectable(false));
                            });

                            if row.response().clicked() {
                                app.window.select.1 = (Some(*index), Some(i));
                                app.window.select.2 = (None, None);
                                app.window.filter.2 = (0..album.list_track.len()).collect();
                            }

                            if row.response().double_clicked() {
                                click = Some(app.window.filter.1.get(i).cloned().unwrap());
                            }
                        });
                    });

                    if sort {
                        app.window.filter.1.reverse();
                    }

                    if let Some(click) = click {
                        let i_group = app.window.select.0.0.unwrap();
                        let i_album = app.window.select.1.0.unwrap();
                        let album = group.list_album.get(click).unwrap();
                        app.window.select.2.0 = Some(0);
                        app.window.queue.0.clear();
                        app.window.queue.1 = 0;

                        for x in 0..album.list_track.len() {
                            app.window.queue.0.push((i_group, i_album, x));
                        }

                        app.track_add(
                            (
                                app.window.select.0.0.unwrap(),
                                app.window.select.1.0.unwrap(),
                                0,
                            ),
                            context,
                        );
                    }
                }
            });
    }

    fn draw_panel_track(app: &mut App, context: &egui::Context) {
        let rect = context.available_rect();

        let mut click = false;

        egui::TopBottomPanel::bottom("panel_track")
            .resizable(false)
            .exact_height(rect.max.y / 2.0)
            .show(context, |ui| {
                let mut right = false;

                if let Some(group) = app.window.select.0.0
                    && let Some(album) = app.window.select.1.0
                {
                    let group = app.library.list_group.get(group).unwrap();
                    let album = group.list_album.get(album).unwrap();

                    ui.add_space(6.0);

                    if ui.text_edit_singleline(&mut app.window.search.2).changed() {
                        app.window.filter.2.clear();
                        app.window.select.2 = (None, None);

                        for (i, track) in album.list_track.iter().enumerate() {
                            if track
                                .name
                                .to_lowercase()
                                .trim()
                                .contains(&app.window.search.2.to_lowercase().trim())
                            {
                                app.window.filter.2.push(i);
                            }
                        }
                    };

                    ui.separator();

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
                        ui.rows(16.0, app.window.filter.2.len(), |mut row| {
                            let i = row.index();
                            if let Some(select) = app.window.select.2.1 {
                                row.set_selected(i == select);
                            }

                            let index = app.window.filter.2.get(i).unwrap();
                            let track = album.list_track.get(*index).unwrap();

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
                                app.window.select.2 = (Some(*index), Some(i));
                                click = true;
                            }
                        });
                    });
                }
            });

        if click {
            app.window.queue.0.clear();
            app.window.queue.0.push((
                app.window.select.0.0.unwrap(),
                app.window.select.1.0.unwrap(),
                app.window.select.2.0.unwrap(),
            ));
            app.window.queue.1 = 0;
            app.track_add(
                (
                    app.window.select.0.0.unwrap(),
                    app.window.select.1.0.unwrap(),
                    app.window.select.2.0.unwrap(),
                ),
                context,
            );
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
