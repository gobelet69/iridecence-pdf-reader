#!/usr/bin/env bash
set -euo pipefail

QA_DIR="${1:-/tmp/iridescence-swift-visual-qa}"
mkdir -p "$QA_DIR"
# Separate bundle identity and index: never clear or replace the running app's state.
QA_BUNDLE="$(mktemp -d /tmp/iridescence-reader-qa.XXXXXX)/Reader QA.app"
APP_BUNDLE="$QA_BUNDLE" ./build_app.sh
/usr/libexec/PlistBuddy -c 'Set :CFBundleIdentifier org.111iridescence.pdfreader.visualqa' "$QA_BUNDLE/Contents/Info.plist"
codesign --force --deep --sign - "$QA_BUNDLE"
IRIDESCENCE_SWIFT_VISUAL_QA=1 \
IRIDESCENCE_SWIFT_QA_DIR="$QA_DIR" \
IRIDESCENCE_QA_CACHE_DIR="$(dirname "$QA_BUNDLE")/cache" \
IRIDESCENCE_QA_EXIT=1 \
"$QA_BUNDLE/Contents/MacOS/Iridescence"
echo "Visual QA captures: $QA_DIR"
