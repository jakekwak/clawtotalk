use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Message in the conversation
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub id: Uuid,
    pub content: String,
    pub message_type: MessageType,
    pub timestamp: DateTime<Utc>,
    pub audio_data: Option<Vec<u8>>,
}

impl Message {
    pub fn new(content: String, message_type: MessageType) -> Self {
        Self {
            id: Uuid::new_v4(),
            content,
            message_type,
            timestamp: Utc::now(),
            audio_data: None,
        }
    }
    
    pub fn with_audio(mut self, audio_data: Vec<u8>) -> Self {
        self.audio_data = Some(audio_data);
        self
    }
}

/// Type of message
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    User,
    Assistant,
    System,
    Error,
}

/// Recording mode
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum RecordingMode {
    Hold,
    Toggle,
    Auto,
}

impl Default for RecordingMode {
    fn default() -> Self {
        RecordingMode::Hold
    }
}

/// Voice Activity Detection settings
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VadSettings {
    pub threshold: f32,
    pub window_size: usize,
    pub silence_duration_ms: u64,
}

impl Default for VadSettings {
    fn default() -> Self {
        Self {
            threshold: 0.02,
            window_size: 1024,
            silence_duration_ms: 1000,
        }
    }
}

/// Audio settings
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AudioSettings {
    pub sample_rate: u32,
    pub channels: u16,
    pub buffer_size: usize,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            sample_rate: 16000,
            channels: 1,
            buffer_size: 4096,
        }
    }
}

/// API keys configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiKeys {
    pub openai_key: Option<String>,
    pub claude_key: Option<String>,
    pub elevenlabs_key: Option<String>,
}

impl Default for ApiKeys {
    fn default() -> Self {
        Self {
            openai_key: None,
            claude_key: None,
            elevenlabs_key: None,
        }
    }
}

/// Server configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub server_url: String,
    pub connection_type: ConnectionType,
    pub auth_token: Option<String>,
    pub timeout_seconds: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            server_url: "http://localhost:8080".to_string(),
            connection_type: ConnectionType::LocalNetwork,
            auth_token: None,
            timeout_seconds: 30,
        }
    }
}

/// Connection type for server
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ConnectionType {
    Tailscale,
    PublicUrl,
    LocalNetwork,
}

/// Application settings
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub server_config: ServerConfig,
    pub recording_mode: RecordingMode,
    pub vad_settings: VadSettings,
    pub audio_settings: AudioSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            server_config: ServerConfig::default(),
            recording_mode: RecordingMode::default(),
            vad_settings: VadSettings::default(),
            audio_settings: AudioSettings::default(),
        }
    }
}

/// Application status
#[derive(Clone, Debug, PartialEq)]
pub enum AppStatus {
    Idle,
    Recording,
    Processing,
    Speaking,
    Error(String),
}

/// Audio level information
#[derive(Clone, Debug)]
pub struct AudioLevel {
    pub current: f32,
    pub peak: f32,
    pub average: f32,
}

impl Default for AudioLevel {
    fn default() -> Self {
        Self {
            current: 0.0,
            peak: 0.0,
            average: 0.0,
        }
    }
}

/// STT API request
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SttRequest {
    pub language: Option<String>,
}

/// STT API response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SttResponse {
    pub transcript: String,
    pub confidence: f32,
    pub language: Option<String>,
}

/// Chat API request
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub conversation_history: Vec<ChatMessage>,
}

/// Chat message for API
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Chat API response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    pub response: String,
    pub model: Option<String>,
}

/// TTS API request
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TtsRequest {
    pub text: String,
    pub voice_id: Option<String>,
}

/// TTS API response (binary audio data)
#[derive(Clone, Debug)]
pub struct TtsResponse {
    pub audio_data: Vec<u8>,
    pub format: String,
}

/// Health check response
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub services: ServiceStatus,
    pub version: Option<String>,
}

/// Service status
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub stt: String,
    pub chat: String,
    pub tts: String,
}

/// Connection status
#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
    Error(String),
}
