use async_trait::async_trait;
use crate::error::ApiError;

/// Speech-to-text trait
#[async_trait]
pub trait SpeechToText: Send + Sync {
    /// Transcribe audio data to text
    async fn transcribe(&self, audio: &[u8]) -> Result<String, ApiError>;
}

/// AI assistant trait
#[async_trait]
pub trait AiAssistant: Send + Sync {
    /// Generate AI response from prompt
    async fn generate_response(&self, prompt: &str) -> Result<String, ApiError>;
}

/// Text-to-speech trait
#[async_trait]
pub trait TextToSpeech: Send + Sync {
    /// Synthesize speech from text
    async fn synthesize(&self, text: &str) -> Result<Vec<u8>, ApiError>;
}

/// API client aggregator
pub struct ApiClient {
    whisper_client: Box<dyn SpeechToText>,
    ai_client: Box<dyn AiAssistant>,
    tts_client: Box<dyn TextToSpeech>,
}

impl ApiClient {
    pub fn new(
        whisper_client: Box<dyn SpeechToText>,
        ai_client: Box<dyn AiAssistant>,
        tts_client: Box<dyn TextToSpeech>,
    ) -> Self {
        Self {
            whisper_client,
            ai_client,
            tts_client,
        }
    }
    
    pub async fn transcribe(&self, audio: &[u8]) -> Result<String, ApiError> {
        self.whisper_client.transcribe(audio).await
    }
    
    pub async fn generate_response(&self, prompt: &str) -> Result<String, ApiError> {
        self.ai_client.generate_response(prompt).await
    }
    
    pub async fn synthesize(&self, text: &str) -> Result<Vec<u8>, ApiError> {
        self.tts_client.synthesize(text).await
    }
}

/// Placeholder Whisper client
pub struct WhisperClient {
    api_key: Option<String>,
}

impl WhisperClient {
    pub fn new(api_key: Option<String>) -> Self {
        Self { api_key }
    }
}

#[async_trait]
impl SpeechToText for WhisperClient {
    async fn transcribe(&self, _audio: &[u8]) -> Result<String, ApiError> {
        // Placeholder - will be implemented in task 6
        log::info!("Transcribing audio (placeholder)");
        Ok("Transcribed text placeholder".to_string())
    }
}

/// Placeholder AI client
pub struct ClaudeClient {
    api_key: Option<String>,
}

impl ClaudeClient {
    pub fn new(api_key: Option<String>) -> Self {
        Self { api_key }
    }
}

#[async_trait]
impl AiAssistant for ClaudeClient {
    async fn generate_response(&self, _prompt: &str) -> Result<String, ApiError> {
        // Placeholder - will be implemented in task 6
        log::info!("Generating AI response (placeholder)");
        Ok("AI response placeholder".to_string())
    }
}

/// Placeholder TTS client
pub struct ElevenLabsClient {
    api_key: Option<String>,
}

impl ElevenLabsClient {
    pub fn new(api_key: Option<String>) -> Self {
        Self { api_key }
    }
}

#[async_trait]
impl TextToSpeech for ElevenLabsClient {
    async fn synthesize(&self, _text: &str) -> Result<Vec<u8>, ApiError> {
        // Placeholder - will be implemented in task 6
        log::info!("Synthesizing speech (placeholder)");
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_api_client_creation() {
        let whisper = Box::new(WhisperClient::new(None));
        let claude = Box::new(ClaudeClient::new(None));
        let elevenlabs = Box::new(ElevenLabsClient::new(None));
        
        let client = ApiClient::new(whisper, claude, elevenlabs);
        
        // Test placeholder methods
        let transcript = client.transcribe(&[]).await;
        assert!(transcript.is_ok());
        
        let response = client.generate_response("test").await;
        assert!(response.is_ok());
        
        let audio = client.synthesize("test").await;
        assert!(audio.is_ok());
    }
}
