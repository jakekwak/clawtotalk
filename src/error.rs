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
    
    #[error("Server unavailable")]
    ServerUnavailable,
    
    #[error("Authentication failed")]
    AuthenticationFailed,
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    
    #[error("Request timeout")]
    Timeout,
    
    #[error("Connection refused")]
    ConnectionRefused,
    
    #[error("Service unavailable")]
    ServiceUnavailable,
}

impl ApiError {
    pub fn is_retryable(&self) -> bool {
        matches!(self, 
            ApiError::NetworkError(_) | 
            ApiError::ServerUnavailable |
            ApiError::Timeout |
            ApiError::RateLimitExceeded |
            ApiError::ServiceUnavailable |
            ApiError::ConnectionRefused
        )
    }
    
    pub fn retry_delay(&self) -> std::time::Duration {
        match self {
            ApiError::RateLimitExceeded => std::time::Duration::from_secs(60),
            ApiError::NetworkError(_) => std::time::Duration::from_secs(5),
            ApiError::ServerUnavailable => std::time::Duration::from_secs(30),
            ApiError::Timeout => std::time::Duration::from_secs(10),
            ApiError::ServiceUnavailable => std::time::Duration::from_secs(30),
            _ => std::time::Duration::from_secs(1),
        }
    }
    
    pub fn user_message(&self) -> String {
        match self {
            ApiError::NetworkError(msg) => format!("네트워크 오류: {}", msg),
            ApiError::ServerUnavailable => "서버에 연결할 수 없습니다. Tailscale 또는 서버 URL을 확인하세요.".to_string(),
            ApiError::AuthenticationFailed => "인증에 실패했습니다. 서버 설정을 확인하세요.".to_string(),
            ApiError::ConnectionRefused => "서버가 응답하지 않습니다. 서버가 실행 중인지 확인하세요.".to_string(),
            ApiError::Timeout => "요청 시간이 초과되었습니다.".to_string(),
            ApiError::RateLimitExceeded => "요청 한도를 초과했습니다. 잠시 후 다시 시도하세요.".to_string(),
            ApiError::ServiceUnavailable => "서비스를 사용할 수 없습니다.".to_string(),
            ApiError::InvalidResponse(msg) => format!("잘못된 응답: {}", msg),
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
