# Telegram Bot - Key Code Snippets

Quick reference for key implementation patterns in the Telegram bot.

## Table of Contents

1. [Bot Initialization](#bot-initialization)
2. [Command Handlers](#command-handlers)
3. [Message Processing](#message-processing)
4. [Personality Integration](#personality-integration)
5. [Security (DM Pairing)](#security-dm-pairing)
6. [Rate Limiting](#rate-limiting)
7. [Message Formatting](#message-formatting)
8. [Message Chunking](#message-chunking)
9. [Testing Patterns](#testing-patterns)

---

## Bot Initialization

```typescript
// web/src/telegram-bot.ts
import { TelegramBot } from './services/telegram/TelegramBot';
import { telegramConfig } from './config/telegram';

async function main() {
  const bot = new TelegramBot(telegramConfig);

  // Graceful shutdown
  process.once('SIGINT', () => bot.stop());
  process.once('SIGTERM', () => bot.stop());

  await bot.start();
  console.log('✅ Bot started');
}
```

## Command Handlers

### /start Command (DM Pairing)

```typescript
// web/src/services/telegram/TelegramBot.ts
private async handleStart(ctx: Context): Promise<void> {
  const userId = ctx.from?.id.toString();

  if (this.isDMPaired(userId)) {
    await ctx.reply('✅ Already paired!');
    return;
  }

  this.pairUser(userId, ctx.from?.username);
  const personality = this.personalityStore.getCurrentPersonality();
  const greeting = this.generateGreeting(personality);

  await ctx.reply(`${greeting}\n\n✅ You are now paired!`);
}
```

### /personality Command

```typescript
private async handlePersonality(ctx: Context): Promise<void> {
  const currentPersonality = this.personalityStore.getCurrentPersonality();
  const info = this.formatPersonalityInfo(currentPersonality);
  await ctx.reply(info, { parse_mode: 'Markdown' });
}

private formatPersonalityInfo(personality: PersonalityConfig): string {
  return `
🎭 **Current Personality**

**Baselines:**
- Energy: ${(personality.energy_baseline * 100).toFixed(0)}%
- Curiosity: ${(personality.curiosity_drive * 100).toFixed(0)}%

**Expression:**
- Movement: ${(personality.movement_expressiveness * 100).toFixed(0)}%
  `;
}
```

## Message Processing

### Full Message Flow

```typescript
private async handleMessage(ctx: Context): Promise<void> {
  const userId = ctx.from?.id.toString();

  // 1. Security check
  if (!this.isDMPaired(userId)) {
    await ctx.reply('⚠️ Please /start first');
    return;
  }

  // 2. Rate limiting
  if (!this.checkRateLimit(userId)) {
    await ctx.reply('⏳ Slow down!');
    return;
  }

  // 3. Normalize message
  const message = this.adapter.normalizeInbound(ctx);

  // 4. Generate response
  const response = await this.generateResponse(message);

  // 5. Send response (with chunking)
  await this.sendResponse(ctx, response);
}
```

### Inbound Normalization

```typescript
// web/src/services/telegram/TelegramAdapter.ts
normalizeInbound(ctx: Context): InternalMessage {
  const message = ctx.message;
  const chat = ctx.chat;
  const from = ctx.from;

  return {
    id: message.message_id.toString(),
    userId: from.id.toString(),
    username: from.username,
    text: message.text || message.caption || '',
    timestamp: message.date * 1000,
    chatId: chat.id.toString(),
    chatType: chat.type,
    attachments: this.extractAttachments(message),
  };
}
```

## Personality Integration

### Response Style Adaptation

```typescript
// web/src/services/telegram/TelegramAdapter.ts
formatPersonalityResponse(text: string, personality: PersonalityConfig): string {
  const style = this.getResponseStyle(personality);

  let formattedText = text;

  // Add enthusiasm for high energy
  if (style.enthusiasm > 0.7) {
    formattedText = formattedText.replace(/\.$/g, '!');
  }

  // Add emojis if playful
  if (style.emoji && personality.light_expressiveness > 0.6) {
    const emojis = ['🤖', '✨', '🎨', '🎮'];
    const emoji = emojis[Math.floor(Math.random() * emojis.length)];
    formattedText = `${emoji} ${formattedText}`;
  }

  return formattedText;
}
```

### Deriving Response Style

```typescript
getResponseStyle(personality: PersonalityConfig): ResponseStyle {
  return {
    enthusiasm: personality.energy_baseline,
    verbosity: personality.curiosity_drive > 0.7 ? 'detailed' : 'concise',
    emoji: personality.movement_expressiveness > 0.6,
    formality: personality.energy_baseline < 0.3 ? 'formal' : 'casual',
  };
}
```

### Greeting Generation

```typescript
private generateGreeting(personality: PersonalityConfig): string {
  const energy = personality.energy_baseline;

  if (energy > 0.7) {
    return '🎉 Hey there! I\'m SO excited to meet you!';
  } else if (energy < 0.3) {
    return '👋 Hello. Nice to meet you.';
  } else {
    return '👋 Hi! I\'m mBot, your robot companion.';
  }
}
```

## Security (DM Pairing)

### Pairing Check

```typescript
private isDMPaired(userId: string): boolean {
  // If pairing not required, allow all
  if (!this.config.dmPairingRequired) {
    return true;
  }

  // Check allowlist
  if (this.config.allowedUsers?.includes(userId)) {
    return true;
  }

  // Check pairing state
  return this.pairedUsers.has(userId);
}
```

### User Pairing

```typescript
private pairUser(userId: string, username?: string): void {
  this.pairedUsers.set(userId, {
    userId,
    username,
    pairedAt: Date.now(),
    lastMessageAt: Date.now(),
  });
  console.log(`✅ User paired: ${username || userId}`);
}
```

## Rate Limiting

### Per-User Rate Limit

```typescript
private checkRateLimit(userId: string): boolean {
  const now = Date.now();
  const entry = this.rateLimits.get(userId);

  if (!entry || now >= entry.resetAt) {
    // Reset or create new entry
    this.rateLimits.set(userId, {
      userId,
      count: 1,
      resetAt: now + 60000, // 1 minute
    });
    return true;
  }

  // Check if under limit
  if (entry.count < this.config.rateLimitPerMinute) {
    entry.count++;
    return true;
  }

  return false; // Rate limit exceeded
}
```

## Message Formatting

### Markdown to Telegram HTML

```typescript
// web/src/services/telegram/TelegramAdapter.ts
private markdownToTelegramHtml(markdown: string): string {
  return markdown
    .replace(/\*\*(.+?)\*\*/g, '<b>$1</b>')      // Bold
    .replace(/\*(.+?)\*/g, '<i>$1</i>')          // Italic
    .replace(/`(.+?)`/g, '<code>$1</code>')      // Inline code
    .replace(/```([\s\S]+?)```/g, '<pre>$1</pre>') // Code block
    .replace(/\[(.+?)\]\((.+?)\)/g, '<a href="$2">$1</a>'); // Links
}
```

### Format Outbound

```typescript
formatOutbound(text: string, options?: { replyMarkup?: any }): TelegramMessage {
  const htmlText = this.markdownToTelegramHtml(text);

  return {
    text: htmlText,
    parse_mode: 'HTML',
    reply_markup: options?.replyMarkup,
    disable_web_page_preview: true,
  };
}
```

## Message Chunking

### Chunk Long Messages

```typescript
chunkMessage(text: string, maxLength: number = 4000): string[] {
  if (text.length <= maxLength) {
    return [text];
  }

  const chunks: string[] = [];
  let currentChunk = '';

  // Split by paragraphs
  const paragraphs = text.split('\n\n');

  for (const para of paragraphs) {
    if (currentChunk.length + para.length + 2 <= maxLength) {
      currentChunk += (currentChunk ? '\n\n' : '') + para;
    } else {
      if (currentChunk) chunks.push(currentChunk);

      // If paragraph too long, split by sentences
      if (para.length > maxLength) {
        const sentences = para.match(/[^.!?]+[.!?]+/g) || [para];
        currentChunk = '';
        for (const sentence of sentences) {
          if (currentChunk.length + sentence.length <= maxLength) {
            currentChunk += sentence;
          } else {
            if (currentChunk) chunks.push(currentChunk);
            currentChunk = sentence.substring(0, maxLength);
          }
        }
      } else {
        currentChunk = para;
      }
    }
  }

  if (currentChunk) chunks.push(currentChunk);
  return chunks;
}
```

### Send with Chunking

```typescript
private async sendResponse(ctx: Context, text: string): Promise<void> {
  const chunks = this.adapter.chunkMessage(text);

  for (const chunk of chunks) {
    const formatted = this.adapter.formatOutbound(chunk);
    await ctx.reply(formatted.text, {
      parse_mode: formatted.parse_mode,
    });
  }
}
```

## Testing Patterns

### Mock Telegram Context

```typescript
// tests/integration/telegram-bot.test.ts
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
```

### Test Personality Integration

```typescript
it('should adjust response style based on personality', () => {
  // High energy personality
  const highEnergy = createDefaultConfig();
  highEnergy.energy_baseline = 0.9;

  personalityStore.updateConfig(highEnergy);

  const personality = personalityStore.getCurrentPersonality();
  expect(personality.energy_baseline).toBe(0.9);
});
```

### Test Message Chunking

```typescript
it('should chunk long messages', () => {
  const longMessage = 'A'.repeat(5000);
  const chunks = adapter.chunkMessage(longMessage);

  expect(chunks.length).toBeGreaterThan(1);
  chunks.forEach(chunk => {
    expect(chunk.length).toBeLessThanOrEqual(4000);
  });
});
```

### Test Markdown Formatting

```typescript
it('should convert Markdown to HTML', () => {
  const markdown = '**Bold** *italic* `code`';
  const formatted = adapter.formatOutbound(markdown);

  expect(formatted.text).toContain('<b>Bold</b>');
  expect(formatted.text).toContain('<i>italic</i>');
  expect(formatted.text).toContain('<code>code</code>');
});
```

---

## Configuration Examples

### Development (.env)

```bash
TELEGRAM_BOT_TOKEN=123456789:ABCdefGHI...
TELEGRAM_DM_PAIRING_REQUIRED=false
TELEGRAM_RATE_LIMIT_PER_MIN=100
NODE_ENV=development
```

### Production (.env)

```bash
TELEGRAM_BOT_TOKEN=123456789:ABCdefGHI...
TELEGRAM_DM_PAIRING_REQUIRED=true
TELEGRAM_RATE_LIMIT_PER_MIN=20
TELEGRAM_ALLOWED_USERS=123456789,987654321
NODE_ENV=production
```

---

## Usage Examples

### Running the Bot

```bash
# Development
npm run telegram:dev

# Production build
npm run telegram:build

# Run tests
npm run telegram:test
```

### Creating a Bot

```
1. Open Telegram, search @BotFather
2. Send: /newbot
3. Choose name: mBot Assistant
4. Choose username: mbot_yourname_bot
5. Copy token: 123456789:ABCdefGHI...
6. Add to .env: TELEGRAM_BOT_TOKEN=...
7. Run: npm run telegram:dev
8. Test: Search for bot, send /start
```

---

## Key Files Reference

| File | Key Classes/Functions |
|------|----------------------|
| `TelegramBot.ts` | TelegramBot class, command handlers |
| `TelegramAdapter.ts` | TelegramAdapter, normalization, formatting |
| `telegram.ts` (config) | loadTelegramConfig(), validateConfig() |
| `telegram.ts` (types) | InternalMessage, TelegramMessage, ResponseStyle |
| `telegram-bot.ts` | main() entry point |

---

**For full implementation details, see:**
- `docs/guides/telegram-bot-guide.md` - Complete guide
- `docs/TELEGRAM_BOT_README.md` - Quick reference
- Source code in `web/src/services/telegram/`
