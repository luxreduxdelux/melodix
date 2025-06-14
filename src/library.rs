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

#[derive(Serialize, Deserialize)]
pub struct Library {
    pub map_artist: HashMap<String, Artist>,
}

impl Default for Library {
    fn default() -> Self {
        let mut map_artist: HashMap<String, Artist> = HashMap::new();
        let mut icon: Option<u8> = None;

        for entry in WalkDir::new("/home/think/Music/") {
            let entry = entry.unwrap();

            // Open the media source.
            let src = std::fs::File::open(entry.path()).expect("failed to open media");

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
                let mut file_song: Option<String> = None;
                let mut file_song_track: Option<usize> = None;

                if let Some(revision) = probed.metadata.get().as_ref().and_then(|m| m.current()) {
                    for tag in revision.tags() {
                        //println!("{}", tag);

                        if let Some(key) = tag.std_key {
                            match key {
                                symphonia::core::meta::StandardTagKey::Artist => {
                                    file_artist = Some(tag.value.to_string())
                                }
                                symphonia::core::meta::StandardTagKey::Album => {
                                    file_album = Some(tag.value.to_string())
                                }
                                symphonia::core::meta::StandardTagKey::TrackTitle => {
                                    file_song = Some(tag.value.to_string())
                                }
                                symphonia::core::meta::StandardTagKey::TrackNumber => {
                                    file_song_track = Some(tag.value.to_string().parse().unwrap());
                                }
                                _ => {}
                            }
                        }
                    }
                }

                // Find the first audio track with a known (decodeable) codec.
                let track = probed
                    .format
                    .tracks()
                    .iter()
                    .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
                    .unwrap();

                // Use the default options for the decoder.
                let dec_opts: DecoderOptions = Default::default();

                // Calculate the needed size for our sample vector
                // We do this now as we will borrow track in the decoder
                let samples_capacity: usize = if let Some(n_frames) = track.codec_params.n_frames {
                    n_frames as usize
                } else {
                    0
                };

                // Create a decoder for the track.
                let mut decoder = symphonia::default::get_codecs()
                    .make(&track.codec_params, &dec_opts)
                    .unwrap();

                // Create sample buffer and retrieve sample rate
                let rate = {
                    // Read first packet and determine sample buffer size
                    println!("enter : {:?}", entry.path());

                    let packet = probed.format.next_packet().unwrap();
                    if let Ok(d_p) = decoder.decode(&packet) {
                        println!("V : {:?}", entry.path());
                        let spec = *d_p.spec();
                        spec.rate as usize
                    } else {
                        println!("X : {:?}", entry.path());
                        1
                    }
                };

                let file_artist = file_artist.unwrap();
                let file_album = file_album.unwrap();
                let file_song = file_song.unwrap();
                let file_song_track = file_song_track.unwrap();

                let artist = map_artist.entry(file_artist).or_default();
                let album = artist.map_album.entry(file_album).or_default();

                album.list_song.push(Song {
                    name: file_song,
                    path: entry.path().display().to_string(),
                    time: samples_capacity / rate,
                    track: file_song_track,
                });
            }
        }

        for artist in map_artist.values_mut() {
            for album in artist.map_album.values_mut() {
                album.list_song.sort_by(|a, b| {
                    if a.track < b.track {
                        return Ordering::Less;
                    }
                    if a.track > b.track {
                        return Ordering::Greater;
                    }

                    Ordering::Equal
                });
            }
        }

        Self { map_artist }
    }
}

//================================================================

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Artist {
    pub map_album: HashMap<String, Album>,
}

//================================================================

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Album {
    pub icon: Option<u8>,
    pub list_song: Vec<Song>,
}

//================================================================

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Song {
    pub name: String,
    pub path: String,
    pub time: usize,
    pub track: usize,
}
