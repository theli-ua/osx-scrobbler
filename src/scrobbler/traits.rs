// Common traits for scrobbling services

use anyhow::Result;

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
    fn now_playing(&self, track: &Track) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Submit a scrobble
    fn scrobble(&self, track: &Track, timestamp: i64) -> impl std::future::Future<Output = Result<()>> + Send;
}
