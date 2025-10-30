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

use crate::{app::*, library::*, script::*, system::*};

//================================================================

use eframe::egui::{self, Color32, Popup, Slider, TextureOptions, Vec2};
use egui_extras::{Column, TableBuilder};
use egui_toast::Toasts;
use rand::seq::{IndexedRandom, SliceRandom};

//================================================================

pub struct Window {
    /// currently active layout (library, queue, etc.)
    pub layout: Layout,
    /// repeat track.
    pub repeat: bool,
    /// randomize queue.
    pub random: bool,
    /// search state, for group, album, track.
    pub search: (String, String, String),
    /// select state, for group, album, track.
    /// index .0 is for the group/album/track index.
    /// index .1 is for the layout index (for adding a highlight to an entry in the window).
    pub select: (
        (Option<usize>, Option<usize>),
        (Option<usize>, Option<usize>),
        (Option<usize>, Option<usize>),
    ),
    /// play state, for group, album, track.
    pub state: Option<(usize, usize, usize)>,
    /// queue state, for group, album, track, and queue index.
    pub queue: (Vec<(usize, usize, usize)>, usize),
    /// toast notification list.
    pub toast: Toasts,
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
    const IMAGE_SKIP_A: eframe::egui::ImageSource<'_> = egui::include_image!("../data/skip_a.svg");
    const IMAGE_SKIP_B: eframe::egui::ImageSource<'_> = egui::include_image!("../data/skip_b.svg");
    const IMAGE_PLAY: eframe::egui::ImageSource<'_> = egui::include_image!("../data/play.svg");
    const IMAGE_PAUSE: eframe::egui::ImageSource<'_> = egui::include_image!("../data/pause.svg");
    const IMAGE_REPEAT: eframe::egui::ImageSource<'_> = egui::include_image!("../data/repeat.svg");
    const IMAGE_RANDOM: eframe::egui::ImageSource<'_> = egui::include_image!("../data/random.svg");
    const IMAGE_VOLUME_A: eframe::egui::ImageSource<'_> =
        egui::include_image!("../data/volume_a.svg");
    const IMAGE_VOLUME_B: eframe::egui::ImageSource<'_> =
        egui::include_image!("../data/volume_b.svg");
    const IMAGE_VOLUME_C: eframe::egui::ImageSource<'_> =
        egui::include_image!("../data/volume_c.svg");
    const IMAGE_VOLUME_D: eframe::egui::ImageSource<'_> =
        egui::include_image!("../data/volume_d.svg");
    const IMAGE_LOGO: eframe::egui::ImageSource<'_> = egui::include_image!("../data/logo.png");

    //================================================================

    pub fn new(library: &Library) -> Self {
        use egui::Align2;

        Self {
            layout: if library.list_group.is_empty() {
                Layout::Welcome
            } else {
                Layout::Library
            },
            repeat: false,
            random: false,
            search: (String::default(), String::default(), String::default()),
            select: ((None, None), (None, None), (None, None)),
            state: None,
            queue: (Vec::default(), 0),
            toast: Toasts::new()
                .anchor(Align2::RIGHT_BOTTOM, (-8.0, -8.0))
                .direction(egui::Direction::BottomUp),
        }
    }

    pub fn draw(app: &mut App, context: &egui::Context) -> anyhow::Result<()> {
        Self::handle_close(app, context);
        Self::handle_track(app, context)?;

        app.window.toast.show(context);

        match app.window.layout {
            Layout::Welcome => Self::draw_welcome(app, context),
            Layout::Library => Self::draw_library(app, context),
            Layout::Queue => Self::draw_queue(app, context),
            Layout::Setup => Self::draw_setup(app, context),
            Layout::About => Self::draw_about(app, context),
        }

        Ok(())
    }

    //================================================================
    // utility.
    //================================================================

    fn handle_close(app: &mut App, context: &egui::Context) {
        let mut close = false;

        context.viewport(|state| {
            if state.input.viewport().close_requested() && !app.system.close {
                close = true;
            }
        });

        if close {
            System::toggle_visible(app, context);
            context.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }
    }

    fn handle_track(app: &mut App, context: &egui::Context) -> anyhow::Result<()> {
        if app.system.sink.empty()
            && let Some(active) = app.window.state
        {
            if app.window.repeat {
                app.track_add(active, context)?;
            } else if app.window.random {
                if app.window.queue.0.len() > 1 {
                    let mut random: Vec<usize> = (0..app.window.queue.0.len()).collect();
                    let mut picker = rand::rng();
                    random.shuffle(&mut picker);

                    let track = random.choose(&mut picker).unwrap();

                    app.window.queue.1 = *track;
                    app.track_add(*app.window.queue.0.get(*track).unwrap(), context)?;
                }
            } else if let Some(track) = app.window.queue.0.get(app.window.queue.1 + 1) {
                app.window.queue.1 += 1;
                app.track_add(*track, context)?;
            }
        }

        Ok(())
    }

    fn draw_button_image(
        ui: &mut egui::Ui,
        image: egui::ImageSource,
        select: bool,
        invert: bool,
    ) -> egui::Response {
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
            ui.horizontal(|ui| {
                if ui.button("Save Queue").clicked() {
                    if let Some(file) = rfd::FileDialog::new().set_file_name("queue.m3u").add_filter("m3u", &["m3u"]).save_file() {
                        let mut file = std::fs::File::create(file).unwrap();
                        let mut writer = m3u::Writer::new(&mut file);

                        for entry in &app.window.queue.0 {
                            let (_, _, track) = app.get_state(*entry);
                            writer.write_entry(&m3u::Entry::Path(track.path.clone().into())).unwrap();
                        }
                    }
                }
                if ui.button("Load Queue").clicked() {
                    if let Some(file) = rfd::FileDialog::new().add_filter("m3u", &["m3u"]).pick_file() {
                        let mut reader = m3u::Reader::open(file).unwrap();
                        let read_playlist: Vec<_> = reader.entries().map(|entry| entry.unwrap()).collect();

                        Self::queue_reset(app);

                        for entry in read_playlist {
                            match entry {
                                m3u::Entry::Path(path) => {
                                    let mut play = Vec::new();

                                    for (i_group, group) in app.library.list_group.iter().enumerate() {
                                        for (i_album, album) in group.list_album.iter().enumerate() {
                                            for (i_track, track) in album.list_track.iter().enumerate() {
                                                if track.path == path.display().to_string() {
                                                    play.push((i_group, i_album, i_track));
                                                }
                                            }
                                        }
                                    }

                                    for (i_g, i_a, i_t) in play {
                                        app.window.queue.0.push((i_g, i_a, i_t));
                                        app.window.queue.1 = 0;
                                    }

                                    if let Some(first) = app.window.queue.0.first() {
                                        App::error_result(app.track_add(*first, context));
                                    }
                                },
                                _ => {}
                            }
                        }
                    }
                }
            });

            ui.separator();

            let table = TableBuilder::new(ui)
                .striped(true)
                .sense(egui::Sense::click())
                .column(Column::auto().resizable(true))
                .column(Column::remainder().resizable(true).clip(true))
                .column(Column::remainder().resizable(true).clip(true))
                .column(Column::remainder().resizable(true).clip(true))
                .column(Column::remainder().resizable(true).clip(true))
                .header(16.0, |mut header| {
                    header.col(|ui| { ui.strong("Number"); });
                    header.col(|ui| { ui.strong("Group");  });
                    header.col(|ui| { ui.strong("Album");  });
                    header.col(|ui| { ui.strong("Track");  });
                    header.col(|ui| { ui.strong("Time");   });
                });

            let mut detach = None;

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
                    row.col(|ui| { ui.add(egui::Label::new(Self::format_time(track.time.as_secs() as usize)).selectable(false)); });

                    row.response().context_menu(|ui| {
                        if ui.button("Remove from queue").clicked() {
                            detach = Some((index, index == app.window.queue.1));
                            ui.close();
                        }

                        for script in &mut app.script.script_list {
                            if let Some(s_queue) = &mut script.0.queue {
                                ui.collapsing(&script.0.name, |ui| {
                                    let table: mlua::Table = script.1.get("queue").unwrap();

                                    for (key, value) in s_queue.iter() {
                                        value.draw(&script.1, &table.get(&**key).unwrap(), ui, (queue.0, queue.1, queue.2));
                                    }
                                });
                            }
                        }
                    });

                    if row.response().clicked() {
                        app.window.queue.1 = index;
                        let _ = app.track_add(*queue, context);
                    }
                })
            });

            if let Some(detach) = detach {
                if detach.1 {
                    App::error_result(app.track_skip_b(context));
                }

                // TO-DO handle queue management in a better way...
                app.window.queue.0.remove(detach.0);
            }
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

    #[rustfmt::skip]
    fn draw_setup(app: &mut App, context: &egui::Context) {
        Self::draw_panel_layout(app, context);

        egui::CentralPanel::default().show(context, |ui| {
            ui.collapsing("General", |ui| {
                if ui.button("Scan Folder").clicked() && let Some(folder) = rfd::FileDialog::new().pick_folder() {
                    app.library = Library::scan(&folder.as_path().display().to_string());
                    app.window.layout = Layout::Library;
                }

                ui.checkbox(&mut app.setting.window_media, "Allow automatic update check").on_hover_text("Will take effect on restart.");
                ui.checkbox(&mut app.setting.window_media, "Allow multi-media key usage").on_hover_text("Will take effect on restart.");
                ui.checkbox(&mut app.setting.window_tray,  "Show tray icon").on_hover_text("Will take effect on restart.");
                ui.checkbox(&mut app.setting.window_push,  "Show track notification").on_hover_text("Will take effect on restart.");
            });

            //================================================================

            ui.collapsing("Window", |ui| {
                if ui.add(egui::Slider::new(&mut app.setting.window_scale, 1.0..=2.0).text("Scale factor")).changed() {
                    context.set_zoom_factor(app.setting.window_scale);
                };

                if ui.checkbox(&mut app.setting.window_theme, "Use alternate theme").clicked() {
                    if app.setting.window_theme {
                        context.set_theme(egui::Theme::Light);
                    } else {
                        context.set_theme(egui::Theme::Dark);
                    }
                };

                ui.checkbox(&mut app.setting.window_time,  "Show track duration");
                ui.checkbox(&mut app.setting.window_date,  "Show track date");
                ui.checkbox(&mut app.setting.window_kind,  "Show track kind");
                ui.checkbox(&mut app.setting.window_track, "Show track number");
            });

            //================================================================

            ui.collapsing("Script", |ui| {
                if ui.button("Open Folder").clicked() {
                    let _ = opener::open(Script::get_path());
                }

                if ui.button("Save Sample Plug-In").clicked() {
                    let path = Script::get_path();

                    let main = std::fs::write(format!("{path}/main.lua"), Script::DATA_MAIN);
                    let meta = std::fs::write(format!("{path}/meta.lua"), Script::DATA_META);

                    if main.is_ok() && meta.is_ok() {
                        let _ = opener::open(path);
                    } else {
                        rfd::MessageDialog::new()
                            .set_level(rfd::MessageLevel::Error)
                            .set_title("Lua Plug-In")
                            .set_description("Could not write the sample Lua plug-in.")
                            .show();
                    }
                }

                ui.checkbox(&mut app.setting.script_allow, "Allow Lua plug-in scripting").on_hover_text("Will take effect on restart.");

                ui.add_enabled_ui(app.setting.script_allow, |ui| {
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
                                    App::error_result(value.draw(&script.1, &table, ui, ()));
                                }
                            });
                        }
                    }
                });
            });
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
        if app.window.state.is_some() {
            egui::TopBottomPanel::top("status").show(context, |ui| {
                egui::ScrollArea::horizontal().show(ui, |ui| {
                    ui.add_space(6.0);

                    ui.horizontal(|ui| {
                        if Self::draw_button_image(
                            ui,
                            Self::IMAGE_SKIP_A,
                            false,
                            app.setting.window_theme,
                        )
                        .clicked()
                        {
                            App::error_result(app.track_skip_a(context));
                        }

                        let image = if app.system.sink.is_paused() {
                            Self::IMAGE_PLAY
                        } else {
                            Self::IMAGE_PAUSE
                        };

                        if Self::draw_button_image(ui, image, false, app.setting.window_theme)
                            .clicked()
                        {
                            app.track_toggle();
                        }

                        if Self::draw_button_image(
                            ui,
                            Self::IMAGE_SKIP_B,
                            false,
                            app.setting.window_theme,
                        )
                        .clicked()
                        {
                            App::error_result(app.track_skip_b(context));
                        }

                        if Self::draw_button_image(
                            ui,
                            Self::IMAGE_REPEAT,
                            app.window.repeat,
                            app.setting.window_theme,
                        )
                        .clicked()
                        {
                            app.window.repeat = !app.window.repeat;
                        }

                        if Self::draw_button_image(
                            ui,
                            Self::IMAGE_RANDOM,
                            app.window.random,
                            app.setting.window_theme,
                        )
                        .clicked()
                        {
                            app.window.random = !app.window.random;
                        }

                        let image = match app.system.sink.volume() {
                            0.00 => Self::IMAGE_VOLUME_A,
                            0.00..0.33 => Self::IMAGE_VOLUME_B,
                            0.33..0.66 => Self::IMAGE_VOLUME_C,
                            _ => Self::IMAGE_VOLUME_D,
                        };

                        let response =
                            Self::draw_button_image(ui, image, false, app.setting.window_theme);

                        Popup::menu(&response).show(|ui| {
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

                        let (_, _, track) = app.get_play_state().unwrap();

                        let play_time =
                            Self::format_time(app.system.sink.get_pos().as_secs() as usize);
                        let track_time = Self::format_time(track.time.as_secs() as usize);

                        ui.label(format!("{play_time}/{track_time}"));

                        let mut seek = app.system.sink.get_pos().as_secs();

                        if ui
                            .add(
                                Slider::new(&mut seek, 0..=track.time.as_secs())
                                    .trailing_fill(true)
                                    .show_value(false),
                            )
                            .changed()
                        {
                            app.track_seek(seek as i64, false);
                        }

                        //================================================================

                        ui.separator();

                        let (group, album, track) = app.get_play_state().unwrap();

                        if let Some(icon) = &track.icon.0 {
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
                        } else if let Some(icon) = &album.icon {
                            let image = egui::Image::new(format!("file://{icon}"))
                                .texture_options(
                                    TextureOptions::default()
                                        .with_mipmap_mode(Some(egui::TextureFilter::Nearest)),
                                )
                                .fit_to_exact_size(Vec2::new(48.0, 48.0));

                            ui.add(image);
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

    fn queue_reset(app: &mut App) {
        app.window.queue.0.clear();
        app.window.queue.1 = 0;
        app.track_stop(false);
    }

    fn queue_play_group(
        app: &mut App,
        i_group: usize,
        context: &egui::Context,
    ) -> anyhow::Result<()> {
        Self::queue_reset(app);
        let group = app.library.list_group.get(i_group).unwrap();

        for (i_album, album) in group.list_album.iter().enumerate() {
            for (i_track, _) in album.list_track.iter().enumerate() {
                app.window.queue.0.push((i_group, i_album, i_track));
            }
        }

        app.track_add((i_group, 0, 0), context)
    }

    fn queue_play_album(
        app: &mut App,
        group: usize,
        album: usize,
        context: &egui::Context,
    ) -> anyhow::Result<()> {
        Self::queue_reset(app);
        let i_group = app.library.list_group.get(group).unwrap();
        let i_album = i_group.list_album.get(album).unwrap();

        for x in 0..i_album.list_track.len() {
            app.window.queue.0.push((group, album, x));
        }

        app.track_add((group, album, 0), context)
    }

    fn queue_play_track(
        app: &mut App,
        group: usize,
        album: usize,
        track: usize,
        context: &egui::Context,
    ) -> anyhow::Result<()> {
        Self::queue_reset(app);
        let i_group = app.library.list_group.get(group).unwrap();
        let i_album = i_group.list_album.get(album).unwrap();

        for x in track..i_album.list_track.len() {
            app.window.queue.0.push((group, album, x));
        }

        app.track_add((group, album, track), context)
    }

    #[rustfmt::skip]
    fn draw_panel_group(app: &mut App, context: &egui::Context) {
        let rect = context.available_rect();

        egui::SidePanel::left("panel_group")
            .resizable(false)
            .exact_width(rect.max.x / 2.0)
            .show(context, |ui| {
                let mut sort = false;
                let mut click = None;

                ui.add_space(6.0);

                if ui.text_edit_singleline(&mut app.window.search.0).changed() {
                    app.library.list_shown.0.clear();
                    app.library.list_shown.1.clear();
                    app.library.list_shown.2.clear();
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
                            app.library.list_shown.0.push(i);
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
                                ui.strong(format!("Group ({})", app.library.list_shown.0.len()));
                                if ui.button("⬆/⬇").clicked() {
                                    sort = true;
                                }
                            });
                        });
                    });

                table.body(|ui| {
                    ui.rows(16.0, app.library.list_shown.0.len(), |mut row| {
                        let i = row.index();
                        if let Some(select) = app.window.select.0.1 {
                            row.set_selected(i == select);
                        }

                        let index = app.library.list_shown.0.get(i).unwrap();
                        let group = app.library.list_group.get(*index).unwrap();

                        row.col(|ui| { ui.add(egui::Label::new(&group.name).selectable(false)); });

                        row.response().context_menu(|ui| {
                            for script in &mut app.script.script_list {
                                if let Some(s_group) = &mut script.0.group {
                                    ui.collapsing(&script.0.name, |ui| {
                                        let table: mlua::Table = script.1.get("group").unwrap();

                                        for (key, value) in s_group.iter() {
                                            value.draw(&script.1, &table.get(&**key).unwrap(), ui, (*index,));
                                        }
                                    });
                                }
                            }
                        });

                        if row.response().clicked() {
                            app.window.select.0 = (Some(*index), Some(i));
                            app.window.select.1 = (None, None);
                            app.window.select.2 = (None, None);
                            app.library.list_shown.1 = (0..group.list_album.len()).collect();
                        }

                        if row.response().double_clicked() {
                            click = Some((
                                app.window.select.0.0.unwrap(),
                            ));
                        }
                    });
                });

                if sort {
                    app.library.list_shown.0.reverse();
                }

                if let Some(click) = click {
                    App::error_result(Self::queue_play_group(app, click.0, context));
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
                        app.library.list_shown.1.clear();
                        app.library.list_shown.2.clear();
                        app.window.select.1 = (None, None);
                        app.window.select.2 = (None, None);

                        for (i, album) in group.list_album.iter().enumerate() {
                            if album
                                .name
                                .to_lowercase()
                                .trim()
                                .contains(app.window.search.1.to_lowercase().trim())
                            {
                                app.library.list_shown.1.push(i);
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
                                    ui.strong(format!(
                                        "Album ({})",
                                        app.library.list_shown.1.len()
                                    ));
                                    if ui.button("⬆/⬇").clicked() {
                                        sort = true;
                                    }
                                });
                            });
                        });

                    table.body(|ui| {
                        ui.rows(16.0, app.library.list_shown.1.len(), |mut row| {
                            let i = row.index();
                            if let Some(select) = app.window.select.1.1 {
                                row.set_selected(i == select);
                            }

                            let index = app.library.list_shown.1.get(i).unwrap();
                            let album = group.list_album.get(*index).unwrap();

                            row.col(|ui| {
                                ui.add(egui::Label::new(&album.name).selectable(false));
                            });

                            row.response().context_menu(|ui| {
                                for script in &mut app.script.script_list {
                                    if let Some(s_album) = &mut script.0.album {
                                        ui.collapsing(&script.0.name, |ui| {
                                            let table: mlua::Table = script.1.get("album").unwrap();

                                            for (key, value) in s_album.iter() {
                                                value.draw(
                                                    &script.1,
                                                    &table.get(&**key).unwrap(),
                                                    ui,
                                                    (select, *index),
                                                );
                                            }
                                        });
                                    }
                                }
                            });

                            if row.response().clicked() {
                                app.window.select.1 = (Some(*index), Some(i));
                                app.window.select.2 = (None, None);
                                app.library.list_shown.2 = (0..album.list_track.len()).collect();
                            }

                            if row.response().double_clicked() {
                                click = Some((
                                    app.window.select.0.0.unwrap(),
                                    app.window.select.1.0.unwrap(),
                                ));
                            }
                        });
                    });

                    if sort {
                        app.library.list_shown.1.reverse();
                    }

                    if let Some(click) = click {
                        App::error_result(Self::queue_play_album(app, click.0, click.1, context));
                    }
                }
            });
    }

    #[rustfmt::skip]
    fn draw_panel_track(app: &mut App, context: &egui::Context) {
        let rect = context.available_rect();

        egui::TopBottomPanel::bottom("panel_track")
            .resizable(false)
            .exact_height(rect.max.y / 2.0)
            .show(context, |ui| {
                let mut click = None;

                if let Some(i_group) = app.window.select.0.0
                    && let Some(i_album) = app.window.select.1.0
                {
                    let group = app.library.list_group.get(i_group).unwrap();
                    let album = group.list_album.get(i_album).unwrap();

                    ui.add_space(6.0);

                    if ui.text_edit_singleline(&mut app.window.search.2).changed() {
                        app.library.list_shown.2.clear();
                        app.window.select.2 = (None, None);

                        for (i, track) in album.list_track.iter().enumerate() {
                            if track
                                .name
                                .to_lowercase()
                                .trim()
                                .contains(app.window.search.2.to_lowercase().trim())
                            {
                                app.library.list_shown.2.push(i);
                            }
                        }
                    };

                    ui.separator();

                    let mut table = TableBuilder::new(ui)
                        .striped(true)
                        .sense(egui::Sense::click());

                    if app.setting.window_track { table = table.column(Column::auto().resizable(true)); }
                                                  table = table.column(Column::remainder().resizable(true).clip(true));
                    if app.setting.window_kind  { table = table.column(Column::remainder().resizable(true).clip(true)); }
                    if app.setting.window_date  { table = table.column(Column::remainder().resizable(true).clip(true)); }
                    if app.setting.window_time  { table = table.column(Column::remainder().resizable(true).clip(true)); }

                    let table = table.header(16.0, |mut header| {
                        if app.setting.window_track { header.col(|ui| { ui.strong("Track"); }); }
                                                      header.col(|ui| { ui.strong("Title"); });
                        if app.setting.window_kind  { header.col(|ui| { ui.strong("Genre"); }); }
                        if app.setting.window_date  { header.col(|ui| { ui.strong("Date");  }); }
                        if app.setting.window_time  { header.col(|ui| { ui.strong("Time");  }); }
                    });

                    table.body(|ui| {
                        ui.rows(16.0, app.library.list_shown.2.len(), |mut row| {
                            let i = row.index();
                            if let Some(select) = app.window.select.2.1 {
                                row.set_selected(i == select);
                            }

                            let index = app.library.list_shown.2.get(i).unwrap();
                            let track = album.list_track.get(*index).unwrap();

                            if app.setting.window_track {
                                row.col(|ui| {
                                    let order = track.track.unwrap_or_default().to_string();
                                    ui.add(egui::Label::new(&order).selectable(false));
                                });
                            }

                            row.col(|ui| {
                                ui.add(egui::Label::new(&track.name).selectable(false));
                            });

                            if app.setting.window_kind {
                                row.col(|ui| {
                                    ui.add(
                                        egui::Label::new(track.kind.as_deref().unwrap_or_default())
                                        .selectable(false),
                                    );
                                });
                            }

                            if app.setting.window_date {
                                row.col(|ui| {
                                    ui.add(
                                        egui::Label::new(track.date.as_deref().unwrap_or_default())
                                        .selectable(false),
                                    );
                                });
                            }

                            if app.setting.window_time {
                                row.col(|ui| {
                                    ui.add(
                                        egui::Label::new(Self::format_time(
                                            track.time.as_secs() as usize
                                        ))
                                        .selectable(false),
                                    );
                                });
                            }

                            row.response().context_menu(|ui| {
                                for script in &mut app.script.script_list {
                                    if let Some(s_track) = &mut script.0.track {
                                        ui.collapsing(&script.0.name, |ui| {
                                            let table: mlua::Table = script.1.get("track").unwrap();

                                            for (key, value) in s_track.iter() {
                                                value.draw(
                                                    &script.1,
                                                    &table.get(&**key).unwrap(),
                                                    ui,
                                                    (i_group, i_album, *index),
                                                );
                                            }
                                        });
                                    }
                                }
                            });

                            if row.response().clicked() {
                                app.window.select.2 = (Some(*index), Some(i));
                                click = Some(*index);
                            }
                        });
                    });

                    if let Some(click) = click {
                        App::error_result(Self::queue_play_track(
                            app,
                            app.window.select.0.0.unwrap(),
                            app.window.select.1.0.unwrap(),
                            click,
                            context,
                        ));
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
