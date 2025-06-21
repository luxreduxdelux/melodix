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

use std::collections::HashMap;
use std::default;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::app::*;
use crate::layout::*;
use crate::library::*;
use crate::setting::*;

use mlua::prelude::*;
use serde::Deserialize;
use serde::Serialize;

//================================================================

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum SettingData {
    String {
        data: String,
        name: String,
        info: String,
        call: Option<String>,
    },
    Number {
        data: f32,
        name: String,
        info: String,
        bind: (f32, f32),
        call: Option<String>,
    },
    Boolean {
        data: bool,
        name: String,
        info: String,
        call: Option<String>,
    },
}

#[derive(Deserialize)]
pub struct Module {
    pub name: String,
    pub info: String,
    pub from: String,
    pub version: String,
    pub setting: Option<HashMap<String, SettingData>>,
}

impl Module {
    pub fn new(lua: &Lua, path: &str) -> Result<(Self, mlua::Table), ()> {
        let file = std::fs::read_to_string(path).map_err(|_| ())?;
        let table = lua.load(file).eval::<mlua::Table>().map_err(|_| ())?;
        let serde = LuaDeserializeOptions::new().deny_unsupported_types(false);
        let value = lua
            .from_value_with(mlua::Value::Table(table.clone()), serde)
            .map_err(|_| ())?;

        Ok((value, table))
    }
}

pub struct Script {
    pub lua: Lua,
    pub script_list: Vec<(Module, mlua::Table)>,
}

#[derive(Default)]
pub struct ScriptState {
    pub text: Arc<Mutex<String>>,
    pub library: Arc<Library>,
    pub setting: Arc<Setting>,
}

impl Script {
    const PATH_SCRIPT: &'static str = "script/";
    pub const CALL_BEGIN: &'static str = "begin";
    pub const CALL_CLOSE: &'static str = "close";
    pub const CALL_LOOP: &'static str = "loop";
    pub const CALL_SEEK: &'static str = "seek";
    pub const CALL_STOP: &'static str = "stop";
    pub const CALL_PLAY: &'static str = "play";
    pub const CALL_SKIP_A: &'static str = "skip_a";
    pub const CALL_SKIP_B: &'static str = "skip_b";
    pub const CALL_PAUSE: &'static str = "pause";

    pub fn new(setting: &Setting) -> Self {
        let lua = unsafe { Lua::unsafe_new() };
        let mut script_list = Vec::new();

        for file in std::fs::read_dir(Self::PATH_SCRIPT).unwrap() {
            let file = file.unwrap().path().display().to_string();
            if let Ok(module) = Module::new(&lua, &file) {
                script_list.push(module);
            }
        }

        let script = Self { lua, script_list };

        script.call(Self::CALL_BEGIN, ());

        script
    }

    pub fn call<M: IntoLuaMulti + Send + Clone + 'static>(&self, entry: &'static str, member: M) {
        for script in &self.script_list {
            if let Ok(function) = script.1.get::<mlua::Function>(entry) {
                let table = script.1.clone();
                let clone = member.clone();

                tokio::spawn(async move {
                    if let Err(error) = function.call_async::<()>((table, clone)).await {
                        App::error(&error.to_string());
                    }
                });
            }
        }
    }
}
