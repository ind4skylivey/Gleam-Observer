#!/bin/bash
# GleamObserver Uninstallation Script

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

# Config
INSTALL_DIR="${HOME}/.local/bin"
DESKTOP_DIR="${HOME}/.local/share/applications"
BIN_NAME="gleam"

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  GleamObserver Uninstallation Script  ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo ""

# Remove binary
if [ -f "${INSTALL_DIR}/${BIN_NAME}" ]; then
    echo -e "${BLUE}➜${NC} Removing binary..."
    rm -f "${INSTALL_DIR}/${BIN_NAME}"
else
    echo -e "${RED}✗${NC} Binary not found at ${INSTALL_DIR}/${BIN_NAME}"
fi

# Remove desktop entry
if [ -f "${DESKTOP_DIR}/gleam-observer.desktop" ]; then
    echo -e "${BLUE}➜${NC} Removing desktop entry..."
    rm -f "${DESKTOP_DIR}/gleam-observer.desktop"
else
    echo -e "${RED}✗${NC} Desktop entry not found"
fi

# Success
echo ""
echo -e "${GREEN}✓ GleamObserver has been uninstalled${NC}"
echo ""
