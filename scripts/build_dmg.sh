#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT_DIR"

VERSION="${1:-2.0.0}"
CODE_SIGN_IDENTITY="${CODE_SIGN_IDENTITY:--}"
UNIVERSAL="${UNIVERSAL:-0}"
APP_NAME="Iridecence PDF Reader.app"
VOL_NAME="Iridescence PDF Reader"

if [ "$UNIVERSAL" = "1" ]; then
    DMG_NAME="Iridescence-PDF-Reader-v${VERSION}-macOS-universal.dmg"
else
    DMG_NAME="Iridescence-PDF-Reader-v${VERSION}-macOS-arm64.dmg"
fi

STAGE_DIR="/tmp/iridecence-dmg-stage"
OUT_DIR="$ROOT_DIR/dist"

echo "=== Packaging ${DMG_NAME} ==="

UNIVERSAL="$UNIVERSAL" PROFILE=release ./build_app.sh

rm -rf "$STAGE_DIR"
mkdir -p "$STAGE_DIR" "$OUT_DIR"

echo "Staging $APP_NAME and /Applications symlink..."
ditto --norsrc --noextattr --noqtn "$APP_NAME" "$STAGE_DIR/$APP_NAME"
xattr -cr "$STAGE_DIR/$APP_NAME"

if [ "$CODE_SIGN_IDENTITY" != "-" ]; then
    echo "Signing with Developer ID: $CODE_SIGN_IDENTITY"
    codesign --force --deep --options runtime --timestamp \
        --sign "$CODE_SIGN_IDENTITY" "$STAGE_DIR/$APP_NAME"
else
    echo "Signing with ad-hoc identity (-)"
    codesign --force --deep --sign - "$STAGE_DIR/$APP_NAME"
fi

codesign --verify --deep --strict --verbose=2 "$STAGE_DIR/$APP_NAME"
ln -s /Applications "$STAGE_DIR/Applications"

echo "Creating DMG at $OUT_DIR/$DMG_NAME..."
rm -f "$OUT_DIR/$DMG_NAME"
hdiutil create \
    -volname "$VOL_NAME" \
    -srcfolder "$STAGE_DIR" \
    -ov \
    -format UDZO \
    "$OUT_DIR/$DMG_NAME"

rm -rf "$STAGE_DIR"

(cd "$OUT_DIR" && shasum -a 256 "$DMG_NAME" > SHA256SUMS.txt && cat SHA256SUMS.txt)
echo "✓ DMG ready at: $OUT_DIR/$DMG_NAME"
