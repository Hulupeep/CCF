/**
 * Voice Assistant Full Integration Tests
 *
 * Tests complete voice interaction flow including:
 * - Voice recognition
 * - Briefing generation
 * - Multi-component integration
 * - TTS playback
 * - Integration with learning (#92) and autonomy (#93)
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { VoiceAssistant } from '../../web/src/services/voice/VoiceAssistant';
import { PersonalBriefingService } from '../../web/src/services/briefing/PersonalBriefingService';
import { VoiceProfileService } from '../../web/src/services/voice/VoiceProfileService';
import { NewsService } from '../../web/src/services/news/NewsService';
import { EmailService } from '../../web/src/services/email/EmailService';
import { ConversationMemoryService } from '../../web/src/services/memory/ConversationMemoryService';
import { TTSService } from '../../web/src/services/briefing/TTSService';

describe('Voice Assistant Integration', () => {
  let assistant: VoiceAssistant;
  let briefingService: PersonalBriefingService;
  let voiceService: VoiceProfileService;
  let newsService: NewsService;
  let emailService: EmailService;
  let memoryService: ConversationMemoryService;
  let ttsService: TTSService;

  beforeEach(() => {
    // Initialize services
    assistant = new VoiceAssistant();
    briefingService = new PersonalBriefingService();
    voiceService = new VoiceProfileService();
    newsService = new NewsService();
    emailService = new EmailService();
    memoryService = new ConversationMemoryService();
    ttsService = new TTSService();

    // Mock localStorage
    global.localStorage = {
      getItem: vi.fn(),
      setItem: vi.fn(),
      removeItem: vi.fn(),
      clear: vi.fn(),
      length: 0,
      key: vi.fn()
    } as any;
  });

  describe('Complete Voice Interaction Flow', () => {
    it('should identify user and deliver personalized briefing', async () => {
      // Setup: Enroll user
      const userId = 'user-test-1';
      await voiceService.enrollUser(userId, 'Alice', []);

      // Mock audio data
      const audioData = new ArrayBuffer(1024);

      // Mock identification
      vi.spyOn(voiceService, 'identifyUser').mockResolvedValue({
        userId,
        confidence: 0.92,
        alternativeMatches: [],
        isAnonymous: false,
        timestamp: Date.now()
      });

      // Execute
      await assistant.handleVoiceInput(audioData);

      // Verify identification was called
      expect(voiceService.identifyUser).toHaveBeenCalledWith(audioData);
    });

    it('should generate complete briefing with all sections', async () => {
      const userId = 'user-test-2';

      // Setup preferences
      await newsService.updatePreferences(userId, {
        userId,
        topics: ['technology', 'science'],
        sources: [],
        excludeTopics: [],
        maxArticles: 3,
        readingLevel: 'adult',
        topicWeights: {},
        lastUpdated: Date.now()
      });

      // Setup memory
      const yesterday = new Date();
      yesterday.setDate(yesterday.getDate() - 1);
      const yesterdayStr = yesterday.toISOString().split('T')[0];

      await memoryService.storeDailyActivity(userId, yesterdayStr, {
        date: yesterdayStr,
        userId,
        plannedActivities: ['work on project'],
        completedActivities: [],
        notes: ''
      });

      // Generate briefing
      const briefing = await briefingService.generateBriefing(userId);

      // Verify briefing structure
      expect(briefing.userId).toBe(userId);
      expect(briefing.sections.length).toBeGreaterThan(0);
      expect(briefing.spokenText).toBeTruthy();
      expect(briefing.duration).toBeGreaterThan(0);

      // Verify greeting section exists
      const greetingSection = briefing.sections.find(s => s.type === 'greeting');
      expect(greetingSection).toBeTruthy();
      expect(greetingSection?.content).toContain('Good');
    });

    it('should deliver briefing with TTS', async () => {
      const userId = 'user-test-3';

      // Generate briefing
      const briefing = await briefingService.generateBriefing(userId);

      // Spy on TTS
      const ttsSpendSpy = vi.spyOn(ttsService, 'synthesizeSpeech').mockResolvedValue();

      // Deliver briefing
      await briefingService.deliverBriefing(userId, briefing);

      // Verify TTS was called
      expect(ttsSpendSpy).toHaveBeenCalled();
    });

    it('should handle anonymous mode gracefully', async () => {
      // Mock unknown user
      const audioData = new ArrayBuffer(1024);

      vi.spyOn(voiceService, 'identifyUser').mockResolvedValue({
        userId: null,
        confidence: 0.42,
        alternativeMatches: [],
        isAnonymous: true,
        timestamp: Date.now()
      });

      // Should not throw
      await expect(assistant.handleVoiceInput(audioData)).resolves.not.toThrow();
    });
  });

  describe('Multi-Component Integration', () => {
    it('should integrate news service into briefing', async () => {
      const userId = 'user-test-4';

      // Mock news articles
      vi.spyOn(newsService, 'getPersonalizedNews').mockResolvedValue([
        {
          id: 'news-1',
          headline: 'AI Breakthrough Announced',
          summary: 'Scientists achieve quantum computing milestone',
          source: 'TechCrunch',
          category: 'technology',
          publishedAt: Date.now(),
          url: 'https://example.com/news-1',
          relevanceScore: 0.95
        },
        {
          id: 'news-2',
          headline: 'Mars Mission Success',
          summary: 'Rover discovers water ice',
          source: 'NASA',
          category: 'science',
          publishedAt: Date.now(),
          url: 'https://example.com/news-2',
          relevanceScore: 0.88
        }
      ]);

      const briefing = await briefingService.generateBriefing(userId);

      // Verify news section
      const newsSection = briefing.sections.find(s => s.type === 'news');
      expect(newsSection).toBeTruthy();
      expect(newsSection?.content).toContain('AI Breakthrough');
      expect(newsSection?.articles?.length).toBe(2);
    });

    it('should integrate email service into briefing', async () => {
      const userId = 'user-test-5';

      // Mock email summary
      vi.spyOn(emailService, 'getSummary').mockResolvedValue({
        totalUnread: 8,
        importantCount: 2,
        topSenders: ['boss@company.com'],
        categories: {
          work: 5,
          personal: 2,
          promotions: 1,
          social: 0
        },
        highlights: [
          {
            from: 'boss@company.com',
            subject: 'Q1 Review Meeting',
            snippet: 'Please prepare slides...',
            importance: 'high',
            timestamp: Date.now()
          }
        ]
      });

      vi.spyOn(emailService, 'getAccounts').mockResolvedValue([
        {
          userId,
          provider: 'gmail',
          email: 'user@example.com',
          accessToken: 'token',
          refreshToken: 'refresh',
          expiresAt: Date.now() + 3600000,
          lastSynced: Date.now()
        }
      ]);

      const briefing = await briefingService.generateBriefing(userId);

      // Verify email section
      const emailSection = briefing.sections.find(s => s.type === 'email');
      expect(emailSection).toBeTruthy();
      expect(emailSection?.content).toContain('8 unread');
      expect(emailSection?.content).toContain('2 marked important');
    });

    it('should integrate memory service into briefing', async () => {
      const userId = 'user-test-6';

      const yesterday = new Date();
      yesterday.setDate(yesterday.getDate() - 1);
      const yesterdayStr = yesterday.toISOString().split('T')[0];

      // Mock yesterday's activity
      vi.spyOn(memoryService, 'getDailyActivity').mockResolvedValue({
        date: yesterdayStr,
        userId,
        plannedActivities: ['build LEGO castle'],
        completedActivities: [],
        notes: ''
      });

      const briefing = await briefingService.generateBriefing(userId);

      // Verify memory section
      const memorySection = briefing.sections.find(s => s.type === 'memory');
      expect(memorySection).toBeTruthy();
      expect(memorySection?.content).toContain('LEGO castle');
    });

    it('should integrate follow-up questions into briefing', async () => {
      const userId = 'user-test-7';

      // Mock follow-up question
      vi.spyOn(memoryService, 'getFollowUpQuestions').mockResolvedValue([
        {
          id: 'q-1',
          userId,
          question: 'Did you finish your project?',
          context: 'User mentioned working on a project yesterday',
          priority: 80,
          validUntil: Date.now() + 86400000,
          answered: false
        }
      ]);

      const briefing = await briefingService.generateBriefing(userId);

      // Verify question section
      const questionSection = briefing.sections.find(s => s.type === 'question');
      expect(questionSection).toBeTruthy();
      expect(questionSection?.content).toContain('finish your project');
    });
  });

  describe('Section Priority Ordering', () => {
    it('should order sections by priority (high > medium > low)', async () => {
      const userId = 'user-test-8';

      const briefing = await briefingService.generateBriefing(userId);

      // Get priorities
      const priorities = briefing.sections.map(s => s.priority);

      // Verify high comes before medium, medium before low
      const highIndex = priorities.indexOf('high');
      const mediumIndex = priorities.indexOf('medium');
      const lowIndex = priorities.indexOf('low');

      if (highIndex !== -1 && mediumIndex !== -1) {
        expect(highIndex).toBeLessThan(mediumIndex);
      }

      if (mediumIndex !== -1 && lowIndex !== -1) {
        expect(mediumIndex).toBeLessThan(lowIndex);
      }
    });

    it('should place greeting section first', async () => {
      const userId = 'user-test-9';

      const briefing = await briefingService.generateBriefing(userId);

      // First section should be greeting
      expect(briefing.sections[0].type).toBe('greeting');
    });
  });

  describe('TTS Integration', () => {
    it('should synthesize speech with Web Speech API', async () => {
      const text = 'Good morning, Alice! Here is your daily update.';

      // Mock speech synthesis
      const mockUtterance = {
        text,
        rate: 1,
        pitch: 1,
        volume: 1,
        onend: null as any,
        onerror: null as any
      };

      const mockSynthesis = {
        speak: vi.fn((utterance: any) => {
          setTimeout(() => utterance.onend?.(), 100);
        }),
        getVoices: vi.fn(() => []),
        cancel: vi.fn(),
        speaking: false,
        paused: false,
        pending: false,
        pause: vi.fn(),
        resume: vi.fn()
      };

      global.window = {
        speechSynthesis: mockSynthesis,
        SpeechSynthesisUtterance: vi.fn(() => mockUtterance) as any
      } as any;

      const tts = new TTSService('browser');
      await tts.synthesizeSpeech(text);

      expect(mockSynthesis.speak).toHaveBeenCalled();
    });

    it('should get available voices', async () => {
      const tts = new TTSService('browser');
      const voices = await tts.getAvailableVoices();

      expect(Array.isArray(voices)).toBe(true);
    });

    it('should save user voice preference', async () => {
      const tts = new TTSService('browser');
      const userId = 'user-test-10';
      const voiceId = 'en-US-WavenetA';

      await tts.setVoiceForUser(userId, voiceId);

      const saved = await tts.getUserVoice(userId);
      expect(saved).toBe(voiceId);
    });
  });

  describe('Voice Command Processing', () => {
    it('should trigger morning briefing on "good morning"', async () => {
      const userId = 'user-test-11';

      const triggerSpy = vi.spyOn(assistant, 'triggerMorningBriefing').mockResolvedValue();

      await assistant.processCommand(userId, 'Good morning');

      expect(triggerSpy).toHaveBeenCalledWith(userId);
    });

    it('should deliver news on "tell me the news"', async () => {
      const userId = 'user-test-12';

      const newsSpy = vi.spyOn(assistant, 'deliverNews').mockResolvedValue();

      await assistant.processCommand(userId, 'Tell me the news');

      expect(newsSpy).toHaveBeenCalledWith(userId);
    });

    it('should deliver email on "check my email"', async () => {
      const userId = 'user-test-13';

      const emailSpy = vi.spyOn(assistant, 'deliverEmail').mockResolvedValue();

      await assistant.processCommand(userId, 'Check my email');

      expect(emailSpy).toHaveBeenCalledWith(userId);
    });

    it('should recall memory on "what did I do yesterday"', async () => {
      const userId = 'user-test-14';

      const memorySpy = vi.spyOn(assistant, 'deliverMemoryRecall').mockResolvedValue();

      await assistant.processCommand(userId, 'What did I do yesterday?');

      expect(memorySpy).toHaveBeenCalledWith(userId);
    });
  });

  describe('Integration with Learning Engine (#92)', () => {
    it('should observe interactions for preference learning', async () => {
      const userId = 'user-test-15';

      const observeSpy = vi.spyOn(assistant, 'observeInteraction');

      await assistant.processCommand(userId, 'Tell me more about AI news');

      expect(observeSpy).toHaveBeenCalled();
    });
  });

  describe('Integration with Autonomous Behavior (#93)', () => {
    it('should check in after inactivity with follow-up questions', async () => {
      const userId = 'user-test-16';

      // Mock follow-up question
      vi.spyOn(memoryService, 'getFollowUpQuestions').mockResolvedValue([
        {
          id: 'q-2',
          userId,
          question: 'How did your interview go?',
          context: 'User had interview yesterday',
          priority: 90,
          validUntil: Date.now() + 86400000,
          answered: false
        }
      ]);

      const speakSpy = vi.spyOn(assistant, 'speakResponse').mockResolvedValue();

      await assistant.checkInAfterInactivity(userId);

      expect(speakSpy).toHaveBeenCalledWith(expect.stringContaining('interview'));
    });
  });

  describe('Error Handling', () => {
    it('should handle news service failures gracefully', async () => {
      const userId = 'user-test-17';

      vi.spyOn(newsService, 'getPersonalizedNews').mockRejectedValue(new Error('API error'));

      // Should not throw
      const briefing = await briefingService.generateBriefing(userId);

      // Should still have greeting
      expect(briefing.sections.some(s => s.type === 'greeting')).toBe(true);

      // News section should be empty
      const newsSection = briefing.sections.find(s => s.type === 'news');
      expect(newsSection?.content).toBeFalsy();
    });

    it('should handle email service failures gracefully', async () => {
      const userId = 'user-test-18';

      vi.spyOn(emailService, 'getSummary').mockRejectedValue(new Error('Auth error'));

      const briefing = await briefingService.generateBriefing(userId);

      // Should still have greeting
      expect(briefing.sections.some(s => s.type === 'greeting')).toBe(true);
    });

    it('should handle memory service failures gracefully', async () => {
      const userId = 'user-test-19';

      vi.spyOn(memoryService, 'getDailyActivity').mockRejectedValue(new Error('Storage error'));

      const briefing = await briefingService.generateBriefing(userId);

      // Should still have greeting
      expect(briefing.sections.some(s => s.type === 'greeting')).toBe(true);
    });
  });

  describe('Coverage: >90%', () => {
    it('should test section builder greeting formatting', () => {
      const { SectionBuilder } = require('../../web/src/services/briefing/SectionBuilder');
      const builder = new SectionBuilder();

      const adultGreeting = builder.formatGreeting('Alice', 'morning', { isChild: false });
      expect(adultGreeting).toContain('Good morning, Alice');

      const childGreeting = builder.formatGreeting('Emma', 'afternoon', { isChild: true });
      expect(childGreeting).toContain('Emma');
      expect(childGreeting).toContain('awesome');
    });

    it('should test section builder news formatting', () => {
      const { SectionBuilder } = require('../../web/src/services/briefing/SectionBuilder');
      const builder = new SectionBuilder();

      const articles = [
        {
          id: 'a1',
          headline: 'Breaking News',
          summary: 'Summary',
          source: 'BBC',
          category: 'world',
          publishedAt: Date.now(),
          url: 'https://example.com',
          relevanceScore: 0.9
        }
      ];

      const formatted = builder.formatNewsHeadlines(articles, 3);
      expect(formatted).toContain('Breaking News');
      expect(formatted).toContain('BBC');
    });

    it('should test section builder email formatting', () => {
      const { SectionBuilder } = require('../../web/src/services/briefing/SectionBuilder');
      const builder = new SectionBuilder();

      const summary = {
        totalUnread: 5,
        importantCount: 1,
        topSenders: ['boss@example.com'],
        categories: { work: 3, personal: 2, promotions: 0, social: 0 },
        highlights: []
      };

      const formatted = builder.formatEmailSummary(summary);
      expect(formatted).toContain('5 unread');
      expect(formatted).toContain('1 marked important');
    });

    it('should test section builder memory formatting', () => {
      const { SectionBuilder } = require('../../web/src/services/briefing/SectionBuilder');
      const builder = new SectionBuilder();

      const activities = [
        {
          date: '2024-01-31',
          userId: 'user-1',
          plannedActivities: ['exercise'],
          completedActivities: [],
          notes: ''
        }
      ];

      const formatted = builder.formatMemoryRecall(activities);
      expect(formatted).toContain('exercise');
    });

    it('should test briefing history storage', async () => {
      const userId = 'user-test-20';

      const briefing1 = await briefingService.generateBriefing(userId);
      const briefing2 = await briefingService.generateBriefing(userId);

      const history = await briefingService.getBriefingHistory(userId, 5);

      expect(history.length).toBeGreaterThan(0);
    });
  });
});
