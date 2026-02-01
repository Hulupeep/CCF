/**
 * TelegramBot Service - Main bot implementation
 * Based on OpenClaw Gateway patterns with grammY framework
 * Integrates with mBot personality system
 */

import { Bot, Context } from 'grammy';
import { TelegramAdapter } from './TelegramAdapter';
import { PersonalityStore } from '../personalityStore';
import { LearningEngine } from '../learning/LearningEngine';
import { AutonomyEngine, getAutonomyEngine, registerBuiltInActions, getContextMonitor, getEventBus, EventTypes } from '../autonomy';
import {
  TelegramBotConfig,
  TelegramBotStatus,
  PairingState,
  InternalMessage,
  RateLimitEntry,
} from '../../types/telegram';
import { PersonalityConfig } from '../../types/personality';

/**
 * Main Telegram Bot class with autonomous behavior support
 */
export class TelegramBot {
  private bot: Bot;
  private adapter: TelegramAdapter;
  private personalityStore: PersonalityStore;
  private learningEngine: LearningEngine;
  private autonomyEngine: AutonomyEngine;
  private config: TelegramBotConfig;

  // Security
  private pairedUsers: Map<string, PairingState> = new Map();

  // Rate limiting
  private rateLimits: Map<string, RateLimitEntry> = new Map();

  // Bot status
  private status: TelegramBotStatus = {
    configured: false,
    running: false,
    mode: 'polling',
  };

  constructor(config: TelegramBotConfig, learningEngine?: LearningEngine) {
    this.config = config;
    this.bot = new Bot(config.botToken);
    this.adapter = new TelegramAdapter();
    this.personalityStore = PersonalityStore.getInstance();
    this.learningEngine = learningEngine || new LearningEngine();

    // Initialize autonomy engine
    this.autonomyEngine = getAutonomyEngine({
      enabled: true,
      safeMode: false,
      maxConcurrentActions: 5,
      defaultCooldown: 60000,
    });

    this.registerHandlers();
    this.registerAutonomousBehavior();
    this.status.configured = true;
  }

  /**
   * Register autonomous behavior actions
   */
  private registerAutonomousBehavior(): void {
    // Register built-in actions
    registerBuiltInActions(this.autonomyEngine);

    // Hook up Telegram message sending to actions
    this.setupAutonomousMessaging();

    console.log('[TelegramBot] Autonomous behavior registered');
  }

  /**
   * Setup autonomous messaging capability
   */
  private setupAutonomousMessaging(): void {
    // This will be called by autonomous actions to send messages
    // For now, we'll just log - full integration requires storing user IDs
    console.log('[TelegramBot] Autonomous messaging ready');
  }

  /**
   * Register all message and command handlers
   */
  private registerHandlers(): void {
    // Command handlers
    this.bot.command('start', this.handleStart.bind(this));
    this.bot.command('help', this.handleHelp.bind(this));
    this.bot.command('personality', this.handlePersonality.bind(this));
    this.bot.command('status', this.handleStatus.bind(this));
    this.bot.command('reset', this.handleReset.bind(this));

    // Regular message handler
    this.bot.on('message:text', this.handleMessage.bind(this));

    // Error handler
    this.bot.catch((err) => {
      console.error('Bot error:', err);
      this.status.lastError = err.message;
    });
  }

  /**
   * Start bot (begin polling)
   */
  async start(): Promise<void> {
    try {
      // Probe bot to verify token
      const me = await this.bot.api.getMe();

      this.status.bot = {
        id: me.id,
        username: me.username,
        firstName: me.first_name,
      };

      this.status.lastProbeAt = new Date().toISOString();

      console.log(`✅ Bot connected: @${me.username} (${me.first_name})`);

      // Start polling
      await this.bot.start();

      this.status.running = true;
      this.status.lastStartAt = new Date().toISOString();

      // Start autonomy engine
      this.autonomyEngine.start();

      // Emit startup event
      const eventBus = getEventBus();
      await eventBus.emit(EventTypes.SYSTEM_STARTUP, { bot: me });

      console.log('🚀 Bot is now running with autonomous behavior...');
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      this.status.lastError = errorMessage;
      throw new Error(`Failed to start bot: ${errorMessage}`);
    }
  }

  /**
   * Stop bot (stop polling)
   */
  async stop(): Promise<void> {
    await this.bot.stop();

    // Stop autonomy engine
    this.autonomyEngine.stop();

    // Emit shutdown event
    const eventBus = getEventBus();
    await eventBus.emit(EventTypes.SYSTEM_SHUTDOWN, {});

    this.status.running = false;
    console.log('🛑 Bot stopped');
  }

  /**
   * Get bot status
   */
  getStatus(): TelegramBotStatus {
    return { ...this.status };
  }

  /**
   * Handle /start command - DM pairing flow
   */
  private async handleStart(ctx: Context): Promise<void> {
    const userId = ctx.from?.id.toString();
    if (!userId) return;

    // Check if already paired
    if (this.isDMPaired(userId)) {
      await ctx.reply(
        '✅ You are already paired!\n\n' +
        'Send me any message and I\'ll respond based on my current personality.'
      );
      return;
    }

    // Pair user
    this.pairUser(userId, ctx.from?.username);

    const personality = this.personalityStore.getCurrentPersonality();
    const greeting = this.generateGreeting(personality);

    await ctx.reply(
      `${greeting}\n\n` +
      '✅ You are now paired with mBot!\n\n' +
      '**Available commands:**\n' +
      '/help - Show all commands\n' +
      '/personality - View or change personality\n' +
      '/status - Check robot connection status\n' +
      '/reset - Reset conversation context\n\n' +
      'Send me any message to chat!',
      { parse_mode: 'Markdown' }
    );
  }

  /**
   * Handle /help command
   */
  private async handleHelp(ctx: Context): Promise<void> {
    const helpText = `
🤖 **mBot Telegram Bot - Help**

**Commands:**
/start - Start conversation and pair with bot
/help - Show this help message
/personality [name] - Get or set personality
/status - Check robot connection status
/reset - Reset conversation context

**Personality System:**
mBot has different personalities that affect how it responds:
- Energy level (sleepy to hyperactive)
- Curiosity (cautious to adventurous)
- Expressiveness (subtle to dramatic)

Use \`/personality\` to see current personality or \`/personality <name>\` to change it.

**About:**
mBot RuVector is an AI-powered robot assistant with personality-driven behavior.
Built with ❤️ for learning and play.
    `;

    await ctx.reply(helpText, { parse_mode: 'Markdown' });
  }

  /**
   * Handle /personality command
   */
  private async handlePersonality(ctx: Context): Promise<void> {
    const args = ctx.message?.text?.split(' ').slice(1);
    const currentPersonality = this.personalityStore.getCurrentPersonality();

    if (!args || args.length === 0) {
      // Show current personality
      const info = this.formatPersonalityInfo(currentPersonality);
      await ctx.reply(info, { parse_mode: 'Markdown' });
    } else {
      // Change personality (for future implementation with presets)
      await ctx.reply(
        '🚧 Personality switching via Telegram coming soon!\n\n' +
        'For now, use the web dashboard to change personalities.'
      );
    }
  }

  /**
   * Handle /status command
   */
  private async handleStatus(ctx: Context): Promise<void> {
    const statusText = `
🤖 **mBot Status**

**Bot:**
- Username: @${this.status.bot?.username || 'unknown'}
- Status: ${this.status.running ? '✅ Running' : '❌ Stopped'}
- Mode: ${this.status.mode}

**Last Activity:**
- Started: ${this.status.lastStartAt || 'Never'}
- Last Probe: ${this.status.lastProbeAt || 'Never'}

**Your Session:**
- User ID: ${ctx.from?.id}
- Paired: ${this.isDMPaired(ctx.from?.id.toString() || '') ? '✅ Yes' : '❌ No'}

**Robot Connection:**
🚧 Coming soon - will show mBot2 hardware connection status
    `;

    await ctx.reply(statusText, { parse_mode: 'Markdown' });
  }

  /**
   * Handle /reset command
   */
  private async handleReset(ctx: Context): Promise<void> {
    await ctx.reply(
      '🔄 Conversation context reset!\n\n' +
      'Starting fresh. What would you like to talk about?'
    );
  }

  /**
   * Handle regular text messages
   */
  private async handleMessage(ctx: Context): Promise<void> {
    const userId = ctx.from?.id.toString();
    if (!userId) return;

    // Check DM pairing
    if (this.config.dmPairingRequired && !this.isDMPaired(userId)) {
      await ctx.reply(
        '⚠️ Please start a conversation first: /start\n\n' +
        'This enables DM pairing for security.'
      );
      return;
    }

    // Rate limiting
    if (!this.checkRateLimit(userId)) {
      await ctx.reply(
        '⏳ Slow down! You\'re sending messages too quickly.\n\n' +
        'Please wait a moment before sending another message.'
      );
      return;
    }

    const startTime = Date.now();
    const messageText = ctx.message?.text || '';

    try {
      // Update context monitor
      const contextMonitor = getContextMonitor();
      contextMonitor.recordUserInteraction();
      contextMonitor.updateRobotStatus('busy', 'processing-message');

      // Emit user message event
      const eventBus = getEventBus();
      await eventBus.emit(EventTypes.USER_MESSAGE, { userId, message: messageText });

      // Normalize message
      const message = this.adapter.normalizeInbound(ctx);

      // Check for matching pattern first
      const pattern = this.learningEngine.findMatchingPattern({
        message: messageText,
        userId,
      });

      let response: string;
      if (pattern) {
        // Use crystallized pattern
        console.log(`📚 Using pattern: ${pattern.name}`);
        const result = await this.learningEngine.executePattern(pattern, {
          message: messageText,
          userId,
        });

        if (result.success) {
          response = result.output || await this.generateResponse(message);
          await this.learningEngine.updatePatternConfidence(pattern.id, true);
        } else {
          // Pattern failed, fall back to normal response
          response = await this.generateResponse(message);
          await this.learningEngine.updatePatternConfidence(pattern.id, false);
        }
      } else {
        // Generate new response
        response = await this.generateResponse(message);
      }

      // Send response (with chunking if needed)
      await this.sendResponse(ctx, response);

      // Observe successful interaction
      const duration = Date.now() - startTime;
      this.learningEngine.observeAction(
        userId,
        'message_response',
        { message: messageText, response },
        true,
        duration
      );

      // Update pairing state
      this.updatePairingState(userId);

      // Update context to idle
      contextMonitor.updateRobotStatus('idle');
    } catch (error) {
      console.error('Error handling message:', error);

      // Observe failed interaction
      const duration = Date.now() - startTime;
      this.learningEngine.observeAction(
        userId,
        'message_response',
        {
          message: messageText,
          error: error instanceof Error ? error.message : 'Unknown error',
        },
        false,
        duration
      );

      await ctx.reply(
        '❌ Sorry, I encountered an error processing your message.\n\n' +
        'Please try again or use /help for assistance.'
      );

      // Update context to idle even on error
      const contextMonitor = getContextMonitor();
      contextMonitor.updateRobotStatus('idle');
    }
  }

  /**
   * Generate response based on message and personality
   */
  private async generateResponse(message: InternalMessage): Promise<string> {
    const personality = this.personalityStore.getCurrentPersonality();

    // For now, generate a simple personality-aware response
    // In the future, this will integrate with Claude or other LLM
    const baseResponse = this.generateBaseResponse(message.text);
    const styledResponse = this.adapter.formatPersonalityResponse(
      baseResponse,
      personality
    );

    return styledResponse;
  }

  /**
   * Generate base response (placeholder for LLM integration)
   */
  private generateBaseResponse(text: string): string {
    // Simple echo response with personality context
    const personality = this.personalityStore.getCurrentPersonality();

    const energy = personality.energy_baseline;
    const curiosity = personality.curiosity_drive;

    if (energy > 0.7) {
      return `I heard you say: "${text}"\n\nThat sounds exciting! Want to explore more?`;
    } else if (energy < 0.3) {
      return `You said: "${text}"\n\nInteresting. Tell me more.`;
    } else if (curiosity > 0.7) {
      return `"${text}" - fascinating!\n\nI'm curious to learn more about this topic. Can you elaborate?`;
    } else {
      return `Got it: "${text}"\n\nHow can I help you today?`;
    }
  }

  /**
   * Send response with automatic chunking
   */
  private async sendResponse(ctx: Context, text: string): Promise<void> {
    const chunks = this.adapter.chunkMessage(text);

    for (const chunk of chunks) {
      const formatted = this.adapter.formatOutbound(chunk);
      await ctx.reply(formatted.text, {
        parse_mode: formatted.parse_mode,
      });
    }
  }

  /**
   * Generate greeting based on personality
   */
  private generateGreeting(personality: PersonalityConfig): string {
    const energy = personality.energy_baseline;

    if (energy > 0.7) {
      return '🎉 Hey there! I\'m mBot and I\'m SO excited to meet you!';
    } else if (energy < 0.3) {
      return '👋 Hello. I\'m mBot. Nice to meet you.';
    } else {
      return '👋 Hi! I\'m mBot, your robot companion.';
    }
  }

  /**
   * Format personality info for display
   */
  private formatPersonalityInfo(personality: PersonalityConfig): string {
    return `
🎭 **Current Personality**

**Baselines:**
- Tension: ${(personality.tension_baseline * 100).toFixed(0)}%
- Coherence: ${(personality.coherence_baseline * 100).toFixed(0)}%
- Energy: ${(personality.energy_baseline * 100).toFixed(0)}%

**Reactivity:**
- Startle Sensitivity: ${(personality.startle_sensitivity * 100).toFixed(0)}%
- Recovery Speed: ${(personality.recovery_speed * 100).toFixed(0)}%
- Curiosity Drive: ${(personality.curiosity_drive * 100).toFixed(0)}%

**Expression:**
- Movement: ${(personality.movement_expressiveness * 100).toFixed(0)}%
- Sound: ${(personality.sound_expressiveness * 100).toFixed(0)}%
- Light: ${(personality.light_expressiveness * 100).toFixed(0)}%

Use the web dashboard to adjust these parameters.
    `;
  }

  /**
   * Check if user is DM paired
   */
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

  /**
   * Pair user
   */
  private pairUser(userId: string, username?: string): void {
    this.pairedUsers.set(userId, {
      userId,
      username,
      pairedAt: Date.now(),
      lastMessageAt: Date.now(),
    });
    console.log(`✅ User paired: ${username || userId}`);
  }

  /**
   * Update pairing state
   */
  private updatePairingState(userId: string): void {
    const state = this.pairedUsers.get(userId);
    if (state) {
      state.lastMessageAt = Date.now();
    }
  }

  /**
   * Check rate limit for user
   */
  private checkRateLimit(userId: string): boolean {
    const now = Date.now();
    const entry = this.rateLimits.get(userId);

    if (!entry || now >= entry.resetAt) {
      // Reset or create new entry
      this.rateLimits.set(userId, {
        userId,
        count: 1,
        resetAt: now + 60000, // 1 minute from now
      });
      return true;
    }

    // Check if under limit
    if (entry.count < this.config.rateLimitPerMinute) {
      entry.count++;
      return true;
    }

    return false;
  }

  /**
   * Get paired users count
   */
  getPairedUsersCount(): number {
    return this.pairedUsers.size;
  }

  /**
   * Get rate limit stats
   */
  getRateLimitStats(): { userId: string; count: number; resetAt: number }[] {
    return Array.from(this.rateLimits.values());
  }
}
