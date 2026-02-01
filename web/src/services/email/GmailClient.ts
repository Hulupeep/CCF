/**
 * Gmail API Client - OAuth2-based Gmail integration
 *
 * Contract: I-EMAIL-001 - OAuth2 authentication
 * Issue: #95 Component 3/5
 */

import { Email } from './EmailService';

export interface GmailTokens {
  access_token: string;
  refresh_token: string;
  expires_in: number;
  scope: string;
  token_type: string;
}

export interface GmailMessage {
  id: string;
  threadId: string;
  labelIds?: string[];
  snippet: string;
  payload?: {
    headers: Array<{ name: string; value: string }>;
    body?: {
      data?: string;
    };
  };
  internalDate: string;
}

export class GmailClient {
  private accessToken: string | null = null;
  private readonly baseUrl = 'https://gmail.googleapis.com/gmail/v1/users/me';
  private readonly clientId = process.env.VITE_GMAIL_CLIENT_ID || '';
  private readonly clientSecret = process.env.VITE_GMAIL_CLIENT_SECRET || '';
  private readonly redirectUri = process.env.VITE_GMAIL_REDIRECT_URI || 'http://localhost:5173/auth/gmail/callback';

  /**
   * Get OAuth2 authorization URL
   */
  async getAuthUrl(): Promise<string> {
    const scopes = [
      'https://www.googleapis.com/auth/gmail.readonly',
      'https://www.googleapis.com/auth/userinfo.email',
    ];

    const params = new URLSearchParams({
      client_id: this.clientId,
      redirect_uri: this.redirectUri,
      response_type: 'code',
      scope: scopes.join(' '),
      access_type: 'offline',
      prompt: 'consent',
    });

    return `https://accounts.google.com/o/oauth2/v2/auth?${params.toString()}`;
  }

  /**
   * Exchange authorization code for tokens
   */
  async exchangeCodeForTokens(code: string): Promise<GmailTokens> {
    const response = await fetch('https://oauth2.googleapis.com/token', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: new URLSearchParams({
        code,
        client_id: this.clientId,
        client_secret: this.clientSecret,
        redirect_uri: this.redirectUri,
        grant_type: 'authorization_code',
      }),
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(`Gmail OAuth failed: ${error.error_description || error.error}`);
    }

    return await response.json();
  }

  /**
   * Refresh access token using refresh token
   */
  async refreshAccessToken(refreshToken: string): Promise<GmailTokens> {
    const response = await fetch('https://oauth2.googleapis.com/token', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: new URLSearchParams({
        refresh_token: refreshToken,
        client_id: this.clientId,
        client_secret: this.clientSecret,
        grant_type: 'refresh_token',
      }),
    });

    if (!response.ok) {
      const error = await response.json();
      throw new Error(`Token refresh failed: ${error.error_description || error.error}`);
    }

    const tokens = await response.json();
    return {
      ...tokens,
      refresh_token: refreshToken, // Preserve refresh token
    };
  }

  /**
   * Set access token for API calls
   */
  async setAccessToken(token: string): Promise<void> {
    this.accessToken = token;
  }

  /**
   * Get user's email address
   */
  async getUserEmail(): Promise<string> {
    const response = await this.makeRequest('/profile');
    return response.emailAddress;
  }

  /**
   * Fetch unread messages
   */
  async fetchUnreadEmails(maxResults: number = 20): Promise<Email[]> {
    const listResponse = await this.makeRequest(
      `/messages?labelIds=UNREAD&maxResults=${maxResults}`
    );

    if (!listResponse.messages || listResponse.messages.length === 0) {
      return [];
    }

    const emails: Email[] = [];

    for (const msg of listResponse.messages) {
      const fullMessage = await this.getMessage(msg.id);
      emails.push(fullMessage);
    }

    return emails;
  }

  /**
   * Get full message details
   */
  async getMessage(messageId: string): Promise<Email> {
    const message: GmailMessage = await this.makeRequest(`/messages/${messageId}`);
    return this.parseMessage(message);
  }

  /**
   * Search messages by query
   */
  async searchMessages(query: string): Promise<Email[]> {
    const encodedQuery = encodeURIComponent(query);
    const listResponse = await this.makeRequest(`/messages?q=${encodedQuery}&maxResults=50`);

    if (!listResponse.messages || listResponse.messages.length === 0) {
      return [];
    }

    const emails: Email[] = [];

    for (const msg of listResponse.messages) {
      const fullMessage = await this.getMessage(msg.id);
      emails.push(fullMessage);
    }

    return emails;
  }

  /**
   * Parse Gmail message into Email interface
   */
  private parseMessage(message: GmailMessage): Email {
    const headers = this.extractHeaders(message);
    const isRead = !message.labelIds?.includes('UNREAD');
    const isStarred = message.labelIds?.includes('STARRED') || false;

    return {
      id: message.id,
      from: headers.from || 'Unknown',
      to: headers.to || '',
      subject: headers.subject || '(No Subject)',
      snippet: message.snippet,
      timestamp: parseInt(message.internalDate),
      isRead,
      isStarred,
      inReplyTo: headers.inReplyTo,
      labels: message.labelIds,
    };
  }

  /**
   * Extract headers from Gmail message
   */
  private extractHeaders(message: GmailMessage): {
    from: string;
    to: string;
    subject: string;
    inReplyTo?: string;
  } {
    const headers = message.payload?.headers || [];
    const result = {
      from: '',
      to: '',
      subject: '',
      inReplyTo: undefined as string | undefined,
    };

    for (const header of headers) {
      const name = header.name.toLowerCase();
      if (name === 'from') result.from = header.value;
      else if (name === 'to') result.to = header.value;
      else if (name === 'subject') result.subject = header.value;
      else if (name === 'in-reply-to') result.inReplyTo = header.value;
    }

    return result;
  }

  /**
   * Make authenticated API request
   */
  private async makeRequest(endpoint: string): Promise<any> {
    if (!this.accessToken) {
      throw new Error('Access token not set');
    }

    const url = endpoint.startsWith('http') ? endpoint : `${this.baseUrl}${endpoint}`;

    const response = await fetch(url, {
      headers: {
        'Authorization': `Bearer ${this.accessToken}`,
        'Content-Type': 'application/json',
      },
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ error: 'Unknown error' }));
      throw new Error(`Gmail API error: ${error.error?.message || error.error || response.statusText}`);
    }

    return await response.json();
  }
}
