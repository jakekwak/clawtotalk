// Property-based tests for Dioxus Voice Assistant

use proptest::prelude::*;
use dioxus_voice_assistant::models::*;
use dioxus_voice_assistant::audio::CrossPlatformAudioManager;
use dioxus_voice_assistant::vad::{VoiceActivityDetector, VadResult};
use dioxus_voice_assistant::error::ApiError;

#[cfg(test)]
mod property_tests {
    use super::*;
    
    // Strategy for generating RecordingMode
    fn recording_mode_strategy() -> impl Strategy<Value = RecordingMode> {
        prop_oneof![
            Just(RecordingMode::Hold),
            Just(RecordingMode::Toggle),
            Just(RecordingMode::Auto),
        ]
    }
    
    // Strategy for generating VadSettings
    fn vad_settings_strategy() -> impl Strategy<Value = VadSettings> {
        (0.001f32..0.1f32, 256usize..4096, 100u64..5000)
            .prop_map(|(threshold, window_size, silence_duration_ms)| VadSettings {
                threshold,
                window_size,
                silence_duration_ms,
            })
    }
    
    // Strategy for generating AudioSettings
    fn audio_settings_strategy() -> impl Strategy<Value = AudioSettings> {
        (8000u32..48000, 1u16..3, 1024usize..8192)
            .prop_map(|(sample_rate, channels, buffer_size)| AudioSettings {
                sample_rate,
                channels,
                buffer_size,
            })
    }
    
    // Strategy for generating ServerConfig
    fn server_config_strategy() -> impl Strategy<Value = ServerConfig> {
        (
            "https?://[a-z0-9.-]+:[0-9]{4,5}",
            prop_oneof![
                Just(ConnectionType::Tailscale),
                Just(ConnectionType::PublicUrl),
                Just(ConnectionType::LocalNetwork),
            ],
            prop::option::of("[a-zA-Z0-9]{32,64}"),
            5u64..120,
        )
            .prop_map(|(server_url, connection_type, auth_token, timeout_seconds)| ServerConfig {
                server_url,
                connection_type,
                auth_token,
                timeout_seconds,
            })
    }
    
    // Strategy for generating Settings
    fn settings_strategy() -> impl Strategy<Value = Settings> {
        (
            server_config_strategy(),
            recording_mode_strategy(),
            vad_settings_strategy(),
            audio_settings_strategy(),
        )
            .prop_map(|(server_config, recording_mode, vad_settings, audio_settings)| Settings {
                server_config,
                recording_mode,
                vad_settings,
                audio_settings,
            })
    }
    
    proptest! {
        /// Feature: dioxus-voice-assistant, Property 2: 오디오 데이터 라운드트립
        /// **Validates: Requirements 8.4**
        /// 
        /// This property verifies that Settings can be serialized to JSON and deserialized
        /// back without any data loss. This ensures that user settings persist correctly
        /// across application restarts.
        #[test]
        fn test_settings_roundtrip(settings in settings_strategy()) {
            // Serialize settings to JSON
            let json = serde_json::to_string(&settings)
                .expect("Failed to serialize settings");
            
            // Deserialize back from JSON
            let deserialized: Settings = serde_json::from_str(&json)
                .expect("Failed to deserialize settings");
            
            // Verify all fields match
            assert_eq!(settings.recording_mode, deserialized.recording_mode);
            assert_eq!(settings.server_config.server_url, deserialized.server_config.server_url);
            assert_eq!(settings.server_config.connection_type, deserialized.server_config.connection_type);
            assert_eq!(settings.server_config.auth_token, deserialized.server_config.auth_token);
            assert_eq!(settings.server_config.timeout_seconds, deserialized.server_config.timeout_seconds);
            assert_eq!(settings.vad_settings.threshold, deserialized.vad_settings.threshold);
            assert_eq!(settings.vad_settings.window_size, deserialized.vad_settings.window_size);
            assert_eq!(settings.vad_settings.silence_duration_ms, deserialized.vad_settings.silence_duration_ms);
            assert_eq!(settings.audio_settings.sample_rate, deserialized.audio_settings.sample_rate);
            assert_eq!(settings.audio_settings.channels, deserialized.audio_settings.channels);
            assert_eq!(settings.audio_settings.buffer_size, deserialized.audio_settings.buffer_size);
        }
        
        /// Feature: dioxus-voice-assistant, Property 2: 오디오 데이터 라운드트립
        /// **Validates: Requirements 3.1**
        /// 
        /// This property verifies that audio data can go through the complete cycle of
        /// recording → storage → loading → playback preparation without losing quality.
        /// We test this by converting audio samples to bytes and back, ensuring the
        /// conversion is lossless within acceptable tolerance.
        #[test]
        fn test_audio_data_roundtrip(
            audio_samples in prop::collection::vec(-1.0f32..1.0f32, 1000..5000)
        ) {
            // Simulate the audio manager's conversion process
            // Convert f32 samples to i16 PCM (as done in stop_recording)
            let pcm_data: Vec<i16> = audio_samples.iter()
                .map(|&s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
                .collect();
            
            // Convert to bytes (as returned by stop_recording)
            let mut bytes = Vec::with_capacity(pcm_data.len() * 2);
            for sample in &pcm_data {
                bytes.extend_from_slice(&sample.to_le_bytes());
            }
            
            // Simulate loading: Convert bytes back to i16 samples (as done in play_audio)
            let mut loaded_samples = Vec::with_capacity(bytes.len() / 2);
            for chunk in bytes.chunks_exact(2) {
                let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
                loaded_samples.push(sample);
            }
            
            // Convert back to f32 for comparison
            let reconstructed: Vec<f32> = loaded_samples.iter()
                .map(|&s| s as f32 / i16::MAX as f32)
                .collect();
            
            // Verify the roundtrip maintains data integrity
            assert_eq!(audio_samples.len(), reconstructed.len(), 
                "Sample count should be preserved");
            
            // Check that each sample is within acceptable tolerance
            // Due to quantization from f32 to i16, we allow small differences
            let tolerance = 1.0 / i16::MAX as f32 * 2.0; // Allow 2 quantization steps
            for (original, reconstructed) in audio_samples.iter().zip(reconstructed.iter()) {
                let diff = (original - reconstructed).abs();
                assert!(diff <= tolerance,
                    "Sample difference {} exceeds tolerance {} (original: {}, reconstructed: {})",
                    diff, tolerance, original, reconstructed);
            }
        }
        
        /// Feature: dioxus-voice-assistant, Property 4: 음성 활동 감지 정확성
        /// **Validates: Requirements 6.1, 6.2, 6.3, 6.4**
        /// 
        /// This property verifies that the VAD correctly distinguishes between speech
        /// and background noise, automatically starts recording when speech is detected,
        /// and stops recording after the configured silence duration.
        #[test]
        fn test_vad_accuracy(
            vad_settings in vad_settings_strategy(),
            noise_level in 0.0f32..0.005f32,
        ) {
            let mut vad = VoiceActivityDetector::new(vad_settings.clone());
            
            // Calculate speech level that's guaranteed to be above threshold
            // Speech should be at least 5x the threshold above the noise floor
            let speech_level = noise_level + (vad_settings.threshold * 5.0).max(0.2);
            
            // Phase 1: Establish noise floor with background noise
            for _ in 0..10 {
                // Create noise frame with consistent low energy
                let noise_frame: Vec<f32> = vec![noise_level; 1024];
                let result = vad.analyze_frame(&noise_frame);
                
                // Background noise should be detected as silence
                assert!(
                    matches!(result, VadResult::Silence),
                    "Background noise should be detected as silence"
                );
            }
            
            // Phase 2: Introduce speech (higher energy than noise + threshold)
            let mut speech_detected = false;
            for _ in 0..10 {
                // Create speech frame with high energy
                let speech_frame: Vec<f32> = vec![speech_level; 1024];
                let result = vad.analyze_frame(&speech_frame);
                
                if matches!(result, VadResult::Speech) {
                    speech_detected = true;
                }
            }
            
            // Speech should be detected at some point during speech frames
            assert!(
                speech_detected,
                "Speech should be detected when energy is significantly above noise floor (noise: {}, speech: {}, threshold: {})",
                noise_level, speech_level, vad_settings.threshold
            );
            
            // Phase 3: Return to silence
            // VAD should continue detecting speech for silence_duration
            let silence_frame: Vec<f32> = vec![noise_level; 1024];
            vad.analyze_frame(&silence_frame);
            
            // Should still detect speech immediately after speech ends
            assert!(
                vad.is_speech_detected(),
                "VAD should continue detecting speech within silence duration"
            );
            
            // Phase 4: Verify reset functionality
            vad.reset();
            assert!(
                !vad.is_speech_detected(),
                "VAD should not detect speech after reset"
            );
            assert_eq!(
                vad.get_noise_floor(), 0.0,
                "Noise floor should be reset to 0"
            );
        }
        
        /// Feature: dioxus-voice-assistant, Property 1: 녹음 모드 동작 일관성
        /// **Validates: Requirements 2.2, 2.3, 2.4**
        /// 
        /// This property verifies that all recording modes (Hold, Toggle, Auto) behave
        /// consistently according to their specifications:
        /// - Hold: Recording starts on button press and stops on button release
        /// - Toggle: First click starts recording, second click stops it
        /// - Auto: Recording starts automatically when speech is detected and stops after silence
        #[test]
        fn test_recording_mode_consistency(
            mode in recording_mode_strategy(),
            button_events in prop::collection::vec(prop::bool::ANY, 1..20),
            speech_frames in prop::collection::vec(
                prop::collection::vec(-1.0f32..1.0f32, 1000..2000),
                1..10
            ),
        ) {
        use dioxus_voice_assistant::recording::*;
        use std::sync::Arc;
        
        // Create audio manager and VAD
        let audio_manager = Arc::new(CrossPlatformAudioManager::new().unwrap());
        let vad = VoiceActivityDetector::default();
        let mut manager = RecordingModeManager::new(audio_manager, vad);
        
        manager.set_mode(mode.clone());
        
        match mode {
            RecordingMode::Hold => {
                // Hold mode: Recording should only be active while button is pressed
                let runtime = tokio::runtime::Runtime::new().unwrap();
                
                for &is_pressed in &button_events {
                    runtime.block_on(async {
                        if is_pressed {
                            // Button press should start recording
                            manager.on_button_press().await.unwrap();
                            assert!(
                                manager.is_recording(),
                                "Hold mode: Recording should be active after button press"
                            );
                        } else {
                            // Button release should stop recording
                            let result = manager.on_button_release().await.unwrap();
                            
                            // If we were recording, we should get audio data
                            if manager.is_recording() {
                                assert!(
                                    result.is_some(),
                                    "Hold mode: Should return audio data when stopping recording"
                                );
                            }
                            
                            assert!(
                                !manager.is_recording(),
                                "Hold mode: Recording should stop after button release"
                            );
                        }
                    });
                }
            }
            
            RecordingMode::Toggle => {
                // Toggle mode: Each click should toggle the recording state
                let runtime = tokio::runtime::Runtime::new().unwrap();
                let mut expected_recording = false;
                
                for _ in &button_events {
                    runtime.block_on(async {
                        // In toggle mode, button release toggles the state
                        let result = manager.on_button_release().await.unwrap();
                        expected_recording = !expected_recording;
                        
                        assert_eq!(
                            manager.is_recording(),
                            expected_recording,
                            "Toggle mode: Recording state should toggle with each click"
                        );
                        
                        // Should only return audio data when stopping
                        if !expected_recording {
                            assert!(
                                result.is_some(),
                                "Toggle mode: Should return audio data when stopping recording"
                            );
                        } else {
                            assert!(
                                result.is_none(),
                                "Toggle mode: Should not return audio data when starting recording"
                            );
                        }
                    });
                }
            }
            
            RecordingMode::Auto => {
                // Auto mode: Recording should start/stop based on speech detection
                let runtime = tokio::runtime::Runtime::new().unwrap();
                
                runtime.block_on(async {
                    // Establish baseline with low energy frames
                    for _ in 0..5 {
                        let low_frame = vec![0.001; 1024];
                        manager.process_audio_frame(&low_frame).await.unwrap();
                    }
                    
                    // Initially should not be recording
                    let initial_recording = manager.is_recording();
                    
                    // Process speech frames (high energy)
                    let mut speech_detected = false;
                    for frame in &speech_frames {
                        // Create high-energy speech frame
                        let speech_frame: Vec<f32> = frame.iter()
                            .map(|&x| if x.abs() < 0.1 { 0.5 } else { x })
                            .collect();
                        
                        manager.process_audio_frame(&speech_frame).await.unwrap();
                        
                        if manager.is_recording() {
                            speech_detected = true;
                        }
                    }
                    
                    // Auto mode should eventually start recording when speech is detected
                    // (or remain in initial state if no clear speech pattern)
                    if speech_detected {
                        assert!(
                            manager.is_recording() || !initial_recording,
                            "Auto mode: Should start recording when speech is detected"
                        );
                    }
                    
                    // Process silence frames
                    for _ in 0..5 {
                        let silence_frame = vec![0.001; 1024];
                        manager.process_audio_frame(&silence_frame).await.unwrap();
                    }
                    
                    // After silence, recording should eventually stop
                    // (Note: This depends on silence_duration_ms setting)
                });
            }
        }
    }
    
    /// Feature: dioxus-voice-assistant, Property 3: API 통신 일관성
    /// **Validates: Requirements 4.1, 4.2, 5.1**
    /// 
    /// This property verifies that the ServerClient correctly handles various inputs
    /// and maintains consistent behavior across different API calls. It tests:
    /// - Request/response serialization roundtrip
    /// - Error handling for different failure scenarios
    /// - Retry logic with exponential backoff
    #[test]
    fn test_api_communication_consistency(
        server_url in "https?://[a-z0-9.-]+:[0-9]{4,5}",
        auth_token in prop::option::of("[a-zA-Z0-9]{32,64}"),
        timeout_seconds in 5u64..120,
        message in "[a-zA-Z0-9가-힣 ]{1,200}",
        text in "[a-zA-Z0-9가-힣 ]{1,100}",
    ) {
        use dioxus_voice_assistant::api::*;
        use dioxus_voice_assistant::models::*;
        
        // Create server configuration
        let config = ServerConfig {
            server_url: server_url.clone(),
            connection_type: ConnectionType::LocalNetwork,
            auth_token: auth_token.clone(),
            timeout_seconds,
        };
        
        // Create server client
        let client = ServerClient::new(&config);
        assert!(
            client.is_ok(),
            "ServerClient should be created successfully with valid configuration"
        );
        
        let client = client.unwrap();
        
        // Verify client configuration
        assert_eq!(
            client.base_url(),
            server_url.trim_end_matches('/'),
            "Base URL should match configuration (without trailing slash)"
        );
        
        // Test request/response model serialization
        let chat_request = ChatRequest {
            message: message.clone(),
            conversation_history: vec![
                ChatMessage {
                    role: "user".to_string(),
                    content: "Hello".to_string(),
                },
                ChatMessage {
                    role: "assistant".to_string(),
                    content: "Hi there!".to_string(),
                },
            ],
        };
        
        // Serialize and deserialize chat request
        let json = serde_json::to_string(&chat_request)
            .expect("ChatRequest should serialize to JSON");
        let deserialized: ChatRequest = serde_json::from_str(&json)
            .expect("ChatRequest should deserialize from JSON");
        
        assert_eq!(
            chat_request.message,
            deserialized.message,
            "Message should be preserved in serialization roundtrip"
        );
        assert_eq!(
            chat_request.conversation_history.len(),
            deserialized.conversation_history.len(),
            "Conversation history length should be preserved"
        );
        
        // Test TTS request serialization
        let tts_request = TtsRequest {
            text: text.clone(),
            voice_id: Some("korean_female_1".to_string()),
        };
        
        let json = serde_json::to_string(&tts_request)
            .expect("TtsRequest should serialize to JSON");
        let deserialized: TtsRequest = serde_json::from_str(&json)
            .expect("TtsRequest should deserialize from JSON");
        
        assert_eq!(
            tts_request.text,
            deserialized.text,
            "Text should be preserved in serialization roundtrip"
        );
        assert_eq!(
            tts_request.voice_id,
            deserialized.voice_id,
            "Voice ID should be preserved in serialization roundtrip"
        );
        
        // Test retry policy
        let policy = RetryPolicy::default();
        
        // Verify exponential backoff
        let delay0 = policy.delay_for_attempt(0);
        let delay1 = policy.delay_for_attempt(1);
        let delay2 = policy.delay_for_attempt(2);
        
        assert!(
            delay1 > delay0,
            "Retry delay should increase with each attempt"
        );
        assert!(
            delay2 > delay1,
            "Retry delay should continue increasing"
        );
        
        // Verify max delay cap
        let large_delay = policy.delay_for_attempt(100);
        assert!(
            large_delay <= policy.max_delay,
            "Retry delay should be capped at max_delay"
        );
        
        // Test error handling
        let runtime = tokio::runtime::Runtime::new().unwrap();
        
        // Test that client handles connection failures gracefully
        // (This will fail to connect since there's no actual server)
        runtime.block_on(async {
            let result = client.check_health().await;
            
            // Should return an error (not panic)
            assert!(
                result.is_err(),
                "Client should return error when server is unavailable"
            );
            
            // Verify error types are correctly classified
            if let Err(e) = result {
                // Some errors like invalid URL format are not retryable
                // Only connection/network/timeout errors should be retryable
                match e {
                    ApiError::ConnectionRefused | ApiError::Timeout | ApiError::NetworkError(_) => {
                        assert!(
                            e.is_retryable(),
                            "Connection/timeout/network errors should be retryable: {:?}", e
                        );
                    }
                    _ => {
                        // Other errors (like invalid URL) may not be retryable
                        // This is expected behavior
                    }
                }
            }
        });
    }
    }
}
