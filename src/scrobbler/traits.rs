// Common traits for scrobbling services

use anyhow::Result;
use std::future::Future;
use std::pin::Pin;

/// Track information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Track {
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub duration: Option<u64>, // Duration in seconds
}

/// Common trait for all scrobbling services
pub trait Scrobbler: Send + Sync {
    /// Update "now playing" status
    fn now_playing(&self, track: &Track) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;

    /// Submit a scrobble
    fn scrobble(&self, track: &Track, timestamp: i64) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;
}
