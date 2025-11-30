#!/bin/bash
# Universal installer for berri-recall
# Usage: curl -fsSL https://raw.githubusercontent.com/monishobaid/berri-recall/main/install.sh | bash

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   berri-recall Installer v0.1.0       ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}Installing berri-recall...${NC}"
echo ""

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

echo "Detected: $OS $ARCH"

case "$OS" in
  Darwin)
    BINARY_URL="https://github.com/monishobaid/berri-recall/releases/latest/download/berri-recall-macos-arm64.tar.gz"
    if [ "$ARCH" = "arm64" ]; then
      PLATFORM="macOS (Apple Silicon)"
    else
      PLATFORM="macOS (Intel - running ARM binary via Rosetta)"
    fi
    ;;
  Linux)
    BINARY_URL="https://github.com/monishobaid/berri-recall/releases/latest/download/berri-recall-linux-amd64.tar.gz"
    PLATFORM="Linux"
    ;;
  *)
    echo -e "${RED}✗ Unsupported OS: $OS${NC}"
    echo ""
    echo "Supported platforms:"
    echo "  - macOS (Intel & Apple Silicon)"
    echo "  - Linux (x86_64)"
    echo ""
    echo "For Windows, download from: https://github.com/monishobaid/berri-recall/releases"
    exit 1
    ;;
esac

echo "Platform: $PLATFORM"
echo ""

# Check if berri-recall is already installed
if command -v berri-recall &> /dev/null; then
    CURRENT_VERSION=$(berri-recall --version 2>&1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || echo "unknown")
    echo -e "${YELLOW}berri-recall is already installed (version: $CURRENT_VERSION)${NC}"
    read -p "Do you want to reinstall? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Installation cancelled."
        exit 0
    fi
fi

# Create temporary directory
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

echo "Downloading from GitHub..."
if ! curl -L "$BINARY_URL" -o berri-recall.tar.gz; then
    echo -e "${RED}✗ Download failed${NC}"
    echo "URL: $BINARY_URL"
    exit 1
fi

echo "Extracting..."
tar -xzf berri-recall.tar.gz

echo "Installing to /usr/local/bin..."
# Try to install without sudo first (if user has write access)
if mv berri-recall /usr/local/bin/ 2>/dev/null; then
    echo -e "${GREEN}✓ Installed without sudo${NC}"
else
    echo "Need sudo for /usr/local/bin..."
    sudo mv berri-recall /usr/local/bin/
fi

# Make executable
chmod +x /usr/local/bin/berri-recall

echo "Cleaning up..."
cd -
rm -rf "$TEMP_DIR"

echo ""
echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║  ✓ Installation Successful!           ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"
echo ""

# Verify installation
VERSION=$(berri-recall --version 2>&1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || echo "unknown")
echo "Installed version: $VERSION"
echo ""

echo -e "${BLUE}Next steps:${NC}"
echo "  1. Run: ${YELLOW}berri-recall setup${NC}"
echo "     This enables automatic command recording"
echo ""
echo "  2. Restart your shell or run:"
echo "     ${YELLOW}source ~/.zshrc${NC}  (for zsh)"
echo "     ${YELLOW}source ~/.bashrc${NC}  (for bash)"
echo ""
echo "  3. Start using:"
echo "     ${YELLOW}berri-recall recent${NC}     - View recent commands"
echo "     ${YELLOW}berri-recall search npm${NC}  - Search commands"
echo "     ${YELLOW}berri-recall analyze${NC}     - Detect patterns"
echo "     ${YELLOW}berri-recall suggest${NC}     - Get suggestions"
echo ""
echo "For help: ${YELLOW}berri-recall help${NC}"
echo "Docs: https://github.com/monishobaid/berri-recall"
echo ""
echo -e "${GREEN}Done! Your terminal will remember everything now.${NC}"
