# Telegram Integration Patterns from OpenClaw

## Overview

OpenClaw implements Telegram integration using the **grammY** library with a plugin-based architecture that separates concerns into inbound normalization, outbound formatting, and bot lifecycle management.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      Telegram Bot API                            │
│                   (grammY Framework)                             │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│                   Telegram Channel Adapter                       │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │   Inbound    │  │   Outbound   │  │   Lifecycle  │         │
│  │  Normalizer  │  │   Adapter    │  │   Manager    │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Gateway (OpenClaw)                          │
│     Channels Registry → Session Manager → Agent Runtime         │
└─────────────────────────────────────────────────────────────────┘
```

## Key Components

### 1. Bot Setup (using grammY)

**Library**: `grammy` (modern Telegram Bot API framework)

**Installation**:
```bash
npm install grammy
```

**Basic Bot Structure**:
```typescript
import { Bot } from "grammy"

const bot = new Bot(process.env.TELEGRAM_BOT_TOKEN)

// Message handler
bot.on("message", async (ctx) => {
  const message = ctx.message
  const chatId = ctx.chat.id
  const userId = ctx.from.id

  // Process message
  await handleInboundMessage(message, chatId, userId)
})

// Start bot
await bot.start()
```

### 2. Inbound Message Normalization

**Purpose**: Convert Telegram-specific message format to standardized internal format

**Location**: `src/channels/plugins/normalize/telegram.ts`

**Key Transformations**:
```typescript
interface NormalizedMessage {
  channel: "telegram"
  chatId: string          // Telegram chat ID
  userId: string          // Sender user ID
  messageId: string       // Message ID for replies
  text: string            // Message text
  timestamp: number       // Unix timestamp
  threadId?: string       // Topic thread ID (for forums)
  replyToId?: string      // ID of message being replied to
  mediaUrls?: string[]    // Attached media
  metadata: {
    username?: string
    firstName?: string
    lastName?: string
    isBot: boolean
    chatType: "private" | "group" | "supergroup" | "channel"
  }
}
```

**Normalization Logic**:
```typescript
export function normalizeTelegramMessage(ctx: Context): NormalizedMessage {
  const message = ctx.message
  const chat = ctx.chat
  const from = ctx.from

  return {
    channel: "telegram",
    chatId: chat.id.toString(),
    userId: from.id.toString(),
    messageId: message.message_id.toString(),
    text: message.text || message.caption || "",
    timestamp: message.date * 1000, // Convert to milliseconds
    threadId: message.message_thread_id?.toString(),
    replyToId: message.reply_to_message?.message_id.toString(),
    mediaUrls: extractMediaUrls(message),
    metadata: {
      username: from.username,
      firstName: from.first_name,
      lastName: from.last_name,
      isBot: from.is_bot,
      chatType: chat.type
    }
  }
}

function extractMediaUrls(message: any): string[] | undefined {
  const urls: string[] = []

  if (message.photo) {
    // Get largest photo
    const largest = message.photo[message.photo.length - 1]
    urls.push(largest.file_id)
  }

  if (message.document) {
    urls.push(message.document.file_id)
  }

  if (message.video) {
    urls.push(message.video.file_id)
  }

  if (message.audio) {
    urls.push(message.audio.file_id)
  }

  if (message.voice) {
    urls.push(message.voice.file_id)
  }

  return urls.length > 0 ? urls : undefined
}
```

### 3. Outbound Message Sending

**Purpose**: Send messages from mBot back to Telegram chats

**Location**: `src/channels/plugins/outbound/telegram.ts`

**Key Features**:
- Markdown → HTML formatting
- Message chunking (4000 char limit)
- Reply threading
- Media attachments
- Button support (inline keyboards)

**Outbound Adapter Interface**:
```typescript
interface ChannelOutboundAdapter {
  deliveryMode: "direct"
  chunker: (text: string) => string[]
  chunkerMode: "markdown"
  textChunkLimit: 4000

  sendText(params: SendTextParams): Promise<SendResult>
  sendMedia(params: SendMediaParams): Promise<SendResult>
  sendPayload(params: SendPayloadParams): Promise<SendResult>
}

interface SendTextParams {
  to: string              // Chat ID
  text: string            // Message content
  accountId?: string      // Bot account ID (for multi-bot setups)
  replyToId?: string      // Reply to specific message
  threadId?: string       // Forum topic thread
  deps?: {                // Dependency injection for testing
    sendTelegram?: typeof sendMessageTelegram
  }
}

interface SendResult {
  channel: "telegram"
  messageId: string
  chatId: string
  success: boolean
  error?: string
}
```

**Implementation**:
```typescript
export const telegramOutbound: ChannelOutboundAdapter = {
  deliveryMode: "direct",
  chunker: markdownToTelegramHtmlChunks,
  chunkerMode: "markdown",
  textChunkLimit: 4000,

  async sendText({ to, text, accountId, replyToId, threadId, deps }) {
    const send = deps?.sendTelegram ?? sendMessageTelegram
    const replyToMessageId = parseReplyToMessageId(replyToId)
    const messageThreadId = parseThreadId(threadId)

    const result = await send(to, text, {
      verbose: false,
      textMode: "html",
      messageThreadId,
      replyToMessageId,
      accountId: accountId ?? undefined
    })

    return { channel: "telegram", ...result }
  },

  async sendMedia({ to, text, mediaUrl, accountId, replyToId, threadId, deps }) {
    const send = deps?.sendTelegram ?? sendMessageTelegram

    const result = await send(to, text, {
      verbose: false,
      mediaUrl,
      textMode: "html",
      messageThreadId: parseThreadId(threadId),
      replyToMessageId: parseReplyToMessageId(replyToId),
      accountId: accountId ?? undefined
    })

    return { channel: "telegram", ...result }
  },

  async sendPayload({ to, payload, accountId, replyToId, threadId, deps }) {
    const send = deps?.sendTelegram ?? sendMessageTelegram
    const telegramData = payload.channelData?.telegram

    // Support for inline keyboards (buttons)
    const buttons = telegramData?.buttons // Array<Array<{ text: string; callback_data: string }>>
    const quoteText = telegramData?.quoteText // Reply with quote

    const result = await send(to, payload.text ?? "", {
      verbose: false,
      textMode: "html",
      messageThreadId: parseThreadId(threadId),
      replyToMessageId: parseReplyToMessageId(replyToId),
      quoteText,
      buttons,
      mediaUrl: payload.mediaUrl,
      accountId: accountId ?? undefined
    })

    return { channel: "telegram", ...result }
  }
}
```

### 4. Message Chunking

**Purpose**: Handle Telegram's 4096 character limit

**Chunking Strategy**:
1. Convert Markdown to Telegram HTML
2. Split at natural boundaries (paragraphs, sentences)
3. Preserve HTML tag integrity
4. Send as multiple messages

**Implementation**:
```typescript
export function markdownToTelegramHtmlChunks(text: string): string[] {
  // Convert Markdown to Telegram HTML
  const html = markdownToTelegramHtml(text)

  // Split into chunks ≤ 4000 chars
  const chunks: string[] = []
  let currentChunk = ""

  const paragraphs = html.split("\n\n")

  for (const para of paragraphs) {
    if (currentChunk.length + para.length + 2 <= 4000) {
      currentChunk += (currentChunk ? "\n\n" : "") + para
    } else {
      if (currentChunk) chunks.push(currentChunk)
      currentChunk = para

      // If single paragraph > 4000, split sentences
      if (para.length > 4000) {
        const sentences = para.match(/[^.!?]+[.!?]+/g) || [para]
        currentChunk = ""
        for (const sentence of sentences) {
          if (currentChunk.length + sentence.length <= 4000) {
            currentChunk += sentence
          } else {
            if (currentChunk) chunks.push(currentChunk)
            currentChunk = sentence
          }
        }
      }
    }
  }

  if (currentChunk) chunks.push(currentChunk)

  return chunks
}

function markdownToTelegramHtml(markdown: string): string {
  return markdown
    .replace(/\*\*(.+?)\*\*/g, "<b>$1</b>")      // Bold
    .replace(/\*(.+?)\*/g, "<i>$1</i>")          // Italic
    .replace(/`(.+?)`/g, "<code>$1</code>")      // Inline code
    .replace(/```(.+?)```/gs, "<pre>$1</pre>")   // Code block
    .replace(/\[(.+?)\]\((.+?)\)/g, '<a href="$2">$1</a>') // Links
}
```

### 5. Bot Lifecycle Management

**Bot Status Tracking**:
```typescript
interface TelegramStatus {
  configured: boolean     // Bot token present
  running: boolean        // Bot currently active
  mode: string            // "polling" or "webhook"
  lastStartAt?: string    // ISO timestamp
  lastProbeAt?: string    // ISO timestamp
  lastError?: string      // Last error message
  probe?: {
    ok: boolean
    status?: string
    error?: string
    bot?: {
      id: number
      username: string
      firstName: string
    }
  }
}
```

**Bot Probe** (health check):
```typescript
async function probeTelegramBot(botToken: string): Promise<TelegramStatus> {
  try {
    const bot = new Bot(botToken)
    const me = await bot.api.getMe()

    return {
      configured: true,
      running: true,
      mode: "polling",
      lastProbeAt: new Date().toISOString(),
      probe: {
        ok: true,
        status: "Connected",
        bot: {
          id: me.id,
          username: me.username,
          firstName: me.first_name
        }
      }
    }
  } catch (error) {
    return {
      configured: true,
      running: false,
      mode: "polling",
      lastProbeAt: new Date().toISOString(),
      lastError: error.message,
      probe: {
        ok: false,
        error: error.message
      }
    }
  }
}
```

### 6. Multi-Account Support

**Purpose**: Run multiple bots simultaneously

**Account Structure**:
```typescript
interface TelegramAccount {
  accountId: string       // Unique identifier
  name?: string           // Human-readable name
  botToken: string        // Bot API token
  configured: boolean
  running: boolean
  lastInboundAt?: string  // Last received message
  lastError?: string
}
```

**Account Registry**:
```typescript
class TelegramAccountManager {
  private accounts: Map<string, TelegramAccount> = new Map()
  private bots: Map<string, Bot> = new Map()

  async addAccount(accountId: string, botToken: string, name?: string) {
    const account: TelegramAccount = {
      accountId,
      name,
      botToken,
      configured: true,
      running: false
    }

    this.accounts.set(accountId, account)

    // Start bot
    await this.startBot(accountId)
  }

  async startBot(accountId: string) {
    const account = this.accounts.get(accountId)
    if (!account) throw new Error(`Account ${accountId} not found`)

    const bot = new Bot(account.botToken)

    // Register handlers
    bot.on("message", async (ctx) => {
      account.lastInboundAt = new Date().toISOString()
      await this.handleMessage(accountId, ctx)
    })

    // Start polling
    await bot.start()

    account.running = true
    this.bots.set(accountId, bot)
  }

  async stopBot(accountId: string) {
    const bot = this.bots.get(accountId)
    if (bot) {
      await bot.stop()
      this.bots.delete(accountId)
    }

    const account = this.accounts.get(accountId)
    if (account) {
      account.running = false
    }
  }

  async sendMessage(accountId: string, chatId: string, text: string) {
    const bot = this.bots.get(accountId)
    if (!bot) throw new Error(`Bot ${accountId} not running`)

    return await bot.api.sendMessage(chatId, text, { parse_mode: "HTML" })
  }
}
```

### 7. Security & Authentication

**Bot Token Security**:
```typescript
// Store in environment variable
const TELEGRAM_BOT_TOKEN = process.env.TELEGRAM_BOT_TOKEN

// Or in config file (encrypted at rest)
const config = {
  channels: {
    telegram: {
      accounts: {
        main: {
          botToken: process.env.TELEGRAM_BOT_TOKEN_MAIN
        }
      }
    }
  }
}
```

**User Allowlisting** (DM Policy):
```typescript
interface TelegramDMPolicy {
  policy: "pairing" | "open" | "closed"
  allowFrom: string[]  // User IDs allowed to DM
}

async function handleInboundMessage(ctx: Context) {
  const userId = ctx.from.id.toString()
  const chatType = ctx.chat.type

  if (chatType === "private") {
    // Check DM policy
    const policy = getTelegramDMPolicy()

    if (policy.policy === "closed") {
      return // Ignore all DMs
    }

    if (policy.policy === "pairing" && !policy.allowFrom.includes(userId)) {
      // Send pairing code
      const code = generatePairingCode()
      await ctx.reply(`Pairing code: ${code}\nAdmin must approve with: openclaw pairing approve telegram ${code}`)
      return
    }
  }

  // Process message normally
  await processMessage(ctx)
}
```

### 8. Advanced Features

**Inline Keyboards (Buttons)**:
```typescript
import { InlineKeyboard } from "grammy"

async function sendMessageWithButtons(chatId: string, text: string) {
  const keyboard = new InlineKeyboard()
    .text("Option 1", "callback_data_1")
    .text("Option 2", "callback_data_2")
    .row()
    .text("Cancel", "cancel")

  await bot.api.sendMessage(chatId, text, {
    reply_markup: keyboard,
    parse_mode: "HTML"
  })
}

// Handle button callbacks
bot.on("callback_query:data", async (ctx) => {
  const data = ctx.callbackQuery.data

  if (data === "callback_data_1") {
    await ctx.answerCallbackQuery({ text: "You clicked Option 1" })
    // Handle action
  }
})
```

**Reply Threading**:
```typescript
// Reply to specific message
await bot.api.sendMessage(chatId, "This is a reply", {
  reply_to_message_id: originalMessageId
})

// Forum topics (supergroups with topics enabled)
await bot.api.sendMessage(chatId, "Message in topic", {
  message_thread_id: topicId
})
```

**Media Handling**:
```typescript
// Send photo
await bot.api.sendPhoto(chatId, photoUrl, {
  caption: "Photo caption",
  parse_mode: "HTML"
})

// Send document
await bot.api.sendDocument(chatId, documentUrl, {
  caption: "File description"
})

// Send multiple media (album)
await bot.api.sendMediaGroup(chatId, [
  { type: "photo", media: photo1Url },
  { type: "photo", media: photo2Url }
])
```

**File Downloads**:
```typescript
// Download file from Telegram
async function downloadTelegramFile(fileId: string): Promise<Buffer> {
  const file = await bot.api.getFile(fileId)
  const url = `https://api.telegram.org/file/bot${BOT_TOKEN}/${file.file_path}`

  const response = await fetch(url)
  return Buffer.from(await response.arrayBuffer())
}
```

## Configuration Example

**OpenClaw Config Format**:
```json
{
  "channels": {
    "telegram": {
      "enabled": true,
      "accounts": {
        "main": {
          "botToken": "${TELEGRAM_BOT_TOKEN}",
          "name": "mBot Main"
        }
      },
      "dm": {
        "policy": "pairing",
        "allowFrom": ["123456789", "987654321"]
      },
      "groups": {
        "activation": "mention",
        "allowFrom": ["-100123456789"]
      }
    }
  }
}
```

## Testing

**Unit Tests**:
```typescript
import { describe, it, expect, vi } from "vitest"

describe("Telegram Outbound Adapter", () => {
  it("should send text message", async () => {
    const mockSend = vi.fn().mockResolvedValue({
      messageId: "123",
      chatId: "456"
    })

    const result = await telegramOutbound.sendText({
      to: "456",
      text: "Hello, world!",
      deps: { sendTelegram: mockSend }
    })

    expect(result).toEqual({
      channel: "telegram",
      messageId: "123",
      chatId: "456"
    })

    expect(mockSend).toHaveBeenCalledWith("456", "Hello, world!", {
      verbose: false,
      textMode: "html",
      messageThreadId: undefined,
      replyToMessageId: undefined,
      accountId: undefined
    })
  })
})
```

## Integration Checklist for mBot

- [ ] Install `grammy` library
- [ ] Create Telegram bot via @BotFather
- [ ] Store bot token securely (environment variable)
- [ ] Implement inbound message handler
- [ ] Normalize Telegram messages to internal format
- [ ] Implement outbound adapter (text, media, buttons)
- [ ] Add message chunking for long responses
- [ ] Configure DM policy (pairing recommended)
- [ ] Add bot lifecycle management (start/stop/probe)
- [ ] Test with real Telegram account
- [ ] Document bot setup instructions

## Useful Resources

- **grammY Documentation**: https://grammy.dev/
- **Telegram Bot API**: https://core.telegram.org/bots/api
- **OpenClaw Telegram Source**: `/home/xanacan/projects/code/openclaw/src/channels/plugins/outbound/telegram.ts`

## Common Pitfalls

1. **Message Limits**: Telegram has 4096 char limit (use chunking)
2. **Rate Limits**: 30 messages/second to same chat
3. **Bot Permissions**: Need admin rights in groups for some features
4. **File IDs**: Telegram file IDs expire, download immediately
5. **HTML Escaping**: Escape `<`, `>`, `&` in HTML mode
6. **Webhook vs Polling**: Polling is simpler for development

## Next Steps

See **AUTONOMOUS_BEHAVIOR.md** for integrating proactive messaging and scheduling with Telegram.
