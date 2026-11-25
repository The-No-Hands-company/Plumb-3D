#!/usr/bin/env bash
set -euo pipefail

VERSION="1.0.0"
APP_NAME="plumb3d"
DIST_DIR="dist"
ARCH="$(uname -m)"
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
PACKAGE_NAME="${APP_NAME}-v${VERSION}-${OS}-${ARCH}"

echo "=========================================================="
echo " Packaging Plumb 3D Suite v${VERSION} for ${OS}-${ARCH}   "
echo "=========================================================="

cargo build --release --workspace

CARGO_TARGET=$(cargo metadata --format-version 1 --no-deps | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')

rm -rf "${DIST_DIR}"
mkdir -p "${DIST_DIR}/${PACKAGE_NAME}"

cp "${CARGO_TARGET}/release/app" "${DIST_DIR}/${PACKAGE_NAME}/${APP_NAME}"
cp README.md "${DIST_DIR}/${PACKAGE_NAME}/"
cp -r docs "${DIST_DIR}/${PACKAGE_NAME}/"

cd "${DIST_DIR}"
tar -czf "${PACKAGE_NAME}.tar.gz" "${PACKAGE_NAME}"
sha256sum "${PACKAGE_NAME}.tar.gz" > "${PACKAGE_NAME}.tar.gz.sha256"

echo ""
echo "Release package created:"
echo "  Archive:  ${DIST_DIR}/${PACKAGE_NAME}.tar.gz"
echo "  Checksum: $(cat "${PACKAGE_NAME}.tar.gz.sha256")"
