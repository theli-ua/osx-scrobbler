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
- 🚀 **Launch at Login** - Optionally start when you log in
- 📊 **Menu Bar Integration** - Lightweight menu bar icon showing current track
- ⚡ **Efficient** - Low resource usage, runs silently in background

## Installation

### Prerequisites

- macOS 10.15 or later
- Rust toolchain (install from [rustup.rs](https://rustup.rs))

### Install from crates.io

```bash
cargo install osx-scrobbler
```

The binary will be installed to `~/.cargo/bin/osx-scrobbler` (ensure `~/.cargo/bin` is in your PATH).

### Set Up as Background Service (Recommended)

To run OSX Scrobbler as a proper macOS background service without any terminal windows:

```bash
# Clone the repo to get the install script
git clone https://github.com/yourusername/osx-scrobbler.git
cd osx-scrobbler

# Run the install script
./install.sh
```

This will:
- Create a macOS Launch Agent
- Start the app automatically at login
- Run in the background without opening Terminal
- Log to `~/Library/Logs/osx-scrobbler.log`

To uninstall the launch agent:
```bash
./uninstall.sh
```

### Running Manually (Alternative)

You can also run directly from the terminal:

```bash
osx-scrobbler
```

**Note:** When launched via Spotlight or by double-clicking the binary, macOS will open a Terminal window. To avoid this, use the launch agent installation method above.

### Building from Source (Alternative)

```bash
git clone https://github.com/yourusername/osx-scrobbler.git
cd osx-scrobbler
cargo build --release
```

The binary will be available at `target/release/osx-scrobbler`.

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

# Launch automatically when you log in
launch_at_login = false
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
- **Launch at Login** - Toggle auto-start
- **Quit** - Exit the application

### Command Line Options

```bash
# Show help
osx-scrobbler --help

# Show version
osx-scrobbler --version

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

1. **Check if running** - `launchctl list | grep osx-scrobbler` (if using launch agent)
2. **Restart the service**:
   ```bash
   launchctl unload ~/Library/LaunchAgents/com.osx-scrobbler.plist
   launchctl load ~/Library/LaunchAgents/com.osx-scrobbler.plist
   ```
3. **Check permissions** - macOS may require accessibility permissions
4. **Menu bar space** - Ensure your menu bar isn't too crowded (try hiding other icons)

### Terminal window opens when launched

This is expected behavior when launching a command-line binary from Spotlight/Finder. To avoid this:

1. **Use the launch agent** (recommended):
   ```bash
   ./install.sh
   ```

2. **Or create an alias/script** - Create a wrapper script that backgrounds the process

### Text cleanup not working

1. **Check config** - Ensure `cleanup.enabled = true`
2. **Test patterns** - Your regex patterns may have syntax errors (check logs for warnings)
3. **Pattern order** - Patterns are applied in order; make sure they don't conflict

### Launch at login not working

1. **Toggle the setting** - Turn it off and on again from the menu
2. **Check System Settings** - Go to System Settings → General → Login Items
3. **Permissions** - macOS may require permission to add login items

## Configuration Reference

### Main Settings

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `refresh_interval` | integer | `5` | How often (in seconds) to poll for now playing info |
| `scrobble_threshold` | integer | `50` | Percentage of track to play before scrobbling (1-100) |
| `launch_at_login` | boolean | `false` | Start app automatically when you log in |

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
