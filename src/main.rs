mod config;
mod media_monitor;
mod scrobbler;

use anyhow::Result;
use media_monitor::{MediaEvents, MediaMonitor};
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

    // Initialize media monitor
    let monitor = Arc::new(MediaMonitor::new(
        Duration::from_secs(config.refresh_interval),
        config.scrobble_threshold,
    ));

    log::info!("Starting OSX Scrobbler...");

    // TODO: Initialize scrobblers
    // TODO: Initialize system tray

    // Start media monitoring
    monitor
        .start_monitoring(|events: MediaEvents| {
            if let Some(track) = events.now_playing {
                log::info!(
                    "Now playing: {} - {} (album: {})",
                    track.artist,
                    track.title,
                    track.album.as_deref().unwrap_or("Unknown")
                );
                // TODO: Send now playing to scrobblers
            }

            if let Some((track, timestamp)) = events.scrobble {
                log::info!(
                    "Scrobble: {} - {} at {}",
                    track.artist,
                    track.title,
                    timestamp.format("%Y-%m-%d %H:%M:%S")
                );
                // TODO: Send scrobble to scrobblers
            }
        })
        .await?;

    Ok(())
}
