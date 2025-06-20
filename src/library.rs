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

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use symphonia::core::codecs::{CODEC_TYPE_NULL, DecoderOptions};
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use walkdir::WalkDir;

//================================================================

#[derive(Default, Serialize, Deserialize)]
pub struct Library {
    pub list_artist: Vec<Artist>,
}

impl Library {
    const PATH_LIBRARY: &'static str = "library.data";

    pub fn new() -> (Self, bool) {
        if let Ok(file) = std::fs::read(Self::PATH_LIBRARY) {
            if let Ok(library) = postcard::from_bytes(&file) {
                return (library, false);
            }
        }

        (Self::default(), true)
    }

    pub fn scan(path: &str) -> Self {
        let mut map_artist: HashMap<String, Artist> = HashMap::new();
        let mut icon: Option<u8> = None;

        for entry in WalkDir::new(path) {
            if let Ok((artist, album, song)) = Song::new(entry.unwrap().path().to_str().unwrap()) {
                let artist = {
                    if let Some(artist) = artist {
                        map_artist.entry(artist.clone()).or_insert(Artist {
                            name: artist,
                            list_album: vec![],
                        })
                    } else {
                        map_artist
                            .entry("< Unknown Artist >".to_string())
                            .or_insert(Artist {
                                name: "< Unknown Artist >".to_string(),
                                list_album: vec![],
                            })
                    }
                };

                let album = album.unwrap_or("< Unknown Album >".to_string());

                artist.insert_song(&album, song);

                /*
                // TO-DO should really leave this up until after the user has selected the album.
                if album.icon.is_none() {
                    for entry in std::fs::read_dir(entry.path().parent().unwrap()).unwrap() {
                        let entry = entry.unwrap().path();

                        if entry.is_file() {
                            let data = std::fs::read(&entry).unwrap();

                            if image::guess_format(&data).is_ok() {
                                println!("Loading cover...{:?}", entry);
                                album.icon = Some(entry.display().to_string());
                                break;
                            }
                        }
                    }
                }

                album.list_song.push(Song {
                    name: file_song,
                    path: entry.path().display().to_string(),
                    time: samples_capacity / rate,
                    track: file_song_track,
                });
                */
            }
        }

        let mut list_artist: Vec<Artist> = map_artist.values().cloned().collect();

        list_artist.sort_by(|a, b| a.name.cmp(&b.name));

        for artist in &mut list_artist {
            artist.list_album.sort_by(|a, b| a.name.cmp(&b.name));

            for album in &mut artist.list_album {
                album.list_song.sort_by(|a, b| {
                    a.track
                        .unwrap_or_default()
                        .cmp(&b.track.unwrap_or_default())
                });
            }
        }

        let library = Self { list_artist };

        let serialize: Vec<u8> = postcard::to_allocvec(&library).unwrap();
        std::fs::write("library.data", serialize).unwrap();

        library
    }
}

//================================================================

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Artist {
    pub name: String,
    pub list_album: Vec<Album>,
}

impl Artist {
    pub fn insert_song(&mut self, album: &str, song: Song) {
        for a in &mut self.list_album {
            if a.name == album {
                a.list_song.push(song);
                return;
            }
        }

        self.list_album.push(Album {
            name: album.to_string(),
            icon: None,
            list_song: vec![song],
        });
    }
}

//================================================================

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Album {
    pub name: String,
    pub icon: Option<String>,
    pub list_song: Vec<Song>,
}

//================================================================

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Song {
    pub name: String,
    pub path: String,
    pub track: Option<usize>,
}

impl Song {
    pub fn new(path: &str) -> Result<(Option<String>, Option<String>, Song), ()> {
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
            let mut file_artist: Option<String> = None;
            let mut file_album: Option<String> = None;
            let mut file_song = Song {
                name: path.to_string(),
                path: path.to_string(),
                track: None,
            };

            if let Some(revision) = probed.metadata.get().as_ref().and_then(|m| m.current()) {
                for tag in revision.tags() {
                    if let Some(key) = tag.std_key {
                        match key {
                            symphonia::core::meta::StandardTagKey::Artist => {
                                file_artist = Some(tag.value.to_string())
                            }
                            symphonia::core::meta::StandardTagKey::Album => {
                                file_album = Some(tag.value.to_string())
                            }
                            symphonia::core::meta::StandardTagKey::TrackTitle => {
                                file_song.name = tag.value.to_string()
                            }
                            symphonia::core::meta::StandardTagKey::TrackNumber => {
                                file_song.track = Some(tag.value.to_string().parse().unwrap());
                            }
                            _ => {}
                        }
                    }
                }
            }

            return Ok((file_artist, file_album, file_song));
        }

        Err(())
    }
}
