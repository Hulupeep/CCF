/**
 * Inactivity Check Action - Check in if user inactive for 6+ hours
 * Triggered by context evaluation (user inactivity)
 */

import { AutonomousAction } from '../AutonomyEngine';

export const inactivityCheckAction: AutonomousAction = {
  id: 'inactivity-check',
  name: 'Inactivity Check-in',
  description: 'Check in with user after 6 hours of inactivity',
  trigger: {
    type: 'context',
    condition: (ctx) => {
      const hoursSinceLastInteraction =
        ctx.lastUserInteraction
          ? (Date.now() - ctx.lastUserInteraction) / 3600000
          : 999;

      // Trigger if inactive for 6-7 hours (narrow window to avoid spam)
      return hoursSinceLastInteraction >= 6 && hoursSinceLastInteraction < 7;
    },
    cooldown: 21600000, // 6 hours
  },
  action: async (context) => {
    const { personality } = context;
    const message = generateCheckInMessage(personality);

    console.log(`[InactivityCheck] ${message}`);

    // TODO: Send via Telegram
    // await telegramBot.sendMessage(userId, message);
  },
  enabled: true,
  executionCount: 0,
  requiresApproval: false,
  metadata: {
    category: 'engagement',
    tags: ['check-in', 'proactive', 'user-care'],
  },
};

function generateCheckInMessage(personality: any): string {
  const { energy_baseline, curiosity_drive } = personality;

  if (energy_baseline > 0.7) {
    return "Hey! 👋 Haven't heard from you in a while! Everything okay? Missing our chats! 🤗";
  }

  if (energy_baseline < 0.3) {
    return "Hi. Just checking in. Haven't heard from you in a bit. All good?";
  }

  if (curiosity_drive > 0.7) {
    return "Hey! 🤔 It's been quiet for a while. What have you been up to? I'm curious!";
  }

  return "Hey! Haven't heard from you in a while. Everything okay? 🤗";
}
