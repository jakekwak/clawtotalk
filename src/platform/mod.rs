/// Platform-specific implementations and optimizations

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "android")]
pub mod android;

#[cfg(target_os = "ios")]
pub mod ios;

use crate::error::AudioError;

/// Request audio permissions for the current platform
pub fn request_audio_permissions() -> Result<(), AudioError> {
    #[cfg(target_os = "windows")]
    {
        windows::request_audio_permissions()
    }
    
    #[cfg(target_os = "macos")]
    {
        macos::request_audio_permissions()
    }
    
    #[cfg(target_os = "android")]
    {
        android::request_audio_permissions()
    }
    
    #[cfg(target_os = "ios")]
    {
        ios::request_audio_permissions()
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "android", target_os = "ios")))]
    {
        log::warn!("Platform not supported for permission requests");
        Ok(())
    }
}

/// Check if audio permissions are granted
pub fn check_audio_permissions() -> bool {
    #[cfg(target_os = "windows")]
    {
        windows::check_audio_permissions()
    }
    
    #[cfg(target_os = "macos")]
    {
        macos::check_audio_permissions()
    }
    
    #[cfg(target_os = "android")]
    {
        android::check_audio_permissions()
    }
    
    #[cfg(target_os = "ios")]
    {
        ios::check_audio_permissions()
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "android", target_os = "ios")))]
    {
        true
    }
}
