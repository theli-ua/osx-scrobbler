// Media monitoring module
// Polls macOS media remote for now playing information

use crate::scrobbler::Track;
use crate::text_cleanup::TextCleaner;
use anyhow::Result;
use chrono::{DateTime, Utc};
use media_remote::prelude::*;
use media_remote::NowPlayingInfo;
use std::sync::{Arc, RwLock};
use std::time::Duration;

const MIN_TRACK_DURATION: u64 = 30; // Minimum track duration in seconds to scrobble
const SCROBBLE_TIME_THRESHOLD: u64 = 240; // 4 minutes in seconds

/// Represents the current play session state
#[derive(Debug, Clone)]
struct PlaySession {
    track: Track,
    started_at: DateTime<Utc>,
    duration: u64, // Track duration in seconds
    scrobbled: bool,
    now_playing_sent: bool,
}

impl PlaySession {
    fn new(track: Track, duration: u64) -> Self {
        Self {
            track,
            started_at: Utc::now(),
            duration,
            scrobbled: false,
            now_playing_sent: false,
        }
    }

    /// Calculate elapsed play time in seconds
    fn elapsed_seconds(&self) -> u64 {
        let elapsed = Utc::now().signed_duration_since(self.started_at);
        elapsed.num_seconds().max(0) as u64
    }

    /// Check if track should be scrobbled based on Last.fm rules
    fn should_scrobble(&self, threshold_percent: u8) -> bool {
        if self.scrobbled {
            return false;
        }

        // Track must be at least 30 seconds long
        if self.duration < MIN_TRACK_DURATION {
            return false;
        }

        let elapsed = self.elapsed_seconds();

        // Scrobble after 50% (configurable) of the track OR 4 minutes, whichever comes first
        let threshold_time = (self.duration * threshold_percent as u64) / 100;
        let scrobble_at = threshold_time.min(SCROBBLE_TIME_THRESHOLD);

        elapsed >= scrobble_at
    }

    /// Check if we should send "now playing" update
    fn should_send_now_playing(&self) -> bool {
        !self.now_playing_sent
    }
}

/// Media monitor that polls macOS media remote
pub struct MediaMonitor {
    now_playing: NowPlayingJXA,
    scrobble_threshold: u8,
    current_session: Arc<RwLock<Option<PlaySession>>>,
    text_cleaner: TextCleaner,
}

impl MediaMonitor {
    pub fn new(_refresh_interval: Duration, scrobble_threshold: u8, text_cleaner: TextCleaner) -> Self {
        Self {
            now_playing: NowPlayingJXA::new(Duration::from_secs(30)),
            scrobble_threshold,
            current_session: Arc::new(RwLock::new(None)),
            text_cleaner,
        }
    }

    /// Convert media_remote NowPlayingInfo to our Track structure
    fn media_info_to_track(&self, info: &NowPlayingInfo) -> Option<Track> {
        let title = info.title.clone()?;
        let artist = info.artist.clone()?;
        let album = info.album.clone();

        // Apply text cleanup
        let title = self.text_cleaner.clean(&title);
        let artist = self.text_cleaner.clean(&artist);
        let album = self.text_cleaner.clean_option(album);

        Some(Track {
            title,
            artist,
            album,
            duration: info.duration.map(|d| d as u64),
        })
    }

    /// Check for track changes and return events (now playing, scrobble)
    pub fn poll(&self) -> Result<MediaEvents> {
        // Clone media info to avoid holding the guard
        let media_info = {
            let guard = self.now_playing.get_info();
            guard.as_ref().cloned()
        };

        let mut events = MediaEvents::default();

        if let Some(info) = media_info {
            // Check if media is playing (not paused)
            let is_playing = info.is_playing.unwrap_or(false);

            if !is_playing {
                // Media is paused or stopped - don't start new session
                // but keep existing session in case playback resumes
                return Ok(events);
            }

            if let Some(track) = self.media_info_to_track(&info) {
                let duration = track.duration.unwrap_or(0);

                let mut session_lock = self.current_session.write().unwrap();

                // Check if this is a new track or continuation
                let is_new_track = match &*session_lock {
                    None => true,
                    Some(session) => {
                        // Check if track changed
                        session.track.title != track.title
                            || session.track.artist != track.artist
                            || session.track.album != track.album
                    }
                };

                if is_new_track {
                    // New track started
                    log::info!(
                        "New track: {} - {} ({}s)",
                        track.artist,
                        track.title,
                        duration
                    );

                    let mut new_session = PlaySession::new(track.clone(), duration);
                    new_session.now_playing_sent = true; // Mark as sent immediately
                    *session_lock = Some(new_session);

                    // Send now playing update
                    events.now_playing = Some(track);
                } else if let Some(session) = session_lock.as_mut() {
                    // Same track, check if we should scrobble
                    if session.should_scrobble(self.scrobble_threshold) {
                        log::info!(
                            "Scrobbling: {} - {} (played {}s / {}s)",
                            session.track.artist,
                            session.track.title,
                            session.elapsed_seconds(),
                            session.duration
                        );

                        events.scrobble = Some((session.track.clone(), session.started_at));
                        session.scrobbled = true;
                    } else if session.should_send_now_playing() {
                        // Send now playing update if not sent yet
                        events.now_playing = Some(session.track.clone());
                        session.now_playing_sent = true;
                    }
                }
            }
        } else {
            // No media playing, clear session
            let mut session_lock = self.current_session.write().unwrap();
            if session_lock.is_some() {
                log::info!("Media stopped, clearing session");
                *session_lock = None;
            }
        }

        Ok(events)
    }
}

/// Events generated by media monitoring
#[derive(Debug, Default)]
pub struct MediaEvents {
    pub now_playing: Option<Track>,
    pub scrobble: Option<(Track, DateTime<Utc>)>,
}

impl MediaEvents {
    #[allow(dead_code)]
    fn has_events(&self) -> bool {
        self.now_playing.is_some() || self.scrobble.is_some()
    }
}
