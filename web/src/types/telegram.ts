/**
 * Telegram bot type definitions
 * Based on OpenClaw architecture patterns
 */

/**
 * Internal message format - normalized from Telegram
 */
export interface InternalMessage {
  id: string;
  userId: string;
  username?: string;
  firstName?: string;
  lastName?: string;
  text: string;
  timestamp: number;
  chatId: string;
  chatType: 'private' | 'group' | 'supergroup' | 'channel';
  replyToId?: string;
  threadId?: string;
  attachments?: Attachment[];
  metadata?: Record<string, any>;
}

/**
 * Telegram message format for sending
 */
export interface TelegramMessage {
  text: string;
  parse_mode: 'HTML' | 'Markdown';
  reply_markup?: any; // InlineKeyboard from grammy
  disable_web_page_preview?: boolean;
}

/**
 * Media attachment
 */
export interface Attachment {
  type: 'photo' | 'voice' | 'document' | 'video' | 'audio';
  fileId: string;
  url?: string;
  caption?: string;
}

/**
 * Response style based on personality parameters
 */
export interface ResponseStyle {
  enthusiasm: number; // 0-1 from energy_baseline
  verbosity: 'concise' | 'detailed'; // from curiosity_drive
  emoji: boolean; // from playfulness
  formality: 'casual' | 'formal'; // from energy_baseline
}

/**
 * Bot configuration
 */
export interface TelegramBotConfig {
  botToken: string;
  allowedUsers?: string[]; // Telegram user IDs
  dmPairingRequired: boolean;
  commandPrefix: string;
  rateLimitPerMinute: number;
  isDevelopment?: boolean;
}

/**
 * Bot status
 */
export interface TelegramBotStatus {
  configured: boolean;
  running: boolean;
  mode: 'polling' | 'webhook';
  lastStartAt?: string;
  lastProbeAt?: string;
  lastError?: string;
  bot?: {
    id: number;
    username: string;
    firstName: string;
  };
}

/**
 * User pairing state
 */
export interface PairingState {
  userId: string;
  username?: string;
  pairedAt: number;
  lastMessageAt: number;
}

/**
 * Message send result
 */
export interface SendResult {
  success: boolean;
  messageId?: string;
  error?: string;
}

/**
 * Command handler type
 */
export type CommandHandler = (ctx: any) => Promise<void>;

/**
 * Rate limit entry
 */
export interface RateLimitEntry {
  userId: string;
  count: number;
  resetAt: number;
}
