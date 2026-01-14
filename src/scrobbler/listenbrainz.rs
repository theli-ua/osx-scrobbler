// ListenBrainz scrobbler implementation

use super::traits::{Scrobbler, Track};
use anyhow::Result;

pub struct ListenBrainzScrobbler {
    name: String,
    token: String,
    api_url: String,
}

impl ListenBrainzScrobbler {
    pub fn new(name: String, token: String, api_url: String) -> Self {
        Self {
            name,
            token,
            api_url,
        }
    }
}

impl Scrobbler for ListenBrainzScrobbler {
    async fn now_playing(&self, _track: &Track) -> Result<()> {
        // TODO: Implement ListenBrainz now playing
        Ok(())
    }

    async fn scrobble(&self, _track: &Track, _timestamp: i64) -> Result<()> {
        // TODO: Implement ListenBrainz scrobble
        Ok(())
    }
}
