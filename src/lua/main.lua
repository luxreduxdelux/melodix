-- Refer to the meta.lua file for more documentation.

---@type plug_in
local plug_in = {
    name    = "Sample Lua Module",
    info    = "Sample.",
    from    = "luxreduxdelux",
    version = "1.0.0",
    setting = {
        button = {
            kind = "Button",
            name = "Sample Button",
            info = "A sample button.",
            call = "call_button"
        },
        toggle = {
            kind = "Toggle",
            data = true,
            name = "Sample Toggle",
            info = "A sample toggle.",
            -- NOTE: This is optional and may be absent.
            call = "call_toggle"
        },
        slider = {
            kind = "Slider",
            data = 0.0,
            name = "Sample Slider",
            info = "A sample slider.",
            bind = { -1.0, 1.0 },
            -- NOTE: This is optional and may be absent.
            call = "call_slider"
        },
        record = {
            kind   = "Record",
            data   = "foo",
            name   = "Sample Record",
            info   = "A sample recorder.",
            censor = false,
            -- NOTE: This is optional and may be absent.
            call   = "call_record"
        }
    }
}

function plug_in.call_button(self)
    print("Button call-back.")
end

function plug_in.call_toggle(self)
    print("Toggle call-back. Data: " .. tostring(self.setting.toggle.data))
end

function plug_in.call_slider(self)
    print("Slider call-back. Data: " .. tostring(self.setting.slider.data))
end

function plug_in.call_record(self)
    print("Record call-back. Data: " .. self.setting.record.data)
end

function plug_in.begin(self)
    print("Module initialization.")
end

function plug_in.close(self)
    print("Module de-initialization.")
end

function plug_in.tick(self)
    -- This method will be ran on every render frame. A render frame will happen after 1 second, or on user input.
end

function plug_in.seek(self, time)
    print("Seek call-back. Seeking to: " .. tostring(time))
end

function plug_in.play(self, time)
    local group, album, track = melodix.get_state()

    print("Play call-back.")
    print("* Group: " .. group.name)
    print("* Album: " .. album.name)
    print("* Track: " .. track.name)
    print("* Time: " .. tostring(time))
end

function plug_in.stop(self)
    print("Stop call-back.")
end

function plug_in.skip_a(self)
    print("Skip - call-back.")
end

function plug_in.skip_b(self)
    print("Skip + call-back.")
end

function plug_in.pause(self, time)
    local group, album, track = melodix.get_state()

    print("Pause call-back.")
    print("* Group: " .. group.name)
    print("* Album: " .. album.name)
    print("* Track: " .. track.name)
    print("* Time: " .. tostring(time))
end

return plug_in
