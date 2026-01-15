# Changelog

## [Unreleased]

## [0.3.0] - 2026-01-14

### Added
- **Self-installation capability** - `--install-app` flag creates macOS app bundle in /Applications/
- **Uninstall command** - `--uninstall-app` flag removes the app bundle cleanly
- Embedded Info.plist template in binary (no external files needed)

### Changed
- **Simplified installation** - Now uses `cargo install` + `--install-app` as the primary method
- Removed unnecessary conditional compilation checks (macOS-only app)

### Removed
- **build-app.sh script** - No longer needed (binary can self-install)
- **Info.plist file** - Now embedded in binary

## [0.2.0]

### Added
- Menu bar tray icon with current track display
- macOS .app bundle installer for proper menu bar integration
- Text cleanup with configurable regex patterns
- Smart logging (console when run from terminal, file when run as app)

## [0.1.0] - Initial Release

### Added
- Basic scrobbling functionality for Last.fm and listenbrainz
- Configuration file support
- Media player monitoring
