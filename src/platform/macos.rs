use crate::error::AudioError;
use log::{info, warn, error};

/// macOS-specific audio optimizations for CoreAudio
pub struct MacOSAudioOptimizer {
    /// Buffer size in frames
    buffer_size: u32,
    /// Sample rate
    sample_rate: u32,
    /// Whether to use hardware acceleration
    hardware_acceleration: bool,
    /// Audio thread priority
    thread_priority: ThreadPriority,
    /// Whether to enable low-latency mode
    low_latency_mode: bool,
}

/// Audio thread priority levels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThreadPriority {
    Normal,
    High,
    Realtime,
}

impl MacOSAudioOptimizer {
    pub fn new() -> Self {
        Self {
            buffer_size: 512, // ~10ms at 48kHz
            sample_rate: 48000,
            hardware_acceleration: true,
            thread_priority: ThreadPriority::High,
            low_latency_mode: false,
        }
    }
    
    /// Create optimizer with optimal settings for low latency
    pub fn with_low_latency() -> Self {
        Self {
            buffer_size: 256, // ~5ms at 48kHz
            sample_rate: 48000,
            hardware_acceleration: true,
            thread_priority: ThreadPriority::Realtime,
            low_latency_mode: true,
        }
    }
    
    /// Enable low-latency mode for CoreAudio
    pub fn enable_low_latency(&mut self) -> Result<(), AudioError> {
        #[cfg(target_os = "macos")]
        {
            info!("Enabling CoreAudio low-latency mode");
            self.low_latency_mode = true;
            self.buffer_size = 256; // 5ms at 48kHz
            self.thread_priority = ThreadPriority::Realtime;
            Ok(())
        }
        
        #[cfg(not(target_os = "macos"))]
        {
            warn!("Low-latency mode only available on macOS");
            Err(AudioError::UnsupportedPlatform)
        }
    }
    
    /// Set optimal buffer size for low latency
    pub fn set_buffer_size(&mut self, size: u32) {
        self.buffer_size = size;
        info!("CoreAudio buffer size set to {} frames", size);
    }
    
    /// Set sample rate
    pub fn set_sample_rate(&mut self, rate: u32) {
        self.sample_rate = rate;
        info!("CoreAudio sample rate set to {} Hz", rate);
    }
    
    /// Enable hardware acceleration
    pub fn enable_hardware_acceleration(&mut self) {
        self.hardware_acceleration = true;
        info!("CoreAudio hardware acceleration enabled");
    }
    
    /// Set audio thread priority
    pub fn set_thread_priority(&mut self, priority: ThreadPriority) {
        self.thread_priority = priority;
        info!("Audio thread priority set to {:?}", priority);
    }
    
    /// Apply optimizations to the current thread
    pub fn apply_thread_optimizations(&self) -> Result<(), AudioError> {
        #[cfg(target_os = "macos")]
        {
            self.set_current_thread_priority()?;
            Ok(())
        }
        
        #[cfg(not(target_os = "macos"))]
        {
            warn!("Thread optimizations only available on macOS");
            Ok(())
        }
    }
    
    /// Set current thread priority (macOS-specific)
    #[cfg(target_os = "macos")]
    fn set_current_thread_priority(&self) -> Result<(), AudioError> {
        unsafe {
            let thread = libc::pthread_self();
            
            match self.thread_priority {
                ThreadPriority::Normal => {
                    // Standard priority - use default scheduling
                    info!("Thread priority set to Normal");
                    Ok(())
                }
                ThreadPriority::High => {
                    // High priority for audio processing
                    // On macOS, we use pthread_setschedparam with SCHED_RR
                    // Note: This may require elevated privileges
                    info!("Attempting to set high thread priority");
                    
                    // Create sched_param using mem::zeroed and then set priority
                    let mut param: libc::sched_param = std::mem::zeroed();
                    // Access the priority field through pointer manipulation
                    // since the struct has private fields
                    let priority_ptr = &mut param as *mut libc::sched_param as *mut i32;
                    *priority_ptr = 63;
                    
                    if libc::pthread_setschedparam(thread, libc::SCHED_RR, &param) == 0 {
                        info!("Thread priority set to High");
                        Ok(())
                    } else {
                        warn!("Failed to set high thread priority, continuing with normal priority");
                        Ok(()) // Non-critical failure
                    }
                }
                ThreadPriority::Realtime => {
                    // Realtime priority for ultra-low latency
                    info!("Attempting to set realtime thread priority");
                    
                    // Create sched_param using mem::zeroed and then set priority
                    let mut param: libc::sched_param = std::mem::zeroed();
                    let priority_ptr = &mut param as *mut libc::sched_param as *mut i32;
                    *priority_ptr = 96;
                    
                    if libc::pthread_setschedparam(thread, libc::SCHED_RR, &param) == 0 {
                        info!("Thread priority set to Realtime");
                        Ok(())
                    } else {
                        warn!("Failed to set realtime thread priority, trying high priority instead");
                        // Fall back to high priority
                        let mut param: libc::sched_param = std::mem::zeroed();
                        let priority_ptr = &mut param as *mut libc::sched_param as *mut i32;
                        *priority_ptr = 63;
                        libc::pthread_setschedparam(thread, libc::SCHED_RR, &param);
                        Ok(()) // Non-critical failure
                    }
                }
            }
        }
    }
    
    /// Get recommended buffer size based on sample rate
    pub fn get_recommended_buffer_size(sample_rate: u32) -> u32 {
        // 10ms buffer for low latency
        sample_rate / 100
    }
    
    /// Get minimum buffer size for low-latency mode
    pub fn get_minimum_buffer_size(sample_rate: u32) -> u32 {
        // 5ms buffer for low-latency mode
        sample_rate / 200
    }
    
    /// Check if running on macOS
    pub fn is_macos() -> bool {
        cfg!(target_os = "macos")
    }
    
    /// Get current configuration
    pub fn get_config(&self) -> MacOSAudioConfig {
        MacOSAudioConfig {
            buffer_size: self.buffer_size,
            sample_rate: self.sample_rate,
            hardware_acceleration: self.hardware_acceleration,
            thread_priority: self.thread_priority,
            low_latency_mode: self.low_latency_mode,
        }
    }
}

/// macOS audio configuration
#[derive(Debug, Clone)]
pub struct MacOSAudioConfig {
    pub buffer_size: u32,
    pub sample_rate: u32,
    pub hardware_acceleration: bool,
    pub thread_priority: ThreadPriority,
    pub low_latency_mode: bool,
}

impl Default for MacOSAudioOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Request macOS audio permissions
pub fn request_audio_permissions() -> Result<(), AudioError> {
    #[cfg(target_os = "macos")]
    {
        info!("macOS audio permissions handled by AVFoundation");
        // macOS will automatically show permission dialog on first microphone access
        // The permission is controlled by Info.plist NSMicrophoneUsageDescription
        
        // Check if microphone access is granted
        if !check_audio_permissions() {
            warn!("Microphone access may be denied in System Preferences");
            return Err(AudioError::PermissionDenied);
        }
        
        Ok(())
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        Err(AudioError::UnsupportedPlatform)
    }
}

/// Check if audio permissions are granted
pub fn check_audio_permissions() -> bool {
    #[cfg(target_os = "macos")]
    {
        // On macOS, we can check the authorization status using AVFoundation
        // For now, we'll use a simple check that assumes permission is granted
        // In a full implementation, this would use Objective-C bindings to AVCaptureDevice
        
        // Note: The actual permission check requires Objective-C runtime
        // which would be implemented using the objc crate in production
        info!("Checking macOS microphone permissions");
        
        // Return true for now - the OS will show permission dialog on first use
        // A full implementation would use:
        // AVCaptureDevice.authorizationStatus(for: .audio)
        true
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

/// Open macOS System Preferences to microphone privacy settings
#[cfg(target_os = "macos")]
pub fn open_microphone_settings() -> Result<(), AudioError> {
    use std::process::Command;
    
    // Open System Preferences to Privacy & Security > Microphone
    let result = Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone")
        .spawn();
    
    match result {
        Ok(_) => {
            info!("Opened macOS microphone settings");
            Ok(())
        }
        Err(e) => {
            error!("Failed to open microphone settings: {}", e);
            Err(AudioError::PlatformError(format!("Failed to open settings: {}", e)))
        }
    }
}

/// Get macOS version information
#[cfg(target_os = "macos")]
pub fn get_macos_version() -> Result<(u32, u32, u32), AudioError> {
    use std::process::Command;
    
    let output = Command::new("sw_vers")
        .arg("-productVersion")
        .output()
        .map_err(|e| AudioError::PlatformError(format!("Failed to get macOS version: {}", e)))?;
    
    let version_str = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = version_str.trim().split('.').collect();
    
    if parts.len() >= 2 {
        let major = parts[0].parse().unwrap_or(0);
        let minor = parts[1].parse().unwrap_or(0);
        let patch = if parts.len() >= 3 { parts[2].parse().unwrap_or(0) } else { 0 };
        
        info!("macOS version: {}.{}.{}", major, minor, patch);
        Ok((major, minor, patch))
    } else {
        Err(AudioError::PlatformError("Failed to parse macOS version".to_string()))
    }
}

/// Check if running on Apple Silicon (M1/M2/M3)
#[cfg(target_os = "macos")]
pub fn is_apple_silicon() -> bool {
    use std::process::Command;
    
    let output = Command::new("uname")
        .arg("-m")
        .output();
    
    if let Ok(output) = output {
        let arch = String::from_utf8_lossy(&output.stdout);
        let is_arm = arch.trim() == "arm64";
        info!("Architecture: {} (Apple Silicon: {})", arch.trim(), is_arm);
        is_arm
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_macos_optimizer_creation() {
        let optimizer = MacOSAudioOptimizer::new();
        assert_eq!(optimizer.buffer_size, 512);
        assert_eq!(optimizer.sample_rate, 48000);
        assert!(optimizer.hardware_acceleration);
        assert_eq!(optimizer.thread_priority, ThreadPriority::High);
        assert!(!optimizer.low_latency_mode);
    }
    
    #[test]
    fn test_low_latency_optimizer() {
        let optimizer = MacOSAudioOptimizer::with_low_latency();
        assert_eq!(optimizer.buffer_size, 256);
        assert_eq!(optimizer.sample_rate, 48000);
        assert!(optimizer.hardware_acceleration);
        assert_eq!(optimizer.thread_priority, ThreadPriority::Realtime);
        assert!(optimizer.low_latency_mode);
    }
    
    #[test]
    fn test_buffer_size_calculation() {
        let size = MacOSAudioOptimizer::get_recommended_buffer_size(48000);
        assert_eq!(size, 480); // 10ms at 48kHz
        
        let size = MacOSAudioOptimizer::get_recommended_buffer_size(16000);
        assert_eq!(size, 160); // 10ms at 16kHz
        
        let min_size = MacOSAudioOptimizer::get_minimum_buffer_size(48000);
        assert_eq!(min_size, 240); // 5ms at 48kHz
    }
    
    #[test]
    fn test_set_buffer_size() {
        let mut optimizer = MacOSAudioOptimizer::new();
        optimizer.set_buffer_size(1024);
        assert_eq!(optimizer.buffer_size, 1024);
    }
    
    #[test]
    fn test_set_sample_rate() {
        let mut optimizer = MacOSAudioOptimizer::new();
        optimizer.set_sample_rate(44100);
        assert_eq!(optimizer.sample_rate, 44100);
    }
    
    #[test]
    fn test_thread_priority() {
        let mut optimizer = MacOSAudioOptimizer::new();
        optimizer.set_thread_priority(ThreadPriority::Realtime);
        assert_eq!(optimizer.thread_priority, ThreadPriority::Realtime);
    }
    
    #[test]
    fn test_get_config() {
        let optimizer = MacOSAudioOptimizer::new();
        let config = optimizer.get_config();
        assert_eq!(config.buffer_size, 512);
        assert_eq!(config.sample_rate, 48000);
        assert!(config.hardware_acceleration);
        assert!(!config.low_latency_mode);
    }
    
    #[test]
    fn test_enable_low_latency() {
        let mut optimizer = MacOSAudioOptimizer::new();
        
        #[cfg(target_os = "macos")]
        {
            let result = optimizer.enable_low_latency();
            assert!(result.is_ok());
            assert!(optimizer.low_latency_mode);
            assert_eq!(optimizer.buffer_size, 256);
            assert_eq!(optimizer.thread_priority, ThreadPriority::Realtime);
        }
    }
}
