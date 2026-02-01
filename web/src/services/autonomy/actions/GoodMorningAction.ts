/**
 * Good Morning Action - Send cheerful morning greeting
 * Triggered daily at 8 AM
 */

import { AutonomousAction } from '../AutonomyEngine';
import { CronExamples } from '../CronScheduler';

export const goodMorningAction: AutonomousAction = {
  id: 'good-morning',
  name: 'Good Morning Message',
  description: 'Send cheerful morning greeting at 8 AM daily',
  trigger: {
    type: 'cron',
    condition: CronExamples.EVERY_DAY_8AM, // '0 8 * * *'
    cooldown: 86400000, // 24 hours
  },
  action: async (context) => {
    const { personality } = context;
    const greeting = generatePersonalizedGreeting(personality);

    console.log(`[GoodMorning] ${greeting}`);

    // TODO: Send via Telegram
    // await telegramBot.sendMessage(userId, greeting);
  },
  enabled: true,
  executionCount: 0,
  requiresApproval: false,
  metadata: {
    category: 'greeting',
    tags: ['morning', 'proactive', 'friendly'],
  },
};

function generatePersonalizedGreeting(personality: any): string {
  const { energy_baseline, curiosity_drive, movement_expressiveness } = personality;

  if (energy_baseline > 0.7 && movement_expressiveness > 0.7) {
    return "🌅 GOOD MORNING! ☀️ I'm SO ready to create something amazing today! What should we build?";
  }

  if (energy_baseline < 0.3) {
    return "☀️ Morning... Slowly waking up here. What's on the agenda today?";
  }

  if (curiosity_drive > 0.7) {
    return "🌄 Good morning! I've been thinking... want to try something new today?";
  }

  return "☀️ Good morning! Ready to make today awesome? What are we working on?";
}
