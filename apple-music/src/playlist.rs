use crate::error::Error;
use crate::script_controller::{ParamType, ScriptController};
use crate::track::Track;
use serde::Deserialize;

/// Provides data related to a specific playlist as well as a list of Tracks (if fetched).
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
    /// The class of the item
    pub class: PlaylistClass,

    /// The id of the item
    pub id: i32,

    /// The index of the item in internal application order
    pub index: i32,

    /// The name of the item
    pub name: String,

    /// The id of the item as a hexadecimal string. This id does not change over time.
    #[serde(rename = "persistentID")]
    pub persistent_id: String,

    /// The description of the playlist
    pub description: Option<String>,

    /// Is this playlist disliked?
    pub disliked: bool,

    /// The total length of all tracks (in seconds)
    pub duration: Option<f32>,

    /// Is this playlist favorited?
    pub favorited: bool,

    /// Folder which contains this playlist (if any)
    pub parent: Option<Box<Playlist>>,

    /// The total size of all tracks (in bytes)
    pub size: Option<i64>,

    /// Special playlist kind
    pub special_kind: Option<SpecialKind>,

    /// The length of all tracks in MM:SS format
    pub time: Option<String>,

    /// Playlist's tracks
    pub tracks: Option<Vec<Track>>,

    /// Is this playlist visible in the Source list?
    pub visible: Option<bool>,
}

impl Playlist {
    /// Fetches Tracks in Playlist and populates `self.tracks`.
    /// WARNING: This fails when more than 900 tracks are fetched due to a JavaScript limit.
    pub fn fetch_playlist_tracks(&mut self) -> Result<(), Error> {
        match ScriptController.execute_script::<Vec<Track>>(
            ParamType::PlaylistTracks,
            Some(self.id),
            None,
        ) {
            Ok(data) => {
                self.tracks = Some(data);
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    /// Searches inside a Playlist for Tracks containing provided query and returns them.
    pub fn search_for_tracks(&self, query: &str) -> Result<Option<Vec<Track>>, Error> {
        match ScriptController.execute_script::<Vec<Track>>(
            ParamType::SearchInPlaylist,
            Some(self.id),
            Some(query),
        ) {
            Ok(data) => Ok(Option::from(data)),
            Err(err) => Err(err),
        }
    }

    /// Reveals and selects Playlist in Apple Music player.
    pub fn reveal_in_player(&self) -> Result<(), Error> {
        let cmd = format!(
            "Application('Music').reveal(Application('Music').playlists.byId({}))",
            self.id
        );

        let _ = ScriptController.execute(cmd.as_str(), None);

        Ok(())
    }

    /// Triggers a download on Apple Music Player for the Playlist.
    pub fn download(&self) -> Result<(), Error> {
        let cmd = format!(
            "Application('Music').download(Application('Music').playlists.byId({}))",
            self.id
        );

        let _ = ScriptController.execute(cmd.as_str(), None);

        Ok(())
    }
}

/// Kind of Special Playlist (Genius, Library, Purchases...).
#[derive(Deserialize, Debug)]
pub enum SpecialKind {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "folder")]
    Folder,
    Genius,
    Library,
    Music,
    PurchasedMusic,
}

/// Type of Playlist (User, Subscription, Folder...).
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum PlaylistClass {
    LibraryPlaylist,
    UserPlaylist,
    SubscriptionPlaylist,
    FolderPlaylist,
}
