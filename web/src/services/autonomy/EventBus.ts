/**
 * Event Bus - Centralized event system for autonomous behavior
 * Implements publish-subscribe pattern for event-driven triggers
 */

export type EventHandler = (event: Event) => void | Promise<void>;

export interface Event {
  type: string;
  timestamp: number;
  data: any;
  metadata?: Record<string, any>;
}

/**
 * Event Bus for coordinating autonomous actions
 */
export class EventBus {
  private handlers: Map<string, Set<EventHandler>> = new Map();
  private eventHistory: Event[] = [];
  private maxHistorySize = 100;

  /**
   * Register event handler
   */
  on(eventType: string, handler: EventHandler): void {
    if (!this.handlers.has(eventType)) {
      this.handlers.set(eventType, new Set());
    }
    this.handlers.get(eventType)!.add(handler);
  }

  /**
   * Unregister event handler
   */
  off(eventType: string, handler: EventHandler): void {
    const handlers = this.handlers.get(eventType);
    if (handlers) {
      handlers.delete(handler);
      if (handlers.size === 0) {
        this.handlers.delete(eventType);
      }
    }
  }

  /**
   * Emit event to all registered handlers
   */
  async emit(eventType: string, data: any, metadata?: Record<string, any>): Promise<void> {
    const event: Event = {
      type: eventType,
      timestamp: Date.now(),
      data,
      metadata,
    };

    // Store in history
    this.eventHistory.push(event);
    if (this.eventHistory.length > this.maxHistorySize) {
      this.eventHistory.shift();
    }

    // Notify handlers
    const handlers = this.handlers.get(eventType);
    if (!handlers || handlers.size === 0) {
      return;
    }

    const promises = Array.from(handlers).map(async (handler) => {
      try {
        await handler(event);
      } catch (error) {
        console.error(`[EventBus] Error in handler for ${eventType}:`, error);
      }
    });

    await Promise.all(promises);
  }

  /**
   * Get recent events of a specific type
   */
  getRecentEvents(eventType: string, limit = 10): Event[] {
    return this.eventHistory
      .filter((e) => e.type === eventType)
      .slice(-limit);
  }

  /**
   * Get all recent events
   */
  getAllRecentEvents(limit = 50): Event[] {
    return this.eventHistory.slice(-limit);
  }

  /**
   * Clear all handlers
   */
  clear(): void {
    this.handlers.clear();
  }

  /**
   * Get handler count for event type
   */
  getHandlerCount(eventType: string): number {
    return this.handlers.get(eventType)?.size || 0;
  }

  /**
   * Get all registered event types
   */
  getEventTypes(): string[] {
    return Array.from(this.handlers.keys());
  }
}

/**
 * Singleton instance
 */
let eventBusInstance: EventBus | null = null;

export function getEventBus(): EventBus {
  if (!eventBusInstance) {
    eventBusInstance = new EventBus();
  }
  return eventBusInstance;
}

/**
 * Standard event types
 */
export const EventTypes = {
  // User events
  USER_MESSAGE: 'user.message',
  USER_INACTIVE: 'user.inactive',
  USER_RETURNED: 'user.returned',

  // Robot events
  ROBOT_CONNECTED: 'robot.connected',
  ROBOT_DISCONNECTED: 'robot.disconnected',
  ROBOT_IDLE: 'robot.idle',
  ROBOT_BUSY: 'robot.busy',

  // Personality events
  PERSONALITY_CHANGED: 'personality.changed',
  PERSONALITY_PARAMETER_UPDATED: 'personality.parameter_updated',

  // Sensor events
  SENSOR_MOTION: 'sensor.motion',
  SENSOR_BATTERY_LOW: 'sensor.battery_low',
  SENSOR_BATTERY_CRITICAL: 'sensor.battery_critical',
  SENSOR_TEMPERATURE: 'sensor.temperature',

  // System events
  SYSTEM_STARTUP: 'system.startup',
  SYSTEM_SHUTDOWN: 'system.shutdown',
  SYSTEM_ERROR: 'system.error',

  // Task events
  TASK_STARTED: 'task.started',
  TASK_COMPLETED: 'task.completed',
  TASK_FAILED: 'task.failed',

  // Learning events
  LEARNING_NEW_PATTERN: 'learning.new_pattern',
  LEARNING_MILESTONE: 'learning.milestone',
} as const;
