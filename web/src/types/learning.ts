/**
 * Learning System Type Definitions
 * Based on OpenClaw Foundry patterns
 */

/**
 * Individual observation of user action
 */
export interface WorkflowObservation {
  id: string;
  userId: string;
  action: string;
  context: Record<string, any>;
  timestamp: number;
  success: boolean;
  duration: number;

  // Optional metadata
  toolsUsed?: string[];
  errorMessage?: string;
  resolution?: string;
}

/**
 * Trigger condition for pattern matching
 */
export interface PatternTrigger {
  type: 'keyword' | 'action' | 'context' | 'composite';
  pattern: string | RegExp;
  keywords?: string[];
  minSimilarity?: number; // Cosine similarity threshold (0-1)
}

/**
 * Action to execute when pattern matches
 */
export interface PatternAction {
  type: 'response' | 'workflow' | 'tool_sequence' | 'custom';
  description: string;
  steps?: string[];
  responseTemplate?: string;
  customHandler?: string; // Function name for custom handlers
}

/**
 * Crystallized pattern from repeated workflows
 */
export interface CrystallizedPattern {
  id: string;
  name: string;
  description: string;
  trigger: PatternTrigger;
  action: PatternAction;

  // Performance metrics
  confidence: number; // 0-1 success rate
  usageCount: number;
  successRate: number;
  avgDuration: number;

  // Timestamps
  createdAt: number;
  lastUsed: number;

  // Learning metadata
  sourceObservations: string[]; // IDs of observations that created this
  crystallizedAt: number;

  // Evolution tracking (ADAS-style)
  version: number;
  improvementTrajectory?: number[]; // Historical success rates
}

/**
 * Pattern statistics for analytics
 */
export interface PatternStatistics {
  totalPatterns: number;
  activePatterns: number;
  totalUsages: number;
  avgSuccessRate: number;
  avgConfidence: number;
  topPatterns: CrystallizedPattern[];
  recentActivity: {
    timestamp: number;
    patternId: string;
    success: boolean;
  }[];
}

/**
 * Learning cycle report (from Overseer)
 */
export interface LearningReport {
  timestamp: number;
  cycleId: string;

  // Analysis results
  observationsAnalyzed: number;
  patternsDetected: number;
  crystallizationCandidates: CrystallizedPattern[];

  // Actions taken
  actionsTriggered: {
    crystallized: string[]; // New pattern IDs
    updated: string[]; // Updated pattern IDs
    pruned: string[]; // Removed pattern IDs
    insights: string[]; // Generated insights
  };

  // Performance metrics
  performanceMetrics: {
    patternUsage: Record<string, number>;
    avgSuccessRate: number;
    avgDuration: number;
    tokensSaved: number; // Estimated tokens saved by using patterns
  };

  // Recommendations
  recommendations: {
    type: 'crystallize' | 'prune' | 'update' | 'monitor';
    patternId: string;
    reason: string;
  }[];
}

/**
 * Context vector for similarity calculation
 */
export interface ContextVector {
  embedding: number[];
  keywords: string[];
  metadata: Record<string, any>;
}

/**
 * Similarity match result
 */
export interface SimilarityMatch {
  observationId: string;
  similarity: number;
  context: Record<string, any>;
}

/**
 * Learning engine configuration
 */
export interface LearningEngineConfig {
  // Crystallization thresholds
  minObservations: number; // Default: 5
  minSuccessRate: number; // Default: 0.7
  minSimilarity: number; // Default: 0.8

  // Pattern lifecycle
  maxPatterns: number; // Default: 1000
  staleThresholdDays: number; // Default: 30

  // Performance
  observationRetentionDays: number; // Default: 90
  enableAutoCleanup: boolean; // Default: true

  // Learning behavior
  enableProactiveSuggestions: boolean; // Default: true
  enableAutoExecution: boolean; // Default: false (require approval)
}

/**
 * Pattern store interface
 */
export interface PatternStore {
  savePattern(pattern: CrystallizedPattern): Promise<void>;
  loadPatterns(): Promise<CrystallizedPattern[]>;
  updatePattern(id: string, updates: Partial<CrystallizedPattern>): Promise<void>;
  deletePattern(id: string): Promise<void>;
  getPatternsByUser(userId: string): Promise<CrystallizedPattern[]>;
  getTopPatterns(limit: number): Promise<CrystallizedPattern[]>;
  getPatternStats(): Promise<PatternStatistics>;
}

/**
 * Observation store interface
 */
export interface ObservationStore {
  saveObservation(observation: WorkflowObservation): Promise<void>;
  loadObservations(filters?: ObservationFilters): Promise<WorkflowObservation[]>;
  deleteObservation(id: string): Promise<void>;
  getObservationsByUser(userId: string): Promise<WorkflowObservation[]>;
  pruneOldObservations(days: number): Promise<number>;
}

/**
 * Filters for observation queries
 */
export interface ObservationFilters {
  userId?: string;
  action?: string;
  success?: boolean;
  startTime?: number;
  endTime?: number;
  limit?: number;
}

/**
 * Pattern execution result
 */
export interface PatternExecutionResult {
  success: boolean;
  patternId: string;
  duration: number;
  output?: any;
  error?: string;
}

/**
 * Learning insight
 */
export interface LearningInsight {
  id: string;
  type: 'pattern_detected' | 'high_success' | 'frequent_failure' | 'optimization';
  message: string;
  confidence: number;
  relatedPatterns: string[];
  createdAt: number;
}
