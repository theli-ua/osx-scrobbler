#!/bin/bash
# OSX Scrobbler uninstall script

set -e

PLIST_NAME="com.osx-scrobbler.plist"
LAUNCH_AGENTS_DIR="$HOME/Library/LaunchAgents"
PLIST_PATH="$LAUNCH_AGENTS_DIR/$PLIST_NAME"

echo "OSX Scrobbler Uninstall"
echo "======================="
echo ""

if [ ! -f "$PLIST_PATH" ]; then
    echo "Launch agent not found at: $PLIST_PATH"
    echo "Nothing to uninstall."
    exit 0
fi

# Unload the launch agent
if launchctl list | grep -q com.osx-scrobbler; then
    echo "Unloading launch agent..."
    launchctl unload "$PLIST_PATH"
else
    echo "Launch agent not currently loaded."
fi

# Remove the plist file
echo "Removing plist file..."
rm "$PLIST_PATH"

echo ""
echo "✓ Uninstall complete!"
echo ""
echo "OSX Scrobbler has been removed from launch agents."
echo "The binary and configuration files remain intact."
echo ""
echo "To completely remove OSX Scrobbler:"
echo "  - Binary: cargo uninstall osx-scrobbler"
echo "  - Config: rm -rf ~/Library/Application\\ Support/osx_scrobbler.conf"
echo "  - Logs:   rm -rf ~/Library/Logs/osx-scrobbler.log"
echo ""
