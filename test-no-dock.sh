#!/bin/bash
# Test script to verify no dock icon appears

echo "Testing OSX Scrobbler (no dock icon)..."
echo ""

# Kill any existing instances
pkill -f osx-scrobbler 2>/dev/null || true

# Unload launch agent if active
launchctl unload ~/Library/LaunchAgents/com.osx-scrobbler.plist 2>/dev/null || true

sleep 1

echo "Starting OSX Scrobbler..."
echo "Check your dock - there should be NO icon for OSX Scrobbler"
echo "You should only see the musical note in the menu bar (top right)"
echo ""
echo "Press Ctrl+C to stop"
echo ""

# Run in background without terminal
nohup osx-scrobbler > /tmp/osx-scrobbler-test.log 2>&1 &

echo "Running... (PID: $!)"
echo "Logs: tail -f /tmp/osx-scrobbler-test.log"

# Wait a bit for app to start
sleep 2

# Check if it's running
if pgrep -f osx-scrobbler > /dev/null; then
    echo "✓ App is running"
    echo "✓ Check your menu bar for the musical note icon"
    echo "✓ Verify NO dock icon appears"
else
    echo "✗ App failed to start"
    echo "Check logs: cat /tmp/osx-scrobbler-test.log"
fi
