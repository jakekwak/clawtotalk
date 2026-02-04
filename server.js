import 'dotenv/config';
import express from 'express';
import multer from 'multer';
import { OpenAI } from 'openai';
import { createReadStream } from 'fs';
import { unlink, rename } from 'fs/promises';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Configuration from environment
const config = {
  port: process.env.PORT || 3333,
  botName: process.env.BOT_NAME || 'Gerty',
  sessionId: process.env.VOICE_SESSION_ID || 'voice-clawtotalk',
  useOpenClaw: process.env.USE_OPENCLAW !== 'false', // Default to true
};

// Validate required environment variables
const requiredEnvVars = ['OPENAI_API_KEY', 'ELEVENLABS_API_KEY', 'ELEVENLABS_VOICE_ID'];
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

// Conversation history for direct Claude mode
const conversationHistory = [];

app.use(express.static(join(__dirname, 'public')));
app.use(express.json());

// Health check
app.get('/health', (req, res) => {
  res.json({ status: 'ok', mode: config.useOpenClaw ? 'openclaw' : 'direct', time: new Date().toISOString() });
});

// Get bot config (for frontend)
app.get('/api/config', (req, res) => {
  res.json({ botName: config.botName, mode: config.useOpenClaw ? 'openclaw' : 'direct' });
});

// Get response from OpenClaw (the real Gerty!)
async function getOpenClawResponse(userText) {
  const startTime = Date.now();
  
  try {
    // Call openclaw agent CLI
    const cmd = `openclaw agent --message ${JSON.stringify(userText)} --session-id ${config.sessionId} --json`;
    console.log(`[OpenClaw] Running: ${cmd.substring(0, 100)}...`);
    
    const { stdout, stderr } = await execAsync(cmd, { 
      timeout: 120000, // 2 minute timeout
      maxBuffer: 10 * 1024 * 1024, // 10MB buffer
    });
    
    if (stderr) {
      console.warn('[OpenClaw] stderr:', stderr);
    }
    
    const result = JSON.parse(stdout);
    const elapsed = Date.now() - startTime;
    console.log(`[OpenClaw] Response in ${elapsed}ms, status: ${result.status}`);
    
    if (result.status !== 'ok' || !result.result?.payloads?.length) {
      throw new Error(result.error || 'No response from OpenClaw');
    }
    
    // Get the text response
    const responseText = result.result.payloads
      .map(p => p.text)
      .filter(Boolean)
      .join('\n');
    
    return responseText || 'I processed your request but have nothing to say.';
    
  } catch (error) {
    console.error('[OpenClaw Error]', error);
    throw error;
  }
}

// Main voice endpoint
app.post('/api/voice', upload.single('audio'), async (req, res) => {
  const startTime = Date.now();
  
  try {
    if (!req.file) {
      return res.status(400).json({ error: 'No audio file provided' });
    }

    console.log(`[${new Date().toISOString()}] Received audio: ${req.file.size} bytes, originalname: ${req.file.originalname}`);

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

    // Step 2: Get response from OpenClaw (the real Gerty!)
    let assistantText;
    if (config.useOpenClaw) {
      assistantText = await getOpenClawResponse(userText);
    } else {
      // Direct Claude mode
      if (!process.env.ANTHROPIC_API_KEY) {
        throw new Error('ANTHROPIC_API_KEY required for direct mode (USE_OPENCLAW=false)');
      }
      const Anthropic = (await import('@anthropic-ai/sdk')).default;
      const anthropic = new Anthropic({ apiKey: process.env.ANTHROPIC_API_KEY });
      
      // Add to conversation history for context
      conversationHistory.push({ role: 'user', content: userText });
      while (conversationHistory.length > 20) conversationHistory.shift();
      
      const systemPrompt = process.env.BOT_SYSTEM_PROMPT || 
        `You are ${config.botName}, a friendly AI assistant. Keep responses conversational and concise for voice. Don't use markdown or formatting.`;
      
      const response = await anthropic.messages.create({
        model: 'claude-sonnet-4-20250514',
        max_tokens: 500,
        system: systemPrompt,
        messages: conversationHistory,
      });
      assistantText = response.content[0].text;
      conversationHistory.push({ role: 'assistant', content: assistantText });
    }
    
    console.log(`[Response] "${assistantText.substring(0, 100)}..."`);

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
    console.log(`[TTS] Audio size: ${audioBuffer.length} bytes`);
    console.log(`[Done] ${elapsed}ms total`);

    res.json({
      userText,
      assistantText,
      audio: audioBuffer.toString('base64'),
      elapsed,
      botName: config.botName,
      mode: config.useOpenClaw ? 'openclaw' : 'direct',
    });

  } catch (error) {
    console.error('[Error]', error);
    if (req.file) {
      await unlink(req.file.path).catch(() => {});
    }
    res.status(500).json({ error: error.message });
  }
});

// Clear conversation
app.post('/api/clear', async (req, res) => {
  console.log('[Session reset requested]');
  if (config.useOpenClaw) {
    // OpenClaw sessions persist across voice - could send /new command if needed
    res.json({ ok: true, note: 'OpenClaw session persists - use /new via another channel to reset' });
  } else {
    // Clear direct mode conversation history
    conversationHistory.length = 0;
    res.json({ ok: true });
  }
});

app.listen(config.port, () => {
  console.log(`🎙️ ${config.botName} voice server running on http://localhost:${config.port}`);
  console.log(`   Mode: ${config.useOpenClaw ? 'OpenClaw (full Gerty!)' : 'Direct Claude'}`);
  console.log(`   Session: ${config.sessionId}`);
});
