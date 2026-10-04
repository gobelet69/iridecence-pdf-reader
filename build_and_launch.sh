#!/bin/bash
set -e

echo "Closing OxiReader..."
killall oxireader || true
killall OxiReader || true
sleep 1

echo "Building..."
./build_app.sh

echo "Launching OxiReader..."
if [ $# -eq 0 ]; then
    open OxiReader.app
else
    open -a "$PWD/OxiReader.app" "$@"
fi
