// ListenBrainz scrobbler implementation
// API Documentation: https://listenbrainz.readthedocs.io/

use super::traits::{Scrobbler, Track};
use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::json;

pub struct ListenBrainzScrobbler {
    name: String,
    token: String,
    api_url: String,
    client: Client,
}

impl ListenBrainzScrobbler {
    pub fn new(name: String, token: String, api_url: String) -> Self {
        Self {
            name,
            token,
            api_url,
            client: Client::new(),
        }
    }

    /// Submit a listen to ListenBrainz
    async fn submit_listen(
        &self,
        listen_type: &str,
        track: &Track,
        timestamp: Option<i64>,
    ) -> Result<()> {
        let mut track_metadata = json!({
            "artist_name": track.artist,
            "track_name": track.title,
        });

        if let Some(ref album) = track.album {
            track_metadata["release_name"] = json!(album);
        }

        let payload = if listen_type == "playing_now" {
            json!({
                "listen_type": listen_type,
                "payload": [{
                    "track_metadata": track_metadata,
                }]
            })
        } else {
            // "single" type requires timestamp
            let ts = timestamp.unwrap_or_else(|| chrono::Utc::now().timestamp());
            json!({
                "listen_type": listen_type,
                "payload": [{
                    "listened_at": ts,
                    "track_metadata": track_metadata,
                }]
            })
        };

        let url = format!("{}/1/submit-listens", self.api_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Token {}", self.token))
            .json(&payload)
            .send()
            .await
            .context("Failed to send request to ListenBrainz")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("ListenBrainz API error ({}): {}", status, body);
        }

        Ok(())
    }
}

impl Scrobbler for ListenBrainzScrobbler {
    fn now_playing(&self, track: &Track) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        let track = track.clone();
        let name = self.name.clone();
        Box::pin(async move {
            log::debug!(
                "Sending now playing to ListenBrainz ({}): {} - {}",
                name,
                track.artist,
                track.title
            );

            self.submit_listen("playing_now", &track, None)
                .await
                .context("Failed to update now playing on ListenBrainz")?;

            log::info!("ListenBrainz ({}): Now playing updated", name);
            Ok(())
        })
    }

    fn scrobble(&self, track: &Track, timestamp: i64) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>> {
        let track = track.clone();
        let name = self.name.clone();
        Box::pin(async move {
            log::debug!(
                "Scrobbling to ListenBrainz ({}): {} - {}",
                name,
                track.artist,
                track.title
            );

            self.submit_listen("single", &track, Some(timestamp))
                .await
                .context("Failed to scrobble to ListenBrainz")?;

            log::info!("ListenBrainz ({}): Scrobbled successfully", name);
            Ok(())
        })
    }
}
