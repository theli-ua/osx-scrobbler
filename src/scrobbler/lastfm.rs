// Last.fm scrobbler implementation

use super::traits::{Scrobbler, Track};
use anyhow::Result;

pub struct LastFmScrobbler {
    api_key: String,
    api_secret: String,
    session_key: String,
}

impl LastFmScrobbler {
    pub fn new(api_key: String, api_secret: String, session_key: String) -> Self {
        Self {
            api_key,
            api_secret,
            session_key,
        }
    }
}

impl Scrobbler for LastFmScrobbler {
    async fn now_playing(&self, _track: &Track) -> Result<()> {
        // TODO: Implement Last.fm now playing
        Ok(())
    }

    async fn scrobble(&self, _track: &Track, _timestamp: i64) -> Result<()> {
        // TODO: Implement Last.fm scrobble
        Ok(())
    }
}
