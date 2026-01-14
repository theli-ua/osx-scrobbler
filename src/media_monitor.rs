// Media monitoring module
// Polls macOS media remote for now playing information

use std::time::Duration;

/// Placeholder for media monitoring implementation
pub struct MediaMonitor {
    refresh_interval: Duration,
}

impl MediaMonitor {
    pub fn new(refresh_interval: Duration) -> Self {
        Self { refresh_interval }
    }
}
