/**
 * Cron Scheduler - Time-based autonomous behavior triggers
 * Based on OpenClaw cron patterns with node-cron integration
 */

import cron, { ScheduledTask } from 'node-cron';

export interface CronJob {
  id: string;
  name: string;
  schedule: string; // Cron expression
  callback: () => void | Promise<void>;
  enabled: boolean;
  lastRun?: number;
  nextRun?: number;
  runCount: number;
  metadata?: Record<string, any>;
}

export interface CronJobConfig {
  id: string;
  name: string;
  schedule: string;
  enabled: boolean;
  metadata?: Record<string, any>;
}

/**
 * Cron scheduler for time-based autonomous actions
 */
export class CronScheduler {
  private jobs: Map<string, { config: CronJob; task: ScheduledTask }> = new Map();

  /**
   * Schedule a new cron job
   */
  schedule(config: CronJobConfig, callback: () => void | Promise<void>): void {
    // Validate cron expression
    if (!cron.validate(config.schedule)) {
      throw new Error(`Invalid cron expression: ${config.schedule}`);
    }

    // Check if job already exists
    if (this.jobs.has(config.id)) {
      throw new Error(`Job already exists: ${config.id}`);
    }

    const job: CronJob = {
      ...config,
      callback,
      runCount: 0,
    };

    // Create scheduled task with error handling
    const task = cron.schedule(
      config.schedule,
      async () => {
        if (!job.enabled) {
          return;
        }

        console.log(`[CronScheduler] Executing job: ${job.name} (${job.id})`);
        job.lastRun = Date.now();
        job.runCount++;

        try {
          await job.callback();
          console.log(`[CronScheduler] Job completed: ${job.name}`);
        } catch (error) {
          console.error(`[CronScheduler] Job failed: ${job.name}`, error);
        }
      },
      {
        scheduled: config.enabled,
      }
    );

    this.jobs.set(config.id, { config: job, task });
    console.log(`[CronScheduler] Scheduled job: ${job.name} (${config.schedule})`);
  }

  /**
   * Unschedule a job
   */
  unschedule(jobId: string): void {
    const entry = this.jobs.get(jobId);
    if (entry) {
      entry.task.stop();
      this.jobs.delete(jobId);
      console.log(`[CronScheduler] Unscheduled job: ${jobId}`);
    }
  }

  /**
   * Enable a job
   */
  enable(jobId: string): void {
    const entry = this.jobs.get(jobId);
    if (entry) {
      entry.config.enabled = true;
      entry.task.start();
      console.log(`[CronScheduler] Enabled job: ${jobId}`);
    }
  }

  /**
   * Disable a job
   */
  disable(jobId: string): void {
    const entry = this.jobs.get(jobId);
    if (entry) {
      entry.config.enabled = false;
      entry.task.stop();
      console.log(`[CronScheduler] Disabled job: ${jobId}`);
    }
  }

  /**
   * Get job status
   */
  getJob(jobId: string): CronJob | undefined {
    return this.jobs.get(jobId)?.config;
  }

  /**
   * Get all jobs
   */
  getAllJobs(): CronJob[] {
    return Array.from(this.jobs.values()).map((entry) => entry.config);
  }

  /**
   * Get enabled jobs count
   */
  getEnabledJobsCount(): number {
    return Array.from(this.jobs.values()).filter((entry) => entry.config.enabled).length;
  }

  /**
   * Stop all jobs
   */
  stopAll(): void {
    for (const [id, entry] of this.jobs) {
      entry.task.stop();
    }
    console.log(`[CronScheduler] Stopped ${this.jobs.size} jobs`);
  }

  /**
   * Clear all jobs
   */
  clear(): void {
    this.stopAll();
    this.jobs.clear();
    console.log('[CronScheduler] Cleared all jobs');
  }
}

/**
 * Singleton instance
 */
let schedulerInstance: CronScheduler | null = null;

export function getCronScheduler(): CronScheduler {
  if (!schedulerInstance) {
    schedulerInstance = new CronScheduler();
  }
  return schedulerInstance;
}

/**
 * Common cron expression examples
 */
export const CronExamples = {
  EVERY_MINUTE: '* * * * *',
  EVERY_5_MINUTES: '*/5 * * * *',
  EVERY_15_MINUTES: '*/15 * * * *',
  EVERY_30_MINUTES: '*/30 * * * *',
  EVERY_HOUR: '0 * * * *',
  EVERY_2_HOURS: '0 */2 * * *',
  EVERY_4_HOURS: '0 */4 * * *',
  EVERY_DAY_8AM: '0 8 * * *',
  EVERY_DAY_8PM: '0 20 * * *',
  WEEKDAYS_8AM: '0 8 * * 1-5',
  WEEKENDS_10AM: '0 10 * * 0,6',
  EVERY_SUNDAY_8PM: '0 20 * * 0',
  FIRST_OF_MONTH: '0 0 1 * *',
} as const;
