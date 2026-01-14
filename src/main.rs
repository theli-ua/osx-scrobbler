mod config;
mod media_monitor;
mod scrobbler;
mod ui;

use anyhow::Result;
use media_monitor::MediaMonitor;
use scrobbler::{lastfm::LastFmScrobbler, listenbrainz::ListenBrainzScrobbler, traits::Scrobbler};
use std::sync::Arc;
use std::time::Duration;
use ui::tray::TrayManager;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger with default level of info if RUST_LOG is not set
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

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

    // Initialize system tray
    let tray = TrayManager::new()?;
    log::info!("System tray initialized");

    // Initialize media monitor
    let monitor = Arc::new(MediaMonitor::new(
        Duration::from_secs(config.refresh_interval),
        config.scrobble_threshold,
    ));

    log::info!("Starting OSX Scrobbler...");

    // Run main loop
    let mut interval = tokio::time::interval(Duration::from_secs(config.refresh_interval));

    loop {
        interval.tick().await;

        // Check for tray events
        if tray.handle_events() {
            log::info!("Quit requested from tray menu");
            break;
        }

        // Poll media state
        match monitor.poll().await {
            Ok(events) => {
                if let Some(ref track) = events.now_playing {
                    let track_str = format!("{} - {}", track.artist, track.title);
                    log::info!(
                        "Now playing: {} (album: {})",
                        track_str,
                        track.album.as_deref().unwrap_or("Unknown")
                    );

                    // Update tray
                    if let Err(e) = tray.update_now_playing(Some(track_str)) {
                        log::error!("Failed to update tray now playing: {}", e);
                    }

                    // Send now playing to all enabled scrobblers
                    let scrobblers_clone = scrobblers.clone();
                    let track_clone = track.clone();
                    tokio::spawn(async move {
                        for scrobbler in &scrobblers_clone {
                            if let Err(e) = scrobbler.now_playing(&track_clone).await {
                                log::error!("Failed to send now playing: {}", e);
                            }
                        }
                    });
                }

                if let Some((ref track, timestamp)) = events.scrobble {
                    let track_str = format!("{} - {}", track.artist, track.title);
                    log::info!(
                        "Scrobble: {} at {}",
                        track_str,
                        timestamp.format("%Y-%m-%d %H:%M:%S")
                    );

                    // Update tray
                    if let Err(e) = tray.update_last_scrobbled(Some(track_str)) {
                        log::error!("Failed to update tray last scrobbled: {}", e);
                    }

                    // Send scrobble to all enabled scrobblers
                    let scrobblers_clone = scrobblers.clone();
                    let track_clone = track.clone();
                    let ts = timestamp.timestamp();
                    tokio::spawn(async move {
                        for scrobbler in &scrobblers_clone {
                            if let Err(e) = scrobbler.scrobble(&track_clone, ts).await {
                                log::error!("Failed to scrobble: {}", e);
                            }
                        }
                    });
                }
            }
            Err(e) => {
                log::error!("Error polling media: {}", e);
            }
        }
    }

    log::info!("OSX Scrobbler shutting down");
    Ok(())
}
