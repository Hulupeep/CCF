/**
 * Multi-Robot Discovery Service
 * Issue #77 - STORY-ARCH-008
 *
 * Implements:
 * - I-DISC-001: mDNS-based discovery (RFC 6762)
 * - Robot health monitoring
 * - Event-based robot updates
 *
 * NOTE: Browser-based mDNS is limited. This implementation provides:
 * - WebSocket-based discovery protocol for browser clients
 * - Server-side mDNS bridge (to be implemented in backend)
 * - Mock discovery for development/testing
 */

import {
  DiscoveredRobot,
  RobotWithState,
  RobotStatus,
  DiscoveryEvent,
  DiscoveryConfig,
  IDiscoveryService,
} from '../types/discovery';

const DEFAULT_CONFIG: Required<DiscoveryConfig> = {
  serviceName: '_mbot._tcp.local',
  timeout: 5000,
  pollingInterval: 3000,
};

/**
 * Robot Discovery Service
 *
 * Browser Implementation Note:
 * - Browsers cannot directly perform mDNS queries (security restriction)
 * - This service connects to a discovery WebSocket endpoint
 * - Backend server performs actual mDNS discovery
 * - Fallback to HTTP polling if WebSocket unavailable
 */
export class RobotDiscoveryService implements IDiscoveryService {
  private config: Required<DiscoveryConfig>;
  private robots: Map<string, RobotWithState> = new Map();
  private subscribers: Set<(event: DiscoveryEvent) => void> = new Set();
  private discoveryWs: WebSocket | null = null;
  private pollingInterval: NodeJS.Timeout | null = null;
  private isRunning = false;

  constructor(config: DiscoveryConfig = {}) {
    this.config = { ...DEFAULT_CONFIG, ...config };
  }

  /**
   * Start discovery service
   */
  async start(): Promise<void> {
    if (this.isRunning) {
      return;
    }

    this.isRunning = true;

    // Try WebSocket-based discovery first
    try {
      await this.startWebSocketDiscovery();
    } catch (error) {
      console.warn('WebSocket discovery failed, falling back to HTTP polling', error);
      this.startHttpPolling();
    }
  }

  /**
   * Stop discovery service
   */
  async stop(): Promise<void> {
    this.isRunning = false;

    if (this.discoveryWs) {
      this.discoveryWs.close();
      this.discoveryWs = null;
    }

    if (this.pollingInterval) {
      clearInterval(this.pollingInterval);
      this.pollingInterval = null;
    }

    this.robots.clear();
  }

  /**
   * Get all discovered robots
   */
  getRobots(): RobotWithState[] {
    return Array.from(this.robots.values());
  }

  /**
   * Subscribe to discovery events
   */
  subscribe(callback: (event: DiscoveryEvent) => void): () => void {
    this.subscribers.add(callback);
    return () => this.subscribers.delete(callback);
  }

  /**
   * WebSocket-based discovery (preferred method)
   * Connects to backend mDNS bridge
   */
  private async startWebSocketDiscovery(): Promise<void> {
    return new Promise((resolve, reject) => {
      const ws = new WebSocket('ws://localhost:8081/discovery');

      ws.onopen = () => {
        console.log('[Discovery] WebSocket connected');
        this.discoveryWs = ws;
        resolve();
      };

      ws.onerror = (error) => {
        console.error('[Discovery] WebSocket error:', error);
        reject(error);
      };

      ws.onmessage = (event) => {
        try {
          const message = JSON.parse(event.data);
          this.handleDiscoveryMessage(message);
        } catch (error) {
          console.error('[Discovery] Failed to parse message:', error);
        }
      };

      ws.onclose = () => {
        console.log('[Discovery] WebSocket closed');
        this.discoveryWs = null;

        // Try to reconnect if still running
        if (this.isRunning) {
          setTimeout(() => {
            if (this.isRunning) {
              this.startWebSocketDiscovery().catch(() => {
                this.startHttpPolling();
              });
            }
          }, 3000);
        }
      };

      // Timeout for connection
      setTimeout(() => {
        if (!this.discoveryWs) {
          ws.close();
          reject(new Error('WebSocket connection timeout'));
        }
      }, this.config.timeout);
    });
  }

  /**
   * HTTP polling fallback
   */
  private startHttpPolling(): void {
    const poll = async () => {
      try {
        const response = await fetch('http://localhost:8081/api/discovery/robots');
        if (!response.ok) {
          throw new Error(`HTTP ${response.status}`);
        }

        const robots: DiscoveredRobot[] = await response.json();
        this.updateRobotsList(robots);
      } catch (error) {
        console.error('[Discovery] HTTP polling error:', error);
        this.emit({ type: 'error', error: error as Error });
      }
    };

    // Initial poll
    poll();

    // Set up interval
    this.pollingInterval = setInterval(poll, this.config.pollingInterval);
  }

  /**
   * Handle discovery message from WebSocket
   */
  private handleDiscoveryMessage(message: any): void {
    switch (message.type) {
      case 'robot_list':
        this.updateRobotsList(message.robots);
        break;

      case 'robot_discovered':
        this.addOrUpdateRobot(message.robot);
        this.emit({ type: 'robot_discovered', robot: message.robot });
        break;

      case 'robot_lost':
        this.removeRobot(message.robotId);
        this.emit({ type: 'robot_lost', robotId: message.robotId });
        break;

      case 'robot_updated':
        this.addOrUpdateRobot(message.robot);
        this.emit({ type: 'robot_updated', robot: message.robot });
        break;

      default:
        console.warn('[Discovery] Unknown message type:', message.type);
    }
  }

  /**
   * Update full robots list (from polling or initial WebSocket message)
   */
  private updateRobotsList(robots: DiscoveredRobot[]): void {
    const currentIds = new Set(this.robots.keys());
    const newIds = new Set<string>();

    // Add or update robots
    robots.forEach(robot => {
      newIds.add(robot.id);
      this.addOrUpdateRobot(robot);
    });

    // Remove robots that are no longer present
    currentIds.forEach(id => {
      if (!newIds.has(id)) {
        this.removeRobot(id);
      }
    });
  }

  /**
   * Add or update a robot
   */
  private addOrUpdateRobot(robot: DiscoveredRobot): void {
    const existing = this.robots.get(robot.id);

    const robotWithState: RobotWithState = {
      ...robot,
      status: existing?.status || 'disconnected',
      lastSeen: Date.now(),
      metadata: existing?.metadata,
    };

    this.robots.set(robot.id, robotWithState);
  }

  /**
   * Remove a robot
   */
  private removeRobot(robotId: string): void {
    const robot = this.robots.get(robotId);
    if (robot) {
      robot.status = 'disconnected';
      // Keep for 30 seconds before removing
      setTimeout(() => {
        this.robots.delete(robotId);
      }, 30000);
    }
  }

  /**
   * Emit event to subscribers
   */
  private emit(event: DiscoveryEvent): void {
    this.subscribers.forEach(callback => {
      try {
        callback(event);
      } catch (error) {
        console.error('[Discovery] Subscriber error:', error);
      }
    });
  }

  /**
   * Update robot connection status
   */
  updateRobotStatus(robotId: string, status: RobotStatus): void {
    const robot = this.robots.get(robotId);
    if (robot) {
      robot.status = status;
      robot.lastSeen = Date.now();
    }
  }
}

/**
 * Mock discovery service for development
 */
export class MockDiscoveryService implements IDiscoveryService {
  private robots: RobotWithState[] = [
    {
      id: 'mbot-001',
      name: 'mBot Alpha',
      ipAddress: '192.168.1.100',
      port: 8081,
      version: '1.0.0',
      status: 'disconnected',
      lastSeen: Date.now(),
      metadata: {
        model: 'mBot2',
        firmware: '2.1.0',
        capabilities: ['drawing', 'personality', 'games'],
      },
    },
    {
      id: 'mbot-002',
      name: 'mBot Beta',
      ipAddress: '192.168.1.101',
      port: 8081,
      version: '1.0.0',
      status: 'disconnected',
      lastSeen: Date.now(),
      metadata: {
        model: 'mBot2',
        firmware: '2.1.0',
        capabilities: ['drawing', 'sorting'],
      },
    },
    {
      id: 'mbot-003',
      name: 'mBot Gamma',
      ipAddress: '192.168.1.102',
      port: 8081,
      version: '0.9.5',
      status: 'disconnected',
      lastSeen: Date.now(),
      metadata: {
        model: 'mBot2',
        firmware: '2.0.5',
        capabilities: ['personality', 'games'],
      },
    },
  ];

  private subscribers: Set<(event: DiscoveryEvent) => void> = new Set();

  async start(): Promise<void> {
    console.log('[MockDiscovery] Started with', this.robots.length, 'robots');
  }

  async stop(): Promise<void> {
    console.log('[MockDiscovery] Stopped');
  }

  getRobots(): RobotWithState[] {
    return [...this.robots];
  }

  subscribe(callback: (event: DiscoveryEvent) => void): () => void {
    this.subscribers.add(callback);
    return () => this.subscribers.delete(callback);
  }

  // Helper for testing: simulate robot discovery
  simulateDiscovery(robot: DiscoveredRobot): void {
    const robotWithState: RobotWithState = {
      ...robot,
      status: 'disconnected',
      lastSeen: Date.now(),
    };
    this.robots.push(robotWithState);
    this.emit({ type: 'robot_discovered', robot });
  }

  private emit(event: DiscoveryEvent): void {
    this.subscribers.forEach(callback => {
      try {
        callback(event);
      } catch (error) {
        console.error('[MockDiscovery] Subscriber error:', error);
      }
    });
  }
}
