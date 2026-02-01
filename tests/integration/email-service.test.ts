/**
 * Email Service Integration Tests
 *
 * Contract: I-EMAIL-001
 * Issue: #95 Component 3/5
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { EmailService, Email } from '../../web/src/services/email/EmailService';
import { EmailAccount } from '../../web/src/types/voice';

// Mock fetch globally
global.fetch = vi.fn();

describe('EmailService Integration Tests', () => {
  let emailService: EmailService;
  let mockAccount: EmailAccount;

  beforeEach(() => {
    emailService = new EmailService();
    mockAccount = {
      userId: 'user123',
      provider: 'gmail',
      email: 'test@example.com',
      accessToken: Buffer.from('mock_access_token').toString('base64'),
      refreshToken: Buffer.from('mock_refresh_token').toString('base64'),
      expiresAt: Date.now() + 3600 * 1000,
      lastSynced: Date.now(),
    };

    vi.clearAllMocks();
  });

  describe('OAuth2 Flow', () => {
    it('should connect Gmail account via OAuth2', async () => {
      const mockTokenResponse = {
        access_token: 'new_access_token',
        refresh_token: 'new_refresh_token',
        expires_in: 3600,
        scope: 'email',
        token_type: 'Bearer',
      };

      const mockProfileResponse = {
        emailAddress: 'test@gmail.com',
      };

      (global.fetch as any)
        .mockResolvedValueOnce({
          ok: true,
          json: async () => mockTokenResponse,
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => mockProfileResponse,
        });

      const account = await emailService.connectAccount('gmail', 'auth_code_123', 'user123');

      expect(account.provider).toBe('gmail');
      expect(account.email).toBe('test@gmail.com');
      expect(account.userId).toBe('user123');
      expect(account.accessToken).toBeDefined();
      expect(account.refreshToken).toBeDefined();
    });

    it('should connect Outlook account via OAuth2', async () => {
      const mockTokenResponse = {
        access_token: 'outlook_token',
        refresh_token: 'outlook_refresh',
        expires_in: 3600,
        scope: 'Mail.Read',
        token_type: 'Bearer',
      };

      const mockUserResponse = {
        mail: 'test@outlook.com',
      };

      (global.fetch as any)
        .mockResolvedValueOnce({
          ok: true,
          json: async () => mockTokenResponse,
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => mockUserResponse,
        });

      const account = await emailService.connectAccount('outlook', 'auth_code_456', 'user456');

      expect(account.provider).toBe('outlook');
      expect(account.email).toBe('test@outlook.com');
      expect(account.userId).toBe('user456');
    });

    it('should handle OAuth2 errors gracefully', async () => {
      (global.fetch as any).mockResolvedValueOnce({
        ok: false,
        json: async () => ({ error: 'invalid_grant', error_description: 'Invalid code' }),
      });

      await expect(
        emailService.connectAccount('gmail', 'invalid_code', 'user123')
      ).rejects.toThrow('Email connection failed');
    });
  });

  describe('Token Refresh', () => {
    it('should refresh expired access token', async () => {
      const expiredAccount = {
        ...mockAccount,
        expiresAt: Date.now() - 1000, // Expired 1 second ago
      };

      const mockRefreshResponse = {
        access_token: 'refreshed_token',
        refresh_token: 'refreshed_refresh_token',
        expires_in: 3600,
      };

      (global.fetch as any).mockResolvedValueOnce({
        ok: true,
        json: async () => mockRefreshResponse,
      });

      const newToken = await emailService.refreshToken(expiredAccount);

      expect(newToken).toBeDefined();
      expect(global.fetch).toHaveBeenCalledWith(
        expect.stringContaining('oauth2.googleapis.com/token'),
        expect.objectContaining({
          method: 'POST',
        })
      );
    });

    it('should handle token refresh failures', async () => {
      (global.fetch as any).mockResolvedValueOnce({
        ok: false,
        json: async () => ({ error: 'invalid_grant' }),
      });

      await expect(emailService.refreshToken(mockAccount)).rejects.toThrow('Token refresh failed');
    });
  });

  describe('Email Fetching', () => {
    it('should fetch unread emails from Gmail', async () => {
      const mockMessagesResponse = {
        messages: [{ id: 'msg1' }, { id: 'msg2' }],
      };

      const mockMessageResponse = {
        id: 'msg1',
        threadId: 'thread1',
        labelIds: ['UNREAD', 'INBOX'],
        snippet: 'Test email snippet',
        payload: {
          headers: [
            { name: 'From', value: 'sender@example.com' },
            { name: 'To', value: 'test@gmail.com' },
            { name: 'Subject', value: 'Test Email' },
          ],
        },
        internalDate: String(Date.now()),
      };

      (global.fetch as any)
        .mockResolvedValueOnce({
          ok: true,
          json: async () => mockMessagesResponse,
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => mockMessageResponse,
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => ({ ...mockMessageResponse, id: 'msg2' }),
        });

      const emails = await emailService.fetchUnreadEmails(mockAccount, 20);

      expect(emails).toHaveLength(2);
      expect(emails[0].subject).toBe('Test Email');
      expect(emails[0].from).toBe('sender@example.com');
    });

    it('should auto-refresh token if expired', async () => {
      const expiredAccount = {
        ...mockAccount,
        expiresAt: Date.now() - 1000,
      };

      const mockRefreshResponse = {
        access_token: 'new_token',
        refresh_token: 'new_refresh',
        expires_in: 3600,
      };

      const mockMessagesResponse = {
        messages: [],
      };

      (global.fetch as any)
        .mockResolvedValueOnce({
          ok: true,
          json: async () => mockRefreshResponse,
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => mockMessagesResponse,
        });

      await emailService.fetchUnreadEmails(expiredAccount, 20);

      expect(global.fetch).toHaveBeenCalledTimes(2);
      // First call: token refresh
      // Second call: fetch messages
    });
  });

  describe('Email Summary Generation', () => {
    it('should generate email summary with categories', async () => {
      const mockEmails: Email[] = [
        {
          id: '1',
          from: 'boss@company.com',
          to: 'test@example.com',
          subject: 'Urgent: Q1 Report',
          snippet: 'Please review',
          timestamp: Date.now(),
          isRead: false,
          isStarred: true,
          importance: 'high',
        },
        {
          id: '2',
          from: 'friend@example.com',
          to: 'test@example.com',
          subject: 'Lunch plans',
          snippet: 'Want to grab lunch?',
          timestamp: Date.now(),
          isRead: false,
          isStarred: false,
        },
        {
          id: '3',
          from: 'newsletter@promo.com',
          to: 'test@example.com',
          subject: 'Sale! 50% off',
          snippet: 'Limited time offer',
          timestamp: Date.now(),
          isRead: false,
          isStarred: false,
          labels: ['CATEGORY_PROMOTIONS'],
        },
      ];

      (global.fetch as any)
        .mockResolvedValueOnce({
          ok: true,
          json: async () => ({ messages: mockEmails.map(e => ({ id: e.id })) }),
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => ({
            id: '1',
            labelIds: ['UNREAD', 'STARRED'],
            snippet: mockEmails[0].snippet,
            payload: {
              headers: [
                { name: 'From', value: mockEmails[0].from },
                { name: 'To', value: mockEmails[0].to },
                { name: 'Subject', value: mockEmails[0].subject },
              ],
            },
            internalDate: String(mockEmails[0].timestamp),
          }),
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => ({
            id: '2',
            labelIds: ['UNREAD'],
            snippet: mockEmails[1].snippet,
            payload: {
              headers: [
                { name: 'From', value: mockEmails[1].from },
                { name: 'To', value: mockEmails[1].to },
                { name: 'Subject', value: mockEmails[1].subject },
              ],
            },
            internalDate: String(mockEmails[1].timestamp),
          }),
        })
        .mockResolvedValueOnce({
          ok: true,
          json: async () => ({
            id: '3',
            labelIds: ['UNREAD', 'CATEGORY_PROMOTIONS'],
            snippet: mockEmails[2].snippet,
            payload: {
              headers: [
                { name: 'From', value: mockEmails[2].from },
                { name: 'To', value: mockEmails[2].to },
                { name: 'Subject', value: mockEmails[2].subject },
              ],
            },
            internalDate: String(mockEmails[2].timestamp),
          }),
        });

      const summary = await emailService.getEmailSummary([mockAccount]);

      expect(summary.totalUnread).toBe(3);
      expect(summary.importantCount).toBeGreaterThan(0);
      expect(summary.categories).toBeDefined();
      expect(summary.highlights).toBeDefined();
    });
  });

  describe('Priority Detection', () => {
    it('should detect high priority emails', async () => {
      const urgentEmail: Email = {
        id: '1',
        from: 'boss@company.com',
        to: 'test@example.com',
        subject: 'URGENT: Action Required',
        snippet: 'Please respond ASAP',
        timestamp: Date.now(),
        isRead: false,
        isStarred: true,
        inReplyTo: 'thread123',
      };

      const importance = await emailService.detectImportance(urgentEmail);

      expect(importance).toBe('high');
    });

    it('should detect medium priority emails', async () => {
      const normalEmail: Email = {
        id: '2',
        from: 'colleague@company.com',
        to: 'test@example.com',
        subject: 'Meeting notes',
        snippet: 'Here are the notes',
        timestamp: Date.now(),
        isRead: false,
        isStarred: false,
      };

      const importance = await emailService.detectImportance(normalEmail);

      expect(importance).toBe('medium');
    });

    it('should detect low priority emails', async () => {
      const promoEmail: Email = {
        id: '3',
        from: 'newsletter@promo.com',
        to: 'test@example.com',
        subject: 'Weekly newsletter',
        snippet: 'This week in tech',
        timestamp: Date.now(),
        isRead: false,
        isStarred: false,
      };

      const importance = await emailService.detectImportance(promoEmail);

      expect(importance).toBe('low');
    });
  });

  describe('Email Categorization', () => {
    it('should categorize work emails', async () => {
      const workEmail: Email = {
        id: '1',
        from: 'colleague@company.com',
        to: 'test@example.com',
        subject: 'Project deadline update',
        snippet: 'The project deadline has been moved',
        timestamp: Date.now(),
        isRead: false,
        isStarred: false,
      };

      const category = await emailService.categorizeEmail(workEmail);

      expect(category).toBe('work');
    });

    it('should categorize promotional emails', async () => {
      const promoEmail: Email = {
        id: '2',
        from: 'deals@store.com',
        to: 'test@example.com',
        subject: 'Limited time sale!',
        snippet: 'Save 50% on all items',
        timestamp: Date.now(),
        isRead: false,
        isStarred: false,
      };

      const category = await emailService.categorizeEmail(promoEmail);

      expect(category).toBe('promotions');
    });

    it('should categorize social emails', async () => {
      const socialEmail: Email = {
        id: '3',
        from: 'notifications@social.com',
        to: 'test@example.com',
        subject: 'Someone liked your post',
        snippet: 'John liked your recent post',
        timestamp: Date.now(),
        isRead: false,
        isStarred: false,
      };

      const category = await emailService.categorizeEmail(socialEmail);

      expect(category).toBe('social');
    });
  });

  describe('Account Disconnection', () => {
    it('should disconnect account successfully', async () => {
      await expect(
        emailService.disconnectAccount('user123', 'gmail')
      ).resolves.not.toThrow();
    });
  });
});
