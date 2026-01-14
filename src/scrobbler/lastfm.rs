// Last.fm scrobbler implementation
// API Documentation: https://www.last.fm/api/scrobbling

use super::traits::{Scrobbler, Track};
use anyhow::{Context, Result};
use reqwest::Client;
use std::collections::BTreeMap;

const LASTFM_API_URL: &str = "https://ws.audioscrobbler.com/2.0/";

pub struct LastFmScrobbler {
    api_key: String,
    api_secret: String,
    session_key: String,
    client: Client,
}

impl LastFmScrobbler {
    pub fn new(api_key: String, api_secret: String, session_key: String) -> Self {
        Self {
            api_key,
            api_secret,
            session_key,
            client: Client::new(),
        }
    }

    /// Generate API signature for Last.fm requests
    /// Signature = md5(param1value1param2value2...SECRET)
    /// Parameters must be sorted alphabetically
    fn generate_signature(&self, params: &BTreeMap<String, String>) -> String {
        let mut sig_string = String::new();
        for (key, value) in params.iter() {
            sig_string.push_str(key);
            sig_string.push_str(value);
        }
        sig_string.push_str(&self.api_secret);

        format!("{:x}", md5::compute(sig_string.as_bytes()))
    }

    /// Make a signed POST request to Last.fm API
    async fn api_request(&self, mut params: BTreeMap<String, String>) -> Result<()> {
        // Add common parameters
        params.insert("api_key".to_string(), self.api_key.clone());
        params.insert("sk".to_string(), self.session_key.clone());

        // Generate signature
        let signature = self.generate_signature(&params);
        params.insert("api_sig".to_string(), signature);

        // Make request
        let response = self
            .client
            .post(LASTFM_API_URL)
            .form(&params)
            .send()
            .await
            .context("Failed to send request to Last.fm")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Last.fm API error ({}): {}", status, body);
        }

        let body = response.text().await?;
        log::debug!("Last.fm response: {}", body);

        // Check for error in response
        if body.contains("<lfm status=\"failed\">") {
            anyhow::bail!("Last.fm API returned error: {}", body);
        }

        Ok(())
    }
}

impl Scrobbler for LastFmScrobbler {
    fn now_playing(&self, track: &Track) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        let track = track.clone();
        Box::pin(async move {
            let mut params = BTreeMap::new();
            params.insert("method".to_string(), "track.updateNowPlaying".to_string());
            params.insert("artist".to_string(), track.artist.clone());
            params.insert("track".to_string(), track.title.clone());

            if let Some(ref album) = track.album {
                params.insert("album".to_string(), album.clone());
            }

            if let Some(duration) = track.duration {
                params.insert("duration".to_string(), duration.to_string());
            }

            log::debug!("Sending now playing to Last.fm: {} - {}", track.artist, track.title);

            self.api_request(params)
                .await
                .context("Failed to update now playing on Last.fm")?;

            log::info!("Last.fm: Now playing updated");
            Ok(())
        })
    }

    fn scrobble(&self, track: &Track, timestamp: i64) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        let track = track.clone();
        Box::pin(async move {
            let mut params = BTreeMap::new();
            params.insert("method".to_string(), "track.scrobble".to_string());
            params.insert("artist".to_string(), track.artist.clone());
            params.insert("track".to_string(), track.title.clone());
            params.insert("timestamp".to_string(), timestamp.to_string());

            if let Some(ref album) = track.album {
                params.insert("album".to_string(), album.clone());
            }

            if let Some(duration) = track.duration {
                params.insert("duration".to_string(), duration.to_string());
            }

            log::debug!("Scrobbling to Last.fm: {} - {}", track.artist, track.title);

            self.api_request(params)
                .await
                .context("Failed to scrobble to Last.fm")?;

            log::info!("Last.fm: Scrobbled successfully");
            Ok(())
        })
    }
}
