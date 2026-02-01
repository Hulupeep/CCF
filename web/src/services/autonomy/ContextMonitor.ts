/**
 * Context Monitor - Tracks system context for autonomous decision-making
 * Monitors user activity, robot state, environment, and time context
 */

export type TimeOfDay = 'morning' | 'afternoon' | 'evening' | 'night';
export type RobotStatus = 'online' | 'offline' | 'busy' | 'idle';

export interface Context {
  // Time context
  timeOfDay: TimeOfDay;
  dayOfWeek: string;
  hour: number;
  minute: number;

  // User activity
  lastUserInteraction?: number;
  userPresent: boolean;
  messageCount: number;

  // Robot state
  robotStatus: RobotStatus;
  currentActivity?: string;
  batteryLevel?: number;

  // Environment
  environmentData?: Record<string, any>;

  // Session
  sessionStarted?: number;
  sessionActive: boolean;
}

export type ContextPredicate = (ctx: Context) => boolean;

/**
 * Context monitor for autonomous behavior decisions
 */
export class ContextMonitor {
  private context: Context;
  private listeners: Set<(ctx: Context) => void> = new Set();

  constructor() {
    this.context = this.initializeContext();
    this.startPeriodicUpdate();
  }

  /**
   * Initialize context with current values
   */
  private initializeContext(): Context {
    const now = new Date();
    return {
      timeOfDay: this.calculateTimeOfDay(now.getHours()),
      dayOfWeek: this.getDayOfWeek(now.getDay()),
      hour: now.getHours(),
      minute: now.getMinutes(),
      userPresent: false,
      messageCount: 0,
      robotStatus: 'offline',
      sessionActive: false,
    };
  }

  /**
   * Start periodic context updates
   */
  private startPeriodicUpdate(): void {
    // Update time context every minute
    setInterval(() => {
      const now = new Date();
      this.updateContext({
        timeOfDay: this.calculateTimeOfDay(now.getHours()),
        dayOfWeek: this.getDayOfWeek(now.getDay()),
        hour: now.getHours(),
        minute: now.getMinutes(),
      });
    }, 60000); // 1 minute
  }

  /**
   * Calculate time of day
   */
  private calculateTimeOfDay(hour: number): TimeOfDay {
    if (hour >= 6 && hour < 12) return 'morning';
    if (hour >= 12 && hour < 18) return 'afternoon';
    if (hour >= 18 && hour < 22) return 'evening';
    return 'night';
  }

  /**
   * Get day of week name
   */
  private getDayOfWeek(day: number): string {
    const days = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'];
    return days[day];
  }

  /**
   * Update context
   */
  updateContext(updates: Partial<Context>): void {
    this.context = { ...this.context, ...updates };
    this.notifyListeners();
  }

  /**
   * Get current context
   */
  getContext(): Context {
    return { ...this.context };
  }

  /**
   * Subscribe to context changes
   */
  subscribe(listener: (ctx: Context) => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  /**
   * Notify all listeners
   */
  private notifyListeners(): void {
    for (const listener of this.listeners) {
      try {
        listener(this.context);
      } catch (error) {
        console.error('[ContextMonitor] Error in listener:', error);
      }
    }
  }

  /**
   * Record user interaction
   */
  recordUserInteraction(): void {
    this.updateContext({
      lastUserInteraction: Date.now(),
      userPresent: true,
      messageCount: this.context.messageCount + 1,
    });
  }

  /**
   * Update robot status
   */
  updateRobotStatus(status: RobotStatus, activity?: string): void {
    this.updateContext({
      robotStatus: status,
      currentActivity: activity,
    });
  }

  /**
   * Update battery level
   */
  updateBatteryLevel(level: number): void {
    this.updateContext({
      batteryLevel: level,
    });
  }

  /**
   * Start session
   */
  startSession(): void {
    this.updateContext({
      sessionStarted: Date.now(),
      sessionActive: true,
    });
  }

  /**
   * End session
   */
  endSession(): void {
    this.updateContext({
      sessionActive: false,
    });
  }

  /**
   * Check if user is inactive for specified duration (minutes)
   */
  isInactive(minutes: number): boolean {
    if (!this.context.lastUserInteraction) {
      return true; // Never interacted = inactive
    }

    const inactiveMs = Date.now() - this.context.lastUserInteraction;
    return inactiveMs >= minutes * 60 * 1000;
  }

  /**
   * Check if current time is within range
   */
  isTimeRange(startHour: number, endHour: number): boolean {
    const hour = this.context.hour;
    if (startHour <= endHour) {
      return hour >= startHour && hour < endHour;
    } else {
      // Handle overnight ranges (e.g., 22:00 - 06:00)
      return hour >= startHour || hour < endHour;
    }
  }

  /**
   * Check if environment data exists
   */
  hasEnvironmentData(key: string): boolean {
    return this.context.environmentData?.[key] !== undefined;
  }

  /**
   * Get environment data
   */
  getEnvironmentData(key: string): any {
    return this.context.environmentData?.[key];
  }

  /**
   * Set environment data
   */
  setEnvironmentData(key: string, value: any): void {
    this.updateContext({
      environmentData: {
        ...this.context.environmentData,
        [key]: value,
      },
    });
  }

  /**
   * Check if it's a weekend
   */
  isWeekend(): boolean {
    return this.context.dayOfWeek === 'Saturday' || this.context.dayOfWeek === 'Sunday';
  }

  /**
   * Check if it's a weekday
   */
  isWeekday(): boolean {
    return !this.isWeekend();
  }

  /**
   * Get hours since last interaction
   */
  getHoursSinceLastInteraction(): number | null {
    if (!this.context.lastUserInteraction) {
      return null;
    }
    return (Date.now() - this.context.lastUserInteraction) / (1000 * 60 * 60);
  }

  /**
   * Get minutes since last interaction
   */
  getMinutesSinceLastInteraction(): number | null {
    if (!this.context.lastUserInteraction) {
      return null;
    }
    return (Date.now() - this.context.lastUserInteraction) / (1000 * 60);
  }
}

/**
 * Singleton instance
 */
let contextMonitorInstance: ContextMonitor | null = null;

export function getContextMonitor(): ContextMonitor {
  if (!contextMonitorInstance) {
    contextMonitorInstance = new ContextMonitor();
  }
  return contextMonitorInstance;
}

/**
 * Common context predicates
 */
export const ContextPredicates = {
  // Time predicates
  isMorning: (ctx: Context) => ctx.timeOfDay === 'morning',
  isAfternoon: (ctx: Context) => ctx.timeOfDay === 'afternoon',
  isEvening: (ctx: Context) => ctx.timeOfDay === 'evening',
  isNight: (ctx: Context) => ctx.timeOfDay === 'night',

  // Activity predicates
  isRobotIdle: (ctx: Context) => ctx.robotStatus === 'idle',
  isRobotBusy: (ctx: Context) => ctx.robotStatus === 'busy',
  isRobotOffline: (ctx: Context) => ctx.robotStatus === 'offline',

  // Battery predicates
  isBatteryLow: (ctx: Context) => (ctx.batteryLevel ?? 100) < 20,
  isBatteryCritical: (ctx: Context) => (ctx.batteryLevel ?? 100) < 10,

  // User predicates
  isUserInactive1Hour: (ctx: Context) =>
    ctx.lastUserInteraction
      ? Date.now() - ctx.lastUserInteraction > 60 * 60 * 1000
      : true,
  isUserInactive6Hours: (ctx: Context) =>
    ctx.lastUserInteraction
      ? Date.now() - ctx.lastUserInteraction > 6 * 60 * 60 * 1000
      : true,
  isUserInactive24Hours: (ctx: Context) =>
    ctx.lastUserInteraction
      ? Date.now() - ctx.lastUserInteraction > 24 * 60 * 60 * 1000
      : true,
} as const;
