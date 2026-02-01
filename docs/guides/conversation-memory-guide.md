# Conversation Memory Guide

## Overview

The Conversation Memory system enables mBot to remember conversations, track daily activities, and generate intelligent follow-up questions. This creates a personal assistant experience where mBot truly knows and remembers users.

## Architecture

### Components

```
┌─────────────────────────────────────────────────────────┐
│                   Memory System                          │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌──────────────────┐      ┌────────────────────┐      │
│  │ Memory Store     │◄─────┤ ConversationMemory │      │
│  │ (IndexedDB)      │      │ Service            │      │
│  └──────────────────┘      └────────────────────┘      │
│           │                          │                   │
│           │                          │                   │
│  ┌────────▼──────────┐      ┌───────▼────────────┐     │
│  │ Conversations     │      │ Daily Activities   │     │
│  │ - Turns           │      │ - Planned          │     │
│  │ - Topics          │      │ - Completed        │     │
│  │ - Key Points      │      │ - Notes            │     │
│  │ - Sentiment       │      │ - Mood             │     │
│  └───────────────────┘      └────────────────────┘     │
│                                                          │
│  ┌──────────────────────────────────────────────┐      │
│  │ KeyPointsExtractor                            │      │
│  │ - Entity recognition                          │      │
│  │ - Intent detection                            │      │
│  │ - Topic identification                        │      │
│  └──────────────────────────────────────────────┘      │
│                                                          │
│  ┌──────────────────────────────────────────────┐      │
│  │ FollowUpGenerator                             │      │
│  │ - Yesterday's activity questions              │      │
│  │ - Ongoing project questions                   │      │
│  │ - Goal recall questions                       │      │
│  └──────────────────────────────────────────────┘      │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

### Data Flow

```
User Conversation
    │
    ▼
Store Conversation ──► Extract Key Points
    │                       │
    │                       ▼
    │                  Identify Topics
    │                       │
    │                       ▼
    │                  Analyze Sentiment
    │                       │
    └───────────────────────┘
                │
                ▼
          IndexedDB Storage
                │
                ▼
    Generate Follow-Up Questions
                │
                ▼
        Present to User
```

## Core Services

### ConversationMemoryService

Manages conversation storage, retrieval, and activity tracking.

#### Store Conversations

```typescript
import { conversationMemoryService } from './services/memory/ConversationMemoryService';

// Store a conversation
const turns = [
  { speaker: 'user', text: 'I want to build a LEGO castle', timestamp: Date.now() },
  { speaker: 'mbot', text: 'That sounds fun!', timestamp: Date.now() },
];

const conversation = await conversationMemoryService.storeConversation(userId, turns);
// Returns: { id, timestamp, turns, topic, sentiment, keyPoints }
```

#### Retrieve Conversations

```typescript
// Get conversations from last 7 days (default)
const recent = await conversationMemoryService.getConversations(userId);

// Get conversations from last 30 days
const month = await conversationMemoryService.getConversations(userId, 30);

// Search conversations
const results = await conversationMemoryService.searchConversations(
  userId,
  'LEGO building'
);
```

#### Track Daily Activities

```typescript
// Store today's activities
const today = '2024-01-15';
await conversationMemoryService.storeDailyActivity(userId, today, [
  'build LEGO castle',
  'practice guitar',
  'read a book',
]);

// Get today's activity
const activity = await conversationMemoryService.getDailyActivity(userId, today);
// Returns: { date, userId, plannedActivities, completedActivities, notes, mood }

// Mark activity as completed
await conversationMemoryService.markActivityCompleted(
  userId,
  today,
  'build LEGO castle'
);

// Get yesterday's activity
const yesterday = await conversationMemoryService.getYesterdayActivity(userId);
```

### KeyPointsExtractor

Extracts meaningful information from conversations using NLP.

#### Extract Key Points

```typescript
import { keyPointsExtractor } from './services/memory/KeyPointsExtractor';

const keyPoints = await keyPointsExtractor.extractKeyPoints(conversation);
// Returns: ['building LEGO castle', 'medieval towers', ...]
```

#### Extract Entities

```typescript
const entities = await keyPointsExtractor.extractEntities(
  'I met Sarah yesterday at the park at 3 PM'
);
// Returns: [
//   { text: 'Sarah', type: 'person', confidence: 0.8 },
//   { text: 'yesterday', type: 'date', confidence: 0.9 },
//   { text: '3 PM', type: 'time', confidence: 0.9 },
// ]
```

#### Identify Topics

```typescript
const topics = await keyPointsExtractor.identifyTopics(conversation);
// Returns: ['lego', 'building', 'castle', ...]
```

### FollowUpGenerator

Generates intelligent follow-up questions based on conversation history.

#### Generate Follow-Ups

```typescript
import { followUpGenerator } from './services/memory/FollowUpGenerator';

// Generate all follow-up questions for user
const questions = await followUpGenerator.generateFollowUps(userId);
// Returns: [
//   {
//     id: 'abc123',
//     userId: 'user-1',
//     question: 'Yesterday you mentioned building a LEGO castle. Did you finish it?',
//     context: 'yesterday_activity:build LEGO castle',
//     priority: 80,
//     validUntil: 1234567890000,
//     answered: false,
//   },
//   ...
// ]
```

#### Check Yesterday's Activity

```typescript
const question = await followUpGenerator.checkYesterdayActivity(userId);
// Returns follow-up question about yesterday's planned activities
```

#### Answer Questions

```typescript
await followUpGenerator.markQuestionAnswered(
  questionId,
  'Yes, I finished building the castle!'
);
```

## React Components

### MemoryTimeline

Displays a timeline of daily activities with date navigation.

```tsx
import { MemoryTimeline } from './components/memory/MemoryTimeline';

function MyComponent() {
  return <MemoryTimeline userId="user-123" />;
}
```

Features:
- Date picker with today/yesterday/custom date
- Activity list with planned/completed status
- Checkbox to mark activities complete
- Mood indicators
- Notes display

### ConversationHistory

Shows past conversations with search functionality.

```tsx
import { ConversationHistory } from './components/memory/ConversationHistory';

function MyComponent() {
  return (
    <ConversationHistory
      userId="user-123"
      days={7} // Optional, default 7
    />
  );
}
```

Features:
- Search conversations
- Expandable conversation cards
- Display conversation turns
- Show key points
- Sentiment indicators
- Timestamp formatting (relative and absolute)

### FollowUpQuestions

Displays and allows answering follow-up questions.

```tsx
import { FollowUpQuestions } from './components/memory/FollowUpQuestions';

function MyComponent() {
  return (
    <FollowUpQuestions
      userId="user-123"
      onAnswer={(questionId, answer) => {
        console.log('User answered:', answer);
      }}
    />
  );
}
```

Features:
- Priority indicators (high/medium/low)
- Context badges (Yesterday, Project, Goal, etc.)
- Time remaining display
- Answer form with submit/cancel
- Dismiss functionality

## Data Retention

### Retention Policy (I-VOICE-004)

The system maintains conversations for **at least 7 days** by default. The retention period is configurable per user via privacy settings.

#### Configure Retention

```typescript
// Set custom retention period (in days)
await conversationMemoryService.pruneOldConversations(userId, 14);

// Clean up old conversations (call periodically)
await conversationMemoryService.pruneAllOldConversations();
```

#### Automatic Cleanup

Set up a periodic cleanup job:

```typescript
// Run daily
setInterval(
  async () => {
    await conversationMemoryService.pruneAllOldConversations();
    await followUpGenerator.cleanupExpiredQuestions();
  },
  24 * 60 * 60 * 1000
); // 24 hours
```

## Follow-Up Question Types

### Yesterday's Activity

When a user mentions planning to do something, mBot follows up the next day.

**Example:**
- User says: "Today I want to build a LEGO castle"
- Next day, mBot asks: "Yesterday you mentioned building a LEGO castle. Did you finish it?"

### Ongoing Projects

When a topic is mentioned multiple times, mBot checks progress.

**Example:**
- User mentions "learning Python" over several days
- mBot asks: "How's your learning Python coming along?"

### Previous Goals

When a user expresses a goal, mBot follows up after a few days.

**Example:**
- User says: "I want to learn guitar"
- 3 days later, mBot asks: "Remember you wanted to learn guitar. How's that going?"

### Upcoming Events

When a user mentions an upcoming event, mBot asks about it after.

**Example:**
- User mentions: "I have a job interview tomorrow"
- Next day, mBot asks: "Your job interview was today, right? How did it go?"

## Integration Example

Complete example integrating all components:

```tsx
import React, { useState } from 'react';
import { MemoryTimeline } from './components/memory/MemoryTimeline';
import { ConversationHistory } from './components/memory/ConversationHistory';
import { FollowUpQuestions } from './components/memory/FollowUpQuestions';
import { conversationMemoryService } from './services/memory/ConversationMemoryService';

function MemoryDashboard({ userId }: { userId: string }) {
  const [activeTab, setActiveTab] = useState<'timeline' | 'history' | 'questions'>(
    'timeline'
  );

  const handleAnswer = async (questionId: string, answer: string) => {
    console.log('User answered:', answer);

    // Store the answer as a conversation
    await conversationMemoryService.storeConversation(userId, [
      { speaker: 'mbot', text: '[Follow-up question]', timestamp: Date.now() },
      { speaker: 'user', text: answer, timestamp: Date.now() },
    ]);
  };

  return (
    <div className="memory-dashboard">
      <nav className="tabs">
        <button
          onClick={() => setActiveTab('timeline')}
          className={activeTab === 'timeline' ? 'active' : ''}
        >
          Timeline
        </button>
        <button
          onClick={() => setActiveTab('history')}
          className={activeTab === 'history' ? 'active' : ''}
        >
          Conversations
        </button>
        <button
          onClick={() => setActiveTab('questions')}
          className={activeTab === 'questions' ? 'active' : ''}
        >
          Follow-Ups
        </button>
      </nav>

      <div className="tab-content">
        {activeTab === 'timeline' && <MemoryTimeline userId={userId} />}
        {activeTab === 'history' && <ConversationHistory userId={userId} days={14} />}
        {activeTab === 'questions' && (
          <FollowUpQuestions userId={userId} onAnswer={handleAnswer} />
        )}
      </div>
    </div>
  );
}
```

## Contract Compliance

### I-VOICE-004: Conversational Memory

**Requirement:** MUST maintain conversation context for at least 7 days and MUST recall previous day's activities when asked.

**Implementation:**
- Default retention: 7 days
- Configurable per user via `PrivacySettings.retentionDays`
- Automatic cleanup via `pruneOldConversations()`
- Yesterday's activity retrieval via `getYesterdayActivity()`

### I-MEMORY-001: Activity Tracking

**Requirement:** MUST track daily activities with timestamp and context, enabling "what did I do yesterday" queries.

**Implementation:**
- `DailyActivity` type with date, planned, completed, notes, mood
- `storeDailyActivity()` stores daily plans
- `markActivityCompleted()` tracks completion
- `getYesterdayActivity()` retrieves yesterday's data

## Testing

### Unit Tests

```typescript
import { conversationMemoryService } from './services/memory/ConversationMemoryService';

describe('ConversationMemoryService', () => {
  it('should store and retrieve conversations', async () => {
    const turns = [
      { speaker: 'user', text: 'Hello', timestamp: Date.now() },
    ];

    const conv = await conversationMemoryService.storeConversation('user-1', turns);
    expect(conv.id).toBeDefined();

    const retrieved = await conversationMemoryService.getConversations('user-1');
    expect(retrieved).toHaveLength(1);
  });
});
```

### Integration Tests

See `tests/integration/conversation-memory.test.ts` for comprehensive test suite covering:
- Conversation storage and retrieval
- Key points extraction
- Activity tracking
- Follow-up generation
- Sentiment analysis
- Data retention

## Performance Considerations

### IndexedDB Storage

- **Efficient indexing:** Conversations indexed by userId and timestamp
- **Fast queries:** Date range queries use compound indexes
- **Local storage:** No network latency

### Memory Usage

- Conversations stored in IndexedDB (not RAM)
- Only active conversations loaded in memory
- Automatic cleanup prevents database bloat

### Optimization Tips

1. **Batch operations:** Store multiple conversations at once
2. **Lazy loading:** Only load visible conversations
3. **Debounce search:** Wait for user to finish typing before searching
4. **Pagination:** Load conversations in pages for large histories

## Privacy & Security

### Data Protection

- All data stored locally in IndexedDB
- No server-side storage
- User controls retention period
- Easy data deletion via `deleteOldConversations()`

### User Consent

Always obtain consent before:
- Recording conversations
- Storing personal information
- Generating follow-up questions

## Troubleshooting

### Conversations not storing

```typescript
// Check if IndexedDB is supported
if (!window.indexedDB) {
  console.error('IndexedDB not supported');
}

// Check for errors
try {
  await conversationMemoryService.storeConversation(userId, turns);
} catch (err) {
  console.error('Failed to store:', err);
}
```

### Follow-up questions not generating

```typescript
// Ensure conversations are stored
const conversations = await conversationMemoryService.getConversations(userId);
console.log('Stored conversations:', conversations.length);

// Check for yesterday's activity
const yesterday = await conversationMemoryService.getYesterdayActivity(userId);
console.log('Yesterday activity:', yesterday);

// Generate questions manually
const questions = await followUpGenerator.generateFollowUps(userId);
console.log('Generated questions:', questions);
```

### Memory timeline not loading

```typescript
// Check activities exist
const today = new Date().toISOString().split('T')[0];
const activity = await conversationMemoryService.getDailyActivity(userId, today);
console.log('Today activity:', activity);
```

## API Reference

See inline JSDoc comments in source files for complete API reference:
- `ConversationMemoryService.ts`
- `FollowUpGenerator.ts`
- `KeyPointsExtractor.ts`
- `MemoryStore.ts`

## Examples

See `/examples/memory` directory for:
- Complete dashboard implementation
- Voice assistant integration
- Mobile-friendly layouts
- Custom styling examples
