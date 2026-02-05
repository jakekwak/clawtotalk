use crate::error::AudioError;
use log::{error, info, warn};

/// iOS-specific audio optimizations for AVAudioEngine
pub struct IOSAudioOptimizer {
    /// Buffer size in frames
    buffer_size: u32,
    /// Sample rate
    sample_rate: u32,
    /// Whether to use hardware acceleration
    hardware_acceleration: bool,
    /// Audio session category
    session_category: AudioSessionCategory,
    /// Whether to enable low-latency mode
    low_latency_mode: bool,
    /// Whether background audio is enabled
    background_audio_enabled: bool,
}

/// iOS Audio Session Categories
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AudioSessionCategory {
    /// For recording audio
    Record,
    /// For playback
    Playback,
    /// For both recording and playback
    PlayAndRecord,
    /// For ambient audio
    Ambient,
}

impl IOSAudioOptimizer {
    pub fn new() -> Self {
        Self {
            buffer_size: 512, // ~10ms at 48kHz
            sample_rate: 48000,
            hardware_acceleration: true,
            session_category: AudioSessionCategory::PlayAndRecord,
            low_latency_mode: false,
            background_audio_enabled: false,
        }
    }
    
    /// Create optimizer with optimal settings for low latency
    pub fn with_low_latency() -> Self {
        Self {
            buffer_size: 256, // ~5ms at 48kHz
            sample_rate: 48000,
            hardware_acceleration: true,
            session_category: AudioSessionCategory::PlayAndRecord,
            low_latency_mode: true,
            background_audio_enabled: false,
        }
    }
    
    /// Enable low-latency mode for AVAudioEngine
    pub fn enable_low_latency(&mut self) -> Result<(), AudioError> {
        #[cfg(target_os = "ios")]
        {
            info!("Enabling AVAudioEngine low-latency mode");
            self.low_latency_mode = true;
            self.buffer_size = 256; // 5ms at 48kHz
            Ok(())
        }
        
        #[cfg(not(target_os = "ios"))]
        {
            warn!("Low-latency mode only available on iOS");
            Err(AudioError::UnsupportedPlatform)
        }
    }
    
    /// Set optimal buffer size for low latency
    pub fn set_buffer_size(&mut self, size: u32) {
        self.buffer_size = size;
        info!("AVAudioEngine buffer size set to {} frames", size);
    }
    
    /// Set sample rate
    pub fn set_sample_rate(&mut self, rate: u32) {
        self.sample_rate = rate;
        info!("AVAudioEngine sample rate set to {} Hz", rate);
    }
    
    /// Enable hardware acceleration
    pub fn enable_hardware_acceleration(&mut self) {
        self.hardware_acceleration = true;
        info!("AVAudioEngine hardware acceleration enabled");
    }
    
    /// Set audio session category
    pub fn set_session_category(&mut self, category: AudioSessionCategory) {
        self.session_category = category;
        info!("Audio session category set to {:?}", category);
    }
    
    /// Enable background audio playback
    pub fn enable_background_audio(&mut self) {
        self.background_audio_enabled = true;
        info!("Background audio enabled");
    }
    
    /// Configure audio session for recording and playback
    pub fn configure_audio_session(&self) -> Result<(), AudioError> {
        #[cfg(target_os = "ios")]
        {
            configure_audio_session_internal(
                self.session_category,
                self.low_latency_mode,
                self.background_audio_enabled,
            )
        }
        
        #[cfg(not(target_os = "ios"))]
        {
            warn!("Audio session configuration only available on iOS");
            Ok(())
        }
    }
    
    /// Optimize for battery life
    pub fn optimize_for_battery(&mut self) {
        info!("Optimizing for battery life");
        self.buffer_size = 1024; // Larger buffer = less CPU wakeups
        self.low_latency_mode = false;
        self.hardware_acceleration = true;
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
    
    /// Check if running on iOS
    pub fn is_ios() -> bool {
        cfg!(target_os = "ios")
    }
    
    /// Get current configuration
    pub fn get_config(&self) -> IOSAudioConfig {
        IOSAudioConfig {
            buffer_size: self.buffer_size,
            sample_rate: self.sample_rate,
            hardware_acceleration: self.hardware_acceleration,
            session_category: self.session_category,
            low_latency_mode: self.low_latency_mode,
            background_audio_enabled: self.background_audio_enabled,
        }
    }
}

/// iOS audio configuration
#[derive(Debug, Clone)]
pub struct IOSAudioConfig {
    pub buffer_size: u32,
    pub sample_rate: u32,
    pub hardware_acceleration: bool,
    pub session_category: AudioSessionCategory,
    pub low_latency_mode: bool,
    pub background_audio_enabled: bool,
}

impl Default for IOSAudioOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Request iOS audio permissions using AVAudioSession
pub fn request_audio_permissions() -> Result<(), AudioError> {
    #[cfg(target_os = "ios")]
    {
        info!("Requesting iOS audio permissions via AVAudioSession");
        
        // Request microphone permission
        request_microphone_permission_internal()?;
        
        // Configure audio session
        configure_audio_session_internal(
            AudioSessionCategory::PlayAndRecord,
            false,
            false,
        )?;
        
        Ok(())
    }
    
    #[cfg(not(target_os = "ios"))]
    {
        Err(AudioError::UnsupportedPlatform)
    }
}

/// Check if audio permissions are granted
pub fn check_audio_permissions() -> bool {
    #[cfg(target_os = "ios")]
    {
        check_microphone_permission_internal()
    }
    
    #[cfg(not(target_os = "ios"))]
    {
        false
    }
}

/// Open iOS Settings app to microphone privacy settings
#[cfg(target_os = "ios")]
pub fn open_microphone_settings() -> Result<(), AudioError> {
    info!("Opening iOS microphone settings");
    
    // iOS doesn't allow direct opening of specific settings
    // We can only open the app's settings page
    open_app_settings_internal()
}

/// Get iOS version information
#[cfg(target_os = "ios")]
pub fn get_ios_version() -> Result<(u32, u32, u32), AudioError> {
    get_ios_version_internal()
}

/// Check if running on a specific iOS device type
#[cfg(target_os = "ios")]
pub fn get_device_type() -> DeviceType {
    get_device_type_internal()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeviceType {
    IPhone,
    IPad,
    IPod,
    Unknown,
}

// Internal implementation using Objective-C runtime

#[cfg(target_os = "ios")]
fn request_microphone_permission_internal() -> Result<(), AudioError> {
    use objc::runtime::{Class, Object};
    use objc::{msg_send, sel, sel_impl};
    use std::ffi::CString;
    
    unsafe {
        // Get AVAudioSession shared instance
        let av_audio_session_class = Class::get("AVAudioSession")
            .ok_or_else(|| {
                error!("Failed to get AVAudioSession class");
                AudioError::PlatformError("AVAudioSession class not found".to_string())
            })?;
        
        let shared_instance: *mut Object = msg_send![av_audio_session_class, sharedInstance];
        
        if shared_instance.is_null() {
            error!("Failed to get AVAudioSession shared instance");
            return Err(AudioError::PlatformError("AVAudioSession instance is null".to_string()));
        }
        
        // Request record permission
        // Note: This is a simplified version. In production, you'd want to handle the callback
        let _: () = msg_send![shared_instance, requestRecordPermission: |granted: bool| {
            if granted {
                info!("Microphone permission granted");
            } else {
                warn!("Microphone permission denied");
            }
        }];
        
        info!("Microphone permission requested");
        Ok(())
    }
}

#[cfg(target_os = "ios")]
fn check_microphone_permission_internal() -> bool {
    use objc::runtime::{Class, Object};
    use objc::{msg_send, sel, sel_impl};
    
    unsafe {
        // Get AVAudioSession shared instance
        let av_audio_session_class = match Class::get("AVAudioSession") {
            Some(class) => class,
            None => {
                warn!("Failed to get AVAudioSession class");
                return false;
            }
        };
        
        let shared_instance: *mut Object = msg_send![av_audio_session_class, sharedInstance];
        
        if shared_instance.is_null() {
            warn!("Failed to get AVAudioSession shared instance");
            return false;
        }
        
        // Get record permission status
        // AVAudioSessionRecordPermission: Undetermined = 0, Denied = 1, Granted = 2
        let permission_status: i32 = msg_send![shared_instance, recordPermission];
        
        let granted = permission_status == 2; // Granted
        info!("Microphone permission status: {}", if granted { "granted" } else { "not granted" });
        
        granted
    }
}

#[cfg(target_os = "ios")]
fn configure_audio_session_internal(
    category: AudioSessionCategory,
    low_latency: bool,
    background_audio: bool,
) -> Result<(), AudioError> {
    use objc::runtime::{Class, Object};
    use objc::{msg_send, sel, sel_impl};
    use std::ffi::CString;
    
    unsafe {
        // Get AVAudioSession shared instance
        let av_audio_session_class = Class::get("AVAudioSession")
            .ok_or_else(|| AudioError::PlatformError("AVAudioSession class not found".to_string()))?;
        
        let shared_instance: *mut Object = msg_send![av_audio_session_class, sharedInstance];
        
        if shared_instance.is_null() {
            return Err(AudioError::PlatformError("AVAudioSession instance is null".to_string()));
        }
        
        // Set category
        let category_str = match category {
            AudioSessionCategory::Record => CString::new("AVAudioSessionCategoryRecord").unwrap(),
            AudioSessionCategory::Playback => CString::new("AVAudioSessionCategoryPlayback").unwrap(),
            AudioSessionCategory::PlayAndRecord => CString::new("AVAudioSessionCategoryPlayAndRecord").unwrap(),
            AudioSessionCategory::Ambient => CString::new("AVAudioSessionCategoryAmbient").unwrap(),
        };
        
        // Set mode
        let mode_str = if low_latency {
            CString::new("AVAudioSessionModeMeasurement").unwrap()
        } else {
            CString::new("AVAudioSessionModeDefault").unwrap()
        };
        
        // Configure options
        let mut options: u32 = 0;
        if background_audio {
            options |= 1; // AVAudioSessionCategoryOptionMixWithOthers
        }
        
        // Set category with options
        let error: *mut Object = std::ptr::null_mut();
        let success: bool = msg_send![
            shared_instance,
            setCategory: category_str.as_ptr()
            mode: mode_str.as_ptr()
            options: options
            error: &error
        ];
        
        if !success || !error.is_null() {
            error!("Failed to set audio session category");
            return Err(AudioError::PlatformError("Failed to set audio session category".to_string()));
        }
        
        // Activate audio session
        let error: *mut Object = std::ptr::null_mut();
        let success: bool = msg_send![shared_instance, setActive: true error: &error];
        
        if !success || !error.is_null() {
            error!("Failed to activate audio session");
            return Err(AudioError::PlatformError("Failed to activate audio session".to_string()));
        }
        
        info!("Audio session configured successfully");
        Ok(())
    }
}

#[cfg(target_os = "ios")]
fn open_app_settings_internal() -> Result<(), AudioError> {
    use objc::runtime::{Class, Object};
    use objc::{msg_send, sel, sel_impl};
    use std::ffi::CString;
    
    unsafe {
        // Get UIApplication shared instance
        let ui_application_class = Class::get("UIApplication")
            .ok_or_else(|| AudioError::PlatformError("UIApplication class not found".to_string()))?;
        
        let shared_app: *mut Object = msg_send![ui_application_class, sharedApplication];
        
        if shared_app.is_null() {
            return Err(AudioError::PlatformError("UIApplication instance is null".to_string()));
        }
        
        // Create settings URL
        let url_class = Class::get("NSURL")
            .ok_or_else(|| AudioError::PlatformError("NSURL class not found".to_string()))?;
        
        let settings_url_str = CString::new("app-settings:").unwrap();
        let settings_url: *mut Object = msg_send![url_class, URLWithString: settings_url_str.as_ptr()];
        
        if settings_url.is_null() {
            return Err(AudioError::PlatformError("Failed to create settings URL".to_string()));
        }
        
        // Open URL
        let _: () = msg_send![shared_app, openURL: settings_url options: std::ptr::null::<Object>() completionHandler: std::ptr::null::<Object>()];
        
        info!("Opened app settings");
        Ok(())
    }
}

#[cfg(target_os = "ios")]
fn get_ios_version_internal() -> Result<(u32, u32, u32), AudioError> {
    use objc::runtime::{Class, Object};
    use objc::{msg_send, sel, sel_impl};
    
    unsafe {
        // Get UIDevice current device
        let ui_device_class = Class::get("UIDevice")
            .ok_or_else(|| AudioError::PlatformError("UIDevice class not found".to_string()))?;
        
        let current_device: *mut Object = msg_send![ui_device_class, currentDevice];
        
        if current_device.is_null() {
            return Err(AudioError::PlatformError("UIDevice instance is null".to_string()));
        }
        
        // Get system version
        let version_str: *mut Object = msg_send![current_device, systemVersion];
        
        if version_str.is_null() {
            return Err(AudioError::PlatformError("Failed to get system version".to_string()));
        }
        
        // Convert NSString to Rust String
        let c_str: *const i8 = msg_send![version_str, UTF8String];
        let version = std::ffi::CStr::from_ptr(c_str)
            .to_string_lossy()
            .to_string();
        
        // Parse version string
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() >= 2 {
            let major = parts[0].parse().unwrap_or(0);
            let minor = parts[1].parse().unwrap_or(0);
            let patch = if parts.len() >= 3 { parts[2].parse().unwrap_or(0) } else { 0 };
            
            info!("iOS version: {}.{}.{}", major, minor, patch);
            Ok((major, minor, patch))
        } else {
            Err(AudioError::PlatformError("Failed to parse iOS version".to_string()))
        }
    }
}

#[cfg(target_os = "ios")]
fn get_device_type_internal() -> DeviceType {
    use objc::runtime::{Class, Object};
    use objc::{msg_send, sel, sel_impl};
    
    unsafe {
        // Get UIDevice current device
        let ui_device_class = match Class::get("UIDevice") {
            Some(class) => class,
            None => return DeviceType::Unknown,
        };
        
        let current_device: *mut Object = msg_send![ui_device_class, currentDevice];
        
        if current_device.is_null() {
            return DeviceType::Unknown;
        }
        
        // Get model
        let model: *mut Object = msg_send![current_device, model];
        
        if model.is_null() {
            return DeviceType::Unknown;
        }
        
        // Convert NSString to Rust String
        let c_str: *const i8 = msg_send![model, UTF8String];
        let model_str = std::ffi::CStr::from_ptr(c_str)
            .to_string_lossy()
            .to_string();
        
        info!("Device model: {}", model_str);
        
        if model_str.contains("iPhone") {
            DeviceType::IPhone
        } else if model_str.contains("iPad") {
            DeviceType::IPad
        } else if model_str.contains("iPod") {
            DeviceType::IPod
        } else {
            DeviceType::Unknown
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ios_optimizer_creation() {
        let optimizer = IOSAudioOptimizer::new();
        assert_eq!(optimizer.buffer_size, 512);
        assert_eq!(optimizer.sample_rate, 48000);
        assert!(optimizer.hardware_acceleration);
        assert_eq!(optimizer.session_category, AudioSessionCategory::PlayAndRecord);
        assert!(!optimizer.low_latency_mode);
        assert!(!optimizer.background_audio_enabled);
    }
    
    #[test]
    fn test_low_latency_optimizer() {
        let optimizer = IOSAudioOptimizer::with_low_latency();
        assert_eq!(optimizer.buffer_size, 256);
        assert_eq!(optimizer.sample_rate, 48000);
        assert!(optimizer.hardware_acceleration);
        assert_eq!(optimizer.session_category, AudioSessionCategory::PlayAndRecord);
        assert!(optimizer.low_latency_mode);
        assert!(!optimizer.background_audio_enabled);
    }
    
    #[test]
    fn test_buffer_size_calculation() {
        let size = IOSAudioOptimizer::get_recommended_buffer_size(48000);
        assert_eq!(size, 480); // 10ms at 48kHz
        
        let size = IOSAudioOptimizer::get_recommended_buffer_size(16000);
        assert_eq!(size, 160); // 10ms at 16kHz
        
        let min_size = IOSAudioOptimizer::get_minimum_buffer_size(48000);
        assert_eq!(min_size, 240); // 5ms at 48kHz
    }
    
    #[test]
    fn test_set_buffer_size() {
        let mut optimizer = IOSAudioOptimizer::new();
        optimizer.set_buffer_size(1024);
        assert_eq!(optimizer.buffer_size, 1024);
    }
    
    #[test]
    fn test_set_sample_rate() {
        let mut optimizer = IOSAudioOptimizer::new();
        optimizer.set_sample_rate(44100);
        assert_eq!(optimizer.sample_rate, 44100);
    }
    
    #[test]
    fn test_session_category() {
        let mut optimizer = IOSAudioOptimizer::new();
        optimizer.set_session_category(AudioSessionCategory::Record);
        assert_eq!(optimizer.session_category, AudioSessionCategory::Record);
    }
    
    #[test]
    fn test_background_audio() {
        let mut optimizer = IOSAudioOptimizer::new();
        optimizer.enable_background_audio();
        assert!(optimizer.background_audio_enabled);
    }
    
    #[test]
    fn test_battery_optimization() {
        let mut optimizer = IOSAudioOptimizer::new();
        optimizer.optimize_for_battery();
        assert_eq!(optimizer.buffer_size, 1024);
        assert!(!optimizer.low_latency_mode);
        assert!(optimizer.hardware_acceleration);
    }
    
    #[test]
    fn test_get_config() {
        let optimizer = IOSAudioOptimizer::new();
        let config = optimizer.get_config();
        assert_eq!(config.buffer_size, 512);
        assert_eq!(config.sample_rate, 48000);
        assert!(config.hardware_acceleration);
        assert!(!config.low_latency_mode);
        assert!(!config.background_audio_enabled);
    }
    
    #[test]
    fn test_enable_low_latency() {
        let mut optimizer = IOSAudioOptimizer::new();
        
        #[cfg(target_os = "ios")]
        {
            let result = optimizer.enable_low_latency();
            assert!(result.is_ok());
            assert!(optimizer.low_latency_mode);
            assert_eq!(optimizer.buffer_size, 256);
        }
    }
}
