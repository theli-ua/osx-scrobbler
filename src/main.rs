mod config;
mod media_monitor;
mod scrobbler;
mod text_cleanup;
mod ui;

use anyhow::Result;
use clap::Parser;
use media_monitor::MediaMonitor;
use scrobbler::{lastfm::LastFmScrobbler, listenbrainz::ListenBrainzScrobbler, traits::Scrobbler};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use ui::tray::{TrayEvent, TrayManager};
use winit::event_loop::{ControlFlow, EventLoop};

#[derive(Debug, Clone)]
enum TrayUpdate {
    NowPlaying(String),
    Scrobbled(String),
}

/// OSX Scrobbler - Music scrobbling for macOS
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Authenticate with Last.fm and obtain session key
    #[arg(long)]
    auth_lastfm: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Handle Last.fm authentication if requested
    if args.auth_lastfm {
        return handle_lastfm_auth();
    }
    // Initialize logger with default level of info if RUST_LOG is not set
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Configure app to be menu bar only (no dock icon) on macOS
    #[cfg(target_os = "macos")]
    {
        use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
        use objc2_foundation::MainThreadMarker;
        unsafe {
            let mtm = MainThreadMarker::new_unchecked();
            let app = NSApplication::sharedApplication(mtm);
            app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
        }
    }

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
    let tray = TrayManager::new(config.launch_at_login)?;
    log::info!("System tray initialized");

    // Initialize text cleaner
    let text_cleaner = text_cleanup::TextCleaner::new(&config.cleanup);
    if config.cleanup.enabled {
        log::info!("Text cleanup enabled with {} patterns", config.cleanup.patterns.len());
    }

    // Initialize media monitor
    let monitor = Arc::new(MediaMonitor::new(
        Duration::from_secs(config.refresh_interval),
        config.scrobble_threshold,
        text_cleaner,
    ));

    log::info!("Starting OSX Scrobbler...");

    // Create channel for tray updates
    let (tray_tx, mut tray_rx) = mpsc::unbounded_channel::<TrayUpdate>();

    // Create tokio runtime for async tasks
    let rt = tokio::runtime::Runtime::new()?;

    // Spawn background task for media monitoring
    let scrobblers_bg = scrobblers.clone();
    let monitor_bg = monitor.clone();
    let refresh_interval = config.refresh_interval;

    rt.spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(refresh_interval));

        loop {
            interval.tick().await;

            // Poll media state
            match monitor_bg.poll().await {
                Ok(events) => {
                    if let Some(ref track) = events.now_playing {
                        let track_str = format!("{} - {}", track.artist, track.title);
                        log::info!(
                            "Now playing: {} (album: {})",
                            track_str,
                            track.album.as_deref().unwrap_or("Unknown")
                        );

                        // Send update to main thread
                        let _ = tray_tx.send(TrayUpdate::NowPlaying(track_str));

                        // Send now playing to all enabled scrobblers
                        let scrobblers_clone = scrobblers_bg.clone();
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

                        // Send update to main thread
                        let _ = tray_tx.send(TrayUpdate::Scrobbled(track_str));

                        // Send scrobble to all enabled scrobblers
                        let scrobblers_clone = scrobblers_bg.clone();
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
    });

    // Run event loop on main thread for tray icon
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut current_config = config.clone();
    let mut should_quit = false;

    event_loop.run(move |_event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);

        // Process tray updates from background thread
        while let Ok(update) = tray_rx.try_recv() {
            match update {
                TrayUpdate::NowPlaying(track) => {
                    if let Err(e) = tray.update_now_playing(Some(track)) {
                        log::error!("Failed to update tray now playing: {}", e);
                    }
                }
                TrayUpdate::Scrobbled(track) => {
                    if let Err(e) = tray.update_last_scrobbled(Some(track)) {
                        log::error!("Failed to update tray last scrobbled: {}", e);
                    }
                }
            }
        }

        // Check for tray events
        if let Some(event) = tray.handle_events() {
            match event {
                TrayEvent::Quit => {
                    log::info!("Quit requested from tray menu");
                    should_quit = true;
                }
                TrayEvent::ToggleLaunchAtLogin => {
                    match tray.toggle_launch_at_login() {
                        Ok(new_state) => {
                            current_config.launch_at_login = new_state;
                            if let Err(e) = current_config.save() {
                                log::error!("Failed to save config: {}", e);
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to toggle launch at login: {}", e);
                        }
                    }
                }
            }
        }

        if should_quit {
            log::info!("OSX Scrobbler shutting down");
            elwt.exit();
        }
    })?;

    Ok(())
}

/// Handle Last.fm authentication flow
fn handle_lastfm_auth() -> Result<()> {
    // Load current config
    let mut config = config::Config::load()?;

    // Check if Last.fm is configured
    let lastfm_config = config
        .lastfm
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Last.fm is not configured in config file"))?;

    if lastfm_config.api_key.is_empty() || lastfm_config.api_secret.is_empty() {
        anyhow::bail!("Last.fm API key and secret must be set in config file before authenticating");
    }

    println!("Last.fm Authentication");
    println!("======================\n");
    println!("API Key: {}", lastfm_config.api_key);
    println!("API Secret: {}\n", lastfm_config.api_secret);

    // Run authentication flow using tokio runtime
    let rt = tokio::runtime::Runtime::new()?;
    let session_key = rt.block_on(scrobbler::lastfm_auth::authenticate(
        &lastfm_config.api_key,
        &lastfm_config.api_secret,
    ))?;

    println!("Session Key: {}\n", session_key);

    // Update config with session key
    if let Some(ref mut lastfm) = config.lastfm {
        lastfm.session_key = session_key;
        lastfm.enabled = true;
    }

    // Save config
    config.save()?;

    println!("Configuration updated successfully!");
    println!("Last.fm is now enabled and ready to use.");
    println!("\nYou can now run the scrobbler normally.");

    Ok(())
}
