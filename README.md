# OSX Scrobbler

A lightweight macOS menu bar application that scrobbles your music to Last.fm and ListenBrainz.

[![Crates.io](https://img.shields.io/crates/v/osx-scrobbler)](https://crates.io/crates/osx-scrobbler)
[![License](https://img.shields.io/crates/l/osx-scrobbler)](LICENSE)
![Platform](https://img.shields.io/badge/platform-macOS-lightgrey)

## Features

- 🎵 **Automatic Scrobbling** - Scrobbles to Last.fm and/or ListenBrainz
- 🎯 **macOS Native** - Uses macOS Media Remote for universal media player support
- 🧹 **Text Cleanup** - Configurable regex patterns to clean track/album names (removes `[Explicit]`, `[Clean]`, etc.)
- 🔄 **Multiple Services** - Support for multiple ListenBrainz instances
- 📊 **Menu Bar Integration** - Lightweight menu bar icon showing current track
- ⚡ **Efficient** - Low resource usage, runs silently in background

## Installation

### Prerequisites

- macOS 10.15 or later
- Rust toolchain (install from [rustup.rs](https://rustup.rs))

### Install from crates.io

The easiest way to install OSX Scrobbler:

```bash
# Install the binary
cargo install osx-scrobbler

# Create a macOS app bundle in /Applications/
osx-scrobbler --install-app
```

That's it! The app will:
- ✅ Show only in menu bar (no dock icon)
- ✅ Run silently in the background
- ✅ Log to `~/Library/Logs/osx-scrobbler.log`

**To start at login:** Add "OSX Scrobbler" to System Settings → General → Login Items

**Note:** If you get a permission error during installation, run with sudo:
```bash
sudo osx-scrobbler --install-app
```

## Configuration

The configuration file is located at:
```
~/Library/Application Support/osx_scrobbler.conf
```

A default configuration will be created automatically on first run.

### Basic Configuration

```toml
# Polling interval in seconds
refresh_interval = 5

# Scrobble after playing this % of the track (or 4 minutes, whichever comes first)
scrobble_threshold = 50
```

### Text Cleanup

Remove unwanted tags from track/album/artist names before scrobbling:

```toml
[cleanup]
enabled = true
patterns = [
    # Remove explicit/clean tags
    "\\s*\\[Explicit\\]",
    "\\s*\\[Clean\\]",
    "\\s*\\(Explicit\\)",
    "\\s*\\(Clean\\)",
    "\\s*- Explicit",
    "\\s*- Clean",
]
```

**Add custom patterns:**
```toml
patterns = [
    # ... default patterns ...

    # Remove featuring artists
    "\\s*\\(feat\\..*?\\)",
    "\\s*\\(ft\\..*?\\)",

    # Remove remastered tags
    "\\s*-\\s*Remastered.*",
    "\\s*\\(Remastered.*?\\)",

    # Remove year tags
    "\\s*\\(\\d{4}\\)",
]
```

Patterns are standard regex and are applied in order. Remember to escape special characters with `\\` in TOML.

### App Filtering

Control which apps OSX Scrobbler listens to for scrobbling. When a new app starts playing music, you'll be prompted to allow or ignore it.

```toml
[app_filtering]
# Whether to prompt when encountering a new app
prompt_for_new_apps = true

# Whether to scrobble from apps that don't provide bundle_id
scrobble_unknown = true

# Apps to scrobble from (bundle IDs)
allowed_apps = [
    "com.spotify.client",
    "com.apple.Music"
]

# Apps to ignore (bundle IDs)
ignored_apps = [
    "com.apple.Safari"  # Don't scrobble YouTube in browser
]
```

**How it works:**
- When music plays from a new app, a dialog will ask whether to allow or ignore scrobbling from that app
- Your choice is automatically saved to the config file
- You can manually edit `allowed_apps` and `ignored_apps` lists
- Apps without a bundle ID (rare) are controlled by the `scrobble_unknown` setting
- Disable prompts by setting `prompt_for_new_apps = false`

**Common bundle IDs:**
- Spotify: `com.spotify.client`
- Apple Music: `com.apple.Music`
- VLC: `org.videolan.vlc`
- Safari (for web players): `com.apple.Safari`
- Google Chrome: `com.google.Chrome`

## Setting Up Scrobbling Services

### Last.fm

#### 1. Get API Credentials

1. Create a Last.fm API account: https://www.last.fm/api/account/create
2. Note your **API Key** and **API Secret**

#### 2. Add to Config

```toml
[lastfm]
enabled = true
api_key = "your_api_key_here"
api_secret = "your_api_secret_here"
session_key = ""  # Leave empty initially
```

#### 3. Authenticate

Run the authentication helper:

```bash
osx-scrobbler --auth-lastfm
```

This will:
1. Generate an authorization token
2. Open a URL in your browser for you to authorize the app
3. Automatically fetch and save the session key to your config

After authentication, your config will have the `session_key` filled in and `enabled = true`.

### ListenBrainz

#### 1. Get Your Token

1. Go to https://listenbrainz.org/profile/
2. Log in or create an account
3. Navigate to "User Settings" → "Music Services"
4. Copy your **User Token**

#### 2. Add to Config

```toml
[[listenbrainz]]
enabled = true
name = "Primary"
token = "your_listenbrainz_token"
api_url = "https://api.listenbrainz.org"
```

#### 3. Multiple Instances (Optional)

You can scrobble to multiple ListenBrainz instances (e.g., self-hosted):

```toml
[[listenbrainz]]
enabled = true
name = "Primary"
token = "token_for_listenbrainz_org"
api_url = "https://api.listenbrainz.org"

[[listenbrainz]]
enabled = true
name = "Self-hosted"
token = "token_for_your_instance"
api_url = "https://your.instance.com"
```

## Usage

### Starting the App

Simply run:
```bash
osx-scrobbler
```

The app will:
- Show a musical note icon in your menu bar
- Monitor your media players automatically
- Scrobble tracks when you've played 50% or 4 minutes (whichever comes first)

### Menu Bar

Click the menu bar icon to see:
- **Now Playing** - Currently playing track
- **Last Scrobbled** - Most recently scrobbled track
- **Quit** - Exit the application

### Command Line Options

```bash
# Show help
osx-scrobbler --help

# Show version
osx-scrobbler --version

# Install as macOS app bundle in /Applications/
osx-scrobbler --install-app

# Uninstall the app bundle from /Applications/
osx-scrobbler --uninstall-app

# Authenticate with Last.fm
osx-scrobbler --auth-lastfm

# Force console output (show logs in terminal even when not running from one)
osx-scrobbler --console
```

### Logging

The app automatically detects how it's being run:

- **From Terminal**: Logs are shown in the terminal (stdout)
- **From Spotlight/Finder**: Logs are written to `~/Library/Logs/osx-scrobbler.log`
- **Force Console Mode**: Use `--console` flag to always show logs in terminal

To view logs when running in background:
```bash
tail -f ~/Library/Logs/osx-scrobbler.log
```

## How Scrobbling Works

The app follows Last.fm's scrobbling rules:

1. **Track must be at least 30 seconds long**
2. **Scrobble submitted after:**
   - Playing 50% of the track duration, OR
   - Playing for 4 minutes
   - (whichever comes first)
3. **Each track is scrobbled only once per play session**
4. **Pausing** doesn't reset the scrobble timer

## Supported Media Players

OSX Scrobbler works with **any media player that integrates with macOS Media Remote**, including:

- Apple Music / iTunes
- Spotify
- YouTube (in Safari/Chrome/Firefox)
- VLC
- IINA
- Swinsian
- And many more!

If it shows up in your macOS Control Center or Lock Screen, it will work with OSX Scrobbler.

## Troubleshooting

### No scrobbles appearing

1. **Check your config** - Ensure `enabled = true` for at least one service
2. **Verify credentials**:
   - Last.fm: Run `osx-scrobbler --auth-lastfm` to re-authenticate
   - ListenBrainz: Verify your token at https://listenbrainz.org/profile/
3. **Check logs**:
   - From terminal: `RUST_LOG=debug osx-scrobbler --console`
   - In background: `tail -f ~/Library/Logs/osx-scrobbler.log`
4. **Track length** - Tracks under 30 seconds are not scrobbled

### Tray icon not appearing

1. **Restart the app** - Quit and relaunch the application
2. **Check permissions** - macOS may require accessibility permissions
3. **Menu bar space** - Ensure your menu bar isn't too crowded (try hiding other icons)

### Text cleanup not working

1. **Check config** - Ensure `cleanup.enabled = true`
2. **Test patterns** - Your regex patterns may have syntax errors (check logs for warnings)
3. **Pattern order** - Patterns are applied in order; make sure they don't conflict

## Configuration Reference

### Main Settings

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `refresh_interval` | integer | `5` | How often (in seconds) to poll for now playing info |
| `scrobble_threshold` | integer | `50` | Percentage of track to play before scrobbling (1-100) |

### Cleanup Settings

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `cleanup.enabled` | boolean | `true` | Enable text cleanup |
| `cleanup.patterns` | array of strings | See config | Regex patterns to remove from track names |

### Last.fm Settings

| Setting | Type | Required | Description |
|---------|------|----------|-------------|
| `lastfm.enabled` | boolean | Yes | Enable Last.fm scrobbling |
| `lastfm.api_key` | string | Yes | Your Last.fm API key |
| `lastfm.api_secret` | string | Yes | Your Last.fm API secret |
| `lastfm.session_key` | string | No* | Session key (obtained via `--auth-lastfm`) |

*Required for scrobbling, but obtained automatically via authentication

### ListenBrainz Settings

| Setting | Type | Required | Description |
|---------|------|----------|-------------|
| `listenbrainz.enabled` | boolean | Yes | Enable this ListenBrainz instance |
| `listenbrainz.name` | string | Yes | Friendly name for this instance |
| `listenbrainz.token` | string | Yes | Your ListenBrainz user token |
| `listenbrainz.api_url` | string | Yes | API URL (usually `https://api.listenbrainz.org`) |

## Development

### Building from Source

```bash
git clone https://github.com/yourusername/osx-scrobbler.git
cd osx-scrobbler
cargo build
```

### Running in Development

```bash
cargo run
```

### Running Tests

```bash
cargo test
```

### Running with Debug Logging

```bash
RUST_LOG=debug cargo run
```

### Linting

```bash
cargo clippy
```

## Credits

Inspired by [rescrobbled](https://github.com/InputUsername/rescrobbled) but built specifically for macOS using the `media_remote` crate.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

```
Copyright 2026 OSX Scrobbler Contributors

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

By contributing to this project, you agree that your contributions will be licensed under the Apache License, Version 2.0.
