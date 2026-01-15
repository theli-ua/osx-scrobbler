// Configuration management module
// Handles loading, saving, and validating configuration

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Refresh interval in seconds for polling now playing status
    pub refresh_interval: u64,

    /// Scrobble after playing this percentage of the track (50% default)
    pub scrobble_threshold: u8,

    /// Text cleanup configuration
    #[serde(default)]
    pub cleanup: CleanupConfig,

    /// App filtering configuration
    #[serde(default)]
    pub app_filtering: AppFilteringConfig,

    /// Last.fm configuration
    pub lastfm: Option<LastFmConfig>,

    /// ListenBrainz configurations (can have multiple instances)
    pub listenbrainz: Vec<ListenBrainzConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupConfig {
    /// Enable text cleanup
    pub enabled: bool,

    /// Regex patterns to remove from track/album/artist names
    /// Applied in order, each pattern is removed from the text
    pub patterns: Vec<String>,
}

impl Default for CleanupConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            patterns: vec![
                r"\s*\[Explicit\]".to_string(),
                r"\s*\[Clean\]".to_string(),
                r"\s*\(Explicit\)".to_string(),
                r"\s*\(Clean\)".to_string(),
                r"\s*- Explicit".to_string(),
                r"\s*- Clean".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastFmConfig {
    pub enabled: bool,
    pub api_key: String,
    pub api_secret: String,
    pub session_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListenBrainzConfig {
    pub enabled: bool,
    pub name: String,
    pub token: String,
    pub api_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppFilteringConfig {
    /// Whether to prompt when encountering a new app
    pub prompt_for_new_apps: bool,

    /// Whether to scrobble from apps that don't provide bundle_id
    pub scrobble_unknown: bool,

    /// Apps to scrobble from (bundle IDs)
    pub allowed_apps: Vec<String>,

    /// Apps to ignore (bundle IDs)
    pub ignored_apps: Vec<String>,
}

impl Default for AppFilteringConfig {
    fn default() -> Self {
        Self {
            prompt_for_new_apps: true,
            scrobble_unknown: true,
            allowed_apps: Vec::new(),
            ignored_apps: Vec::new(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            refresh_interval: 5,
            scrobble_threshold: 50,
            cleanup: CleanupConfig::default(),
            app_filtering: AppFilteringConfig::default(),
            lastfm: Some(LastFmConfig {
                enabled: false,
                api_key: String::new(),
                api_secret: String::new(),
                session_key: String::new(),
            }),
            listenbrainz: vec![ListenBrainzConfig {
                enabled: false,
                name: "Primary".to_string(),
                token: String::new(),
                api_url: "https://api.listenbrainz.org".to_string(),
            }],
        }
    }
}

impl Config {
    /// Get the path to the configuration file
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to get config directory")?;

        Ok(config_dir.join("osx_scrobbler.conf"))
    }

    /// Load configuration from file, or create default if it doesn't exist
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            log::info!("Config file not found, creating default at {:?}", config_path);
            let default_config = Self::default();
            default_config.save()?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(&config_path)
            .context("Failed to read config file")?;

        let config: Config = toml::from_str(&content)
            .context("Failed to parse config file")?;

        config.validate()?;

        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }

        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;

        fs::write(&config_path, content)
            .context("Failed to write config file")?;

        log::info!("Config saved to {:?}", config_path);

        Ok(())
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate refresh interval
        if self.refresh_interval == 0 {
            anyhow::bail!("refresh_interval must be greater than 0");
        }

        // Validate scrobble threshold (should be 1-100%)
        if self.scrobble_threshold == 0 || self.scrobble_threshold > 100 {
            anyhow::bail!("scrobble_threshold must be between 1 and 100");
        }

        // Check that at least one scrobbler is enabled
        let lastfm_enabled = self.lastfm.as_ref().map(|l| l.enabled).unwrap_or(false);
        let listenbrainz_enabled = self.listenbrainz.iter().any(|l| l.enabled);

        if !lastfm_enabled && !listenbrainz_enabled {
            log::warn!("No scrobbling services are enabled");
        }

        // Validate Last.fm config if enabled
        if let Some(lastfm) = &self.lastfm {
            if lastfm.enabled {
                if lastfm.api_key.is_empty() {
                    anyhow::bail!("Last.fm api_key is required when Last.fm is enabled");
                }
                if lastfm.api_secret.is_empty() {
                    anyhow::bail!("Last.fm api_secret is required when Last.fm is enabled");
                }
            }
        }

        // Validate ListenBrainz configs if enabled
        for lb in &self.listenbrainz {
            if lb.enabled {
                if lb.token.is_empty() {
                    anyhow::bail!("ListenBrainz token is required when enabled (instance: {})", lb.name);
                }
                if lb.api_url.is_empty() {
                    anyhow::bail!("ListenBrainz api_url is required (instance: {})", lb.name);
                }
            }
        }

        // Validate app filtering - check for conflicts
        for bundle_id in &self.app_filtering.allowed_apps {
            if self.app_filtering.ignored_apps.contains(bundle_id) {
                anyhow::bail!(
                    "Bundle ID '{}' appears in both allowed_apps and ignored_apps",
                    bundle_id
                );
            }
        }

        Ok(())
    }
}
