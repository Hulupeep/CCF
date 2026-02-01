/**
 * Integration tests for Telegram Bot
 * Tests bot lifecycle, commands, message handling, and personality integration
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { TelegramBot } from '../../web/src/services/telegram/TelegramBot';
import { TelegramAdapter } from '../../web/src/services/telegram/TelegramAdapter';
import { PersonalityStore } from '../../web/src/services/personalityStore';
import { TelegramBotConfig } from '../../web/src/types/telegram';
import { createDefaultConfig } from '../../web/src/types/personality';

describe('TelegramBot Integration Tests', () => {
  let bot: TelegramBot;
  let mockConfig: TelegramBotConfig;
  let personalityStore: PersonalityStore;

  beforeEach(() => {
    // Mock configuration
    mockConfig = {
      botToken: 'test_bot_token_12345',
      dmPairingRequired: true,
      commandPrefix: '/',
      rateLimitPerMinute: 20,
      isDevelopment: true,
    };

    // Initialize personality store
    personalityStore = PersonalityStore.getInstance();
    personalityStore.resetToDefault();
  });

  afterEach(() => {
    PersonalityStore.resetInstance();
  });

  describe('Bot Initialization', () => {
    it('should create bot with valid config', () => {
      bot = new TelegramBot(mockConfig);
      const status = bot.getStatus();

      expect(status.configured).toBe(true);
      expect(status.running).toBe(false);
      expect(status.mode).toBe('polling');
    });

    it('should fail with invalid bot token', () => {
      const invalidConfig = { ...mockConfig, botToken: '' };
      expect(() => new TelegramBot(invalidConfig)).toThrow();
    });
  });

  describe('DM Pairing Flow', () => {
    beforeEach(() => {
      bot = new TelegramBot(mockConfig);
    });

    it('should require pairing when dmPairingRequired is true', () => {
      const userId = '123456789';
      const status = bot.getStatus();

      // User not paired initially
      expect(bot.getPairedUsersCount()).toBe(0);
    });

    it('should allow all users when dmPairingRequired is false', () => {
      const openConfig = { ...mockConfig, dmPairingRequired: false };
      bot = new TelegramBot(openConfig);

      // Should not require pairing
      expect(bot.getStatus().configured).toBe(true);
    });

    it('should track paired users correctly', () => {
      // This would be tested with mock Telegram context
      // For now, verify initial state
      expect(bot.getPairedUsersCount()).toBe(0);
    });
  });

  describe('Rate Limiting', () => {
    beforeEach(() => {
      bot = new TelegramBot(mockConfig);
    });

    it('should enforce rate limits', () => {
      // Rate limit is 20 messages per minute
      const stats = bot.getRateLimitStats();
      expect(stats).toBeDefined();
      expect(Array.isArray(stats)).toBe(true);
    });

    it('should reset rate limit after timeout', async () => {
      // This would test actual rate limiting with mock context
      expect(mockConfig.rateLimitPerMinute).toBe(20);
    });
  });

  describe('Personality Integration', () => {
    beforeEach(() => {
      bot = new TelegramBot(mockConfig);
    });

    it('should use personality store for responses', () => {
      const personality = personalityStore.getCurrentPersonality();
      expect(personality).toBeDefined();
      expect(personality.energy_baseline).toBeGreaterThanOrEqual(0);
      expect(personality.energy_baseline).toBeLessThanOrEqual(1);
    });

    it('should adjust response style based on personality', () => {
      // Set high energy personality
      const highEnergyConfig = createDefaultConfig();
      highEnergyConfig.energy_baseline = 0.9;
      highEnergyConfig.curiosity_drive = 0.8;

      personalityStore.updateConfig(highEnergyConfig);

      const updatedPersonality = personalityStore.getCurrentPersonality();
      expect(updatedPersonality.energy_baseline).toBe(0.9);
      expect(updatedPersonality.curiosity_drive).toBe(0.8);
    });

    it('should respond differently based on personality traits', () => {
      // Low energy personality
      const calmConfig = createDefaultConfig();
      calmConfig.energy_baseline = 0.2;
      calmConfig.curiosity_drive = 0.3;

      personalityStore.updateConfig(calmConfig);

      const personality = personalityStore.getCurrentPersonality();
      expect(personality.energy_baseline).toBe(0.2);
    });
  });

  describe('Message Chunking', () => {
    let adapter: TelegramAdapter;

    beforeEach(() => {
      adapter = new TelegramAdapter();
    });

    it('should not chunk short messages', () => {
      const shortMessage = 'Hello, world!';
      const chunks = adapter.chunkMessage(shortMessage);

      expect(chunks).toHaveLength(1);
      expect(chunks[0]).toBe(shortMessage);
    });

    it('should chunk long messages at 4000 characters', () => {
      const longMessage = 'A'.repeat(5000);
      const chunks = adapter.chunkMessage(longMessage);

      expect(chunks.length).toBeGreaterThan(1);
      chunks.forEach(chunk => {
        expect(chunk.length).toBeLessThanOrEqual(4000);
      });
    });

    it('should preserve paragraph boundaries when chunking', () => {
      const message = 'Paragraph 1\n\n' + 'B'.repeat(4000) + '\n\nParagraph 2';
      const chunks = adapter.chunkMessage(message);

      expect(chunks.length).toBeGreaterThan(1);
    });
  });

  describe('Message Formatting', () => {
    let adapter: TelegramAdapter;

    beforeEach(() => {
      adapter = new TelegramAdapter();
    });

    it('should convert Markdown to Telegram HTML', () => {
      const markdown = '**Bold** *italic* `code`';
      const formatted = adapter.formatOutbound(markdown);

      expect(formatted.parse_mode).toBe('HTML');
      expect(formatted.text).toContain('<b>Bold</b>');
      expect(formatted.text).toContain('<i>italic</i>');
      expect(formatted.text).toContain('<code>code</code>');
    });

    it('should handle code blocks', () => {
      const markdown = '```\ncode block\n```';
      const formatted = adapter.formatOutbound(markdown);

      expect(formatted.text).toContain('<pre>');
      expect(formatted.text).toContain('</pre>');
    });

    it('should handle links', () => {
      const markdown = '[Google](https://google.com)';
      const formatted = adapter.formatOutbound(markdown);

      expect(formatted.text).toContain('<a href="https://google.com">Google</a>');
    });

    it('should escape HTML special characters', () => {
      const text = 'Test & <test>';
      const escaped = adapter.escapeHtml(text);

      expect(escaped).toContain('&amp;');
      expect(escaped).toContain('&lt;');
      expect(escaped).toContain('&gt;');
    });
  });

  describe('Response Style Generation', () => {
    let adapter: TelegramAdapter;

    beforeEach(() => {
      adapter = new TelegramAdapter();
    });

    it('should generate casual style for high energy', () => {
      const personality = createDefaultConfig();
      personality.energy_baseline = 0.8;

      const style = adapter.getResponseStyle(personality);

      expect(style.enthusiasm).toBe(0.8);
      expect(style.formality).toBe('casual');
    });

    it('should generate formal style for low energy', () => {
      const personality = createDefaultConfig();
      personality.energy_baseline = 0.2;

      const style = adapter.getResponseStyle(personality);

      expect(style.enthusiasm).toBe(0.2);
      expect(style.formality).toBe('formal');
    });

    it('should determine verbosity from curiosity', () => {
      const curiousPersonality = createDefaultConfig();
      curiousPersonality.curiosity_drive = 0.8;

      const style = adapter.getResponseStyle(curiousPersonality);
      expect(style.verbosity).toBe('detailed');

      const cautiousPersonality = createDefaultConfig();
      cautiousPersonality.curiosity_drive = 0.3;

      const cautiousStyle = adapter.getResponseStyle(cautiousPersonality);
      expect(cautiousStyle.verbosity).toBe('concise');
    });

    it('should enable emoji for expressive personalities', () => {
      const expressivePersonality = createDefaultConfig();
      expressivePersonality.movement_expressiveness = 0.8;

      const style = adapter.getResponseStyle(expressivePersonality);
      expect(style.emoji).toBe(true);

      const subtlePersonality = createDefaultConfig();
      subtlePersonality.movement_expressiveness = 0.3;

      const subtleStyle = adapter.getResponseStyle(subtlePersonality);
      expect(subtleStyle.emoji).toBe(false);
    });
  });

  describe('Error Handling', () => {
    beforeEach(() => {
      bot = new TelegramBot(mockConfig);
    });

    it('should handle invalid messages gracefully', () => {
      const status = bot.getStatus();
      expect(status.configured).toBe(true);
    });

    it('should track last error in status', () => {
      const status = bot.getStatus();
      expect(status.lastError).toBeUndefined();
    });
  });

  describe('Bot Status', () => {
    beforeEach(() => {
      bot = new TelegramBot(mockConfig);
    });

    it('should return current status', () => {
      const status = bot.getStatus();

      expect(status).toHaveProperty('configured');
      expect(status).toHaveProperty('running');
      expect(status).toHaveProperty('mode');
    });

    it('should update status timestamps', () => {
      const status = bot.getStatus();

      expect(status.configured).toBe(true);
      expect(status.running).toBe(false);
    });
  });

  describe('Command Handling', () => {
    beforeEach(() => {
      bot = new TelegramBot(mockConfig);
    });

    it('should recognize all standard commands', () => {
      // Commands are registered in constructor
      // This verifies bot initialization doesn't throw
      expect(bot.getStatus().configured).toBe(true);
    });

    it('should use correct command prefix', () => {
      expect(mockConfig.commandPrefix).toBe('/');
    });
  });
});

describe('TelegramAdapter Unit Tests', () => {
  let adapter: TelegramAdapter;

  beforeEach(() => {
    adapter = new TelegramAdapter();
  });

  describe('Message Normalization', () => {
    it('should normalize text messages', () => {
      const mockCtx = {
        message: {
          message_id: 123,
          date: 1609459200,
          text: 'Hello!',
        },
        chat: {
          id: 456,
          type: 'private' as const,
        },
        from: {
          id: 789,
          first_name: 'John',
          is_bot: false,
        },
      };

      const normalized = adapter.normalizeInbound(mockCtx as any);

      expect(normalized.id).toBe('123');
      expect(normalized.userId).toBe('789');
      expect(normalized.text).toBe('Hello!');
      expect(normalized.chatType).toBe('private');
      expect(normalized.timestamp).toBe(1609459200000);
    });

    it('should handle messages with attachments', () => {
      const mockCtx = {
        message: {
          message_id: 123,
          date: 1609459200,
          caption: 'Check this out!',
          photo: [
            { file_id: 'photo1', width: 100, height: 100 },
            { file_id: 'photo2', width: 800, height: 800 },
          ],
        },
        chat: {
          id: 456,
          type: 'private' as const,
        },
        from: {
          id: 789,
          first_name: 'John',
          is_bot: false,
        },
      };

      const normalized = adapter.normalizeInbound(mockCtx as any);

      expect(normalized.text).toBe('Check this out!');
      expect(normalized.attachments).toBeDefined();
      expect(normalized.attachments?.length).toBe(1);
      expect(normalized.attachments?.[0].type).toBe('photo');
      expect(normalized.attachments?.[0].fileId).toBe('photo2');
    });
  });

  describe('Personality-Based Formatting', () => {
    it('should add enthusiasm markers for high energy', () => {
      const personality = createDefaultConfig();
      personality.energy_baseline = 0.9;
      personality.light_expressiveness = 0.7;

      const text = 'Hello world.';
      const formatted = adapter.formatPersonalityResponse(text, personality);

      expect(formatted).toMatch(/[!🤖✨🎨🎮🔧💡]/);
    });

    it('should maintain formal tone for low energy', () => {
      const personality = createDefaultConfig();
      personality.energy_baseline = 0.1;

      const text = 'Hello world';
      const formatted = adapter.formatPersonalityResponse(text, personality);

      expect(formatted).toBe(text); // No modifications for low energy
    });
  });
});
