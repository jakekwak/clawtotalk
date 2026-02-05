#!/bin/bash

# Universal Binary Builder for macOS
# Builds both Intel and Apple Silicon binaries and combines them

set -e

# Configuration
EXECUTABLE_NAME="dioxus-voice-assistant"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Building Universal Binary for macOS${NC}"

# Check if both targets are installed
echo -e "${YELLOW}Checking Rust targets...${NC}"

if ! rustup target list | grep -q "x86_64-apple-darwin (installed)"; then
    echo -e "${YELLOW}Installing x86_64-apple-darwin target...${NC}"
    rustup target add x86_64-apple-darwin
fi

if ! rustup target list | grep -q "aarch64-apple-darwin (installed)"; then
    echo -e "${YELLOW}Installing aarch64-apple-darwin target...${NC}"
    rustup target add aarch64-apple-darwin
fi

# Build for Intel (x86_64)
echo -e "${YELLOW}Building for Intel (x86_64)...${NC}"
cargo build --release --target x86_64-apple-darwin

if [ ! -f "target/x86_64-apple-darwin/release/$EXECUTABLE_NAME" ]; then
    echo -e "${RED}Intel build failed${NC}"
    exit 1
fi

# Build for Apple Silicon (ARM64)
echo -e "${YELLOW}Building for Apple Silicon (ARM64)...${NC}"
cargo build --release --target aarch64-apple-darwin

if [ ! -f "target/aarch64-apple-darwin/release/$EXECUTABLE_NAME" ]; then
    echo -e "${RED}Apple Silicon build failed${NC}"
    exit 1
fi

# Create universal binary
echo -e "${YELLOW}Creating universal binary...${NC}"
mkdir -p target/universal/release

lipo -create \
    "target/x86_64-apple-darwin/release/$EXECUTABLE_NAME" \
    "target/aarch64-apple-darwin/release/$EXECUTABLE_NAME" \
    -output "target/universal/release/$EXECUTABLE_NAME"

# Verify universal binary
echo -e "${YELLOW}Verifying universal binary...${NC}"
lipo -info "target/universal/release/$EXECUTABLE_NAME"

# Get file sizes
INTEL_SIZE=$(du -h "target/x86_64-apple-darwin/release/$EXECUTABLE_NAME" | cut -f1)
ARM_SIZE=$(du -h "target/aarch64-apple-darwin/release/$EXECUTABLE_NAME" | cut -f1)
UNIVERSAL_SIZE=$(du -h "target/universal/release/$EXECUTABLE_NAME" | cut -f1)

# Print summary
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}Universal Binary Build Summary${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "Intel (x86_64): ${GREEN}$INTEL_SIZE${NC}"
echo -e "Apple Silicon (ARM64): ${GREEN}$ARM_SIZE${NC}"
echo -e "Universal Binary: ${GREEN}$UNIVERSAL_SIZE${NC}"
echo -e "Location: ${GREEN}target/universal/release/$EXECUTABLE_NAME${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

echo -e "${GREEN}Universal binary created successfully!${NC}"
echo -e ""
echo -e "${GREEN}To create an app bundle with the universal binary:${NC}"
echo -e "  1. Copy the universal binary to target/release/"
echo -e "     cp target/universal/release/$EXECUTABLE_NAME target/release/"
echo -e "  2. Run the app bundle builder"
echo -e "     ./macos/build_app_bundle.sh"
