# Wave 7 API Reference

**Version:** 1.0.0
**Last Updated:** 2026-02-01

## Table of Contents

1. [Multi-Robot Coordination API](#multi-robot-coordination-api)
2. [Swarm Play Modes API](#swarm-play-modes-api)
3. [Cloud Sync API](#cloud-sync-api)
4. [Personality Marketplace API](#personality-marketplace-api)
5. [Learning from Play API](#learning-from-play-api)
6. [Predictive Behavior API](#predictive-behavior-api)
7. [Voice Control API](#voice-control-api)

---

## Multi-Robot Coordination API

### MultiRobotCoordination Service

```typescript
interface MultiRobotCoordination {
  // Coordination modes
  coordinateMovement(robots: Robot[], config: CoordinationConfig): Promise<void>;
  executeSwarmBehavior(behavior: SwarmBehavior, robots: Robot[]): Promise<void>;

  // Formation control
  formLine(robots: Robot[], spacing: number): Promise<void>;
  formCircle(robots: Robot[], radius: number): Promise<void>;
  formGrid(robots: Robot[], rows: number, cols: number): Promise<void>;

  // Synchronization
  syncActions(robots: Robot[], action: RobotAction): Promise<void>;
  syncState(robots: Robot[]): Promise<void>;

  // Collision avoidance
  enableCollisionAvoidance(enabled: boolean): void;
  setMinDistance(distance: number): void; // cm
}
```

### Types

```typescript
interface CoordinationConfig {
  mode: 'leader-follower' | 'consensus' | 'centralized' | 'distributed';
  formation: 'line' | 'circle' | 'grid' | 'custom';
  spacing: number; // cm
  leader?: string; // Robot ID
  collisionAvoidance: boolean;
}

interface RobotAction {
  type: 'move' | 'rotate' | 'stop' | 'custom';
  params: Record<string, any>;
  duration?: number; // milliseconds
}
```

---

## Swarm Play Modes API

### SwarmController

```typescript
interface SwarmController {
  // Behavior control
  setBehavior(behavior: SwarmBehavior, params: SwarmParams): void;
  getCurrentBehavior(): SwarmBehavior;

  // Swarm management
  addRobot(robot: Robot): void;
  removeRobot(robotId: string): void;
  getRobots(): Robot[];

  // Parameters
  setSpeed(speed: number): void; // 0.1-1.0
  setSpacing(spacing: number): void; // cm
  setFormation(formation: 'tight' | 'loose'): void;

  // Events
  on(event: SwarmEvent, callback: (data: any) => void): void;
}
```

### Swarm Behaviors

```typescript
type SwarmBehavior = 'follow' | 'circle' | 'wave' | 'random';

interface SwarmParams {
  speed: number; // 0.1-1.0
  spacing: number; // cm
  formation: 'tight' | 'loose';
  leaderRotation?: boolean; // For follow mode
  waveAmplitude?: number; // For wave mode
  randomSeed?: number; // For random mode
}

type SwarmEvent =
  | 'behavior_changed'
  | 'robot_added'
  | 'robot_removed'
  | 'formation_complete'
  | 'collision_detected';
```

---

## Cloud Sync API

See: [Cloud Sync API Documentation](CLOUD_SYNC_API.md) (Detailed spec in separate file)

### Quick Reference

```typescript
interface CloudSync {
  // Initialization
  initialize(config: CloudSyncConfig): Promise<void>;

  // Sync operations
  syncAll(): Promise<SyncResult>;
  syncData(types: DataType[]): Promise<SyncResult>;
  pushChanges(): Promise<void>;
  pullChanges(): Promise<void>;

  // Conflict resolution
  setConflictStrategy(strategy: 'last-write-wins' | 'manual'): void;
  resolveConflict(conflictId: string, resolution: 'local' | 'remote'): Promise<void>;

  // Events
  onSyncComplete(callback: () => void): void;
  onSyncError(callback: (error: Error) => void): void;
  onConflict(callback: (conflict: Conflict) => void): void;
}
```

---

## Personality Marketplace API

### PersonalityMarketplace Service

```typescript
interface PersonalityMarketplace {
  // Browse
  browse(filter: MarketplaceFilter): Promise<MarketplacePersonality[]>;
  search(query: string): Promise<MarketplacePersonality[]>;
  getFeatured(): Promise<MarketplacePersonality[]>;
  getPopular(limit?: number): Promise<MarketplacePersonality[]>;

  // Details
  getDetails(id: string): Promise<PersonalityDetails>;
  getReviews(id: string): Promise<Review[]>;

  // Download
  download(id: string): Promise<PersonalityConfig>;
  checkUpdates(id: string): Promise<boolean>;

  // Publish
  publish(data: PublishData): Promise<string>; // Returns personality ID
  update(id: string, data: Partial<PublishData>): Promise<void>;
  unpublish(id: string): Promise<void>;

  // Reviews
  submitReview(id: string, review: ReviewData): Promise<void>;
  updateReview(reviewId: string, review: ReviewData): Promise<void>;
}
```

### Types

```typescript
interface MarketplacePersonality {
  id: string;
  name: string;
  description: string;
  author: string;
  category: string;
  tags: string[];
  rating: number; // 0-5
  reviewCount: number;
  downloadCount: number;
  createdAt: number;
  updatedAt: number;
  thumbnail?: string;
}

interface MarketplaceFilter {
  category?: string;
  tags?: string[];
  minRating?: number;
  sortBy?: 'rating' | 'downloads' | 'recent';
  limit?: number;
}

interface PublishData {
  name: string;
  description: string;
  personality: PersonalityConfig;
  category: string;
  tags: string[];
  thumbnail?: string;
  license?: string;
}

interface Review {
  id: string;
  userId: string;
  userName: string;
  rating: number; // 1-5
  comment: string;
  createdAt: number;
  helpful: number; // Helpful vote count
}
```

---

## Learning from Play API

### LearningMonitor Service

```typescript
interface LearningMonitor {
  // Control
  enableLearning(): Promise<void>;
  disableLearning(): void;
  isLearningEnabled(): boolean;

  // Configuration
  setLearningRate(rate: number): void; // α (0.1-0.3)
  setDiscountFactor(gamma: number): void; // γ (0.9-0.99)
  setExplorationRate(epsilon: number): void; // ε
  setExplorationDecay(decay: number): void;

  // Training
  recordExperience(experience: Experience): void;
  trainBatch(batchSize: number): Promise<void>;

  // Policy
  getQValue(state: State, action: Action): number;
  getBestAction(state: State): Action;
  exportPolicy(): Promise<Policy>;
  importPolicy(policy: Policy): Promise<void>;

  // Monitoring
  onProgress(callback: (stats: LearningStats) => void): void;
  getStats(): LearningStats;
  reset(): void;
}
```

### Types

```typescript
interface Experience {
  state: State;
  action: Action;
  reward: number;
  nextState: State;
  done: boolean;
}

interface State {
  // Game-specific state representation
  features: number[];
  metadata?: Record<string, any>;
}

interface Action {
  type: string;
  params: Record<string, any>;
}

interface LearningStats {
  episodeCount: number;
  wins: number;
  losses: number;
  draws: number;
  winRate: number;
  avgReward: number;
  explorationRate: number;
  qValueStability: number; // 0-1, higher is more stable
}

interface Policy {
  version: string;
  qTable: Map<string, Map<string, number>>; // state -> action -> Q-value
  metadata: {
    episodes: number;
    totalReward: number;
    learningRate: number;
    discountFactor: number;
  };
}
```

---

## Predictive Behavior API

### PredictionEngine Service

```typescript
interface PredictionEngine {
  // Control
  start(): void;
  stop(): void;
  isRunning(): boolean;

  // Prediction
  predictNext(context: PredictionContext): Promise<Prediction>;
  predictSequence(context: PredictionContext, length: number): Promise<Prediction[]>;

  // Training
  recordAction(action: UserAction): void;
  updateModel(): Promise<void>;

  // Configuration
  setConfidenceThreshold(threshold: number): void; // 0-1
  setHistoryWindow(window: number): void; // Number of past actions to consider

  // Monitoring
  getAccuracy(): Promise<AccuracyMetrics>;
  exportModel(): Promise<PredictionModel>;
  importModel(model: PredictionModel): Promise<void>;
}
```

### Types

```typescript
interface PredictionContext {
  currentState: State;
  history: UserAction[];
  environment?: Record<string, any>;
}

interface UserAction {
  type: string;
  timestamp: number;
  params: Record<string, any>;
  outcome?: string;
}

interface Prediction {
  action: string;
  confidence: number; // 0-1
  timing: number; // Milliseconds until predicted action
  intent?: string; // Predicted intent
  alternatives: Array<{
    action: string;
    confidence: number;
  }>;
}

interface AccuracyMetrics {
  overallAccuracy: number; // 0-1
  precisionByAction: Map<string, number>;
  recallByAction: Map<string, number>;
  avgConfidence: number;
  calibrationError: number; // How well confidence matches accuracy
}
```

---

## Voice Control API

### VoiceCommands Service

```typescript
interface VoiceCommands {
  // Control
  start(): Promise<void>;
  stop(): void;
  isListening(): boolean;

  // Configuration
  setLanguage(language: SupportedLanguage): void;
  setWakeWord(word: string): void;
  enableWakeWord(enabled: boolean): void;

  // Commands
  registerCommand(command: CommandPattern, handler: CommandHandler): void;
  unregisterCommand(commandId: string): void;
  getRegisteredCommands(): CommandPattern[];

  // Recognition
  onCommand(callback: (command: RecognizedCommand) => void): void;
  onError(callback: (error: Error) => void): void;

  // NLP
  parseIntent(text: string): Promise<Intent>;
}
```

### Types

```typescript
type SupportedLanguage = 'en-US' | 'es-ES' | 'fr-FR' | 'de-DE' | 'ja-JP';

interface CommandPattern {
  id: string;
  patterns: string[]; // e.g., ["start drawing", "begin artwork"]
  description: string;
  category: string;
  parameters?: ParameterDefinition[];
}

interface RecognizedCommand {
  text: string;
  command: CommandPattern;
  parameters: Record<string, any>;
  confidence: number; // 0-1
  timestamp: number;
}

interface Intent {
  action: string;
  entities: Entity[];
  confidence: number;
}

interface Entity {
  type: string; // e.g., "personality", "game", "number"
  value: any;
  text: string;
}

type CommandHandler = (params: Record<string, any>) => void | Promise<void>;
```

---

## Performance Profiling API

### Profiler Service

```typescript
interface Profiler {
  // Control
  start(name: string): void;
  stop(name: string): ProfileResult;
  measure(name: string, fn: () => void | Promise<void>): Promise<ProfileResult>;

  // Marks
  mark(name: string): void;
  clearMarks(): void;

  // Reports
  getReport(): ProfileReport;
  exportFlamegraph(): Promise<string>; // SVG data
  exportJSON(): Promise<string>;

  // Monitoring
  onLongTask(callback: (task: LongTask) => void): void;
  setLongTaskThreshold(ms: number): void; // Default: 50ms
}
```

### Types

```typescript
interface ProfileResult {
  name: string;
  duration: number; // milliseconds
  startTime: number;
  endTime: number;
  metadata?: Record<string, any>;
}

interface ProfileReport {
  profiles: ProfileResult[];
  summary: {
    totalDuration: number;
    avgDuration: number;
    minDuration: number;
    maxDuration: number;
  };
  longTasks: LongTask[];
  timestamp: number;
}

interface LongTask {
  name: string;
  duration: number;
  threshold: number;
  stackTrace?: string;
  timestamp: number;
}
```

---

## Animation API

### Animation Utilities

```typescript
interface AnimationConfig {
  duration: number; // milliseconds
  easing: EasingFunction;
  delay?: number;
  onComplete?: () => void;
}

type EasingFunction =
  | 'linear'
  | 'easeIn'
  | 'easeOut'
  | 'easeInOut'
  | 'spring'
  | ((t: number) => number); // Custom easing

interface SpringConfig {
  tension: number; // Default: 170
  friction: number; // Default: 26
  mass?: number; // Default: 1
}

// Spring animation helper
function useSpring(value: number, config: SpringConfig): AnimatedValue;

// Gesture animation helper
function useGesture(config: GestureConfig): GestureHandlers;
```

---

## Contract References

All Wave 7 APIs enforce:

| Contract | Description | APIs Affected |
|----------|-------------|---------------|
| **I-COORD-001** | Collision avoidance required | Multi-Robot Coordination |
| **I-SWARM-001** | Min 2 robots for swarm | Swarm Play Modes |
| **I-CLOUD-001** | Encryption at rest | Cloud Sync |
| **I-LEARN-001** | Convergence guarantee | Learning from Play |
| **I-VOICE-001** | Wake word accuracy >95% | Voice Control |

---

**Last Updated:** 2026-02-01
**Status:** Production Ready ✅
