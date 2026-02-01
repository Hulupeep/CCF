/**
 * mBot Mobile App - Type Definitions
 * Issue: #88 (STORY-MOBILE-001)
 * Invariants: I-MOBILE-001, I-MOBILE-002, I-MOBILE-003
 */

// Robot Discovery
export interface Robot {
  id: string;
  name: string;
  ipAddress: string;
  port: number;
  status: 'online' | 'offline';
  lastSeen: number;
}

// Neural State
export interface NeuralState {
  timestamp: number;
  activeNodes: NeuralNode[];
  connections: Connection[];
  activity: number;
}

export interface NeuralNode {
  id: string;
  type: 'sensory' | 'motor' | 'cognitive';
  position: { x: number; y: number; z: number };
  activation: number;
}

export interface Connection {
  from: string;
  to: string;
  weight: number;
  active: boolean;
}

// Personality Configuration
export interface PersonalityConfig {
  energy: number;        // 0-1
  tension: number;       // 0-1
  curiosity: number;     // 0-1
  playfulness: number;   // 0-1
  confidence: number;    // 0-1
  focus: number;         // 0-1
  empathy: number;       // 0-1
  creativity: number;    // 0-1
  persistence: number;   // 0-1
}

// Drawing
export interface Drawing {
  id: string;
  name: string;
  timestamp: number;
  thumbnailUrl?: string;
  strokes: DrawingStroke[];
  duration: number;
}

export interface DrawingStroke {
  points: { x: number; y: number }[];
  timestamp: number;
}

// App State
export interface AppState {
  connected: boolean;
  currentRobot?: Robot;
  neuralState?: NeuralState;
  personalityConfig?: PersonalityConfig;
  cachedDrawings: Drawing[];
  lastSync: number;
}

// App Settings
export interface AppSettings {
  autoReconnect: boolean;
  reconnectDelay: number;      // ms (I-MOBILE-001: must be ≤ 5000)
  cacheExpiry: number;         // hours (I-MOBILE-003: must be ≥ 24)
  notificationsEnabled: boolean;
  theme: 'light' | 'dark';
}

// WebSocket Messages
export type WebSocketMessage =
  | { type: 'neural_state'; payload: NeuralState }
  | { type: 'personality_update'; payload: PersonalityConfig }
  | { type: 'drawing_complete'; payload: Drawing }
  | { type: 'status_change'; payload: { status: string } }
  | { type: 'error'; payload: { message: string } };

// Service Interfaces
export interface MobileAppService {
  // Discovery
  discoverRobots(): Promise<Robot[]>;
  connectToRobot(robotId: string): Promise<void>;
  disconnectFromRobot(): Promise<void>;

  // Real-time state
  subscribeToNeuralState(callback: (state: NeuralState) => void): Subscription;
  updatePersonality(config: PersonalityConfig): Promise<void>;

  // Gallery
  fetchDrawings(): Promise<Drawing[]>;
  downloadDrawing(id: string): Promise<Drawing>;

  // Settings
  getSettings(): AppSettings;
  updateSettings(settings: Partial<AppSettings>): void;
}

export interface Subscription {
  unsubscribe(): void;
}

// Screen Props
export interface DiscoveryScreenProps {
  robots: Robot[];
  scanning: boolean;
  error?: string;
  onStartScan: () => void;
  onConnect: (robotId: string) => void;
}

export interface PersonalityMixerScreenProps {
  config: PersonalityConfig;
  connected: boolean;
  onUpdateSlider: (param: keyof PersonalityConfig, value: number) => void;
  onReset: () => void;
  onSavePreset: (name: string) => void;
}

export interface NeuralVisualizerScreenProps {
  neuralState: NeuralState;
  connected: boolean;
  onToggleAnimation: () => void;
  onZoom: (factor: number) => void;
}

export interface GalleryScreenProps {
  drawings: Drawing[];
  loading: boolean;
  onFetchDrawings: () => void;
  onOpenDrawing: (id: string) => void;
  onDeleteDrawing: (id: string) => void;
}
