# Windows Platform Optimizations

This document describes the Windows-specific optimizations implemented for the Dioxus Voice Assistant.

## Overview

The Windows platform implementation includes several optimizations to achieve low-latency audio processing and optimal performance on Windows 10/11 systems.

## WASAPI Audio Backend Optimizations

### 1. Exclusive Mode
- **What it is**: Direct hardware access bypassing the Windows audio mixer
- **Benefits**: Lower latency (5-10ms vs 20-30ms in shared mode)
- **Trade-off**: Exclusive control of audio device (other apps can't use it)
- **Usage**: Automatically enabled in low-latency mode

### 2. Event-Driven Processing
- **What it is**: Audio processing triggered by hardware events instead of polling
- **Benefits**: Lower CPU usage, more consistent timing
- **Implementation**: Uses WASAPI event-driven mode

### 3. Buffer Size Optimization
- **Shared Mode**: 10ms buffer (480 frames at 48kHz)
- **Exclusive Mode**: 5ms buffer (240 frames at 48kHz)
- **Configurable**: Can be adjusted based on system capabilities

### 4. Thread Priority Management
Three priority levels available:
- **Normal**: Standard thread priority
- **High**: Elevated priority for audio threads (default)
- **TimeCritical**: Highest priority for ultra-low latency

### 5. MMCSS Integration
- **What it is**: Multimedia Class Scheduler Service
- **Benefits**: Windows reserves CPU time for audio threads
- **Task Class**: "Pro Audio" for professional audio applications
- **Result**: Prevents audio glitches during system load

## Permission Handling

### Microphone Privacy Settings
- Automatic detection of Windows 10/11 privacy settings
- Registry-based permission checking
- Direct link to Windows Settings for easy configuration

### Permission Flow
1. App checks registry for microphone access permission
2. If denied, shows user-friendly error message
3. Provides button to open Windows Settings
4. User enables microphone access
5. App automatically detects permission grant

### Registry Keys Checked
```
HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\microphone
```

## Application Manifest

### DPI Awareness
- **PerMonitorV2**: Best DPI scaling for high-resolution displays
- Prevents blurry UI on 4K monitors
- Automatic scaling for multi-monitor setups

### Windows Compatibility
- Explicitly declares Windows 10 and Windows 11 support
- Ensures proper behavior on latest Windows versions

### Execution Level
- **asInvoker**: Runs with user privileges (no UAC prompt)
- No administrator rights required
- Better security posture

## Resource Embedding

### Application Icon
- Multi-resolution ICO file (16x16 to 256x256)
- Embedded in executable at build time
- Visible in taskbar, window title, and file explorer

### Version Information
- Company name, product name, version
- File description and copyright
- Visible in file properties dialog

### Manifest Embedding
- Application manifest embedded in executable
- No external manifest file needed
- Ensures compatibility settings are always applied

## Build Configuration

### Linked Libraries
- **ole32**: COM support for Windows APIs
- **winmm**: Multimedia timer functions
- **avrt**: MMCSS (Multimedia Class Scheduler Service)

### Resource Compilation
- Uses `embed-resource` crate
- Automatically compiles .rc files
- Works with both MSVC and GNU toolchains

## Performance Characteristics

### Latency
- **Shared Mode**: ~10-15ms round-trip latency
- **Exclusive Mode**: ~5-8ms round-trip latency
- **Target**: <100ms for recording start (requirement 11.2)

### CPU Usage
- Event-driven processing reduces polling overhead
- High-priority threads prevent audio dropouts
- MMCSS ensures consistent performance under load

### Memory
- Optimized buffer sizes reduce memory footprint
- Efficient audio format conversion
- No memory leaks in audio streams

## Usage Examples

### Basic Usage (Default Settings)
```rust
use dioxus_voice_assistant::audio::CrossPlatformAudioManager;

let manager = CrossPlatformAudioManager::new()?;
// Uses default settings: shared mode, 10ms buffer, high priority
```

### Low-Latency Mode
```rust
let mut manager = CrossPlatformAudioManager::new()?;

#[cfg(target_os = "windows")]
{
    manager.enable_low_latency()?;
    // Enables: exclusive mode, 5ms buffer, time-critical priority
}
```

### Custom Configuration
```rust
use dioxus_voice_assistant::platform::windows::{WindowsAudioOptimizer, ThreadPriority};

let mut optimizer = WindowsAudioOptimizer::new();
optimizer.enable_exclusive_mode()?;
optimizer.set_buffer_size(240); // 5ms at 48kHz
optimizer.set_thread_priority(ThreadPriority::TimeCritical);
optimizer.apply_thread_optimizations()?;
```

### Permission Handling
```rust
use dioxus_voice_assistant::platform::windows;

// Check if microphone access is granted
if !windows::check_audio_permissions() {
    // Open Windows Settings for user to grant permission
    windows::open_microphone_settings()?;
}
```

## Troubleshooting

### High Latency
1. Enable low-latency mode
2. Check for other audio applications
3. Update audio drivers
4. Disable audio enhancements in Windows

### Permission Denied
1. Open Windows Settings → Privacy → Microphone
2. Enable "Allow apps to access your microphone"
3. Enable access for the specific app
4. Restart the application

### Audio Glitches
1. Increase buffer size
2. Close other audio applications
3. Check CPU usage
4. Update audio drivers

### Build Errors
1. Ensure Windows SDK is installed
2. Install Visual Studio Build Tools
3. For GNU toolchain, install mingw-w64
4. Check that embed-resource is in Cargo.toml

## Testing

### Unit Tests
All Windows-specific code includes unit tests:
```bash
cargo test --lib platform::windows
```

### Integration Tests
Test full audio pipeline on Windows:
```bash
cargo test --test integration_tests
```

### Manual Testing
1. Build release version: `cargo build --release`
2. Run executable: `target/release/dioxus-voice-assistant.exe`
3. Test microphone recording
4. Verify low latency (<100ms)
5. Check CPU usage (<10% idle)

## Future Improvements

### Planned
- [ ] Automatic buffer size adjustment based on system capabilities
- [ ] Audio device hot-plugging support
- [ ] ASIO driver support for professional audio interfaces
- [ ] Spatial audio support for Windows 11

### Under Consideration
- [ ] Windows Hello integration for authentication
- [ ] Windows notification system integration
- [ ] Windows 11 Snap Layouts support
- [ ] ARM64 Windows support

## References

- [WASAPI Documentation](https://docs.microsoft.com/en-us/windows/win32/coreaudio/wasapi)
- [MMCSS Documentation](https://docs.microsoft.com/en-us/windows/win32/procthread/multimedia-class-scheduler-service)
- [Windows App Manifest](https://docs.microsoft.com/en-us/windows/win32/sbscs/application-manifests)
- [DPI Awareness](https://docs.microsoft.com/en-us/windows/win32/hidpi/high-dpi-desktop-application-development-on-windows)
