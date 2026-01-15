// System tray implementation

use anyhow::{Context, Result};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    Icon, TrayIcon, TrayIconBuilder,
};

/// Create a simple icon for the tray
fn create_icon() -> Result<Icon> {
    // Create a simple 22x22 template icon (macOS standard size)
    // Template icons are monochrome and automatically adapt to the menu bar theme
    let width = 22;
    let height = 22;
    let mut rgba = vec![0u8; width * height * 4];

    // Draw a simple, bold musical note
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 4;

            // Large note head (filled circle) - bottom area
            let note_head_x = 9.0;
            let note_head_y = 15.0;
            let note_head_radius = 4.0;
            let dx = x as f32 - note_head_x;
            let dy = y as f32 - note_head_y;
            let is_note_head = (dx * dx + dy * dy) <= (note_head_radius * note_head_radius);

            // Thick stem (vertical line) - from note head up
            let is_stem = (12..=14).contains(&x) && (3..=15).contains(&y);

            // Simple flag (diagonal line)
            let is_flag = ((14..=16).contains(&x) && (3..=5).contains(&y)) ||
                          ((15..=17).contains(&x) && (5..=7).contains(&y)) ||
                          ((16..=18).contains(&x) && (7..=9).contains(&y));

            if is_note_head || is_stem || is_flag {
                rgba[idx] = 0;       // R - black for template icons
                rgba[idx + 1] = 0;   // G
                rgba[idx + 2] = 0;   // B
                rgba[idx + 3] = 255; // A - fully opaque
            } else {
                rgba[idx + 3] = 0;   // Transparent background
            }
        }
    }

    log::info!("Creating tray icon with {}x{} pixels (musical note)", width, height);
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
}

/// System tray manager
pub struct TrayManager {
    _tray_icon: TrayIcon,
    state: TrayState,
    #[allow(dead_code)]
    menu: Menu,
    now_playing_item: MenuItem,
    last_scrobble_item: MenuItem,
    quit_item: MenuItem,
}

impl TrayManager {
    /// Create a new tray manager
    pub fn new() -> Result<Self> {
        let state = TrayState::default();

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
        let icon = create_icon()?;
        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu.clone()))
            .with_tooltip("OSX Scrobbler")
            .with_icon(icon)
            .with_icon_as_template(true)
            .build()
            .context("Failed to create tray icon")?;

        log::info!("Tray icon created successfully");

        Ok(Self {
            _tray_icon: tray_icon,
            state,
            menu,
            now_playing_item,
            last_scrobble_item,
            quit_item,
        })
    }

    /// Update the now playing display
    pub fn update_now_playing(&mut self, track: Option<String>) -> Result<()> {
        let text = if let Some(ref t) = track {
            format!("Now Playing: {}", t)
        } else {
            "Now Playing: None".to_string()
        };

        self.now_playing_item.set_text(text);
        self.state.now_playing = track;

        Ok(())
    }

    /// Update the last scrobbled display
    pub fn update_last_scrobbled(&mut self, track: Option<String>) -> Result<()> {
        let text = if let Some(ref t) = track {
            format!("Last Scrobbled: {}", t)
        } else {
            "Last Scrobbled: None".to_string()
        };

        self.last_scrobble_item.set_text(text);
        self.state.last_scrobbled = track;

        Ok(())
    }

    /// Check for menu events and return the event if any
    pub fn handle_events(&self) -> Option<TrayEvent> {
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == self.quit_item.id() {
                log::info!("Quit menu item clicked");
                return Some(TrayEvent::Quit);
            }
        }
        None
    }
}
