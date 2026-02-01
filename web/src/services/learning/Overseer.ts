/**
 * Overseer - Autonomous learning agent
 * Runs periodic learning cycles to detect, crystallize, and prune patterns
 * Based on OpenClaw Foundry's self-improvement architecture
 */

import { LearningEngine } from './LearningEngine';
import { PatternStore } from './PatternStore';
import {
  LearningReport,
  CrystallizedPattern,
  LearningInsight,
} from '../../types/learning';

/**
 * Overseer configuration
 */
export interface OverseerConfig {
  intervalMs: number; // How often to run learning cycle (default: 1 hour)
  enableAutoCleanup: boolean; // Auto-prune stale patterns
  enableAutoCrystallization: boolean; // Auto-crystallize patterns
  requireApproval: boolean; // Require approval before crystallization
  minConfidenceForAuto: number; // Min confidence for auto-crystallization
}

const DEFAULT_CONFIG: OverseerConfig = {
  intervalMs: 3600000, // 1 hour
  enableAutoCleanup: true,
  enableAutoCrystallization: true,
  requireApproval: false,
  minConfidenceForAuto: 0.85,
};

/**
 * Autonomous learning agent that monitors and improves the system
 */
export class Overseer {
  private learningEngine: LearningEngine;
  private patternStore: PatternStore;
  private config: OverseerConfig;

  private intervalId: NodeJS.Timeout | null = null;
  private running: boolean = false;

  // Learning history
  private reports: LearningReport[] = [];
  private maxReports: number = 50;

  constructor(
    learningEngine: LearningEngine,
    patternStore: PatternStore,
    config?: Partial<OverseerConfig>
  ) {
    this.learningEngine = learningEngine;
    this.patternStore = patternStore;
    this.config = { ...DEFAULT_CONFIG, ...config };
  }

  /**
   * Start the overseer
   */
  start(): void {
    if (this.running) {
      console.warn('Overseer already running');
      return;
    }

    console.log(`🔬 Overseer started (cycle every ${this.config.intervalMs / 1000}s)`);

    this.running = true;

    // Run immediately
    this.runLearningCycle().catch((error) => {
      console.error('Learning cycle failed:', error);
    });

    // Schedule periodic runs
    this.intervalId = setInterval(() => {
      this.runLearningCycle().catch((error) => {
        console.error('Learning cycle failed:', error);
      });
    }, this.config.intervalMs);
  }

  /**
   * Stop the overseer
   */
  stop(): void {
    if (!this.running) {
      return;
    }

    console.log('🛑 Overseer stopped');

    this.running = false;

    if (this.intervalId) {
      clearInterval(this.intervalId);
      this.intervalId = null;
    }
  }

  /**
   * Run a single learning cycle
   */
  async runLearningCycle(): Promise<LearningReport> {
    const cycleId = `cycle-${Date.now()}`;
    console.log(`🔬 Running learning cycle: ${cycleId}`);

    const startTime = Date.now();

    // 1. Analyze recent observations
    const newPatterns = this.learningEngine.detectPatterns();
    console.log(`  📊 Detected ${newPatterns.length} pattern candidates`);

    // 2. Evaluate existing patterns
    await this.evaluatePatternPerformance();

    // 3. Crystallize high-confidence patterns
    const crystallized: string[] = [];
    for (const pattern of newPatterns) {
      if (await this.shouldCrystallizePattern(pattern)) {
        await this.crystallizePattern(pattern);
        crystallized.push(pattern.id);
      }
    }
    console.log(`  ✨ Crystallized ${crystallized.length} patterns`);

    // 4. Prune stale patterns
    const pruned: string[] = [];
    if (this.config.enableAutoCleanup) {
      const prunedIds = this.learningEngine.pruneStalePatterns();
      pruned.push(...prunedIds);

      // Also prune from storage
      for (const id of prunedIds) {
        await this.patternStore.deletePattern(id);
      }
      console.log(`  🗑️  Pruned ${pruned.length} stale patterns`);
    }

    // 5. Generate insights
    const insights = await this.generateInsights();
    console.log(`  💡 Generated ${insights.length} insights`);

    // 6. Calculate performance metrics
    const stats = this.learningEngine.getPerformanceStats();
    const patternStats = await this.patternStore.getPatternStats();

    // 7. Build recommendations
    const recommendations = this.buildRecommendations(newPatterns);

    // 8. Create report
    const report: LearningReport = {
      timestamp: Date.now(),
      cycleId,
      observationsAnalyzed: stats.totalObservations,
      patternsDetected: newPatterns.length,
      crystallizationCandidates: newPatterns,
      actionsTriggered: {
        crystallized,
        updated: [],
        pruned,
        insights: insights.map((i) => i.message),
      },
      performanceMetrics: {
        patternUsage: this.calculatePatternUsage(),
        avgSuccessRate: stats.avgSuccessRate,
        avgDuration: 0, // TODO: Calculate from observations
        tokensSaved: stats.tokensSaved,
      },
      recommendations,
    };

    // Store report
    this.reports.push(report);
    if (this.reports.length > this.maxReports) {
      this.reports.shift();
    }

    const duration = Date.now() - startTime;
    console.log(`✅ Learning cycle completed in ${duration}ms`);

    return report;
  }

  /**
   * Get all reports
   */
  getReports(): LearningReport[] {
    return [...this.reports];
  }

  /**
   * Get latest report
   */
  getLatestReport(): LearningReport | null {
    return this.reports.length > 0 ? this.reports[this.reports.length - 1] : null;
  }

  /**
   * Get overseer status
   */
  getStatus() {
    return {
      running: this.running,
      config: this.config,
      totalCycles: this.reports.length,
      lastCycle: this.getLatestReport(),
    };
  }

  // ========================================
  // Private methods
  // ========================================

  /**
   * Evaluate performance of existing patterns
   */
  private async evaluatePatternPerformance(): Promise<void> {
    const patterns = this.learningEngine.getPatterns();

    for (const pattern of patterns) {
      // Check if pattern needs updating
      if (pattern.usageCount > 20) {
        // Analyze recent trajectory
        const trajectory = pattern.improvementTrajectory || [];
        if (trajectory.length > 5) {
          const recentPerformance = trajectory.slice(-5);
          const avgRecent =
            recentPerformance.reduce((sum, v) => sum + v, 0) / recentPerformance.length;

          if (avgRecent < pattern.successRate * 0.8) {
            // Performance degrading
            console.log(`  ⚠️  Pattern ${pattern.name} performance degrading`);
          }
        }
      }

      // Persist updated pattern
      await this.patternStore.savePattern(pattern);
    }
  }

  /**
   * Check if pattern should be crystallized
   */
  private async shouldCrystallizePattern(
    pattern: CrystallizedPattern
  ): Promise<boolean> {
    // Check if auto-crystallization is enabled
    if (!this.config.enableAutoCrystallization) {
      return false;
    }

    // Check confidence threshold
    if (pattern.confidence < this.config.minConfidenceForAuto) {
      return false;
    }

    // Check if pattern already exists
    const existing = this.learningEngine.getPattern(pattern.id);
    if (existing) {
      return false;
    }

    // If approval required, skip for now
    // (In real implementation, would queue for approval)
    if (this.config.requireApproval) {
      console.log(`  📋 Pattern ${pattern.name} queued for approval`);
      return false;
    }

    return true;
  }

  /**
   * Crystallize a pattern
   */
  private async crystallizePattern(pattern: CrystallizedPattern): Promise<void> {
    // Add to learning engine
    this.learningEngine.addPattern(pattern);

    // Persist to storage
    await this.patternStore.savePattern(pattern);

    console.log(`  ✨ Crystallized pattern: ${pattern.name}`);
  }

  /**
   * Generate insights from learning data
   */
  private async generateInsights(): Promise<LearningInsight[]> {
    const insights: LearningInsight[] = [];
    const stats = this.learningEngine.getPerformanceStats();
    const patterns = this.learningEngine.getPatterns();

    // High success rate insight
    const highPerformers = patterns.filter((p) => p.successRate > 0.9);
    if (highPerformers.length > 0) {
      insights.push({
        id: `insight-high-success-${Date.now()}`,
        type: 'high_success',
        message: `${highPerformers.length} patterns have >90% success rate`,
        confidence: 0.95,
        relatedPatterns: highPerformers.map((p) => p.id),
        createdAt: Date.now(),
      });
    }

    // Pattern detection insight
    if (stats.totalPatterns > 10) {
      insights.push({
        id: `insight-pattern-count-${Date.now()}`,
        type: 'pattern_detected',
        message: `System has learned ${stats.totalPatterns} patterns from ${stats.totalObservations} observations`,
        confidence: 1.0,
        relatedPatterns: [],
        createdAt: Date.now(),
      });
    }

    // Token savings insight
    if (stats.tokensSaved > 1000) {
      insights.push({
        id: `insight-tokens-${Date.now()}`,
        type: 'optimization',
        message: `Estimated ${stats.tokensSaved} tokens saved by using crystallized patterns`,
        confidence: 0.8,
        relatedPatterns: [],
        createdAt: Date.now(),
      });
    }

    return insights;
  }

  /**
   * Build recommendations
   */
  private buildRecommendations(
    candidates: CrystallizedPattern[]
  ): LearningReport['recommendations'] {
    const recommendations: LearningReport['recommendations'] = [];

    // Recommend crystallization for high-confidence patterns
    for (const pattern of candidates) {
      if (pattern.confidence > 0.85 && !this.learningEngine.getPattern(pattern.id)) {
        recommendations.push({
          type: 'crystallize',
          patternId: pattern.id,
          reason: `High confidence (${(pattern.confidence * 100).toFixed(1)}%) with ${pattern.usageCount} observations`,
        });
      }
    }

    // Recommend monitoring for medium-confidence patterns
    for (const pattern of candidates) {
      if (pattern.confidence >= 0.7 && pattern.confidence <= 0.85) {
        recommendations.push({
          type: 'monitor',
          patternId: pattern.id,
          reason: `Medium confidence (${(pattern.confidence * 100).toFixed(1)}%) - needs more observations`,
        });
      }
    }

    return recommendations;
  }

  /**
   * Calculate pattern usage statistics
   */
  private calculatePatternUsage(): Record<string, number> {
    const patterns = this.learningEngine.getPatterns();
    const usage: Record<string, number> = {};

    for (const pattern of patterns) {
      usage[pattern.id] = pattern.usageCount;
    }

    return usage;
  }
}
