use crate::playlist::Playlist;
use crate::track::Track;
use serde::Deserialize;

/// Contains data related to the Apple Music player, as well as a list of user's Playlists.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApplicationData {
    /// Is AirPlay currently enabled?
    pub airplay_enabled: Option<bool>,

    /// Is a track currently being converted?
    pub converting: Option<bool>,

    /// The currently selected AirPlay device(s)
    pub current_airplay_devices: Vec<AirplayDevice>,

    /// The currently selected encoder (MP3, AIFF, WAV, etc.)
    pub current_encoder: Encoder,

    /// The playlist containing the currently targeted track
    pub current_playlist: Option<Playlist>,

    /// The name of the current track in the playing stream (provided by streaming server)
    pub current_stream_title: Option<String>,

    /// The URL of the playing stream or streaming web site (provided by streaming server)
    pub current_stream_url: Option<String>,

    /// The currently selected visual plug-in
    pub current_visual: Visual,

    /// Is the equalizer enabled?
    pub eq_enabled: bool,

    /// True if all AppleScript track indices should be independent of the play order of the owning playlist.
    pub fixed_indexing: bool,

    /// Is this the active application?
    pub frontmost: bool,

    /// Is the application using the entire screen?
    pub full_screen: bool,

    /// The name of the application
    pub name: Option<String>,

    /// Has the sound output been muted?
    pub mute: bool,

    /// : the player’s position within the currently playing track in seconds.
    pub player_position: Option<f64>,

    /// Is the player stopped, paused, or playing?
    pub player_state: Option<PlayerState>,

    /// List of all user playlists
    pub playlists: Option<Vec<Playlist>>,

    /// Track selected on the app by the user
    pub selection: Option<Vec<Track>>,

    /// Are songs played in random order?
    pub shuffle_enabled: bool,

    /// The playback shuffle mode
    pub shuffle_mode: ShuffleMode,

    /// The playback repeat mode
    pub song_repeat: SongRepeat,

    /// The sound output volume (0 = minimum, 100 = maximum)
    pub sound_volume: i8,

    /// The version of the application
    pub version: Option<String>,

    /// List of visuals
    pub visuals: Vec<Visual>,

    /// Are visuals currently being displayed?
    pub visuals_enabled: bool,
}

/// Information about devices connected via AirPlay.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AirplayDevice {
    /// The class of the item
    pub class: String,

    /// The id of the item
    pub id: i32,

    /// The index of the item in internal application order
    pub index: i32,

    /// The name of the item
    pub name: String,

    /// The id of the item as a hexadecimal string. This id does not change over time.
    #[serde(rename = "persistentID")]
    pub persistent_id: String,

    /// Is the device currently being played to?
    pub active: Option<bool>,

    /// Is the device currently available?
    pub available: Option<bool>,

    /// The kind of the device
    pub kind: AirplayDeviceKind,

    /// The network (MAC) address of the device
    pub network_address: Option<String>,

    /// Is the device password- or passcode-protected?
    pub protected: Option<bool>,

    /// Is the device currently selected?
    pub selected: bool,

    /// Does the device support audio playback?
    pub supports_audio: Option<bool>,

    /// Does the device support video playback?
    pub supports_video: Option<bool>,

    /// The output volume for the device (0 = minimum, 100 = maximum)
    pub sound_volume: i8,
}

/// Information about a given Encoder.
#[derive(Deserialize, Debug)]
pub struct Encoder {
    /// The class of the item
    pub class: String,

    /// The id of the item
    pub id: i32,

    /// The index of the item in internal application order
    pub index: i32,

    /// The name of the item
    pub name: String,

    /// The data format created by the encoder
    pub format: Option<String>,
}

/// Information about a given Equalizer Preset.
#[derive(Deserialize, Debug)]
pub struct EqPreset {
    /// The class of the item
    pub class: String,

    /// The id of the item
    pub id: i32,

    /// The index of the item in internal application order
    pub index: i32,

    /// The name of the item
    pub name: String,

    /// The equalizer 32 Hz band level (-12.0 dB to +12.0 dB)
    pub band1: f32,

    /// The equalizer 32 Hz band level (-12.0 dB to +12.0 dB)
    pub band2: f32,

    /// The equalizer 32 Hz band level (-12.0 dB to +12.0 dB)
    pub band3: f32,

    /// The equalizer 32 Hz band level (-12.0 dB to +12.0 dB)
    pub band4: f32,

    /// The equalizer 32 Hz band level (-12.0 dB to +12.0 dB)
    pub band5: f32,

    /// The equalizer 32 Hz band level (-12.0 dB to +12.0 dB)
    pub band6: f32,

    /// The equalizer 32 Hz band level (-12.0 dB to +12.0 dB)
    pub band7: f32,

    /// The equalizer 32 Hz band level (-12.0 dB to +12.0 dB)
    pub band8: f32,

    /// The equalizer 32 Hz band level (-12.0 dB to +12.0 dB)
    pub band9: f32,

    /// The equalizer 32 Hz band level (-12.0 dB to +12.0 dB)
    pub band10: f32,

    /// Can this preset be modified?
    pub modifiable: Option<bool>,

    /// The equalizer 32 Hz band level (-12.0 dB to +12.0 dB)
    pub preamp: f32,

    /// Should tracks which refer to this preset be updated when the preset is renamed or deleted?
    pub update_tracks: bool,
}

/// Information about a given Visual.
#[derive(Deserialize, Debug)]
pub struct Visual {
    /// The class of the item
    pub class: String,

    /// The id of the item
    pub id: i32,

    /// The index of the item in internal application order
    pub index: i32,

    /// The name of the item
    pub name: String,
}

/// Current State of Player.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum PlayerState {
    Stopped,
    Playing,
    Paused,
    FastForwarding,
    Rewinding,
}

/// Type of Shuffle (Songs, Albums, Groupings).
#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ShuffleMode {
    Songs,
    Albums,
    Groupings,
}

/// Type of Song Repeat (Off, One, All).
#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum SongRepeat {
    Off,
    One,
    All,
}

/// Type of Airplay Device (AppleTV, Bluetooth, Computer ...).
#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum AirplayDeviceKind {
    Computer,
    AirportExpress,
    AppleTV,
    #[serde(rename = "Airplay device")]
    AirplayDevice,
    #[serde(rename = "Bluetooth device")]
    BluetoothDevice,
    HomePod,
    Unknown,
}
