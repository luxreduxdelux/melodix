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
use crate::setting::*;

use mlua::prelude::*;

//================================================================

pub struct Script {
    pub lua: Lua,
    pub script_list: Vec<mlua::Table>,
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
        let lua = Lua::new();
        let mut script_list = Vec::new();

        for file in std::fs::read_dir(Self::PATH_SCRIPT).unwrap() {
            let file = file.unwrap().path();
            let file = std::fs::read_to_string(file).unwrap();

            match lua.load(file).eval::<mlua::Table>() {
                Ok(value) => {
                    // TO-DO load script data from melodix.data
                    /*
                    let script_name = value.get::<String>("name").unwrap();

                    if let Ok(set) = value.get::<mlua::Table>("setting")
                        && let Some(entry) = setting.script_setting.get(&script_name)
                    {
                        for pair in set.pairs::<String, mlua::Value>() {
                            let (key, value) = pair.unwrap();
                        }
                    }
                    */

                    script_list.push(value);
                }
                Err(message) => {
                    App::error(&message.to_string());
                }
            };
        }

        let script = Self { lua, script_list };

        script.call(Self::CALL_BEGIN, ());

        script
    }

    pub fn call<M: IntoLuaMulti + Copy>(&self, entry: &'static str, member: M) {
        for script in &self.script_list {
            if let Ok(function) = script.get::<mlua::Function>(entry) {
                if let Err(error) = function.call::<()>((script, member)) {
                    App::error(&error.to_string());
                }
            }
        }
    }
}
