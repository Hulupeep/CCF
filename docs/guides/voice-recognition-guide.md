# Voice Recognition & Speaker Identification Guide

## Overview

The mBot Voice Recognition system enables personalized interactions by identifying users through voice biometrics with ≥85% confidence. This guide covers setup, usage, and best practices.

## Contracts

This system implements the following invariants:

- **I-VOICE-001**: Speaker identification ≥85% confidence
- **I-VOICE-002**: Privacy protection with explicit consent
- **I-VOICE-003**: Personalized content per user
- **I-VOICE-004**: 7-day conversational memory
- **I-VOICE-005**: Support for 10 distinct voice profiles
- **I-VOICE-006**: Anonymous mode fallback

## Quick Start

### 1. Voice Enrollment

Users must enroll their voice before mBot can recognize them:

```tsx
import { VoiceEnrollment } from './components/voice/VoiceEnrollment';

function App() {
  const handleEnrollmentComplete = (userId: string) => {
    console.log('User enrolled:', userId);
    // Store userId for the session
  };

  return (
    <VoiceEnrollment
      onComplete={handleEnrollmentComplete}
      onCancel={() => console.log('Enrollment cancelled')}
    />
  );
}
```

**Enrollment Process:**
1. User enters name and age (optional)
2. User grants consent for voice data collection
3. User records 3 voice samples by saying provided phrases
4. System creates voice profile with biometric signature
5. Profile stored locally (localStorage)

### 2. Speaker Identification

Identify users from voice audio:

```typescript
import VoiceProfileService from './services/voice/VoiceProfileService';
import AudioCapture from './services/voice/AudioCapture';

const voiceService = new VoiceProfileService();
const audioCapture = new AudioCapture();

// Start recording
await audioCapture.startRecording();

// ... user speaks ...

// Stop and identify
const audioData = await audioCapture.stopRecording();
const identification = await voiceService.identifyUser(audioData);

if (identification.confidence >= 0.85) {
  console.log('Identified user:', identification.userId);
} else {
  console.log('Anonymous mode:', identification.isAnonymous);
}
```

### 3. Profile Management

```typescript
// Get all profiles
const profiles = await voiceService.getAllProfiles();

// Get specific profile
const profile = await voiceService.getProfile(userId);

// Update profile
await voiceService.updateProfile(userId, {
  metadata: {
    age: 30,
    isChild: false,
    language: 'en-US'
  }
});

// Delete profile
await voiceService.deleteProfile(userId);
```

## Architecture

### Components

```
┌─────────────────────────────────────┐
│   VoiceEnrollment (UI Component)    │
└────────────┬────────────────────────┘
             │
             ▼
┌─────────────────────────────────────┐
│   VoiceProfileService               │
│   - enrollUser()                    │
│   - identifyUser()                  │
│   - manageProfiles()                │
└────────────┬────────────────────────┘
             │
             ├────────────┬────────────┐
             ▼            ▼            ▼
    ┌───────────┐  ┌──────────┐  ┌─────────┐
    │AudioCapture│  │WhisperAPI│  │localStorage│
    └───────────┘  └──────────┘  └─────────┘
```

### Data Flow

1. **Enrollment:**
   ```
   User Input → AudioCapture → VoiceSample[]
        → VoiceProfileService.enrollUser()
        → Extract Voiceprint → Store Profile → localStorage
   ```

2. **Identification:**
   ```
   Audio Input → AudioCapture → ArrayBuffer
        → VoiceProfileService.identifyUser()
        → Extract Features → Compare Voiceprints
        → VoiceIdentification (confidence score)
   ```

## Privacy & Security

### Privacy Protection (I-VOICE-002)

1. **Explicit Consent**: Users must check consent box before enrollment
2. **Local Storage**: All voice data stored locally (no cloud transmission)
3. **Profile Isolation**: No cross-user data sharing
4. **Data Deletion**: Users can delete profiles anytime

### Data Stored

```typescript
interface VoiceProfile {
  userId: string;
  name: string;
  voiceSamples: VoiceSample[];      // Original audio samples
  voiceprint: Float32Array;          // Biometric signature (128-dim)
  confidence: number;
  enrolledAt: number;
  lastUsed: number;
  useCount: number;
  metadata: {
    age?: number;
    isChild: boolean;
    language: string;
  };
}
```

### Storage Location

- **Key**: `mbot_voice_profiles`
- **Format**: JSON (voiceprints serialized as arrays)
- **Size**: ~50-100KB per profile

## Anonymous Mode (I-VOICE-006)

When voice recognition fails or confidence < 85%, the system operates in anonymous mode:

```typescript
const identification = await voiceService.identifyUser(audioData);

if (identification.isAnonymous) {
  // Generic greeting, no personalized data
  console.log("Hello! I don't think we've met.");
} else {
  // Personalized experience
  const profile = await voiceService.getProfile(identification.userId);
  console.log(`Hello, ${profile.name}!`);
}
```

**Anonymous Mode Features:**
- No personal data shown
- Generic responses
- Offer enrollment option
- Full functionality (limited personalization)

## Multi-User Support (I-VOICE-005)

The system supports up to 10 voice profiles:

```typescript
// Check capacity
const stats = voiceService.getStats();
console.log(`Profiles: ${stats.totalProfiles}/${stats.maxProfiles}`);

// Handle capacity limit
try {
  await voiceService.enrollUser(name, samples);
} catch (error) {
  if (error.message.includes('Maximum 10')) {
    console.log('Profile limit reached. Please delete unused profiles.');
  }
}
```

## Confidence Threshold

The system requires ≥85% confidence for positive identification:

```typescript
const CONFIDENCE_THRESHOLD = 0.85;

const identification = await voiceService.identifyUser(audioData);

if (identification.confidence >= CONFIDENCE_THRESHOLD) {
  // High confidence - proceed with identification
  return identification.userId;
} else {
  // Low confidence - use anonymous mode
  return null;
}
```

### Alternative Matches

The system provides alternative match suggestions:

```typescript
const identification = await voiceService.identifyUser(audioData);

console.log('Best match:', identification.userId, identification.confidence);
console.log('Alternatives:');
identification.alternativeMatches.forEach(match => {
  console.log(`- ${match.userId}: ${match.confidence.toFixed(2)}`);
});
```

## Performance

### Latency Requirements

- **Enrollment**: < 5 seconds for 3 samples
- **Identification**: < 2 seconds per audio sample
- **Profile loading**: < 100ms from localStorage

### Optimization Tips

1. **Pre-load profiles** at app startup
2. **Cache voiceprints** in memory
3. **Use Web Workers** for audio processing (future enhancement)
4. **Batch operations** when possible

## Browser Support

| Browser | Support | Notes |
|---------|---------|-------|
| Chrome | ✅ Full | Recommended |
| Edge | ✅ Full | Chromium-based |
| Safari | ✅ Full | iOS 14.1+ |
| Firefox | ⚠️ Limited | Requires getUserMedia flag |

### Feature Detection

```typescript
const audioCapture = new AudioCapture();

if (!audioCapture.isSupported()) {
  alert('Voice recording not supported in your browser');
}
```

## Testing

### Unit Tests

```bash
npm test -- voice-recognition
```

### Integration Tests

```bash
npm test -- tests/integration/voice-recognition.test.ts
```

### Test Coverage

The voice system includes comprehensive tests:

- Voice enrollment flow (3+ samples)
- Speaker identification (≥85% accuracy)
- Multi-user profile management
- Anonymous mode fallback
- Privacy compliance
- Performance benchmarks

**Target**: >90% coverage

## API Reference

### VoiceProfileService

```typescript
class VoiceProfileService {
  // Enrollment
  async enrollUser(name: string, samples: VoiceSample[]): Promise<VoiceProfile>
  async addVoiceSample(userId: string, sample: VoiceSample): Promise<void>

  // Identification
  async identifyUser(audioData: ArrayBuffer): Promise<VoiceIdentification>
  async verifyUser(userId: string, audioData: ArrayBuffer): Promise<boolean>

  // Management
  async getProfile(userId: string): Promise<VoiceProfile | null>
  async getAllProfiles(): Promise<VoiceProfile[]>
  async updateProfile(userId: string, updates: Partial<VoiceProfile>): Promise<void>
  async deleteProfile(userId: string): Promise<void>
  async clearAll(): Promise<void>

  // Statistics
  getStats(): { totalProfiles, maxProfiles, totalUses, averageConfidence }
}
```

### AudioCapture

```typescript
class AudioCapture {
  // Setup
  isSupported(): boolean
  async requestPermission(): Promise<boolean>
  async getAudioDevices(): Promise<MediaDeviceInfo[]>

  // Recording
  async startRecording(deviceId?: string): Promise<void>
  async stopRecording(): Promise<ArrayBuffer>
  isRecording(): boolean
  getRecordingDuration(): number

  // Monitoring
  async getAudioLevel(): Promise<number>

  // Cleanup
  dispose(): void
}
```

### WhisperAPI

```typescript
class WhisperAPI {
  constructor(config: WhisperConfig)

  async transcribeAudio(audioData: ArrayBuffer): Promise<string>
  async transcribeDetailed(audioData: ArrayBuffer): Promise<TranscriptionResult>
  async translateToEnglish(audioData: ArrayBuffer): Promise<string>

  isConfigured(): boolean
  setApiKey(apiKey: string): void
}
```

## Troubleshooting

### Common Issues

**1. Microphone Permission Denied**
```typescript
const hasPermission = await audioCapture.requestPermission();
if (!hasPermission) {
  // Guide user to browser settings
  alert('Please enable microphone access in your browser settings');
}
```

**2. Low Confidence Scores**
- Ensure quiet environment during enrollment
- Record more voice samples (use `addVoiceSample()`)
- Check microphone quality
- Verify sample rate compatibility

**3. Profile Not Found**
```typescript
const profile = await voiceService.getProfile(userId);
if (!profile) {
  console.log('Profile deleted or never created');
  // Offer re-enrollment
}
```

**4. localStorage Full**
- Each profile: ~50-100KB
- Limit: 10 profiles (~1MB total)
- Solution: Delete unused profiles

### Debug Mode

```typescript
// Enable detailed logging
localStorage.setItem('debug_voice', 'true');

// View stored profiles
const stored = localStorage.getItem('mbot_voice_profiles');
console.log(JSON.parse(stored));
```

## Future Enhancements

1. **Neural Network Models**: Replace simple similarity with SpeechBrain/Resemblyzer
2. **Real-time STT**: Integrate continuous speech-to-text
3. **Multi-language**: Support languages beyond English
4. **Noise Reduction**: Audio preprocessing for better accuracy
5. **Cloud Sync**: Optional encrypted cloud backup
6. **Voice Commands**: Integrate with robot control system

## Related Documentation

- [Voice Command Contract](/docs/contracts/feature_voice.yml)
- [Architecture Overview](/docs/contracts/feature_architecture.yml)
- [Privacy Policy](/docs/privacy-policy.md)
- [API Reference](/docs/api-reference.md)

## Support

For issues or questions:
- GitHub: https://github.com/Hulupeep/mbot_ruvector/issues
- Tag: `voice-recognition`
