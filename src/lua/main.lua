---@type plug_in
local plug_in = {
    name    = "Sample Lua Module",
    info    = "Sample.",
    from    = "luxreduxdelux",
    version = "1.0.0",
    setting = {
        setting_button = {
            kind = "Button",
            data = true,
            name = "Toggle setting",
            info = "Toggle setting.",
            call = "call_setting_button"
        },
        setting_toggle = {
            kind = "Toggle",
            data = true,
            name = "Toggle setting",
            info = "Toggle setting.",
            call = "call_setting_toggle"
        },
        setting_toggle = {
            kind = "Toggle",
            data = true,
            name = "Toggle setting",
            info = "Toggle setting.",
            call = "call_setting_toggle"
        }
    },
    group   = {
        search = {
            kind = "Button",
            name = "Group Button",
            info = "Group Button",
            call = "call_group"
        }
    },
    album   = {
        search = {
            kind = "Button",
            name = "Album Button",
            info = "Album Button",
            call = "call_album"
        }
    },
    track   = {
        search = {
            kind = "Button",
            name = "Track Button",
            info = "Track Button",
            call = "call_track"
        }
    },
    queue   = {
        search = {
            kind = "Button",
            name = "Queue Button",
            info = "Queue Button",
            call = "call_queue"
        }
    }
}

function plug_in.begin(self)
    self.discord                = require("melodix_discord")
    self.setting.cover_art.data = self.discord:get_cover_art()
    self.library                = melodix.get_library()
end

function plug_in.stop(self)
    if not self.discord:state_stop() then
        melodix.set_toast(2, "Could not set Discord state.", 5.0)
    end
end

function plug_in.play(self, time)
    local list, position = melodix.get_queue()

    local group = self.library.list_group[list[1][1]]
    local album = group.list_album[list[1][2]]
    local track = album.list_track[list[1][3]]

    print(group.name)
    print(album.name)
    print(track.name)
    print(position)

    local group, album, track = melodix.get_state()

    if group and album and track then
        if not self.discord:state_play(group.name, album.name, track.name, time, track.time.secs) then
            melodix.set_toast(2, "Could not set Discord state.", 5.0)
        end
    end
end

function plug_in.pause(self, time)
    local group, album, track = melodix.get_state()

    if group and album and track then
        if not self.discord:state_play(group.name, album.name, track.name, 0, 0) then
            melodix.set_toast(2, "Could not set Discord state.", 5.0)
        end
    end
end

function plug_in.call_cover_art(self)
    self.discord:set_cover_art(self.setting.cover_art.data)
end

function plug_in.call_queue(self, group, album, track)
    print("call_queue")

    local state = melodix.get_state()
    local group = self.library.list_group[group + 1]
    local album = group.list_album[album + 1]
    local track = album.list_track[track + 1]

    print(group.name, album.name, track.name)
end

return plug_in
