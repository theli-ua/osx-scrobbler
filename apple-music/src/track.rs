use crate::error::Error;
use crate::script_controller::{ParamType, ScriptController};
use serde::Deserialize;
use std::io::Read;
use urlencoding::encode;

/// Provides data related to a specific Track as well as its artworks.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    /// The class of the Track
    pub class: TrackKind,

    /// The id of the item
    pub id: i32,

    /// The index of the item in internal application order
    pub index: i32,

    /// The name of the item
    pub name: String,

    /// The id of the item as a hexadecimal string. This id does not change over time.
    #[serde(rename = "persistentID")]
    pub persistent_id: String,

    /// The album name of the track
    pub album: String,

    /// The album artist of the track
    pub album_artist: String,

    /// Is the album for this track disliked?
    pub album_disliked: bool,

    /// Is the album for this track favorited?
    pub album_favorited: bool,

    /// The rating of the album for this track (0 to 100)
    pub album_rating: Option<i16>,

    /// The rating kind of the album rating for this track
    pub album_rating_kind: Option<Kind>,

    /// The raw data of the artworks attached to this track.
    pub artworks_raw_data: Option<Vec<Artwork>>,

    /// The URL of the main artwork for this track.
    artwork_url: Option<String>,

    /// The artist of the track
    pub artist: String,

    /// The bit rate of the track (in kbps)
    pub bit_rate: Option<i16>,

    /// The bookmark time of the track in seconds
    pub bookmark: i8,

    /// Is the playback position for this track remembered?
    pub bookmarkable: bool,

    /// The tempo of this track in beats per minute
    pub bpm: i16,

    /// The category of the track
    pub category: String,

    /// The iCloud status of the track
    pub cloud_status: Option<CloudStatus>,

    /// Freeform notes about the track
    pub comment: String,

    /// Is this track from a compilation album?
    pub compilation: bool,

    /// The composer of the track
    pub composer: String,

    /// The common, unique ID for this track. If two tracks in different playlists have the same database ID, they are sharing the same data.
    #[serde(rename = "databaseID")]
    pub database_id: i32,

    /// The date the track was added to the playlist
    pub date_added: String,

    /// The description of the track
    pub description: String,

    /// Is this track disliked?
    pub disliked: bool,

    /// The Apple ID of the person who downloaded this track
    #[serde(rename = "downloaderAppleID")]
    pub downloader_apple_id: Option<String>,

    /// The name of the person who downloaded this track
    pub downloader_name: Option<String>,

    /// The length of the track in seconds
    pub duration: f64,

    /// Is this track checked for playback?
    pub enabled: bool,

    /// The episode ID of the track
    #[serde(rename = "episodeID")]
    pub episode_id: Option<String>,

    /// The episode number of the track
    pub episode_number: i16,

    /// The name of the EQ preset of the track
    pub eq: String,

    /// The stop time of the track in seconds
    pub finish: f64,

    /// Is this track from a gapless album?
    pub gapless: Option<bool>,

    /// The music/audio genre (category) of the track
    pub genre: String,

    /// The grouping (piece) of the track. Generally used to denote movements within a classical work.
    pub grouping: String,

    /// A text description of the track
    pub kind: Option<String>,

    /// The long description of the track
    pub long_description: Option<String>,

    /// Is this track favorited?
    pub favorited: bool,

    /// The lyrics of the track
    pub lyrics: Option<String>,

    /// The media kind of the track
    pub media_kind: MediaKind,

    /// The modification date of the content of this track
    pub modification_date: Option<String>,

    /// The movement name of the track
    pub movement: Option<String>,

    /// The total number of movements in the work
    pub movement_count: i16,

    /// The index of the movement in the work
    pub movement_number: i16,

    /// Number of times this track has been played
    pub played_count: i16,

    /// The date and time this track was last played
    pub played_date: Option<String>,

    /// The Apple ID of the person who purchased this track
    #[serde(rename = "purchaserAppleID")]
    pub purchaser_apple_id: Option<String>,

    /// The name of the person who purchased this track
    pub purchaser_name: Option<String>,

    /// The rating of this track (0 to 100)
    pub rating: i16,

    /// The rating kind of this track
    pub rating_kind: Option<Kind>,

    /// The release date of this track
    pub release_date: Option<String>,

    /// The sample rate of the track (in Hz)
    pub sample_rate: Option<i32>,

    /// The season number of the track
    pub season_number: Option<i16>,

    /// Is this track included when shuffling?
    pub shufflable: bool,

    /// Number of times this track has been skipped
    pub skipped_count: i16,

    /// The date and time this track was last skipped
    pub skipped_date: Option<String>,

    /// The show name of the track
    pub show: Option<String>,

    /// Override string to use for the track when sorting by album
    pub sort_album: Option<String>,

    /// Override string to use for the track when sorting by artist
    pub sort_artist: Option<String>,

    /// Override string to use for the track when sorting by album artist
    pub sort_album_artist: Option<String>,

    /// Override string to use for the track when sorting by name
    pub sort_name: Option<String>,

    /// Override string to use for the track when sorting by composer
    pub sort_composer: Option<String>,

    /// Override string to use for the track when sorting by show name
    pub sort_show: Option<String>,

    /// The size of the track (in bytes)
    pub size: Option<i64>,

    /// The start time of the track in seconds
    pub start: f64,

    /// The length of the track in MM:SS format
    pub time: String,

    /// The total number of tracks on the source album
    pub track_count: i16,

    /// The Apple Music URL for this Track.
    track_url: Option<String>,

    /// The index of the track on the source album
    pub track_number: i16,

    /// Is this track unplayed?
    pub unplayed: bool,

    /// Relative volume adjustment of the track (-100% to 100%)
    pub volume_adjustment: i16,

    /// The work name of the track
    pub work: Option<String>,

    /// The year the track was recorded/released
    pub year: i16,
}

impl Track {
    /// Method that either returns an already fetched artwork_url, or fetches it and then returns it.
    pub fn artwork_url(&mut self) -> &Option<String> {
        if self.artwork_url == None {
            self.fetch_itunes_store_data()
        }

        return &self.artwork_url;
    }

    /// Method that either returns an already fetched track_url, or fetches it and then returns it.
    pub fn track_url(&mut self) -> &Option<String> {
        if self.track_url == None {
            self.fetch_itunes_store_data()
        }

        return &self.track_url;
    }

    /// Returns a list of all artworks with their raw_data.
    /// Recommended to use Track.get_artwork_url() instead.
    pub fn fetch_artworks_raw_data(&mut self) -> Result<(), Error> {
        match ScriptController.execute_script::<Vec<Artwork>>(
            ParamType::Artworks,
            Some(self.id),
            None,
        ) {
            Ok(data) => {
                self.artworks_raw_data = Some(data);
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    /// Reveals and selects Track in Apple Music.
    pub fn reveal_in_player(&self) -> Result<(), Error> {
        let cmd = format!(
            "Application('Music').reveal(Application('Music').tracks.byId({}))",
            self.id
        );

        let _ = ScriptController.execute(cmd.as_str(), None);

        Ok(())
    }

    /// Triggers a download on Apple Music Player for the Track.
    pub fn download(&self) -> Result<(), Error> {
        let cmd = format!(
            "Application('Music').download(Application('Music').tracks.byId({}))",
            self.id
        );

        let _ = ScriptController.execute(cmd.as_str(), None);

        Ok(())
    }

    /// Favorites / "Unfavorites" a Track.
    pub fn set_favorited(&self, value: bool) -> Result<(), Error> {
        let cmd = format!(
            "Application('Music').tracks.byId({}).favorited = {}",
            self.id, value
        );

        let _ = ScriptController.execute(cmd.as_str(), None);

        Ok(())
    }

    /// Dislikes / "Undislikes" a Track.
    pub fn set_disliked(&self, value: bool) -> Result<(), Error> {
        let cmd = format!(
            "Application('Music').tracks.byId({}).disliked = {}",
            self.id, value
        );

        let _ = ScriptController.execute(cmd.as_str(), None);

        Ok(())
    }

    /// Search for a song in the Itunes Store and extract its artwork_url & track_url.
    fn fetch_itunes_store_data(&mut self) {
        let request = format!(
            "https://itunes.apple.com/search?term={}&entity=song&limit=200",
            encode(self.name.as_str())
        );
        self.fetch_itunes_store_by_request(request);

        if self.artwork_url == None {
            let request = format!(
                "https://itunes.apple.com/search?term={}&entity=song&attribute=albumTerm&limit=200",
                encode(self.album.as_str())
            );
            self.fetch_itunes_store_by_request(request);
        }

        if self.artwork_url == None {
            let request = format!(
                "https://itunes.apple.com/search?term={}&entity=song&limit=200",
                encode(self.artist.as_str())
            );
            self.fetch_itunes_store_by_request(request);
        }
    }

    fn fetch_itunes_store_by_request(&mut self, request: String) {
        let mut res = reqwest::blocking::get(request).unwrap();
        let mut body = String::new();
        res.read_to_string(&mut body).unwrap();

        if let Ok(search) = serde_json::from_str::<ITunesStoreSearch>(body.as_str()) {
            if search.result_count == 1 {
                self.artwork_url = Some(search.results[0].clone().artwork_url_100);
                self.track_url = Some(search.results[0].clone().track_view_url);
            } else {
                let result = search.results.iter().find(|result| {
                    (&result.track_name.to_lowercase() == &self.name.to_lowercase()
                        || &result.track_censored_name.to_lowercase() == &self.name.to_lowercase())
                        && (&result.artist_name.to_lowercase() == &self.artist.to_lowercase()
                            || &result.collection_name.to_lowercase() == &self.album.to_lowercase())
                });

                match result {
                    Some(data) => {
                        self.artwork_url = Some(data.clone().artwork_url_100);
                        self.track_url = Some(data.clone().track_view_url);
                    }
                    None => (),
                }
            }
        }
    }
}

/// Data for a given Artwork.
#[derive(Deserialize, Debug)]
pub struct Artwork {
    /// The class of the item.
    pub class: String,

    /// Data for the artwork, in the form of a picture.
    pub data: Option<String>,

    /// Description of artwork as a string.
    pub description: Option<String>,

    /// Was this artwork downloaded by Apple Music ?
    pub downloaded: bool,

    /// The data format for this piece of artwork.
    pub format: Option<String>,

    /// Kind or purpose of this piece of artwork.
    pub kind: i32,

    /// Data for this artwork, in original format.
    pub raw_data: String,
}

/// Struct representing a search through the Itunes Store.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ITunesStoreSearch {
    pub result_count: i32,
    pub results: Vec<ITunesStoreData>,
}

/// Struct representing an object from the Itunes Store.
#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ITunesStoreData {
    /// The Artist of the Track
    pub artist_name: String,

    /// The censored name of the Track.
    pub track_censored_name: String,

    /// The name of the Track.
    pub track_name: String,

    /// The URL of the main artwork for this track.
    pub artwork_url_100: String,

    /// The Apple Music URL this track.
    pub track_view_url: String,

    /// The Album name of this Track.
    pub collection_name: String,
}

/// Type of Rating: User-made or Computed.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    User,
    Computed,
}

/// iCloud status for Track.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum CloudStatus {
    Unknown,
    Purchased,
    Matched,
    Uploaded,
    Ineligible,
    Removed,
    Error,
    Duplicate,
    Subscription,
    Prerelease,

    #[serde(rename = "no longer available")]
    NoLongerAvailable,
    NotUploaded,
}

/// Type of Media: Song, MusicVideo or Unknown.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum MediaKind {
    Song,
    #[serde(rename = "music video")]
    MusicVideo,
    Unknown,
}

/// Type of Track: From an URL, a File, or Shared.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum TrackKind {
    SharedTrack,
    FileTrack,
    UrlTrack,
}
