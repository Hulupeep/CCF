/**
 * Multi-Robot Coordination Service
 *
 * WebSocket-based coordination for 2-4 robots.
 * Implements invariants I-MULTI-001 through I-MULTI-006 from feature_multi_robot.yml
 */

// Contract: I-MULTI-004 - Maximum 4 robots
export const MAX_ROBOTS = 4;

// Contract: I-MULTI-001 - Discovery timeout 5 seconds
export const DISCOVERY_TIMEOUT_MS = 5000;

// Contract: I-MULTI-003 - Election timeout 3 seconds
export const ELECTION_TIMEOUT_MS = 3000;

// Contract: I-MULTI-005 - Sync interval 100ms
export const SYNC_INTERVAL_MS = 100;

// Contract: I-MULTI-002 - Heartbeat interval for consistency
export const HEARTBEAT_INTERVAL_MS = 1000;

// Contract: I-MULTI-006 - Disconnect detection timeout
export const DISCONNECT_TIMEOUT_MS = 3000;

/**
 * Message action types
 */
export type MessageAction = 'sync' | 'command' | 'state' | 'heartbeat' | 'election' | 'election_ack';

/**
 * Coordination message structure
 * Contract: I-MULTI-002 - Includes timestamp and sequence for ordering
 */
export interface CoordinationMessage {
  fromRobot: string;
  toRobots: string[];
  action: MessageAction;
  payload: MessagePayload;
  timestamp: number;
  sequence: number;
}

/**
 * Message payload variants
 */
export type MessagePayload =
  | { type: 'heartbeat' }
  | { type: 'state_update'; position: Position; status: RobotStatus }
  | { type: 'command'; commandType: string; params: number[] }
  | { type: 'election'; priority: number }
  | { type: 'election_ack'; electedId: string };

/**
 * 2D Position
 */
export interface Position {
  x: number;
  y: number;
}

/**
 * Robot status
 */
export type RobotStatus = 'idle' | 'moving' | 'executing';

/**
 * Robot role in coordination
 */
export type RobotRole = 'leader' | 'follower';

/**
 * Robot state
 * Contract: I-MULTI-002 - Includes sequence and lastHeartbeat for ordering and consistency
 */
export interface RobotState {
  id: string;
  role: RobotRole;
  position: Position;
  status: RobotStatus;
  lastHeartbeat: number;
  sequence: number;
}

/**
 * Coordination configuration
 */
export interface CoordinationConfig {
  maxRobots: number;
  heartbeatInterval: number;
  discoveryTimeout: number;
  syncInterval: number;
  electionTimeout: number;
}

/**
 * Default coordination configuration
 */
export const DEFAULT_CONFIG: CoordinationConfig = {
  maxRobots: MAX_ROBOTS,
  heartbeatInterval: HEARTBEAT_INTERVAL_MS,
  discoveryTimeout: DISCOVERY_TIMEOUT_MS,
  syncInterval: SYNC_INTERVAL_MS,
  electionTimeout: ELECTION_TIMEOUT_MS,
};

/**
 * Coordination error types
 */
export class CoordinationError extends Error {
  constructor(
    message: string,
    public code: 'TOO_MANY_ROBOTS' | 'ROBOT_NOT_FOUND' | 'ELECTION_TIMEOUT' | 'DISCOVERY_TIMEOUT' | 'INVALID_MESSAGE' | 'NETWORK_ERROR'
  ) {
    super(message);
    this.name = 'CoordinationError';
  }
}

/**
 * Event types for coordination events
 */
export type CoordinationEvent =
  | { type: 'robot_discovered'; robotId: string }
  | { type: 'robot_connected'; robotId: string }
  | { type: 'robot_disconnected'; robotId: string }
  | { type: 'leader_elected'; leaderId: string }
  | { type: 'state_synced'; robotId: string }
  | { type: 'message_received'; message: CoordinationMessage };

/**
 * Multi-Robot Coordination Manager
 * Contract: I-MULTI-002 - Maintains consistent state across robots
 */
export class MultiRobotCoordination {
  private localId: string;
  private robots: Map<string, RobotState>;
  private config: CoordinationConfig;
  private sequence: number;
  private role: RobotRole;
  private electionInProgress: boolean;
  private electionStartTime: number;
  private ws: WebSocket | null;
  private heartbeatTimer: NodeJS.Timeout | null;
  private syncTimer: NodeJS.Timeout | null;
  private disconnectCheckTimer: NodeJS.Timeout | null;
  private eventListeners: Array<(event: CoordinationEvent) => void>;

  constructor(localId: string, config: Partial<CoordinationConfig> = {}) {
    this.localId = localId;
    this.robots = new Map();
    this.config = { ...DEFAULT_CONFIG, ...config };
    this.sequence = 0;
    this.role = 'follower';
    this.electionInProgress = false;
    this.electionStartTime = 0;
    this.ws = null;
    this.heartbeatTimer = null;
    this.syncTimer = null;
    this.disconnectCheckTimer = null;
    this.eventListeners = [];

    // Add self to robots
    this.robots.set(localId, {
      id: localId,
      role: 'follower',
      position: { x: 0, y: 0 },
      status: 'idle',
      lastHeartbeat: Date.now(),
      sequence: 0,
    });
  }

  /**
   * Connect to coordination server
   */
  async connect(wsUrl: string): Promise<void> {
    return new Promise((resolve, reject) => {
      this.ws = new WebSocket(wsUrl);

      this.ws.onopen = () => {
        console.log('[MultiRobot] Connected to coordination server');
        this.startHeartbeat();
        this.startSyncLoop();
        this.startDisconnectCheck();
        resolve();
      };

      this.ws.onerror = (error) => {
        console.error('[MultiRobot] WebSocket error:', error);
        reject(new CoordinationError('Failed to connect', 'NETWORK_ERROR'));
      };

      this.ws.onclose = () => {
        console.log('[MultiRobot] Disconnected from coordination server');
        this.stopTimers();
      };

      this.ws.onmessage = (event) => {
        try {
          const message: CoordinationMessage = JSON.parse(event.data);
          this.handleMessage(message);
        } catch (error) {
          console.error('[MultiRobot] Failed to parse message:', error);
        }
      };
    });
  }

  /**
   * Disconnect from coordination
   */
  disconnect(): void {
    this.stopTimers();
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  /**
   * Add event listener
   */
  addEventListener(listener: (event: CoordinationEvent) => void): void {
    this.eventListeners.push(listener);
  }

  /**
   * Remove event listener
   */
  removeEventListener(listener: (event: CoordinationEvent) => void): void {
    this.eventListeners = this.eventListeners.filter(l => l !== listener);
  }

  /**
   * Emit event to all listeners
   */
  private emit(event: CoordinationEvent): void {
    this.eventListeners.forEach(listener => listener(event));
  }

  /**
   * Add a discovered robot to the mesh
   * Contract: I-MULTI-004 - Enforces max 4 robots
   */
  addRobot(robotId: string): void {
    if (this.robots.size >= this.config.maxRobots) {
      throw new CoordinationError(
        `Maximum ${this.config.maxRobots} robots allowed`,
        'TOO_MANY_ROBOTS'
      );
    }

    if (!this.robots.has(robotId)) {
      this.robots.set(robotId, {
        id: robotId,
        role: 'follower',
        position: { x: 0, y: 0 },
        status: 'idle',
        lastHeartbeat: Date.now(),
        sequence: 0,
      });

      this.emit({ type: 'robot_discovered', robotId });
    }
  }

  /**
   * Remove a robot from the mesh
   * Contract: I-MULTI-006 - Handle disconnects gracefully
   */
  removeRobot(robotId: string): void {
    const robot = this.robots.get(robotId);
    if (!robot) return;

    const wasLeader = robot.role === 'leader';
    this.robots.delete(robotId);

    this.emit({ type: 'robot_disconnected', robotId });

    // If leader disconnected, trigger election
    if (wasLeader) {
      this.startElection();
    }
  }

  /**
   * Update robot state
   * Contract: I-MULTI-002 - State changes use sequence numbers
   */
  updateRobotState(
    robotId: string,
    position: Position,
    status: RobotStatus,
    sequence: number,
    timestamp: number
  ): void {
    const robot = this.robots.get(robotId);
    if (!robot) {
      throw new CoordinationError(`Robot ${robotId} not found`, 'ROBOT_NOT_FOUND');
    }

    // Only update if sequence is newer (prevents out-of-order updates)
    if (sequence > robot.sequence) {
      robot.position = position;
      robot.status = status;
      robot.sequence = sequence;
      robot.lastHeartbeat = timestamp;

      this.emit({ type: 'state_synced', robotId });
    }
  }

  /**
   * Process heartbeat
   * Contract: I-MULTI-002 - Heartbeat maintains consistency
   */
  processHeartbeat(robotId: string, timestamp: number): void {
    const robot = this.robots.get(robotId);
    if (!robot) return;

    robot.lastHeartbeat = timestamp;
  }

  /**
   * Check for disconnected robots
   * Contract: I-MULTI-006 - Detect disconnects via timeout
   */
  private detectDisconnects(): void {
    const currentTime = Date.now();
    const disconnected: string[] = [];

    this.robots.forEach((robot, robotId) => {
      if (robotId !== this.localId && this.isDisconnected(robot, currentTime)) {
        disconnected.push(robotId);
      }
    });

    // Remove disconnected robots
    disconnected.forEach(robotId => this.removeRobot(robotId));
  }

  /**
   * Check if robot is disconnected
   */
  private isDisconnected(robot: RobotState, currentTime: number): boolean {
    return currentTime - robot.lastHeartbeat > DISCONNECT_TIMEOUT_MS;
  }

  /**
   * Start leader election
   * Contract: I-MULTI-003 - Bully algorithm, completes in 3s
   */
  startElection(): void {
    console.log('[MultiRobot] Starting leader election');
    this.electionInProgress = true;
    this.electionStartTime = Date.now();

    // Broadcast election message
    this.sendMessage({
      fromRobot: this.localId,
      toRobots: [], // Broadcast
      action: 'election',
      payload: { type: 'election', priority: this.getPriority() },
      timestamp: Date.now(),
      sequence: this.nextSequence(),
    });
  }

  /**
   * Get election priority (based on robot ID)
   * Contract: I-MULTI-003 - Deterministic election
   */
  private getPriority(): number {
    // Use robot ID hash as priority
    let hash = 0;
    for (let i = 0; i < this.localId.length; i++) {
      hash = ((hash << 5) - hash) + this.localId.charCodeAt(i);
      hash = hash & hash; // Convert to 32bit integer
    }
    return Math.abs(hash);
  }

  /**
   * Process election message
   * Contract: I-MULTI-003 - Bully algorithm
   */
  private processElection(fromRobot: string, priority: number): void {
    const myPriority = this.getPriority();

    if (priority > myPriority) {
      // Higher priority robot found, acknowledge it
      this.sendMessage({
        fromRobot: this.localId,
        toRobots: [fromRobot],
        action: 'election_ack',
        payload: { type: 'election_ack', electedId: fromRobot },
        timestamp: Date.now(),
        sequence: this.nextSequence(),
      });
    }
    // If I have higher priority, ignore and continue election
  }

  /**
   * Check if election has timed out
   * Contract: I-MULTI-003 - Election completes within 3 seconds
   */
  private checkElectionTimeout(): void {
    if (!this.electionInProgress) return;

    const elapsed = Date.now() - this.electionStartTime;
    if (elapsed >= this.config.electionTimeout) {
      // Election timeout - become leader
      console.log('[MultiRobot] Election timeout - becoming leader');
      this.becomeLeader();
    }
  }

  /**
   * Become the leader
   */
  private becomeLeader(): void {
    this.role = 'leader';
    this.electionInProgress = false;

    const robot = this.robots.get(this.localId);
    if (robot) {
      robot.role = 'leader';
    }

    this.emit({ type: 'leader_elected', leaderId: this.localId });
  }

  /**
   * Handle incoming message
   */
  private handleMessage(message: CoordinationMessage): void {
    this.emit({ type: 'message_received', message });

    switch (message.action) {
      case 'heartbeat':
        this.processHeartbeat(message.fromRobot, message.timestamp);
        break;

      case 'state':
        if (message.payload.type === 'state_update') {
          this.updateRobotState(
            message.fromRobot,
            message.payload.position,
            message.payload.status,
            message.sequence,
            message.timestamp
          );
        }
        break;

      case 'election':
        if (message.payload.type === 'election') {
          this.processElection(message.fromRobot, message.payload.priority);
        }
        break;

      case 'election_ack':
        if (message.payload.type === 'election_ack') {
          // Someone acknowledged a higher priority robot
          this.electionInProgress = false;
        }
        break;

      case 'command':
        // Handle coordinated command
        console.log('[MultiRobot] Received command:', message.payload);
        break;

      default:
        console.warn('[MultiRobot] Unknown message action:', message.action);
    }
  }

  /**
   * Send message via WebSocket
   */
  private sendMessage(message: CoordinationMessage): void {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
      console.warn('[MultiRobot] WebSocket not connected');
      return;
    }

    this.ws.send(JSON.stringify(message));
  }

  /**
   * Start heartbeat loop
   * Contract: I-MULTI-002 - Regular heartbeats for consistency
   */
  private startHeartbeat(): void {
    this.heartbeatTimer = setInterval(() => {
      this.sendMessage({
        fromRobot: this.localId,
        toRobots: [], // Broadcast
        action: 'heartbeat',
        payload: { type: 'heartbeat' },
        timestamp: Date.now(),
        sequence: this.nextSequence(),
      });

      // Update own heartbeat
      const robot = this.robots.get(this.localId);
      if (robot) {
        robot.lastHeartbeat = Date.now();
      }
    }, this.config.heartbeatInterval);
  }

  /**
   * Start state sync loop
   * Contract: I-MULTI-005 - Sync every 100ms
   */
  private startSyncLoop(): void {
    this.syncTimer = setInterval(() => {
      const robot = this.robots.get(this.localId);
      if (!robot) return;

      this.sendMessage({
        fromRobot: this.localId,
        toRobots: [], // Broadcast
        action: 'state',
        payload: {
          type: 'state_update',
          position: robot.position,
          status: robot.status,
        },
        timestamp: Date.now(),
        sequence: this.nextSequence(),
      });
    }, this.config.syncInterval);
  }

  /**
   * Start disconnect detection loop
   * Contract: I-MULTI-006 - Check for disconnects
   */
  private startDisconnectCheck(): void {
    this.disconnectCheckTimer = setInterval(() => {
      this.detectDisconnects();
      this.checkElectionTimeout();
    }, 1000);
  }

  /**
   * Stop all timers
   */
  private stopTimers(): void {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer);
      this.heartbeatTimer = null;
    }
    if (this.syncTimer) {
      clearInterval(this.syncTimer);
      this.syncTimer = null;
    }
    if (this.disconnectCheckTimer) {
      clearInterval(this.disconnectCheckTimer);
      this.disconnectCheckTimer = null;
    }
  }

  /**
   * Get next sequence number
   * Contract: I-MULTI-002 - Sequence numbers for ordering
   */
  private nextSequence(): number {
    this.sequence = (this.sequence + 1) & 0xFFFFFFFF; // Keep as 32-bit
    return this.sequence;
  }

  /**
   * Get local robot state
   */
  getLocalState(): RobotState | undefined {
    return this.robots.get(this.localId);
  }

  /**
   * Update local position
   */
  updateLocalPosition(position: Position): void {
    const robot = this.robots.get(this.localId);
    if (robot) {
      robot.position = position;
      robot.sequence = this.nextSequence();
    }
  }

  /**
   * Update local status
   */
  updateLocalStatus(status: RobotStatus): void {
    const robot = this.robots.get(this.localId);
    if (robot) {
      robot.status = status;
      robot.sequence = this.nextSequence();
    }
  }

  /**
   * Get current leader
   */
  getLeader(): RobotState | undefined {
    for (const robot of this.robots.values()) {
      if (robot.role === 'leader') {
        return robot;
      }
    }
    return undefined;
  }

  /**
   * Check if this robot is the leader
   */
  isLeader(): boolean {
    return this.role === 'leader';
  }

  /**
   * Get all connected robots
   */
  getConnectedRobots(): RobotState[] {
    return Array.from(this.robots.values());
  }

  /**
   * Get robot count
   */
  getRobotCount(): number {
    return this.robots.size;
  }

  /**
   * Send coordinated command to all robots
   */
  sendCoordinatedCommand(commandType: string, params: number[]): void {
    if (!this.isLeader()) {
      console.warn('[MultiRobot] Only leader can send coordinated commands');
      return;
    }

    this.sendMessage({
      fromRobot: this.localId,
      toRobots: [], // Broadcast
      action: 'command',
      payload: { type: 'command', commandType, params },
      timestamp: Date.now(),
      sequence: this.nextSequence(),
    });
  }
}
