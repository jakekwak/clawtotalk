use crate::error::ApiError;
use crate::models::{
    ServerConfig, SttResponse, ChatRequest, ChatResponse, 
    TtsRequest, HealthResponse, ChatMessage
};
use reqwest::{Client, multipart};
use std::time::Duration;

/// Server client for HTTP REST API communication
#[derive(Clone)]
pub struct ServerClient {
    base_url: String,
    http_client: Client,
    auth_token: Option<String>,
}

impl ServerClient {
    /// Create a new server client from configuration
    pub fn new(config: &ServerConfig) -> Result<Self, ApiError> {
        let timeout = Duration::from_secs(config.timeout_seconds);
        
        let http_client = Client::builder()
            .timeout(timeout)
            .connect_timeout(Duration::from_secs(10))
            .build()
            .map_err(|e| ApiError::NetworkError(format!("Failed to create HTTP client: {}", e)))?;
        
        Ok(Self {
            base_url: config.server_url.trim_end_matches('/').to_string(),
            http_client,
            auth_token: config.auth_token.clone(),
        })
    }
    
    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
    
    /// Build request with authentication if available
    fn build_request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        let mut request = self.http_client.request(method, &url);
        
        if let Some(token) = &self.auth_token {
            request = request.bearer_auth(token);
        }
        
        request
    }
    
    /// Handle HTTP errors and convert to ApiError
    fn handle_response_error(&self, status: reqwest::StatusCode, body: String) -> ApiError {
        match status.as_u16() {
            401 | 403 => ApiError::AuthenticationFailed,
            429 => ApiError::RateLimitExceeded,
            500..=599 => ApiError::ServerUnavailable,
            _ => ApiError::InvalidResponse(format!("HTTP {}: {}", status, body)),
        }
    }
    
    /// Transcribe audio to text via STT API
    pub async fn transcribe_audio(&self, audio: &[u8], language: Option<String>) -> Result<SttResponse, ApiError> {
        let form = multipart::Form::new()
            .part("audio", multipart::Part::bytes(audio.to_vec())
                .file_name("audio.wav")
                .mime_str("audio/wav")
                .map_err(|e| ApiError::NetworkError(format!("Failed to create multipart: {}", e)))?);
        
        let mut request = self.build_request(reqwest::Method::POST, "/api/stt")
            .multipart(form);
        
        if let Some(lang) = language {
            request = request.query(&[("language", lang)]);
        }
        
        let response = request
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    ApiError::Timeout
                } else if e.is_connect() {
                    ApiError::ConnectionRefused
                } else {
                    ApiError::NetworkError(e.to_string())
                }
            })?;
        
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(self.handle_response_error(status, body));
        }
        
        response
            .json::<SttResponse>()
            .await
            .map_err(|e| ApiError::InvalidResponse(format!("Failed to parse STT response: {}", e)))
    }
    
    /// Get AI response via Chat API
    pub async fn get_ai_response(&self, message: &str, history: Vec<ChatMessage>) -> Result<ChatResponse, ApiError> {
        let request_body = ChatRequest {
            message: message.to_string(),
            conversation_history: history,
        };
        
        let response = self.build_request(reqwest::Method::POST, "/api/chat")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    ApiError::Timeout
                } else if e.is_connect() {
                    ApiError::ConnectionRefused
                } else {
                    ApiError::NetworkError(e.to_string())
                }
            })?;
        
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(self.handle_response_error(status, body));
        }
        
        response
            .json::<ChatResponse>()
            .await
            .map_err(|e| ApiError::InvalidResponse(format!("Failed to parse chat response: {}", e)))
    }
    
    /// Synthesize speech from text via TTS API
    pub async fn synthesize_speech(&self, text: &str, voice_id: Option<String>) -> Result<Vec<u8>, ApiError> {
        let request_body = TtsRequest {
            text: text.to_string(),
            voice_id,
        };
        
        let response = self.build_request(reqwest::Method::POST, "/api/tts")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    ApiError::Timeout
                } else if e.is_connect() {
                    ApiError::ConnectionRefused
                } else {
                    ApiError::NetworkError(e.to_string())
                }
            })?;
        
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(self.handle_response_error(status, body));
        }
        
        response
            .bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| ApiError::InvalidResponse(format!("Failed to read TTS audio: {}", e)))
    }
    
    /// Check server health status
    pub async fn check_health(&self) -> Result<HealthResponse, ApiError> {
        let response = self.build_request(reqwest::Method::GET, "/api/health")
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    ApiError::Timeout
                } else if e.is_connect() {
                    ApiError::ConnectionRefused
                } else {
                    ApiError::NetworkError(e.to_string())
                }
            })?;
        
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(self.handle_response_error(status, body));
        }
        
        response
            .json::<HealthResponse>()
            .await
            .map_err(|e| ApiError::InvalidResponse(format!("Failed to parse health response: {}", e)))
    }
}


/// Retry policy for API requests
#[derive(Clone, Debug)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
        }
    }
}

impl RetryPolicy {
    /// Calculate delay for a given retry attempt
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let delay_secs = self.initial_delay.as_secs_f64() 
            * self.backoff_multiplier.powi(attempt as i32);
        
        // Handle overflow or NaN by capping at max_delay
        if delay_secs.is_nan() || delay_secs.is_infinite() || delay_secs < 0.0 {
            return self.max_delay;
        }
        
        let delay = Duration::from_secs_f64(delay_secs.min(self.max_delay.as_secs_f64()));
        delay.min(self.max_delay)
    }
}

/// Retry a future with exponential backoff
pub async fn retry_with_backoff<F, Fut, T>(
    mut operation: F,
    policy: &RetryPolicy,
) -> Result<T, ApiError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, ApiError>>,
{
    let mut last_error = None;
    
    for attempt in 0..=policy.max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if !e.is_retryable() || attempt == policy.max_retries {
                    return Err(e);
                }
                
                let delay = policy.delay_for_attempt(attempt);
                log::warn!("Request failed (attempt {}/{}): {}. Retrying in {:?}", 
                    attempt + 1, policy.max_retries, e, delay);
                
                tokio::time::sleep(delay).await;
                last_error = Some(e);
            }
        }
    }
    
    Err(last_error.unwrap_or_else(|| ApiError::NetworkError("Max retries exceeded".to_string())))
}

impl ServerClient {
    /// Transcribe audio with retry logic
    pub async fn transcribe_audio_with_retry(
        &self,
        audio: &[u8],
        language: Option<String>,
        policy: &RetryPolicy,
    ) -> Result<SttResponse, ApiError> {
        let audio = audio.to_vec();
        retry_with_backoff(
            || {
                let audio = audio.clone();
                let language = language.clone();
                async move { self.transcribe_audio(&audio, language).await }
            },
            policy,
        ).await
    }
    
    /// Get AI response with retry logic
    pub async fn get_ai_response_with_retry(
        &self,
        message: &str,
        history: Vec<ChatMessage>,
        policy: &RetryPolicy,
    ) -> Result<ChatResponse, ApiError> {
        let message = message.to_string();
        retry_with_backoff(
            || {
                let message = message.clone();
                let history = history.clone();
                async move { self.get_ai_response(&message, history).await }
            },
            policy,
        ).await
    }
    
    /// Synthesize speech with retry logic
    pub async fn synthesize_speech_with_retry(
        &self,
        text: &str,
        voice_id: Option<String>,
        policy: &RetryPolicy,
    ) -> Result<Vec<u8>, ApiError> {
        let text = text.to_string();
        retry_with_backoff(
            || {
                let text = text.clone();
                let voice_id = voice_id.clone();
                async move { self.synthesize_speech(&text, voice_id).await }
            },
            policy,
        ).await
    }
    
    /// Check health with retry logic
    pub async fn check_health_with_retry(
        &self,
        policy: &RetryPolicy,
    ) -> Result<HealthResponse, ApiError> {
        retry_with_backoff(
            || async { self.check_health().await },
            policy,
        ).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ServerConfig, ConnectionType};
    
    #[test]
    fn test_server_client_creation() {
        let config = ServerConfig {
            server_url: "http://localhost:8080".to_string(),
            connection_type: ConnectionType::LocalNetwork,
            auth_token: None,
            timeout_seconds: 30,
        };
        
        let client = ServerClient::new(&config);
        assert!(client.is_ok());
        
        let client = client.unwrap();
        assert_eq!(client.base_url(), "http://localhost:8080");
    }
    
    #[test]
    fn test_server_client_with_auth() {
        let config = ServerConfig {
            server_url: "https://api.example.com".to_string(),
            connection_type: ConnectionType::PublicUrl,
            auth_token: Some("test-token".to_string()),
            timeout_seconds: 60,
        };
        
        let client = ServerClient::new(&config);
        assert!(client.is_ok());
    }
    
    #[test]
    fn test_retry_policy_delay() {
        let policy = RetryPolicy::default();
        
        let delay0 = policy.delay_for_attempt(0);
        assert_eq!(delay0, Duration::from_secs(1));
        
        let delay1 = policy.delay_for_attempt(1);
        assert_eq!(delay1, Duration::from_secs(2));
        
        let delay2 = policy.delay_for_attempt(2);
        assert_eq!(delay2, Duration::from_secs(4));
    }
    
    #[test]
    fn test_retry_policy_max_delay() {
        let policy = RetryPolicy {
            max_retries: 10,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
        };
        
        let delay10 = policy.delay_for_attempt(10);
        assert_eq!(delay10, Duration::from_secs(10)); // Capped at max_delay
    }
    
    #[tokio::test]
    async fn test_empty_audio_handling() {
        let config = ServerConfig::default();
        let client = ServerClient::new(&config).unwrap();
        
        // This will fail to connect since there's no server, but tests the client setup
        let result = client.transcribe_audio(&[], None).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_timeout_scenario() {
        let config = ServerConfig {
            server_url: "http://192.0.2.1:9999".to_string(), // Non-routable IP
            connection_type: ConnectionType::LocalNetwork,
            auth_token: None,
            timeout_seconds: 1,
        };
        
        let client = ServerClient::new(&config).unwrap();
        let result = client.check_health().await;
        
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, ApiError::Timeout | ApiError::ConnectionRefused | ApiError::NetworkError(_)));
        }
    }
    
    #[tokio::test]
    async fn test_invalid_url() {
        let config = ServerConfig {
            server_url: "not-a-valid-url".to_string(),
            connection_type: ConnectionType::LocalNetwork,
            auth_token: None,
            timeout_seconds: 5,
        };
        
        let client = ServerClient::new(&config).unwrap();
        let result = client.check_health().await;
        assert!(result.is_err());
    }
}
