#!/bin/bash
set -e

PROFILE="${PROFILE:-release}"
if [ "$PROFILE" = "release" ]; then
    echo "Building release binary..."
    CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/target cargo build --release
    BIN_PATH="/tmp/target/release/oxireader"
else
    echo "Building debug binary..."
    CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/target cargo build
    BIN_PATH="/tmp/target/debug/oxireader"
fi

echo "Creating App Bundle structure..."
mkdir -p "Iridecence PDF Reader.app/Contents/MacOS"
mkdir -p "Iridecence PDF Reader.app/Contents/Resources"

echo "Copying binary ($BIN_PATH)..."
cp "$BIN_PATH" "Iridecence PDF Reader.app/Contents/MacOS/oxireader"

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
    <key>CFBundleDisplayName</key>
    <string>Iridecence PDF Reader</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleShortVersionString</key>
    <string>0.5.0</string>
    <key>CFBundleVersion</key>
    <string>5</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSPrincipalClass</key>
    <string>NSApplication</string>
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeName</key>
            <string>PDF Document</string>
            <key>CFBundleTypeRole</key>
            <string>Viewer</string>
            <key>LSHandlerRank</key>
            <string>Alternate</string>
            <key>LSItemContentTypes</key>
            <array>
                <string>com.adobe.pdf</string>
            </array>
        </dict>
        <dict>
            <key>CFBundleTypeName</key>
            <string>Folder</string>
            <key>CFBundleTypeRole</key>
            <string>Viewer</string>
            <key>LSHandlerRank</key>
            <string>Alternate</string>
            <key>LSItemContentTypes</key>
            <array>
                <string>public.folder</string>
            </array>
        </dict>
    </array>
</dict>
</plist>
PLIST

echo "Done! Run: open 'Iridecence PDF Reader.app'"
