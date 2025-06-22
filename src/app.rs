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

use crate::library::*;
use crate::script::*;
use crate::setting::*;
use crate::system::*;
use crate::window::*;

//================================================================

use eframe::CreationContext;
use eframe::egui;
use std::io::BufReader;
use std::time::Duration;

//================================================================

// TO-DO minimize/hide window, tray icon, welcome menu, configuration menu, plug-in menu, plug-in setting data, add Group/Album/Track header

pub struct App {
    pub library: Library,
    pub setting: Setting,
    pub window: Window,
    pub script: Script,
    pub system: System,
}

impl App {
    // get the currently active group, album, and track.
    pub fn get_play_state(&self) -> (&Group, &Album, &Track) {
        let group = self
            .library
            .list_group
            .get(self.window.state.as_ref().unwrap().0)
            .unwrap();
        let album = group
            .list_album
            .get(self.window.state.as_ref().unwrap().1)
            .unwrap();
        let track = album
            .list_track
            .get(self.window.state.as_ref().unwrap().2)
            .unwrap();

        (group, album, track)
    }

    pub fn track_add(
        &mut self,
        track: (usize, usize, usize),
        context: &egui::Context,
    ) -> anyhow::Result<()> {
        self.window.state = Some((track.0, track.1, track.2));

        let (_, _, track) = self.get_play_state();

        let file = std::fs::File::open(&track.path)?;
        let file = rodio::Decoder::new(BufReader::new(file))?;

        let state = self.window.state.unwrap();
        self.window.state = Some((state.0, state.1, state.2));

        self.system.sink.stop();
        self.system.sink.append(file);
        self.system.sink.play();

        ScriptData::set_state(self);
        ScriptData::set_queue(self);

        self.script
            .call(Script::CALL_PLAY, self.system.sink.get_pos().as_secs());

        Ok(())
    }

    pub fn track_toggle(&self) {
        if self.system.sink.is_paused() {
            self.system.sink.play();

            self.script
                .call(Script::CALL_PLAY, self.system.sink.get_pos().as_secs());
        } else {
            self.system.sink.pause();

            self.script
                .call(Script::CALL_PAUSE, self.system.sink.get_pos().as_secs());
        }
    }

    pub fn track_seek(&self, seek: i64, delta: bool) {
        let seek = {
            if delta {
                seek + self.system.sink.get_pos().as_secs() as i64
            } else {
                seek
            }
        };

        let _ = self.system.sink.try_seek(Duration::from_secs(seek as u64));

        self.script.call(Script::CALL_SEEK, seek);
    }

    pub fn track_play(&self) {
        self.system.sink.play();

        self.script.call(Script::CALL_PLAY, ());
    }

    pub fn track_pause(&self) {
        self.system.sink.pause();

        self.script.call(Script::CALL_PAUSE, ());
    }

    pub fn track_set_volume(&self, volume: f32) {
        self.system.sink.set_volume(volume);

        // TO-DO does there need to be a volume call-back?
    }

    pub fn track_stop(&mut self) {
        self.window.state = None;
        self.system.sink.stop();

        self.script.call(Script::CALL_STOP, ());
    }

    pub fn track_skip_a(&mut self, context: &egui::Context) -> anyhow::Result<()> {
        if self.window.queue.1 > 0 {
            if let Some(track) = self.window.queue.0.get(self.window.queue.1 - 1) {
                self.window.queue.1 -= 1;
                self.track_add(*track, context)?
            }
        }

        self.script.call(Script::CALL_SKIP_A, ());

        Ok(())
    }

    pub fn track_skip_b(&mut self, context: &egui::Context) -> anyhow::Result<()> {
        if let Some(track) = self.window.queue.0.get(self.window.queue.1 + 1) {
            self.window.queue.1 += 1;
            self.track_add(*track, context)?
        }

        self.script.call(Script::CALL_SKIP_B, ());

        Ok(())
    }

    pub fn error(message: &str) {
        rfd::MessageDialog::new()
            .set_level(rfd::MessageLevel::Error)
            .set_title("Error")
            .set_description(message)
            .show();
    }

    pub fn new(context: &CreationContext) -> Self {
        let library = Library::new();
        let setting = Setting::new(context);

        Self {
            script: Script::new(&library, &setting),
            window: Window::new(&library),
            system: System::new(context),
            library,
            setting,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, context: &egui::Context, _: &mut eframe::Frame) {
        if let Some(event) = self.system.poll_event() {
            if let Err(error) = System::make_event(event, self, context) {
                Self::error(&error.to_string());
            }
        }

        if let Err(error) = Window::draw(self, context) {
            Self::error(&error.to_string());
        }
    }
}
