import 'dotenv/config';
import express from 'express';
import multer from 'multer';
import { OpenAI } from 'openai';
import Anthropic from '@anthropic-ai/sdk';
import { createReadStream } from 'fs';
import { unlink } from 'fs/promises';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const app = express();
const upload = multer({ dest: 'uploads/' });

const openai = new OpenAI({ apiKey: process.env.OPENAI_API_KEY });
const anthropic = new Anthropic({ apiKey: process.env.ANTHROPIC_API_KEY });

// Conversation history (in-memory for now)
const conversationHistory = [];
const MAX_HISTORY = 20;

// System prompt for Gerty
const SYSTEM_PROMPT = `You are Gerty, a friendly and capable AI assistant. You're speaking via voice, so keep responses conversational and concise. 
- Don't use markdown, bullet points, or formatting - just natural speech
- Keep responses brief unless asked for detail (1-3 sentences typically)
- Be warm and personable, like talking to a friend
- You can ask follow-up questions to clarify
- If you don't know something, say so naturally`;

app.use(express.static(join(__dirname, 'public')));
app.use(express.json());

// Health check
app.get('/health', (req, res) => {
  res.json({ status: 'ok', time: new Date().toISOString() });
});

// Main voice endpoint
app.post('/api/voice', upload.single('audio'), async (req, res) => {
  const startTime = Date.now();
  
  try {
    if (!req.file) {
      return res.status(400).json({ error: 'No audio file provided' });
    }

    console.log(`[${new Date().toISOString()}] Received audio: ${req.file.size} bytes`);

    // Step 1: Transcribe with Whisper
    const transcription = await openai.audio.transcriptions.create({
      file: createReadStream(req.file.path),
      model: 'whisper-1',
      language: 'en',
    });

    const userText = transcription.text.trim();
    console.log(`[STT] "${userText}"`);

    // Clean up uploaded file
    await unlink(req.file.path).catch(() => {});

    if (!userText) {
      return res.status(400).json({ error: 'Could not transcribe audio' });
    }

    // Step 2: Add to history and get Claude response
    conversationHistory.push({ role: 'user', content: userText });
    
    // Trim history if too long
    while (conversationHistory.length > MAX_HISTORY) {
      conversationHistory.shift();
    }

    const response = await anthropic.messages.create({
      model: 'claude-sonnet-4-20250514',
      max_tokens: 500,
      system: SYSTEM_PROMPT,
      messages: conversationHistory,
    });

    const assistantText = response.content[0].text;
    console.log(`[Claude] "${assistantText}"`);

    conversationHistory.push({ role: 'assistant', content: assistantText });

    // Step 3: Convert to speech with ElevenLabs
    const ttsResponse = await fetch(
      `https://api.elevenlabs.io/v1/text-to-speech/${process.env.ELEVENLABS_VOICE_ID}`,
      {
        method: 'POST',
        headers: {
          'Accept': 'audio/mpeg',
          'Content-Type': 'application/json',
          'xi-api-key': process.env.ELEVENLABS_API_KEY,
        },
        body: JSON.stringify({
          text: assistantText,
          model_id: 'eleven_turbo_v2_5',
          voice_settings: {
            stability: 0.5,
            similarity_boost: 0.75,
          },
        }),
      }
    );

    if (!ttsResponse.ok) {
      const err = await ttsResponse.text();
      console.error('[TTS Error]', err);
      return res.status(500).json({ error: 'TTS failed', details: err });
    }

    const audioBuffer = Buffer.from(await ttsResponse.arrayBuffer());
    const elapsed = Date.now() - startTime;
    console.log(`[Done] ${elapsed}ms total`);

    // Return both transcript and audio
    res.json({
      userText,
      assistantText,
      audio: audioBuffer.toString('base64'),
      elapsed,
    });

  } catch (error) {
    console.error('[Error]', error);
    // Clean up file on error
    if (req.file) {
      await unlink(req.file.path).catch(() => {});
    }
    res.status(500).json({ error: error.message });
  }
});

// Clear conversation history
app.post('/api/clear', (req, res) => {
  conversationHistory.length = 0;
  console.log('[History cleared]');
  res.json({ ok: true });
});

const PORT = process.env.PORT || 3333;
app.listen(PORT, () => {
  console.log(`🎙️ ClawToTalk running on http://localhost:${PORT}`);
});
