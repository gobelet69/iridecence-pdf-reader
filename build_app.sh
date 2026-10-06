#!/bin/bash
set -e

echo "Building release binary..."
CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/target cargo build

echo "Creating App Bundle structure..."
mkdir -p "Iridecence PDF Reader.app/Contents/MacOS"
mkdir -p "Iridecence PDF Reader.app/Contents/Resources"

echo "Copying binary..."
cp /tmp/target/debug/oxireader "Iridecence PDF Reader.app/Contents/MacOS/"

echo "Copying icon..."
cp AppIcon.icns "Iridecence PDF Reader.app/Contents/Resources/"

echo "Creating Info.plist..."
cat > "Iridecence PDF Reader.app/Contents/Info.plist" << 'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>oxireader</string>
    <key>CFBundleIdentifier</key>
    <string>com.iridecence.pdfreader</string>
    <key>CFBundleName</key>
    <string>Iridecence PDF Reader</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleVersion</key>
    <string>1.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.15</string>
</dict>
</plist>
PLIST

echo "Done! Run: open 'Iridecence PDF Reader.app'"
