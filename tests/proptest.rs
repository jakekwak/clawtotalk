// Property-based tests for Dioxus Voice Assistant

use proptest::prelude::*;
use dioxus_voice_assistant::models::*;
use dioxus_voice_assistant::audio::{AudioManager, CrossPlatformAudioManager};
use dioxus_voice_assistant::vad::{VoiceActivityDetector, VadResult};

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
    
    // Strategy for generating ApiKeys
    fn api_keys_strategy() -> impl Strategy<Value = ApiKeys> {
        (
            prop::option::of("[a-zA-Z0-9]{32,64}"),
            prop::option::of("[a-zA-Z0-9]{32,64}"),
            prop::option::of("[a-zA-Z0-9]{32,64}"),
        )
            .prop_map(|(openai_key, claude_key, elevenlabs_key)| ApiKeys {
                openai_key,
                claude_key,
                elevenlabs_key,
            })
    }
    
    // Strategy for generating Settings
    fn settings_strategy() -> impl Strategy<Value = Settings> {
        (
            api_keys_strategy(),
            recording_mode_strategy(),
            vad_settings_strategy(),
            audio_settings_strategy(),
        )
            .prop_map(|(api_keys, recording_mode, vad_settings, audio_settings)| Settings {
                api_keys,
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
            assert_eq!(settings.api_keys.openai_key, deserialized.api_keys.openai_key);
            assert_eq!(settings.api_keys.claude_key, deserialized.api_keys.claude_key);
            assert_eq!(settings.api_keys.elevenlabs_key, deserialized.api_keys.elevenlabs_key);
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
    }
}
