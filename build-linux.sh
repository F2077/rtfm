#!/usr/bin/env bash
# Linux Build Script for RTFM (for WSL/Ubuntu)
# This script builds the rtfm binary for Linux x86_64

set -e

echo "======================================"
echo "RTFM Linux Build Script"
echo "======================================"

# Color definitions
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if we're in WSL
if grep -qEi "(Microsoft|WSL)" /proc/version &> /dev/null ; then
    echo -e "${GREEN}✓ Running in WSL${NC}"
else
    echo -e "${YELLOW}⚠ Not running in WSL, assuming native Linux${NC}"
fi

# Check Rust installation
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}✗ Cargo not found!${NC}"
    echo "Please install Rust from: https://rustup.rs/"
    echo "Run: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo -e "${GREEN}✓ Cargo found: $(cargo --version)${NC}"

# Check required dependencies
echo ""
echo "Checking system dependencies..."

MISSING_DEPS=()

# Check for essential build tools
if ! command -v gcc &> /dev/null; then
    MISSING_DEPS+=("gcc")
fi

if ! command -v pkg-config &> /dev/null; then
    MISSING_DEPS+=("pkg-config")
fi

# Check for OpenSSL development files
if ! pkg-config --exists openssl 2>/dev/null; then
    MISSING_DEPS+=("libssl-dev")
fi

if [ ${#MISSING_DEPS[@]} -ne 0 ]; then
    echo -e "${RED}✗ Missing dependencies: ${MISSING_DEPS[*]}${NC}"
    echo ""
    echo "Please run:"
    echo "  sudo apt update"
    echo "  sudo apt install -y build-essential pkg-config libssl-dev"
    exit 1
fi

echo -e "${GREEN}✓ All dependencies satisfied${NC}"

# Clean previous build (optional)
echo ""
read -p "Clean previous build? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "Cleaning..."
    cargo clean
    echo -e "${GREEN}✓ Cleaned${NC}"
fi

# Build release
echo ""
echo "======================================"
echo "Building release binary..."
echo "======================================"

cargo build --release

if [ $? -eq 0 ]; then
    echo ""
    echo -e "${GREEN}======================================"
    echo "✓ Build successful!"
    echo "======================================${NC}"
    echo ""
    echo "Binary location:"
    echo "  ./target/release/rtfm"
    echo ""
    echo "Binary size:"
    ls -lh ./target/release/rtfm | awk '{print "  " $5}'
    echo ""
    echo "To test the binary:"
    echo "  ./target/release/rtfm --version"
    echo "  ./target/release/rtfm --help"
    echo ""
    echo "To install system-wide:"
    echo "  sudo cp ./target/release/rtfm /usr/local/bin/"
else
    echo -e "${RED}✗ Build failed!${NC}"
    exit 1
fi

# Optional: Strip binary for smaller size
echo ""
read -p "Strip debug symbols to reduce size? (Y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Nn]$ ]]; then
    echo "Stripping debug symbols..."
    strip ./target/release/rtfm
    echo -e "${GREEN}✓ Stripped${NC}"
    echo ""
    echo "New binary size:"
    ls -lh ./target/release/rtfm | awk '{print "  " $5}'
fi

echo ""
echo -e "${GREEN}Done!${NC}"
