# OSX Scrobbler - Development Tracker

## Project Overview
A macOS-specific scrobbling application that monitors now playing information using the `media_remote` crate and submits scrobbles to Last.fm and/or ListenBrainz services.

**Inspiration**: [rescrobbled](https://github.com/InputUsername/rescrobbled) - but using `media_remote` for macOS integration instead of MPRIS.

## Requirements

### Core Functionality
- [ ] Use `media_remote` crate to get now playing information from macOS media controls
- [ ] Submit scrobbles to Last.fm
- [ ] Submit scrobbles to ListenBrainz (support multiple instances)
- [ ] Track "now playing" and submit scrobbles when appropriate (after song plays for certain duration)
- [ ] Configurable refresh interval for polling now playing status

### Configuration
- [ ] Support configuration file at XDG config directory: `~/.config/osx_scrobbler.conf` (or appropriate XDG location)
- [ ] Create default configuration file if not present
- [ ] Configuration schema:
  - [ ] Last.fm settings (API key, secret, session key)
  - [ ] Multiple ListenBrainz instances (token, API URL)
  - [ ] Refresh interval for now playing polling
  - [ ] Scrobble threshold (% or seconds of track played before scrobbling)
  - [ ] Launch at login setting

### User Interface
- [ ] Status bar (menu bar) icon only - no dock presence
- [ ] System tray menu with:
  - [ ] Display last scrobbled song
  - [ ] Display current now playing
  - [ ] Toggle "Start with system" option
  - [ ] Quit option
  - [ ] Preferences/Settings option (optional)
- [ ] Launch at system startup support

### Technical Requirements
- [ ] macOS-specific implementation
- [ ] Efficient background operation
- [ ] Handle media player state changes (play, pause, stop, track change)
- [ ] Deduplication - don't scrobble the same play multiple times
- [ ] Error handling for network failures
- [ ] Logging for debugging

### Development Process
- [ ] Create git commits for distinct changes to maintain clear history
- [ ] Each commit should represent a logical, self-contained change
- [ ] Use descriptive commit messages that explain what and why

## Technical Architecture

### Components to Implement

#### 1. Media Monitoring
- **Module**: `media_monitor.rs`
- Track state:
  - Current track info (title, artist, album, duration)
  - Play state (playing, paused, stopped)
  - Play position
  - Timestamp when track started playing
- Poll using `media_remote` at configurable interval
- Detect track changes
- Calculate scrobble eligibility

#### 2. Configuration Management
- **Module**: `config.rs`
- Config file format (TOML recommended for Rust)
- XDG base directory support (use `dirs` or `xdg` crate)
- Default config generation
- Config validation
- Hot reload support (optional)

#### 3. Scrobbling Services
- **Module**: `scrobbler/mod.rs`
  - `scrobbler/lastfm.rs` - Last.fm API client
  - `scrobbler/listenbrainz.rs` - ListenBrainz API client
  - `scrobbler/traits.rs` - Common scrobbler trait
- Implement "now playing" updates
- Implement scrobble submission
- Queue management for offline/failed scrobbles
- Rate limiting and retry logic

#### 4. Status Bar UI
- **Module**: `ui/tray.rs`
- macOS menu bar integration (use `tao` + `muda` or similar)
- Dynamic menu updates
- Event handling

#### 5. System Integration
- **Module**: `system.rs`
- Launch at login configuration (use `auto-launch` crate)
- Background service management

#### 6. State Management
- **Module**: `state.rs`
- Application state
- Last scrobbled track
- Scrobble queue
- Persistence across restarts

## Dependencies to Add

```toml
[dependencies]
media-remote = "0.2.3"  # Already present
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"  # Config file format
dirs = "5.0"  # XDG directory support
reqwest = { version = "0.12", features = ["json"] }  # HTTP client
tokio = { version = "1", features = ["full"] }  # Async runtime
anyhow = "1.0"  # Error handling
log = "0.4"  # Logging
env_logger = "0.11"  # Simple logger implementation
chrono = "0.4"  # Time handling
md5 = "0.7"  # For Last.fm API signing
tao = "0.16"  # Window/menu management
muda = "0.11"  # Menu bar support
auto-launch = "0.5"  # Launch at login
```

## Implementation Tasks

### Phase 1: Configuration & Project Structure ✅
- [x] Set up project structure with modules
- [x] Add required dependencies to Cargo.toml
- [x] Implement config.rs with TOML support
- [x] Add XDG directory support
- [x] Create default config generation
- [x] Add config validation

### Phase 2: Scrobbling Services Integration
- [x] Define common Scrobbler trait
- [ ] Implement Last.fm API client
  - [ ] Authentication flow
  - [ ] Now playing updates
  - [ ] Scrobble submission
  - [ ] API signature generation
- [ ] Implement ListenBrainz API client
  - [ ] Token authentication
  - [ ] Now playing updates
  - [ ] Scrobble submission
  - [ ] Support multiple instances
- [ ] Add scrobble queue with persistence

### Phase 3: Media Monitoring ✅
- [x] Implement media_monitor.rs
- [x] Poll media state at configurable interval
- [x] Track play sessions
- [x] Detect track changes
- [x] Calculate scrobble eligibility (50% or 4 minutes rule)
- [x] Handle edge cases (pause, seek, repeat)
- [x] Deduplication logic

### Phase 4: Status Bar UI
- [ ] Create menu bar icon
- [ ] Implement system tray menu
- [ ] Display current now playing
- [ ] Display last scrobbled track
- [ ] Add "Start with system" toggle
- [ ] Add "Quit" option
- [ ] Handle menu item clicks

### Phase 5: System Integration
- [ ] Implement launch at login functionality
- [ ] Add persistence for launch preference
- [ ] Test startup behavior
- [ ] Ensure app runs in background (no dock icon)

### Phase 6: State Management & Persistence
- [ ] Implement application state
- [ ] Persist scrobble queue
- [ ] Save last scrobbled track
- [ ] Handle graceful shutdown

### Phase 7: Error Handling & Polish
- [ ] Add comprehensive error handling
- [ ] Implement retry logic for failed scrobbles
- [ ] Add logging throughout
- [ ] Handle network failures gracefully
- [ ] Test with various media players
- [ ] Documentation

## Current Status

**Phase**: Phase 2 - Scrobbling Services Integration
**Last Updated**: 2026-01-14

### Completed
- [x] Initial project setup with media_remote
- [x] Requirements gathering
- [x] Phase 1: Configuration & Project Structure
  - [x] Project structure with modules
  - [x] Dependencies added
  - [x] Configuration system with TOML
  - [x] XDG directory support
  - [x] Default config generation
  - [x] Config validation
- [x] Common Scrobbler trait defined
- [x] Phase 3: Media Monitoring
  - [x] Media monitoring with configurable polling
  - [x] Play session tracking
  - [x] Track change detection
  - [x] Scrobble eligibility calculation (Last.fm rules)
  - [x] Edge case handling (pause, deduplication)
  - [x] Event system (now playing, scrobble)

### In Progress
- [ ] Phase 2: Scrobbling Services Integration

### Next Steps
1. Implement Last.fm API client (authentication, now playing, scrobble)
2. Implement ListenBrainz API client
3. Integrate scrobblers with media monitor
4. Add scrobble queue with persistence

## Configuration File Example

```toml
# osx_scrobbler.conf

# Refresh interval in seconds
refresh_interval = 5

# Scrobble after playing this percentage of the track (or 4 minutes, whichever comes first)
scrobble_threshold = 50

# Launch at system startup
launch_at_login = false

[lastfm]
enabled = true
api_key = "your_api_key_here"
api_secret = "your_api_secret_here"
# Session key is obtained after authentication
session_key = ""

[[listenbrainz]]
enabled = true
name = "Primary"
token = "your_listenbrainz_token"
api_url = "https://api.listenbrainz.org"

# Example of multiple ListenBrainz instances
# [[listenbrainz]]
# enabled = false
# name = "Self-hosted"
# token = "your_token"
# api_url = "https://your.instance.com"
```

## Notes & Decisions

### Scrobble Rules (from Last.fm specification)
1. Track must be longer than 30 seconds
2. Scrobble after:
   - Track has been playing for at least 50% of its duration, OR
   - Track has been playing for at least 4 minutes (whichever comes first)
3. Submit at most once per track per play session

### macOS Menu Bar
- Use `tao` for window management (cross-platform but works well on macOS)
- Use `muda` for menu bar integration
- Ensure app doesn't appear in dock (set appropriate Info.plist settings or use NSApplication settings)

### Authentication
- **Last.fm**: Requires web-based authentication flow to get session key (one-time setup)
- **ListenBrainz**: Simple token-based authentication

## References
- [rescrobbled GitHub](https://github.com/InputUsername/rescrobbled)
- [Last.fm API Documentation](https://www.last.fm/api)
- [ListenBrainz API Documentation](https://listenbrainz.readthedocs.io/)
- [media_remote crate](https://crates.io/crates/media-remote)
