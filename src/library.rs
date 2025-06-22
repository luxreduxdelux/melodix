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

use rodio::Source;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::BufReader;
use std::path::Path;
use std::time::UNIX_EPOCH;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use walkdir::WalkDir;

//================================================================

use rayon::prelude::*;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Library {
    pub list_group: Vec<Group>,
}

impl Library {
    const PATH_LIBRARY: &'static str = "library.data";

    pub fn new() -> Self {
        if let Ok(file) = std::fs::read(Self::PATH_LIBRARY) {
            if let Ok(library) = postcard::from_bytes(&file) {
                return library;
            }
        }

        Self::default()
    }

    pub fn scan(path: &str) -> Self {
        let path: Vec<walkdir::DirEntry> =
            WalkDir::new(path).into_iter().map(|x| x.unwrap()).collect();

        let mut map_group: HashMap<String, Group> = HashMap::default();

        let track_list: Vec<(Option<String>, Option<String>, Track)> = path
            .par_iter()
            .filter_map(|entry| Track::new(entry.path().to_str().unwrap()))
            .collect();

        for (group, album, track) in track_list {
            let group = {
                if let Some(group) = group {
                    map_group.entry(group.clone()).or_insert(Group {
                        name: group,
                        list_album: vec![],
                    })
                } else {
                    map_group
                        .entry("< Unknown Group >".to_string())
                        .or_insert(Group {
                            name: "< Unknown Group >".to_string(),
                            list_album: vec![],
                        })
                }
            };

            let album = album.unwrap_or("< Unknown Album >".to_string());

            group.insert_track(&album, track);
        }

        let mut list_group: Vec<Group> = map_group.values().cloned().collect();

        list_group.sort_by(|a, b| a.name.cmp(&b.name));

        for group in &mut list_group {
            group.list_album.sort_by(|a, b| a.name.cmp(&b.name));

            for album in &mut group.list_album {
                album.list_track.sort_by(|a, b| {
                    a.track
                        .unwrap_or_default()
                        .cmp(&b.track.unwrap_or_default())
                });
            }
        }

        let library = Self { list_group };

        let serialize: Vec<u8> = postcard::to_allocvec(&library).unwrap();
        std::fs::write("library.data", serialize).unwrap();

        library
    }
}

//================================================================

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub name: String,
    pub list_album: Vec<Album>,
}

impl Group {
    fn get_image(path: &str) -> Option<String> {
        let path = Path::new(path);
        let path = path.parent().unwrap();

        for path in std::fs::read_dir(path).unwrap() {
            let path = path.unwrap().path().display().to_string();

            if path.ends_with(".jpg") {
                println!("Found {path}");
                return Some(path);
            }
        }

        None
    }

    pub fn insert_track(&mut self, album: &str, track: Track) {
        for a in &mut self.list_album {
            if a.name == album {
                a.list_track.push(track);
                return;
            }
        }

        let icon = {
            if track.icon.is_none() {
                Self::get_image(&track.path)
            } else {
                None
            }
        };

        self.list_album.push(Album {
            name: album.to_string(),
            icon,
            list_track: vec![track],
        });
    }
}

//================================================================

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Album {
    pub name: String,
    pub icon: Option<String>,
    pub list_track: Vec<Track>,
}

//================================================================

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub name: String,
    pub path: String,
    pub time: u64,
    pub date: Option<String>,
    pub kind: Option<String>,
    pub icon: Option<Vec<u8>>,
    pub track: Option<usize>,
}

impl Track {
    pub fn new(path: &str) -> Option<(Option<String>, Option<String>, Track)> {
        // Open the media source.
        let src = std::fs::File::open(path).expect("failed to open media");

        // Create the media source stream.
        let mss = MediaSourceStream::new(Box::new(src), Default::default());

        // Create a probe hint using the file's extension. [Optional]
        let mut hint = Hint::new();
        hint.with_extension("mp3");

        // Use the default options for metadata and format readers.
        let meta_opts: MetadataOptions = Default::default();
        let fmt_opts: FormatOptions = Default::default();

        // Probe the media source.
        if let Ok(mut probed) =
            symphonia::default::get_probe().format(&hint, mss, &fmt_opts, &meta_opts)
        {
            let time = {
                if let Ok(src) = std::fs::File::open(path)
                    && let Ok(source) = rodio::Decoder::new(BufReader::new(src))
                {
                    let duration = source.total_duration().unwrap_or_default().as_secs();

                    if duration == 0 {
                        if let Ok(source) = mp3_duration::from_path(path) {
                            source.as_secs()
                        } else {
                            0
                        }
                    } else {
                        duration
                    }
                } else {
                    0
                }
                /*
                 if let Ok(source) = mp3_duration::from_path(path) {
                    source.as_secs()
                } else {
                    0
                }
                */
            };

            let mut file_group: Option<String> = None;
            let mut file_album: Option<String> = None;
            let mut file_track = Track {
                name: path.to_string(),
                path: path.to_string(),
                date: None,
                kind: None,
                time,
                icon: None,
                track: None,
            };

            if let Some(revision) = probed.metadata.get().as_ref().and_then(|m| m.current()) {
                for tag in revision.tags() {
                    if let Some(key) = tag.std_key {
                        match key {
                            symphonia::core::meta::StandardTagKey::Artist => {
                                file_group = Some(tag.value.to_string())
                            }
                            symphonia::core::meta::StandardTagKey::Album => {
                                file_album = Some(tag.value.to_string())
                            }
                            symphonia::core::meta::StandardTagKey::Genre => {
                                file_track.kind = Some(tag.value.to_string())
                            }
                            symphonia::core::meta::StandardTagKey::Date => {
                                file_track.date = Some(tag.value.to_string())
                            }
                            symphonia::core::meta::StandardTagKey::TrackTitle => {
                                file_track.name = tag.value.to_string()
                            }
                            symphonia::core::meta::StandardTagKey::TrackNumber => {
                                file_track.track = Some(tag.value.to_string().parse().unwrap());
                            }
                            _ => {}
                        }
                    }
                }

                if let Some(visual) = revision.visuals().first() {
                    file_track.icon = Some(visual.data.to_vec());
                }
            }

            return Some((file_group, file_album, file_track));
        }

        None
    }
}
