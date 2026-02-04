# 🎙️ ClawToTalk

A browser-based voice interface for AI assistants. Talk to Claude using your voice, get spoken responses back.

**Push-to-talk, toggle, or hands-free auto-detection** — works on desktop and mobile.

![ClawToTalk Demo](https://img.shields.io/badge/Status-Working-brightgreen)

## Features

- 🎤 **Three Recording Modes**
  - **Hold** — Push-to-talk (hold button while speaking)
  - **Toggle** — Tap to start, tap again to stop
  - **Auto** — Voice activity detection, auto-stops on silence
- 🗣️ **Natural Voice Responses** — ElevenLabs text-to-speech
- 💬 **Conversation Memory** — Maintains context within session
- 📱 **Mobile Friendly** — Works on iOS Safari and Android Chrome
- ⚡ **Fast** — ~2-3 second total latency
- 🎨 **Customizable** — Change bot name, voice, and personality

## How It Works

```
[Your Voice] → [Whisper STT] → [Claude AI] → [ElevenLabs TTS] → [Speaker]
     🎤              📝              🧠              🔊            🔈
```

1. Record audio in browser using MediaRecorder API
2. Send to OpenAI Whisper for speech-to-text
3. Claude generates a conversational response
4. ElevenLabs converts response to natural speech
5. Audio plays back in browser

## Quick Start

### Prerequisites

You'll need API keys from:
- [OpenAI](https://platform.openai.com/api-keys) — for Whisper speech-to-text
- [Anthropic](https://console.anthropic.com/) — for Claude
- [ElevenLabs](https://elevenlabs.io/) — for text-to-speech

### Installation

```bash
# Clone the repository
git clone https://github.com/ktamas77/clawdtotalk.git
cd clawdtotalk

# Install dependencies
npm install

# Copy environment template
cp .env.example .env

# Edit .env with your API keys
nano .env  # or use your preferred editor

# Start the server
npm start
```

### Open in Browser

Navigate to `http://localhost:3333`

For mobile access on your local network:
- Find your computer's local IP (e.g., `192.168.1.100`)
- Open `http://192.168.1.100:3333` on your phone

## Configuration

Edit `.env` to customize:

```bash
# Required API Keys
OPENAI_API_KEY=sk-proj-...
ANTHROPIC_API_KEY=sk-ant-...
ELEVENLABS_API_KEY=...
ELEVENLABS_VOICE_ID=SAz9YHcvj6GT2YYXdXww

# Server
PORT=3333

# Customization
BOT_NAME=Gerty
BOT_SYSTEM_PROMPT=You are a friendly AI assistant...
```

### Choosing a Voice

Browse voices at [ElevenLabs Voice Library](https://elevenlabs.io/voice-library). 

Popular options:
- `SAz9YHcvj6GT2YYXdXww` — River (calm, neutral)
- `21m00Tcm4TlvDq8ikWAM` — Rachel (clear, American)
- `AZnzlk1XvdvUeBnXmlld` — Domi (expressive)

## Usage

### Recording Modes

| Mode | How to Use | Best For |
|------|------------|----------|
| **Hold** | Press and hold the button while speaking, release when done | Quick questions, noisy environments |
| **Toggle** | Tap to start recording, tap again to stop | Longer messages |
| **Auto** | Tap once, speak naturally, stops after 1.5s of silence | Hands-free, continuous conversation |

### Tips

- Speak clearly and at a normal pace
- Wait for the response to finish before speaking again
- Use "Clear conversation" to reset context
- On iOS, use "Hold" mode for best reliability

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | GET | Web interface |
| `/health` | GET | Health check |
| `/api/config` | GET | Get bot configuration |
| `/api/voice` | POST | Process voice (multipart form with `audio` field) |
| `/api/clear` | POST | Clear conversation history |

## Deployment

### Local Network (Recommended for Home Use)

Just run `npm start` — accessible from any device on your network.

### Tailscale (Access from Anywhere)

```bash
# Install Tailscale on your server
# Then expose the port:
tailscale serve 3333
```

Access via `https://your-machine.tailnet-name.ts.net/`

### Production (Docker)

```dockerfile
FROM node:22-alpine
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production
COPY . .
EXPOSE 3333
CMD ["npm", "start"]
```

## Troubleshooting

### "Microphone access denied"
- Check browser permissions for the site
- On iOS, ensure Safari has microphone access in Settings

### "Invalid file format" error
- This can happen on iOS Safari — try refreshing the page
- Use "Hold" mode which is more reliable on mobile

### High latency
- ElevenLabs is usually the bottleneck
- Try the `eleven_turbo_v2` model for faster responses
- Check your internet connection

### No audio playback
- Check device volume
- Some browsers block autoplay — interact with the page first

## Tech Stack

- **Backend:** Node.js, Express
- **STT:** OpenAI Whisper API
- **LLM:** Anthropic Claude
- **TTS:** ElevenLabs
- **Frontend:** Vanilla HTML/CSS/JS, Web Audio API

## Contributing

Contributions welcome! Please open an issue or PR.

Ideas for improvements:
- Wake word detection ("Hey Assistant")
- Persistent conversation history
- Multiple voice options in UI
- Streaming responses
- WebSocket for lower latency

## License

MIT License — feel free to use, modify, and distribute.

## Credits

Built with ❤️ by [Tamas Kalman](https://github.com/ktamas77) and Gerty 🤖

---

*ClawToTalk is part of the OpenClaw ecosystem — giving AI assistants a voice.*
