#!/bin/bash
# Build OSX Scrobbler as a proper macOS .app bundle

set -e

echo "Building OSX Scrobbler.app..."
echo ""

# Build release binary
cargo build --release

APP_NAME="OSX Scrobbler"
APP_DIR="target/release/$APP_NAME.app"
CONTENTS_DIR="$APP_DIR/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
RESOURCES_DIR="$CONTENTS_DIR/Resources"

# Clean old app bundle
rm -rf "$APP_DIR"

# Create app bundle structure
mkdir -p "$MACOS_DIR"
mkdir -p "$RESOURCES_DIR"

# Copy binary
cp target/release/osx-scrobbler "$MACOS_DIR/osx-scrobbler"
chmod +x "$MACOS_DIR/osx-scrobbler"

# Copy Info.plist
cp Info.plist "$CONTENTS_DIR/Info.plist"

# Update version in Info.plist from Cargo.toml
VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
# Replace both CFBundleVersion and CFBundleShortVersionString
plutil -replace CFBundleVersion -string "$VERSION" "$CONTENTS_DIR/Info.plist" 2>/dev/null || \
    sed -i '' "s|<key>CFBundleVersion</key>[[:space:]]*<string>[^<]*</string>|<key>CFBundleVersion</key><string>$VERSION</string>|" "$CONTENTS_DIR/Info.plist"
plutil -replace CFBundleShortVersionString -string "$VERSION" "$CONTENTS_DIR/Info.plist" 2>/dev/null || \
    sed -i '' "s|<key>CFBundleShortVersionString</key>[[:space:]]*<string>[^<]*</string>|<key>CFBundleShortVersionString</key><string>$VERSION</string>|" "$CONTENTS_DIR/Info.plist"

echo "✓ Created app bundle: $APP_DIR"
echo ""
echo "To install:"
echo "  cp -r \"$APP_DIR\" /Applications/"
echo ""
echo "To run:"
echo "  open \"$APP_DIR\""
echo ""
echo "The app will:"
echo "  - Run in the background (no dock icon)"
echo "  - Show only a menu bar icon"
echo "  - Log to ~/Library/Logs/osx-scrobbler.log"
echo ""
