---@meta

---The Melodix API.
---@class melodix
melodix = {}

---Library class.
---@class library
---@field list_group table # A table array of each group.
library = {}

---Group class.
---@class group
---@field name       string # Group name.
---@field list_album table  # Album list. A table array of each album.
group = {}

---Album class.
---@class album
---@field name       string       # Album name.
---@field icon       string | nil # Album icon. Absolute path to the album cover; *may* be nil.
---@field list_track table        # Track list. A table array of each track.
album = {}

---Track class.
---@class track
---@field name   string       # Track name.
---@field path   string       # Track path. Absolute path to the track file.
---@field time   time         # Track time.
---@field date   string | nil # Track date. *May* be nil.
---@field kind   string | nil # Track kind. *May* be nil.
---@field icon   table        # Track icon. A table where the first element *may* be the album cover data and the second element *may* be the dimension of the album cover, if they are present in the meta-data.
---@field track  number | nil # Track number. *May* be nil.
track = {}

---Time class.
---@class time
---@field secs number # Time in seconds.

---Toast notification kind.
---@enum toast_kind
TOAST_KIND = {
    INFO    = 0,
    WARNING = 1,
    FAILURE = 2,
    SUCCESS = 3,
}

---Get the library.
---@return library library # Library.
function melodix.get_library() end

---Get the currently playing group, album and track data.
---@return group | nil group # Current group. *May* be nil.
---@return album | nil album # Current album. *May* be nil.
---@return track | nil track # Current track. *May* be nil.
function melodix.get_state() end

---Get the currently playing group, album and track data.
---@return table  queue # A table array containing a table where the first element is the group index, the second element is the album index, and the last index is the track index.
---@return number index # Index into the queue as the current entry.
function melodix.get_queue() end

---Get the currently playing group, album and track data.
---@param kind toast_kind # Toast kind.
---@param text string     # Toast text.
---@param time number     # Toast time.
function melodix.set_toast(kind, text, time) end
