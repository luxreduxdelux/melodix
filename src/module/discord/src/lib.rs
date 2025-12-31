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

use discord_presence::Client;
use mlua::prelude::*;
use musicbrainz_rs::{
    client::MusicBrainzClient,
    entity::{CoverartResponse, release::*},
    prelude::*,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::UNIX_EPOCH,
};

//================================================================

struct Discord {
    status_client: Client,
    brainz_client: MusicBrainzClient,
    setting: Arc<Mutex<Setting>>,
    connect: Arc<Mutex<bool>>,
}

impl Discord {
    const USER_AGENT: &'static str =
        "MelodixDiscord/1.0.0 (https://github.com/luxreduxdelux/melodix)";
    const DISCORD_APP: u64 = 1385408557796687923;

    fn new(_: &Lua, _: ()) -> mlua::Result<Self> {
        let mut brainz_client = MusicBrainzClient::default();
        brainz_client
            .set_user_agent(Self::USER_AGENT)
            .map_err(|_| {
                mlua::Error::runtime("MelodixDiscord: Could not connect to MusicBrainz.")
            })?;

        //================================================================

        let mut status_client = Client::new(Self::DISCORD_APP);
        status_client.start();

        let connect = Arc::new(Mutex::new(false));
        let discord = connect.clone();
        status_client
            .on_ready(move |_| {
                *discord.lock().unwrap() = true;
            })
            .persist();

        //================================================================

        Ok(Self {
            status_client,
            brainz_client,
            setting: Arc::new(Mutex::new(Setting::default())),
            connect,
        })
    }

    fn apply_state(
        mut d_client: Client,
        group: String,
        album: String,
        track: String,
        image: Option<String>,
        time_a: u32,
        time_b: u32,
    ) {
        // calculate begin/end time-stamp.
        let unix: u64 = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("MelodixDiscord: Could not get system time.")
            .as_secs();
        let time_a = unix - time_a as u64;
        let time_b = unix + time_b as u64;

        //================================================================

        // set Discord status.
        std::thread::spawn(move || {
            d_client
                .set_activity(|act| {
                    act.details(track)
                        .state(group)
                        ._type(discord_presence::models::ActivityType::Listening)
                        .timestamps(|f| f.start(time_a).end(time_b))
                        .assets(|f| {
                            f.small_text(album)
                                .small_image(image.unwrap_or("icon".to_string()))
                        })
                })
                .expect("MelodixDiscord: Could not apply Discord state.");
        });
    }

    fn clear_state(mut d_client: Client) {
        // set Discord status.
        std::thread::spawn(move || {
            d_client
                .clear_activity()
                .expect("MelodixDiscord: Could not clear Discord state.");
        });
    }

    fn get_cover(
        d_client: Client,
        m_client: MusicBrainzClient,
        setting: Arc<Mutex<Setting>>,
        group: String,
        album: String,
        track: String,
        time_a: u32,
        time_b: u32,
    ) {
        let query = ReleaseSearchQuery::query_builder()
            .artist(&group)
            .and()
            .release(&album)
            .build();

        if let Ok(search) = Release::search(query).execute_with_client(&m_client) {
            for release in search.entities {
                if let Ok(fetch) = release.get_coverart().execute_with_client(&m_client) {
                    match fetch {
                        CoverartResponse::Json(cover) => {
                            if let Some(cover) = cover.images.first() {
                                let mut setting = setting.lock().unwrap();
                                let cache = &mut setting.cache;

                                cache.insert(
                                    (group.clone(), album.clone()),
                                    CacheEntry::Path(cover.image.clone()),
                                );
                                Self::apply_state(
                                    d_client,
                                    group,
                                    album,
                                    track,
                                    Some(cover.image.clone()),
                                    time_a,
                                    time_b,
                                );
                                return;
                            }
                        }
                        CoverartResponse::Url(cover) => {
                            let mut setting = setting.lock().unwrap();
                            let cache = &mut setting.cache;

                            cache.insert(
                                (group.clone(), album.clone()),
                                CacheEntry::Path(cover.clone()),
                            );
                            Self::apply_state(
                                d_client,
                                group,
                                album,
                                track,
                                Some(cover),
                                time_a,
                                time_b,
                            );
                            return;
                        }
                    }
                }
            }

            let mut setting = setting.lock().unwrap();
            let cache = &mut setting.cache;

            cache.insert((group.clone(), album.clone()), CacheEntry::Null);
            Self::apply_state(d_client, group, album, track, None, time_a, time_b);
        }
    }
}

impl mlua::UserData for Discord {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut(
            "state_play",
            |_, this, (group, album, track, time_a, time_b): (String, String, String, u32, u32)| {
                if !*this.connect.lock().unwrap() {
                    return Ok(false);
                }

                //================================================================

                // acquire setting lock.
                let lock = this.setting.lock().unwrap();

                //================================================================

                // do not use MusicBrainz cover art. set state with no image.
                if !lock.cover {
                    Self::apply_state(
                        this.status_client.clone(),
                        group,
                        album,
                        track,
                        None,
                        time_a,
                        time_b,
                    );
                    return Ok(true);
                }

                //================================================================

                // check if path is in cache.
                let path = lock.cache.get(&(group.clone(), album.clone()));

                // if cache is in path...
                if let Some(cache) = path {
                    match cache {
                        // path was in cache, set state with image.
                        CacheEntry::Path(path) => {
                            Self::apply_state(
                                this.status_client.clone(),
                                group,
                                album,
                                track,
                                Some(path.to_string()),
                                time_a,
                                time_b,
                            );
                            return Ok(true);
                        }
                        // image could not be found in a previous instance, set state with no image.
                        CacheEntry::Null => {
                            Self::apply_state(
                                this.status_client.clone(),
                                group,
                                album,
                                track,
                                None,
                                time_a,
                                time_b,
                            );
                            return Ok(true);
                        }
                    }
                }

                //================================================================

                // clone the group, album, and brainz client to move into async thread.
                let clone_state = this.status_client.clone();
                let clone_brain = this.brainz_client.clone();
                let clone_cache = this.setting.clone();
                let clone_group = group.clone();
                let clone_album = album.clone();
                let clone_track = album.clone();

                // get image from MusicBrainz.
                std::thread::spawn(move || {
                    Self::get_cover(
                        clone_state,
                        clone_brain,
                        clone_cache,
                        clone_group,
                        clone_album,
                        clone_track,
                        time_a,
                        time_b,
                    );
                });

                Ok(true)
            },
        );

        methods.add_method_mut("state_stop", |_, this, _: ()| {
            if !*this.connect.lock().unwrap() {
                return Ok(false);
            }

            Self::clear_state(this.status_client.clone());
            Ok(true)
        });

        methods.add_method_mut("get_cover_art", |_, this, _: ()| {
            Ok(this.setting.lock().unwrap().cover)
        });

        methods.add_method_mut("set_cover_art", |_, this, state: bool| {
            this.setting.lock().unwrap().cover = state;
            Ok(())
        });

        methods.add_method_mut("get_warn", |_, this, _: ()| {
            Ok(this.setting.lock().unwrap().warn)
        });

        methods.add_method_mut("set_warn", |_, this, warn: bool| {
            this.setting.lock().unwrap().warn = warn;
            Ok(())
        });
    }
}

#[derive(Serialize, Deserialize)]
struct Setting {
    cache: HashMap<(String, String), CacheEntry>,
    cover: bool,
    warn: bool,
}

impl Setting {
    const PATH_DATA: &'static str = "script/discord.data";

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
                cache: HashMap::default(),
                cover: true,
                warn: true,
            }
        }
    }
}

impl Drop for Setting {
    fn drop(&mut self) {
        let serialize: Vec<u8> =
            postcard::to_allocvec(&*self).expect("MelodixDiscord: Could not write setting data.");
        std::fs::write(Self::get_configuration_path(), serialize)
            .expect("MelodixDiscord: Could not write setting data.");
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum CacheEntry {
    Path(String),
    Null,
}

//================================================================

#[mlua::lua_module]
fn melodix_discord(lua: &Lua) -> LuaResult<Discord> {
    Ok(Discord::new(lua, ())?)
}
