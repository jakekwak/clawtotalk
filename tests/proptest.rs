// Property-based tests for Dioxus Voice Assistant

use proptest::prelude::*;
use dioxus_voice_assistant::models::*;

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
    }
}
