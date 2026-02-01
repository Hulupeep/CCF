# Voice Assistant Complete Guide

## Overview

The mBot Voice Assistant provides personalized daily briefings, voice-activated interactions, conversational memory, and integration with news and email services. This guide covers complete setup, usage, and integration.

## Features

### 1. Voice Recognition & Speaker Identification
- Multi-user voice profiles (up to 10 users)
- ≥85% identification confidence (Contract: I-VOICE-001)
- Voice enrollment with 3+ samples
- Anonymous mode for unknown users (Contract: I-VOICE-006)

### 2. Personalized Morning Briefings
- Customized greeting based on time of day
- News headlines filtered by user preferences
- Email summary with priority detection
- Yesterday's activity recall
- Intelligent follow-up questions

### 3. Conversational Memory
- 7-day conversation retention (Contract: I-VOICE-004)
- Daily activity tracking (Contract: I-MEMORY-001)
- Key points extraction
- Sentiment analysis
- Multi-user context switching

### 4. News Personalization
- Topic-based filtering (Contract: I-NEWS-001)
- Source preferences
- Relevance scoring
- Adaptive preference learning
- Age-appropriate content filtering

### 5. Email Integration
- OAuth2 authentication (Contract: I-EMAIL-001)
- Gmail and Outlook support
- Email summarization
- Priority detection
- Category-based organization

### 6. Privacy Controls
- Explicit consent required (Contract: I-VOICE-002)
- Granular permission settings
- Configurable data retention
- Family data sharing controls
- No cross-user data leakage

## Architecture

### Component Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Voice Assistant                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   Voice      │  │   Personal   │  │     TTS      │      │
│  │ Recognition  │  │   Briefing   │  │   Service    │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│         │                  │                  │              │
│         ▼                  ▼                  ▼              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │    Voice     │  │    News      │  │    Email     │      │
│  │   Profile    │  │   Service    │  │   Service    │      │
│  │   Service    │  └──────────────┘  └──────────────┘      │
│  └──────────────┘                                           │
│         │                                                    │
│         ▼                                                    │
│  ┌──────────────────────────────────────────────────┐      │
│  │         Conversation Memory Service              │      │
│  └──────────────────────────────────────────────────┘      │
└─────────────────────────────────────────────────────────────┘
```

### Service Responsibilities

#### VoiceAssistant
- Main orchestration controller
- Voice input processing
- Command routing
- Integration coordination

#### PersonalBriefingService
- Briefing generation
- Section building
- Content prioritization
- Delivery coordination

#### VoiceProfileService
- Voice enrollment
- Speaker identification
- Profile management
- Biometric storage

#### NewsService
- News fetching
- Personalization
- Preference learning
- Relevance scoring

#### EmailService
- OAuth2 authentication
- Email fetching
- Summarization
- Priority detection

#### ConversationMemoryService
- Conversation storage
- Activity tracking
- Follow-up questions
- Memory retrieval

#### TTSService
- Speech synthesis
- Voice selection
- Audio playback
- Multi-provider support

## Setup

### 1. Voice Enrollment

```typescript
import { VoiceProfileService } from './services/voice/VoiceProfileService';

const voiceService = new VoiceProfileService();

// Enroll new user
const userId = 'user-1';
const name = 'Alice';
const samples = []; // Array of 3+ VoiceSample objects

await voiceService.enrollUser(userId, name, samples);
```

**UI Flow:**
1. Navigate to `/voice/enroll`
2. Click "Enroll New Voice"
3. Enter name
4. Record 3+ voice samples (3-5 seconds each)
5. Wait for enrollment completion

### 2. Configure News Preferences

```typescript
import { NewsService } from './services/news/NewsService';

const newsService = new NewsService();

await newsService.updatePreferences(userId, {
  userId,
  topics: ['technology', 'science', 'sports'],
  sources: ['bbc', 'reuters'],
  excludeTopics: ['politics'],
  maxArticles: 5,
  readingLevel: 'adult',
  topicWeights: {},
  lastUpdated: Date.now()
});
```

**UI Flow:**
1. Navigate to `/voice/preferences`
2. Select topics of interest
3. Set maximum articles
4. Choose reading level
5. Click "Save Preferences"

### 3. Connect Email Accounts

```typescript
import { EmailService } from './services/email/EmailService';

const emailService = new EmailService();

// Initiate OAuth2 flow
await emailService.connectGmail(userId);
// or
await emailService.connectOutlook(userId);
```

**UI Flow:**
1. Navigate to `/voice/email`
2. Click "Connect Gmail" or "Connect Outlook"
3. Complete OAuth2 authorization
4. Verify connection success

### 4. Configure Privacy Settings

```typescript
import { ConversationMemoryService } from './services/memory/ConversationMemoryService';

const memoryService = new ConversationMemoryService();

await memoryService.updatePrivacySettings(userId, {
  allowVoiceRecording: true,
  allowEmailAccess: true,
  allowNewsPersonalization: true,
  shareDataWithFamily: false,
  retentionDays: 7
});
```

**UI Flow:**
1. Navigate to `/voice/privacy`
2. Toggle permission checkboxes
3. Set data retention period
4. Click "Save Settings"

## Usage

### Daily Briefing

#### Automatic Trigger
```typescript
// Registered with Autonomy Engine (#93)
// Triggers at 8 AM daily or when user voice detected in morning window

import { registerVoiceActions } from './services/autonomy/actions/VoiceBriefingAction';

registerVoiceActions(autonomyEngine);
```

#### Manual Trigger
```typescript
import { VoiceAssistant } from './services/voice/VoiceAssistant';

const assistant = new VoiceAssistant();

// User says "Good morning"
await assistant.triggerMorningBriefing(userId);
```

**UI Flow:**
1. Navigate to `/voice/dashboard`
2. Click "Generate Briefing"
3. Click "Play Briefing" to hear it aloud
4. View sections in detail

### Voice Commands

Supported commands:
- **"Good morning"** → Trigger morning briefing
- **"What's new?"** → News headlines
- **"Check my email"** → Email summary
- **"What did I do yesterday?"** → Memory recall
- **"Tell me more about [topic]"** → Detailed news

### Conversation Memory

#### Store Daily Activity
```typescript
import { ConversationMemoryService } from './services/memory/ConversationMemoryService';

const memoryService = new ConversationMemoryService();

await memoryService.storeDailyActivity(userId, '2024-02-01', {
  date: '2024-02-01',
  userId,
  plannedActivities: ['work on project', 'exercise'],
  completedActivities: ['work on project'],
  notes: 'Good progress on project',
  mood: 'productive'
});
```

#### Store Conversation
```typescript
await memoryService.storeConversationTurn({
  speaker: 'user',
  text: 'I want to learn guitar',
  timestamp: Date.now(),
  intent: 'hobby_interest'
}, userId);

await memoryService.storeConversationTurn({
  speaker: 'mbot',
  text: 'That\'s great! What song would you like to start with?',
  timestamp: Date.now()
}, userId);
```

#### Create Follow-Up Question
```typescript
await memoryService.storeFollowUpQuestion({
  id: crypto.randomUUID(),
  userId,
  question: 'How did your guitar practice go?',
  context: 'User expressed interest in learning guitar',
  priority: 70,
  validUntil: Date.now() + (7 * 24 * 60 * 60 * 1000), // 7 days
  answered: false
});
```

## Integration

### With Learning Engine (#92)

```typescript
import { VoiceAssistant } from './services/voice/VoiceAssistant';

const assistant = new VoiceAssistant();

// Observe interaction for preference learning
await assistant.observeInteraction(
  userId,
  'Tell me more about AI news',
  'Here are more details...',
  true // success
);

// Learning engine adjusts news preferences based on interactions
```

### With Autonomous Behavior Engine (#93)

```typescript
import { AutonomyEngine } from './services/autonomy/AutonomyEngine';
import { registerVoiceActions } from './services/autonomy/actions/VoiceBriefingAction';

const autonomyEngine = new AutonomyEngine();

// Register voice actions
registerVoiceActions(autonomyEngine);

// Actions are now triggered automatically:
// - Morning briefing at 8 AM
// - Proactive check-in after inactivity
// - Follow-up questions when user returns
```

## API Reference

### VoiceAssistant

```typescript
class VoiceAssistant {
  async handleVoiceInput(audioData: ArrayBuffer): Promise<void>;
  async identifyUser(audioData: ArrayBuffer): Promise<VoiceIdentification>;
  async transcribeAudio(audioData: ArrayBuffer): Promise<string>;
  async processCommand(userId: string, text: string): Promise<void>;
  async generateResponse(userId: string, text: string): Promise<string>;
  async speakResponse(text: string): Promise<void>;
  async triggerMorningBriefing(userId: string): Promise<void>;
  async checkInAfterInactivity(userId: string): Promise<void>;
  async observeInteraction(userId: string, input: string, response: string, success: boolean): Promise<void>;
  startListening(callback: (text: string) => void): void;
  stopListening(): void;
  isCurrentlyListening(): boolean;
}
```

### PersonalBriefingService

```typescript
class PersonalBriefingService {
  async generateBriefing(userId: string): Promise<PersonalBriefing>;
  async deliverBriefing(userId: string, briefing: PersonalBriefing): Promise<void>;
  async speakBriefing(text: string): Promise<void>;
  async getBriefingHistory(userId: string, limit?: number): Promise<PersonalBriefing[]>;
}
```

### VoiceProfileService

```typescript
class VoiceProfileService {
  async enrollUser(userId: string, name: string, samples: VoiceSample[]): Promise<VoiceProfile>;
  async identifyUser(audioData: ArrayBuffer): Promise<VoiceIdentification>;
  async getProfile(userId: string): Promise<VoiceProfile | null>;
  async getAllProfiles(): Promise<VoiceProfile[]>;
  async updateProfile(userId: string, updates: Partial<VoiceProfile>): Promise<void>;
  async deleteProfile(userId: string): Promise<void>;
}
```

### TTSService

```typescript
class TTSService {
  async synthesizeSpeech(text: string, options?: TTSOptions): Promise<void>;
  async playAudio(audioData: ArrayBuffer): Promise<void>;
  async getAvailableVoices(): Promise<Voice[]>;
  async setVoiceForUser(userId: string, voiceId: string): Promise<void>;
  async getUserVoice(userId: string): Promise<string | null>;
  stop(): void;
  isSpeaking(): boolean;
}
```

## Testing

### Unit Tests

```bash
npm test tests/integration/voice-assistant-full.test.ts
```

Coverage: >90%

### E2E Journey Tests

```bash
npm run test:journeys tests/journeys/voice-morning-briefing.journey.spec.ts
npm run test:journeys tests/journeys/voice-daily-planning.journey.spec.ts
npm run test:journeys tests/journeys/voice-memory-recall.journey.spec.ts
```

## Troubleshooting

### Voice Recognition Low Confidence

**Problem:** Voice identification confidence < 85%

**Solutions:**
1. Re-enroll with more/better samples
2. Record in quiet environment
3. Speak clearly and consistently
4. Check microphone quality

### Briefing Not Playing

**Problem:** TTS not working

**Solutions:**
1. Check browser speech synthesis support
2. Verify audio permissions
3. Test with different voice provider
4. Check console for errors

### News Not Personalizing

**Problem:** Getting irrelevant news

**Solutions:**
1. Verify preferences saved correctly
2. Check news API quota/limits
3. Interact with news (click "tell me more")
4. Clear and reset preferences

### Email Not Connecting

**Problem:** OAuth2 flow fails

**Solutions:**
1. Verify OAuth2 credentials configured
2. Check redirect URI matches
3. Clear browser cookies/storage
4. Try different email provider

## Performance

### Metrics

- Voice identification: <2s
- Briefing generation: <3s
- TTS synthesis: <1s per section
- Memory retrieval: <500ms

### Optimization Tips

1. Limit news articles to 3-5
2. Use browser TTS for lower latency
3. Cache voice profiles
4. Prune old conversations regularly

## Privacy & Security

### Data Storage

- Voice biometrics: Encrypted in localStorage
- OAuth tokens: Encrypted in localStorage
- Conversations: Retained per user policy (7-365 days)
- Activities: Retained per user policy

### Compliance

- GDPR: Data deletion on request
- COPPA: Age-appropriate content filtering
- OAuth2: Secure email access
- Explicit consent: Required for all features

## Support

For issues or questions:
- GitHub Issues: https://github.com/Hulupeep/mbot_ruvector/issues
- Contract Reference: `docs/contracts/feature_voice.yml`
- Issue #95: https://github.com/Hulupeep/mbot_ruvector/issues/95
