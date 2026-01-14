mod config;
mod media_monitor;
mod scrobbler;

use anyhow::Result;

fn main() -> Result<()> {
    // Initialize logger
    env_logger::init();

    // Load configuration
    let config = config::Config::load()?;
    log::info!("Configuration loaded successfully");
    log::info!("Refresh interval: {}s", config.refresh_interval);
    log::info!("Scrobble threshold: {}%", config.scrobble_threshold);

    // TODO: Initialize scrobblers
    // TODO: Initialize media monitor
    // TODO: Initialize system tray
    // TODO: Start main event loop

    Ok(())
}
