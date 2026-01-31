/**
 * Neural State Types
 * Data contract for neural visualizer
 */

export type NeuralMode = 'Calm' | 'Active' | 'Spike' | 'Protect';

export interface NeuralState {
  timestamp: number;
  mode: NeuralMode;
  tension: number;
  coherence: number;
  energy: number;
  curiosity: number;
  distance?: number;
  gyro?: number;
  sound?: number;
  light?: number;
}

export interface TimelineState {
  history: NeuralState[];
  maxDuration: number; // milliseconds
  currentTime: number;
  isPaused: boolean;
  playbackSpeed: 1 | 0.5 | 2;
}

export interface VisualizerConfig {
  updateRate: number; // Hz
  maxHistory: number; // seconds
  showGrid: boolean;
  showLabels: boolean;
  colorScheme: 'default' | 'high-contrast';
}
