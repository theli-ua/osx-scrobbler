mod config;
mod media_monitor;
mod scrobbler;
mod text_cleanup;
mod ui;

use anyhow::Result;
use clap::Parser;
use media_monitor::MediaMonitor;
use scrobbler::Service;
use std::sync::Arc;
use std::time::Duration;
use ui::tray::{TrayEvent, TrayManager};
use winit::event_loop::{ControlFlow, EventLoop};

/// OSX Scrobbler - Music scrobbling for macOS
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Authenticate with Last.fm and obtain session key
    #[arg(long)]
    auth_lastfm: bool,

    /// Force console output (show logs in terminal)
    #[arg(long)]
    console: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Handle Last.fm authentication if requested
    if args.auth_lastfm {
        return handle_lastfm_auth();
    }

    // Set up logging based on environment
    setup_logging(args.console)?;

    // Load configuration
    let config = config::Config::load()?;
    log::info!("Configuration loaded successfully");
    log::info!("Refresh interval: {}s", config.refresh_interval);
    log::info!("Scrobble threshold: {}%", config.scrobble_threshold);

    // Initialize scrobblers
    let mut scrobblers: Vec<Service> = Vec::new();

    // Initialize Last.fm if enabled
    if let Some(ref lastfm_config) = config.lastfm {
        if lastfm_config.enabled {
            if !lastfm_config.session_key.is_empty() {
                log::info!("Last.fm scrobbler enabled");
                let service = Service::lastfm(
                    lastfm_config.api_key.clone(),
                    lastfm_config.api_secret.clone(),
                    lastfm_config.session_key.clone(),
                );
                scrobblers.push(service);
            } else {
                log::warn!("Last.fm is enabled but session_key is not set. Skipping Last.fm.");
            }
        }
    }

    // Initialize ListenBrainz instances if enabled
    for lb_config in &config.listenbrainz {
        if lb_config.enabled {
            log::info!("ListenBrainz scrobbler enabled: {}", lb_config.name);
            match Service::listenbrainz(
                lb_config.name.clone(),
                lb_config.token.clone(),
                lb_config.api_url.clone(),
            ) {
                Ok(service) => scrobblers.push(service),
                Err(e) => log::error!("Failed to initialize ListenBrainz: {}", e),
            }
        }
    }

    if scrobblers.is_empty() {
        log::warn!("No scrobblers enabled! The app will monitor media but won't scrobble anywhere.");
    }

    // Initialize system tray
    let tray = TrayManager::new()?;
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
    #[derive(Debug, Clone)]
    enum TrayUpdate {
        NowPlaying(String),
        Scrobbled(String),
    }

    let (tx, rx) = std::sync::mpsc::channel::<TrayUpdate>();
    let (shutdown_tx, shutdown_rx) = std::sync::mpsc::channel::<()>();

    // Spawn background thread for media monitoring
    let scrobblers_bg = Arc::new(scrobblers);
    let monitor_bg = monitor.clone();
    let refresh_interval = config.refresh_interval;

    std::thread::spawn(move || {
        loop {
            // Check for shutdown signal with timeout
            match shutdown_rx.recv_timeout(Duration::from_secs(refresh_interval)) {
                Ok(_) => {
                    log::info!("Background thread received shutdown signal");
                    break;
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    // Normal timeout, continue polling
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    log::info!("Shutdown channel disconnected, exiting background thread");
                    break;
                }
            }

            // Poll media state
            match monitor_bg.poll() {
                Ok(events) => {
                    if let Some(ref track) = events.now_playing {
                        log::info!(
                            "Now playing: {} - {} (album: {})",
                            track.artist,
                            track.title,
                            track.album.as_deref().unwrap_or("Unknown")
                        );

                        // Send now playing to all enabled scrobblers
                        for scrobbler in scrobblers_bg.iter() {
                            if let Err(e) = scrobbler.now_playing(track) {
                                log::error!("Failed to send now playing: {}", e);
                            }
                        }

                        // Update tray
                        let track_str = format!("{} - {}", track.artist, track.title);
                        let _ = tx.send(TrayUpdate::NowPlaying(track_str));
                    }

                    if let Some((ref track, timestamp)) = events.scrobble {
                        log::info!(
                            "Scrobble: {} - {} at {}",
                            track.artist,
                            track.title,
                            timestamp.format("%Y-%m-%d %H:%M:%S")
                        );

                        // Send scrobble to all enabled scrobblers
                        for scrobbler in scrobblers_bg.iter() {
                            if let Err(e) = scrobbler.scrobble(track, timestamp) {
                                log::error!("Failed to scrobble: {}", e);
                            }
                        }

                        // Update tray
                        let track_str = format!("{} - {}", track.artist, track.title);
                        let _ = tx.send(TrayUpdate::Scrobbled(track_str));
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

    // Configure app to be menu bar only (no dock icon)
    // MUST be set AFTER EventLoop creation as winit creates NSApplication
    use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
    use objc2_foundation::MainThreadMarker;
    unsafe {
        let mtm = MainThreadMarker::new_unchecked();
        let app = NSApplication::sharedApplication(mtm);
        app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
    }
    log::info!("Set activation policy to Accessory (no dock icon)");

    let mut should_quit = false;

    #[allow(deprecated)]
    event_loop.run(move |_event, elwt| {
        // Wake up every 100ms to check for tray events and updates
        elwt.set_control_flow(ControlFlow::WaitUntil(
            std::time::Instant::now() + Duration::from_millis(100)
        ));

        // Process tray updates from background thread
        while let Ok(update) = rx.try_recv() {
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
                    log::info!("OSX Scrobbler shutting down");
                    // Signal background thread to shutdown
                    let _ = shutdown_tx.send(());
                    should_quit = true;
                }
            }
        }

        if should_quit {
            elwt.exit();
        }
    })?;

    log::info!("Application exited cleanly");
    Ok(())
}

/// Set up logging based on whether we're running from a terminal
fn setup_logging(force_console: bool) -> Result<()> {
    use std::io::Write;

    // Check if stdout is a TTY (terminal)
    let is_terminal = atty::is(atty::Stream::Stdout);
    let use_console = force_console || is_terminal;

    if use_console {
        // Running from terminal - log to console
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    } else {
        // Not running from terminal (e.g., launched via Spotlight)
        // Log to file instead
        let log_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
            .join("Library")
            .join("Logs");

        std::fs::create_dir_all(&log_dir)?;
        let log_file = log_dir.join("osx-scrobbler.log");

        let target = Box::new(std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file)?);

        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .target(env_logger::Target::Pipe(target))
            .format(|buf, record| {
                writeln!(
                    buf,
                    "[{}] {} - {}",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                    record.level(),
                    record.args()
                )
            })
            .init();

        // Log where we're logging to (this will go to the file)
        log::info!("OSX Scrobbler started (logging to {})", log_file.display());
    }

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

    // Run authentication flow
    let session_key = scrobbler::lastfm_auth::authenticate(
        &lastfm_config.api_key,
        &lastfm_config.api_secret,
    )?;

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
