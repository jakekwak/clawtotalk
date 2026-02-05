# iOS Platform Optimization

This directory contains iOS-specific configurations and build scripts for the Dioxus Voice Assistant.

## Overview

The iOS implementation uses AVAudioEngine for audio processing and AVAudioSession for managing audio permissions and background audio capabilities. The app is optimized for both performance and battery life.

## Features

### 1. AVAudioEngine Integration
- **Low-latency audio processing**: Configurable buffer sizes (256-1024 frames)
- **Hardware acceleration**: Leverages iOS audio hardware for efficient processing
- **Multiple audio session categories**: Record, Playback, PlayAndRecord, Ambient
- **Background audio support**: Continue audio playback/recording in background

### 2. Permission Management
- **Microphone access**: Automatic permission request via AVAudioSession
- **Speech recognition**: Optional speech recognition permission
- **Local network**: Access to Tailscale or local server connections
- **Clear permission descriptions**: User-friendly explanations in Info.plist

### 3. Battery Optimization
- **Adaptive buffer sizing**: Larger buffers when battery life is priority
- **Efficient audio session management**: Proper activation/deactivation
- **Background mode optimization**: Minimal CPU usage in background
- **Hardware acceleration**: Offload processing to dedicated audio hardware

### 4. Background Audio
- **Continuous recording**: Record audio even when app is in background
- **Audio playback**: Play TTS responses in background
- **Session interruption handling**: Gracefully handle phone calls and other interruptions

## Requirements

- **iOS Version**: 14.0 or later
- **Device**: iPhone, iPad, or iPod touch with microphone
- **Xcode**: 14.0 or later (for building)
- **Apple Developer Account**: Required for device testing and App Store distribution

## Building for iOS

### Prerequisites

1. Install Rust iOS targets:
```bash
rustup target add aarch64-apple-ios
rustup target add aarch64-apple-ios-sim
rustup target add x86_64-apple-ios
```

2. Install Xcode from the Mac App Store

3. Install Xcode Command Line Tools:
```bash
xcode-select --install
```

### Build Script

Use the provided build script to compile for iOS:

```bash
./ios/build_ios.sh
```

This will:
- Build for iOS device (ARM64)
- Build for iOS simulator (ARM64 and x86_64)
- Create a universal simulator library
- Generate an app bundle structure

### Manual Build

For device (ARM64):
```bash
cargo build --release --target aarch64-apple-ios --lib
```

For simulator (ARM64 - Apple Silicon Macs):
```bash
cargo build --release --target aarch64-apple-ios-sim --lib
```

For simulator (x86_64 - Intel Macs):
```bash
cargo build --release --target x86_64-apple-ios --lib
```

## Configuration

### Info.plist

The `Info.plist` file contains all necessary permissions and configurations:

#### Required Permissions
- **NSMicrophoneUsageDescription**: Microphone access for voice recording
- **NSLocalNetworkUsageDescription**: Local network access for server connection

#### Optional Permissions
- **NSSpeechRecognitionUsageDescription**: Speech recognition (if using on-device STT)

#### Background Modes
- **audio**: Background audio recording and playback
- **fetch**: Background server communication
- **remote-notification**: Push notifications (optional)

### Audio Session Categories

The app supports multiple audio session categories:

```rust
pub enum AudioSessionCategory {
    Record,        // Recording only
    Playback,      // Playback only
    PlayAndRecord, // Both (default)
    Ambient,       // Mix with other audio
}
```

### Performance Modes

#### Low-Latency Mode
```rust
let mut optimizer = IOSAudioOptimizer::with_low_latency();
// Buffer: 256 frames (~5ms at 48kHz)
// Priority: Realtime audio processing
```

#### Battery-Optimized Mode
```rust
let mut optimizer = IOSAudioOptimizer::new();
optimizer.optimize_for_battery();
// Buffer: 1024 frames (~21ms at 48kHz)
// Priority: Battery life
```

## Usage

### Request Permissions

```rust
use dioxus_voice_assistant::platform::ios;

// Request microphone permission
ios::request_audio_permissions()?;

// Check if permission is granted
if ios::check_audio_permissions() {
    println!("Microphone access granted");
} else {
    println!("Microphone access denied");
    // Open settings for user to grant permission
    ios::open_microphone_settings()?;
}
```

### Configure Audio Session

```rust
use dioxus_voice_assistant::platform::ios::{IOSAudioOptimizer, AudioSessionCategory};

let mut optimizer = IOSAudioOptimizer::new();

// Set category for recording and playback
optimizer.set_session_category(AudioSessionCategory::PlayAndRecord);

// Enable background audio
optimizer.enable_background_audio();

// Configure the audio session
optimizer.configure_audio_session()?;
```

### Enable Low-Latency Mode

```rust
let mut optimizer = IOSAudioOptimizer::new();
optimizer.enable_low_latency()?;

// Or create with low-latency preset
let optimizer = IOSAudioOptimizer::with_low_latency();
```

### Battery Optimization

```rust
let mut optimizer = IOSAudioOptimizer::new();
optimizer.optimize_for_battery();
```

## Testing

### Simulator Testing

1. Build for simulator:
```bash
cargo build --release --target aarch64-apple-ios-sim --lib
```

2. Create an Xcode project and link the library

3. Run in iOS Simulator

**Note**: Microphone access may be limited in simulator. Test on real device for full functionality.

### Device Testing

1. Connect your iOS device via USB

2. Build for device:
```bash
cargo build --release --target aarch64-apple-ios --lib
```

3. Use Xcode to sign and deploy to device

4. Grant microphone permission when prompted

5. Test all features:
   - Voice recording
   - Audio playback
   - Background audio
   - Server connection (Tailscale or public URL)

## Troubleshooting

### Permission Denied

If microphone permission is denied:
1. The app will show an error message
2. User can open Settings via the app
3. Navigate to Privacy & Security > Microphone
4. Enable permission for Voice Assistant

### Background Audio Not Working

Ensure Info.plist has:
```xml
<key>UIBackgroundModes</key>
<array>
    <string>audio</string>
</array>
```

And audio session is configured:
```rust
optimizer.enable_background_audio();
optimizer.configure_audio_session()?;
```

### High Battery Drain

Switch to battery-optimized mode:
```rust
optimizer.optimize_for_battery();
```

This increases buffer size and reduces CPU wakeups.

### Audio Latency Issues

Enable low-latency mode:
```rust
optimizer.enable_low_latency()?;
```

This reduces buffer size and increases thread priority.

## Performance Benchmarks

### Startup Time
- **Target**: < 3 seconds
- **Typical**: 1-2 seconds on iPhone 12 or later

### Recording Latency
- **Target**: < 100ms
- **Low-latency mode**: ~5-10ms
- **Normal mode**: ~20-30ms
- **Battery-optimized**: ~40-50ms

### Battery Usage
- **Active recording**: ~5-10% per hour
- **Background audio**: ~3-5% per hour
- **Idle with server connection**: ~1-2% per hour

## App Store Submission

### Requirements

1. **Code Signing**: Set up in Xcode
2. **Provisioning Profile**: Create in Apple Developer Portal
3. **App Icon**: Add to Assets.xcassets
4. **Screenshots**: Required for App Store listing
5. **Privacy Policy**: Required for microphone access

### Checklist

- [ ] All permissions have clear descriptions in Info.plist
- [ ] App icon is added (all required sizes)
- [ ] Launch screen is configured
- [ ] Code signing is set up
- [ ] App has been tested on real device
- [ ] Privacy policy is available
- [ ] App Store screenshots are prepared
- [ ] App Store description is written

### Submission Process

1. Archive the app in Xcode
2. Validate the archive
3. Upload to App Store Connect
4. Fill in app metadata
5. Submit for review

## Platform-Specific Notes

### iPhone vs iPad

The app supports both iPhone and iPad with adaptive layouts:
- iPhone: Portrait and landscape
- iPad: All orientations including split-view

### iOS Version Support

- **Minimum**: iOS 14.0
- **Recommended**: iOS 15.0 or later
- **Tested**: iOS 14.0 - 17.0

### Device Compatibility

- iPhone 6s and later
- iPad (5th generation) and later
- iPad Pro (all models)
- iPad Air (3rd generation) and later
- iPad mini (5th generation) and later
- iPod touch (7th generation)

## Resources

- [AVAudioEngine Documentation](https://developer.apple.com/documentation/avfaudio/avaudioengine)
- [AVAudioSession Documentation](https://developer.apple.com/documentation/avfaudio/avaudiosession)
- [iOS App Distribution Guide](https://developer.apple.com/documentation/xcode/distributing-your-app-for-beta-testing-and-releases)
- [Background Execution](https://developer.apple.com/documentation/avfoundation/media_playback/creating_a_basic_video_player_ios_and_tvos/enabling_background_audio)

## License

MIT License - See LICENSE file for details
