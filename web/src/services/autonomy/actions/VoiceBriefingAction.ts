/**
 * Voice Briefing Action - Integration with Autonomous Behavior Engine (#93)
 *
 * Registers voice briefing as an autonomous action that can be triggered
 * by time of day, user presence, or voice detection events
 */

import { AutonomousAction, ActionContext, ActionResult } from '../../../types/autonomy';
import { VoiceAssistant } from '../../voice/VoiceAssistant';
import { VoiceProfileService } from '../../voice/VoiceProfileService';

const voiceAssistant = new VoiceAssistant();
const voiceService = new VoiceProfileService();

/**
 * Morning Briefing Action
 *
 * Triggered at configured time each morning or when user voice is detected
 */
export const MorningBriefingAction: AutonomousAction = {
  id: 'voice-morning-briefing',
  name: 'Voice Morning Briefing',
  description: 'Deliver personalized morning briefing when user is detected',
  category: 'voice-assistant',
  enabled: true,

  triggers: [
    {
      type: 'cron',
      config: {
        schedule: '0 8 * * *', // 8 AM daily
        timezone: 'America/New_York'
      }
    },
    {
      type: 'event',
      config: {
        eventType: 'USER_VOICE_DETECTED',
        conditions: {
          timeWindow: { start: '06:00', end: '10:00' } // Only in morning window
        }
      }
    }
  ],

  preconditions: [
    {
      type: 'user_present',
      description: 'User must be detected nearby'
    },
    {
      type: 'time_of_day',
      description: 'Morning hours (6 AM - 10 AM)',
      config: { start: '06:00', end: '10:00' }
    },
    {
      type: 'not_recently_executed',
      description: 'Not delivered in last 12 hours',
      config: { cooldownMinutes: 720 }
    }
  ],

  async execute(context: ActionContext): Promise<ActionResult> {
    try {
      // Identify user if audio data provided
      let userId: string | null = null;

      if (context.audioData) {
        const identification = await voiceService.identifyUser(context.audioData);
        if (identification.confidence >= 0.85) {
          userId = identification.userId;
        }
      }

      // Use default user if no identification
      if (!userId && context.userId) {
        userId = context.userId;
      }

      if (!userId) {
        return {
          success: false,
          error: 'No user identified',
          data: {}
        };
      }

      // Trigger morning briefing
      await voiceAssistant.triggerMorningBriefing(userId);

      return {
        success: true,
        data: {
          userId,
          timestamp: Date.now(),
          actionType: 'morning_briefing'
        }
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
        data: {}
      };
    }
  },

  async rollback(context: ActionContext): Promise<void> {
    // Nothing to rollback for voice briefing
    console.log('MorningBriefingAction rollback (no-op)');
  }
};

/**
 * Proactive Check-In Action
 *
 * Checks in with user after period of inactivity
 */
export const ProactiveCheckInAction: AutonomousAction = {
  id: 'voice-proactive-checkin',
  name: 'Proactive Check-In',
  description: 'Check in with user after inactivity with follow-up questions',
  category: 'voice-assistant',
  enabled: true,

  triggers: [
    {
      type: 'event',
      config: {
        eventType: 'USER_INACTIVE',
        conditions: {
          inactivityMinutes: 120 // 2 hours
        }
      }
    },
    {
      type: 'event',
      config: {
        eventType: 'USER_RETURNS',
        conditions: {
          absenceMinutes: 240 // 4 hours
        }
      }
    }
  ],

  preconditions: [
    {
      type: 'user_present',
      description: 'User must be detected'
    },
    {
      type: 'has_pending_questions',
      description: 'Must have unanswered follow-up questions',
      config: { minQuestions: 1 }
    }
  ],

  async execute(context: ActionContext): Promise<ActionResult> {
    try {
      const userId = context.userId;

      if (!userId) {
        return {
          success: false,
          error: 'No user ID provided',
          data: {}
        };
      }

      // Check in with follow-up questions
      await voiceAssistant.checkInAfterInactivity(userId);

      return {
        success: true,
        data: {
          userId,
          timestamp: Date.now(),
          actionType: 'proactive_checkin'
        }
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
        data: {}
      };
    }
  },

  async rollback(context: ActionContext): Promise<void> {
    console.log('ProactiveCheckInAction rollback (no-op)');
  }
};

/**
 * Register voice actions with Autonomy Engine
 */
export function registerVoiceActions(autonomyEngine: any): void {
  autonomyEngine.registerAction(MorningBriefingAction);
  autonomyEngine.registerAction(ProactiveCheckInAction);

  console.log('Voice assistant actions registered with Autonomy Engine');
}
