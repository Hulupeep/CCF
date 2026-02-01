/**
 * Autonomy Engine Integration Tests
 * Tests autonomous behavior system components
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import {
  AutonomyEngine,
  EventBus,
  CronScheduler,
  ContextMonitor,
  EventTypes,
  ContextPredicates,
  AutonomousAction,
  ActionContext,
} from '../../src/services/autonomy';

describe('Autonomy Engine', () => {
  let engine: AutonomyEngine;
  let eventBus: EventBus;
  let scheduler: CronScheduler;
  let contextMonitor: ContextMonitor;

  beforeEach(() => {
    engine = new AutonomyEngine({ enabled: true, safeMode: false });
    eventBus = new EventBus();
    scheduler = new CronScheduler();
    contextMonitor = new ContextMonitor();
  });

  afterEach(() => {
    engine.stop();
    scheduler.clear();
    eventBus.clear();
  });

  describe('Action Registration', () => {
    it('should register and store actions', () => {
      const action: Omit<AutonomousAction, 'executionCount' | 'lastExecuted'> = {
        id: 'test-action',
        name: 'Test Action',
        description: 'Test action for unit testing',
        trigger: { type: 'event', condition: 'test.event' },
        action: async () => {
          console.log('Test action executed');
        },
        enabled: true,
      };

      engine.registerAction(action);

      const retrieved = engine.getAction('test-action');
      expect(retrieved).toBeDefined();
      expect(retrieved?.name).toBe('Test Action');
      expect(retrieved?.executionCount).toBe(0);
    });

    it('should unregister actions', () => {
      const action: Omit<AutonomousAction, 'executionCount' | 'lastExecuted'> = {
        id: 'test-action',
        name: 'Test Action',
        description: 'Test',
        trigger: { type: 'event', condition: 'test.event' },
        action: async () => {},
        enabled: true,
      };

      engine.registerAction(action);
      expect(engine.getAction('test-action')).toBeDefined();

      engine.unregisterAction('test-action');
      expect(engine.getAction('test-action')).toBeUndefined();
    });

    it('should enable and disable actions', () => {
      const action: Omit<AutonomousAction, 'executionCount' | 'lastExecuted'> = {
        id: 'test-action',
        name: 'Test Action',
        description: 'Test',
        trigger: { type: 'event', condition: 'test.event' },
        action: async () => {},
        enabled: true,
      };

      engine.registerAction(action);

      engine.disableAction('test-action');
      expect(engine.getAction('test-action')?.enabled).toBe(false);

      engine.enableAction('test-action');
      expect(engine.getAction('test-action')?.enabled).toBe(true);
    });
  });

  describe('Cron Triggers', () => {
    it('should execute cron-based action', async () => {
      let executed = false;

      const action: Omit<AutonomousAction, 'executionCount' | 'lastExecuted'> = {
        id: 'cron-test',
        name: 'Cron Test',
        description: 'Test cron execution',
        trigger: {
          type: 'cron',
          condition: '* * * * * *', // Every second
        },
        action: async () => {
          executed = true;
        },
        enabled: true,
      };

      engine.registerAction(action);
      engine.start();

      // Wait for cron to fire
      await new Promise((resolve) => setTimeout(resolve, 1500));

      expect(executed).toBe(true);
      expect(engine.getAction('cron-test')?.executionCount).toBeGreaterThan(0);
    });

    it('should respect cooldown period', async () => {
      let executionCount = 0;

      const action: Omit<AutonomousAction, 'executionCount' | 'lastExecuted'> = {
        id: 'cooldown-test',
        name: 'Cooldown Test',
        description: 'Test cooldown enforcement',
        trigger: {
          type: 'cron',
          condition: '* * * * * *', // Every second
          cooldown: 3000, // 3 seconds cooldown
        },
        action: async () => {
          executionCount++;
        },
        enabled: true,
      };

      engine.registerAction(action);
      engine.start();

      // Wait for 5 seconds - should execute at most twice (0s and 3s)
      await new Promise((resolve) => setTimeout(resolve, 5000));

      expect(executionCount).toBeLessThanOrEqual(2);
    });
  });

  describe('Event Triggers', () => {
    it('should execute event-based action', async () => {
      let executed = false;
      let eventData: any = null;

      const action: Omit<AutonomousAction, 'executionCount' | 'lastExecuted'> = {
        id: 'event-test',
        name: 'Event Test',
        description: 'Test event execution',
        trigger: {
          type: 'event',
          condition: EventTypes.USER_MESSAGE,
        },
        action: async (context) => {
          executed = true;
          eventData = context;
        },
        enabled: true,
      };

      engine.registerAction(action);
      engine.start();

      // Emit event
      await eventBus.emit(EventTypes.USER_MESSAGE, { userId: '123', message: 'hello' });

      // Give it time to execute
      await new Promise((resolve) => setTimeout(resolve, 100));

      expect(executed).toBe(true);
      expect(eventData).toBeDefined();
    });

    it('should handle battery low event', async () => {
      let alertSent = false;

      const action: Omit<AutonomousAction, 'executionCount' | 'lastExecuted'> = {
        id: 'battery-test',
        name: 'Battery Low Test',
        description: 'Test battery alert',
        trigger: {
          type: 'event',
          condition: EventTypes.SENSOR_BATTERY_LOW,
        },
        action: async (context) => {
          alertSent = true;
          expect(context.systemContext.batteryLevel).toBeLessThan(20);
        },
        enabled: true,
      };

      engine.registerAction(action);
      engine.start();

      // Update battery level
      contextMonitor.updateBatteryLevel(15);

      // Emit battery low event
      await eventBus.emit(EventTypes.SENSOR_BATTERY_LOW, { level: 15 });

      await new Promise((resolve) => setTimeout(resolve, 100));

      expect(alertSent).toBe(true);
    });
  });

  describe('Context Triggers', () => {
    it('should execute context-based action', async () => {
      let executed = false;

      const action: Omit<AutonomousAction, 'executionCount' | 'lastExecuted'> = {
        id: 'context-test',
        name: 'Context Test',
        description: 'Test context evaluation',
        trigger: {
          type: 'context',
          condition: (ctx) => ctx.timeOfDay === 'morning',
        },
        action: async () => {
          executed = true;
        },
        enabled: true,
      };

      engine.registerAction(action);
      engine.start();

      // Set context to morning
      const hour = 8; // 8 AM
      contextMonitor.updateContext({
        timeOfDay: 'morning',
        hour,
      });

      // Manually trigger evaluation (normally happens every minute)
      await engine['evaluateContextTriggers']();

      expect(executed).toBe(true);
    });

    it('should detect user inactivity', () => {
      // Set last interaction to 7 hours ago
      const sevenHoursAgo = Date.now() - 7 * 60 * 60 * 1000;
      contextMonitor.updateContext({ lastUserInteraction: sevenHoursAgo });

      const isInactive = contextMonitor.isInactive(6); // 6 hour threshold
      expect(isInactive).toBe(true);
    });

    it('should detect time ranges', () => {
      contextMonitor.updateContext({ hour: 10 }); // 10 AM

      const isMorning = contextMonitor.isTimeRange(6, 12); // 6 AM - 12 PM
      expect(isMorning).toBe(true);

      const isEvening = contextMonitor.isTimeRange(18, 22); // 6 PM - 10 PM
      expect(isEvening).toBe(false);
    });
  });

  describe('Personality-Driven Decisions', () => {
    it('should consider personality in action execution', async () => {
      let executionAttempts = 0;

      const action: Omit<AutonomousAction, 'executionCount' | 'lastExecuted'> = {
        id: 'personality-test',
        name: 'Personality Test',
        description: 'Test personality influence',
        trigger: {
          type: 'event',
          condition: 'test.event',
        },
        action: async () => {
          executionAttempts++;
        },
        enabled: true,
      };

      engine.registerAction(action);
      engine.start();

      // Emit multiple events - personality should prevent some executions
      for (let i = 0; i < 10; i++) {
        await eventBus.emit('test.event', {});
        await new Promise((resolve) => setTimeout(resolve, 50));
      }

      // Not all should execute due to personality randomization
      expect(executionAttempts).toBeGreaterThan(0);
      expect(executionAttempts).toBeLessThan(10);
    });
  });

  describe('Safety System', () => {
    it('should block actions in safe mode', async () => {
      const safeEngine = new AutonomyEngine({ enabled: true, safeMode: true });
      let executed = false;

      const action: Omit<AutonomousAction, 'executionCount' | 'lastExecuted'> = {
        id: 'safe-test',
        name: 'Safe Mode Test',
        description: 'Test safe mode blocking',
        trigger: {
          type: 'event',
          condition: 'test.event',
        },
        action: async () => {
          executed = true;
        },
        enabled: true,
        requiresApproval: true,
      };

      safeEngine.registerAction(action);
      safeEngine.start();

      await eventBus.emit('test.event', {});
      await new Promise((resolve) => setTimeout(resolve, 100));

      expect(executed).toBe(false); // Should be blocked
      safeEngine.stop();
    });

    it('should allow actions without approval requirement', async () => {
      const safeEngine = new AutonomyEngine({ enabled: true, safeMode: true });
      let executed = false;

      const action: Omit<AutonomousAction, 'executionCount' | 'lastExecuted'> = {
        id: 'safe-test',
        name: 'Safe Action Test',
        description: 'Test safe action execution',
        trigger: {
          type: 'event',
          condition: 'test.event',
        },
        action: async () => {
          executed = true;
        },
        enabled: true,
        requiresApproval: false, // Explicitly safe
      };

      safeEngine.registerAction(action);
      safeEngine.start();

      await eventBus.emit('test.event', {});
      await new Promise((resolve) => setTimeout(resolve, 100));

      expect(executed).toBe(true); // Should execute
      safeEngine.stop();
    });
  });

  describe('Statistics and Monitoring', () => {
    it('should track execution count', async () => {
      let count = 0;

      const action: Omit<AutonomousAction, 'executionCount' | 'lastExecuted'> = {
        id: 'count-test',
        name: 'Count Test',
        description: 'Test execution counting',
        trigger: {
          type: 'event',
          condition: 'test.event',
        },
        action: async () => {
          count++;
        },
        enabled: true,
      };

      engine.registerAction(action);
      engine.start();

      // Execute multiple times
      for (let i = 0; i < 5; i++) {
        await eventBus.emit('test.event', {});
        await new Promise((resolve) => setTimeout(resolve, 50));
      }

      const stats = engine.getStats();
      expect(stats.totalExecutions).toBeGreaterThan(0);
      expect(engine.getAction('count-test')?.executionCount).toBe(count);
    });

    it('should provide engine statistics', () => {
      const action1: Omit<AutonomousAction, 'executionCount' | 'lastExecuted'> = {
        id: 'action-1',
        name: 'Action 1',
        description: 'Test',
        trigger: { type: 'event', condition: 'test' },
        action: async () => {},
        enabled: true,
      };

      const action2: Omit<AutonomousAction, 'executionCount' | 'lastExecuted'> = {
        id: 'action-2',
        name: 'Action 2',
        description: 'Test',
        trigger: { type: 'event', condition: 'test' },
        action: async () => {},
        enabled: false,
      };

      engine.registerAction(action1);
      engine.registerAction(action2);

      const stats = engine.getStats();
      expect(stats.totalActions).toBe(2);
      expect(stats.enabledActions).toBe(1);
    });
  });
});

describe('Event Bus', () => {
  let eventBus: EventBus;

  beforeEach(() => {
    eventBus = new EventBus();
  });

  afterEach(() => {
    eventBus.clear();
  });

  it('should register and emit events', async () => {
    let received = false;
    let data: any = null;

    eventBus.on('test.event', (event) => {
      received = true;
      data = event.data;
    });

    await eventBus.emit('test.event', { message: 'hello' });

    expect(received).toBe(true);
    expect(data.message).toBe('hello');
  });

  it('should support multiple handlers', async () => {
    let count = 0;

    eventBus.on('test.event', () => count++);
    eventBus.on('test.event', () => count++);
    eventBus.on('test.event', () => count++);

    await eventBus.emit('test.event', {});

    expect(count).toBe(3);
  });

  it('should track event history', async () => {
    await eventBus.emit('event.1', { value: 1 });
    await eventBus.emit('event.2', { value: 2 });
    await eventBus.emit('event.1', { value: 3 });

    const event1History = eventBus.getRecentEvents('event.1');
    expect(event1History.length).toBe(2);
  });
});

describe('Context Monitor', () => {
  let contextMonitor: ContextMonitor;

  beforeEach(() => {
    contextMonitor = new ContextMonitor();
  });

  it('should track user interactions', () => {
    const before = contextMonitor.getContext();
    expect(before.lastUserInteraction).toBeUndefined();

    contextMonitor.recordUserInteraction();

    const after = contextMonitor.getContext();
    expect(after.lastUserInteraction).toBeDefined();
    expect(after.messageCount).toBe(1);
  });

  it('should detect time of day correctly', () => {
    const ctx = contextMonitor.getContext();
    expect(['morning', 'afternoon', 'evening', 'night']).toContain(ctx.timeOfDay);
  });

  it('should update robot status', () => {
    contextMonitor.updateRobotStatus('busy', 'processing-message');

    const ctx = contextMonitor.getContext();
    expect(ctx.robotStatus).toBe('busy');
    expect(ctx.currentActivity).toBe('processing-message');
  });

  it('should detect inactivity', () => {
    // Just interacted
    contextMonitor.recordUserInteraction();
    expect(contextMonitor.isInactive(5)).toBe(false);

    // Set to 6 hours ago
    const sixHoursAgo = Date.now() - 6 * 60 * 60 * 1000;
    contextMonitor.updateContext({ lastUserInteraction: sixHoursAgo });
    expect(contextMonitor.isInactive(5)).toBe(true);
  });
});
