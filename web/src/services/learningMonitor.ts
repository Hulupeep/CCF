/**
 * Learning Monitor Service
 *
 * Monitors and visualizes reinforcement learning progress.
 * Connects to mBot's learning system via WebSocket.
 */

export interface LearningMetrics {
  episodeCount: number;
  averageReward: number;
  winRate: number;
  lossRate: number;
  drawRate: number;
  epsilonCurrent: number;
  learningRateCurrent: number;
  convergenceScore: number;
}

export interface LearningConfig {
  learningRate: number;
  discountFactor: number;
  epsilonStart: number;
  epsilonEnd: number;
  epsilonDecay: number;
  maxEpisodes: number;
}

export interface RewardHistory {
  episode: number;
  reward: number;
  outcome: 'win' | 'loss' | 'draw';
  timestamp: number;
}

export type LearningStatus = 'idle' | 'training' | 'converged' | 'error';

export interface LearningState {
  status: LearningStatus;
  gameType: string;
  metrics: LearningMetrics;
  config: LearningConfig;
  rewardHistory: RewardHistory[];
  isEnabled: boolean;
}

/**
 * Learning Monitor - tracks RL progress
 */
export class LearningMonitor {
  private ws: WebSocket | null = null;
  private listeners: Map<string, Set<(data: any) => void>> = new Map();
  private state: LearningState;

  constructor(private wsUrl: string = 'ws://localhost:8080/learning') {
    this.state = {
      status: 'idle',
      gameType: '',
      metrics: this.getDefaultMetrics(),
      config: this.getDefaultConfig(),
      rewardHistory: [],
      isEnabled: false,
    };
  }

  /**
   * Connect to learning system
   */
  connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      try {
        this.ws = new WebSocket(this.wsUrl);

        this.ws.onopen = () => {
          console.log('[LearningMonitor] Connected');
          resolve();
        };

        this.ws.onerror = (error) => {
          console.error('[LearningMonitor] WebSocket error:', error);
          this.state.status = 'error';
          reject(error);
        };

        this.ws.onmessage = (event) => {
          this.handleMessage(event.data);
        };

        this.ws.onclose = () => {
          console.log('[LearningMonitor] Disconnected');
          this.state.status = 'idle';
        };
      } catch (error) {
        reject(error);
      }
    });
  }

  /**
   * Disconnect from learning system
   */
  disconnect(): void {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  /**
   * Enable learning for a game
   */
  enableLearning(gameType: string): void {
    this.send({
      type: 'enable_learning',
      gameType,
    });
    this.state.isEnabled = true;
    this.state.gameType = gameType;
  }

  /**
   * Disable learning
   */
  disableLearning(): void {
    this.send({
      type: 'disable_learning',
    });
    this.state.isEnabled = false;
  }

  /**
   * Save current policy
   */
  savePolicy(gameType: string): Promise<void> {
    return new Promise((resolve, reject) => {
      this.send({
        type: 'save_policy',
        gameType,
      });

      // Wait for confirmation
      const handler = (data: any) => {
        if (data.type === 'policy_saved') {
          this.off('message', handler);
          resolve();
        }
      };
      this.on('message', handler);

      setTimeout(() => {
        this.off('message', handler);
        reject(new Error('Save policy timeout'));
      }, 5000);
    });
  }

  /**
   * Load saved policy
   */
  loadPolicy(gameType: string): Promise<void> {
    return new Promise((resolve, reject) => {
      this.send({
        type: 'load_policy',
        gameType,
      });

      const handler = (data: any) => {
        if (data.type === 'policy_loaded') {
          this.off('message', handler);
          resolve();
        }
      };
      this.on('message', handler);

      setTimeout(() => {
        this.off('message', handler);
        reject(new Error('Load policy timeout'));
      }, 5000);
    });
  }

  /**
   * Reset learning for a game
   */
  resetLearning(gameType: string): void {
    this.send({
      type: 'reset_learning',
      gameType,
    });
    this.state.rewardHistory = [];
    this.state.metrics = this.getDefaultMetrics();
  }

  /**
   * Send user feedback
   */
  sendFeedback(behaviorId: string, rating: 'good' | 'bad' | 'neutral'): void {
    this.send({
      type: 'user_feedback',
      behaviorId,
      rating,
    });
  }

  /**
   * Get current learning state
   */
  getState(): LearningState {
    return { ...this.state };
  }

  /**
   * Get current metrics
   */
  getMetrics(): LearningMetrics {
    return { ...this.state.metrics };
  }

  /**
   * Subscribe to events
   */
  on(event: string, callback: (data: any) => void): void {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, new Set());
    }
    this.listeners.get(event)!.add(callback);
  }

  /**
   * Unsubscribe from events
   */
  off(event: string, callback: (data: any) => void): void {
    const listeners = this.listeners.get(event);
    if (listeners) {
      listeners.delete(callback);
    }
  }

  /**
   * Handle incoming messages
   */
  private handleMessage(data: string): void {
    try {
      const message = JSON.parse(data);

      switch (message.type) {
        case 'metrics_update':
          this.state.metrics = message.metrics;
          this.emit('metrics', message.metrics);
          break;

        case 'episode_complete':
          this.state.rewardHistory.push({
            episode: message.episode,
            reward: message.reward,
            outcome: message.outcome,
            timestamp: Date.now(),
          });

          // Keep only last 1000 episodes
          if (this.state.rewardHistory.length > 1000) {
            this.state.rewardHistory.shift();
          }

          this.emit('episode', message);
          break;

        case 'status_update':
          this.state.status = message.status;
          this.emit('status', message.status);
          break;

        case 'convergence':
          this.state.status = 'converged';
          this.emit('converged', message);
          break;

        default:
          this.emit('message', message);
      }
    } catch (error) {
      console.error('[LearningMonitor] Failed to parse message:', error);
    }
  }

  /**
   * Send message to learning system
   */
  private send(message: any): void {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    } else {
      console.warn('[LearningMonitor] WebSocket not connected');
    }
  }

  /**
   * Emit event to listeners
   */
  private emit(event: string, data: any): void {
    const listeners = this.listeners.get(event);
    if (listeners) {
      listeners.forEach((callback) => callback(data));
    }
  }

  /**
   * Default metrics
   */
  private getDefaultMetrics(): LearningMetrics {
    return {
      episodeCount: 0,
      averageReward: 0,
      winRate: 0,
      lossRate: 0,
      drawRate: 0,
      epsilonCurrent: 1.0,
      learningRateCurrent: 0.1,
      convergenceScore: 0,
    };
  }

  /**
   * Default config
   */
  private getDefaultConfig(): LearningConfig {
    return {
      learningRate: 0.1,
      discountFactor: 0.9,
      epsilonStart: 1.0,
      epsilonEnd: 0.1,
      epsilonDecay: 0.995,
      maxEpisodes: 1000,
    };
  }
}

/**
 * Create a learning monitor instance
 */
export function createLearningMonitor(wsUrl?: string): LearningMonitor {
  return new LearningMonitor(wsUrl);
}
