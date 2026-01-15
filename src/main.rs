mod config;
mod media_monitor;
mod scrobbler;
mod text_cleanup;
mod ui;

use anyhow::Result;
use backoff::{retry, ExponentialBackoff};
use clap::Parser;
use media_monitor::MediaMonitor;
use scrobbler::Service;
use std::time::{Duration, Instant};
use ui::tray::TrayManager;
use winit::event_loop::{ControlFlow, EventLoop};

/// OSX Scrobbler - Music scrobbling for macOS
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Authenticate with Last.fm and obtain session key
    #[arg(long)]
    auth_lastfm: bool,

    /// Install OSX Scrobbler as a macOS app bundle in /Applications/
    #[arg(long)]
    install_app: bool,

    /// Uninstall the app bundle from /Applications/
    #[arg(long)]
    uninstall_app: bool,

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

    // Handle app installation if requested
    if args.install_app {
        return handle_install_app();
    }

    // Handle app uninstallation if requested
    if args.uninstall_app {
        return handle_uninstall_app();
    }

    // Set up logging based on environment
    setup_logging(args.console)?;

    // Load configuration (mutable for app filtering updates)
    let mut config = config::Config::load()?;
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
            let name = lb_config.name.clone();
            let token = lb_config.token.clone();
            let api_url = lb_config.api_url.clone();

            let backoff = ExponentialBackoff {
                max_elapsed_time: Some(Duration::from_secs(30)),
                ..Default::default()
            };

            let result = retry(backoff, || {
                Service::listenbrainz(name.clone(), token.clone(), api_url.clone())
                    .map_err(backoff::Error::transient)
            });

            match result {
                Ok(service) => scrobblers.push(service),
                Err(e) => log::error!("Failed to initialize ListenBrainz after retries: {}", e),
            }
        }
    }

    if scrobblers.is_empty() {
        log::warn!(
            "No scrobblers enabled! The app will monitor media but won't scrobble anywhere."
        );
    }

    // Initialize system tray
    let mut tray = TrayManager::new()?;
    log::info!("System tray initialized");

    // Initialize text cleaner
    let text_cleaner = text_cleanup::TextCleaner::new(&config.cleanup);
    if config.cleanup.enabled {
        log::info!(
            "Text cleanup enabled with {} patterns",
            config.cleanup.patterns.len()
        );
    }

    // Initialize media monitor
    let mut media_monitor = MediaMonitor::new(config.scrobble_threshold, text_cleaner);

    log::info!("Starting OSX Scrobbler...");

    // Setup polling state
    let refresh_interval = Duration::from_secs(config.refresh_interval);
    let mut next_poll_time = Instant::now();

    // Define user events for tray menu actions
    #[derive(Debug, Clone, Copy)]
    enum UserEvent {
        TrayQuit,
    }

    // Run event loop on main thread for tray icon
    let event_loop = EventLoop::<UserEvent>::with_user_event()
        .build()
        .expect("Failed to create event loop");

    // Get proxy to send events from other threads
    let event_proxy = event_loop.create_proxy();

    // Spawn minimal thread to forward tray menu events to main event loop
    // This allows event-based wakeup instead of polling
    let quit_item_id = tray.quit_item.id().clone();
    std::thread::spawn(move || {
        use tray_icon::menu::MenuEvent;
        loop {
            if let Ok(event) = MenuEvent::receiver().recv() {
                if event.id == quit_item_id {
                    log::info!("Quit menu item clicked");
                    let _ = event_proxy.send_event(UserEvent::TrayQuit);
                }
            }
        }
    });

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

    #[allow(deprecated)]
    event_loop.run(move |event, elwt| {
        // Handle user events (tray menu actions)
        if let winit::event::Event::UserEvent(UserEvent::TrayQuit) = event {
            log::info!("OSX Scrobbler shutting down");
            elwt.exit();
            return;
        }

        let now = Instant::now();

        // Only wake up when we need to poll media
        elwt.set_control_flow(ControlFlow::WaitUntil(next_poll_time));

        // Check if it's time to poll media
        if now >= next_poll_time {
            match media_monitor.poll(&config.app_filtering) {
                Ok(events) => {
                    // Handle now_playing event
                    if let Some((ref track, ref bundle_id)) = events.now_playing {
                        log::info!(
                            "Now playing: {} - {} (album: {}) from {:?}",
                            track.artist,
                            track.title,
                            track.album.as_deref().unwrap_or("Unknown"),
                            bundle_id
                        );

                        // Send to scrobblers immediately with retries
                        for scrobbler in &scrobblers {
                            let backoff = ExponentialBackoff {
                                max_elapsed_time: Some(Duration::from_secs(10)),
                                ..Default::default()
                            };

                            let result = retry(backoff, || {
                                scrobbler
                                    .now_playing(track)
                                    .map_err(backoff::Error::transient)
                            });

                            if let Err(e) = result {
                                log::error!("Failed to send now playing after retries: {}", e);
                            }
                        }

                        // Update tray immediately
                        let track_str = format!("{} - {}", track.artist, track.title);
                        if let Err(e) = tray.update_now_playing(Some(track_str)) {
                            log::error!("Failed to update tray now playing: {}", e);
                        }
                    }

                    // Handle scrobble event
                    if let Some((ref track, timestamp, ref bundle_id)) = events.scrobble {
                        log::info!(
                            "Scrobble: {} - {} at {} from {:?}",
                            track.artist,
                            track.title,
                            timestamp.format("%Y-%m-%d %H:%M:%S"),
                            bundle_id
                        );

                        for scrobbler in &scrobblers {
                            let backoff = ExponentialBackoff {
                                max_elapsed_time: Some(Duration::from_secs(30)),
                                ..Default::default()
                            };

                            let result = retry(backoff, || {
                                scrobbler
                                    .scrobble(track, timestamp)
                                    .map_err(backoff::Error::transient)
                            });

                            if let Err(e) = result {
                                log::error!("Failed to scrobble after retries: {}", e);
                            }
                        }

                        let track_str = format!("{} - {}", track.artist, track.title);
                        if let Err(e) = tray.update_last_scrobbled(Some(track_str)) {
                            log::error!("Failed to update tray last scrobbled: {}", e);
                        }
                    }

                    // Handle unknown app event (blocking dialog)
                    if let Some(ref bundle_id) = events.unknown_app {
                        use ui::app_dialog::{show_app_prompt, AppChoice};

                        log::info!("Prompting user for app: {}", bundle_id);
                        let choice = show_app_prompt(bundle_id);

                        match choice {
                            AppChoice::Allow => {
                                log::info!("User allowed app: {}", bundle_id);
                                if !config.app_filtering.allowed_apps.contains(bundle_id) {
                                    config.app_filtering.allowed_apps.push(bundle_id.clone());
                                    if let Err(e) = config.save() {
                                        log::error!("Failed to save config: {}", e);
                                    } else {
                                        log::info!("Added {} to allowed apps", bundle_id);
                                    }
                                }
                            }
                            AppChoice::Ignore => {
                                log::info!("User ignored app: {}", bundle_id);
                                if !config.app_filtering.ignored_apps.contains(bundle_id) {
                                    config.app_filtering.ignored_apps.push(bundle_id.clone());
                                    if let Err(e) = config.save() {
                                        log::error!("Failed to save config: {}", e);
                                    } else {
                                        log::info!("Added {} to ignored apps", bundle_id);
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    log::error!("Error polling media: {}", e);
                }
            }

            // Schedule next poll
            next_poll_time = now + refresh_interval;
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

        let target = Box::new(
            std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_file)?,
        );

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
        anyhow::bail!(
            "Last.fm API key and secret must be set in config file before authenticating"
        );
    }

    println!("Last.fm Authentication");
    println!("======================\n");
    println!("API Key: {}", lastfm_config.api_key);
    println!("API Secret: {}\n", lastfm_config.api_secret);

    // Run authentication flow
    let session_key =
        scrobbler::lastfm_auth::authenticate(&lastfm_config.api_key, &lastfm_config.api_secret)?;

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

/// Info.plist template for macOS app bundle
const INFO_PLIST_TEMPLATE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>OSX Scrobbler</string>
    <key>CFBundleDisplayName</key>
    <string>OSX Scrobbler</string>
    <key>CFBundleIdentifier</key>
    <string>com.osxscrobbler</string>
    <key>CFBundleVersion</key>
    <string>{VERSION}</string>
    <key>CFBundleShortVersionString</key>
    <string>{VERSION}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleExecutable</key>
    <string>osx-scrobbler</string>
    <key>LSUIElement</key>
    <true/>
    <key>LSMinimumSystemVersion</key>
    <string>10.15</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>"#;

/// Install OSX Scrobbler as a macOS app bundle in /Applications/
fn handle_install_app() -> Result<()> {
    use std::fs;
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;

    println!("OSX Scrobbler App Bundle Installer");
    println!("===================================\n");

    let app_name = "OSX Scrobbler.app";
    let app_path = std::path::Path::new("/Applications").join(app_name);
    let contents_dir = app_path.join("Contents");
    let macos_dir = contents_dir.join("MacOS");

    // Check if app already exists
    if app_path.exists() {
        print!(
            "App bundle already exists at {}. Overwrite? [y/N] ",
            app_path.display()
        );
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Installation cancelled.");
            return Ok(());
        }

        println!("Removing existing app bundle...");
        fs::remove_dir_all(&app_path)?;
    }

    // Create directory structure
    println!("Creating app bundle structure...");
    match fs::create_dir_all(&macos_dir) {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            eprintln!("\n❌ Permission denied creating app bundle.");
            eprintln!("\nTry running with sudo:");
            eprintln!("  sudo osx-scrobbler --install-app\n");
            return Err(e.into());
        }
        Err(e) => return Err(e.into()),
    }

    // Get current binary path
    let current_exe = std::env::current_exe()?;
    let target_binary = macos_dir.join("osx-scrobbler");

    // Copy binary
    println!("Copying binary to app bundle...");
    fs::copy(&current_exe, &target_binary)?;

    // Set executable permissions
    println!("Setting executable permissions...");
    let mut perms = fs::metadata(&target_binary)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&target_binary, perms)?;

    // Create Info.plist
    println!("Creating Info.plist...");
    let version = env!("CARGO_PKG_VERSION");
    let info_plist = INFO_PLIST_TEMPLATE.replace("{VERSION}", version);
    let plist_path = contents_dir.join("Info.plist");
    fs::write(&plist_path, info_plist)?;

    println!("\n✅ Successfully installed OSX Scrobbler!");
    println!("\nApp bundle location:");
    println!("  {}", app_path.display());
    println!("\nTo launch the app:");
    println!("  open \"{}\"\n", app_path.display());
    println!("Or simply open it from Finder.\n");
    println!("💡 To start at login:");
    println!("  System Settings → General → Login Items → Add \"OSX Scrobbler\"\n");

    Ok(())
}

/// Uninstall the app bundle from /Applications/
fn handle_uninstall_app() -> Result<()> {
    use std::fs;
    use std::io::Write;

    println!("OSX Scrobbler App Bundle Uninstaller");
    println!("====================================\n");

    let app_name = "OSX Scrobbler.app";
    let app_path = std::path::Path::new("/Applications").join(app_name);

    // Check if app exists
    if !app_path.exists() {
        println!("❌ App bundle not found at {}", app_path.display());
        println!("\nNothing to uninstall.");
        return Ok(());
    }

    // Confirm with user
    print!("Remove app bundle at {}? [y/N] ", app_path.display());
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if !input.trim().eq_ignore_ascii_case("y") {
        println!("Uninstallation cancelled.");
        return Ok(());
    }

    // Remove app bundle
    println!("\nRemoving app bundle...");
    match fs::remove_dir_all(&app_path) {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
            eprintln!("\n❌ Permission denied removing app bundle.");
            eprintln!("\nTry running with sudo:");
            eprintln!("  sudo osx-scrobbler --uninstall-app\n");
            return Err(e.into());
        }
        Err(e) => return Err(e.into()),
    }

    println!("\n✅ Successfully uninstalled OSX Scrobbler!");
    println!("\nThe app bundle has been removed from /Applications/");
    println!("The binary at ~/.cargo/bin/osx-scrobbler is still available.\n");

    Ok(())
}
