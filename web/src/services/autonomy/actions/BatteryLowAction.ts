/**
 * Battery Low Action - Alert when battery drops below 20%
 * Triggered by sensor.battery_low event
 */

import { AutonomousAction } from '../AutonomyEngine';
import { EventTypes } from '../EventBus';

export const batteryLowAction: AutonomousAction = {
  id: 'battery-low-alert',
  name: 'Low Battery Alert',
  description: 'Notify user when battery drops below 20%',
  trigger: {
    type: 'event',
    condition: EventTypes.SENSOR_BATTERY_LOW,
    cooldown: 3600000, // 1 hour (prevent spam)
  },
  action: async (context) => {
    const { systemContext } = context;
    const batteryLevel = systemContext.batteryLevel ?? 0;

    console.log(`[BatteryLow] Battery at ${batteryLevel}%`);

    const message = `⚠️ My battery is getting low (${batteryLevel}%). I might need to recharge soon!`;

    // TODO: Send via Telegram
    // await telegramBot.sendMessage(userId, message);
  },
  enabled: true,
  executionCount: 0,
  requiresApproval: false,
  metadata: {
    category: 'hardware',
    tags: ['battery', 'alert', 'maintenance'],
    priority: 'high',
  },
};

export const batteryCriticalAction: AutonomousAction = {
  id: 'battery-critical-alert',
  name: 'Critical Battery Alert',
  description: 'Urgent notification when battery drops below 10%',
  trigger: {
    type: 'event',
    condition: EventTypes.SENSOR_BATTERY_CRITICAL,
    cooldown: 1800000, // 30 minutes
  },
  action: async (context) => {
    const { systemContext } = context;
    const batteryLevel = systemContext.batteryLevel ?? 0;

    console.log(`[BatteryCritical] Battery CRITICAL at ${batteryLevel}%`);

    const message = `🚨 URGENT: Battery critically low (${batteryLevel}%)! I need to be charged NOW or I'll shut down!`;

    // TODO: Send via Telegram (high priority)
    // await telegramBot.sendMessage(userId, message, { priority: 'urgent' });
  },
  enabled: true,
  executionCount: 0,
  requiresApproval: false,
  metadata: {
    category: 'hardware',
    tags: ['battery', 'critical', 'urgent'],
    priority: 'critical',
  },
};
