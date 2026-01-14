mod config;
mod media_monitor;
mod scrobbler;

use anyhow::Result;
use media_monitor::{MediaEvents, MediaMonitor};
use scrobbler::{lastfm::LastFmScrobbler, listenbrainz::ListenBrainzScrobbler, traits::Scrobbler};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();

    // Load configuration
    let config = config::Config::load()?;
    log::info!("Configuration loaded successfully");
    log::info!("Refresh interval: {}s", config.refresh_interval);
    log::info!("Scrobble threshold: {}%", config.scrobble_threshold);

    // Initialize scrobblers
    let mut scrobblers: Vec<Arc<dyn Scrobbler>> = Vec::new();

    // Initialize Last.fm if enabled
    if let Some(ref lastfm_config) = config.lastfm {
        if lastfm_config.enabled {
            if !lastfm_config.session_key.is_empty() {
                log::info!("Last.fm scrobbler enabled");
                let lastfm = Arc::new(LastFmScrobbler::new(
                    lastfm_config.api_key.clone(),
                    lastfm_config.api_secret.clone(),
                    lastfm_config.session_key.clone(),
                ));
                scrobblers.push(lastfm);
            } else {
                log::warn!("Last.fm is enabled but session_key is not set. Skipping Last.fm.");
            }
        }
    }

    // Initialize ListenBrainz instances if enabled
    for lb_config in &config.listenbrainz {
        if lb_config.enabled {
            log::info!("ListenBrainz scrobbler enabled: {}", lb_config.name);
            let listenbrainz = Arc::new(ListenBrainzScrobbler::new(
                lb_config.name.clone(),
                lb_config.token.clone(),
                lb_config.api_url.clone(),
            ));
            scrobblers.push(listenbrainz);
        }
    }

    if scrobblers.is_empty() {
        log::warn!("No scrobblers enabled! The app will monitor media but won't scrobble anywhere.");
    }

    // Initialize media monitor
    let monitor = Arc::new(MediaMonitor::new(
        Duration::from_secs(config.refresh_interval),
        config.scrobble_threshold,
    ));

    log::info!("Starting OSX Scrobbler...");

    // TODO: Initialize system tray

    // Start media monitoring
    monitor
        .start_monitoring(move |events: MediaEvents| {
            let scrobblers = scrobblers.clone();

            // Spawn async task to handle events
            tokio::spawn(async move {
                if let Some(track) = events.now_playing {
                    log::info!(
                        "Now playing: {} - {} (album: {})",
                        track.artist,
                        track.title,
                        track.album.as_deref().unwrap_or("Unknown")
                    );

                    // Send now playing to all enabled scrobblers
                    for scrobbler in &scrobblers {
                        if let Err(e) = scrobbler.now_playing(&track).await {
                            log::error!("Failed to send now playing: {}", e);
                        }
                    }
                }

                if let Some((track, timestamp)) = events.scrobble {
                    log::info!(
                        "Scrobble: {} - {} at {}",
                        track.artist,
                        track.title,
                        timestamp.format("%Y-%m-%d %H:%M:%S")
                    );

                    // Send scrobble to all enabled scrobblers
                    let ts = timestamp.timestamp();
                    for scrobbler in &scrobblers {
                        if let Err(e) = scrobbler.scrobble(&track, ts).await {
                            log::error!("Failed to scrobble: {}", e);
                        }
                    }
                }
            });
        })
        .await?;

    Ok(())
}
