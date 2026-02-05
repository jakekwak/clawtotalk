#!/bin/bash
# Build script for Android APK

set -e

echo "Building Dioxus Voice Assistant for Android..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if cargo-apk is installed
if ! command -v cargo-apk &> /dev/null; then
    echo -e "${RED}Error: cargo-apk is not installed${NC}"
    echo "Install it with: cargo install cargo-apk"
    exit 1
fi

# Check if Android NDK is configured
if [ -z "$ANDROID_NDK_HOME" ]; then
    echo -e "${YELLOW}Warning: ANDROID_NDK_HOME is not set${NC}"
    echo "Set it to your Android NDK path, e.g.:"
    echo "export ANDROID_NDK_HOME=\$HOME/Android/Sdk/ndk/25.2.9519653"
fi

# Build type (debug or release)
BUILD_TYPE=${1:-debug}

if [ "$BUILD_TYPE" = "release" ]; then
    echo -e "${GREEN}Building release APK...${NC}"
    cargo apk build --release --target aarch64-linux-android
    
    APK_PATH="target/release/apk/dioxus-voice-assistant.apk"
    echo -e "${GREEN}Release APK built successfully!${NC}"
else
    echo -e "${GREEN}Building debug APK...${NC}"
    cargo apk build --target aarch64-linux-android
    
    APK_PATH="target/debug/apk/dioxus-voice-assistant.apk"
    echo -e "${GREEN}Debug APK built successfully!${NC}"
fi

echo -e "${GREEN}APK location: $APK_PATH${NC}"

# Optional: Install to connected device
if [ "$2" = "install" ]; then
    echo -e "${YELLOW}Installing APK to device...${NC}"
    
    if ! command -v adb &> /dev/null; then
        echo -e "${RED}Error: adb is not installed${NC}"
        exit 1
    fi
    
    # Check if device is connected
    if ! adb devices | grep -q "device$"; then
        echo -e "${RED}Error: No Android device connected${NC}"
        exit 1
    fi
    
    adb install -r "$APK_PATH"
    echo -e "${GREEN}APK installed successfully!${NC}"
    
    # Optional: Launch app
    if [ "$3" = "launch" ]; then
        echo -e "${YELLOW}Launching app...${NC}"
        adb shell am start -n com.dioxus.voiceassistant/.MainActivity
        echo -e "${GREEN}App launched!${NC}"
    fi
fi

echo -e "${GREEN}Build complete!${NC}"
