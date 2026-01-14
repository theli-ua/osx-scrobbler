// System tray implementation

use anyhow::{Context, Result};
use std::sync::{Arc, RwLock};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    TrayIcon, TrayIconBuilder,
};

/// Shared state for the tray icon
#[derive(Debug, Clone, Default)]
pub struct TrayState {
    pub now_playing: Option<String>,
    pub last_scrobbled: Option<String>,
}

/// System tray manager
pub struct TrayManager {
    _tray_icon: TrayIcon,
    state: Arc<RwLock<TrayState>>,
    #[allow(dead_code)]
    menu: Menu,
    now_playing_item: MenuItem,
    last_scrobble_item: MenuItem,
    quit_item: MenuItem,
}

impl TrayManager {
    /// Create a new tray manager
    pub fn new() -> Result<Self> {
        let state = Arc::new(RwLock::new(TrayState::default()));

        // Create menu items
        let now_playing_item = MenuItem::new("Now Playing: None", false, None);
        let last_scrobble_item = MenuItem::new("Last Scrobbled: None", false, None);
        let separator = PredefinedMenuItem::separator();
        let quit_item = MenuItem::new("Quit", true, None);

        // Build menu
        let menu = Menu::new();
        menu.append(&now_playing_item)
            .context("Failed to add now playing item")?;
        menu.append(&last_scrobble_item)
            .context("Failed to add last scrobble item")?;
        menu.append(&separator)
            .context("Failed to add separator")?;
        menu.append(&quit_item)
            .context("Failed to add quit item")?;

        // Create tray icon
        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu.clone()))
            .with_tooltip("OSX Scrobbler")
            .build()
            .context("Failed to create tray icon")?;

        Ok(Self {
            _tray_icon: tray_icon,
            state,
            menu,
            now_playing_item,
            last_scrobble_item,
            quit_item,
        })
    }

    /// Get a clone of the state for updating
    #[allow(dead_code)]
    pub fn state(&self) -> Arc<RwLock<TrayState>> {
        self.state.clone()
    }

    /// Update the now playing display
    pub fn update_now_playing(&self, track: Option<String>) -> Result<()> {
        let text = if let Some(ref t) = track {
            format!("Now Playing: {}", t)
        } else {
            "Now Playing: None".to_string()
        };

        self.now_playing_item.set_text(text);

        let mut state = self.state.write().unwrap();
        state.now_playing = track;

        Ok(())
    }

    /// Update the last scrobbled display
    pub fn update_last_scrobbled(&self, track: Option<String>) -> Result<()> {
        let text = if let Some(ref t) = track {
            format!("Last Scrobbled: {}", t)
        } else {
            "Last Scrobbled: None".to_string()
        };

        self.last_scrobble_item.set_text(text);

        let mut state = self.state.write().unwrap();
        state.last_scrobbled = track;

        Ok(())
    }

    /// Check for menu events and return true if quit was clicked
    pub fn handle_events(&self) -> bool {
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == self.quit_item.id() {
                log::info!("Quit menu item clicked");
                return true;
            }
        }
        false
    }
}
