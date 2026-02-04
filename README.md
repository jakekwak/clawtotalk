# 🎙️ ClawToTalk

Voice interface for Gerty - push-to-talk in your browser.

## Quick Start

```bash
npm install
npm start
```

Open http://localhost:3333

## How to Use

1. Hold the big button and speak
2. Release when done
3. Wait for Gerty's response
4. Repeat!

## Architecture

```
Browser → [Audio Recording]
    ↓
Server → [Whisper STT] → [Claude] → [ElevenLabs TTS]
    ↓
Browser ← [Audio Playback]
```

## Tech Stack

- **Frontend:** Vanilla HTML/JS with MediaRecorder API
- **Backend:** Node.js + Express
- **STT:** OpenAI Whisper
- **LLM:** Claude (Anthropic)
- **TTS:** ElevenLabs (River voice)

## Environment Variables

Copy `.env.example` to `.env` and fill in:

```
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
ELEVENLABS_API_KEY=sk_...
ELEVENLABS_VOICE_ID=SAz9YHcvj6GT2YYXdXww
PORT=3333
```

## PWA Support

Can be installed as a Progressive Web App on mobile devices. Add to home screen for app-like experience.

## Future Ideas

- Wake word detection ("Hey Gerty")
- Conversation history persistence
- Multiple voice options
- Push notifications for async responses
- Integration with OpenClaw for full capabilities

## License

MIT - Squidcode LLC
