# Voice Control Setup Guide

**Feature:** Voice Commands with NLP
**Time:** 20 minutes
**Difficulty:** Intermediate

## Overview

Set up speech-to-text voice commands with natural language processing for hands-free robot control.

## Step 1: Choose Speech API

### Option A: Web Speech API (Free, Browser-only)

```typescript
// No setup required - built into modern browsers
const recognition = new (window.SpeechRecognition || window.webkitSpeechRecognition)();
```

**Pros:**
- Free
- No API keys
- Works offline
- Good accuracy

**Cons:**
- Browser-only (not for mobile apps)
- Limited languages
- No custom vocabulary

### Option B: Google Cloud Speech-to-Text (Paid, Production)

1. Go to [Google Cloud Console](https://console.cloud.google.com)
2. Enable "Speech-to-Text API"
3. Create API key
4. Add to `.env.local`:

```bash
NEXT_PUBLIC_SPEECH_API_KEY=AIza...
NEXT_PUBLIC_SPEECH_API_ENDPOINT=https://speech.googleapis.com/v1/speech:recognize
```

**Pros:**
- High accuracy
- Many languages
- Custom vocabulary
- Works everywhere

**Cons:**
- Paid ($0.006 per 15 seconds)
- Requires API key
- Network dependent

### Option C: OpenAI Whisper (Self-hosted)

```bash
pip install openai-whisper
whisper --model base --language en input.wav
```

**Pros:**
- Free (self-hosted)
- Excellent accuracy
- Offline capable
- Open source

**Cons:**
- Requires GPU for real-time
- Setup complexity
- Large model files

## Step 2: Install Dependencies

```bash
npm install --save \
  @speechly/browser-client \
  natural \
  compromise
```

## Step 3: Configure Voice Control

```typescript
// services/voiceCommands.ts
import { SpeechRecognition } from '@speechly/browser-client';
import natural from 'natural';
import nlp from 'compromise';

export class VoiceCommands {
  private recognition: SpeechRecognition;
  private commands: Map<string, CommandHandler> = new Map();

  constructor() {
    this.recognition = new SpeechRecognition();
    this.setupRecognition();
  }

  private setupRecognition() {
    this.recognition.continuous = true;
    this.recognition.interimResults = true;
    this.recognition.lang = 'en-US';

    this.recognition.onresult = (event) => {
      const last = event.results.length - 1;
      const transcript = event.results[last][0].transcript.toLowerCase();
      const confidence = event.results[last][0].confidence;

      if (event.results[last].isFinal && confidence > 0.7) {
        this.processCommand(transcript);
      }
    };
  }

  registerCommand(pattern: string, handler: CommandHandler) {
    this.commands.set(pattern, handler);
  }

  private async processCommand(transcript: string) {
    // Extract intent using NLP
    const intent = this.parseIntent(transcript);

    // Find matching command
    for (const [pattern, handler] of this.commands) {
      if (this.matchesPattern(transcript, pattern)) {
        await handler(intent.entities);
        return;
      }
    }

    console.warn('No command matched:', transcript);
  }

  private parseIntent(text: string) {
    const doc = nlp(text);

    return {
      action: doc.verbs().first().text(),
      entities: {
        personality: doc.match('#Adjective').out('array'),
        number: doc.values().first().number(),
        game: this.extractGame(text)
      }
    };
  }

  private extractGame(text: string): string | null {
    const games = ['tictactoe', 'tic tac toe', 'chase', 'simon', 'simon says'];
    for (const game of games) {
      if (text.includes(game)) {
        return game.replace(' ', '');
      }
    }
    return null;
  }

  private matchesPattern(text: string, pattern: string): boolean {
    // Simple pattern matching (can be enhanced with regex)
    const keywords = pattern.toLowerCase().split(' ');
    return keywords.every(keyword => text.includes(keyword));
  }

  start() {
    this.recognition.start();
  }

  stop() {
    this.recognition.stop();
  }
}
```

## Step 4: Register Commands

```typescript
// Configure commands
import { voiceCommands } from '@/services/voiceCommands';

// Basic commands
voiceCommands.registerCommand('start drawing', async () => {
  console.log('Starting drawing...');
  // Send command to robot
});

voiceCommands.registerCommand('stop', async () => {
  console.log('Stopping...');
});

voiceCommands.registerCommand('play', async (entities) => {
  const game = entities.game || 'tictactoe';
  console.log(`Starting ${game}...`);
});

// Personality commands
voiceCommands.registerCommand('make personality', async (entities) => {
  const traits = entities.personality;
  console.log('Setting personality:', traits);
});

voiceCommands.registerCommand('set curiosity', async (entities) => {
  const value = entities.number / 100; // "set curiosity to 80" -> 0.8
  personalityStore.updateParameter('curiosity', value);
});

// Start listening
voiceCommands.start();
```

## Step 5: Add Wake Word (Optional)

```typescript
// services/wakeWord.ts
import Porcupine from '@picovoice/porcupine-web';

export async function setupWakeWord() {
  const porcupine = await Porcupine.create(
    'ACCESS_KEY', // Get from picovoice.ai
    ['hey mbot'], // Wake words
    (keyword) => {
      console.log('Wake word detected:', keyword);
      voiceCommands.start();

      // Auto-stop after 10 seconds of silence
      setTimeout(() => voiceCommands.stop(), 10000);
    }
  );

  porcupine.start();
}
```

## Step 6: Multi-Language Support

```typescript
const SUPPORTED_LANGUAGES = {
  'en-US': {
    commands: {
      'start drawing': 'Start drawing',
      'stop': 'Stop'
    }
  },
  'es-ES': {
    commands: {
      'empezar dibujo': 'Start drawing',
      'parar': 'Stop'
    }
  },
  'fr-FR': {
    commands: {
      'commencer dessin': 'Start drawing',
      'arrêter': 'Stop'
    }
  }
};

voiceCommands.setLanguage('es-ES');
```

## Step 7: Test Voice Control

```typescript
// Test in browser console
voiceCommands.start();

// Say commands:
// "Hey mBot, start drawing"
// "Hey mBot, play Tic-Tac-Toe"
// "Hey mBot, set curiosity to eighty"
// "Hey mBot, stop"
```

## Step 8: Add UI Feedback

```typescript
import { VoiceControl } from '@/components/VoiceControl';

function App() {
  const [listening, setListening] = useState(false);
  const [transcript, setTranscript] = useState('');

  return (
    <VoiceControl
      onListeningChange={setListening}
      onTranscript={setTranscript}
      onCommand={(command) => console.log('Command:', command)}
    />
  );
}
```

## Troubleshooting

### Microphone Access Denied

```typescript
// Check permissions
navigator.permissions.query({ name: 'microphone' }).then((result) => {
  if (result.state === 'denied') {
    alert('Please allow microphone access');
  }
});
```

### Poor Recognition Accuracy

1. Reduce background noise
2. Speak clearly and slowly
3. Use a better microphone
4. Increase confidence threshold:

```typescript
this.recognition.onresult = (event) => {
  const confidence = event.results[last][0].confidence;
  if (confidence > 0.8) { // Increase from 0.7
    // Process command
  }
};
```

### Commands Not Matching

Add more variations:

```typescript
voiceCommands.registerCommand('start drawing', handler);
voiceCommands.registerCommand('begin drawing', handler);
voiceCommands.registerCommand('draw', handler);
```

---

**Last Updated:** 2026-02-01
