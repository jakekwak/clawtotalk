# iOS Quick Start Guide

## 5-Minute Setup

### 1. Install Prerequisites

```bash
# Install Rust iOS targets
rustup target add aarch64-apple-ios
rustup target add aarch64-apple-ios-sim
```

### 2. Build for iOS

```bash
# Quick build for simulator (Apple Silicon Mac)
cargo build --release --target aarch64-apple-ios-sim --lib

# Or use the build script
./ios/build_ios.sh
```

### 3. Basic Usage

```rust
use dioxus_voice_assistant::platform::ios;

// Request microphone permission
ios::request_audio_permissions()?;

// Check permission status
if ios::check_audio_permissions() {
    println!("Ready to record!");
}
```

### 4. Configure Audio Session

```rust
use dioxus_voice_assistant::platform::ios::{IOSAudioOptimizer, AudioSessionCategory};

let mut optimizer = IOSAudioOptimizer::new();
optimizer.set_session_category(AudioSessionCategory::PlayAndRecord);
optimizer.enable_background_audio();
optimizer.configure_audio_session()?;
```

## Common Configurations

### Low-Latency Mode (for real-time interaction)

```rust
let optimizer = IOSAudioOptimizer::with_low_latency();
// Buffer: 256 frames (~5ms)
// Best for: Real-time voice interaction
```

### Battery-Optimized Mode (for extended use)

```rust
let mut optimizer = IOSAudioOptimizer::new();
optimizer.optimize_for_battery();
// Buffer: 1024 frames (~21ms)
// Best for: Long recording sessions
```

### Background Audio Mode

```rust
let mut optimizer = IOSAudioOptimizer::new();
optimizer.enable_background_audio();
optimizer.configure_audio_session()?;
// Enables: Recording/playback in background
```

## Testing

### Simulator

```bash
# Build for simulator
cargo build --target aarch64-apple-ios-sim --lib

# Note: Microphone may not work in simulator
# Use real device for full testing
```

### Real Device

1. Connect iPhone/iPad via USB
2. Build for device: `cargo build --target aarch64-apple-ios --lib`
3. Use Xcode to sign and deploy
4. Grant microphone permission when prompted

## Troubleshooting

### Permission Denied
- Check Info.plist has `NSMicrophoneUsageDescription`
- Guide user to Settings > Privacy > Microphone

### Background Audio Not Working
- Verify `UIBackgroundModes` includes "audio" in Info.plist
- Call `enable_background_audio()` before configuring session

### Build Errors
- Ensure Xcode is installed: `xcode-select --install`
- Verify iOS targets: `rustup target list | grep ios`

## Next Steps

- Read [ios/README.md](README.md) for detailed documentation
- Check [IOS_OPTIMIZATION.md](../IOS_OPTIMIZATION.md) for optimization guide
- Review [Info.plist](Info.plist) for permission configuration

## Support

For issues or questions:
1. Check the troubleshooting section in ios/README.md
2. Review Apple's AVAudioEngine documentation
3. Verify Info.plist permissions are correctly configured
