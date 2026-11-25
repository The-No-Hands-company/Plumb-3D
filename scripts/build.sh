#!/usr/bin/env bash
set -euo pipefail

echo "========================================="
echo " Building Plumb 3D Suite (Release Mode)  "
echo "========================================="

cargo build --release --workspace

TARGET_DIR=$(cargo metadata --format-version 1 --no-deps | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')
echo ""
echo "✅ Build completed successfully."
echo "Binary available at: ${TARGET_DIR}/release/app"
