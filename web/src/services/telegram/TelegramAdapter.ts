/**
 * TelegramAdapter - Message normalization and formatting
 * Based on OpenClaw channel adapter patterns
 * Handles bidirectional message conversion between Telegram and internal format
 */

import { Context } from 'grammy';
import {
  InternalMessage,
  TelegramMessage,
  Attachment,
  ResponseStyle,
} from '../../types/telegram';
import { PersonalityConfig } from '../../types/personality';

/**
 * TelegramAdapter class
 * Handles message normalization (Telegram → Internal) and formatting (Internal → Telegram)
 */
export class TelegramAdapter {
  /**
   * Normalize incoming Telegram message to internal format
   * Based on OpenClaw normalization patterns
   */
  normalizeInbound(ctx: Context): InternalMessage {
    const message = ctx.message;
    const chat = ctx.chat;
    const from = ctx.from;

    if (!message || !chat || !from) {
      throw new Error('Invalid context: missing message, chat, or from');
    }

    return {
      id: message.message_id.toString(),
      userId: from.id.toString(),
      username: from.username,
      firstName: from.first_name,
      lastName: from.last_name,
      text: this.extractText(message),
      timestamp: message.date * 1000, // Convert to milliseconds
      chatId: chat.id.toString(),
      chatType: chat.type,
      replyToId: (message as any).reply_to_message?.message_id.toString(),
      threadId: (message as any).message_thread_id?.toString(),
      attachments: this.extractAttachments(message),
      metadata: {
        isBot: from.is_bot,
        languageCode: from.language_code,
      },
    };
  }

  /**
   * Extract text from message (handles captions for media)
   */
  private extractText(message: any): string {
    return message.text || message.caption || '';
  }

  /**
   * Extract media attachments from message
   * Based on OpenClaw media extraction
   */
  private extractAttachments(message: any): Attachment[] | undefined {
    const attachments: Attachment[] = [];

    // Photos (get largest resolution)
    if (message.photo && message.photo.length > 0) {
      const largest = message.photo[message.photo.length - 1];
      attachments.push({
        type: 'photo',
        fileId: largest.file_id,
        caption: message.caption,
      });
    }

    // Documents
    if (message.document) {
      attachments.push({
        type: 'document',
        fileId: message.document.file_id,
        caption: message.caption,
      });
    }

    // Videos
    if (message.video) {
      attachments.push({
        type: 'video',
        fileId: message.video.file_id,
        caption: message.caption,
      });
    }

    // Audio
    if (message.audio) {
      attachments.push({
        type: 'audio',
        fileId: message.audio.file_id,
        caption: message.caption,
      });
    }

    // Voice messages
    if (message.voice) {
      attachments.push({
        type: 'voice',
        fileId: message.voice.file_id,
      });
    }

    return attachments.length > 0 ? attachments : undefined;
  }

  /**
   * Format outbound message for Telegram
   * Converts Markdown to Telegram HTML
   */
  formatOutbound(text: string, options?: { replyMarkup?: any }): TelegramMessage {
    const htmlText = this.markdownToTelegramHtml(text);

    return {
      text: htmlText,
      parse_mode: 'HTML',
      reply_markup: options?.replyMarkup,
      disable_web_page_preview: true,
    };
  }

  /**
   * Convert Markdown to Telegram HTML
   * Based on OpenClaw chunking patterns
   */
  private markdownToTelegramHtml(markdown: string): string {
    return markdown
      .replace(/\*\*(.+?)\*\*/g, '<b>$1</b>') // Bold
      .replace(/\*(.+?)\*/g, '<i>$1</i>') // Italic
      .replace(/`(.+?)`/g, '<code>$1</code>') // Inline code
      .replace(/```([\s\S]+?)```/g, '<pre>$1</pre>') // Code block
      .replace(/\[(.+?)\]\((.+?)\)/g, '<a href="$2">$1</a>') // Links
      .replace(/&/g, '&amp;') // Escape ampersands
      .replace(/</g, '&lt;') // Escape < (but preserve our tags)
      .replace(/>/g, '&gt;'); // Escape >
  }

  /**
   * Chunk message for Telegram's 4096 character limit
   * Based on OpenClaw chunking strategy
   */
  chunkMessage(text: string, maxLength: number = 4000): string[] {
    if (text.length <= maxLength) {
      return [text];
    }

    const chunks: string[] = [];
    let currentChunk = '';

    // Split by paragraphs first
    const paragraphs = text.split('\n\n');

    for (const para of paragraphs) {
      if (currentChunk.length + para.length + 2 <= maxLength) {
        currentChunk += (currentChunk ? '\n\n' : '') + para;
      } else {
        if (currentChunk) chunks.push(currentChunk);

        // If single paragraph is too long, split by sentences
        if (para.length > maxLength) {
          const sentences = para.match(/[^.!?]+[.!?]+/g) || [para];
          currentChunk = '';
          for (const sentence of sentences) {
            if (currentChunk.length + sentence.length <= maxLength) {
              currentChunk += sentence;
            } else {
              if (currentChunk) chunks.push(currentChunk);
              // If single sentence is too long, hard split
              currentChunk = sentence.substring(0, maxLength);
              let remaining = sentence.substring(maxLength);
              while (remaining.length > 0) {
                chunks.push(currentChunk);
                currentChunk = remaining.substring(0, maxLength);
                remaining = remaining.substring(maxLength);
              }
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

  /**
   * Format response based on personality traits
   * Integrates with mBot personality system
   */
  formatPersonalityResponse(text: string, personality: PersonalityConfig): string {
    const style = this.getResponseStyle(personality);

    let formattedText = text;

    // Add enthusiasm markers based on energy level
    if (style.enthusiasm > 0.7) {
      formattedText = formattedText.replace(/\.$/g, '!');
    }

    // Add emojis if playful
    if (style.emoji && personality.light_expressiveness > 0.6) {
      const emojis = ['🤖', '✨', '🎨', '🎮', '🔧', '💡'];
      const randomEmoji = emojis[Math.floor(Math.random() * emojis.length)];
      formattedText = `${randomEmoji} ${formattedText}`;
    }

    // Adjust formality
    if (style.formality === 'casual') {
      formattedText = formattedText
        .replace(/Hello/g, 'Hey')
        .replace(/Greetings/g, 'Hi there');
    }

    return formattedText;
  }

  /**
   * Derive response style from personality parameters
   */
  getResponseStyle(personality: PersonalityConfig): ResponseStyle {
    return {
      enthusiasm: personality.energy_baseline,
      verbosity: personality.curiosity_drive > 0.7 ? 'detailed' : 'concise',
      emoji: personality.movement_expressiveness > 0.6,
      formality: personality.energy_baseline < 0.3 ? 'formal' : 'casual',
    };
  }

  /**
   * Escape HTML special characters
   */
  escapeHtml(text: string): string {
    return text
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#039;');
  }
}
