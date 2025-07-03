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

use crate::{library::*, script::*, setting::*, system::*, window::*};

//================================================================

use eframe::{CreationContext, egui};
use std::{io::BufReader, time::Duration};

//================================================================

pub struct App {
    pub library: Library,
    pub setting: Setting,
    pub window: Window,
    pub script: Script,
    pub system: System,
}

impl App {
    pub fn new(context: &CreationContext) -> anyhow::Result<Self> {
        let library = Library::new();
        let setting = Setting::new(context);
        let window = Window::new(&library);

        Ok(Self {
            script: Script::new(&setting)?,
            system: System::new(&setting, context)?,
            window,
            library,
            setting,
        })
    }

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
        context.forget_all_images();

        // set active window track state.
        self.window.state = Some((track.0, track.1, track.2));
        // get group, album, track data from window state.
        let (group, album, track) = self.get_play_state();

        let file = rodio::Decoder::new(BufReader::new(std::fs::File::open(&track.path)?))?;

        // send push notification.
        self.system
            .push_notification(context, (group, album, track))?;

        // kill the current track, add new track.
        self.system.sink.stop();
        self.system.sink.append(file);

        // append call-back for when the track is over.
        let clone = context.clone();
        self.system
            .sink
            .append(rodio::source::EmptyCallback::<f32>::new(Box::new(
                move || {
                    clone.request_repaint();
                },
            )));

        self.system.sink.play();

        //self.script
        //    .call(Script::CALL_PLAY, self.system.sink.get_pos().as_secs());

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
                self.track_add(*track, context)?;
                self.script.call(Script::CALL_SKIP_A, ());
            } else {
                self.track_stop();
            }
        }

        Ok(())
    }

    pub fn track_skip_b(&mut self, context: &egui::Context) -> anyhow::Result<()> {
        if let Some(track) = self.window.queue.0.get(self.window.queue.1 + 1) {
            self.window.queue.1 += 1;
            self.track_add(*track, context)?;
            self.script.call(Script::CALL_SKIP_B, ());
        } else {
            self.track_stop();
        }

        Ok(())
    }

    pub fn error(message: &str) {
        rfd::MessageDialog::new()
            .set_level(rfd::MessageLevel::Error)
            .set_title("Error")
            .set_description(message)
            .show();
    }

    pub fn error_result(result: anyhow::Result<()>) {
        if let Err(error) = result {
            rfd::MessageDialog::new()
                .set_level(rfd::MessageLevel::Error)
                .set_title("Error")
                .set_description(error.to_string())
                .show();
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
