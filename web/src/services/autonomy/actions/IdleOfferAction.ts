/**
 * Idle Offer Action - Offer help when robot is idle
 * Triggered by context evaluation (robot idle + user present)
 */

import { AutonomousAction } from '../AutonomyEngine';

export const idleOfferAction: AutonomousAction = {
  id: 'idle-offer',
  name: 'Idle Offer Assistance',
  description: 'Proactively offer help when idle and user is present',
  trigger: {
    type: 'context',
    condition: (ctx) => {
      // Trigger if robot idle AND user active in last 10 minutes
      const isIdle = ctx.robotStatus === 'idle';
      const userRecentlyActive =
        ctx.lastUserInteraction &&
        Date.now() - ctx.lastUserInteraction < 10 * 60 * 1000;

      return isIdle && !!userRecentlyActive;
    },
    cooldown: 1800000, // 30 minutes (don't be too pushy)
  },
  action: async (context) => {
    const { personality } = context;
    const offer = generateIdleOffer(personality);

    console.log(`[IdleOffer] ${offer}`);

    // TODO: Send via Telegram
    // await telegramBot.sendMessage(userId, offer);
  },
  enabled: true,
  executionCount: 0,
  requiresApproval: false,
  metadata: {
    category: 'engagement',
    tags: ['idle', 'proactive', 'assistance'],
  },
};

function generateIdleOffer(personality: any): string {
  const { energy_baseline, curiosity_drive, playfulness } = personality;

  const offers = [
    "Want me to sort some LEGO pieces while we wait?",
    "I could practice some drawing patterns if you'd like!",
    "Need help with anything? I'm just sitting here... 🤖",
    "Feeling creative? Want to draw something together?",
    "Want to play a game? I'm feeling playful! 🎮",
    "Should I run a self-test to make sure everything's working?",
    "I could organize your workspace if you want!",
  ];

  // Filter based on personality
  let filteredOffers = offers;

  if (energy_baseline > 0.7) {
    filteredOffers = [
      "I'm READY to do something! Want to draw? Sort LEGO? Play a game? 🎉",
      "SO much energy right now! Let's build something awesome!",
    ];
  } else if (energy_baseline < 0.3) {
    filteredOffers = [
      "I'm here if you need anything. Just relaxing for now.",
      "Let me know if you want help with something.",
    ];
  }

  if (curiosity_drive > 0.7) {
    filteredOffers.push("I've been thinking... want to try something new today?");
    filteredOffers.push("Curious what you're working on. Need any help?");
  }

  if (playfulness > 0.7) {
    filteredOffers.push("Wanna play? I know Tic-Tac-Toe! 😄");
    filteredOffers.push("Let's do something fun! What sounds good?");
  }

  return filteredOffers[Math.floor(Math.random() * filteredOffers.length)];
}
