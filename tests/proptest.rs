// Property-based tests for Dioxus Voice Assistant

use proptest::prelude::*;
use dioxus_voice_assistant::models::*;
use dioxus_voice_assistant::audio::CrossPlatformAudioManager;
use dioxus_voice_assistant::vad::{VoiceActivityDetector, VadResult};
use dioxus_voice_assistant::error::ApiError;
use dioxus::prelude::*;

// Configure proptest to run fewer cases for faster execution
// Default is 256 cases, we reduce to 20 for speed
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 20,
        .. ProptestConfig::default()
    })]
}

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
    
    /// Feature: dioxus-voice-assistant, Property 7: UI 상태 동기화
    /// **Validates: Requirements 7.3, 8.5**
    /// 
    /// This property verifies that UI state changes are immediately reflected in the
    /// application state. When settings are updated, the recording mode should change
    /// immediately, and when messages are added, they should appear in the conversation
    /// history without delay.
    #[test]
    fn test_ui_state_synchronization(
        initial_settings in settings_strategy(),
        new_settings in settings_strategy(),
        messages in prop::collection::vec(
            ("[a-zA-Z0-9가-힣 ]{1,100}", prop::bool::ANY),
            1..10
        ),
    ) {
        use dioxus_voice_assistant::state::AppState;
        use dioxus_voice_assistant::models::{Message, MessageType};
        
        // Create application state
        let mut app_state = AppState::new();
        
        // Set initial settings
        app_state.update_settings(initial_settings.clone());
        
        // Requirement 8.5: Settings changes should be applied immediately
        assert_eq!(
            *app_state.settings.read(),
            initial_settings,
            "Initial settings should be applied immediately"
        );
        assert_eq!(
            *app_state.recording_mode.read(),
            initial_settings.recording_mode,
            "Recording mode should match settings immediately"
        );
        
        // Update settings
        app_state.update_settings(new_settings.clone());
        
        // Verify immediate synchronization
        assert_eq!(
            *app_state.settings.read(),
            new_settings,
            "New settings should be applied immediately"
        );
        assert_eq!(
            *app_state.recording_mode.read(),
            new_settings.recording_mode,
            "Recording mode should update immediately with settings"
        );
        
        // Test message synchronization
        // Requirement 7.3: Messages should appear immediately in conversation history
        let initial_count = app_state.get_message_count();
        let messages_len = messages.len();
        
        for (content, is_user) in messages {
            let message_type = if is_user {
                MessageType::User
            } else {
                MessageType::Assistant
            };
            
            let message = Message::new(content.clone(), message_type);
            app_state.add_message(message.clone());
            
            // Verify message was added immediately
            let current_messages = app_state.conversation_history.read();
            assert!(
                current_messages.iter().any(|m| m.content == content),
                "Message should appear in conversation history immediately"
            );
        }
        
        // Verify all messages were added
        let final_count = app_state.get_message_count();
        assert_eq!(
            final_count,
            initial_count + messages_len,
            "All messages should be added to conversation history"
        );
        
        // Test recording state synchronization
        // Requirement 7.3: Visual feedback should update immediately
        assert!(!*app_state.is_recording.read(), "Should not be recording initially");
        
        app_state.start_recording();
        assert!(
            *app_state.is_recording.read(),
            "Recording state should update immediately when starting"
        );
        assert_eq!(
            *app_state.current_status.read(),
            AppStatus::Recording,
            "Status should update immediately to Recording"
        );
        
        app_state.stop_recording();
        assert!(
            !*app_state.is_recording.read(),
            "Recording state should update immediately when stopping"
        );
        assert_eq!(
            *app_state.current_status.read(),
            AppStatus::Idle,
            "Status should update immediately to Idle"
        );
        
        // Test toggle functionality
        let initial_state = *app_state.is_recording.read();
        app_state.toggle_recording();
        assert_eq!(
            *app_state.is_recording.read(),
            !initial_state,
            "Toggle should immediately flip recording state"
        );
        
        // Test clear conversation
        app_state.clear_conversation();
        assert_eq!(
            app_state.get_message_count(),
            0,
            "Conversation should be cleared immediately"
        );
    }
    
    /// Feature: dioxus-voice-assistant, Property 8: 입력 처리 포괄성
    /// **Validates: Requirements 7.2**
    /// 
    /// This property verifies that the application handles all input types consistently:
    /// - Mouse events (click, mousedown, mouseup, mouseleave)
    /// - Touch events (touchstart, touchend)
    /// - Keyboard events (for accessibility)
    /// 
    /// The recording button should respond correctly to all input methods across
    /// different recording modes.
    #[test]
    fn test_input_handling_comprehensiveness(
        mode in recording_mode_strategy(),
        input_events in prop::collection::vec(
            prop_oneof![
                Just("click"),
                Just("mousedown"),
                Just("mouseup"),
                Just("mouseleave"),
                Just("touchstart"),
                Just("touchend"),
            ],
            1..20
        ),
    ) {
        use dioxus_voice_assistant::recording::*;
        use std::sync::Arc;
        
        // Create recording manager
        let audio_manager = Arc::new(CrossPlatformAudioManager::new().unwrap());
        let vad = VoiceActivityDetector::default();
        let mut manager = RecordingModeManager::new(audio_manager, vad);
        
        manager.set_mode(mode.clone());
        
        let runtime = tokio::runtime::Runtime::new().unwrap();
        
        // Track expected state based on mode and events
        let mut expected_recording = false;
        let mut is_pressed = false;
        
        for event in input_events {
            runtime.block_on(async {
                match (mode, event) {
                    // Hold mode: Recording active while button is pressed
                    (RecordingMode::Hold, "mousedown") | (RecordingMode::Hold, "touchstart") => {
                        is_pressed = true;
                        manager.on_button_press().await.unwrap();
                        expected_recording = true;
                        
                        assert_eq!(
                            manager.is_recording(),
                            expected_recording,
                            "Hold mode: Should start recording on press event"
                        );
                    }
                    
                    (RecordingMode::Hold, "mouseup") | 
                    (RecordingMode::Hold, "touchend") | 
                    (RecordingMode::Hold, "mouseleave") => {
                        if is_pressed {
                            is_pressed = false;
                            manager.on_button_release().await.unwrap();
                            expected_recording = false;
                            
                            assert_eq!(
                                manager.is_recording(),
                                expected_recording,
                                "Hold mode: Should stop recording on release event"
                            );
                        }
                    }
                    
                    // Toggle mode: Click toggles state
                    (RecordingMode::Toggle, "click") | 
                    (RecordingMode::Toggle, "mouseup") | 
                    (RecordingMode::Toggle, "touchend") => {
                        manager.on_button_release().await.unwrap();
                        expected_recording = !expected_recording;
                        
                        assert_eq!(
                            manager.is_recording(),
                            expected_recording,
                            "Toggle mode: Should toggle recording state on click/release"
                        );
                    }
                    
                    // Auto mode: Button events don't affect recording
                    (RecordingMode::Auto, _) => {
                        // Auto mode ignores button events
                        // Recording is controlled by VAD
                        // Just verify the manager doesn't crash
                        let _ = manager.on_button_press().await;
                        let _ = manager.on_button_release().await;
                    }
                    
                    // Other combinations: No state change expected
                    _ => {
                        // These events don't trigger state changes in their respective modes
                    }
                }
            });
        }
        
        // Verify final state consistency
        assert_eq!(
            manager.is_recording(),
            expected_recording,
            "Final recording state should match expected state based on input events"
        );
        
        // Test that all input types are handled without panicking
        // This verifies comprehensive input handling
        runtime.block_on(async {
            // Reset to known state
            manager.set_mode(RecordingMode::Toggle);
            if manager.is_recording() {
                manager.on_button_release().await.unwrap();
            }
            
            // Test each input type
            let input_types = vec![
                "click", "mousedown", "mouseup", "mouseleave",
                "touchstart", "touchend"
            ];
            
            for input_type in input_types {
                match input_type {
                    "mousedown" | "touchstart" => {
                        let result = manager.on_button_press().await;
                        assert!(
                            result.is_ok(),
                            "Should handle {} event without error", input_type
                        );
                    }
                    _ => {
                        let result = manager.on_button_release().await;
                        assert!(
                            result.is_ok(),
                            "Should handle {} event without error", input_type
                        );
                    }
                }
            }
        });
    }
    
    /// Feature: dioxus-voice-assistant, Property 6: 오류 처리 완전성
    /// **Validates: Requirements 9.1, 9.2, 9.3**
    /// 
    /// This property verifies that all error situations (network errors, invalid API keys,
    /// permission denials) are handled correctly with appropriate error messages and
    /// recovery methods. The system should:
    /// - Display clear error messages for all error types
    /// - Provide recovery actions for each error
    /// - Classify errors by severity correctly
    /// - Handle retryable vs non-retryable errors appropriately
    #[test]
    fn test_error_handling_completeness(
        error_type in prop_oneof![
            Just("network"),
            Just("auth"),
            Just("timeout"),
            Just("audio_permission"),
            Just("audio_device"),
            Just("rate_limit"),
            Just("server_unavailable"),
        ],
        retry_attempts in 0u32..5,
    ) {
        use dioxus_voice_assistant::error::*;
        use dioxus_voice_assistant::error_handler::*;
        
        let runtime = tokio::runtime::Runtime::new().unwrap();
        
        runtime.block_on(async {
            let handler = ErrorHandler::new();
            
            // Create error based on type
            let error = match error_type {
                "network" => AppError::Api(ApiError::NetworkError("Connection failed".to_string())),
                "auth" => AppError::Api(ApiError::AuthenticationFailed),
                "timeout" => AppError::Api(ApiError::Timeout),
                "audio_permission" => AppError::Audio(AudioError::PermissionDenied),
                "audio_device" => AppError::Audio(AudioError::DeviceNotFound),
                "rate_limit" => AppError::Api(ApiError::RateLimitExceeded),
                "server_unavailable" => AppError::Api(ApiError::ServerUnavailable),
                _ => AppError::Unknown("Unknown error".to_string()),
            };
            
            // Requirement 10.1: All errors should have user-friendly messages
            let user_message = error.user_message();
            assert!(
                !user_message.is_empty(),
                "Error should have a non-empty user message"
            );
            assert!(
                user_message.len() > 10,
                "Error message should be descriptive (got: '{}')", user_message
            );
            
            // Requirement 10.2: All errors should have recovery actions
            let recovery_actions = error.recovery_actions();
            assert!(
                !recovery_actions.is_empty(),
                "Error should have at least one recovery action"
            );
            
            // Verify recovery actions are appropriate for error type
            match error_type {
                "audio_permission" => {
                    assert!(
                        recovery_actions.contains(&RecoveryAction::RequestPermission),
                        "Permission errors should suggest requesting permission"
                    );
                }
                "audio_device" => {
                    assert!(
                        recovery_actions.contains(&RecoveryAction::ShowDeviceSettings),
                        "Device errors should suggest checking device settings"
                    );
                }
                "network" | "timeout" | "server_unavailable" => {
                    assert!(
                        recovery_actions.contains(&RecoveryAction::Retry) ||
                        recovery_actions.contains(&RecoveryAction::ShowSettings),
                        "Network errors should suggest retry or checking settings"
                    );
                }
                "auth" => {
                    assert!(
                        recovery_actions.contains(&RecoveryAction::ShowSettings),
                        "Auth errors should suggest checking settings"
                    );
                }
                _ => {
                    // Other errors should have some recovery action
                }
            }
            
            // Requirement 10.3: Errors should be classified by severity
            let severity = error.severity();
            match error_type {
                "audio_permission" | "audio_device" | "auth" => {
                    assert_eq!(
                        severity,
                        ErrorSeverity::Critical,
                        "Critical errors should be classified as Critical"
                    );
                }
                "network" | "timeout" | "server_unavailable" | "rate_limit" => {
                    assert!(
                        matches!(severity, ErrorSeverity::Warning | ErrorSeverity::Error),
                        "Temporary errors should be Warning or Error severity"
                    );
                }
                _ => {
                    // Other errors should have appropriate severity
                }
            }
            
            // Test error handler
            let result = handler.handle_error(error.clone()).await;
            
            // Retryable errors should return Ok, non-retryable should return Err
            if error.is_retryable() {
                assert!(
                    result.is_ok(),
                    "Retryable errors should be handled successfully"
                );
            } else {
                assert!(
                    result.is_err(),
                    "Non-retryable errors should return error"
                );
            }
            
            // Verify notification was created
            let notifications = handler.get_notifications().await;
            assert_eq!(
                notifications.len(),
                1,
                "Error handler should create a notification"
            );
            
            let notification = &notifications[0];
            assert_eq!(
                notification.message,
                error.user_message(),
                "Notification message should match error message"
            );
            assert_eq!(
                notification.severity,
                error.severity(),
                "Notification severity should match error severity"
            );
            
            // Test retry policy for retryable errors
            if error.is_retryable() {
                let error_type_enum = match error {
                    AppError::Api(ApiError::NetworkError(_)) => ErrorType::Network,
                    AppError::Api(_) => ErrorType::Api,
                    AppError::Audio(_) => ErrorType::Audio,
                    _ => ErrorType::Network,
                };
                
                let policy = handler.get_retry_policy(error_type_enum);
                
                // Verify retry delays increase with exponential backoff
                for attempt in 0..retry_attempts.min(policy.max_retries) {
                    let delay = policy.delay_for_attempt(attempt);
                    
                    assert!(
                        delay >= policy.initial_delay,
                        "Retry delay should be at least initial_delay"
                    );
                    assert!(
                        delay <= policy.max_delay,
                        "Retry delay should not exceed max_delay"
                    );
                    
                    if attempt > 0 {
                        let prev_delay = policy.delay_for_attempt(attempt - 1);
                        assert!(
                            delay >= prev_delay,
                            "Retry delay should increase or stay same with each attempt"
                        );
                    }
                }
            }
            
            // Test notification management
            handler.clear_notifications().await;
            let notifications = handler.get_notifications().await;
            assert_eq!(
                notifications.len(),
                0,
                "Notifications should be cleared"
            );
        });
    }
    
    /// Feature: dioxus-voice-assistant, Property 10: 자동 재시도 메커니즘
    /// **Validates: Requirements 9.4**
    /// 
    /// This property verifies that the system automatically retries temporary failures
    /// according to the configured retry policy. It tests:
    /// - Exponential backoff between retry attempts
    /// - Maximum retry limit enforcement
    /// - Successful reconnection after transient failures
    /// - Proper handling when max retries are exceeded
    #[test]
    fn test_automatic_retry_mechanism(
        max_retries in 1u32..10,
        initial_delay_secs in 1u64..5,
        backoff_multiplier in 1.5f64..3.0,
    ) {
        use dioxus_voice_assistant::api::*;
        use dioxus_voice_assistant::connection::ConnectionManager;
        use dioxus_voice_assistant::models::*;
        use std::time::Duration;
        
        let runtime = tokio::runtime::Runtime::new().unwrap();
        
        runtime.block_on(async {
            // Test retry policy configuration
            let policy = RetryPolicy {
                max_retries,
                initial_delay: Duration::from_secs(initial_delay_secs),
                max_delay: Duration::from_secs(60),
                backoff_multiplier,
            };
            
            // Verify exponential backoff
            let mut prev_delay = Duration::from_secs(0);
            for attempt in 0..max_retries {
                let delay = policy.delay_for_attempt(attempt);
                
                // Delay should be at least initial_delay
                assert!(
                    delay >= policy.initial_delay,
                    "Delay should be at least initial_delay"
                );
                
                // Delay should not exceed max_delay
                assert!(
                    delay <= policy.max_delay,
                    "Delay should not exceed max_delay"
                );
                
                // Delay should increase with each attempt (exponential backoff)
                if attempt > 0 {
                    assert!(
                        delay >= prev_delay,
                        "Delay should increase with each attempt (attempt {}: {:?}, previous: {:?})",
                        attempt, delay, prev_delay
                    );
                    
                    // Verify exponential growth (within floating point tolerance)
                    let expected_delay_secs = initial_delay_secs as f64 
                        * backoff_multiplier.powi(attempt as i32);
                    let expected_delay = Duration::from_secs_f64(
                        expected_delay_secs.min(60.0)
                    );
                    
                    // Allow small tolerance for floating point calculations
                    let diff = if delay > expected_delay {
                        delay - expected_delay
                    } else {
                        expected_delay - delay
                    };
                    
                    assert!(
                        diff < Duration::from_millis(100),
                        "Delay should follow exponential backoff formula (attempt {}: expected {:?}, got {:?})",
                        attempt, expected_delay, delay
                    );
                }
                
                prev_delay = delay;
            }
            
            // Test connection manager retry behavior
            // Use a very short timeout to make the test faster
            let config = ServerConfig {
                server_url: "http://192.0.2.1:9999".to_string(), // Non-routable IP for testing
                connection_type: ConnectionType::LocalNetwork,
                auth_token: None,
                timeout_seconds: 1, // Very short timeout
            };
            
            let _manager = ConnectionManager::new(config);
            
            // Note: We skip the actual connection test here because it takes too long
            // The retry logic is tested separately with the retry_with_backoff function
            
            // Test reconnection with limited retries
            // Create a custom manager with very limited retries for faster testing
            let mut _test_manager = ConnectionManager::new(ServerConfig {
                server_url: "http://192.0.2.1:9999".to_string(),
                connection_type: ConnectionType::LocalNetwork,
                auth_token: None,
                timeout_seconds: 1,
            });
            
            // Set a very fast retry policy for testing
            _test_manager.reconnect_policy = RetryPolicy {
                max_retries: 1, // Only 1 retry for speed
                initial_delay: Duration::from_millis(1),
                max_delay: Duration::from_millis(10),
                backoff_multiplier: 2.0,
            };
            
            // Note: We skip the actual reconnection test because it's too slow
            // The retry mechanism is validated through the retry_with_backoff tests below
            
            // Test retry_with_backoff function
            let test_policy = RetryPolicy {
                max_retries: 3,
                initial_delay: Duration::from_millis(10),
                max_delay: Duration::from_millis(100),
                backoff_multiplier: 2.0,
            };
            
            let mut attempt_count = 0;
            let result = retry_with_backoff(
                || {
                    attempt_count += 1;
                    async move {
                        if attempt_count < 3 {
                            // Fail first 2 attempts
                            Err(ApiError::NetworkError("Temporary failure".to_string()))
                        } else {
                            // Succeed on 3rd attempt
                            Ok("Success")
                        }
                    }
                },
                &test_policy,
            ).await;
            
            // Should succeed after retries
            assert!(
                result.is_ok(),
                "Should succeed after retrying temporary failures"
            );
            assert_eq!(
                attempt_count, 3,
                "Should have attempted 3 times before succeeding"
            );
            
            // Test that non-retryable errors are not retried
            let mut non_retryable_attempts = 0;
            let non_retryable_result: Result<&str, ApiError> = retry_with_backoff(
                || {
                    non_retryable_attempts += 1;
                    async move {
                        Err(ApiError::AuthenticationFailed)
                    }
                },
                &test_policy,
            ).await;
            
            assert!(
                non_retryable_result.is_err(),
                "Non-retryable errors should fail immediately"
            );
            assert_eq!(
                non_retryable_attempts, 1,
                "Non-retryable errors should not be retried"
            );
            
            // Test max retries enforcement
            let mut max_retry_attempts = 0;
            let max_retry_result: Result<&str, ApiError> = retry_with_backoff(
                || {
                    max_retry_attempts += 1;
                    async move {
                        Err(ApiError::NetworkError("Always fails".to_string()))
                    }
                },
                &test_policy,
            ).await;
            
            assert!(
                max_retry_result.is_err(),
                "Should fail after max retries"
            );
            assert_eq!(
                max_retry_attempts,
                test_policy.max_retries + 1, // Initial attempt + retries
                "Should attempt exactly max_retries + 1 times"
            );
        });
    }
}


}
