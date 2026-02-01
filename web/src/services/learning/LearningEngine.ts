/**
 * LearningEngine - Core pattern detection and crystallization
 * Based on OpenClaw Foundry's self-learning architecture
 */

import {
  WorkflowObservation,
  CrystallizedPattern,
  PatternTrigger,
  PatternAction,
  LearningEngineConfig,
  ContextVector,
  SimilarityMatch,
  PatternExecutionResult,
  LearningInsight,
} from '../../types/learning';

/**
 * Default configuration following Foundry's thresholds
 */
const DEFAULT_CONFIG: LearningEngineConfig = {
  minObservations: 5, // Foundry: 5+ successful uses
  minSuccessRate: 0.7, // Foundry: 70%+ success rate
  minSimilarity: 0.8, // Cosine similarity threshold
  maxPatterns: 1000,
  staleThresholdDays: 30,
  observationRetentionDays: 90,
  enableAutoCleanup: true,
  enableProactiveSuggestions: true,
  enableAutoExecution: false,
};

/**
 * Main Learning Engine
 */
export class LearningEngine {
  private observations: Map<string, WorkflowObservation> = new Map();
  private patterns: Map<string, CrystallizedPattern> = new Map();
  private insights: Map<string, LearningInsight> = new Map();
  private config: LearningEngineConfig;

  // Performance tracking
  private tokensSaved: number = 0;
  private executionCount: number = 0;

  constructor(config?: Partial<LearningEngineConfig>) {
    this.config = { ...DEFAULT_CONFIG, ...config };
  }

  /**
   * Observe a user action
   */
  observeAction(
    userId: string,
    action: string,
    context: Record<string, any>,
    success: boolean,
    duration?: number
  ): string {
    const observation: WorkflowObservation = {
      id: this.generateObservationId(userId, action),
      userId,
      action,
      context,
      timestamp: Date.now(),
      success,
      duration: duration || 0,
    };

    this.observations.set(observation.id, observation);

    // Auto-cleanup if enabled
    if (this.config.enableAutoCleanup && this.observations.size > 10000) {
      this.pruneOldObservations();
    }

    return observation.id;
  }

  /**
   * Track interaction with detailed context
   */
  trackInteraction(
    userId: string,
    message: string,
    response: string,
    satisfaction: boolean,
    metadata?: Record<string, any>
  ): string {
    return this.observeAction(
      userId,
      'message_interaction',
      {
        message,
        response,
        satisfaction,
        ...metadata,
      },
      satisfaction
    );
  }

  /**
   * Detect patterns from observations
   * Foundry: 5+ uses, 70%+ success rate
   */
  detectPatterns(): CrystallizedPattern[] {
    const grouped = this.groupSimilarObservations();
    const candidates: CrystallizedPattern[] = [];

    for (const [key, observations] of grouped.entries()) {
      if (this.shouldCrystallize(observations)) {
        const pattern = this.crystallizePattern(observations);
        candidates.push(pattern);
      }
    }

    return candidates;
  }

  /**
   * Check if observations meet crystallization threshold
   */
  shouldCrystallize(observations: WorkflowObservation[]): boolean {
    if (observations.length < this.config.minObservations) {
      return false;
    }

    const successCount = observations.filter((o) => o.success).length;
    const successRate = successCount / observations.length;

    return successRate >= this.config.minSuccessRate;
  }

  /**
   * Crystallize a pattern from observations
   */
  crystallizePattern(observations: WorkflowObservation[]): CrystallizedPattern {
    const successCount = observations.filter((o) => o.success).length;
    const successRate = successCount / observations.length;

    // Extract common action and context
    const action = observations[0].action;
    const commonKeywords = this.extractCommonKeywords(observations);

    // Build trigger
    const trigger: PatternTrigger = {
      type: 'keyword',
      pattern: this.buildPatternRegex(commonKeywords),
      keywords: commonKeywords,
      minSimilarity: this.config.minSimilarity,
    };

    // Build action
    const actionSteps = this.extractActionSteps(observations);
    const patternAction: PatternAction = {
      type: 'workflow',
      description: `Automated workflow for: ${action}`,
      steps: actionSteps,
    };

    // Calculate metrics
    const totalDuration = observations.reduce((sum, o) => sum + o.duration, 0);
    const avgDuration = totalDuration / observations.length;

    const pattern: CrystallizedPattern = {
      id: this.generatePatternId(action),
      name: this.generatePatternName(action, commonKeywords),
      description: `Crystallized from ${observations.length} observations`,
      trigger,
      action: patternAction,
      confidence: successRate,
      usageCount: observations.length,
      successRate,
      avgDuration,
      createdAt: Date.now(),
      lastUsed: Date.now(),
      sourceObservations: observations.map((o) => o.id),
      crystallizedAt: Date.now(),
      version: 1,
      improvementTrajectory: [successRate],
    };

    return pattern;
  }

  /**
   * Find matching pattern for given context
   */
  findMatchingPattern(context: Record<string, any>): CrystallizedPattern | null {
    let bestMatch: CrystallizedPattern | null = null;
    let bestScore = 0;

    for (const pattern of this.patterns.values()) {
      const score = this.calculatePatternMatch(pattern, context);

      if (score > bestScore && score >= this.config.minSimilarity) {
        bestScore = score;
        bestMatch = pattern;
      }
    }

    return bestMatch;
  }

  /**
   * Execute a crystallized pattern
   */
  async executePattern(
    pattern: CrystallizedPattern,
    context: Record<string, any>
  ): Promise<PatternExecutionResult> {
    const startTime = Date.now();

    try {
      // Update usage stats
      pattern.usageCount++;
      pattern.lastUsed = Date.now();
      this.executionCount++;

      // Execute based on action type
      let output: any;
      switch (pattern.action.type) {
        case 'response':
          output = this.executeResponse(pattern, context);
          break;
        case 'workflow':
          output = await this.executeWorkflow(pattern, context);
          break;
        case 'tool_sequence':
          output = await this.executeToolSequence(pattern, context);
          break;
        default:
          throw new Error(`Unknown action type: ${pattern.action.type}`);
      }

      const duration = Date.now() - startTime;

      // Estimate tokens saved (rough heuristic)
      const estimatedTokensSaved = Math.floor(pattern.action.description.length / 4);
      this.tokensSaved += estimatedTokensSaved;

      return {
        success: true,
        patternId: pattern.id,
        duration,
        output,
      };
    } catch (error) {
      const duration = Date.now() - startTime;
      return {
        success: false,
        patternId: pattern.id,
        duration,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  /**
   * Update pattern confidence based on execution outcome
   */
  updatePatternConfidence(patternId: string, success: boolean): void {
    const pattern = this.patterns.get(patternId);
    if (!pattern) return;

    // Update success rate using exponential moving average
    const alpha = 0.2; // Learning rate
    const newSuccessRate = success
      ? pattern.successRate * (1 - alpha) + alpha
      : pattern.successRate * (1 - alpha);

    pattern.successRate = newSuccessRate;
    pattern.confidence = newSuccessRate;

    // Track trajectory
    if (!pattern.improvementTrajectory) {
      pattern.improvementTrajectory = [];
    }
    pattern.improvementTrajectory.push(newSuccessRate);

    // Auto-prune low-performing patterns
    if (pattern.usageCount > 10 && pattern.successRate < 0.5) {
      this.patterns.delete(patternId);
      this.addInsight({
        id: `insight-${Date.now()}`,
        type: 'frequent_failure',
        message: `Pattern "${pattern.name}" pruned due to low success rate (${(pattern.successRate * 100).toFixed(1)}%)`,
        confidence: 0.9,
        relatedPatterns: [patternId],
        createdAt: Date.now(),
      });
    }
  }

  /**
   * Prune stale patterns
   */
  pruneStalePatterns(thresholdDays?: number): string[] {
    const threshold = thresholdDays || this.config.staleThresholdDays;
    const cutoffTime = Date.now() - threshold * 24 * 60 * 60 * 1000;
    const prunedIds: string[] = [];

    for (const [id, pattern] of this.patterns.entries()) {
      if (pattern.lastUsed < cutoffTime) {
        this.patterns.delete(id);
        prunedIds.push(id);
      }
    }

    if (prunedIds.length > 0) {
      this.addInsight({
        id: `insight-${Date.now()}`,
        type: 'optimization',
        message: `Pruned ${prunedIds.length} stale patterns not used in ${threshold} days`,
        confidence: 1.0,
        relatedPatterns: prunedIds,
        createdAt: Date.now(),
      });
    }

    return prunedIds;
  }

  /**
   * Get all patterns
   */
  getPatterns(): CrystallizedPattern[] {
    return Array.from(this.patterns.values());
  }

  /**
   * Get pattern by ID
   */
  getPattern(id: string): CrystallizedPattern | undefined {
    return this.patterns.get(id);
  }

  /**
   * Add pattern manually
   */
  addPattern(pattern: CrystallizedPattern): void {
    this.patterns.set(pattern.id, pattern);
  }

  /**
   * Get all insights
   */
  getInsights(): LearningInsight[] {
    return Array.from(this.insights.values());
  }

  /**
   * Get performance stats
   */
  getPerformanceStats() {
    return {
      totalPatterns: this.patterns.size,
      totalObservations: this.observations.size,
      executionCount: this.executionCount,
      tokensSaved: this.tokensSaved,
      avgSuccessRate: this.calculateAvgSuccessRate(),
    };
  }

  // ========================================
  // Private helper methods
  // ========================================

  private generateObservationId(userId: string, action: string): string {
    return `obs-${userId}-${action}-${Date.now()}`;
  }

  private generatePatternId(action: string): string {
    return `pattern-${action}-${Date.now()}`;
  }

  private generatePatternName(action: string, keywords: string[]): string {
    if (keywords.length > 0) {
      return `${action}: ${keywords.slice(0, 3).join(', ')}`;
    }
    return action;
  }

  /**
   * Group observations by similarity
   */
  private groupSimilarObservations(): Map<string, WorkflowObservation[]> {
    const groups = new Map<string, WorkflowObservation[]>();

    for (const observation of this.observations.values()) {
      const key = this.getObservationKey(observation);
      const group = groups.get(key) || [];
      group.push(observation);
      groups.set(key, group);
    }

    return groups;
  }

  private getObservationKey(observation: WorkflowObservation): string {
    // Simple grouping by action + context keywords
    const keywords = this.extractKeywords(observation.context);
    return `${observation.action}:${keywords.sort().join(',')}`;
  }

  private extractKeywords(context: Record<string, any>): string[] {
    const keywords: string[] = [];

    for (const [key, value] of Object.entries(context)) {
      if (typeof value === 'string') {
        const words = value.toLowerCase().match(/\b\w{4,}\b/g) || [];
        keywords.push(...words);
      }
    }

    // Return unique keywords
    return Array.from(new Set(keywords));
  }

  private extractCommonKeywords(observations: WorkflowObservation[]): string[] {
    const keywordCounts = new Map<string, number>();

    for (const obs of observations) {
      const keywords = this.extractKeywords(obs.context);
      for (const keyword of keywords) {
        keywordCounts.set(keyword, (keywordCounts.get(keyword) || 0) + 1);
      }
    }

    // Get keywords that appear in at least 50% of observations
    const threshold = observations.length * 0.5;
    const common: string[] = [];

    for (const [keyword, count] of keywordCounts.entries()) {
      if (count >= threshold) {
        common.push(keyword);
      }
    }

    return common.slice(0, 5); // Top 5 keywords
  }

  private buildPatternRegex(keywords: string[]): string {
    if (keywords.length === 0) {
      return '.*';
    }
    return keywords.map((k) => `\\b${k}\\b`).join('|');
  }

  private extractActionSteps(observations: WorkflowObservation[]): string[] {
    // Extract common tool sequences
    const steps: string[] = [];

    for (const obs of observations) {
      if (obs.toolsUsed && obs.toolsUsed.length > 0) {
        steps.push(...obs.toolsUsed);
      }
    }

    // Return unique steps
    return Array.from(new Set(steps));
  }

  /**
   * Calculate pattern match score using cosine similarity
   */
  private calculatePatternMatch(
    pattern: CrystallizedPattern,
    context: Record<string, any>
  ): number {
    const contextKeywords = this.extractKeywords(context);
    const patternKeywords = pattern.trigger.keywords || [];

    if (contextKeywords.length === 0 || patternKeywords.length === 0) {
      return 0;
    }

    // Simple keyword overlap scoring
    const intersection = contextKeywords.filter((k) => patternKeywords.includes(k));
    const union = new Set([...contextKeywords, ...patternKeywords]);

    return intersection.length / union.size;
  }

  private executeResponse(
    pattern: CrystallizedPattern,
    context: Record<string, any>
  ): string {
    const template = pattern.action.responseTemplate || pattern.action.description;

    // Simple template substitution
    let response = template;
    for (const [key, value] of Object.entries(context)) {
      response = response.replace(`{${key}}`, String(value));
    }

    return response;
  }

  private async executeWorkflow(
    pattern: CrystallizedPattern,
    context: Record<string, any>
  ): Promise<any> {
    // Execute workflow steps sequentially
    const results: any[] = [];

    for (const step of pattern.action.steps || []) {
      // In real implementation, this would execute actual tools
      // For now, just log the step
      console.log(`Executing step: ${step}`, context);
      results.push({ step, status: 'completed' });
    }

    return { steps: results };
  }

  private async executeToolSequence(
    pattern: CrystallizedPattern,
    context: Record<string, any>
  ): Promise<any> {
    // Similar to workflow, but specifically for tool sequences
    return this.executeWorkflow(pattern, context);
  }

  private pruneOldObservations(): number {
    const cutoffTime =
      Date.now() - this.config.observationRetentionDays * 24 * 60 * 60 * 1000;
    let prunedCount = 0;

    for (const [id, observation] of this.observations.entries()) {
      if (observation.timestamp < cutoffTime) {
        this.observations.delete(id);
        prunedCount++;
      }
    }

    return prunedCount;
  }

  private addInsight(insight: LearningInsight): void {
    this.insights.set(insight.id, insight);

    // Keep only recent insights
    if (this.insights.size > 100) {
      const sorted = Array.from(this.insights.values()).sort(
        (a, b) => b.createdAt - a.createdAt
      );
      this.insights.clear();
      sorted.slice(0, 100).forEach((i) => this.insights.set(i.id, i));
    }
  }

  private calculateAvgSuccessRate(): number {
    if (this.patterns.size === 0) return 0;

    const total = Array.from(this.patterns.values()).reduce(
      (sum, p) => sum + p.successRate,
      0
    );

    return total / this.patterns.size;
  }
}
