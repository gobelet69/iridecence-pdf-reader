#!/bin/bash
set -e

PASS=${1:-pass-01}
QA_DIR="/tmp/iridecence-visual-qa/$PASS"
mkdir -p "$QA_DIR"

echo "=== Visual QA Runner: $PASS ==="
echo "Target directory: $QA_DIR"

# 1. Build app
echo "Building debug binary..."
CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/target cargo build
mkdir -p "Iridecence PDF Reader.app/Contents/MacOS"
mkdir -p "Iridecence PDF Reader.app/Contents/Resources"
cp /tmp/target/debug/oxireader "Iridecence PDF Reader.app/Contents/MacOS/"
if [ -f AppIcon.icns ]; then
    cp AppIcon.icns "Iridecence PDF Reader.app/Contents/Resources/"
fi

# 2. Kill existing running app instances to avoid conflicts
pkill -9 -f "Iridecence PDF Reader" || true
pkill -9 -f "oxireader" || true
rm -rf "$HOME/Library/Saved Application State/com.iridecence.pdfreader.savedState"
defaults write com.iridecence.pdfreader ApplePersistenceIgnoreState YES
sleep 1

SAFE_PASS=$(echo "$PASS" | tr '/' '_')
LOG_FILE="/tmp/visual_qa_${SAFE_PASS}.log"

# 3. Launch with QA environment variables
echo "Launching app with IRIDESCENCE_VISUAL_QA=1..."
IRIDESCENCE_VISUAL_QA=1 IRIDESCENCE_VISUAL_QA_PASS="$PASS" "Iridecence PDF Reader.app/Contents/MacOS/oxireader" > "$LOG_FILE" 2>&1 &
APP_PID=$!

echo "App launched with PID $APP_PID, waiting for QA sequence..."

# 4. Wait for QA sequence to complete (or timeout after 45 seconds)
TIMEOUT=45
ELAPSED=0
DONE=0

while [ $ELAPSED -lt $TIMEOUT ]; do
    if grep -q "\[VISUAL_QA_DONE\]" "$LOG_FILE" 2>/dev/null; then
        echo "Visual QA sequence completed successfully!"
        DONE=1
        break
    fi
    sleep 1
    ELAPSED=$((ELAPSED + 1))
done

if [ $DONE -eq 0 ]; then
    echo "Warning: QA sequence did not report completion within ${TIMEOUT}s. Inspecting log:"
    cat "$LOG_FILE" | tail -n 20
fi

# 5. Report generated screenshots
COUNT=$(ls -1 "$QA_DIR"/*.png 2>/dev/null | wc -l | tr -d ' ')
echo "Captured $COUNT / 19 screenshots in $QA_DIR:"
ls -lh "$QA_DIR"/*.png 2>/dev/null || true

# Terminate test process
kill $APP_PID 2>/dev/null || true
echo "=== End of Visual QA Run for $PASS ==="
