// Media monitoring module
// Polls macOS media remote for now playing information

use crate::config::AppFilteringConfig;
use crate::scrobbler::Track;
use crate::text_cleanup::TextCleaner;
use anyhow::Result;
use chrono::{DateTime, Utc};
use media_remote::prelude::*;
use media_remote::NowPlayingInfo;
use objc2::rc::autoreleasepool;
use std::time::Duration;
use std::time::SystemTime;

const MIN_TRACK_DURATION: u64 = 30; // Minimum track duration in seconds to scrobble
const SCROBBLE_TIME_THRESHOLD: u64 = 240; // 4 minutes in seconds

/// Action to take based on app filtering
#[derive(Debug, PartialEq)]
enum AppFilterAction {
    Allow,
    Ignore,
    PromptUser,
}

/// Represents the current play session state
#[derive(Debug, Clone)]
struct PlaySession {
    track: Track,
    bundle_id: Option<String>,
    started_at: DateTime<Utc>,
    duration: u64, // Track duration in seconds
    scrobbled: bool,
    now_playing_sent: bool,
    info_update_time: Option<SystemTime>,
}

impl PlaySession {
    fn new(
        track: Track,
        bundle_id: Option<String>,
        duration: u64,
        info_update_time: Option<SystemTime>,
    ) -> Self {
        Self {
            track,
            bundle_id,
            started_at: Utc::now(),
            duration,
            scrobbled: false,
            now_playing_sent: false,
            info_update_time,
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
    current_session: Option<PlaySession>,
    text_cleaner: TextCleaner,
}

impl MediaMonitor {
    pub fn new(scrobble_threshold: u8, text_cleaner: TextCleaner) -> Self {
        Self {
            now_playing: NowPlayingJXA::new(Duration::from_secs(30)),
            scrobble_threshold,
            current_session: None,
            text_cleaner,
        }
    }

    /// Check if an app should be scrobbled based on filtering config
    fn should_scrobble_app(
        &self,
        bundle_id: &Option<String>,
        app_filtering: &AppFilteringConfig,
    ) -> AppFilterAction {
        match bundle_id {
            None => {
                // No bundle ID - use scrobble_unknown setting
                if app_filtering.scrobble_unknown {
                    AppFilterAction::Allow
                } else {
                    AppFilterAction::Ignore
                }
            }
            Some(id) if id.is_empty() => {
                // Empty bundle ID - treat as None
                if app_filtering.scrobble_unknown {
                    AppFilterAction::Allow
                } else {
                    AppFilterAction::Ignore
                }
            }
            Some(id) => {
                // Check allowed list first
                if app_filtering.allowed_apps.contains(id) {
                    return AppFilterAction::Allow;
                }
                // Check ignored list
                if app_filtering.ignored_apps.contains(id) {
                    return AppFilterAction::Ignore;
                }
                // Unknown app - prompt if enabled
                if app_filtering.prompt_for_new_apps {
                    AppFilterAction::PromptUser
                } else {
                    // Don't prompt, default to allowing
                    AppFilterAction::Allow
                }
            }
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
    pub fn poll(&mut self, app_filtering: &AppFilteringConfig) -> Result<MediaEvents> {
        // Clone media info to avoid holding the guard
        let media_info = autoreleasepool(|_| {
            let guard = self.now_playing.get_info();
            guard.as_ref().cloned()
        });

        let mut events = MediaEvents::default();

        if let Some(info) = media_info {
            // Check if media is playing (not paused)
            let is_playing = info.is_playing.unwrap_or(false);

            if !is_playing {
                // Media is paused or stopped - don't start new session
                // but keep existing session in case playback resumes
                return Ok(events);
            }
            log::debug!("now playing info: {:?}", info);

            if let Some(track) = self.media_info_to_track(&info) {
                let duration = track.duration.unwrap_or(0);
                let bundle_id = info.bundle_id.clone();

                // Check if we should scrobble from this app
                match self.should_scrobble_app(&bundle_id, app_filtering) {
                    AppFilterAction::Ignore => {
                        log::debug!("Ignoring playback from {:?}", bundle_id);
                        return Ok(events);
                    }
                    AppFilterAction::PromptUser => {
                        // Emit event to prompt user
                        if let Some(ref id) = bundle_id {
                            events.unknown_app = Some(id.clone());
                        }
                        return Ok(events);
                    }
                    AppFilterAction::Allow => {
                        // Continue with normal processing
                    }
                }

                // Check if this is a new track or continuation
                let is_new_track = match &self.current_session {
                    None => true,
                    Some(session) => {
                        // Check if track changed
                        session.track != track || session.info_update_time != info.info_update_time
                    }
                };

                if is_new_track {
                    // New track started
                    log::info!(
                        "New track: {} - {} ({}s) from {:?}",
                        track.artist,
                        track.title,
                        duration,
                        bundle_id
                    );

                    let mut new_session = PlaySession::new(
                        track.clone(),
                        bundle_id.clone(),
                        duration,
                        info.info_update_time,
                    );
                    new_session.now_playing_sent = true; // Mark as sent immediately
                    self.current_session = Some(new_session);

                    // Send now playing update
                    events.now_playing = Some((track, bundle_id));
                } else if let Some(session) = self.current_session.as_mut() {
                    // Same track, check if we should scrobble
                    if session.should_scrobble(self.scrobble_threshold) {
                        log::info!(
                            "Scrobbling: {} - {} (played {}s / {}s)",
                            session.track.artist,
                            session.track.title,
                            session.elapsed_seconds(),
                            session.duration
                        );

                        events.scrobble = Some((
                            session.track.clone(),
                            session.started_at,
                            session.bundle_id.clone(),
                        ));
                        session.scrobbled = true;
                    } else if session.should_send_now_playing() {
                        // Send now playing update if not sent yet
                        events.now_playing =
                            Some((session.track.clone(), session.bundle_id.clone()));
                        session.now_playing_sent = true;
                    }
                }
            }
        } else {
            // No media playing, clear session
            if self.current_session.is_some() {
                log::info!("Media stopped, clearing session");
                self.current_session = None;
            }
        }

        Ok(events)
    }
}

/// Events generated by media monitoring
#[derive(Debug, Default)]
pub struct MediaEvents {
    pub now_playing: Option<(Track, Option<String>)>,
    pub scrobble: Option<(Track, DateTime<Utc>, Option<String>)>,
    pub unknown_app: Option<String>,
}
