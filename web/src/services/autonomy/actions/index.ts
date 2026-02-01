/**
 * Built-in Autonomous Actions
 * Export all autonomous action definitions
 */

import { goodMorningAction } from './GoodMorningAction';
import { inactivityCheckAction } from './InactivityCheckAction';
import { batteryLowAction, batteryCriticalAction } from './BatteryLowAction';
import { weeklyRecapAction } from './WeeklyRecapAction';
import { idleOfferAction } from './IdleOfferAction';
import { AutonomousAction } from '../AutonomyEngine';

/**
 * All built-in autonomous actions
 */
export const builtInActions: AutonomousAction[] = [
  goodMorningAction,
  inactivityCheckAction,
  batteryLowAction,
  batteryCriticalAction,
  weeklyRecapAction,
  idleOfferAction,
];

/**
 * Register all built-in actions with the autonomy engine
 */
export function registerBuiltInActions(engine: any): void {
  for (const action of builtInActions) {
    engine.registerAction(action);
  }
  console.log(`[BuiltInActions] Registered ${builtInActions.length} built-in actions`);
}

// Re-export individual actions
export {
  goodMorningAction,
  inactivityCheckAction,
  batteryLowAction,
  batteryCriticalAction,
  weeklyRecapAction,
  idleOfferAction,
};
