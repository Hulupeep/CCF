#!/usr/bin/env node
/**
 * Telegram Bot Entry Point
 * Main executable for running the mBot Telegram bot
 */

import 'dotenv/config';
import { TelegramBot } from './services/telegram/TelegramBot';
import { telegramConfig } from './config/telegram';

/**
 * Main function
 */
async function main() {
  console.log('🤖 mBot Telegram Bot Starting...\n');

  // Display configuration
  console.log('📋 Configuration:');
  console.log(`   - DM Pairing Required: ${telegramConfig.dmPairingRequired}`);
  console.log(`   - Rate Limit: ${telegramConfig.rateLimitPerMinute} messages/min`);
  console.log(`   - Command Prefix: ${telegramConfig.commandPrefix}`);
  console.log(`   - Development Mode: ${telegramConfig.isDevelopment}`);
  if (telegramConfig.allowedUsers) {
    console.log(`   - Allowed Users: ${telegramConfig.allowedUsers.length} configured`);
  }
  console.log('');

  // Create bot instance
  const bot = new TelegramBot(telegramConfig);

  // Graceful shutdown handlers
  const shutdown = async (signal: string) => {
    console.log(`\n📡 Received ${signal}, shutting down gracefully...`);
    await bot.stop();
    process.exit(0);
  };

  process.once('SIGINT', () => shutdown('SIGINT'));
  process.once('SIGTERM', () => shutdown('SIGTERM'));

  // Handle uncaught errors
  process.on('uncaughtException', (error) => {
    console.error('❌ Uncaught exception:', error);
    shutdown('uncaughtException');
  });

  process.on('unhandledRejection', (reason, promise) => {
    console.error('❌ Unhandled rejection at:', promise, 'reason:', reason);
  });

  // Start bot
  try {
    await bot.start();

    // Display status every 5 minutes
    setInterval(() => {
      const status = bot.getStatus();
      console.log(`\n📊 Status Update:`);
      console.log(`   - Running: ${status.running}`);
      console.log(`   - Bot: @${status.bot?.username || 'unknown'}`);
      console.log(`   - Paired Users: ${bot.getPairedUsersCount()}`);
      console.log(`   - Last Probe: ${status.lastProbeAt || 'Never'}`);
    }, 5 * 60 * 1000);

  } catch (error) {
    console.error('❌ Failed to start bot:', error);
    process.exit(1);
  }
}

// Run main function
if (require.main === module) {
  main().catch((error) => {
    console.error('❌ Fatal error:', error);
    process.exit(1);
  });
}

export { main };
