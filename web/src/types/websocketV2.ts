/**
 * WebSocket Protocol V2 Type Definitions
 * Contract: ARCH-005 (Transport Layer Abstraction)
 *
 * Invariants:
 * - I-WS-V2-001: State Consistency - Client state matches robot after sync
 * - I-WS-V2-002: Message Order - Messages processed in order sent
 */

import { PersonalityConfig } from './personality';
import { NeuralState } from './neural';
import { Station } from './inventory';

/**
 * Protocol version
 */
export const PROTOCOL_VERSION = 2;

/**
 * Message batching window (100ms per requirements)
 */
export const BATCH_WINDOW_MS = 100;

/**
 * Auto-reconnect timeout (30s per requirements)
 */
export const RECONNECT_TIMEOUT_MS = 30000;

/**
 * WebSocket V2 message types
 */
export type MessageType = 'state' | 'command' | 'event' | 'batch' | 'ping' | 'pong';

/**
 * Core WebSocket V2 message structure
 */
export interface WebSocketMessage {
  /** Protocol version (always 2) */
  version: number;

  /** Message type */
  type: MessageType;

  /** Message payload */
  payload: any;

  /** Message timestamp (milliseconds since epoch) */
  timestamp: number;

  /** Optional sequence number for ordering (I-WS-V2-002) */
  sequence?: number;
}

/**
 * Full state snapshot sent on connect (I-WS-V2-001)
 */
export interface StateSnapshot {
  /** Personality configuration */
  personality: PersonalityState;

  /** Neural system state */
  neural_state: NeuralStateData;

  /** Inventory state (if available) */
  inventory?: InventoryState;

  /** Current game state (if in game) */
  game_state?: GameState;

  /** Robot capabilities and features */
  capabilities: RobotCapabilities;
}

/**
 * Personality state within snapshot
 */
export interface PersonalityState extends PersonalityConfig {}

/**
 * Neural state data within snapshot
 */
export interface NeuralStateData {
  mode: 'Calm' | 'Active' | 'Spike' | 'Protect';
  tension: number;
  coherence: number;
  energy: number;
  curiosity: number;
  distance?: number;
  gyro?: number;
  sound?: number;
  light?: number;
}

/**
 * Inventory state within snapshot
 */
export interface InventoryState {
  red: number;
  green: number;
  blue: number;
  yellow: number;
  last_updated: number;
}

/**
 * Game state within snapshot
 */
export interface GameState {
  game_type: 'tictactoe' | 'memory' | 'simon';
  state: any;
}

/**
 * Robot capabilities
 */
export interface RobotCapabilities {
  has_drawing: boolean;
  has_sorter: boolean;
  has_games: boolean;
  has_learning_lab: boolean;
  firmware_version: string;
}

/**
 * Command message from client to robot
 */
export interface CommandMessage {
  command: string;
  params: Record<string, any>;
}

/**
 * Event message from robot to client
 */
export interface EventMessage {
  event: string;
  data: any;
}

/**
 * Connection state
 */
export type ConnectionState =
  | 'disconnected'
  | 'connecting'
  | 'connected'
  | 'reconnecting'
  | 'error';

/**
 * Connection statistics
 */
export interface ConnectionStats {
  /** Total messages sent */
  messagesSent: number;

  /** Total messages received */
  messagesReceived: number;

  /** Current round-trip time (ms) */
  latency: number;

  /** Number of reconnection attempts */
  reconnectAttempts: number;

  /** Time of last successful connection */
  lastConnected: number | null;

  /** Time of last disconnect */
  lastDisconnected: number | null;
}

/**
 * WebSocket V2 hook options
 */
export interface UseWebSocketV2Options {
  /** WebSocket URL (default: ws://localhost:8080) */
  url?: string;

  /** Auto-connect on mount (default: true) */
  autoConnect?: boolean;

  /** Message batching window in ms (default: 100) */
  batchWindow?: number;

  /** Max reconnection attempts (default: Infinity) */
  maxReconnectAttempts?: number;

  /** Reconnection delay in ms (default: 1000) */
  reconnectDelay?: number;

  /** Enable debug logging (default: false) */
  debug?: boolean;
}

/**
 * WebSocket V2 hook return value
 */
export interface UseWebSocketV2Return {
  /** Current connection state */
  connectionState: ConnectionState;

  /** Full state snapshot (I-WS-V2-001) */
  state: StateSnapshot | null;

  /** Connection statistics */
  stats: ConnectionStats;

  /** Send a command to the robot */
  sendCommand: (command: string, params: Record<string, any>) => void;

  /** Manually trigger reconnection */
  reconnect: () => void;

  /** Manually disconnect */
  disconnect: () => void;

  /** Subscribe to specific events */
  on: <T = any>(event: string, handler: (data: T) => void) => () => void;
}

/**
 * Validate protocol version
 */
export function validateVersion(message: WebSocketMessage): boolean {
  return message.version === PROTOCOL_VERSION;
}

/**
 * Create a command message
 */
export function createCommand(command: string, params: Record<string, any>): WebSocketMessage {
  return {
    version: PROTOCOL_VERSION,
    type: 'command',
    payload: { command, params },
    timestamp: Date.now(),
  };
}

/**
 * Create a ping message
 */
export function createPing(): WebSocketMessage {
  return {
    version: PROTOCOL_VERSION,
    type: 'ping',
    payload: {},
    timestamp: Date.now(),
  };
}

/**
 * Check if message is a state snapshot
 */
export function isStateMessage(message: WebSocketMessage): message is WebSocketMessage & {
  payload: StateSnapshot;
} {
  return message.type === 'state';
}

/**
 * Check if message is an event
 */
export function isEventMessage(message: WebSocketMessage): message is WebSocketMessage & {
  payload: EventMessage;
} {
  return message.type === 'event';
}

/**
 * Check if message is a pong
 */
export function isPongMessage(message: WebSocketMessage): boolean {
  return message.type === 'pong';
}
