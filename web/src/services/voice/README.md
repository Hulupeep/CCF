# Voice Recognition Services

This directory contains the core services for voice recognition and speaker identification.

## Services

### VoiceProfileService
Manages voice enrollment, speaker identification, and profile management.

**Key Features:**
- Enroll users with voice samples
- Identify speakers with ≥85% confidence
- Support up to 10 voice profiles
- Anonymous mode fallback
- Privacy-first (local storage only)

**Usage:**
```typescript
const service = new VoiceProfileService();

// Enroll user
const profile = await service.enrollUser('Alice', samples);

// Identify from audio
const identification = await service.identifyUser(audioData);
if (identification.confidence >= 0.85) {
  console.log('Identified:', identification.userId);
}
```

### AudioCapture
Handles microphone access and audio recording.

**Key Features:**
- Browser compatibility checks
- Microphone permission handling
- Audio device enumeration
- Recording controls
- Audio level monitoring

**Usage:**
```typescript
const capture = new AudioCapture();

// Check support
if (!capture.isSupported()) {
  console.error('Not supported');
}

// Record audio
await capture.startRecording();
// ... user speaks ...
const audioData = await capture.stopRecording();
```

### WhisperAPI
Integration with OpenAI Whisper for speech-to-text.

**Key Features:**
- Audio transcription
- Detailed segmentation
- Language translation
- API key management

**Usage:**
```typescript
const whisper = new WhisperAPI({ apiKey: 'your-key' });

const text = await whisper.transcribeAudio(audioData);
console.log('Transcript:', text);
```

## Contract Compliance

All services implement the following contracts:

- **I-VOICE-001**: Speaker identification ≥85%
- **I-VOICE-002**: Privacy protection
- **I-VOICE-005**: Multi-user support (10 profiles)
- **I-VOICE-006**: Anonymous mode fallback

## File Structure

```
services/voice/
├── VoiceProfileService.ts    # Speaker identification
├── AudioCapture.ts            # Audio recording
├── WhisperAPI.ts              # Speech-to-text
└── README.md                  # This file
```

## Testing

Run tests:
```bash
npm test -- voice-recognition
```

## Documentation

See [Voice Recognition Guide](/docs/guides/voice-recognition-guide.md) for complete documentation.
