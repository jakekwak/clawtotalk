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

/// Application settings
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub api_keys: ApiKeys,
    pub recording_mode: RecordingMode,
    pub vad_settings: VadSettings,
    pub audio_settings: AudioSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            api_keys: ApiKeys::default(),
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
