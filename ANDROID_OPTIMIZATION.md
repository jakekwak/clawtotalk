# Android Platform Optimization Guide

This document describes the Android-specific optimizations implemented for the Dioxus Voice Assistant application.

## Overview

The Android platform implementation includes:
- Runtime permission handling via JNI
- Battery optimization management
- Background service support
- Memory and performance optimizations
- Network efficiency improvements

## Implementation Details

### 1. Permission Management

#### Runtime Permission System
The app implements a comprehensive permission system using JNI to interact with Android APIs:

**Features:**
- Check permission status before accessing microphone
- Request permissions with proper Android dialogs
- Handle permission grant/deny results
- Show permission rationale when needed
- Direct users to app settings for manual permission grant

**Code Location:** `src/platform/android.rs`

**Key Functions:**
```rust
// Check if audio permission is granted
pub fn check_audio_permissions() -> bool

// Request audio recording permission
pub fn request_audio_permissions() -> Result<(), AudioError>

// Check if should show permission rationale
pub fn should_show_permission_rationale() -> Result<bool, AudioError>

// Open app settings for manual permission grant
pub fn open_app_settings() -> Result<(), AudioError>
```

#### Permission Flow
1. **App Launch**: Check if RECORD_AUDIO permission is granted
2. **First Request**: Show system permission dialog
3. **User Denies**: Show rationale explaining why permission is needed
4. **Permanently Denied**: Provide button to open app settings
5. **Granted**: Proceed with audio recording

### 2. Battery Optimization

#### Background Restrictions
Android aggressively restricts background apps to save battery. The app handles this by:

**Detection:**
- Check if app is subject to battery restrictions
- Detect if background execution is limited
- Monitor battery optimization status

**Exemption Request:**
- Request user to exempt app from battery optimization
- Explain benefits of exemption (reliable audio recording)
- Handle user acceptance/rejection gracefully

**Code Functions:**
```rust
// Request battery optimization exemption
pub fn request_battery_optimization_exemption() -> Result<(), AudioError>

// Check if background restrictions are enabled
pub fn check_background_restrictions() -> Result<bool, AudioError>
```

#### Best Practices
- Only request exemption when necessary (e.g., for Auto recording mode)
- Clearly explain to users why exemption improves experience
- Provide fallback functionality if exemption is denied
- Monitor battery usage and optimize accordingly

### 3. Background Service Handling

#### Foreground Service
For reliable background audio recording, the app uses a foreground service:

**Requirements:**
- Android 8.0+ (API 26) requires foreground service for background audio
- Android 14+ (API 34) requires specific foreground service type (microphone)
- Must display persistent notification while service is running

**Service Configuration:**
```xml
<service
    android:name=".AudioRecordingService"
    android:enabled="true"
    android:exported="false"
    android:foregroundServiceType="microphone">
</service>
```

**Permissions:**
```xml
<uses-permission android:name="android.permission.FOREGROUND_SERVICE" />
<uses-permission android:name="android.permission.FOREGROUND_SERVICE_MICROPHONE" />
```

#### Service Lifecycle
1. **Start**: User initiates recording while app is in background
2. **Notification**: Display ongoing notification with recording status
3. **Recording**: Process audio in foreground service
4. **Stop**: User stops recording or completes interaction
5. **Cleanup**: Remove notification and stop service

#### Limitations
- Android 12+ restricts starting services from background
- App must be in foreground or have recent user interaction
- Service may be killed under extreme memory pressure
- Implement service restart logic for reliability

### 4. Memory Optimization

#### Audio Buffer Management
Efficient memory usage is critical on mobile devices:

**Strategies:**
- Use ring buffers for audio data to avoid allocations
- Limit conversation history size (e.g., last 50 messages)
- Release audio buffers immediately after processing
- Compress audio data before network transmission

**Implementation:**
```rust
// Efficient audio buffer handling
const MAX_BUFFER_SIZE: usize = 16384; // 16KB chunks
const MAX_CONVERSATION_HISTORY: usize = 50;

// Reuse buffers instead of allocating new ones
let mut audio_buffer = Vec::with_capacity(MAX_BUFFER_SIZE);
```

#### Memory Monitoring
- Monitor app memory usage via Android profiler
- Implement memory pressure callbacks
- Release caches when memory is low
- Avoid memory leaks in JNI code

### 5. Network Optimization

#### Bandwidth Efficiency
Mobile data is expensive and limited:

**Optimizations:**
- Compress audio before transmission (Opus codec recommended)
- Use efficient serialization (MessagePack or Protocol Buffers)
- Implement request batching where possible
- Cache server responses appropriately

**Audio Compression:**
```rust
// Example: Compress audio to reduce bandwidth
// Original: 16kHz, 16-bit PCM = ~32KB/sec
// Compressed: Opus codec = ~8KB/sec (4x reduction)
```

#### Connection Management
- Detect network type (WiFi vs mobile data)
- Adjust quality based on connection speed
- Implement exponential backoff for retries
- Handle network transitions gracefully

**Network Detection:**
```rust
// Check network state before large transfers
if is_mobile_data() && !user_allowed_mobile_data() {
    show_wifi_recommendation();
}
```

### 6. Performance Optimizations

#### App Startup Time
Target: < 3 seconds (Requirement 11.1)

**Optimizations:**
- Lazy load non-critical components
- Initialize audio system asynchronously
- Defer network checks until needed
- Use splash screen for perceived performance

**Implementation:**
```rust
// Lazy initialization
static AUDIO_MANAGER: OnceLock<AudioManager> = OnceLock::new();

fn get_audio_manager() -> &'static AudioManager {
    AUDIO_MANAGER.get_or_init(|| {
        AudioManager::new().expect("Failed to initialize audio")
    })
}
```

#### Recording Latency
Target: < 100ms (Requirement 11.2)

**Optimizations:**
- Pre-initialize audio stream on app start
- Use low-latency audio mode
- Minimize buffer sizes (trade-off with reliability)
- Optimize audio processing pipeline

**Low-Latency Configuration:**
```rust
// Configure CPAL for low latency
let config = StreamConfig {
    channels: 1,
    sample_rate: SampleRate(16000),
    buffer_size: BufferSize::Fixed(256), // Small buffer for low latency
};
```

### 7. Android-Specific UI Considerations

#### Touch Optimization
- Larger touch targets (minimum 48dp)
- Proper touch feedback (ripple effects)
- Gesture support (swipe, long-press)
- Accessibility support (TalkBack)

#### Screen Sizes
- Responsive layout for different screen sizes
- Support for tablets and foldables
- Handle orientation changes gracefully
- Optimize for different aspect ratios

#### Material Design
- Follow Material Design 3 guidelines
- Use Android system colors and themes
- Implement proper elevation and shadows
- Support dark mode

## Testing on Android

### Device Testing
Test on various Android versions and devices:
- **Android 8.0 (API 26)**: Minimum supported version
- **Android 10 (API 29)**: Scoped storage changes
- **Android 12 (API 31)**: Background restrictions
- **Android 14 (API 34)**: Latest features and restrictions

### Permission Testing
1. **First Install**: Verify permission request appears
2. **Deny Permission**: Check error handling and rationale
3. **Revoke Permission**: Test re-request flow
4. **Permanently Deny**: Verify settings redirect works

### Battery Testing
1. **Doze Mode**: Test app behavior in Doze mode
2. **App Standby**: Verify functionality in standby
3. **Battery Saver**: Test with battery saver enabled
4. **Background Restrictions**: Verify detection and handling

### Performance Testing
1. **Startup Time**: Measure cold start time
2. **Recording Latency**: Measure time from button press to recording start
3. **Memory Usage**: Monitor memory consumption over time
4. **Battery Drain**: Measure battery usage during recording

## Build and Deployment

### Prerequisites
```bash
# Install Rust Android targets
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add i686-linux-android
rustup target add x86_64-linux-android

# Install cargo-apk
cargo install cargo-apk

# Set Android NDK path
export ANDROID_NDK_HOME=$HOME/Android/Sdk/ndk/25.2.9519653
```

### Build Commands
```bash
# Debug build
./android/build_android.sh debug

# Release build
./android/build_android.sh release

# Build and install
./android/build_android.sh release install

# Build, install, and launch
./android/build_android.sh release install launch
```

### Release Checklist
- [ ] Test on minimum SDK version (API 26)
- [ ] Test on target SDK version (API 34)
- [ ] Verify all permissions work correctly
- [ ] Test battery optimization handling
- [ ] Verify background service functionality
- [ ] Check memory usage and leaks
- [ ] Test network efficiency
- [ ] Verify performance requirements met
- [ ] Test on various screen sizes
- [ ] Verify accessibility features
- [ ] Sign APK with release key
- [ ] Generate ProGuard mapping file

## Troubleshooting

### Common Issues

#### Permission Denied Error
**Symptom:** App crashes or shows error when trying to record
**Solution:**
1. Check AndroidManifest.xml includes RECORD_AUDIO permission
2. Verify permission is requested at runtime
3. Check user granted permission in system settings
4. Review logcat for permission-related errors

#### Background Recording Fails
**Symptom:** Recording stops when app goes to background
**Solution:**
1. Verify foreground service is implemented
2. Check notification is displayed
3. Request battery optimization exemption
4. Test on different Android versions

#### High Battery Drain
**Symptom:** App uses excessive battery
**Solution:**
1. Profile app with Android Profiler
2. Check for wake locks not being released
3. Verify audio stream is stopped when not recording
4. Optimize network requests (reduce frequency)

#### Memory Leaks
**Symptom:** App memory usage grows over time
**Solution:**
1. Use LeakCanary to detect leaks
2. Check JNI global references are released
3. Verify audio buffers are freed
4. Review conversation history size limits

### Debug Commands
```bash
# View logs
adb logcat | grep "VoiceAssistant"

# Check permissions
adb shell dumpsys package com.dioxus.voiceassistant | grep permission

# Monitor memory
adb shell dumpsys meminfo com.dioxus.voiceassistant

# Check battery stats
adb shell dumpsys batterystats com.dioxus.voiceassistant

# Force stop app
adb shell am force-stop com.dioxus.voiceassistant

# Clear app data
adb shell pm clear com.dioxus.voiceassistant
```

## Performance Benchmarks

### Target Metrics
- **App Startup**: < 3 seconds (cold start)
- **Recording Latency**: < 100ms
- **Memory Usage**: < 150MB (typical)
- **Battery Drain**: < 5% per hour (active recording)
- **Network Usage**: < 1MB per minute (compressed audio)

### Measurement Tools
- Android Studio Profiler
- Systrace
- Battery Historian
- Network Profiler
- Memory Profiler

## Future Improvements

### Planned Optimizations
1. **Adaptive Quality**: Adjust audio quality based on network conditions
2. **Offline Mode**: Cache responses for offline playback
3. **Voice Activation**: Always-on voice detection (with user consent)
4. **Multi-language**: Optimize for multiple languages
5. **Wear OS**: Support for Android Wear devices

### Research Areas
- On-device VAD for reduced latency
- Edge ML for local speech recognition
- Bluetooth audio device support
- Android Auto integration

## References

- [Android Developer Guide](https://developer.android.com/guide)
- [Android Performance Patterns](https://www.youtube.com/playlist?list=PLWz5rJ2EKKc9CBxr3BVjPTPoDPLdPIFCE)
- [Android Battery Optimization](https://developer.android.com/topic/performance/power)
- [Android Audio Guide](https://developer.android.com/guide/topics/media/audio-app/building-an-audio-app)
- [JNI Tips](https://developer.android.com/training/articles/perf-jni)
