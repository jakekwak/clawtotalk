use crate::error::AudioError;
use log::{info, warn, error};

/// Windows-specific audio optimizations for WASAPI
pub struct WindowsAudioOptimizer {
    /// Whether exclusive mode is enabled
    exclusive_mode: bool,
    /// Buffer size in frames
    buffer_size: u32,
    /// Whether to use event-driven mode
    event_driven: bool,
    /// Audio thread priority
    thread_priority: ThreadPriority,
}

/// Audio thread priority levels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThreadPriority {
    Normal,
    High,
    TimeCritical,
}

impl WindowsAudioOptimizer {
    pub fn new() -> Self {
        Self {
            exclusive_mode: false,
            buffer_size: 480, // 10ms at 48kHz
            event_driven: true,
            thread_priority: ThreadPriority::High,
        }
    }
    
    /// Create optimizer with optimal settings for low latency
    pub fn with_low_latency() -> Self {
        Self {
            exclusive_mode: true,
            buffer_size: 240, // 5ms at 48kHz
            event_driven: true,
            thread_priority: ThreadPriority::TimeCritical,
        }
    }
    
    /// Enable WASAPI exclusive mode for lower latency
    pub fn enable_exclusive_mode(&mut self) -> Result<(), AudioError> {
        #[cfg(target_os = "windows")]
        {
            info!("Enabling WASAPI exclusive mode");
            self.exclusive_mode = true;
            Ok(())
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            warn!("Exclusive mode only available on Windows");
            Err(AudioError::UnsupportedPlatform)
        }
    }
    
    /// Set optimal buffer size for low latency
    pub fn set_buffer_size(&mut self, size: u32) {
        self.buffer_size = size;
        info!("WASAPI buffer size set to {} frames", size);
    }
    
    /// Enable event-driven mode for better efficiency
    pub fn enable_event_driven(&mut self) {
        self.event_driven = true;
        info!("WASAPI event-driven mode enabled");
    }
    
    /// Set audio thread priority
    pub fn set_thread_priority(&mut self, priority: ThreadPriority) {
        self.thread_priority = priority;
        info!("Audio thread priority set to {:?}", priority);
    }
    
    /// Apply optimizations to the current thread
    pub fn apply_thread_optimizations(&self) -> Result<(), AudioError> {
        #[cfg(target_os = "windows")]
        {
            self.set_current_thread_priority()?;
            self.enable_mmcss()?;
            Ok(())
        }
        
        #[cfg(not(target_os = "windows"))]
        {
            warn!("Thread optimizations only available on Windows");
            Ok(())
        }
    }
    
    /// Set current thread priority (Windows-specific)
    #[cfg(target_os = "windows")]
    fn set_current_thread_priority(&self) -> Result<(), AudioError> {
        use windows::Win32::System::Threading::{
            GetCurrentThread, SetThreadPriority,
            THREAD_PRIORITY_NORMAL, THREAD_PRIORITY_HIGHEST, THREAD_PRIORITY_TIME_CRITICAL,
        };
        
        unsafe {
            let handle = GetCurrentThread();
            let priority = match self.thread_priority {
                ThreadPriority::Normal => THREAD_PRIORITY_NORMAL,
                ThreadPriority::High => THREAD_PRIORITY_HIGHEST,
                ThreadPriority::TimeCritical => THREAD_PRIORITY_TIME_CRITICAL,
            };
            
            if SetThreadPriority(handle, priority).is_ok() {
                info!("Thread priority set successfully");
                Ok(())
            } else {
                error!("Failed to set thread priority");
                Err(AudioError::PlatformError("Failed to set thread priority".to_string()))
            }
        }
    }
    
    /// Enable MMCSS (Multimedia Class Scheduler Service) for audio threads
    #[cfg(target_os = "windows")]
    fn enable_mmcss(&self) -> Result<(), AudioError> {
        use windows::Win32::Media::Audio::{AvSetMmThreadCharacteristicsW, AvRevertMmThreadCharacteristics};
        use windows::core::PCWSTR;
        
        unsafe {
            let task_name = "Pro Audio\0".encode_utf16().collect::<Vec<u16>>();
            let mut task_index = 0u32;
            
            let handle = AvSetMmThreadCharacteristicsW(
                PCWSTR(task_name.as_ptr()),
                &mut task_index,
            );
            
            if !handle.is_invalid() {
                info!("MMCSS enabled for audio thread");
                // Note: In production, store handle and call AvRevertMmThreadCharacteristics on cleanup
                Ok(())
            } else {
                warn!("Failed to enable MMCSS, continuing without it");
                Ok(()) // Non-critical failure
            }
        }
    }
    
    /// Get recommended buffer size based on sample rate
    pub fn get_recommended_buffer_size(sample_rate: u32) -> u32 {
        // 10ms buffer for low latency
        sample_rate / 100
    }
    
    /// Get minimum buffer size for exclusive mode
    pub fn get_minimum_buffer_size(sample_rate: u32) -> u32 {
        // 5ms buffer for exclusive mode
        sample_rate / 200
    }
    
    /// Check if running on Windows
    pub fn is_windows() -> bool {
        cfg!(target_os = "windows")
    }
    
    /// Get current configuration
    pub fn get_config(&self) -> WindowsAudioConfig {
        WindowsAudioConfig {
            exclusive_mode: self.exclusive_mode,
            buffer_size: self.buffer_size,
            event_driven: self.event_driven,
            thread_priority: self.thread_priority,
        }
    }
}

/// Windows audio configuration
#[derive(Debug, Clone)]
pub struct WindowsAudioConfig {
    pub exclusive_mode: bool,
    pub buffer_size: u32,
    pub event_driven: bool,
    pub thread_priority: ThreadPriority,
}

impl Default for WindowsAudioOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Request Windows audio permissions
pub fn request_audio_permissions() -> Result<(), AudioError> {
    #[cfg(target_os = "windows")]
    {
        info!("Windows audio permissions are handled by the OS");
        // Windows 10+ handles microphone permissions through Settings
        // The app will automatically trigger permission dialog on first use
        
        // Check if microphone access is enabled in Windows Settings
        if !check_microphone_privacy_settings() {
            warn!("Microphone access may be disabled in Windows Privacy Settings");
            return Err(AudioError::PermissionDenied);
        }
        
        Ok(())
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Err(AudioError::UnsupportedPlatform)
    }
}

/// Check Windows microphone privacy settings
#[cfg(target_os = "windows")]
fn check_microphone_privacy_settings() -> bool {
    use windows::Win32::System::Registry::{RegOpenKeyExW, RegQueryValueExW, RegCloseKey, HKEY_CURRENT_USER, KEY_READ};
    use windows::core::PCWSTR;
    
    unsafe {
        let subkey = "Software\\Microsoft\\Windows\\CurrentVersion\\CapabilityAccessManager\\ConsentStore\\microphone\0"
            .encode_utf16()
            .collect::<Vec<u16>>();
        
        let mut hkey = std::mem::zeroed();
        let result = RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            0,
            KEY_READ,
            &mut hkey,
        );
        
        if result.is_err() {
            warn!("Could not check microphone privacy settings");
            return true; // Assume allowed if we can't check
        }
        
        let value_name = "Value\0".encode_utf16().collect::<Vec<u16>>();
        let mut data = vec![0u8; 256];
        let mut data_size = data.len() as u32;
        
        let result = RegQueryValueExW(
            hkey,
            PCWSTR(value_name.as_ptr()),
            None,
            None,
            Some(data.as_mut_ptr()),
            Some(&mut data_size),
        );
        
        RegCloseKey(hkey);
        
        if result.is_ok() {
            // Check if value is "Allow"
            let value = String::from_utf8_lossy(&data[..data_size as usize]);
            info!("Microphone privacy setting: {}", value);
            value.contains("Allow")
        } else {
            true // Assume allowed if we can't read the value
        }
    }
}

/// Check if audio permissions are granted
pub fn check_audio_permissions() -> bool {
    #[cfg(target_os = "windows")]
    {
        check_microphone_privacy_settings()
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

/// Open Windows microphone privacy settings
#[cfg(target_os = "windows")]
pub fn open_microphone_settings() -> Result<(), AudioError> {
    use std::process::Command;
    
    // Open Windows Settings to microphone privacy page
    let result = Command::new("cmd")
        .args(&["/C", "start", "ms-settings:privacy-microphone"])
        .spawn();
    
    match result {
        Ok(_) => {
            info!("Opened Windows microphone settings");
            Ok(())
        }
        Err(e) => {
            error!("Failed to open microphone settings: {}", e);
            Err(AudioError::PlatformError(format!("Failed to open settings: {}", e)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_windows_optimizer_creation() {
        let optimizer = WindowsAudioOptimizer::new();
        assert_eq!(optimizer.buffer_size, 480);
        assert!(!optimizer.exclusive_mode);
        assert!(optimizer.event_driven);
        assert_eq!(optimizer.thread_priority, ThreadPriority::High);
    }
    
    #[test]
    fn test_low_latency_optimizer() {
        let optimizer = WindowsAudioOptimizer::with_low_latency();
        assert_eq!(optimizer.buffer_size, 240);
        assert!(optimizer.exclusive_mode);
        assert!(optimizer.event_driven);
        assert_eq!(optimizer.thread_priority, ThreadPriority::TimeCritical);
    }
    
    #[test]
    fn test_buffer_size_calculation() {
        let size = WindowsAudioOptimizer::get_recommended_buffer_size(48000);
        assert_eq!(size, 480); // 10ms at 48kHz
        
        let size = WindowsAudioOptimizer::get_recommended_buffer_size(16000);
        assert_eq!(size, 160); // 10ms at 16kHz
        
        let min_size = WindowsAudioOptimizer::get_minimum_buffer_size(48000);
        assert_eq!(min_size, 240); // 5ms at 48kHz
    }
    
    #[test]
    fn test_set_buffer_size() {
        let mut optimizer = WindowsAudioOptimizer::new();
        optimizer.set_buffer_size(960);
        assert_eq!(optimizer.buffer_size, 960);
    }
    
    #[test]
    fn test_thread_priority() {
        let mut optimizer = WindowsAudioOptimizer::new();
        optimizer.set_thread_priority(ThreadPriority::TimeCritical);
        assert_eq!(optimizer.thread_priority, ThreadPriority::TimeCritical);
    }
    
    #[test]
    fn test_get_config() {
        let optimizer = WindowsAudioOptimizer::new();
        let config = optimizer.get_config();
        assert_eq!(config.buffer_size, 480);
        assert!(!config.exclusive_mode);
        assert!(config.event_driven);
    }
}
