#!/bin/bash

set -e

APP_NAME="OxiReader"
APP_BUNDLE="$APP_NAME.app"
BIN_DIR="$APP_BUNDLE/Contents/MacOS"
RES_DIR="$APP_BUNDLE/Contents/Resources"

echo "Building release binary..."
source ~/.cargo/env 2>/dev/null || export PATH="$HOME/.cargo/bin:$PATH"; cargo build

echo "Creating bundle structure..."
mkdir -p "$BIN_DIR"
mkdir -p "$RES_DIR"

echo "Copying binary..."
cp target/debug/oxireader "$BIN_DIR/"

echo "Creating Info.plist..."
cat > "$APP_BUNDLE/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>oxireader</string>
    <key>CFBundleIdentifier</key>
    <string>com.example.oxireader2</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>OxiReader</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>CFBundleSupportedPlatforms</key>
    <array>
        <string>MacOSX</string>
    </array>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

echo "Done! The application is available at $APP_BUNDLE"
