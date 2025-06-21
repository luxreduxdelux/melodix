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

use discord_presence::{Client, Event};
use mlua::prelude::*;
use musicbrainz_rs::{
    client::MusicBrainzClient, entity::CoverartResponse, entity::release::*, prelude::*,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::Mutex;
use std::{collections::HashMap, time::UNIX_EPOCH};

//================================================================

struct Discord {
    status_client: Client,
    brainz_client: MusicBrainzClient,
    cache: HashMap<(String, String), CacheEntry>,
    tokio: tokio::runtime::Runtime,
    ready: Arc<Mutex<bool>>,
}

impl Discord {
    const USER_AGENT: &'static str =
        "MelodixDiscord/1.0.0 (https://github.com/luxreduxdelux/melodix)";
    const PATH_CACHE: &'static str = "script/discord.data";

    fn new() -> mlua::Result<Self> {
        let mut brainz_client = MusicBrainzClient::default();
        brainz_client
            .set_user_agent(Self::USER_AGENT)
            .map_err(|_| {
                mlua::Error::runtime("MelodixDiscord: Could not connect to MusicBrainz.")
            })?;

        //================================================================

        let mut status_client = Client::new(1385408557796687923);
        status_client.start();

        let ready = Arc::new(Mutex::new(false));
        let clone = ready.clone();
        status_client
            .on_ready(move |_| {
                let mut clone = clone.lock().unwrap();
                *clone = true;
                println!("ready!");
            })
            .persist();

        //================================================================

        let tokio = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|_| mlua::Error::runtime("MelodixDiscord: Could not create runtime."))?;

        //================================================================

        let cache = {
            if let Ok(file) = std::fs::read(Self::PATH_CACHE) {
                postcard::from_bytes(&file).unwrap_or_default()
            } else {
                HashMap::default()
            }
        };

        //================================================================

        Ok(Self {
            status_client,
            brainz_client,
            cache,
            tokio,
            ready,
        })
    }

    async fn get_cover(client: MusicBrainzClient, group: &str, album: &str) -> Option<String> {
        let query = ReleaseSearchQuery::query_builder()
            .artist(group)
            .and()
            .release(album)
            .build();

        if let Ok(search) = Release::search(query).execute_with_client(&client).await {
            for release in search.entities {
                if let Ok(fetch) = release.get_coverart().execute_with_client(&client).await {
                    match fetch {
                        CoverartResponse::Json(cover) => {
                            if let Some(cover) = cover.images.first() {
                                return Some(cover.image.clone());
                            }
                        }
                        CoverartResponse::Url(cover) => {
                            return Some(cover);
                        }
                    }
                }
            }
        };

        None
    }
}

impl Drop for Discord {
    fn drop(&mut self) {
        let serialize: Vec<u8> = postcard::to_allocvec(&self.cache)
            .expect("MelodixDiscord: Could not write image cache.");
        std::fs::write(Self::PATH_CACHE, serialize)
            .expect("MelodixDiscord: Could not write image cache.");
    }
}

impl mlua::UserData for Discord {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_async_method_mut(
            "set_state",
            |_,
             mut this,
             (group, album, track, image, time): (String, String, String, bool, u32)| async move {
                println!("==== BEGIN DISCORD {album} {track}");

                if !*this.ready.lock().unwrap() {
                    println!("Could not set Discord status.");
                    return Ok(());
                }

                // enter into tokio guard.
                let _enter = this.tokio.enter();

                //================================================================

                // get image from MusicBrainz.
                let image = {
                    // don't use an external image.
                    if !image {
                        None
                    } else {
                        match this.cache.get(&(group.clone(), album.clone())) {
                            Some(cache) => match cache {
                                CacheEntry::Path(path) => {
                                    println!("DISCORD: Get path from cache");
                                    Some(path).cloned()
                                }
                                CacheEntry::Null => None,
                            },
                            None => {
                                // clone the group, album, and brainz client to move into async thread.
                                let clone_group = group.clone();
                                let clone_album = album.clone();
                                let clone_brain = this.brainz_client.clone();

                                // get image from MusicBrainz.
                                let image = tokio::spawn(async move {
                                    Self::get_cover(clone_brain, &clone_group, &clone_album).await
                                });
                                let image = this.tokio.block_on(image).expect(
                                    "MelodixDiscord: Could not retrieve MusicBrainz image.",
                                );

                                if let Some(image) = image {
                                    // insert path into cache.
                                    this.cache.insert(
                                        (group.clone(), album.clone()),
                                        CacheEntry::Path(image.clone()),
                                    );

                                    println!("DISCORD: Get path from MusicBrainz");

                                    Some(image)
                                } else {
                                    // insert null into cache.
                                    this.cache
                                        .insert((group.clone(), album.clone()), CacheEntry::Null);

                                    None
                                }
                            }
                        }
                    }
                };

                //================================================================

                // calculate begin/end time-stamp.
                let time_a: u64 = std::time::SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("MelodixDiscord: Could not get system time.")
                    .as_secs();
                let time_b = time_a + time as u64;

                //================================================================

                // set Discord status.
                this.status_client
                    .set_activity(|act| {
                        act.details(track)
                            .state(group)
                            ._type(discord_presence::models::ActivityType::Listening)
                            .timestamps(|f| f.start(time_a).end(time_b))
                            .assets(|f| {
                                if let Some(image) = image {
                                    f.small_text(album).small_image(image)
                                } else {
                                    f.small_text(album)
                                }
                            })
                    })
                    .expect("MelodixDiscord: Could not set Discord state.");

                println!("==== CLOSE DISCORD");

                Ok(())
            },
        );
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum CacheEntry {
    Path(String),
    Null,
}

//================================================================

#[mlua::lua_module]
fn melodix_discord(_: &Lua) -> LuaResult<Discord> {
    Discord::new()
}
