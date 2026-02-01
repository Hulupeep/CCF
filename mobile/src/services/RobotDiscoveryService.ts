/**
 * Robot Discovery Service
 * Issue: #88 (STORY-MOBILE-001)
 * Uses mDNS/network scanning to find robots
 */

import { Robot } from '../types';

export class RobotDiscoveryService {
  private discoveredRobots: Map<string, Robot> = new Map();
  private scanTimer: NodeJS.Timeout | null = null;

  /**
   * Discover robots on local network
   * Should complete within 10 seconds
   */
  async discoverRobots(): Promise<Robot[]> {
    console.log('[Discovery] Starting robot scan...');

    // Clear old robots
    this.discoveredRobots.clear();

    // In production, this would use mDNS (Bonjour/Zeroconf)
    // For now, we'll simulate network scanning
    return new Promise((resolve) => {
      // Simulate discovery delay
      setTimeout(() => {
        // Mock discovered robots
        const mockRobots: Robot[] = [
          {
            id: 'robot-001',
            name: 'mBot-Kitchen',
            ipAddress: '192.168.1.100',
            port: 8080,
            status: 'online',
            lastSeen: Date.now(),
          },
          {
            id: 'robot-002',
            name: 'mBot-Lab',
            ipAddress: '192.168.1.101',
            port: 8080,
            status: 'online',
            lastSeen: Date.now(),
          },
        ];

        mockRobots.forEach((robot) => {
          this.discoveredRobots.set(robot.id, robot);
        });

        console.log(`[Discovery] Found ${mockRobots.length} robots`);
        resolve(mockRobots);
      }, 2000); // Simulated discovery time
    });
  }

  /**
   * Start continuous scanning
   */
  startScanning(interval: number = 30000): void {
    this.stopScanning();

    this.scanTimer = setInterval(() => {
      this.discoverRobots().catch((error) => {
        console.error('[Discovery] Scan error:', error);
      });
    }, interval);
  }

  /**
   * Stop continuous scanning
   */
  stopScanning(): void {
    if (this.scanTimer) {
      clearInterval(this.scanTimer);
      this.scanTimer = null;
    }
  }

  /**
   * Get robot by ID
   */
  getRobot(robotId: string): Robot | undefined {
    return this.discoveredRobots.get(robotId);
  }

  /**
   * Get all discovered robots
   */
  getAllRobots(): Robot[] {
    return Array.from(this.discoveredRobots.values());
  }

  /**
   * Update robot status
   */
  updateRobotStatus(robotId: string, status: 'online' | 'offline'): void {
    const robot = this.discoveredRobots.get(robotId);
    if (robot) {
      robot.status = status;
      robot.lastSeen = Date.now();
      this.discoveredRobots.set(robotId, robot);
    }
  }
}

/**
 * Production implementation notes:
 *
 * For iOS:
 * - Use react-native-zeroconf or react-native-bonjour
 * - Service type: '_mbot._tcp'
 * - Requires NSBonjourServices in Info.plist
 *
 * For Android:
 * - Use react-native-zeroconf
 * - Requires INTERNET and ACCESS_WIFI_STATE permissions
 *
 * Example with react-native-zeroconf:
 *
 * import Zeroconf from 'react-native-zeroconf';
 *
 * const zeroconf = new Zeroconf();
 *
 * zeroconf.on('resolved', service => {
 *   const robot: Robot = {
 *     id: service.name,
 *     name: service.name,
 *     ipAddress: service.addresses[0],
 *     port: service.port,
 *     status: 'online',
 *     lastSeen: Date.now(),
 *   };
 *   this.discoveredRobots.set(robot.id, robot);
 * });
 *
 * zeroconf.scan('mbot', 'tcp', 'local.');
 */
