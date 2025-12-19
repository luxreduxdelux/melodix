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

//================================================================

use eframe::{CreationContext, egui};
use serde::{Deserialize, Serialize};

//================================================================

#[derive(Clone, Serialize, Deserialize)]
pub struct Setting {
    pub window_scale: f32,
    pub window_theme: bool,
    pub window_media: bool,
    pub window_tray: bool,
    pub window_push: bool,
    pub window_time: bool,
    pub window_date: bool,
    pub window_kind: bool,
    pub window_track: bool,
    pub script_allow: bool,
}

impl Setting {
    const PATH_SETTING: &'static str = "setting.data";

    pub fn new(context: &CreationContext) -> Self {
        if let Ok(file) = std::fs::read(App::get_configuration_path(Self::PATH_SETTING, false)) {
            if let Ok(setting) = postcard::from_bytes::<Self>(&file) {
                context.egui_ctx.set_zoom_factor(setting.window_scale);

                if setting.window_theme {
                    context.egui_ctx.set_theme(egui::Theme::Light);
                }

                return setting;
            }
        }

        Self::default()
    }
}

impl Default for Setting {
    fn default() -> Self {
        Self {
            window_scale: 1.0,
            window_theme: false,
            window_media: true,
            window_tray: true,
            window_push: true,
            window_time: true,
            window_date: true,
            window_kind: true,
            window_track: true,
            script_allow: true,
        }
    }
}

impl Drop for Setting {
    fn drop(&mut self) {
        let serialize: Vec<u8> = postcard::to_allocvec(&self).unwrap();
        std::fs::write(App::get_configuration_path(Self::PATH_SETTING, false), serialize).unwrap();
    }
}
