/**
 * Outlook/Microsoft Graph API Client - OAuth2-based Outlook integration
 *
 * Contract: I-EMAIL-001 - OAuth2 authentication
 * Issue: #95 Component 3/5
 */

import { Email } from './EmailService';

export interface OutlookTokens {
  access_token: string;
  refresh_token: string;
  expires_in: number;
  scope: string;
  token_type: string;
}

export interface OutlookMessage {
  id: string;
  subject: string;
  bodyPreview: string;
  from: {
    emailAddress: {
      name: string;
      address: string;
    };
  };
  toRecipients: Array<{
    emailAddress: {
      name: string;
      address: string;
    };
  }>;
  receivedDateTime: string;
  isRead: boolean;
  flag?: {
    flagStatus: string;
  };
  conversationId?: string;
  categories?: string[];
  importance?: 'low' | 'normal' | 'high';
}

export class OutlookClient {
  private accessToken: string | null = null;
  private readonly baseUrl = 'https://graph.microsoft.com/v1.0';
  private readonly clientId = process.env.VITE_OUTLOOK_CLIENT_ID || '';
  private readonly clientSecret = process.env.VITE_OUTLOOK_CLIENT_SECRET || '';
  private readonly redirectUri = process.env.VITE_OUTLOOK_REDIRECT_URI || 'http://localhost:5173/auth/outlook/callback';

  /**
   * Get OAuth2 authorization URL
   */
  async getAuthUrl(): Promise<string> {
    const scopes = [
      'https://graph.microsoft.com/Mail.Read',
      'https://graph.microsoft.com/User.Read',
    ];

    const params = new URLSearchParams({
      client_id: this.clientId,
      response_type: 'code',
      redirect_uri: this.redirectUri,
      response_mode: 'query',
      scope: scopes.join(' '),
    });

    return `https://login.microsoftonline.com/common/oauth2/v2.0/authorize?${params.toString()}`;
  }

  /**
   * Exchange authorization code for tokens
   */
  async exchangeCodeForTokens(code: string): Promise<OutlookTokens> {
    const response = await fetch('https://login.microsoftonline.com/common/oauth2/v2.0/token', {
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
      throw new Error(`Outlook OAuth failed: ${error.error_description || error.error}`);
    }

    return await response.json();
  }

  /**
   * Refresh access token using refresh token
   */
  async refreshAccessToken(refreshToken: string): Promise<OutlookTokens> {
    const response = await fetch('https://login.microsoftonline.com/common/oauth2/v2.0/token', {
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
    const response = await this.makeRequest('/me');
    return response.mail || response.userPrincipalName;
  }

  /**
   * Fetch unread messages
   */
  async fetchUnreadEmails(maxResults: number = 20): Promise<Email[]> {
    const response = await this.makeRequest(
      `/me/mailFolders/inbox/messages?$filter=isRead eq false&$top=${maxResults}&$orderby=receivedDateTime desc`
    );

    if (!response.value || response.value.length === 0) {
      return [];
    }

    return response.value.map((msg: OutlookMessage) => this.parseMessage(msg));
  }

  /**
   * Get message by ID
   */
  async getMessage(messageId: string): Promise<Email> {
    const message: OutlookMessage = await this.makeRequest(`/me/messages/${messageId}`);
    return this.parseMessage(message);
  }

  /**
   * Search messages
   */
  async searchMessages(query: string): Promise<Email[]> {
    const encodedQuery = encodeURIComponent(query);
    const response = await this.makeRequest(
      `/me/messages?$search="${encodedQuery}"&$top=50&$orderby=receivedDateTime desc`
    );

    if (!response.value || response.value.length === 0) {
      return [];
    }

    return response.value.map((msg: OutlookMessage) => this.parseMessage(msg));
  }

  /**
   * Parse Outlook message into Email interface
   */
  private parseMessage(message: OutlookMessage): Email {
    const from = `${message.from.emailAddress.name} <${message.from.emailAddress.address}>`;
    const to = message.toRecipients
      .map(r => `${r.emailAddress.name} <${r.emailAddress.address}>`)
      .join(', ');

    const isStarred = message.flag?.flagStatus === 'flagged';
    const timestamp = new Date(message.receivedDateTime).getTime();

    let importance: 'high' | 'medium' | 'low' = 'medium';
    if (message.importance === 'high') importance = 'high';
    else if (message.importance === 'low') importance = 'low';

    return {
      id: message.id,
      from,
      to,
      subject: message.subject,
      snippet: message.bodyPreview,
      timestamp,
      isRead: message.isRead,
      isStarred,
      inReplyTo: message.conversationId,
      labels: message.categories,
      importance,
    };
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
      const error = await response.json().catch(() => ({ error: { message: 'Unknown error' } }));
      throw new Error(`Outlook API error: ${error.error?.message || response.statusText}`);
    }

    return await response.json();
  }
}
