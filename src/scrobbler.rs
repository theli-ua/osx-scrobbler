// Scrobbler implementations for Last.fm and ListenBrainz

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use listenbrainz::ListenBrainz;
use rustfm_scrobble_proxy::{Scrobble, Scrobbler as LastFmScrobbler};

/// Last.fm authentication helper
pub mod lastfm_auth {
    use anyhow::{Context, Result};
    use rustfm_scrobble_proxy::Scrobbler;
    use serde::Deserialize;

    const LASTFM_API_URL: &str = "https://ws.audioscrobbler.com/2.0/";
    const LASTFM_AUTH_URL: &str = "https://www.last.fm/api/auth/";

    #[derive(Debug, Deserialize)]
    struct LastFmResponse {
        token: Option<String>,
    }

    /// Get an authentication token from Last.fm
    fn get_token(api_key: &str, api_secret: &str) -> Result<String> {
        // Create API signature for getToken request
        let sig_string = format!("api_key{}method{}{}", api_key, "auth.gettoken", api_secret);
        let signature = format!("{:x}", md5::compute(sig_string.as_bytes()));

        // Build form-encoded body
        let body = format!(
            "method=auth.gettoken&api_key={}&api_sig={}&format=json",
            api_key, signature
        );

        let response = attohttpc::post(LASTFM_API_URL)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .text(body)
            .send()
            .context("Failed to get token from Last.fm")?;

        if !response.is_success() {
            anyhow::bail!("Last.fm API error: {}", response.status());
        }

        let data: LastFmResponse = response.json()?;
        data.token
            .ok_or_else(|| anyhow::anyhow!("No token in Last.fm response"))
    }

    /// Perform the complete Last.fm authentication flow using token-based auth
    /// Returns the session key on success
    pub fn authenticate(api_key: &str, api_secret: &str) -> Result<String> {
        println!("Starting Last.fm authentication...\n");

        // Step 1: Get authentication token
        println!("Getting authorization token...");
        let token = get_token(api_key, api_secret)?;
        println!("Token obtained: {}\n", token);

        // Step 2: Direct user to authorize
        let auth_url = format!("{}?api_key={}&token={}", LASTFM_AUTH_URL, api_key, token);
        println!("Please authorize this application:");
        println!("  {}\n", auth_url);
        println!("Opening authorization URL in your browser...");

        let _ = std::process::Command::new("open").arg(&auth_url).spawn();

        println!("\nAfter authorizing, press Enter to continue...");

        // Wait for user to press Enter
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        // Step 3: Exchange token for session key
        println!("\nExchanging token for session key...");
        let mut scrobbler = Scrobbler::new(api_key, api_secret);
        let session = scrobbler.authenticate_with_token(&token)?;
        println!("Session key obtained successfully!\n");

        Ok(session.key)
    }
}

/// Represents a music track
#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub duration: Option<u64>,
}

/// Scrobbling service
pub enum Service {
    LastFm(LastFmScrobbler),
    ListenBrainz { name: String, client: ListenBrainz },
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
                    .with_context(|| {
                        format!("Failed to update now playing on ListenBrainz ({})", name)
                    })?;
                log::info!("ListenBrainz ({}): Now playing updated", name);
            }
        }
        Ok(())
    }

    /// Scrobble a track
    pub fn scrobble(&self, track: &Track, timestamp: DateTime<Utc>) -> Result<()> {
        match self {
            Self::LastFm(scrobbler) => {
                let mut scrobble =
                    Scrobble::new(&track.artist, &track.title, track.album.as_deref());
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
