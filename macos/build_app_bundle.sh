#!/bin/bash

# macOS App Bundle Builder for Dioxus Voice Assistant
# This script creates a proper macOS application bundle

set -e

# Configuration
APP_NAME="DioxusVoiceAssistant"
BUNDLE_ID="com.squidcode.dioxus-voice-assistant"
VERSION="0.1.0"
EXECUTABLE_NAME="dioxus-voice-assistant"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Building macOS App Bundle for ${APP_NAME}${NC}"

# Detect architecture
ARCH=$(uname -m)
if [ "$ARCH" = "arm64" ]; then
    echo -e "${GREEN}Detected Apple Silicon (ARM64)${NC}"
    TARGET="aarch64-apple-darwin"
elif [ "$ARCH" = "x86_64" ]; then
    echo -e "${GREEN}Detected Intel (x86_64)${NC}"
    TARGET="x86_64-apple-darwin"
else
    echo -e "${RED}Unknown architecture: $ARCH${NC}"
    exit 1
fi

# Build the project
echo -e "${YELLOW}Building release binary for $TARGET...${NC}"
cargo build --release --target $TARGET

# Check if build succeeded
if [ ! -f "target/$TARGET/release/$EXECUTABLE_NAME" ]; then
    echo -e "${RED}Build failed: executable not found${NC}"
    exit 1
fi

# Create app bundle structure
echo -e "${YELLOW}Creating app bundle structure...${NC}"
APP_BUNDLE="${APP_NAME}.app"
rm -rf "$APP_BUNDLE"

mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources"
mkdir -p "$APP_BUNDLE/Contents/Frameworks"

# Copy executable
echo -e "${YELLOW}Copying executable...${NC}"
cp "target/$TARGET/release/$EXECUTABLE_NAME" "$APP_BUNDLE/Contents/MacOS/"
chmod +x "$APP_BUNDLE/Contents/MacOS/$EXECUTABLE_NAME"

# Copy Info.plist
echo -e "${YELLOW}Copying Info.plist...${NC}"
if [ -f "macos/Info.plist" ]; then
    cp "macos/Info.plist" "$APP_BUNDLE/Contents/"
else
    echo -e "${RED}Warning: Info.plist not found${NC}"
fi

# Copy icon if available
if [ -f "macos/AppIcon.icns" ]; then
    echo -e "${YELLOW}Copying app icon...${NC}"
    cp "macos/AppIcon.icns" "$APP_BUNDLE/Contents/Resources/"
else
    echo -e "${YELLOW}Warning: AppIcon.icns not found, skipping icon${NC}"
fi

# Create PkgInfo file
echo -e "${YELLOW}Creating PkgInfo...${NC}"
echo -n "APPL????" > "$APP_BUNDLE/Contents/PkgInfo"

echo -e "${GREEN}App bundle created successfully: $APP_BUNDLE${NC}"

# Optional: Code signing
if [ -n "$CODESIGN_IDENTITY" ]; then
    echo -e "${YELLOW}Code signing with identity: $CODESIGN_IDENTITY${NC}"
    
    if [ -f "macos/entitlements.plist" ]; then
        codesign --force --deep --sign "$CODESIGN_IDENTITY" \
            --entitlements "macos/entitlements.plist" \
            --options runtime \
            "$APP_BUNDLE"
    else
        codesign --force --deep --sign "$CODESIGN_IDENTITY" \
            --options runtime \
            "$APP_BUNDLE"
    fi
    
    # Verify signature
    echo -e "${YELLOW}Verifying code signature...${NC}"
    codesign --verify --verbose "$APP_BUNDLE"
    
    echo -e "${GREEN}Code signing completed${NC}"
else
    echo -e "${YELLOW}Skipping code signing (set CODESIGN_IDENTITY to enable)${NC}"
fi

# Optional: Create DMG
if [ "$CREATE_DMG" = "1" ]; then
    echo -e "${YELLOW}Creating DMG installer...${NC}"
    
    DMG_NAME="${APP_NAME}-${VERSION}-${ARCH}.dmg"
    DMG_TEMP="dmg_temp"
    
    rm -rf "$DMG_TEMP"
    mkdir -p "$DMG_TEMP"
    
    cp -R "$APP_BUNDLE" "$DMG_TEMP/"
    ln -s /Applications "$DMG_TEMP/Applications"
    
    hdiutil create -volname "$APP_NAME" \
        -srcfolder "$DMG_TEMP" \
        -ov -format UDZO \
        "$DMG_NAME"
    
    rm -rf "$DMG_TEMP"
    
    echo -e "${GREEN}DMG created: $DMG_NAME${NC}"
fi

# Print summary
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}Build Summary${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "App Bundle: ${GREEN}$APP_BUNDLE${NC}"
echo -e "Architecture: ${GREEN}$ARCH${NC}"
echo -e "Target: ${GREEN}$TARGET${NC}"
echo -e "Version: ${GREEN}$VERSION${NC}"

# Check bundle size
BUNDLE_SIZE=$(du -sh "$APP_BUNDLE" | cut -f1)
echo -e "Bundle Size: ${GREEN}$BUNDLE_SIZE${NC}"

echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}To run the app:${NC}"
echo -e "  open $APP_BUNDLE"
echo -e ""
echo -e "${GREEN}To create a DMG:${NC}"
echo -e "  CREATE_DMG=1 ./macos/build_app_bundle.sh"
echo -e ""
echo -e "${GREEN}To sign the app:${NC}"
echo -e "  CODESIGN_IDENTITY=\"Developer ID Application: Your Name\" ./macos/build_app_bundle.sh"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
