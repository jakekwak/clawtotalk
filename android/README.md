# Android Platform Configuration

This directory contains Android-specific configuration files for the Dioxus Voice Assistant application.

## Files

### AndroidManifest.xml
The main Android manifest file that declares:
- **Permissions**: Audio recording, internet access, network state, wake lock, battery optimization
- **Features**: Microphone hardware requirement
- **Activities**: MainActivity as the app entry point
- **Services**: AudioRecordingService for background audio processing

## Permissions

### Required Permissions
1. **RECORD_AUDIO**: Required for voice recording functionality
   - Runtime permission (dangerous) - must be requested at runtime
   - User must explicitly grant this permission

2. **INTERNET**: Required for server communication
   - Normal permission - granted automatically

3. **ACCESS_NETWORK_STATE**: Monitor network connectivity
   - Normal permission - granted automatically

4. **WAKE_LOCK**: Keep device awake during recording
   - Normal permission - granted automatically

### Optional Permissions
1. **REQUEST_IGNORE_BATTERY_OPTIMIZATIONS**: Exempt app from battery restrictions
   - Special permission - requires user action
   - Improves reliability for background audio processing

2. **FOREGROUND_SERVICE**: Run foreground service
   - Normal permission for API 28+

3. **FOREGROUND_SERVICE_MICROPHONE**: Specify microphone usage in foreground service
   - Required for Android 14+ (API 34+)

## Battery Optimization

The app implements battery optimization handling to ensure reliable audio recording:

### Features
- **Check battery restrictions**: Detect if app is restricted
- **Request exemption**: Guide user to exempt app from battery optimization
- **Foreground service**: Use foreground service for background recording

### Best Practices
1. Only request battery exemption when necessary
2. Explain to users why exemption is needed
3. Provide fallback for users who deny exemption

## Background Service Handling

### Foreground Service
The app uses a foreground service for background audio recording:
- Shows persistent notification when recording
- Prevents system from killing the service
- Required for Android 8.0+ (API 26+)

### Service Lifecycle
1. **Start**: When user initiates recording in background
2. **Notification**: Display ongoing notification
3. **Stop**: When recording completes or user stops

### Limitations
- Android 12+ restricts background service starts
- App must be in foreground or have recent user interaction
- Consider using WorkManager for deferred tasks

## Build Configuration

### Gradle Setup (if using Gradle)
```gradle
android {
    compileSdkVersion 34
    defaultConfig {
        applicationId "com.dioxus.voiceassistant"
        minSdkVersion 26  // Android 8.0
        targetSdkVersion 34  // Android 14
        versionCode 1
        versionName "1.0"
    }
}
```

### Cargo-APK Configuration
Add to `Cargo.toml`:
```toml
[package.metadata.android]
package = "com.dioxus.voiceassistant"
label = "Voice Assistant"
icon = "@mipmap/ic_launcher"
assets = "assets"
res = "android/res"
manifest = "android/AndroidManifest.xml"

[package.metadata.android.sdk]
min_sdk_version = 26
target_sdk_version = 34
```

## Runtime Permission Handling

The app implements proper runtime permission handling:

### Permission Flow
1. **Check permission**: Before accessing microphone
2. **Request if needed**: Show system permission dialog
3. **Handle result**: Process grant/deny
4. **Show rationale**: Explain why permission is needed (if previously denied)
5. **Open settings**: Guide user to app settings (if permanently denied)

### Code Example
```rust
use dioxus_voice_assistant::platform::android;

// Check permission
if !android::check_audio_permissions() {
    // Request permission
    android::request_audio_permissions()?;
}

// Check if should show rationale
if android::should_show_permission_rationale()? {
    // Show explanation to user
    show_permission_explanation();
}

// If permanently denied, open settings
if permission_permanently_denied {
    android::open_app_settings()?;
}
```

## Testing

### Permission Testing
1. **First install**: Test permission request flow
2. **Deny permission**: Test error handling
3. **Revoke permission**: Test re-request flow
4. **Permanently deny**: Test settings redirect

### Battery Optimization Testing
1. **Check restriction status**: Verify detection
2. **Request exemption**: Test user flow
3. **Verify exemption**: Confirm app is exempt

### Background Service Testing
1. **Start service**: Test foreground service start
2. **Notification**: Verify notification display
3. **Recording**: Test audio recording in background
4. **Stop service**: Test clean shutdown

## Troubleshooting

### Permission Denied
- Ensure AndroidManifest.xml includes RECORD_AUDIO permission
- Check that permission is requested at runtime
- Verify user granted permission in system settings

### Background Recording Fails
- Check battery optimization status
- Verify foreground service is running
- Ensure notification is displayed
- Check Android version restrictions

### Service Killed by System
- Request battery optimization exemption
- Use foreground service with notification
- Reduce memory usage
- Handle service restart gracefully

## Android Version Compatibility

### Minimum SDK: 26 (Android 8.0)
- Foreground service support
- Runtime permissions
- Modern audio APIs

### Target SDK: 34 (Android 14)
- Latest security features
- Foreground service types
- Enhanced privacy controls

### Version-Specific Features
- **API 26+**: Foreground service required for background audio
- **API 28+**: Background restrictions
- **API 29+**: Scoped storage
- **API 31+**: Bluetooth permissions split
- **API 33+**: Notification permission required
- **API 34+**: Foreground service types required

## Resources

- [Android Permissions](https://developer.android.com/guide/topics/permissions/overview)
- [Background Execution Limits](https://developer.android.com/about/versions/oreo/background)
- [Foreground Services](https://developer.android.com/guide/components/foreground-services)
- [Battery Optimization](https://developer.android.com/training/monitoring-device-state/doze-standby)
