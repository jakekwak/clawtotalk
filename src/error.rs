use thiserror::Error;

/// Audio-related errors
#[derive(Debug, Clone, Error)]
pub enum AudioError {
    #[error("Audio device not found")]
    DeviceNotFound,
    
    #[error("Microphone permission denied")]
    PermissionDenied,
    
    #[error("Recording failed: {0}")]
    RecordingFailed(String),
    
    #[error("Playback failed: {0}")]
    PlaybackFailed(String),
    
    #[error("Unsupported audio format")]
    UnsupportedFormat,
}

impl AudioError {
    pub fn user_message(&self) -> String {
        match self {
            AudioError::DeviceNotFound => "오디오 장치를 찾을 수 없습니다".to_string(),
            AudioError::PermissionDenied => "마이크 권한이 필요합니다".to_string(),
            AudioError::RecordingFailed(msg) => format!("녹음 실패: {}", msg),
            AudioError::PlaybackFailed(msg) => format!("재생 실패: {}", msg),
            AudioError::UnsupportedFormat => "지원되지 않는 오디오 형식입니다".to_string(),
        }
    }
    
    pub fn recovery_action(&self) -> RecoveryAction {
        match self {
            AudioError::PermissionDenied => RecoveryAction::RequestPermission,
            AudioError::DeviceNotFound => RecoveryAction::ShowDeviceSettings,
            _ => RecoveryAction::Retry,
        }
    }
}

/// API-related errors
#[derive(Debug, Clone, Error)]
pub enum ApiError {
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Authentication failed")]
    AuthenticationFailed,
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    
    #[error("Service unavailable")]
    ServiceUnavailable,
}

impl ApiError {
    pub fn is_retryable(&self) -> bool {
        matches!(self, 
            ApiError::NetworkError(_) | 
            ApiError::RateLimitExceeded | 
            ApiError::ServiceUnavailable
        )
    }
    
    pub fn retry_delay(&self) -> std::time::Duration {
        match self {
            ApiError::RateLimitExceeded => std::time::Duration::from_secs(60),
            ApiError::NetworkError(_) => std::time::Duration::from_secs(5),
            ApiError::ServiceUnavailable => std::time::Duration::from_secs(30),
            _ => std::time::Duration::from_secs(1),
        }
    }
}

/// Application-level errors
#[derive(Debug, Clone, Error)]
pub enum AppError {
    #[error("Audio error: {0}")]
    Audio(#[from] AudioError),
    
    #[error("API error: {0}")]
    Api(#[from] ApiError),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Recovery actions for errors
#[derive(Debug, Clone)]
pub enum RecoveryAction {
    RequestPermission,
    ShowDeviceSettings,
    Retry,
    ShowSettings,
    None,
}
