#!/usr/bin/env bash
set -euo pipefail

QA_DIR="${1:-/tmp/iridescence-swift-visual-qa}"
mkdir -p "$QA_DIR"

echo "=== Iridescence Native Swift Visual QA Suite ==="
echo "Output directory: $QA_DIR"

./build_app.sh

pkill -9 -f "Iridecence PDF Reader" || true
pkill -9 -f "Iridescence" || true
rm -rf "$HOME/Library/Saved Application State/com.iridecence.pdfreader.savedState"
defaults write com.iridecence.pdfreader ApplePersistenceIgnoreState YES

echo "Running 22-state Visual QA harness..."
IRIDESCENCE_SWIFT_VISUAL_QA=1 \
IRIDESCENCE_SWIFT_QA_DIR="$QA_DIR" \
IRIDESCENCE_QA_EXIT=1 \
"Iridecence PDF Reader.app/Contents/MacOS/Iridescence"

COUNT=$(ls -1 "$QA_DIR"/*.png 2>/dev/null | wc -l | tr -d ' ')
echo "Captured $COUNT screenshots in $QA_DIR:"
ls -lh "$QA_DIR"/*.png
echo "=== Visual QA Complete ==="
