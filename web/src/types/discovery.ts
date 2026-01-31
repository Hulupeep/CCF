/**
 * Multi-Robot Discovery Types
 * Issue #77 - STORY-ARCH-008
 *
 * Implements:
 * - I-DISC-001: mDNS standard protocol (RFC 6762)
 * - TypeScript interfaces for robot discovery
 * - Health status tracking
 */

/**
 * Discovered robot interface as per contract
 */
export interface DiscoveredRobot {
  id: string;
  name: string;
  ipAddress: string;
  port: number;
  version: string;
}

/**
 * Robot with connection state
 */
export interface RobotWithState extends DiscoveredRobot {
  status: RobotStatus;
  lastSeen: number;
  metadata?: RobotMetadata;
}

/**
 * Robot health status
 */
export type RobotStatus = 'connected' | 'disconnected' | 'error' | 'discovering';

/**
 * Optional robot metadata from mDNS TXT records
 */
export interface RobotMetadata {
  model?: string;
  firmware?: string;
  capabilities?: string[];
  uptime?: number;
}

/**
 * Discovery service configuration
 */
export interface DiscoveryConfig {
  serviceName?: string;
  timeout?: number;
  pollingInterval?: number;
}

/**
 * mDNS service discovery result
 */
export interface MDNSServiceInfo {
  name: string;
  type: string;
  domain: string;
  host: string;
  port: number;
  addresses: string[];
  txt?: Record<string, string>;
}

/**
 * Discovery events
 */
export type DiscoveryEvent =
  | { type: 'robot_discovered'; robot: DiscoveredRobot }
  | { type: 'robot_lost'; robotId: string }
  | { type: 'robot_updated'; robot: DiscoveredRobot }
  | { type: 'error'; error: Error };

/**
 * Discovery service interface
 */
export interface IDiscoveryService {
  start(): Promise<void>;
  stop(): Promise<void>;
  getRobots(): RobotWithState[];
  subscribe(callback: (event: DiscoveryEvent) => void): () => void;
}
