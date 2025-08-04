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
use std::{
    collections::{BTreeMap, HashMap},
    fs::FileType,
    io::BufReader,
    path::Path,
    time::{Duration, SystemTime},
};
use symphonia::core::{
    formats::FormatOptions, io::MediaSourceStream, meta::MetadataOptions, probe::Hint,
};
use walkdir::{DirEntry, WalkDir};

//================================================================

use rayon::prelude::*;

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Library {
    pub list_group: Vec<Group>,
    #[serde(skip)]
    pub list_shown: (Vec<usize>, Vec<usize>, Vec<usize>),
}

impl Library {
    const PATH_LIBRARY: &'static str = "library.data";

    fn get_path() -> String {
        let home = {
            if let Some(path) = std::env::home_dir() {
                let path = format!("{}/.melodix/", path.display());

                if let Ok(false) = std::fs::exists(&path) {
                    std::fs::create_dir(&path).unwrap();
                }

                path
            } else {
                String::default()
            }
        };

        format!("{home}{}", Self::PATH_LIBRARY)
    }

    pub fn new() -> Self {
        if let Ok(file) = std::fs::read(Self::get_path())
            && let Ok(library) = postcard::from_bytes::<Self>(&file)
        {
            return Self {
                list_shown: (
                    (0..library.list_group.len()).collect(),
                    Vec::default(),
                    Vec::default(),
                ),
                list_group: library.list_group,
            };
        }

        Self::default()
    }

    pub fn scan(path: &str) -> Self {
        let path: Vec<walkdir::DirEntry> = WalkDir::new(path)
            .into_iter()
            .filter_map(|x| {
                let x = x.expect("Library::scan(): Couldn't obtain directory entry.");

                if x.file_type().is_file() {
                    if let Some(extension) = x.path().extension()
                        // in the interest of speed, just check for extension rather than an actual file type check.
                        && (extension == "mp3" || extension == "flac" || extension == "wav")
                    {
                        Some(x)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        let mut track_list: Vec<(String, String, Track)> =
            path.par_iter().filter_map(Track::new).collect();

        track_list.par_sort_by(|(_, a_album, a_track), (_, b_album, b_track)| {
            let a_track = a_track.track.unwrap_or_default();
            let b_track = b_track.track.unwrap_or_default();

            a_album.cmp(b_album).then(a_track.cmp(&b_track))
        });

        let mut map_group: HashMap<String, Group> = HashMap::default();

        for (group, album, track) in track_list {
            let group = {
                map_group.entry(group.clone()).or_insert(Group {
                    name: group,
                    list_album: vec![],
                })
            };

            group.insert_track(&album, track);
        }

        let mut list_group: Vec<Group> = map_group.into_values().collect();

        list_group.par_sort_by(|a, b| a.name.cmp(&b.name));

        let library = Self {
            list_shown: (
                (0..list_group.len()).collect(),
                Vec::default(),
                Vec::default(),
            ),
            list_group,
        };

        let serialize: Vec<u8> = postcard::to_allocvec(&library).unwrap();
        std::fs::write(Self::get_path(), serialize).unwrap();

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

        let path = std::fs::read_dir(path).unwrap().par_bridge();

        let path = path.into_par_iter().find_any(|path| {
            let path = path.as_ref().unwrap().path();
            let file = path.extension().unwrap_or_default();

            // in the interest of speed, just check for extension rather than an actual file type check.

            file == "png" || file == "jpg" || file == "jpeg"
        });

        path.map(|p| p.unwrap().path().display().to_string())
    }

    fn insert_track(&mut self, album: &str, track: Track) {
        if let Some(album) = self.list_album.par_iter_mut().find_any(|x| x.name == album) {
            album.list_track.push(track);
        } else {
            let icon = Self::get_image(&track.path);

            self.list_album.push(Album {
                name: album.to_string(),
                icon,
                list_track: vec![track],
            });
        }
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
    pub time: Duration,
    pub date: Option<String>,
    pub kind: Option<String>,
    pub icon: (Option<Vec<u8>>, Option<(u32, u32)>),
    pub track: Option<usize>,
}

impl Track {
    const GENRE_LIST: [&str; 192] = [
        "Blues",
        "Classic Rock",
        "Country",
        "Dance",
        "Disco",
        "Funk",
        "Grunge",
        "Hip-Hop",
        "Jazz",
        "Metal",
        "New Age",
        "Oldies",
        "Other",
        "Pop",
        "R&B",
        "Rap",
        "Reggae",
        "Rock",
        "Techno",
        "Industrial",
        "Alternative",
        "Ska",
        "Death Metal",
        "Pranks",
        "Soundtrack",
        "Euro-Techno",
        "Ambient",
        "Trip-Hop",
        "Vocal",
        "Jazz+Funk",
        "Fusion",
        "Trance",
        "Classical",
        "Instrumental",
        "Acid",
        "House",
        "Game",
        "Sound Clip",
        "Gospel",
        "Noise",
        "Alternative Rock",
        "Bass",
        "Soul",
        "Punk",
        "Space",
        "Meditative",
        "Instrumental Pop",
        "Instrumental Rock",
        "Ethnic",
        "Gothic",
        "Darkwave",
        "Techno-Industrial",
        "Electronic",
        "Pop-Folk",
        "Eurodance",
        "Dream",
        "Southern Rock",
        "Comedy",
        "Cult",
        "Gangsta Rap",
        "Top 40",
        "Christian Rap",
        "Pop/Funk",
        "Jungle",
        "Native American",
        "Cabaret",
        "New Wave",
        "Psychedelic",
        "Rave",
        "Showtunes",
        "Trailer",
        "Lo-Fi",
        "Tribal",
        "Acid Punk",
        "Acid Jazz",
        "Polka",
        "Retro",
        "Musical",
        "Rock & Roll",
        "Hard Rock",
        "Folk",
        "Folk-Rock",
        "National Folk",
        "Swing",
        "Fast Fusion",
        "Bebob",
        "Latin",
        "Revival",
        "Celtic",
        "Bluegrass",
        "Avantgarde",
        "Gothic Rock",
        "Progressive Rock",
        "Psychedelic Rock",
        "Symphonic Rock",
        "Slow Rock",
        "Big Band",
        "Chorus",
        "Easy Listening",
        "Acoustic",
        "Humour",
        "Speech",
        "Chanson",
        "Opera",
        "Chamber Music",
        "Sonata",
        "Symphony",
        "Booty Bass",
        "Primus",
        "Porn Groove",
        "Satire",
        "Slow Jam",
        "Club",
        "Tango",
        "Samba",
        "Folklore",
        "Ballad",
        "Power Ballad",
        "Rhythmic Soul",
        "Freestyle",
        "Duet",
        "Punk Rock",
        "Drum Solo",
        "A Cappella",
        "Euro-House",
        "Dance Hall",
        "Goa",
        "Drum & Bass",
        "Club-House",
        "Hardcore",
        "Terror",
        "Indie",
        "BritPop",
        "Negerpunk",
        "Polsk Punk",
        "Beat",
        "Christian Gangsta Rap",
        "Heavy Metal",
        "Black Metal",
        "Crossover",
        "Contemporary Christian",
        "Christian Rock",
        "Merengue",
        "Salsa",
        "Thrash Metal",
        "Anime",
        "JPop",
        "Synthpop",
        "Abstract",
        "Art Rock",
        "Baroque",
        "Bhangra",
        "Big Beat",
        "Breakbeat",
        "Chillout",
        "Downtempo",
        "Dub",
        "EBM",
        "Eclectic",
        "Electro",
        "Electroclash",
        "Emo",
        "Experimental",
        "Garage",
        "Global",
        "IDM",
        "Illbient",
        "Industro-Goth",
        "Jam Band",
        "Krautrock",
        "Leftfield",
        "Lounge",
        "Math Rock",
        "New Romantic",
        "Nu-Breakz",
        "Post-Punk",
        "Post-Rock",
        "Psytrance",
        "Shoegaze",
        "Space Rock",
        "Trop Rock",
        "World Music",
        "Neoclassical",
        "Audiobook",
        "Audio Theatre",
        "Neue Deutsche Welle",
        "Podcast",
        "Indie Rock",
        "G-Funk",
        "Dubstep",
        "Garage Rock",
        "Psybient",
    ];

    fn get_track_time(path: &Path) -> Duration {
        // rodio can never retrieve the duration for an .MP3 file, so we test this first.
        if let Some(extension) = path.extension()
            && extension == "mp3"
        {
            if let Ok(source) = mp3_duration::from_path(path) {
                return source;
            }
        }

        // use normal rodio method of retrieving duration.
        if let Ok(src) = std::fs::File::open(path)
            && let Ok(source) = rodio::Decoder::new(BufReader::new(src))
        {
            if let Some(duration) = source.total_duration() {
                return duration;
            }
        }

        Duration::default()
    }

    fn new(path: &DirEntry) -> Option<(String, String, Track)> {
        let path = path.path();

        // Open the media source.
        let src = std::fs::File::open(path).expect("failed to open media");

        // Create the media source stream.
        let mss = MediaSourceStream::new(Box::new(src), Default::default());

        // Create a probe hint using the file's extension. [Optional]
        let mut hint = Hint::new();
        if let Some(extension) = path.extension() {
            hint.with_extension(extension.to_str().unwrap());
        }

        // Use the default options for metadata and format readers.
        let meta_opts: MetadataOptions = Default::default();
        let fmt_opts: FormatOptions = Default::default();

        // Probe the media source.
        if let Ok(mut probed) =
            symphonia::default::get_probe().format(&hint, mss, &fmt_opts, &meta_opts)
        {
            let mut file_group = None;
            let mut file_album = None;
            let mut file_track = Track {
                name: path.to_str().unwrap().to_string(),
                path: path.to_str().unwrap().to_string(),
                date: None,
                kind: None,
                time: Self::get_track_time(path),
                icon: (None, None),
                track: None,
            };

            if let Some(revision) = probed.metadata.get().as_ref().and_then(|m| m.current()) {
                for tag in revision.tags() {
                    if let Some(key) = tag.std_key {
                        match key {
                            symphonia::core::meta::StandardTagKey::Artist => {
                                file_group = Some(tag.value.to_string());
                            }
                            symphonia::core::meta::StandardTagKey::Album => {
                                file_album = Some(tag.value.to_string());
                            }
                            symphonia::core::meta::StandardTagKey::Genre => {
                                let value = tag.value.to_string();

                                let mut split: Vec<&str> = value.split(&['(', ')']).collect();

                                // clear every empty entry.
                                split.retain(|x| !x.is_empty());

                                for entry in &mut split {
                                    // entry is a numerical ID3v1 genre.
                                    // https://en.wikipedia.org/wiki/List_of_ID3v1_genres
                                    if let Ok(index) = entry.parse::<usize>() {
                                        if let Some(genre) = Self::GENRE_LIST.get(index) {
                                            // genre index is within range.
                                            *entry = genre;
                                        } else {
                                            // unknown genre.
                                            *entry = "Unknown";
                                        }
                                    }
                                }

                                // join each genre together.
                                let value = split.join("|");

                                file_track.kind = Some(value);
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
                    let size = {
                        if let Some(size) = visual.dimensions {
                            Some((size.width, size.height))
                        } else if let Ok(image) = image::load_from_memory(&visual.data) {
                            Some((image.width(), image.height()))
                        } else {
                            None
                        }
                    };

                    file_track.icon = (Some(visual.data.to_vec()), size);
                }
            }

            return Some((
                file_group.unwrap_or_else(|| "< Unknown Group >".to_string()),
                file_album.unwrap_or_else(|| "< Unknown Album >".to_string()),
                file_track,
            ));
        }

        None
    }
}
