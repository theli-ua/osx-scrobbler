// System tray implementation

use anyhow::{Context, Result};
use auto_launch::AutoLaunch;
use std::sync::{Arc, RwLock};
use tray_icon::{
    menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    Icon, TrayIcon, TrayIconBuilder,
};

/// Create a simple icon for the tray
fn create_icon() -> Result<Icon> {
    // Create a simple 16x16 icon with a musical note
    // This is a basic icon - in production, you'd use a proper PNG/ICO file
    let width = 16;
    let height = 16;
    let mut rgba = vec![0u8; width * height * 4];

    // Draw a simple musical note shape
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 4;

            // Create a simple note pattern
            let is_note = (x >= 6 && x <= 7 && y >= 3 && y <= 14) // stem
                || (x >= 8 && x <= 10 && y >= 3 && y <= 5) // flag
                || (x >= 4 && x <= 9 && y >= 12 && y <= 15); // head

            if is_note {
                rgba[idx] = 255;     // R
                rgba[idx + 1] = 255; // G
                rgba[idx + 2] = 255; // B
                rgba[idx + 3] = 255; // A
            }
        }
    }

    Icon::from_rgba(rgba, width as u32, height as u32)
        .context("Failed to create icon from RGBA data")
}

/// Shared state for the tray icon
#[derive(Debug, Clone, Default)]
pub struct TrayState {
    pub now_playing: Option<String>,
    pub last_scrobbled: Option<String>,
}

/// Events that can be triggered from the tray menu
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayEvent {
    Quit,
    ToggleLaunchAtLogin,
}

/// System tray manager
pub struct TrayManager {
    _tray_icon: TrayIcon,
    state: Arc<RwLock<TrayState>>,
    #[allow(dead_code)]
    menu: Menu,
    now_playing_item: MenuItem,
    last_scrobble_item: MenuItem,
    launch_at_login_item: CheckMenuItem,
    quit_item: MenuItem,
    auto_launch: AutoLaunch,
}

impl TrayManager {
    /// Create a new tray manager
    pub fn new(launch_at_login: bool) -> Result<Self> {
        let state = Arc::new(RwLock::new(TrayState::default()));

        // Set up auto-launch
        let auto_launch = AutoLaunch::new(
            "OSX Scrobbler",
            &std::env::current_exe()
                .context("Failed to get current executable path")?
                .to_string_lossy(),
            false,
            &[] as &[&str],
        );

        // Sync auto-launch state with config
        let is_enabled = auto_launch.is_enabled().unwrap_or(false);
        if launch_at_login && !is_enabled {
            if let Err(e) = auto_launch.enable() {
                log::warn!("Failed to enable launch at login: {}", e);
            }
        } else if !launch_at_login && is_enabled {
            if let Err(e) = auto_launch.disable() {
                log::warn!("Failed to disable launch at login: {}", e);
            }
        }

        // Create menu items
        let now_playing_item = MenuItem::new("Now Playing: None", false, None);
        let last_scrobble_item = MenuItem::new("Last Scrobbled: None", false, None);
        let separator1 = PredefinedMenuItem::separator();
        let launch_at_login_item = CheckMenuItem::new("Launch at Login", true, launch_at_login, None);
        let separator2 = PredefinedMenuItem::separator();
        let quit_item = MenuItem::new("Quit", true, None);

        // Build menu
        let menu = Menu::new();
        menu.append(&now_playing_item)
            .context("Failed to add now playing item")?;
        menu.append(&last_scrobble_item)
            .context("Failed to add last scrobble item")?;
        menu.append(&separator1)
            .context("Failed to add separator")?;
        menu.append(&launch_at_login_item)
            .context("Failed to add launch at login item")?;
        menu.append(&separator2)
            .context("Failed to add separator")?;
        menu.append(&quit_item)
            .context("Failed to add quit item")?;

        // Create tray icon
        let icon = create_icon()?;
        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu.clone()))
            .with_tooltip("OSX Scrobbler")
            .with_icon(icon)
            .build()
            .context("Failed to create tray icon")?;

        Ok(Self {
            _tray_icon: tray_icon,
            state,
            menu,
            now_playing_item,
            last_scrobble_item,
            launch_at_login_item,
            quit_item,
            auto_launch,
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

    /// Toggle launch at login
    pub fn toggle_launch_at_login(&self) -> Result<bool> {
        let is_enabled = self.auto_launch.is_enabled().unwrap_or(false);
        let new_state = !is_enabled;

        if new_state {
            self.auto_launch
                .enable()
                .context("Failed to enable launch at login")?;
            log::info!("Launch at login enabled");
        } else {
            self.auto_launch
                .disable()
                .context("Failed to disable launch at login")?;
            log::info!("Launch at login disabled");
        }

        self.launch_at_login_item.set_checked(new_state);

        Ok(new_state)
    }

    /// Check for menu events and return the event if any
    pub fn handle_events(&self) -> Option<TrayEvent> {
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == self.quit_item.id() {
                log::info!("Quit menu item clicked");
                return Some(TrayEvent::Quit);
            } else if event.id == self.launch_at_login_item.id() {
                log::info!("Launch at login toggle clicked");
                return Some(TrayEvent::ToggleLaunchAtLogin);
            }
        }
        None
    }
}
