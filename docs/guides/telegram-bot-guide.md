# mBot Telegram Bot Guide

Complete guide for setting up and using the mBot Telegram bot integration.

## Table of Contents

1. [Overview](#overview)
2. [Quick Start (5 Minutes)](#quick-start-5-minutes)
3. [Creating a Telegram Bot](#creating-a-telegram-bot)
4. [Configuration](#configuration)
5. [Available Commands](#available-commands)
6. [Personality Integration](#personality-integration)
7. [Security (DM Pairing)](#security-dm-pairing)
8. [Troubleshooting](#troubleshooting)
9. [Advanced Usage](#advanced-usage)

---

## Overview

The mBot Telegram bot allows users to interact with mBot directly through Telegram messenger. The bot integrates with mBot's personality system to provide dynamic, personality-driven responses.

### Features

- **Personality-Driven Responses** - Bot behavior adapts to mBot's current personality
- **DM Pairing Security** - Optional pairing flow to restrict access
- **Rate Limiting** - Prevents abuse with configurable limits
- **Message Chunking** - Handles long responses automatically
- **Rich Commands** - Full command set for interaction and configuration

### Architecture

Based on OpenClaw gateway patterns:
- **grammY Framework** - Modern Telegram Bot API library
- **Message Normalization** - Standardized internal format
- **Adapter Pattern** - Clean separation between Telegram and mBot
- **Personality Integration** - Uses PersonalityStore singleton

---

## Quick Start (5 Minutes)

### Prerequisites

- Node.js 18+
- npm or yarn
- Telegram account

### Step 1: Create Bot with BotFather

1. Open Telegram and search for `@BotFather`
2. Send `/newbot` command
3. Follow prompts to choose name and username
4. Copy the bot token (looks like `123456789:ABCdefGHIjklMNOpqrsTUVwxyz`)

### Step 2: Configure Environment

```bash
cd /home/xanacan/projects/code/mbot/mbot_ruvector/web

# Copy example env file
cp .env.telegram.example .env

# Edit .env and add your bot token
nano .env
```

Set these variables:
```bash
TELEGRAM_BOT_TOKEN=your_bot_token_here
TELEGRAM_DM_PAIRING_REQUIRED=true
TELEGRAM_RATE_LIMIT_PER_MIN=20
```

### Step 3: Install Dependencies

```bash
npm install
```

### Step 4: Run Bot

```bash
npm run telegram:dev
```

You should see:
```
✅ Bot connected: @YourBotName (Your Bot)
🚀 Bot is now running...
```

### Step 5: Test in Telegram

1. Search for your bot username in Telegram
2. Send `/start` to pair with the bot
3. Send any message to test interaction

---

## Creating a Telegram Bot

### Using BotFather

BotFather is Telegram's official bot for creating and managing bots.

#### Creating a New Bot

1. **Start conversation with BotFather**
   - Search for `@BotFather` in Telegram
   - Click "Start" or send `/start`

2. **Create bot**
   ```
   /newbot
   ```

3. **Choose bot name**
   ```
   Enter name: mBot Assistant
   ```

4. **Choose username**
   ```
   Enter username: mbot_yourname_bot
   ```
   - Must end with "bot"
   - Must be unique

5. **Copy token**
   ```
   Done! Your token is: 123456789:ABCdefGHIjklMNOpqrsTUVwxyz
   ```

#### Customizing Bot

After creation, you can customize:

```
/setdescription - Set bot description
/setabouttext - Set about text
/setuserpic - Set bot profile picture
/setcommands - Set command list
```

##### Setting Commands

Send to BotFather:
```
/setcommands
```

Then send this command list:
```
start - Start conversation and pair with bot
help - Show help message
personality - View or change personality
status - Check robot connection status
reset - Reset conversation context
```

---

## Configuration

### Environment Variables

All configuration is done via environment variables in `.env`:

#### Required

| Variable | Description | Example |
|----------|-------------|---------|
| `TELEGRAM_BOT_TOKEN` | Bot token from BotFather | `123456:ABC...` |

#### Optional

| Variable | Default | Description |
|----------|---------|-------------|
| `TELEGRAM_DM_PAIRING_REQUIRED` | `true` | Require /start before messages |
| `TELEGRAM_RATE_LIMIT_PER_MIN` | `20` | Messages per minute per user |
| `TELEGRAM_ALLOWED_USERS` | (empty) | Comma-separated user IDs |
| `BOT_COMMAND_PREFIX` | `/` | Command prefix character |
| `BOT_PERSONALITY_DEFAULT` | `curious` | Default personality |
| `NODE_ENV` | `development` | Environment mode |

### Configuration Examples

#### Development (Open Access)

```bash
TELEGRAM_BOT_TOKEN=your_token
TELEGRAM_DM_PAIRING_REQUIRED=false
TELEGRAM_RATE_LIMIT_PER_MIN=100
NODE_ENV=development
```

#### Production (Restricted)

```bash
TELEGRAM_BOT_TOKEN=your_token
TELEGRAM_DM_PAIRING_REQUIRED=true
TELEGRAM_RATE_LIMIT_PER_MIN=20
TELEGRAM_ALLOWED_USERS=123456789,987654321
NODE_ENV=production
```

#### Family Use (Moderate Security)

```bash
TELEGRAM_BOT_TOKEN=your_token
TELEGRAM_DM_PAIRING_REQUIRED=true
TELEGRAM_RATE_LIMIT_PER_MIN=50
# No allowed users - anyone can pair
NODE_ENV=production
```

---

## Available Commands

### `/start`

**Purpose:** Initiate DM pairing and start conversation

**Usage:**
```
/start
```

**Response:**
- Greeting based on current personality
- Confirmation of pairing
- List of available commands

**Example:**
```
User: /start

Bot: 👋 Hi! I'm mBot, your robot companion.

✅ You are now paired with mBot!

Available commands:
/help - Show all commands
/personality - View or change personality
/status - Check robot connection status
/reset - Reset conversation context

Send me any message to chat!
```

---

### `/help`

**Purpose:** Display help information and command list

**Usage:**
```
/help
```

**Response:**
- Complete command reference
- Personality system overview
- About information

---

### `/personality [name]`

**Purpose:** View current personality or change personality (future)

**Usage:**
```
/personality              # View current
/personality energetic    # Change (coming soon)
```

**Response (view):**
```
🎭 Current Personality

Baselines:
- Tension: 50%
- Coherence: 50%
- Energy: 50%

Reactivity:
- Startle Sensitivity: 50%
- Recovery Speed: 50%
- Curiosity Drive: 50%

Expression:
- Movement: 50%
- Sound: 50%
- Light: 50%

Use the web dashboard to adjust these parameters.
```

---

### `/status`

**Purpose:** Check bot and robot connection status

**Usage:**
```
/status
```

**Response:**
```
🤖 mBot Status

Bot:
- Username: @mbot_demo_bot
- Status: ✅ Running
- Mode: polling

Last Activity:
- Started: 2026-02-01T10:30:00Z
- Last Probe: 2026-02-01T10:35:00Z

Your Session:
- User ID: 123456789
- Paired: ✅ Yes

Robot Connection:
🚧 Coming soon - will show mBot2 hardware status
```

---

### `/reset`

**Purpose:** Reset conversation context

**Usage:**
```
/reset
```

**Response:**
```
🔄 Conversation context reset!

Starting fresh. What would you like to talk about?
```

---

## Personality Integration

The bot integrates deeply with mBot's personality system to provide dynamic responses.

### How It Works

1. **Personality Store** - Bot uses singleton PersonalityStore
2. **Response Styling** - Adapts tone, verbosity, emoji usage
3. **Real-Time Updates** - Changes to personality affect responses immediately

### Personality Parameters

#### Baselines (How it settles)

| Parameter | Effect on Bot |
|-----------|---------------|
| **Tension** | Affects anxiety in responses |
| **Coherence** | Affects message structure |
| **Energy** | Affects enthusiasm and formality |

#### Reactivity (How it responds)

| Parameter | Effect on Bot |
|-----------|---------------|
| **Startle Sensitivity** | Affects reaction to commands |
| **Recovery Speed** | Affects context persistence |
| **Curiosity Drive** | Affects verbosity and detail |

#### Expression (How it shows feelings)

| Parameter | Effect on Bot |
|-----------|---------------|
| **Movement** | Affects emoji usage |
| **Sound** | (Future: voice messages) |
| **Light** | Affects formatting style |

### Response Styles

#### High Energy (0.7-1.0)

```
User: Hello
Bot: 🎉 Hey there! That's so exciting!
```

- Casual language
- Exclamation marks
- Emojis
- Enthusiastic tone

#### Low Energy (0.0-0.3)

```
User: Hello
Bot: Hello. How may I assist you?
```

- Formal language
- Periods instead of exclamations
- No emojis
- Reserved tone

#### High Curiosity (0.7-1.0)

```
User: I like robots
Bot: Robots - fascinating! I'm curious to learn more about this topic. What specifically interests you about robotics? Is it the mechanical aspects, the AI, or something else entirely?
```

- Detailed responses
- Follow-up questions
- Explores topics deeply

#### Low Curiosity (0.0-0.3)

```
User: I like robots
Bot: Got it. How can I help?
```

- Concise responses
- Direct answers
- Minimal elaboration

---

## Security (DM Pairing)

### What is DM Pairing?

DM Pairing is a security feature that requires users to explicitly pair with the bot before sending messages.

### Why Use It?

- **Prevent Abuse** - Unknown users can't spam your bot
- **Track Users** - Know who's using your bot
- **Privacy** - Control who has access

### How It Works

1. **User sends message** → Bot checks pairing status
2. **If not paired** → Bot prompts with `/start` command
3. **User sends `/start`** → Bot pairs the user
4. **Paired** → User can now chat normally

### Configuration

Enable/disable in `.env`:

```bash
# Enable pairing (recommended)
TELEGRAM_DM_PAIRING_REQUIRED=true

# Disable pairing (open access)
TELEGRAM_DM_PAIRING_REQUIRED=false
```

### Allowlists

For extra security, restrict to specific users:

```bash
TELEGRAM_ALLOWED_USERS=123456789,987654321,555555555
```

**How to get user ID:**
1. User sends message to bot
2. Check bot logs for user ID
3. Add to allowlist

**Or use this bot:**
- Search for `@userinfobot` in Telegram
- Send `/start` to see your user ID

---

## Troubleshooting

### Bot Won't Start

**Error:** `TELEGRAM_BOT_TOKEN is required`

**Solution:**
```bash
# Check .env file exists
ls -la .env

# Verify token is set
cat .env | grep TELEGRAM_BOT_TOKEN

# Token should look like: 123456789:ABCdefGHI...
```

---

**Error:** `Failed to start bot: 401 Unauthorized`

**Solution:** Token is invalid
1. Go to @BotFather
2. Send `/token`
3. Select your bot
4. Copy new token
5. Update `.env`

---

**Error:** `Failed to start bot: Network error`

**Solution:** Check internet connection
```bash
# Test connectivity
ping api.telegram.org

# Check firewall
sudo ufw status
```

---

### Bot Not Responding

**Issue:** Bot online but doesn't respond to messages

**Checklist:**
1. Is DM pairing required? Send `/start` first
2. Check bot logs for errors
3. Verify rate limit not exceeded
4. Restart bot: `npm run telegram:dev`

---

**Issue:** Rate limit errors

**Solution:** Adjust rate limit in `.env`:
```bash
TELEGRAM_RATE_LIMIT_PER_MIN=50  # Increase from 20
```

---

### Message Formatting Issues

**Issue:** HTML tags showing in messages

**Cause:** Markdown not properly converted

**Solution:** Bot auto-converts Markdown → HTML
- Use standard Markdown syntax
- Avoid mixing HTML and Markdown

---

**Issue:** Long messages not sending

**Cause:** Telegram 4096 character limit

**Solution:** Bot auto-chunks messages
- If seeing issues, check for very long single paragraphs
- Break content into paragraphs

---

### Permission Errors

**Issue:** `Error: EACCES: permission denied`

**Solution:**
```bash
# Fix file permissions
chmod +x src/telegram-bot.ts

# Fix directory permissions
chmod -R 755 node_modules
```

---

## Advanced Usage

### Running as System Service

#### Using systemd (Linux)

Create service file: `/etc/systemd/system/mbot-telegram.service`

```ini
[Unit]
Description=mBot Telegram Bot
After=network.target

[Service]
Type=simple
User=mbot
WorkingDirectory=/home/xanacan/projects/code/mbot/mbot_ruvector/web
Environment=NODE_ENV=production
ExecStart=/usr/bin/npm run telegram:dev
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl daemon-reload
sudo systemctl enable mbot-telegram
sudo systemctl start mbot-telegram

# Check status
sudo systemctl status mbot-telegram

# View logs
sudo journalctl -u mbot-telegram -f
```

---

### Production Deployment

#### Using PM2

```bash
# Install PM2
npm install -g pm2

# Start bot
pm2 start src/telegram-bot.ts --name mbot-telegram

# Save configuration
pm2 save

# Setup startup
pm2 startup
```

#### Using Docker

Create `Dockerfile`:

```dockerfile
FROM node:18-alpine

WORKDIR /app

COPY package*.json ./
RUN npm ci --only=production

COPY . .

RUN npm run build

CMD ["npm", "run", "telegram:build"]
```

Build and run:
```bash
docker build -t mbot-telegram .
docker run -d --name mbot-telegram --env-file .env mbot-telegram
```

---

### Webhook Mode (Production)

For production, use webhooks instead of polling:

1. **Set up HTTPS endpoint**
   ```
   https://your-domain.com/webhook
   ```

2. **Configure webhook in .env**
   ```bash
   TELEGRAM_WEBHOOK_URL=https://your-domain.com/webhook
   TELEGRAM_WEBHOOK_SECRET=your_random_secret
   ```

3. **Modify bot code** to use webhook mode (future feature)

---

### Monitoring

#### Built-in Status Updates

Bot logs status every 5 minutes:
```
📊 Status Update:
   - Running: true
   - Bot: @mbot_demo_bot
   - Paired Users: 5
   - Last Probe: 2026-02-01T10:35:00Z
```

#### External Monitoring

Use health check endpoint (future feature):
```bash
curl http://localhost:3000/health
```

---

### Integration with Other Services

#### Connect to Claude Flow

```bash
# Run hooks before bot starts
npx @claude-flow/cli@latest hooks pre-task --task-id "telegram-bot"

# Store bot metrics in memory
npx @claude-flow/cli@latest memory store --key "telegram/status" --value "running"
```

#### Connect to mBot Hardware

Future feature: Bot will connect to mBot2 via serial port to show real hardware status in `/status` command.

---

## Best Practices

1. **Security**
   - Always enable DM pairing in production
   - Use strong, random tokens
   - Regularly rotate bot token
   - Monitor for abuse

2. **Rate Limiting**
   - Start conservative (20 msgs/min)
   - Monitor usage patterns
   - Adjust based on actual needs

3. **Error Handling**
   - Monitor bot logs
   - Set up alerts for failures
   - Have restart mechanism

4. **User Experience**
   - Keep responses concise
   - Use personality wisely
   - Provide helpful error messages

5. **Performance**
   - Use webhook mode in production
   - Monitor response times
   - Cache frequently used data

---

## Next Steps

- [ ] Set up bot with BotFather
- [ ] Configure environment variables
- [ ] Test in development
- [ ] Deploy to production
- [ ] Monitor and iterate

## Support

For issues or questions:
- Check GitHub issues
- Review bot logs
- Test with @BotFather
- Verify Telegram API status

---

**Built with ❤️ using grammY and mBot RuVector**
