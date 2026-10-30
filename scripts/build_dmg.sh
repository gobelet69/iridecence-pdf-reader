#!/bin/bash
set -euo pipefail

VERSION="${1:-0.5.1}"
CODE_SIGN_IDENTITY="${CODE_SIGN_IDENTITY:--}"
APP_NAME="Iridecence PDF Reader.app"
VOL_NAME="Iridecence PDF Reader"
DMG_NAME="Iridecence-PDF-Reader-v${VERSION}-macOS-arm64.dmg"
STAGE_DIR="/tmp/iridecence-dmg-stage"
OUT_DIR="/tmp/iridecence-release"

if [ ! -d "$APP_NAME" ]; then
    ./build_app.sh
fi

rm -rf "$STAGE_DIR" "$OUT_DIR"
mkdir -p "$STAGE_DIR" "$OUT_DIR"

echo "Staging $APP_NAME and /Applications symlink..."
ditto --norsrc --noextattr --noqtn "$APP_NAME" "$STAGE_DIR/$APP_NAME"
xattr -cr "$STAGE_DIR/$APP_NAME"
codesign --force --deep --options runtime --timestamp=none \
    --sign "$CODE_SIGN_IDENTITY" "$STAGE_DIR/$APP_NAME"
codesign --verify --deep --strict --verbose=2 "$STAGE_DIR/$APP_NAME"
ln -s /Applications "$STAGE_DIR/Applications"

echo "Creating $OUT_DIR/$DMG_NAME..."
hdiutil create \
    -volname "$VOL_NAME" \
    -srcfolder "$STAGE_DIR" \
    -ov \
    -format UDZO \
    "$OUT_DIR/$DMG_NAME"

(cd "$OUT_DIR" && shasum -a 256 "$DMG_NAME" > SHA256SUMS.txt && cat SHA256SUMS.txt)
echo "DMG ready at: $OUT_DIR/$DMG_NAME"
