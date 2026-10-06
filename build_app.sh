#!/bin/bash
set -e

echo "Building release binary..."
CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/target cargo build

echo "Creating App Bundle structure..."
mkdir -p OxiReader.app/Contents/MacOS
mkdir -p OxiReader.app/Contents/Resources

echo "Copying binary..."
cp /tmp/target/debug/oxireader OxiReader.app/Contents/MacOS/

echo "Creating Info.plist..."
cat > OxiReader.app/Contents/Info.plist << 'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>oxireader</string>
    <key>CFBundleIdentifier</key>
    <string>com.theodeville.oxireader</string>
    <key>CFBundleName</key>
    <string>OxiReader</string>
    <key>CFBundleVersion</key>
    <string>1.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.15</string>
</dict>
</plist>
PLIST

echo "Done! Run: open OxiReader.app"
