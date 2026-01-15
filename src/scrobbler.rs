// Scrobbler implementations for Last.fm and ListenBrainz

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use listenbrainz::ListenBrainz;
use rustfm_scrobble_proxy::{Scrobble, Scrobbler as LastFmScrobbler};

pub mod lastfm_auth;

/// Represents a music track
#[derive(Debug, Clone)]
pub struct Track {
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub duration: Option<u64>,
}

/// Scrobbling service
pub enum Service {
    LastFm(LastFmScrobbler),
    ListenBrainz {
        name: String,
        client: ListenBrainz,
    },
}

impl Service {
    /// Create a Last.fm service
    pub fn lastfm(api_key: String, api_secret: String, session_key: String) -> Self {
        let mut scrobbler = LastFmScrobbler::new(&api_key, &api_secret);
        scrobbler.authenticate_with_session_key(&session_key);
        Self::LastFm(scrobbler)
    }

    /// Create a ListenBrainz service
    pub fn listenbrainz(name: String, token: String, api_url: String) -> Result<Self> {
        let mut client = if api_url == "https://api.listenbrainz.org" {
            ListenBrainz::new()
        } else {
            ListenBrainz::new_with_url(&api_url)
        };

        client
            .authenticate(&token)
            .with_context(|| format!("Failed to authenticate with ListenBrainz ({})", name))?;

        Ok(Self::ListenBrainz { name, client })
    }

    /// Submit a "now playing" update
    pub fn now_playing(&self, track: &Track) -> Result<()> {
        match self {
            Self::LastFm(scrobbler) => {
                let scrobble = Scrobble::new(&track.artist, &track.title, track.album.as_deref());
                scrobbler
                    .now_playing(&scrobble)
                    .context("Failed to update now playing on Last.fm")?;
                log::info!("Last.fm: Now playing updated");
            }
            Self::ListenBrainz { name, client } => {
                client
                    .playing_now(&track.artist, &track.title, track.album.as_deref())
                    .with_context(|| format!("Failed to update now playing on ListenBrainz ({})", name))?;
                log::info!("ListenBrainz ({}): Now playing updated", name);
            }
        }
        Ok(())
    }

    /// Scrobble a track
    pub fn scrobble(&self, track: &Track, timestamp: DateTime<Utc>) -> Result<()> {
        match self {
            Self::LastFm(scrobbler) => {
                let mut scrobble = Scrobble::new(&track.artist, &track.title, track.album.as_deref());
                scrobble.with_timestamp(timestamp.timestamp() as u64);
                scrobbler
                    .scrobble(&scrobble)
                    .context("Failed to scrobble to Last.fm")?;
                log::info!("Last.fm: Scrobbled successfully");
            }
            Self::ListenBrainz { name, client } => {
                // ListenBrainz uses current time for .listen(), so we use import() for historical timestamps
                // But for recent scrobbles we can just use listen()
                client
                    .listen(&track.artist, &track.title, track.album.as_deref())
                    .with_context(|| format!("Failed to scrobble to ListenBrainz ({})", name))?;
                log::info!("ListenBrainz ({}): Scrobbled successfully", name);
            }
        }
        Ok(())
    }
}
