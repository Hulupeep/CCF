/**
 * Telegram bot configuration
 * Validates environment variables and exports typed config
 */

import { TelegramBotConfig } from '../types/telegram';

/**
 * Load and validate Telegram configuration from environment
 */
export function loadTelegramConfig(): TelegramBotConfig {
  const botToken = process.env.TELEGRAM_BOT_TOKEN;

  if (!botToken) {
    throw new Error(
      'TELEGRAM_BOT_TOKEN is required. Please set it in your .env file.\n' +
      'Get a bot token from @BotFather on Telegram.'
    );
  }

  // Parse allowed users (comma-separated user IDs)
  const allowedUsersStr = process.env.TELEGRAM_ALLOWED_USERS || '';
  const allowedUsers = allowedUsersStr
    .split(',')
    .map((id) => id.trim())
    .filter((id) => id.length > 0);

  // Parse DM pairing requirement
  const dmPairingRequired = process.env.TELEGRAM_DM_PAIRING_REQUIRED !== 'false';

  // Parse rate limit
  const rateLimitPerMinute = parseInt(
    process.env.TELEGRAM_RATE_LIMIT_PER_MIN || '20',
    10
  );

  // Command prefix
  const commandPrefix = process.env.BOT_COMMAND_PREFIX || '/';

  // Development mode
  const isDevelopment = process.env.NODE_ENV !== 'production';

  return {
    botToken,
    allowedUsers: allowedUsers.length > 0 ? allowedUsers : undefined,
    dmPairingRequired,
    commandPrefix,
    rateLimitPerMinute,
    isDevelopment,
  };
}

/**
 * Validate configuration
 */
export function validateConfig(config: TelegramBotConfig): void {
  if (!config.botToken || config.botToken.trim().length === 0) {
    throw new Error('Bot token is required');
  }

  if (config.rateLimitPerMinute < 1 || config.rateLimitPerMinute > 1000) {
    throw new Error('Rate limit must be between 1 and 1000');
  }

  if (!config.commandPrefix || config.commandPrefix.length === 0) {
    throw new Error('Command prefix is required');
  }
}

/**
 * Get default personality name from environment
 */
export function getDefaultPersonality(): string {
  return process.env.BOT_PERSONALITY_DEFAULT || 'curious';
}

/**
 * Export configuration
 */
export const telegramConfig = loadTelegramConfig();
validateConfig(telegramConfig);
