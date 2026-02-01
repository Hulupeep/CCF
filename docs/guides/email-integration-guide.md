# Email Integration Guide

## Overview

The mBot email integration provides OAuth2-based access to Gmail and Outlook accounts for personalized email summaries and priority detection.

**Contract:** I-EMAIL-001 - OAuth2 authentication only, no credential storage

**Issue:** #95 Component 3/5

## Features

- ✅ OAuth2 authentication (Gmail and Outlook)
- ✅ Automatic token refresh
- ✅ Email priority detection
- ✅ Category classification (work, personal, promotions, social)
- ✅ Intelligent email summarization
- ✅ VIP sender support
- ✅ Multi-account management

## OAuth2 Setup

### Gmail Setup

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project or select existing
3. Enable Gmail API
4. Create OAuth2 credentials:
   - Application type: Web application
   - Authorized redirect URIs: `http://localhost:5173/auth/gmail/callback`
5. Copy Client ID and Client Secret

Set environment variables:

```bash
VITE_GMAIL_CLIENT_ID=your_client_id
VITE_GMAIL_CLIENT_SECRET=your_client_secret
VITE_GMAIL_REDIRECT_URI=http://localhost:5173/auth/gmail/callback
```

### Outlook Setup

1. Go to [Azure Portal](https://portal.azure.com/)
2. Navigate to Azure Active Directory > App registrations
3. Create new registration:
   - Name: mBot Email Integration
   - Supported account types: Accounts in any organizational directory and personal Microsoft accounts
   - Redirect URI: `http://localhost:5173/auth/outlook/callback`
4. Go to Certificates & secrets > New client secret
5. Go to API permissions > Add permission > Microsoft Graph > Delegated permissions:
   - Mail.Read
   - User.Read

Set environment variables:

```bash
VITE_OUTLOOK_CLIENT_ID=your_client_id
VITE_OUTLOOK_CLIENT_SECRET=your_client_secret
VITE_OUTLOOK_REDIRECT_URI=http://localhost:5173/auth/outlook/callback
```

## Usage

### Connecting an Account

```typescript
import { EmailService } from './services/email/EmailService';

const emailService = new EmailService();

// User clicks "Connect Gmail" button
// 1. Redirect to OAuth URL
const gmailClient = new GmailClient();
const authUrl = await gmailClient.getAuthUrl();
window.location.href = authUrl;

// 2. After redirect back with code
const account = await emailService.connectAccount('gmail', authCode, userId);
console.log('Connected:', account.email);
```

### Fetching Emails

```typescript
// Fetch unread emails
const emails = await emailService.fetchUnreadEmails(account, 20);

for (const email of emails) {
  console.log(`From: ${email.from}`);
  console.log(`Subject: ${email.subject}`);
  console.log(`Importance: ${email.importance}`);
}
```

### Getting Email Summary

```typescript
// Get summary across all accounts
const summary = await emailService.getEmailSummary([gmailAccount, outlookAccount]);

console.log(`Total unread: ${summary.totalUnread}`);
console.log(`Important: ${summary.importantCount}`);
console.log(`Categories:`, summary.categories);

// Top highlights
for (const highlight of summary.highlights) {
  console.log(`${highlight.from}: ${highlight.subject}`);
}
```

### Priority Detection

```typescript
const importance = await emailService.detectImportance(email);
// Returns: 'high' | 'medium' | 'low'

const category = await emailService.categorizeEmail(email);
// Returns: 'work' | 'personal' | 'promotions' | 'social'
```

### VIP Senders

```typescript
import { PriorityDetector } from './services/email/PriorityDetector';

const detector = new PriorityDetector();

// Add VIP sender
detector.addVIPSender('boss@company.com');

// Emails from VIP senders get +40 priority score
const importance = detector.detectImportance(emailFromBoss);
// More likely to be 'high'
```

## React Components

### EmailConnect

```tsx
import { EmailConnect } from './components/email/EmailConnect';

function EmailSettings() {
  return (
    <EmailConnect
      userId="user123"
      onConnected={(provider, account) => {
        console.log(`Connected ${provider}:`, account.email);
      }}
      onError={(error) => {
        console.error('Connection failed:', error);
      }}
    />
  );
}
```

### EmailAccounts

```tsx
import { EmailAccounts } from './components/email/EmailAccounts';

function Dashboard() {
  return (
    <EmailAccounts
      userId="user123"
      onDisconnect={(account) => {
        console.log('Disconnected:', account.email);
      }}
    />
  );
}
```

## Priority Detection Algorithm

The priority detector uses a scoring system:

| Factor | Points | Description |
|--------|--------|-------------|
| VIP Sender | +40 | Email from known VIP |
| Urgent Keywords | +30 | Contains "urgent", "important", "asap", etc. |
| Thread Participation | +20 | Reply to existing conversation |
| Starred/Flagged | +10 | User manually marked |
| Provider Importance | +20 | Outlook "high importance" flag |
| Work Category | +5 | Work-related content |

**Scoring:**
- Score ≥ 60: **High** priority
- Score ≥ 30: **Medium** priority
- Score < 30: **Low** priority

## Category Classification

Emails are classified by keyword analysis:

| Category | Keywords |
|----------|----------|
| **Work** | meeting, project, deadline, report, task, proposal, review, approval, budget, client, invoice |
| **Promotions** | sale, discount, offer, deal, coupon, promo, limited time, save, shop, buy now |
| **Social** | liked, commented, shared, tagged, mentioned, followed, friend request, notification |
| **Personal** | Default if no other category matches |

Gmail labels are also used for classification when available.

## Security Best Practices

### Token Encryption

⚠️ **Important:** The current implementation uses base64 encoding as a placeholder. In production:

```typescript
// Production: Use proper encryption
import { KMSClient, EncryptCommand } from '@aws-sdk/client-kms';

async function encryptToken(token: string): Promise<string> {
  const kms = new KMSClient({ region: 'us-east-1' });
  const command = new EncryptCommand({
    KeyId: process.env.KMS_KEY_ID,
    Plaintext: Buffer.from(token),
  });
  const result = await kms.send(command);
  return Buffer.from(result.CiphertextBlob).toString('base64');
}
```

### Token Storage

- ✅ Store tokens encrypted at rest
- ✅ Use environment variables for OAuth credentials
- ✅ Never log access/refresh tokens
- ✅ Implement token rotation
- ✅ Revoke tokens on disconnect

### OAuth2 Best Practices

- Use `state` parameter to prevent CSRF
- Validate redirect URIs strictly
- Request minimal scopes (Mail.Read only)
- Handle token expiration gracefully
- Implement proper error handling

## Testing

Run integration tests:

```bash
npm test tests/integration/email-service.test.ts
```

Test coverage includes:
- OAuth2 flow
- Token refresh
- Email fetching
- Priority detection
- Category classification
- Error handling

## Troubleshooting

### OAuth2 Errors

**Error: `invalid_grant`**
- Solution: Authorization code expired or already used
- Action: Restart OAuth flow

**Error: `redirect_uri_mismatch`**
- Solution: Redirect URI doesn't match registered URI
- Action: Check environment variables

**Error: `insufficient_scope`**
- Solution: Missing required permissions
- Action: Re-authenticate with correct scopes

### Token Issues

**Error: `Token expired`**
- Solution: Automatic refresh failed
- Action: Check refresh token validity

**Error: `Token refresh failed`**
- Solution: Refresh token revoked or expired
- Action: User must re-authenticate

### API Errors

**Gmail API Error: `dailyLimitExceeded`**
- Solution: Exceeded API quota
- Action: Wait until quota resets or upgrade quota

**Outlook API Error: `429 Too Many Requests`**
- Solution: Rate limit hit
- Action: Implement exponential backoff

## API Reference

### EmailService

```typescript
class EmailService {
  // Connect account via OAuth2
  connectAccount(provider: 'gmail' | 'outlook', authCode: string, userId: string): Promise<EmailAccount>

  // Disconnect account
  disconnectAccount(userId: string, provider: string): Promise<void>

  // Refresh access token
  refreshToken(account: EmailAccount): Promise<string>

  // Fetch unread emails
  fetchUnreadEmails(account: EmailAccount, maxResults?: number): Promise<Email[]>

  // Get email summary
  getEmailSummary(accounts: EmailAccount[]): Promise<EmailSummary>

  // Detect importance
  detectImportance(email: Email): Promise<'high' | 'medium' | 'low'>

  // Categorize email
  categorizeEmail(email: Email): Promise<'work' | 'personal' | 'promotions' | 'social'>
}
```

### GmailClient

```typescript
class GmailClient {
  // Get OAuth2 URL
  getAuthUrl(): Promise<string>

  // Exchange code for tokens
  exchangeCodeForTokens(code: string): Promise<GmailTokens>

  // Refresh token
  refreshAccessToken(refreshToken: string): Promise<GmailTokens>

  // Fetch emails
  fetchUnreadEmails(maxResults?: number): Promise<Email[]>

  // Get single message
  getMessage(messageId: string): Promise<Email>

  // Search messages
  searchMessages(query: string): Promise<Email[]>
}
```

### OutlookClient

```typescript
class OutlookClient {
  // Get OAuth2 URL
  getAuthUrl(): Promise<string>

  // Exchange code for tokens
  exchangeCodeForTokens(code: string): Promise<OutlookTokens>

  // Refresh token
  refreshAccessToken(refreshToken: string): Promise<OutlookTokens>

  // Fetch emails
  fetchUnreadEmails(maxResults?: number): Promise<Email[]>

  // Get single message
  getMessage(messageId: string): Promise<Email>

  // Search messages
  searchMessages(query: string): Promise<Email[]>
}
```

### PriorityDetector

```typescript
class PriorityDetector {
  // Manage VIP senders
  addVIPSender(email: string): void
  removeVIPSender(email: string): void
  isVIPSender(from: string): boolean

  // Detection
  detectImportance(email: Email): 'high' | 'medium' | 'low'
  categorizeEmail(email: Email): 'work' | 'personal' | 'promotions' | 'social'

  // Learning (future)
  learnFromInteraction(email: Email, interaction: string): Promise<void>
}
```

## Related Documentation

- [Voice Assistant Guide](./voice-assistant-guide.md) - Full voice assistant system
- [Issue #95](https://github.com/Hulupeep/mbot_ruvector/issues/95) - Story details
- [Contract I-EMAIL-001](../contracts/feature_voice.yml) - Email integration contract

## Support

For issues or questions:
- GitHub Issues: https://github.com/Hulupeep/mbot_ruvector/issues
- Contract: I-EMAIL-001 in `docs/contracts/`
