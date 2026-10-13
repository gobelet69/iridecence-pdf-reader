#!/usr/bin/env bash
set -euo pipefail

echo "========================================================"
echo "  Iridescence PDF Reader — Autonomous Verification Suite"
echo "========================================================"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
cd "${ROOT_DIR}"

echo "Step 1: Running cargo check..."
cargo check

echo "Step 2: Running cargo test (Unit & Regression Tests)..."
cargo test

echo "Step 3: Building macOS application bundle..."
./build_app.sh

echo "========================================================"
echo "  All verification gates passed successfully!           "
echo "========================================================"
