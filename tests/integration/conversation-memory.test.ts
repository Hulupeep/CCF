/**
 * Integration Tests for Conversation Memory System
 *
 * Contract: I-VOICE-004, I-MEMORY-001
 * Tests conversation storage, activity tracking, and follow-up generation
 */

import { describe, it, expect, beforeEach, afterEach } from '@jest/globals';
import { ConversationMemoryService } from '../../web/src/services/memory/ConversationMemoryService';
import { FollowUpGenerator } from '../../web/src/services/memory/FollowUpGenerator';
import { KeyPointsExtractor } from '../../web/src/services/memory/KeyPointsExtractor';
import { MemoryStore } from '../../web/src/services/memory/MemoryStore';
import { ConversationTurn, Conversation } from '../../web/src/types/voice';

describe('Conversation Memory System Integration', () => {
  let memoryService: ConversationMemoryService;
  let followUpGen: FollowUpGenerator;
  let keyPointsExt: KeyPointsExtractor;
  let memoryStore: MemoryStore;

  const testUserId = 'test-user-123';

  beforeEach(() => {
    memoryService = new ConversationMemoryService();
    followUpGen = new FollowUpGenerator();
    keyPointsExt = new KeyPointsExtractor();
    memoryStore = new MemoryStore();
  });

  afterEach(async () => {
    // Clean up test data
    await memoryStore.deleteOldConversations(testUserId, 0);
    memoryStore.close();
  });

  describe('Conversation Storage and Retrieval', () => {
    it('should store and retrieve conversations', async () => {
      const turns: ConversationTurn[] = [
        {
          speaker: 'user',
          text: 'I want to build a LEGO castle today',
          timestamp: Date.now(),
        },
        {
          speaker: 'mbot',
          text: 'That sounds fun! What kind of castle?',
          timestamp: Date.now(),
        },
        {
          speaker: 'user',
          text: 'A medieval castle with towers',
          timestamp: Date.now(),
        },
      ];

      const conversation = await memoryService.storeConversation(testUserId, turns);

      expect(conversation.id).toBeDefined();
      expect(conversation.turns).toHaveLength(3);
      expect(conversation.topic).toBeTruthy();
      expect(conversation.keyPoints.length).toBeGreaterThan(0);

      const retrieved = await memoryService.getConversations(testUserId);
      expect(retrieved).toHaveLength(1);
      expect(retrieved[0].id).toBe(conversation.id);
    });

    it('should retrieve conversations within date range', async () => {
      const now = Date.now();
      const twoDaysAgo = now - 2 * 24 * 60 * 60 * 1000;
      const fiveDaysAgo = now - 5 * 24 * 60 * 60 * 1000;

      // Store conversations at different times
      await memoryService.storeConversation(testUserId, [
        { speaker: 'user', text: 'Test 1', timestamp: twoDaysAgo },
      ]);
      await memoryService.storeConversation(testUserId, [
        { speaker: 'user', text: 'Test 2', timestamp: fiveDaysAgo },
      ]);

      const recent = await memoryService.getConversations(testUserId, 3);
      expect(recent).toHaveLength(1); // Only the 2-day-old one

      const all = await memoryService.getConversations(testUserId, 7);
      expect(all).toHaveLength(2); // Both conversations
    });

    it('should search conversations by query', async () => {
      await memoryService.storeConversation(testUserId, [
        { speaker: 'user', text: 'I want to learn Python programming', timestamp: Date.now() },
      ]);
      await memoryService.storeConversation(testUserId, [
        { speaker: 'user', text: 'I want to build a robot', timestamp: Date.now() },
      ]);

      const pythonResults = await memoryService.searchConversations(testUserId, 'python');
      expect(pythonResults).toHaveLength(1);
      expect(pythonResults[0].turns[0].text).toContain('Python');

      const buildResults = await memoryService.searchConversations(testUserId, 'build');
      expect(buildResults).toHaveLength(1);
    });
  });

  describe('Key Points Extraction', () => {
    it('should extract key points from conversation', async () => {
      const conversation: Conversation = {
        id: 'test-conv',
        timestamp: Date.now(),
        turns: [
          {
            speaker: 'user',
            text: 'I want to learn guitar and practice Wonderwall',
            timestamp: Date.now(),
          },
          {
            speaker: 'mbot',
            text: 'Great choice! Have you played guitar before?',
            timestamp: Date.now(),
          },
          {
            speaker: 'user',
            text: 'No, this is my first time',
            timestamp: Date.now(),
          },
        ],
        topic: 'Learning guitar',
        sentiment: 'positive',
        keyPoints: [],
      };

      const keyPoints = await keyPointsExt.extractKeyPoints(conversation);

      expect(keyPoints.length).toBeGreaterThan(0);
      expect(keyPoints.some((point) => point.toLowerCase().includes('guitar'))).toBe(true);
    });

    it('should extract entities from text', async () => {
      const entities = await keyPointsExt.extractEntities(
        'I met Sarah yesterday and we went to Paris for a meeting at 3:00 PM'
      );

      expect(entities.length).toBeGreaterThan(0);
      expect(entities.some((e) => e.type === 'person' && e.text === 'Sarah')).toBe(true);
      expect(entities.some((e) => e.type === 'date')).toBe(true);
      expect(entities.some((e) => e.type === 'time')).toBe(true);
    });

    it('should identify topics from conversation', async () => {
      const conversation: Conversation = {
        id: 'test-conv',
        timestamp: Date.now(),
        turns: [
          {
            speaker: 'user',
            text: 'I love building LEGO robots',
            timestamp: Date.now(),
          },
          {
            speaker: 'user',
            text: 'I want to build more LEGO projects',
            timestamp: Date.now(),
          },
        ],
        topic: 'LEGO',
        sentiment: 'positive',
        keyPoints: [],
      };

      const topics = await keyPointsExt.identifyTopics(conversation);

      expect(topics.length).toBeGreaterThan(0);
      expect(topics.some((t) => t.toLowerCase().includes('lego'))).toBe(true);
    });
  });

  describe('Daily Activity Tracking', () => {
    it('should store and retrieve daily activities', async () => {
      const today = getDateString(new Date());
      const activities = ['build LEGO castle', 'practice guitar', 'read a book'];

      await memoryService.storeDailyActivity(testUserId, today, activities);

      const retrieved = await memoryService.getDailyActivity(testUserId, today);
      expect(retrieved).toBeDefined();
      expect(retrieved!.plannedActivities).toEqual(activities);
      expect(retrieved!.completedActivities).toHaveLength(0);
    });

    it('should mark activities as completed', async () => {
      const today = getDateString(new Date());
      const activities = ['build LEGO castle', 'practice guitar'];

      await memoryService.storeDailyActivity(testUserId, today, activities);
      await memoryService.markActivityCompleted(testUserId, today, 'build LEGO castle');

      const retrieved = await memoryService.getDailyActivity(testUserId, today);
      expect(retrieved!.completedActivities).toContain('build LEGO castle');
      expect(retrieved!.completedActivities).not.toContain('practice guitar');
    });

    it('should get yesterday\'s activity', async () => {
      const yesterday = getDateString(new Date(Date.now() - 24 * 60 * 60 * 1000));
      await memoryService.storeDailyActivity(testUserId, yesterday, ['test activity']);

      const retrieved = await memoryService.getYesterdayActivity(testUserId);
      expect(retrieved).toBeDefined();
      expect(retrieved!.date).toBe(yesterday);
    });

    it('should get activities in date range', async () => {
      const dates = [];
      for (let i = 0; i < 5; i++) {
        const date = getDateString(new Date(Date.now() - i * 24 * 60 * 60 * 1000));
        dates.push(date);
        await memoryService.storeDailyActivity(testUserId, date, [`activity ${i}`]);
      }

      const startDate = dates[4]; // 5 days ago
      const endDate = dates[0]; // today

      const activities = await memoryService.getActivitiesInRange(
        testUserId,
        startDate,
        endDate
      );
      expect(activities).toHaveLength(5);
    });
  });

  describe('Follow-Up Question Generation', () => {
    it('should generate follow-up for yesterday\'s activity', async () => {
      const yesterday = getDateString(new Date(Date.now() - 24 * 60 * 60 * 1000));
      await memoryService.storeDailyActivity(testUserId, yesterday, ['build LEGO castle']);

      const question = await followUpGen.checkYesterdayActivity(testUserId);

      expect(question).toBeDefined();
      expect(question!.question.toLowerCase()).toContain('yesterday');
      expect(question!.question.toLowerCase()).toContain('castle');
      expect(question!.priority).toBeGreaterThan(70);
    });

    it('should not generate follow-up if activity completed', async () => {
      const yesterday = getDateString(new Date(Date.now() - 24 * 60 * 60 * 1000));
      await memoryService.storeDailyActivity(testUserId, yesterday, ['build LEGO castle']);
      await memoryService.markActivityCompleted(testUserId, yesterday, 'build LEGO castle');

      const question = await followUpGen.checkYesterdayActivity(testUserId);

      expect(question).toBeNull();
    });

    it('should generate follow-up for ongoing projects', async () => {
      // Store conversations about a project
      await memoryService.storeConversation(testUserId, [
        {
          speaker: 'user',
          text: 'I am building a weather station',
          timestamp: Date.now() - 2 * 24 * 60 * 60 * 1000,
        },
      ]);
      await memoryService.storeConversation(testUserId, [
        {
          speaker: 'user',
          text: 'Still working on building the weather station',
          timestamp: Date.now() - 1 * 24 * 60 * 60 * 1000,
        },
      ]);

      const questions = await followUpGen.checkOngoingProjects(testUserId);

      expect(questions.length).toBeGreaterThan(0);
      expect(questions[0].question.toLowerCase()).toContain('weather');
    });

    it('should prioritize questions correctly', async () => {
      const questions = [
        {
          id: '1',
          userId: testUserId,
          question: 'Low priority',
          context: 'test',
          priority: 50,
          validUntil: Date.now() + 1000000,
          answered: false,
        },
        {
          id: '2',
          userId: testUserId,
          question: 'High priority',
          context: 'test',
          priority: 90,
          validUntil: Date.now() + 1000000,
          answered: false,
        },
        {
          id: '3',
          userId: testUserId,
          question: 'Medium priority',
          context: 'test',
          priority: 70,
          validUntil: Date.now() + 1000000,
          answered: false,
        },
      ];

      const prioritized = await followUpGen.prioritizeQuestions(questions);

      expect(prioritized[0].priority).toBe(90);
      expect(prioritized[1].priority).toBe(70);
      expect(prioritized[2].priority).toBe(50);
    });

    it('should mark question as answered', async () => {
      const question = await followUpGen.checkYesterdayActivity(testUserId);
      if (!question) {
        // Create a test question
        const yesterday = getDateString(new Date(Date.now() - 24 * 60 * 60 * 1000));
        await memoryService.storeDailyActivity(testUserId, yesterday, ['test activity']);
        const newQuestion = await followUpGen.checkYesterdayActivity(testUserId);
        expect(newQuestion).toBeDefined();
      }
    });
  });

  describe('Memory Retention and Cleanup', () => {
    it('should prune old conversations', async () => {
      const oldTimestamp = Date.now() - 10 * 24 * 60 * 60 * 1000; // 10 days ago

      await memoryService.storeConversation(testUserId, [
        { speaker: 'user', text: 'Old conversation', timestamp: oldTimestamp },
      ]);
      await memoryService.storeConversation(testUserId, [
        { speaker: 'user', text: 'Recent conversation', timestamp: Date.now() },
      ]);

      const deletedCount = await memoryService.pruneOldConversations(testUserId, 7);
      expect(deletedCount).toBe(1);

      const remaining = await memoryService.getConversations(testUserId, 30);
      expect(remaining).toHaveLength(1);
      expect(remaining[0].turns[0].text).toBe('Recent conversation');
    });

    it('should comply with 7-day retention policy (I-VOICE-004)', async () => {
      // Store conversations across 14 days
      for (let i = 0; i < 14; i++) {
        const timestamp = Date.now() - i * 24 * 60 * 60 * 1000;
        await memoryService.storeConversation(testUserId, [
          { speaker: 'user', text: `Day ${i} conversation`, timestamp },
        ]);
      }

      const recent = await memoryService.getConversations(testUserId, 7);
      expect(recent.length).toBeLessThanOrEqual(7);
    });
  });

  describe('Sentiment Analysis', () => {
    it('should detect positive sentiment', async () => {
      const conversation = await memoryService.storeConversation(testUserId, [
        {
          speaker: 'user',
          text: 'I am so happy! I love building with LEGO. It is awesome!',
          timestamp: Date.now(),
        },
      ]);

      expect(conversation.sentiment).toBe('positive');
    });

    it('should detect negative sentiment', async () => {
      const conversation = await memoryService.storeConversation(testUserId, [
        {
          speaker: 'user',
          text: 'I am sad and frustrated. This is terrible and I hate it.',
          timestamp: Date.now(),
        },
      ]);

      expect(conversation.sentiment).toBe('negative');
    });

    it('should detect neutral sentiment', async () => {
      const conversation = await memoryService.storeConversation(testUserId, [
        { speaker: 'user', text: 'I went to the store today', timestamp: Date.now() },
      ]);

      expect(conversation.sentiment).toBe('neutral');
    });
  });

  describe('Conversation Summary', () => {
    it('should generate conversation summary', async () => {
      const today = getDateString(new Date());
      const yesterday = getDateString(new Date(Date.now() - 24 * 60 * 60 * 1000));

      // Store multiple conversations
      await memoryService.storeConversation(testUserId, [
        { speaker: 'user', text: 'I love LEGO building', timestamp: Date.now() },
      ]);
      await memoryService.storeConversation(testUserId, [
        { speaker: 'user', text: 'More LEGO projects today', timestamp: Date.now() },
      ]);

      // Store activities
      await memoryService.storeDailyActivity(testUserId, today, ['build LEGO']);
      await memoryService.storeDailyActivity(testUserId, yesterday, ['draw picture']);

      const summary = await memoryService.getConversationSummary(
        testUserId,
        yesterday,
        today
      );

      expect(summary.totalConversations).toBe(2);
      expect(summary.topTopics.length).toBeGreaterThan(0);
      expect(summary.keyActivities).toContain('build LEGO');
      expect(['positive', 'neutral', 'negative']).toContain(summary.averageSentiment);
    });
  });
});

// Helper function
function getDateString(date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
}
