# macOS Platform Configuration

This directory contains macOS-specific configuration files for the Dioxus Voice Assistant.

## Files

### Info.plist
The `Info.plist` file contains essential metadata and configuration for the macOS application bundle:

- **Bundle Identifier**: `com.squidcode.dioxus-voice-assistant`
- **Minimum macOS Version**: 10.15 (Catalina)
- **Privacy Permissions**: Microphone access description
- **Network Security**: Configured for Tailscale and local network access
- **High Resolution Support**: Enabled for Retina displays

## Building for macOS

### Development Build
```bash
cargo build --target x86_64-apple-darwin
# or for Apple Silicon
cargo build --target aarch64-apple-darwin
```

### Release Build
```bash
cargo build --release --target x86_64-apple-darwin
# or for Apple Silicon
cargo build --release --target aarch64-apple-darwin
```

### Universal Binary (Intel + Apple Silicon)
```bash
# Build for both architectures
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Create universal binary
lipo -create \
    target/x86_64-apple-darwin/release/dioxus-voice-assistant \
    target/aarch64-apple-darwin/release/dioxus-voice-assistant \
    -output target/release/dioxus-voice-assistant-universal
```

## Creating an App Bundle

To create a proper macOS application bundle:

```bash
# Create bundle structure
mkdir -p DioxusVoiceAssistant.app/Contents/MacOS
mkdir -p DioxusVoiceAssistant.app/Contents/Resources

# Copy executable
cp target/release/dioxus-voice-assistant DioxusVoiceAssistant.app/Contents/MacOS/

# Copy Info.plist
cp macos/Info.plist DioxusVoiceAssistant.app/Contents/

# Copy icon (if available)
# cp macos/AppIcon.icns DioxusVoiceAssistant.app/Contents/Resources/
```

## Code Signing

For distribution, the app must be signed with an Apple Developer certificate:

```bash
# Sign the app bundle
codesign --force --deep --sign "Developer ID Application: Your Name" \
    DioxusVoiceAssistant.app

# Verify signature
codesign --verify --verbose DioxusVoiceAssistant.app

# Check entitlements
codesign -d --entitlements - DioxusVoiceAssistant.app
```

### Entitlements

Create an `entitlements.plist` file for additional permissions:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.device.audio-input</key>
    <true/>
    <key>com.apple.security.network.client</key>
    <true/>
    <key>com.apple.security.network.server</key>
    <true/>
</dict>
</plist>
```

Sign with entitlements:
```bash
codesign --force --deep --sign "Developer ID Application: Your Name" \
    --entitlements macos/entitlements.plist \
    DioxusVoiceAssistant.app
```

## Notarization

For distribution outside the Mac App Store, the app must be notarized:

```bash
# Create a zip archive
ditto -c -k --keepParent DioxusVoiceAssistant.app DioxusVoiceAssistant.zip

# Submit for notarization
xcrun notarytool submit DioxusVoiceAssistant.zip \
    --apple-id "your-email@example.com" \
    --team-id "YOUR_TEAM_ID" \
    --password "app-specific-password" \
    --wait

# Staple the notarization ticket
xcrun stapler staple DioxusVoiceAssistant.app

# Verify notarization
spctl -a -vv DioxusVoiceAssistant.app
```

## Creating a DMG Installer

```bash
# Create a temporary directory
mkdir -p dmg_temp
cp -R DioxusVoiceAssistant.app dmg_temp/

# Create symbolic link to Applications folder
ln -s /Applications dmg_temp/Applications

# Create DMG
hdiutil create -volname "Dioxus Voice Assistant" \
    -srcfolder dmg_temp \
    -ov -format UDZO \
    DioxusVoiceAssistant.dmg

# Clean up
rm -rf dmg_temp
```

## Permissions

The app requires the following permissions:

1. **Microphone Access**: Required for voice recording
   - Configured in Info.plist with `NSMicrophoneUsageDescription`
   - User will see a permission dialog on first microphone access

2. **Network Access**: Required for server communication
   - Configured to allow local network access (Tailscale)
   - HTTPS connections to external servers

## Testing Permissions

To test microphone permissions:

```bash
# Reset permissions (for testing)
tccutil reset Microphone com.squidcode.dioxus-voice-assistant

# Check current permissions
sqlite3 ~/Library/Application\ Support/com.apple.TCC/TCC.db \
    "SELECT * FROM access WHERE service='kTCCServiceMicrophone'"
```

## Troubleshooting

### Permission Denied
If microphone access is denied:
1. Open System Preferences → Security & Privacy → Privacy → Microphone
2. Enable access for "Dioxus Voice Assistant"
3. Restart the application

### Code Signing Issues
If you encounter code signing errors:
```bash
# Remove existing signature
codesign --remove-signature DioxusVoiceAssistant.app

# Re-sign with verbose output
codesign --force --deep --sign "Developer ID Application: Your Name" \
    --verbose=4 DioxusVoiceAssistant.app
```

### Gatekeeper Issues
If macOS blocks the app:
```bash
# Remove quarantine attribute
xattr -dr com.apple.quarantine DioxusVoiceAssistant.app

# Or allow the app in System Preferences
# System Preferences → Security & Privacy → General → "Open Anyway"
```

## Performance Optimization

The macOS implementation includes several optimizations:

1. **CoreAudio Backend**: Native audio processing with low latency
2. **Hardware Acceleration**: Enabled for audio processing
3. **Thread Priority**: High-priority threads for audio processing
4. **Buffer Optimization**: Configurable buffer sizes (5-10ms)
5. **Apple Silicon Support**: Optimized for M1/M2/M3 processors

## References

- [Apple Developer Documentation](https://developer.apple.com/documentation/)
- [App Sandbox](https://developer.apple.com/documentation/security/app_sandbox)
- [Code Signing Guide](https://developer.apple.com/library/archive/documentation/Security/Conceptual/CodeSigningGuide/)
- [Notarization Guide](https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution)
- [CoreAudio Documentation](https://developer.apple.com/documentation/coreaudio)
