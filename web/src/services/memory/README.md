# Conversation Memory System

Component 4/5 of the Voice-Activated Personal Assistant (#95)

## Overview

This system provides conversation tracking, daily activity management, and intelligent follow-up question generation for mBot's personal assistant features.

## Components

### Services

- **ConversationMemoryService** - Store and retrieve conversations with key point extraction
- **FollowUpGenerator** - Generate contextual follow-up questions
- **KeyPointsExtractor** - Extract entities, topics, and important facts using NLP
- **MemoryStore** - IndexedDB persistence layer

### React Components

- **MemoryTimeline** - Visual timeline of daily activities
- **ConversationHistory** - Searchable conversation history
- **FollowUpQuestions** - Interactive follow-up question interface

## Features

### Conversation Memory (I-VOICE-004)
- 7+ day retention (configurable)
- Full-text search
- Key point extraction
- Sentiment analysis
- Topic identification

### Activity Tracking (I-MEMORY-001)
- Daily activity planning
- Completion tracking
- Yesterday recall
- Date range queries
- Mood tracking

### Follow-Up Generation
- Yesterday's activity questions (priority: 80)
- Ongoing project questions (priority: 70)
- Previous goal recall (priority: 60)
- Event follow-ups (priority: 90)
- Intelligent prioritization

## Quick Start

```typescript
import { conversationMemoryService } from './services/memory';

// Store a conversation
const conversation = await conversationMemoryService.storeConversation(userId, [
  { speaker: 'user', text: 'I want to build a LEGO castle', timestamp: Date.now() },
  { speaker: 'mbot', text: 'That sounds fun!', timestamp: Date.now() },
]);

// Get recent conversations
const recent = await conversationMemoryService.getConversations(userId, 7);

// Track daily activities
const today = '2024-01-15';
await conversationMemoryService.storeDailyActivity(userId, today, [
  'build LEGO castle',
  'practice guitar',
]);

// Generate follow-ups
import { followUpGenerator } from './services/memory';
const questions = await followUpGenerator.generateFollowUps(userId);
```

## Architecture

```
┌─────────────────────────────────────────┐
│         Memory System                    │
├─────────────────────────────────────────┤
│                                          │
│  ConversationMemoryService               │
│  ├─ Store conversations                 │
│  ├─ Search & retrieve                   │
│  ├─ Track activities                    │
│  └─ Generate summaries                  │
│                                          │
│  KeyPointsExtractor                      │
│  ├─ Entity recognition                  │
│  ├─ Topic identification                │
│  ├─ Intent detection                    │
│  └─ Sentiment analysis                  │
│                                          │
│  FollowUpGenerator                       │
│  ├─ Yesterday activity questions        │
│  ├─ Ongoing project questions           │
│  ├─ Goal recall questions               │
│  └─ Priority ranking                    │
│                                          │
│  MemoryStore (IndexedDB)                │
│  ├─ Conversations                       │
│  ├─ Activities                          │
│  └─ Questions                           │
│                                          │
└─────────────────────────────────────────┘
```

## Data Types

### Conversation
```typescript
{
  id: string;
  timestamp: number;
  turns: ConversationTurn[];
  topic: string;
  sentiment: 'positive' | 'neutral' | 'negative';
  keyPoints: string[];
}
```

### DailyActivity
```typescript
{
  date: string; // YYYY-MM-DD
  userId: string;
  plannedActivities: string[];
  completedActivities: string[];
  notes: string;
  mood?: string;
}
```

### FollowUpQuestion
```typescript
{
  id: string;
  userId: string;
  question: string;
  context: string;
  priority: number; // 0-100
  validUntil: number;
  answered: boolean;
  answer?: string;
}
```

## Testing

Run integration tests:
```bash
npm test tests/integration/conversation-memory.test.ts
```

Test coverage:
- Conversation storage/retrieval: ✓
- Key points extraction: ✓
- Activity tracking: ✓
- Follow-up generation: ✓
- Sentiment analysis: ✓
- Data retention: ✓

Target: >90% coverage

## Contract Compliance

### I-VOICE-004: Conversational Memory
- ✓ 7+ day retention
- ✓ Yesterday activity recall
- ✓ Search functionality
- ✓ Context preservation

### I-MEMORY-001: Activity Tracking
- ✓ Daily activity storage
- ✓ Timestamp tracking
- ✓ Completion status
- ✓ "What did I do yesterday" queries

## Performance

- **Storage:** IndexedDB (local, no network)
- **Search:** Indexed queries (<50ms)
- **Memory:** Lazy loading, minimal RAM usage
- **Cleanup:** Automatic retention policy enforcement

## Documentation

See full documentation in `/docs/guides/conversation-memory-guide.md`

## Dependencies

- `uuid` - ID generation (already installed)
- `indexeddb` - Built-in browser API

## Files Created

### Services (1,900+ lines)
- `ConversationMemoryService.ts` - Core memory service
- `FollowUpGenerator.ts` - Question generation
- `KeyPointsExtractor.ts` - NLP extraction
- `MemoryStore.ts` - IndexedDB storage

### Components (1,050+ lines)
- `MemoryTimeline.tsx` - Activity timeline
- `ConversationHistory.tsx` - Conversation viewer
- `FollowUpQuestions.tsx` - Question interface

### Tests (600+ lines)
- `conversation-memory.test.ts` - Integration tests

### Documentation (350+ lines)
- `conversation-memory-guide.md` - Complete guide

**Total: ~4,000 lines of production code**

## Next Steps

1. Add CSS styling for components
2. Integrate with VoiceIdentificationService (#95 Component 1)
3. Connect to PersonalBriefingService (#95 Component 2)
4. Add voice command triggers
5. Deploy and test with real users

## Related Issues

- #95 - Voice-Activated Personal Assistant (Parent)
- #92 - Self-Learning Engine (Preference learning)
- #93 - Autonomous Behavior (Proactive briefings)
