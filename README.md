# 🎙️ Dioxus Voice Assistant

A cross-platform native voice assistant application built with Dioxus 0.7. Talk using your voice, get spoken responses back on Windows, macOS, Android, and iOS.

**Push-to-talk, toggle, or hands-free auto-detection** — works on all platforms with native performance.

<p align="center">
  <img src="screenshot.png" alt="Dioxus Voice Assistant Screenshot" width="300">
</p>

## ✨ Features

- 🎤 **Three Recording Modes**
  - **Hold** — Push-to-talk (hold button while speaking)
  - **Toggle** — Tap to start, tap again to stop
  - **Auto** — Voice activity detection, auto-stops on silence
- 🗣️ **Natural Voice Responses** — ElevenLabs text-to-speech
- 🤖 **Flexible Backend**
  - **OpenClaw Mode** — Full AI assistant with tools, memory, integrations
  - **Direct Mode** — Simple Claude API for standalone use
  - **Mock Mode** — Test without API keys
- 📱 **Cross-Platform** — Windows, macOS, Android, iOS
- ⚡ **Fast** — Startup < 3s, Recording latency < 100ms
- 🔒 **Secure** — API keys stay on server, never exposed to client

## 🏗️ Architecture

This is a **client-server application**:

- **Client** (this repo): Native Dioxus app that handles UI, audio recording/playback, and VAD
- **Server**: Node.js backend that handles STT (Whisper), AI (Claude/OpenClaw), and TTS (ElevenLabs)

```
[Client App] ←→ HTTP/REST ←→ [Server] ←→ [OpenAI/Claude/ElevenLabs APIs]
```

The server can run on:
- Your local machine (localhost)
- A Mac Mini on your network
- A VPS in the cloud
- Accessible via Tailscale VPN or public URL

## 🚀 Quick Start

### 5-Minute Test (No API Keys Required)

```bash
# 1. Run automated tests
./quick_test.sh

# 2. Start mock server
echo "MOCK_MODE=true" > .env
npm install && npm start

# 3. Run client app (in another terminal)
cargo run --release
```

**See [QUICKSTART.md](QUICKSTART.md) for detailed step-by-step instructions.**

### For Production Use

See the [Local Testing Guide](LOCAL_TESTING_GUIDE.md) for comprehensive setup instructions.

## 📚 Documentation

- **[Local Testing Guide](LOCAL_TESTING_GUIDE.md)** - Complete setup and testing instructions
- **[Architecture](ARCHITECTURE.md)** - System design and components
- **[Build Guide](BUILD.md)** - Platform-specific build instructions
- **[Final Verification Report](FINAL_VERIFICATION_REPORT.md)** - Test results and status

### Platform-Specific Guides

- **[Windows Optimization](WINDOWS_OPTIMIZATION.md)**
- **[macOS Optimization](MACOS_OPTIMIZATION.md)**
- **[Android Optimization](ANDROID_OPTIMIZATION.md)**
- **[iOS Optimization](IOS_OPTIMIZATION.md)**

## 🧪 Testing

### Run All Tests

```bash
# Quick test script (recommended)
./quick_test.sh

# Or manually:
cargo test --all-features
```

### Test Categories

```bash
# Unit tests (59 tests)
cargo test --lib

# Integration tests (16 tests)
cargo test --test integration_tests

# Property-based tests (10 tests)
cargo test --test proptest

# Performance tests (5 tests)
cargo test --test performance_proptest
```

**Total: 90 tests, all passing ✅**

## 🔧 Server Setup

### Option 1: Mock Server (No API Keys)

Perfect for testing the client app:

```bash
# Create .env
echo "MOCK_MODE=true
PORT=3333
BOT_NAME=TestBot" > .env

# Install and run
npm install
npm start
```

### Option 2: Real Server (With API Keys)

```bash
# Copy and edit .env
cp .env.example .env
nano .env  # Add your API keys

# Install and run
npm install
npm start
```

Required API keys:
- [OpenAI API Key](https://platform.openai.com/api-keys) — for Whisper STT
- [ElevenLabs API Key](https://elevenlabs.io/) — for TTS
- [Anthropic API Key](https://console.anthropic.com/) — for Claude (if not using OpenClaw)

## 🖥️ Client App

### Desktop (macOS/Windows/Linux)

```bash
# Development
cargo run

# Release build
cargo build --release
./target/release/dioxus-voice-assistant
```

### Mobile

#### iOS
```bash
cd ios
./build_ios.sh simulator
# Open in Xcode and run
```

#### Android
```bash
cd android
./build_android.sh
adb install ../target/android/release/dioxus-voice-assistant.apk
```

See platform-specific guides for detailed instructions.

## 📊 Performance

All performance requirements met:

- ✅ **App startup time**: < 3 seconds (avg ~100ms)
- ✅ **Recording latency**: < 100ms (avg 5-50ms)
- ✅ **Memory optimized**: Buffer pooling, pagination
- ✅ **Battery optimized**: Efficient audio processing
- ✅ **Network optimized**: Audio compression, connection pooling

## 🔌 Server API

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Health check and mode info |
| `/api/config` | GET | Get bot configuration |
| `/api/voice` | POST | Process voice (multipart form) |
| `/api/clear` | POST | Clear conversation |

## 🌐 Network Options

### Local Network
```
Server URL: http://192.168.1.100:3333
Connection Type: LocalNetwork
```

### Tailscale VPN
```
Server URL: http://100.x.x.x:3333
Connection Type: Tailscale
```

### Public URL (Cloudflare Tunnel)
```
Server URL: https://your-domain.com
Connection Type: PublicUrl
```

## 🛠️ Tech Stack

### Client
- **Framework**: Dioxus 0.7
- **Language**: Rust
- **Audio**: CPAL (cross-platform audio library)
- **HTTP**: reqwest
- **State Management**: Dioxus Signals

### Server
- **Runtime**: Node.js
- **Framework**: Express
- **STT**: OpenAI Whisper API
- **LLM**: Claude (via OpenClaw or direct)
- **TTS**: ElevenLabs

## 🧩 Project Structure

```
.
├── src/
│   ├── audio.rs          # Audio recording/playback
│   ├── api.rs            # Server API client
│   ├── vad.rs            # Voice activity detection
│   ├── recording.rs      # Recording modes
│   ├── ui/               # Dioxus UI components
│   └── platform/         # Platform-specific code
├── tests/
│   ├── proptest.rs       # Property-based tests
│   ├── integration_tests.rs
│   └── performance_proptest.rs
├── server.js             # Node.js backend
├── quick_test.sh         # Quick test script
└── LOCAL_TESTING_GUIDE.md
```

## 🤝 Contributing

Contributions welcome! Areas for improvement:

- [ ] Wake word detection
- [ ] Streaming responses
- [ ] Multiple voice options in UI
- [ ] WebSocket for real-time communication
- [ ] Offline mode with local models

## 📄 License

MIT License — use, modify, distribute freely.

## 🙏 Credits

Built with Dioxus 0.7 and powered by OpenAI, Anthropic, and ElevenLabs APIs.

Part of the [OpenClaw](https://github.com/openclaw/openclaw) ecosystem.

---

**Ready to test?** Run `./quick_test.sh` or see [LOCAL_TESTING_GUIDE.md](LOCAL_TESTING_GUIDE.md) for detailed instructions.
