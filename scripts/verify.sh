#!/usr/bin/env bash
set -euo pipefail

echo "========================================================"
echo "  Iridescence PDF Reader — Native Swift Verification    "
echo "========================================================"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${ROOT_DIR}"

echo "Step 1: Verifying zero Rust dependencies remain..."
if [ -f "Cargo.toml" ] || [ -d "src" ]; then
    echo "ERROR: Legacy Rust files (Cargo.toml or src/) still exist!"
    exit 1
fi
echo "  -> Confirmed 100% pure Swift repository."

echo "Step 2: Running Swift Package Manager build check..."
swift build --scratch-path /tmp/swift-build

echo "Step 3: Running Automated Unit & Integration Tests (XCTest)..."
swift test --scratch-path /tmp/swift-build

echo "Step 4: Building macOS Application Bundle via xcodebuild..."
./build_app.sh

echo "========================================================"
echo "  All verification gates passed successfully!           "
echo "========================================================"
