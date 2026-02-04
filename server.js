import 'dotenv/config';
import express from 'express';
import multer from 'multer';
import { OpenAI } from 'openai';
import Anthropic from '@anthropic-ai/sdk';
import { createReadStream } from 'fs';
import { unlink, rename } from 'fs/promises';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Configuration from environment
const config = {
  port: process.env.PORT || 3333,
  botName: process.env.BOT_NAME || 'Assistant',
  systemPrompt: process.env.BOT_SYSTEM_PROMPT || 
    `You are a friendly and capable AI assistant. You're speaking via voice, so keep responses conversational and concise. Don't use markdown or formatting - just natural speech. Keep responses brief unless asked for detail.`,
};

// Validate required environment variables
const requiredEnvVars = ['OPENAI_API_KEY', 'ANTHROPIC_API_KEY', 'ELEVENLABS_API_KEY', 'ELEVENLABS_VOICE_ID'];
for (const envVar of requiredEnvVars) {
  if (!process.env[envVar]) {
    console.error(`❌ Missing required environment variable: ${envVar}`);
    console.error('   Copy .env.example to .env and fill in your API keys.');
    process.exit(1);
  }
}

const app = express();
const upload = multer({ dest: 'uploads/' });

const openai = new OpenAI({ apiKey: process.env.OPENAI_API_KEY });
const anthropic = new Anthropic({ apiKey: process.env.ANTHROPIC_API_KEY });

// Conversation history (in-memory)
const conversationHistory = [];
const MAX_HISTORY = 20;

app.use(express.static(join(__dirname, 'public')));
app.use(express.json());

// Health check
app.get('/health', (req, res) => {
  res.json({ status: 'ok', time: new Date().toISOString() });
});

// Get bot config (for frontend)
app.get('/api/config', (req, res) => {
  res.json({ botName: config.botName });
});

// Main voice endpoint
app.post('/api/voice', upload.single('audio'), async (req, res) => {
  const startTime = Date.now();
  
  try {
    if (!req.file) {
      return res.status(400).json({ error: 'No audio file provided' });
    }

    console.log(`[${new Date().toISOString()}] Received audio: ${req.file.size} bytes, originalname: ${req.file.originalname}, mimetype: ${req.file.mimetype}`);

    // Step 1: Transcribe with Whisper
    const origName = req.file.originalname || 'recording.webm';
    const ext = origName.split('.').pop() || 'webm';
    const audioPath = req.file.path + '.' + ext;
    await rename(req.file.path, audioPath);
    
    console.log(`[Audio] ${origName} -> ${audioPath} (${req.file.size} bytes)`);
    
    const transcription = await openai.audio.transcriptions.create({
      file: createReadStream(audioPath),
      model: 'whisper-1',
      language: 'en',
    });
    
    req.file.path = audioPath;

    const userText = transcription.text.trim();
    console.log(`[STT] "${userText}"`);

    // Clean up uploaded file
    await unlink(req.file.path).catch(() => {});

    if (!userText) {
      return res.status(400).json({ error: 'Could not transcribe audio' });
    }

    // Step 2: Add to history and get Claude response
    conversationHistory.push({ role: 'user', content: userText });
    
    while (conversationHistory.length > MAX_HISTORY) {
      conversationHistory.shift();
    }

    const response = await anthropic.messages.create({
      model: 'claude-sonnet-4-20250514',
      max_tokens: 500,
      system: config.systemPrompt,
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

    res.json({
      userText,
      assistantText,
      audio: audioBuffer.toString('base64'),
      elapsed,
      botName: config.botName,
    });

  } catch (error) {
    console.error('[Error]', error);
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

app.listen(config.port, () => {
  console.log(`🎙️ ${config.botName} voice server running on http://localhost:${config.port}`);
});
