/**
 * Autonomy Engine - Core system for autonomous proactive behavior
 * Coordinates cron scheduling, event-driven actions, and context-aware decisions
 * Based on OpenClaw autonomous behavior patterns
 */

import { CronScheduler, getCronScheduler, CronJobConfig } from './CronScheduler';
import { EventBus, getEventBus, Event, EventHandler } from './EventBus';
import { ContextMonitor, getContextMonitor, Context, ContextPredicate } from './ContextMonitor';
import { PersonalityStore } from '../personalityStore';
import { PersonalityConfig } from '../../types/personality';

export interface TriggerCondition {
  type: 'cron' | 'event' | 'context';
  condition: string | EventHandler | ContextPredicate;
  cooldown?: number; // Minimum time between executions (ms)
}

export interface ActionContext {
  trigger: TriggerCondition;
  systemContext: Context;
  personality: PersonalityConfig;
  metadata?: Record<string, any>;
}

export type ActionFunction = (context: ActionContext) => void | Promise<void>;

export interface AutonomousAction {
  id: string;
  name: string;
  description: string;
  trigger: TriggerCondition;
  action: ActionFunction;
  enabled: boolean;
  lastExecuted?: number;
  executionCount: number;
  requiresApproval?: boolean;
  metadata?: Record<string, any>;
}

export interface AutonomyEngineConfig {
  enabled: boolean;
  safeMode: boolean; // Require approval for all actions
  maxConcurrentActions: number;
  defaultCooldown: number; // ms
}

/**
 * Autonomy Engine - Orchestrates autonomous behavior
 */
export class AutonomyEngine {
  private actions: Map<string, AutonomousAction> = new Map();
  private scheduler: CronScheduler;
  private eventBus: EventBus;
  private contextMonitor: ContextMonitor;
  private personalityStore: PersonalityStore;
  private config: AutonomyEngineConfig;
  private running = false;
  private contextCheckInterval?: NodeJS.Timeout;

  constructor(config?: Partial<AutonomyEngineConfig>) {
    this.config = {
      enabled: true,
      safeMode: false,
      maxConcurrentActions: 5,
      defaultCooldown: 60000, // 1 minute
      ...config,
    };

    this.scheduler = getCronScheduler();
    this.eventBus = getEventBus();
    this.contextMonitor = getContextMonitor();
    this.personalityStore = PersonalityStore.getInstance();
  }

  /**
   * Start the autonomy engine
   */
  start(): void {
    if (this.running) {
      console.warn('[AutonomyEngine] Already running');
      return;
    }

    console.log('[AutonomyEngine] Starting...');
    this.running = true;

    // Register all actions with their triggers
    for (const action of this.actions.values()) {
      if (action.enabled) {
        this.registerTrigger(action);
      }
    }

    // Start context-based evaluation loop
    this.startContextEvaluation();

    console.log(`[AutonomyEngine] Started with ${this.actions.size} actions`);
  }

  /**
   * Stop the autonomy engine
   */
  stop(): void {
    if (!this.running) {
      return;
    }

    console.log('[AutonomyEngine] Stopping...');
    this.running = false;

    // Stop context evaluation
    if (this.contextCheckInterval) {
      clearInterval(this.contextCheckInterval);
      this.contextCheckInterval = undefined;
    }

    // Stop scheduler (but keep jobs registered)
    // this.scheduler.stopAll();

    console.log('[AutonomyEngine] Stopped');
  }

  /**
   * Register an autonomous action
   */
  registerAction(action: Omit<AutonomousAction, 'executionCount' | 'lastExecuted'>): void {
    const fullAction: AutonomousAction = {
      ...action,
      executionCount: 0,
      lastExecuted: undefined,
    };

    this.actions.set(action.id, fullAction);
    console.log(`[AutonomyEngine] Registered action: ${action.name} (${action.id})`);

    // If engine is running and action is enabled, register trigger immediately
    if (this.running && action.enabled) {
      this.registerTrigger(fullAction);
    }
  }

  /**
   * Unregister an action
   */
  unregisterAction(actionId: string): void {
    const action = this.actions.get(actionId);
    if (!action) {
      return;
    }

    // Unregister trigger based on type
    if (action.trigger.type === 'cron') {
      this.scheduler.unschedule(actionId);
    } else if (action.trigger.type === 'event' && typeof action.trigger.condition === 'string') {
      // Event handlers can't be easily unregistered without storing references
      // This is a limitation - consider storing handler references
    }

    this.actions.delete(actionId);
    console.log(`[AutonomyEngine] Unregistered action: ${actionId}`);
  }

  /**
   * Enable an action
   */
  enableAction(actionId: string): void {
    const action = this.actions.get(actionId);
    if (!action) {
      console.warn(`[AutonomyEngine] Action not found: ${actionId}`);
      return;
    }

    action.enabled = true;

    if (this.running) {
      this.registerTrigger(action);
    }

    console.log(`[AutonomyEngine] Enabled action: ${action.name}`);
  }

  /**
   * Disable an action
   */
  disableAction(actionId: string): void {
    const action = this.actions.get(actionId);
    if (!action) {
      return;
    }

    action.enabled = false;

    if (action.trigger.type === 'cron') {
      this.scheduler.disable(actionId);
    }

    console.log(`[AutonomyEngine] Disabled action: ${action.name}`);
  }

  /**
   * Register trigger for an action
   */
  private registerTrigger(action: AutonomousAction): void {
    switch (action.trigger.type) {
      case 'cron':
        this.registerCronTrigger(action);
        break;
      case 'event':
        this.registerEventTrigger(action);
        break;
      case 'context':
        // Context triggers are evaluated in the evaluation loop
        break;
      default:
        console.warn(`[AutonomyEngine] Unknown trigger type for action: ${action.id}`);
    }
  }

  /**
   * Register cron trigger
   */
  private registerCronTrigger(action: AutonomousAction): void {
    if (typeof action.trigger.condition !== 'string') {
      console.warn(`[AutonomyEngine] Invalid cron expression for action: ${action.id}`);
      return;
    }

    const cronConfig: CronJobConfig = {
      id: action.id,
      name: action.name,
      schedule: action.trigger.condition,
      enabled: action.enabled,
      metadata: action.metadata,
    };

    this.scheduler.schedule(cronConfig, async () => {
      await this.executeAction(action);
    });
  }

  /**
   * Register event trigger
   */
  private registerEventTrigger(action: AutonomousAction): void {
    if (typeof action.trigger.condition !== 'string') {
      console.warn(`[AutonomyEngine] Invalid event type for action: ${action.id}`);
      return;
    }

    const eventType = action.trigger.condition;

    this.eventBus.on(eventType, async () => {
      await this.executeAction(action);
    });
  }

  /**
   * Start context-based evaluation loop
   */
  private startContextEvaluation(): void {
    // Evaluate context-based triggers every minute
    this.contextCheckInterval = setInterval(async () => {
      if (!this.config.enabled) {
        return;
      }

      await this.evaluateContextTriggers();
    }, 60000); // 1 minute
  }

  /**
   * Evaluate all context-based triggers
   */
  private async evaluateContextTriggers(): Promise<void> {
    const context = this.contextMonitor.getContext();

    for (const action of this.actions.values()) {
      if (!action.enabled || action.trigger.type !== 'context') {
        continue;
      }

      if (typeof action.trigger.condition !== 'function') {
        continue;
      }

      try {
        const shouldExecute = action.trigger.condition(context);
        if (shouldExecute) {
          await this.executeAction(action);
        }
      } catch (error) {
        console.error(`[AutonomyEngine] Error evaluating context trigger for ${action.id}:`, error);
      }
    }
  }

  /**
   * Execute an autonomous action
   */
  async executeAction(action: AutonomousAction): Promise<void> {
    if (!this.config.enabled || !action.enabled) {
      return;
    }

    // Check cooldown
    if (!this.checkCooldown(action)) {
      console.log(`[AutonomyEngine] Action ${action.id} on cooldown`);
      return;
    }

    // Check if action can be executed safely
    if (!this.canExecuteSafely(action)) {
      console.log(`[AutonomyEngine] Action ${action.id} blocked by safety system`);
      return;
    }

    // Check personality-driven decision
    const personality = this.personalityStore.getCurrentPersonality();
    const shouldExecute = await this.shouldTakeAction(action, personality);

    if (!shouldExecute) {
      console.log(`[AutonomyEngine] Personality decided not to execute ${action.id}`);
      return;
    }

    console.log(`[AutonomyEngine] Executing action: ${action.name}`);

    try {
      const actionContext: ActionContext = {
        trigger: action.trigger,
        systemContext: this.contextMonitor.getContext(),
        personality,
        metadata: action.metadata,
      };

      await action.action(actionContext);

      // Update execution metadata
      action.lastExecuted = Date.now();
      action.executionCount++;

      console.log(`[AutonomyEngine] Action completed: ${action.name} (total executions: ${action.executionCount})`);
    } catch (error) {
      console.error(`[AutonomyEngine] Action failed: ${action.name}`, error);
    }
  }

  /**
   * Check if action is within cooldown period
   */
  private checkCooldown(action: AutonomousAction): boolean {
    if (!action.lastExecuted) {
      return true; // Never executed, no cooldown
    }

    const cooldown = action.trigger.cooldown ?? this.config.defaultCooldown;
    const timeSinceLastExecution = Date.now() - action.lastExecuted;

    return timeSinceLastExecution >= cooldown;
  }

  /**
   * Check if action can be executed safely
   */
  private canExecuteSafely(action: AutonomousAction): boolean {
    // Safe mode requires approval for all actions
    if (this.config.safeMode && action.requiresApproval !== false) {
      return false;
    }

    // Check if action explicitly requires approval
    if (action.requiresApproval) {
      return false;
    }

    return true;
  }

  /**
   * Personality-driven decision making
   */
  private async shouldTakeAction(
    action: AutonomousAction,
    personality: PersonalityConfig
  ): Promise<boolean> {
    const { energy_baseline, curiosity_drive } = personality;

    // High energy = more proactive
    if (energy_baseline > 0.7) {
      return Math.random() < 0.9; // 90% chance
    }

    // Low energy = less proactive
    if (energy_baseline < 0.3) {
      return Math.random() < 0.5; // 50% chance
    }

    // High curiosity = more likely to take action
    if (curiosity_drive > 0.7) {
      return Math.random() < 0.8; // 80% chance
    }

    return Math.random() < 0.7; // Default 70% chance
  }

  /**
   * Request human approval (placeholder)
   */
  async requestHumanApproval(action: AutonomousAction): Promise<boolean> {
    // TODO: Implement approval flow
    console.log(`[AutonomyEngine] Approval requested for: ${action.name}`);
    return false;
  }

  /**
   * Get action by ID
   */
  getAction(actionId: string): AutonomousAction | undefined {
    return this.actions.get(actionId);
  }

  /**
   * Get all actions
   */
  getAllActions(): AutonomousAction[] {
    return Array.from(this.actions.values());
  }

  /**
   * Get enabled actions count
   */
  getEnabledActionsCount(): number {
    return Array.from(this.actions.values()).filter((a) => a.enabled).length;
  }

  /**
   * Get total execution count
   */
  getTotalExecutionCount(): number {
    return Array.from(this.actions.values()).reduce((sum, a) => sum + a.executionCount, 0);
  }

  /**
   * Get engine statistics
   */
  getStats() {
    return {
      running: this.running,
      totalActions: this.actions.size,
      enabledActions: this.getEnabledActionsCount(),
      totalExecutions: this.getTotalExecutionCount(),
      config: this.config,
    };
  }

  /**
   * Update configuration
   */
  updateConfig(updates: Partial<AutonomyEngineConfig>): void {
    this.config = { ...this.config, ...updates };
    console.log('[AutonomyEngine] Configuration updated:', this.config);
  }
}

/**
 * Singleton instance
 */
let autonomyEngineInstance: AutonomyEngine | null = null;

export function getAutonomyEngine(config?: Partial<AutonomyEngineConfig>): AutonomyEngine {
  if (!autonomyEngineInstance) {
    autonomyEngineInstance = new AutonomyEngine(config);
  }
  return autonomyEngineInstance;
}
