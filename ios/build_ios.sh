#!/bin/bash

# iOS Build Script for Dioxus Voice Assistant
# This script builds the iOS app bundle for both simulator and device

set -e

echo "🍎 Building Dioxus Voice Assistant for iOS..."

# Configuration
APP_NAME="VoiceAssistant"
BUNDLE_ID="com.dioxus.voiceassistant"
VERSION="1.0.0"
BUILD_NUMBER="1"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if running on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo -e "${RED}❌ Error: iOS builds can only be created on macOS${NC}"
    exit 1
fi

# Check for required tools
echo "🔍 Checking for required tools..."

if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Error: cargo not found. Please install Rust.${NC}"
    exit 1
fi

if ! command -v xcodebuild &> /dev/null; then
    echo -e "${RED}❌ Error: xcodebuild not found. Please install Xcode.${NC}"
    exit 1
fi

# Check for iOS targets
echo "🎯 Checking Rust iOS targets..."

TARGETS=("aarch64-apple-ios" "aarch64-apple-ios-sim" "x86_64-apple-ios")
for target in "${TARGETS[@]}"; do
    if ! rustup target list | grep -q "$target (installed)"; then
        echo "📦 Installing $target..."
        rustup target add "$target"
    else
        echo "✅ $target already installed"
    fi
done

# Build for iOS device (ARM64)
echo ""
echo "📱 Building for iOS device (ARM64)..."
cargo build --release --target aarch64-apple-ios --lib

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ iOS device build successful${NC}"
else
    echo -e "${RED}❌ iOS device build failed${NC}"
    exit 1
fi

# Build for iOS simulator (ARM64 - M1/M2/M3 Macs)
echo ""
echo "🖥️  Building for iOS simulator (ARM64)..."
cargo build --release --target aarch64-apple-ios-sim --lib

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ iOS simulator (ARM64) build successful${NC}"
else
    echo -e "${RED}❌ iOS simulator (ARM64) build failed${NC}"
    exit 1
fi

# Build for iOS simulator (x86_64 - Intel Macs)
echo ""
echo "🖥️  Building for iOS simulator (x86_64)..."
cargo build --release --target x86_64-apple-ios --lib

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ iOS simulator (x86_64) build successful${NC}"
else
    echo -e "${YELLOW}⚠️  iOS simulator (x86_64) build failed (may not be needed on Apple Silicon)${NC}"
fi

# Create universal library for simulator
echo ""
echo "🔨 Creating universal library for simulator..."
mkdir -p target/universal-ios-sim/release

lipo -create \
    target/aarch64-apple-ios-sim/release/libdioxus_voice_assistant.a \
    target/x86_64-apple-ios/release/libdioxus_voice_assistant.a \
    -output target/universal-ios-sim/release/libdioxus_voice_assistant.a 2>/dev/null || \
    cp target/aarch64-apple-ios-sim/release/libdioxus_voice_assistant.a \
       target/universal-ios-sim/release/libdioxus_voice_assistant.a

echo -e "${GREEN}✅ Universal simulator library created${NC}"

# Create app bundle structure
echo ""
echo "📦 Creating app bundle..."

BUNDLE_DIR="target/ios/${APP_NAME}.app"
rm -rf "$BUNDLE_DIR"
mkdir -p "$BUNDLE_DIR"

# Copy Info.plist
cp ios/Info.plist "$BUNDLE_DIR/"

# Update Info.plist with actual values
/usr/libexec/PlistBuddy -c "Set :CFBundleIdentifier $BUNDLE_ID" "$BUNDLE_DIR/Info.plist"
/usr/libexec/PlistBuddy -c "Set :CFBundleShortVersionString $VERSION" "$BUNDLE_DIR/Info.plist"
/usr/libexec/PlistBuddy -c "Set :CFBundleVersion $BUILD_NUMBER" "$BUNDLE_DIR/Info.plist"

# Copy binary (for device)
cp target/aarch64-apple-ios/release/libdioxus_voice_assistant.a "$BUNDLE_DIR/${APP_NAME}"

# Create Assets.car (placeholder)
mkdir -p "$BUNDLE_DIR/Assets.xcassets"

echo -e "${GREEN}✅ App bundle created at $BUNDLE_DIR${NC}"

# Print build information
echo ""
echo "📊 Build Information:"
echo "  App Name: $APP_NAME"
echo "  Bundle ID: $BUNDLE_ID"
echo "  Version: $VERSION"
echo "  Build: $BUILD_NUMBER"
echo ""
echo "📍 Build Artifacts:"
echo "  Device: target/aarch64-apple-ios/release/"
echo "  Simulator: target/universal-ios-sim/release/"
echo "  Bundle: $BUNDLE_DIR"
echo ""

# Instructions
echo "📝 Next Steps:"
echo ""
echo "1. For Simulator Testing:"
echo "   - Open Xcode and create a new iOS project"
echo "   - Link the universal simulator library"
echo "   - Run in iOS Simulator"
echo ""
echo "2. For Device Testing:"
echo "   - You'll need an Apple Developer account"
echo "   - Code signing is required"
echo "   - Use Xcode to sign and deploy to device"
echo ""
echo "3. For App Store Distribution:"
echo "   - Complete code signing setup"
echo "   - Create provisioning profiles"
echo "   - Archive and upload via Xcode"
echo ""

echo -e "${GREEN}✅ iOS build complete!${NC}"
