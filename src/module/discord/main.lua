--[[
-- Copyright (c) 2025 luxreduxdelux
--
-- Redistribution and use in source and binary forms, with or without
-- modification, are permitted provided that the following conditions are met:
--
-- 1. Redistributions of source code must retain the above copyright notice,
-- this list of conditions and the following disclaimer.
--
-- 2. Redistributions in binary form must reproduce the above copyright notice,
-- this list of conditions and the following disclaimer in the documentation
-- and/or other materials provided with the distribution.
--
-- Subject to the terms and conditions of this license, each copyright holder
-- and contributor hereby grants to those receiving rights under this license
-- a perpetual, worldwide, non-exclusive, no-charge, royalty-free, irrevocable
-- (except for failure to satisfy the conditions of this license) patent license
-- to make, have made, use, offer to sell, sell, import, and otherwise transfer
-- this software, where such license applies only to those patent claims, already
-- acquired or hereafter acquired, licensable by such copyright holder or
-- contributor that are necessarily infringed by:
--
-- (a) their Contribution(s) (the licensed copyrights of copyright holders and
-- non-copyrightable additions of contributors, in source or binary form) alone;
-- or
--
-- (b) combination of their Contribution(s) with the work of authorship to which
-- such Contribution(s) was added by such copyright holder or contributor, if,
-- at the time the Contribution is added, such addition causes such combination
-- to be necessarily infringed. The patent license shall not apply to any other
-- combinations which include the Contribution.
--
-- Except as expressly stated above, no rights or licenses from any copyright
-- holder or contributor is granted under this license, whether expressly, by
-- implication, estoppel or otherwise.
--
-- DISCLAIMER
--
-- THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
-- AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
-- IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
-- DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDERS OR CONTRIBUTORS BE LIABLE
-- FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
-- DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
-- SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
-- CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
-- OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
-- OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
--]]

---@type plug_in
local plug_in = {
    name    = "Discord Rich Presence",
    info    = "Set your Discord Rich Presence status to the currently playing song, complete with cover art.",
    from    = "luxreduxdelux",
    version = "1.0.0",
    setting = {
        warn = {
            kind = "Toggle",
            data = true,
            name = "Warn on failure to connect to Discord",
            info = "Send a warning toast notification on failure to connect to Discord.",
            call = "call_warn"
        },
        cover_art = {
            kind = "Toggle",
            data = true,
            name = "Use MusicBrainz cover art",
            info = "Automatically use the cover art from MusicBrainz for the currently playing song.",
            call = "call_cover_art"
        }
    }
}

function plug_in.begin(self)
    self.discord                = require("melodix_discord")
    self.setting.cover_art.data = self.discord:get_cover_art()
    self.library                = melodix.get_library()
end

function plug_in.play(self, time)
    local group, album, track = melodix.get_state()

    if group and album and track then
        if not self.discord:state_play(group.name, album.name, track.name, time, track.time.secs) and self.setting.warn.data then
            melodix.set_toast(2, "Could not set Discord state.", 5.0)
        end
    end
end

function plug_in.stop(self)
    if not self.discord:state_stop() and self.setting.warn.data then
        melodix.set_toast(2, "Could not set Discord state.", 5.0)
    end
end

function plug_in.pause(self, time)
    local group, album, track = melodix.get_state()

    if group and album and track then
        if not self.discord:state_play(group.name, album.name, track.name, 0, 0) and self.setting.warn.data then
            melodix.set_toast(2, "Could not set Discord state.", 5.0)
        end
    end
end

function plug_in.call_warn(self)
    self.discord:set_warn(self.setting.warn.data)
end

function plug_in.call_cover_art(self)
    self.discord:set_cover_art(self.setting.cover_art.data)
end

return plug_in
