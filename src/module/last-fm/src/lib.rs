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

use mlua::prelude::*;
use rustfm_scrobble::{Scrobble, Scrobbler};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

//================================================================

#[derive(Default)]
struct LastFM {
    client: Option<Arc<Scrobbler>>,
    setting: Setting,
}

impl mlua::UserData for LastFM {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut(
            "connect",
            |_, this, (user, pass, key, key_secret): (String, String, String, String)| {
                let mut client = Scrobbler::new(&key, &key_secret);

                let result = if let Err(error) = client.authenticate_with_password(&user, &pass) {
                    Ok(Some(error.to_string()))
                } else {
                    Ok(None)
                };

                this.client = Some(Arc::new(client));

                result
            },
        );

        methods.add_method_mut(
            "state_play",
            |_, this, (group, album, track): (String, String, String)| {
                if let Some(client) = &this.client {
                    let client = client.clone();

                    std::thread::spawn(move || {
                        let song = Scrobble::new(&group, &track, &album);
                        client.now_playing(&song).unwrap();
                    });
                }

                Ok(())
            },
        );

        methods.add_method_mut(
            "state_scrobble",
            |_, this, (group, album, track): (String, String, String)| {
                if let Some(client) = &this.client {
                    let client = client.clone();

                    std::thread::spawn(move || {
                        let song = Scrobble::new(&group, &track, &album);
                        client.scrobble(&song).unwrap();
                    });
                }

                Ok(())
            },
        );

        methods.add_method_mut("get_warn", |_, this, _: ()| Ok(this.setting.warn));

        methods.add_method_mut("set_warn", |_, this, warn: bool| {
            this.setting.warn = warn;
            Ok(())
        });

        methods.add_method_mut("get_user", |_, this, _: ()| Ok(this.setting.user.clone()));

        methods.add_method_mut("set_user", |_, this, user: String| {
            this.setting.user = user;
            Ok(())
        });

        methods.add_method_mut("get_pass", |_, this, _: ()| Ok(this.setting.pass.clone()));

        methods.add_method_mut("set_pass", |_, this, pass: String| {
            this.setting.pass = pass;
            Ok(())
        });

        methods.add_method_mut("get_key", |_, this, _: ()| Ok(this.setting.key.clone()));

        methods.add_method_mut("set_key", |_, this, key: String| {
            this.setting.key = key;
            Ok(())
        });

        methods.add_method_mut("get_key_secret", |_, this, _: ()| {
            Ok(this.setting.key_secret.clone())
        });

        methods.add_method_mut("set_key_secret", |_, this, key_secret: String| {
            this.setting.key_secret = key_secret;
            Ok(())
        });
    }
}

#[derive(Serialize, Deserialize)]
struct Setting {
    warn: bool,
    user: String,
    pass: String,
    key: String,
    key_secret: String,
}

impl Setting {
    const PATH_DATA: &'static str = "script/last_fm.data";

    fn get_configuration_path() -> String {
        let home = {
            if let Some(path) = dirs::config_dir() {
                let path = format!("{}/melodix/", path.display());

                if let Ok(false) = std::fs::exists(&path) {
                    std::fs::create_dir(&path).unwrap();
                }

                path
            } else {
                String::default()
            }
        };

        format!("{home}{}", Self::PATH_DATA)
    }
}

impl Default for Setting {
    fn default() -> Self {
        if let Ok(file) = std::fs::read(Self::get_configuration_path())
            && let Ok(data) = postcard::from_bytes(&file)
        {
            data
        } else {
            Self {
                warn: true,
                user: String::default(),
                pass: String::default(),
                key: String::default(),
                key_secret: String::default(),
            }
        }
    }
}

impl Drop for Setting {
    fn drop(&mut self) {
        let serialize: Vec<u8> =
            postcard::to_allocvec(&*self).expect("MelodixLastFM: Could not write setting data.");
        std::fs::write(Self::get_configuration_path(), serialize)
            .expect("MelodixLastFM: Could not write setting data.");
    }
}

//================================================================

#[mlua::lua_module]
fn melodix_last_fm(_: &Lua) -> LuaResult<LastFM> {
    Ok(LastFM::default())
}
