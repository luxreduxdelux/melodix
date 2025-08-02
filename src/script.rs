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

use crate::{app::*, library::*, setting::*, window::*};

//================================================================

use eframe::egui::{self};
use mlua::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

//================================================================

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum SettingData {
    Button {
        name: String,
        info: String,
        call: Option<String>,
    },
    Toggle {
        name: String,
        info: String,
        call: Option<String>,
    },
    Slider {
        name: String,
        info: String,
        bind: (f32, f32),
        call: Option<String>,
    },
    Record {
        name: String,
        info: String,
        call: Option<String>,
    },
}

impl SettingData {
    pub fn draw<M: IntoLuaMulti + Send + 'static>(
        &self,
        script: &mlua::Table,
        table: &mlua::Table,
        ui: &mut egui::Ui,
        member: M,
    ) {
        match self {
            SettingData::Button { name, info, call } => {
                let widget = ui.button(name);

                if widget.on_hover_text(info).clicked() {
                    if let Some(call) = call {
                        Script::safe_call(script.clone(), script.get(&**call).unwrap(), member);
                    }
                }
            }
            SettingData::Toggle { name, info, call } => {
                let mut data: bool = table.get("data").unwrap();
                let widget = ui.checkbox(&mut data, name);

                if widget.on_hover_text(info).clicked() {
                    table.set("data", data).unwrap();

                    if let Some(call) = call {
                        Script::safe_call(script.clone(), script.get(&**call).unwrap(), member);
                    }
                }
            }
            SettingData::Slider {
                name,
                info,
                bind,
                call,
            } => {
                let mut data: f32 = table.get("data").unwrap();
                let widget = ui.add(egui::Slider::new(&mut data, bind.0..=bind.1).text(name));

                if widget.on_hover_text(info).drag_stopped() {
                    table.set("data", data).unwrap();

                    if let Some(call) = call {
                        Script::safe_call(script.clone(), script.get(&**call).unwrap(), member);
                    }
                }
            }
            SettingData::Record { name, info, call } => {
                let mut data: String = table.get("data").unwrap();
                let widget = ui.label(name).id;
                let widget = ui.text_edit_singleline(&mut data).labelled_by(widget);

                if widget.on_hover_text(info).changed() {
                    table.set("data", data).unwrap();

                    if let Some(call) = call {
                        Script::safe_call(script.clone(), script.get(&**call).unwrap(), member);
                    }
                }
            }
        };
    }
}

#[derive(Deserialize)]
pub struct Module {
    pub name: String,
    pub info: String,
    pub from: String,
    pub version: String,
    pub setting: Option<HashMap<String, SettingData>>,
    pub album: Option<HashMap<String, SettingData>>,
    pub group: Option<HashMap<String, SettingData>>,
    pub track: Option<HashMap<String, SettingData>>,
    pub queue: Option<HashMap<String, SettingData>>,
}

impl Module {
    pub fn new(lua: &Lua, path: &str) -> Result<(Self, mlua::Table), ()> {
        // load the Lua source code.
        let file = std::fs::read_to_string(path).map_err(|_| ())?;

        // retrieve the module table.
        let table = lua.load(file).eval::<mlua::Table>().map_err(|_| ())?;
        let serde = LuaDeserializeOptions::new().deny_unsupported_types(false);
        let value = lua
            .from_value_with(mlua::Value::Table(table.clone()), serde)
            .map_err(|_| ())?;

        // return module info and raw Lua table.
        Ok((value, table))
    }
}

pub struct Script {
    #[allow(dead_code)]
    pub lua: Lua,
    pub script_list: Vec<(Module, mlua::Table)>,
    pub initialize: bool,
}

impl Script {
    const PATH_SCRIPT: &'static str = "script/";
    pub const DATA_MAIN: &'static str = include_str!("lua/main.lua");
    pub const DATA_META: &'static str = include_str!("lua/meta.lua");
    pub const CALL_BEGIN: &'static str = "begin";
    pub const CALL_CLOSE: &'static str = "close";
    pub const CALL_SEEK: &'static str = "seek";
    pub const CALL_STOP: &'static str = "stop";
    pub const CALL_PLAY: &'static str = "play";
    pub const CALL_SKIP_A: &'static str = "skip_a";
    pub const CALL_SKIP_B: &'static str = "skip_b";
    pub const CALL_PAUSE: &'static str = "pause";

    fn get_path() -> String {
        let home = {
            if let Some(path) = std::env::home_dir() {
                let path = format!("{}/.melodix/", path.display().to_string());

                if let Ok(false) = std::fs::exists(&path) {
                    std::fs::create_dir(&path).unwrap();
                }

                path
            } else {
                String::default()
            }
        };

        format!("{home}{}", Self::PATH_SCRIPT)
    }

    pub fn new(setting: &Setting) -> anyhow::Result<Self> {
        let lua = unsafe { Lua::unsafe_new() };
        let mut script_list = Vec::new();
        let path = Self::get_path();

        if setting.script_allow {
            if let Ok(true) = std::fs::exists(&path) {
                for file in std::fs::read_dir(&path)? {
                    let file = file?.path().display().to_string();
                    if let Ok(module) = Module::new(&lua, &file) {
                        script_list.push(module);
                    }
                }
            }
        }

        Self::set_global(&lua)?;

        Ok(Self {
            lua,
            script_list,
            initialize: false,
        })
    }

    // TO-DO rename to call, old call should be call_all
    pub fn safe_call<M: IntoLuaMulti + Send + 'static>(
        table: mlua::Table,
        entry: mlua::Function,
        member: M,
    ) {
        if let Err(error) = entry.call::<()>((table, member)) {
            App::error(&error.to_string());
        }
    }

    pub fn call<M: IntoLuaMulti + Send + Clone + 'static>(&self, entry: &'static str, member: M) {
        for script in &self.script_list {
            if let Ok(function) = script.1.get::<mlua::Function>(entry) {
                let table = script.1.clone();
                let clone = member.clone();

                if let Err(error) = function.call::<()>((table, clone)) {
                    App::error(&error.to_string());
                }
            }
        }
    }

    fn set_global(lua: &Lua) -> anyhow::Result<()> {
        let melodix = lua.create_table()?;

        melodix.set("get_library", lua.create_function(Self::get_library)?)?;
        melodix.set("get_state", lua.create_function(Self::get_state)?)?;
        melodix.set("get_queue", lua.create_function(Self::get_queue)?)?;
        melodix.set("set_toast", lua.create_function(Self::set_toast)?)?;

        lua.globals().set("melodix", melodix)?;

        Ok(())
    }

    fn get_library(lua: &Lua, _: ()) -> mlua::Result<mlua::Value> {
        let app = App::dereference();

        lua.to_value(&app.library)
    }

    fn get_state(lua: &Lua, _: ()) -> mlua::Result<(mlua::Value, mlua::Value, mlua::Value)> {
        let app = App::dereference();

        if let Some((group, album, track)) = app.get_play_state() {
            Ok((
                lua.to_value(&group)?,
                lua.to_value(&album)?,
                lua.to_value(&track)?,
            ))
        } else {
            Ok((mlua::Nil, mlua::Nil, mlua::Nil))
        }
    }

    fn get_queue(lua: &Lua, _: ()) -> mlua::Result<(mlua::Value, mlua::Value)> {
        let app = App::dereference();

        let queue = &app.window.queue.0;
        let queue: Vec<(usize, usize, usize)> = queue
            .into_iter()
            .map(|(group, album, track)| (group + 1, album + 1, track + 1))
            .collect();

        Ok((
            lua.to_value(&queue)?,
            lua.to_value(&(app.window.queue.1 + 1))?,
        ))
    }

    fn set_toast(_: &Lua, (kind, text, time): (usize, String, f64)) -> mlua::Result<()> {
        let app = App::dereference();

        app.window.toast.add(egui_toast::Toast {
            text: text.into(),
            kind: match kind {
                0 => egui_toast::ToastKind::Info,
                1 => egui_toast::ToastKind::Warning,
                2 => egui_toast::ToastKind::Error,
                _ => egui_toast::ToastKind::Success,
            },
            options: egui_toast::ToastOptions::default()
                .duration_in_seconds(time)
                .show_progress(true)
                .show_icon(true),
            ..Default::default()
        });

        Ok(())
    }
}

impl Drop for Script {
    fn drop(&mut self) {
        self.call(Self::CALL_CLOSE, ());
    }
}
