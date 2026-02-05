# Dioxus Voice Assistant Architecture

## Overview

This document describes the architecture of the Dioxus Voice Assistant, a cross-platform native application built with Rust and Dioxus 0.7.

## Project Structure

```
dioxus-voice-assistant/
├── src/
│   ├── main.rs              # Application entry point
│   ├── audio.rs             # Audio management traits and implementations
│   ├── api.rs               # API client traits and implementations
│   ├── models.rs            # Data models and types
│   ├── state.rs             # Application state management
│   ├── vad.rs               # Voice Activity Detection
│   └── error.rs             # Error types and handling
├── tests/
│   └── proptest.rs          # Property-based tests
├── Cargo.toml               # Rust dependencies
├── Dioxus.toml              # Dioxus configuration
├── build.rs                 # Platform-specific build script
└── .cargo/
    └── config.toml          # Cross-platform build configuration
```

## Core Components

### 1. Audio Management (`audio.rs`)

**AudioManager Trait**: Defines the interface for cross-platform audio operations
- `start_recording()`: Begin audio capture
- `stop_recording()`: End capture and return audio data
- `play_audio()`: Play audio data
- `get_audio_level()`: Get current audio level
- `request_permissions()`: Request platform-specific permissions

**CrossPlatformAudioManager**: Implementation using CPAL for cross-platform audio

### 2. API Clients (`api.rs`)

**SpeechToText Trait**: Interface for speech-to-text services
- `transcribe()`: Convert audio to text

**AiAssistant Trait**: Interface for AI response generation
- `generate_response()`: Generate AI response from prompt

**TextToSpeech Trait**: Interface for text-to-speech services
- `synthesize()`: Convert text to audio

**Implementations**:
- `WhisperClient`: OpenAI Whisper integration
- `ClaudeClient`: Claude/OpenClaw integration
- `ElevenLabsClient`: ElevenLabs TTS integration

### 3. Voice Activity Detection (`vad.rs`)

**VoiceActivityDetector**: Real-time speech detection
- Energy-based detection algorithm
- Configurable threshold and silence duration
- Frame-by-frame analysis

### 4. Data Models (`models.rs`)

**Core Types**:
- `Message`: Conversation message with metadata
- `MessageType`: User, Assistant, System, Error
- `RecordingMode`: Hold, Toggle, Auto
- `Settings`: Application configuration
- `AppStatus`: Current application state
- `AudioLevel`: Audio level information

### 5. State Management (`state.rs`)

**AppState**: Global application state using Dioxus signals
- Recording mode and status
- Conversation history
- Settings
- Current status

### 6. Error Handling (`error.rs`)

**Error Types**:
- `AudioError`: Audio-related errors
- `ApiError`: API communication errors
- `AppError`: Application-level errors
- `RecoveryAction`: Error recovery strategies

## Platform Support

### Desktop
- **Windows**: WASAPI audio backend
- **macOS**: CoreAudio backend
- **Linux**: ALSA backend

### Mobile
- **Android**: OpenSLES audio, NDK integration
- **iOS**: AVFoundation, AudioToolbox

### Web (Optional)
- Web Audio API
- MediaRecorder API

## Build Configuration

### Cross-Platform Targets

```bash
# Desktop
x86_64-pc-windows-msvc
x86_64-apple-darwin
aarch64-apple-darwin

# Mobile
aarch64-linux-android
armv7-linux-androideabi
aarch64-apple-ios
x86_64-apple-ios
```

### Platform-Specific Dependencies

- **Android**: JNI, NDK, OpenSLES
- **iOS**: Objective-C runtime, AVFoundation
- **Windows**: ole32, winmm
- **macOS**: CoreAudio, AudioUnit

## Testing Strategy

### Unit Tests
- Component-specific functionality
- Edge cases and error conditions
- Platform-specific behavior

### Property-Based Tests (Proptest)
- Universal correctness properties
- Randomized input testing
- System invariants

### Integration Tests
- Full workflow testing
- Cross-component interactions
- Platform compatibility

## Dependencies

### Core
- `dioxus 0.7`: UI framework
- `dioxus-signals`: State management
- `tokio`: Async runtime
- `cpal`: Cross-platform audio

### API & Networking
- `reqwest`: HTTP client
- `serde`: Serialization

### Testing
- `proptest`: Property-based testing
- `tokio-test`: Async testing

## Next Steps

This is the foundation for the Dioxus Voice Assistant. Subsequent tasks will implement:
1. Core data models and state management
2. Audio system with CPAL
3. Voice Activity Detection
4. Recording modes (Hold, Toggle, Auto)
5. API clients (Whisper, Claude, ElevenLabs)
6. User interface components
7. Error handling and recovery
8. Platform-specific optimizations
9. Performance tuning
10. Integration testing

See `tasks.md` for the complete implementation plan.
