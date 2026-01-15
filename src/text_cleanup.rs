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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disabled_cleaner_returns_unchanged() {
        let config = CleanupConfig {
            enabled: false,
            patterns: vec![r"\s*\[Explicit\]".to_string()],
        };
        let cleaner = TextCleaner::new(&config);

        assert_eq!(cleaner.clean("Song [Explicit]"), "Song [Explicit]");
    }

    #[test]
    fn test_removes_explicit_tags() {
        let config = CleanupConfig {
            enabled: true,
            patterns: vec![
                r"\s*\[Explicit\]".to_string(),
                r"\s*\(Explicit\)".to_string(),
            ],
        };
        let cleaner = TextCleaner::new(&config);

        assert_eq!(cleaner.clean("Song [Explicit]"), "Song");
        assert_eq!(cleaner.clean("Song (Explicit)"), "Song");
        assert_eq!(cleaner.clean("Song [Explicit] (Explicit)"), "Song");
    }

    #[test]
    fn test_removes_clean_tags() {
        let config = CleanupConfig {
            enabled: true,
            patterns: vec![r"\s*\[Clean\]".to_string()],
        };
        let cleaner = TextCleaner::new(&config);

        assert_eq!(cleaner.clean("Song [Clean]"), "Song");
    }

    #[test]
    fn test_trims_whitespace() {
        let config = CleanupConfig {
            enabled: true,
            patterns: vec![r"\s*\[Explicit\]".to_string()],
        };
        let cleaner = TextCleaner::new(&config);

        assert_eq!(cleaner.clean("  Song [Explicit]  "), "Song");
    }

    #[test]
    fn test_multiple_patterns() {
        let config = CleanupConfig {
            enabled: true,
            patterns: vec![
                r"\s*\[Explicit\]".to_string(),
                r"\s*- Remastered.*".to_string(),
            ],
        };
        let cleaner = TextCleaner::new(&config);

        assert_eq!(
            cleaner.clean("Song [Explicit] - Remastered 2020"),
            "Song"
        );
    }

    #[test]
    fn test_clean_option_with_some() {
        let config = CleanupConfig {
            enabled: true,
            patterns: vec![r"\s*\[Explicit\]".to_string()],
        };
        let cleaner = TextCleaner::new(&config);

        assert_eq!(
            cleaner.clean_option(Some("Song [Explicit]".to_string())),
            Some("Song".to_string())
        );
    }

    #[test]
    fn test_clean_option_with_none() {
        let config = CleanupConfig {
            enabled: true,
            patterns: vec![r"\s*\[Explicit\]".to_string()],
        };
        let cleaner = TextCleaner::new(&config);

        assert_eq!(cleaner.clean_option(None), None);
    }

    #[test]
    fn test_invalid_pattern_is_skipped() {
        let config = CleanupConfig {
            enabled: true,
            patterns: vec![
                r"[invalid(".to_string(), // Invalid regex
                r"\s*\[Explicit\]".to_string(),
            ],
        };
        let cleaner = TextCleaner::new(&config);

        // Should still clean with the valid pattern
        assert_eq!(cleaner.clean("Song [Explicit]"), "Song");
    }
}
