// Text cleanup module
// Applies regex patterns to clean up track/album/artist names

use crate::config::CleanupConfig;
use regex::Regex;

pub struct TextCleaner {
    enabled: bool,
    patterns: Vec<Regex>,
}

impl TextCleaner {
    /// Create a new text cleaner from config
    pub fn new(config: &CleanupConfig) -> Self {
        let patterns = if config.enabled {
            config
                .patterns
                .iter()
                .filter_map(|pattern| {
                    match Regex::new(pattern) {
                        Ok(re) => Some(re),
                        Err(e) => {
                            log::warn!("Invalid regex pattern '{}': {}", pattern, e);
                            None
                        }
                    }
                })
                .collect()
        } else {
            Vec::new()
        };

        Self {
            enabled: config.enabled,
            patterns,
        }
    }

    /// Clean a text string by applying all patterns
    pub fn clean(&self, text: &str) -> String {
        if !self.enabled {
            return text.to_string();
        }

        let mut result = text.to_string();
        for pattern in &self.patterns {
            result = pattern.replace_all(&result, "").to_string();
        }

        // Trim any extra whitespace
        result.trim().to_string()
    }

    /// Clean an optional string
    pub fn clean_option(&self, text: Option<String>) -> Option<String> {
        text.map(|s| self.clean(&s))
    }
}
