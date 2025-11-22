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
    name    = "last.fm Scrobble",
    info    = "Scrobble music and set your Now Playing status on last.fm.",
    from    = "luxreduxdelux",
    version = "1.0.0",
    setting = {
        log = {
            kind = "Button",
            data = "",
            name = "Connect",
            info = "Connect to last.fm. This is done automatically on launch, but if you haven't set your user data, you must connect after doing so.",
            call = "call_connect",
        },
        warn = {
            kind = "Toggle",
            data = true,
            name = "Warn on failure to connect to last.fm",
            info = "Send a warning toast notification on failure to connect to last.fm.",
            call = "call_warn"
        },
        user = {
            kind   = "Record",
            data   = "",
            name   = "User name",
            info   = "last.fm user-name.",
            censor = false,
            call   = "call_user",
        },
        pass = {
            kind   = "Record",
            data   = "",
            name   = "User pass",
            info   = "last.fm pass-word.",
            censor = true,
            call   = "call_pass",
        },
        key = {
            kind   = "Record",
            data   = "",
            name   = "API key",
            info   = "last.fm API key.",
            censor = true,
            call   = "call_key",
        },
        key_secret = {
            kind   = "Record",
            data   = "",
            name   = "API secret key",
            info   = "last.fm API secret key.",
            censor = true,
            call   = "call_key_secret",
        }
    }
}

function plug_in.begin(self)
    self.last_fm = require("melodix_last_fm")

    self.setting.user.data       = self.last_fm:get_user()
    self.setting.pass.data       = self.last_fm:get_pass()
    self.setting.key.data        = self.last_fm:get_key()
    self.setting.key_secret.data = self.last_fm:get_key_secret()
    self.library                 = melodix.get_library()
    self.scrobble                = false
    
    local error = self.last_fm:connect(self.setting.user.data, self.setting.pass.data, self.setting.key.data, self.setting.key_secret.data)

    if error then
        melodix.set_toast(2, "Could not connect to last.fm. (" .. error .. ")", 5.0)
    end
end

function plug_in.play(self, time)
    local group, album, track = melodix.get_state()

    -- If we set a previous play-back state...
    if self.previous_group and self.previous_album and self.previous_track then
        -- ...and the new play-back state is different from the previous one, set the scrobble state to false.
        if not (self.previous_group == group.name) or
           not (self.previous_album == album.name) or
           not (self.previous_track == track.name)
        then
            -- Set the currently playing state, and set the scrobble state to false.
            self.scrobble = false
        end
    end

    self.previous_group = group.name
    self.previous_album = album.name
    self.previous_track = track.name

    self.last_fm:state_play(group.name, album.name, track.name)
end

function plug_in.tick(self, time)
    local group, album, track = melodix.get_state()

    -- If we have a valid play-back state...
    if group and album and track then
        -- ...and we meet the time requirement for a scrobble and we haven't sent out a scrobble request yet...
        if not self.scrobble and track.time.secs >= 30.0 and (time >= 60.0 * 4.0 or time >= track.time.secs / 2.0) then
            -- Write the scrobble.
            self.scrobble = true
            self.last_fm:state_scrobble(group.name, album.name, track.name)
        end
    end
end

function plug_in.call_connect(self)
    local error = self.last_fm:connect(self.setting.user.data, self.setting.pass.data, self.setting.key.data, self.setting.key_secret.data)

    if error then
        melodix.set_toast(2, "Could not connect to last.fm. (" .. error .. ")", 5.0)
    end
end

function plug_in.call_warn(self)
    self.last_fm:set_warn(self.setting.warn.data)
end

function plug_in.call_user(self)
    self.last_fm:set_user(self.setting.user.data)
end

function plug_in.call_pass(self)
    self.last_fm:set_pass(self.setting.pass.data)
end

function plug_in.call_key(self)
    self.last_fm:set_key(self.setting.key.data)
end

function plug_in.call_key_secret(self)
    self.last_fm:set_key_secret(self.setting.key_secret.data)
end

return plug_in