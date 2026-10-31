#!/usr/bin/env bash
set -euo pipefail

PROFILE="${PROFILE:-release}"
BUILD_DIR="/tmp/xcode-build"

if [ "$PROFILE" = "release" ]; then
    echo "Building native Swift Release binary via xcodebuild..."
    xcodebuild -project Iridescence.xcodeproj \
        -target Iridescence \
        -configuration Release \
        ARCHS=arm64 \
        ONLY_ACTIVE_ARCH=YES \
        SYMROOT="$BUILD_DIR" \
        build -quiet
    BIN_PATH="$BUILD_DIR/Release/Iridescence"
else
    echo "Building native Swift Debug binary via xcodebuild..."
    xcodebuild -project Iridescence.xcodeproj \
        -target Iridescence \
        -configuration Debug \
        ARCHS=arm64 \
        ONLY_ACTIVE_ARCH=YES \
        SYMROOT="$BUILD_DIR" \
        build -quiet
    BIN_PATH="$BUILD_DIR/Debug/Iridescence"
fi

APP_BUNDLE="Iridecence PDF Reader.app"

echo "Creating App Bundle structure ($APP_BUNDLE)..."
rm -rf "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources"

echo "Copying native Swift binary ($BIN_PATH)..."
cp "$BIN_PATH" "$APP_BUNDLE/Contents/MacOS/Iridescence"
chmod +x "$APP_BUNDLE/Contents/MacOS/Iridescence"

if [ -f AppIcon.icns ]; then
    echo "Copying app icon..."
    cp AppIcon.icns "$APP_BUNDLE/Contents/Resources/"
fi

echo "Creating Info.plist..."
cat > "$APP_BUNDLE/Contents/Info.plist" << 'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>Iridescence</string>
    <key>CFBundleIdentifier</key>
    <string>com.iridecence.pdfreader</string>
    <key>CFBundleName</key>
    <string>Iridescence PDF Reader</string>
    <key>CFBundleDisplayName</key>
    <string>Iridescence PDF Reader</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleShortVersionString</key>
    <string>2.0.0</string>
    <key>CFBundleVersion</key>
    <string>200</string>
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

echo "Ad-hoc signing $APP_BUNDLE..."
codesign --force --deep --sign - "$APP_BUNDLE" >/dev/null 2>&1 || true

echo "Done! Built 100% Native Swift '$APP_BUNDLE' (v2.0.0)."
