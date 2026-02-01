/**
 * Email Service - Orchestrates email integration for Gmail and Outlook
 *
 * Contract: I-EMAIL-001 - OAuth2 authentication only, no credential storage
 * Issue: #95 Component 3/5
 */

import { EmailAccount, EmailSummary, EmailHighlight } from '../../types/voice';
import { GmailClient } from './GmailClient';
import { OutlookClient } from './OutlookClient';
import { PriorityDetector } from './PriorityDetector';

export interface Email {
  id: string;
  from: string;
  to: string;
  subject: string;
  snippet: string;
  body?: string;
  timestamp: number;
  isRead: boolean;
  isStarred: boolean;
  inReplyTo?: string;
  labels?: string[];
  category?: 'work' | 'personal' | 'promotions' | 'social';
  importance?: 'high' | 'medium' | 'low';
}

export interface EmailClientInterface {
  fetchUnreadEmails(maxResults?: number): Promise<Email[]>;
  getMessage(messageId: string): Promise<Email>;
  searchMessages(query: string): Promise<Email[]>;
}

export class EmailService {
  private gmailClients: Map<string, GmailClient> = new Map();
  private outlookClients: Map<string, OutlookClient> = new Map();
  private priorityDetector: PriorityDetector;

  constructor() {
    this.priorityDetector = new PriorityDetector();
  }

  /**
   * Connect a new email account via OAuth2
   * Contract: I-EMAIL-001 - OAuth2 only
   */
  async connectAccount(
    provider: 'gmail' | 'outlook',
    authCode: string,
    userId: string
  ): Promise<EmailAccount> {
    try {
      if (provider === 'gmail') {
        const client = new GmailClient();
        const tokens = await client.exchangeCodeForTokens(authCode);
        const email = await client.getUserEmail();

        const account: EmailAccount = {
          userId,
          provider: 'gmail',
          email,
          accessToken: this.encryptToken(tokens.access_token),
          refreshToken: this.encryptToken(tokens.refresh_token),
          expiresAt: Date.now() + (tokens.expires_in * 1000),
          lastSynced: Date.now(),
        };

        // Store client for future use
        this.gmailClients.set(userId, client);

        return account;
      } else if (provider === 'outlook') {
        const client = new OutlookClient();
        const tokens = await client.exchangeCodeForTokens(authCode);
        const email = await client.getUserEmail();

        const account: EmailAccount = {
          userId,
          provider: 'outlook',
          email,
          accessToken: this.encryptToken(tokens.access_token),
          refreshToken: this.encryptToken(tokens.refresh_token),
          expiresAt: Date.now() + (tokens.expires_in * 1000),
          lastSynced: Date.now(),
        };

        this.outlookClients.set(userId, client);

        return account;
      } else {
        throw new Error(`Unsupported provider: ${provider}`);
      }
    } catch (error) {
      console.error(`Failed to connect ${provider} account:`, error);
      throw new Error(`Email connection failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  /**
   * Disconnect an email account
   */
  async disconnectAccount(userId: string, provider: string): Promise<void> {
    if (provider === 'gmail') {
      this.gmailClients.delete(userId);
    } else if (provider === 'outlook') {
      this.outlookClients.delete(userId);
    }

    // In production, also revoke OAuth2 tokens with provider
    console.log(`Disconnected ${provider} account for user ${userId}`);
  }

  /**
   * Refresh OAuth2 access token
   */
  async refreshToken(account: EmailAccount): Promise<string> {
    try {
      const decryptedRefreshToken = this.decryptToken(account.refreshToken);

      if (account.provider === 'gmail') {
        const client = this.getOrCreateGmailClient(account.userId);
        const tokens = await client.refreshAccessToken(decryptedRefreshToken);
        return this.encryptToken(tokens.access_token);
      } else if (account.provider === 'outlook') {
        const client = this.getOrCreateOutlookClient(account.userId);
        const tokens = await client.refreshAccessToken(decryptedRefreshToken);
        return this.encryptToken(tokens.access_token);
      } else {
        throw new Error(`Unsupported provider: ${account.provider}`);
      }
    } catch (error) {
      console.error('Token refresh failed:', error);
      throw new Error(`Token refresh failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  /**
   * Fetch unread emails from account
   */
  async fetchUnreadEmails(account: EmailAccount, maxResults: number = 20): Promise<Email[]> {
    try {
      // Check if token needs refresh
      if (account.expiresAt < Date.now()) {
        const newToken = await this.refreshToken(account);
        account.accessToken = newToken;
        account.expiresAt = Date.now() + 3600 * 1000; // 1 hour
      }

      const decryptedToken = this.decryptToken(account.accessToken);

      if (account.provider === 'gmail') {
        const client = this.getOrCreateGmailClient(account.userId);
        await client.setAccessToken(decryptedToken);
        return await client.fetchUnreadEmails(maxResults);
      } else if (account.provider === 'outlook') {
        const client = this.getOrCreateOutlookClient(account.userId);
        await client.setAccessToken(decryptedToken);
        return await client.fetchUnreadEmails(maxResults);
      } else {
        throw new Error(`Unsupported provider: ${account.provider}`);
      }
    } catch (error) {
      console.error('Failed to fetch emails:', error);
      throw new Error(`Email fetch failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  /**
   * Get summarized email overview for user
   */
  async getEmailSummary(accounts: EmailAccount[]): Promise<EmailSummary> {
    try {
      const allEmails: Email[] = [];

      // Fetch from all connected accounts
      for (const account of accounts) {
        const emails = await this.fetchUnreadEmails(account, 50);
        allEmails.push(...emails);
      }

      // Categorize and prioritize
      const categorized = this.categorizeEmails(allEmails);
      const highlights = await this.getHighlights(allEmails);

      const summary: EmailSummary = {
        totalUnread: allEmails.length,
        importantCount: allEmails.filter(e => e.importance === 'high').length,
        topSenders: this.getTopSenders(allEmails, 5),
        categories: categorized,
        highlights,
      };

      return summary;
    } catch (error) {
      console.error('Failed to generate email summary:', error);
      throw new Error(`Email summary failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  /**
   * Summarize emails with AI-powered insights
   */
  async summarizeEmails(emails: Email[]): Promise<EmailHighlight[]> {
    return this.getHighlights(emails);
  }

  /**
   * Detect email importance
   */
  async detectImportance(email: Email): Promise<'high' | 'medium' | 'low'> {
    return this.priorityDetector.detectImportance(email);
  }

  /**
   * Categorize email
   */
  async categorizeEmail(email: Email): Promise<'work' | 'personal' | 'promotions' | 'social'> {
    return this.priorityDetector.categorizeEmail(email);
  }

  // Private helper methods

  private getOrCreateGmailClient(userId: string): GmailClient {
    if (!this.gmailClients.has(userId)) {
      this.gmailClients.set(userId, new GmailClient());
    }
    return this.gmailClients.get(userId)!;
  }

  private getOrCreateOutlookClient(userId: string): OutlookClient {
    if (!this.outlookClients.has(userId)) {
      this.outlookClients.set(userId, new OutlookClient());
    }
    return this.outlookClients.get(userId)!;
  }

  private categorizeEmails(emails: Email[]): EmailSummary['categories'] {
    const categories = {
      work: 0,
      personal: 0,
      promotions: 0,
      social: 0,
    };

    for (const email of emails) {
      const category = email.category || this.priorityDetector.categorizeEmail(email);
      categories[category]++;
    }

    return categories;
  }

  private async getHighlights(emails: Email[]): Promise<EmailHighlight[]> {
    const highlights: EmailHighlight[] = [];

    // Prioritize and select top emails
    const prioritized = emails
      .map(email => ({
        email,
        importance: this.priorityDetector.detectImportance(email),
      }))
      .sort((a, b) => {
        const importanceOrder = { high: 3, medium: 2, low: 1 };
        return importanceOrder[b.importance] - importanceOrder[a.importance];
      });

    // Take top 5 high/medium priority emails
    for (const { email, importance } of prioritized.slice(0, 5)) {
      highlights.push({
        from: email.from,
        subject: email.subject,
        snippet: email.snippet,
        importance,
        timestamp: email.timestamp,
      });
    }

    return highlights;
  }

  private getTopSenders(emails: Email[], limit: number): string[] {
    const senderCounts = new Map<string, number>();

    for (const email of emails) {
      senderCounts.set(email.from, (senderCounts.get(email.from) || 0) + 1);
    }

    return Array.from(senderCounts.entries())
      .sort((a, b) => b[1] - a[1])
      .slice(0, limit)
      .map(([sender]) => sender);
  }

  /**
   * Simple encryption placeholder
   * In production: Use proper encryption (AES-256-GCM with secure key management)
   */
  private encryptToken(token: string): string {
    // Production: Use @aws-sdk/client-kms or similar
    return Buffer.from(token).toString('base64');
  }

  private decryptToken(encrypted: string): string {
    // Production: Use @aws-sdk/client-kms or similar
    return Buffer.from(encrypted, 'base64').toString('utf-8');
  }
}
