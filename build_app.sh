#!/bin/bash
set -euo pipefail

PROFILE="${PROFILE:-release}"
APP_BUNDLE="Iridecence PDF Reader.app"
UNIVERSAL="${UNIVERSAL:-0}"
VERSION=$(grep -m1 '^version' Cargo.toml | cut -d '"' -f 2)

echo "=== Building Iridescence PDF Reader v${VERSION} (${PROFILE}) ==="

if [ "$UNIVERSAL" = "1" ]; then
    echo "Building Universal 2 binary (aarch64 + x86_64)..."
    CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/target cargo build --release --target aarch64-apple-darwin
    CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/target cargo build --release --target x86_64-apple-darwin
elif [ "$PROFILE" = "release" ]; then
    echo "Building release binary..."
    CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/target cargo build --release
else
    echo "Building debug binary..."
    CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/target cargo build
fi

echo "Creating App Bundle structure..."
rm -rf "$APP_BUNDLE"
mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources"

if [ "$UNIVERSAL" = "1" ]; then
    lipo -create \
        /tmp/target/aarch64-apple-darwin/release/oxireader \
        /tmp/target/x86_64-apple-darwin/release/oxireader \
        -output "$APP_BUNDLE/Contents/MacOS/oxireader"
    echo "Universal 2 binary assembled:"
    file "$APP_BUNDLE/Contents/MacOS/oxireader"
elif [ "$PROFILE" = "release" ]; then
    cp "/tmp/target/release/oxireader" "$APP_BUNDLE/Contents/MacOS/oxireader"
else
    cp "/tmp/target/debug/oxireader" "$APP_BUNDLE/Contents/MacOS/oxireader"
fi

chmod +x "$APP_BUNDLE/Contents/MacOS/oxireader"

echo "Copying icon..."
cp AppIcon.icns "$APP_BUNDLE/Contents/Resources/"

echo "Creating Info.plist for v${VERSION}..."
cat > "$APP_BUNDLE/Contents/Info.plist" << PLIST
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
    <string>${VERSION}</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>LSMinimumSystemVersion</key>
    <string>14.0</string>
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

echo "Stripping extended attributes & ad-hoc signing..."
xattr -cr "$APP_BUNDLE"
codesign --force --deep --sign - "$APP_BUNDLE"
codesign --verify --deep --strict --verbose=1 "$APP_BUNDLE"

echo "✓ Done! Application bundle verified at: $APP_BUNDLE"
