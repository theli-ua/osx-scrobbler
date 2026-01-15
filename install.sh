#!/bin/bash
# OSX Scrobbler installation script for launch agent

set -e

PLIST_NAME="com.osx-scrobbler.plist"
LAUNCH_AGENTS_DIR="$HOME/Library/LaunchAgents"
PLIST_PATH="$LAUNCH_AGENTS_DIR/$PLIST_NAME"
LOG_PATH="$HOME/Library/Logs/osx-scrobbler.log"

# Find the binary
if command -v osx-scrobbler &> /dev/null; then
    BINARY_PATH=$(command -v osx-scrobbler)
else
    echo "Error: osx-scrobbler binary not found in PATH"
    echo "Please install it first with: cargo install osx-scrobbler"
    exit 1
fi

echo "OSX Scrobbler Installation"
echo "=========================="
echo ""
echo "Binary: $BINARY_PATH"
echo "Plist:  $PLIST_PATH"
echo "Logs:   $LOG_PATH"
echo ""

# Create LaunchAgents directory if it doesn't exist
mkdir -p "$LAUNCH_AGENTS_DIR"

# Create the plist file with proper paths
cat > "$PLIST_PATH" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.osx-scrobbler</string>

    <key>ProgramArguments</key>
    <array>
        <string>$BINARY_PATH</string>
    </array>

    <key>RunAtLoad</key>
    <true/>

    <key>KeepAlive</key>
    <true/>

    <key>ProcessType</key>
    <string>Interactive</string>

    <key>StandardOutPath</key>
    <string>$LOG_PATH</string>

    <key>StandardErrorPath</key>
    <string>$LOG_PATH</string>
</dict>
</plist>
EOF

# Unload if already loaded
if launchctl list | grep -q com.osx-scrobbler; then
    echo "Unloading existing service..."
    launchctl unload "$PLIST_PATH" 2>/dev/null || true
fi

# Load the launch agent
echo "Loading launch agent..."
launchctl load "$PLIST_PATH"

echo ""
echo "✓ Installation complete!"
echo ""
echo "OSX Scrobbler will now:"
echo "  - Start automatically when you log in"
echo "  - Run in the background without a terminal window"
echo "  - Show a menu bar icon"
echo "  - Log to: $LOG_PATH"
echo ""
echo "To view logs:"
echo "  tail -f $LOG_PATH"
echo ""
echo "To uninstall:"
echo "  ./uninstall.sh"
echo ""
