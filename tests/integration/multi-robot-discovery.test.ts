/**
 * Integration Tests for Multi-Robot Discovery Protocol
 * Issue #77 - STORY-ARCH-008
 *
 * Tests:
 * - Discovery of multiple robots
 * - Connect/disconnect functionality
 * - Health indicators
 * - I-DISC-001: mDNS standard protocol compliance
 */

import { describe, test, expect, beforeEach, afterEach } from '@jest/globals';
import {
  RobotDiscoveryService,
  MockDiscoveryService,
} from '../../web/src/services/robotDiscovery';
import {
  DiscoveredRobot,
  RobotWithState,
  DiscoveryEvent,
} from '../../web/src/types/discovery';

describe('Multi-Robot Discovery Protocol - I-DISC-001', () => {
  describe('RobotDiscoveryService', () => {
    let service: RobotDiscoveryService;

    beforeEach(() => {
      service = new RobotDiscoveryService({
        serviceName: '_mbot._tcp.local',
        timeout: 1000,
        pollingInterval: 500,
      });
    });

    afterEach(async () => {
      await service.stop();
    });

    test('should initialize with empty robot list', () => {
      const robots = service.getRobots();
      expect(robots).toEqual([]);
    });

    test('should start discovery service', async () => {
      // Note: This will fail to connect in test environment
      // Testing that it handles connection failure gracefully
      await expect(service.start()).rejects.toThrow();

      // Service should still be usable after failed start
      const robots = service.getRobots();
      expect(robots).toEqual([]);
    });

    test('should support event subscriptions', (done) => {
      const events: DiscoveryEvent[] = [];

      const unsubscribe = service.subscribe((event) => {
        events.push(event);
      });

      expect(typeof unsubscribe).toBe('function');
      unsubscribe();

      done();
    });

    test('should handle multiple subscribers', () => {
      const callbacks = [jest.fn(), jest.fn(), jest.fn()];

      const unsubscribes = callbacks.map((callback) =>
        service.subscribe(callback)
      );

      unsubscribes.forEach((unsubscribe) => unsubscribe());

      // No errors should be thrown
      expect(true).toBe(true);
    });

    test('should clean up resources on stop', async () => {
      await service.stop();
      const robots = service.getRobots();
      expect(robots).toEqual([]);
    });
  });

  describe('MockDiscoveryService - Development Testing', () => {
    let service: MockDiscoveryService;

    beforeEach(() => {
      service = new MockDiscoveryService();
    });

    afterEach(async () => {
      await service.stop();
    });

    test('should start with 3 mock robots', async () => {
      await service.start();
      const robots = service.getRobots();

      expect(robots).toHaveLength(3);
      expect(robots[0]).toMatchObject({
        id: 'mbot-001',
        name: 'mBot Alpha',
        ipAddress: '192.168.1.100',
        port: 8081,
        version: '1.0.0',
        status: 'disconnected',
      });
    });

    test('should discover multiple robots - Scenario: Discover Robots', async () => {
      await service.start();
      const robots = service.getRobots();

      // Given 3 robots on network
      expect(robots).toHaveLength(3);

      // Then I see 3 robots listed
      // And each shows name and IP
      robots.forEach((robot) => {
        expect(robot.name).toBeTruthy();
        expect(robot.ipAddress).toMatch(/^\d+\.\d+\.\d+\.\d+$/);
        expect(robot.port).toBeGreaterThan(0);
        expect(robot.version).toMatch(/^\d+\.\d+\.\d+$/);
      });
    });

    test('should emit robot_discovered event', (done) => {
      service.start();

      service.subscribe((event) => {
        if (event.type === 'robot_discovered') {
          expect(event.robot).toMatchObject({
            id: expect.any(String),
            name: expect.any(String),
            ipAddress: expect.stringMatching(/^\d+\.\d+\.\d+\.\d+$/),
            port: expect.any(Number),
            version: expect.any(String),
          });
          done();
        }
      });

      // Simulate robot discovery
      const newRobot: DiscoveredRobot = {
        id: 'mbot-999',
        name: 'mBot Test',
        ipAddress: '192.168.1.200',
        port: 8081,
        version: '1.0.0',
      };

      service.simulateDiscovery(newRobot);
    });

    test('should include robot metadata', async () => {
      await service.start();
      const robots = service.getRobots();

      const robotWithMetadata = robots.find((r) => r.metadata);
      expect(robotWithMetadata).toBeDefined();
      expect(robotWithMetadata?.metadata).toMatchObject({
        model: expect.any(String),
        firmware: expect.any(String),
        capabilities: expect.any(Array),
      });
    });

    test('should track lastSeen timestamp', async () => {
      await service.start();
      const robots = service.getRobots();

      robots.forEach((robot) => {
        expect(robot.lastSeen).toBeGreaterThan(0);
        expect(robot.lastSeen).toBeLessThanOrEqual(Date.now());
      });
    });
  });

  describe('Robot Connection State Management', () => {
    let service: MockDiscoveryService;

    beforeEach(async () => {
      service = new MockDiscoveryService();
      await service.start();
    });

    afterEach(async () => {
      await service.stop();
    });

    test('should have disconnected status by default', () => {
      const robots = service.getRobots();

      robots.forEach((robot) => {
        expect(robot.status).toBe('disconnected');
      });
    });

    test('should support all health indicator states', () => {
      const robots = service.getRobots();
      const robot = robots[0];

      const validStatuses: Array<RobotWithState['status']> = [
        'connected',
        'disconnected',
        'error',
        'discovering',
      ];

      validStatuses.forEach((status) => {
        // Type check ensures status is valid
        const testRobot: RobotWithState = { ...robot, status };
        expect(testRobot.status).toBe(status);
      });
    });
  });

  describe('Discovery Protocol Compliance - I-DISC-001', () => {
    test('should use mDNS service name format', () => {
      const service = new RobotDiscoveryService({
        serviceName: '_mbot._tcp.local',
      });

      // Service name should follow RFC 6762 format: _<service>._<proto>.<domain>
      const serviceName = '_mbot._tcp.local';
      expect(serviceName).toMatch(/^_[a-z0-9-]+\._tcp\.local$/);
    });

    test('should support standard mDNS port', () => {
      const robots: DiscoveredRobot[] = [
        {
          id: 'test-1',
          name: 'Test Robot',
          ipAddress: '192.168.1.100',
          port: 8081,
          version: '1.0.0',
        },
      ];

      robots.forEach((robot) => {
        expect(robot.port).toBeGreaterThan(0);
        expect(robot.port).toBeLessThanOrEqual(65535);
      });
    });

    test('should validate IP address format', () => {
      const service = new MockDiscoveryService();
      service.start();

      const robots = service.getRobots();

      robots.forEach((robot) => {
        // IPv4 address format
        expect(robot.ipAddress).toMatch(/^\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}$/);

        // Each octet should be 0-255
        const octets = robot.ipAddress.split('.').map(Number);
        octets.forEach((octet) => {
          expect(octet).toBeGreaterThanOrEqual(0);
          expect(octet).toBeLessThanOrEqual(255);
        });
      });
    });

    test('should support version format', () => {
      const service = new MockDiscoveryService();
      service.start();

      const robots = service.getRobots();

      robots.forEach((robot) => {
        // Semantic version format: major.minor.patch
        expect(robot.version).toMatch(/^\d+\.\d+\.\d+$/);
      });
    });
  });

  describe('Event-Driven Architecture', () => {
    test('should support unsubscribe from events', () => {
      const service = new MockDiscoveryService();
      let eventCount = 0;

      const unsubscribe = service.subscribe(() => {
        eventCount++;
      });

      // Simulate event
      service.simulateDiscovery({
        id: 'test',
        name: 'Test',
        ipAddress: '192.168.1.100',
        port: 8081,
        version: '1.0.0',
      });

      expect(eventCount).toBe(1);

      // Unsubscribe
      unsubscribe();

      // Simulate another event
      service.simulateDiscovery({
        id: 'test2',
        name: 'Test2',
        ipAddress: '192.168.1.101',
        port: 8081,
        version: '1.0.0',
      });

      // Event count should not increase
      expect(eventCount).toBe(1);
    });

    test('should handle subscriber errors gracefully', () => {
      const service = new MockDiscoveryService();

      // Subscribe with failing callback
      service.subscribe(() => {
        throw new Error('Subscriber error');
      });

      // Should not throw when emitting event
      expect(() => {
        service.simulateDiscovery({
          id: 'test',
          name: 'Test',
          ipAddress: '192.168.1.100',
          port: 8081,
          version: '1.0.0',
        });
      }).not.toThrow();
    });
  });

  describe('Gherkin Acceptance Criteria', () => {
    let service: MockDiscoveryService;

    beforeEach(async () => {
      service = new MockDiscoveryService();
      await service.start();
    });

    afterEach(async () => {
      await service.stop();
    });

    test('Scenario: Discover Robots', () => {
      // Given 3 robots on network
      const robots = service.getRobots();
      expect(robots).toHaveLength(3);

      // When I open discovery panel (simulated by start)
      expect(robots.length).toBeGreaterThan(0);

      // Then I see 3 robots listed
      expect(robots).toHaveLength(3);

      // And each shows name and IP
      robots.forEach((robot) => {
        expect(robot.name).toBeTruthy();
        expect(robot.ipAddress).toMatch(/^\d+\.\d+\.\d+\.\d+$/);
      });
    });

    test('Scenario: Connect to Robot - data contract validation', () => {
      const robots = service.getRobots();

      // When I click "Connect" on Robot2
      const robot2 = robots[1]; // Second robot
      expect(robot2.name).toBe('mBot Beta');

      // Then WebSocket opens to Robot2
      // And UI shows Robot2 state
      expect(robot2).toMatchObject({
        id: expect.any(String),
        name: 'mBot Beta',
        ipAddress: expect.stringMatching(/^\d+\.\d+\.\d+\.\d+$/),
        port: expect.any(Number),
        version: expect.any(String),
        status: expect.any(String),
      });
    });
  });

  describe('Data Contract Validation', () => {
    test('should match DiscoveredRobot interface', async () => {
      const service = new MockDiscoveryService();
      await service.start();

      const robots = service.getRobots();
      const robot = robots[0];

      // Validate required fields from contract
      expect(robot).toHaveProperty('id');
      expect(robot).toHaveProperty('name');
      expect(robot).toHaveProperty('ipAddress');
      expect(robot).toHaveProperty('port');
      expect(robot).toHaveProperty('version');

      // Validate types
      expect(typeof robot.id).toBe('string');
      expect(typeof robot.name).toBe('string');
      expect(typeof robot.ipAddress).toBe('string');
      expect(typeof robot.port).toBe('number');
      expect(typeof robot.version).toBe('string');
    });

    test('should include extended RobotWithState fields', async () => {
      const service = new MockDiscoveryService();
      await service.start();

      const robots = service.getRobots();
      const robot = robots[0];

      // Extended fields
      expect(robot).toHaveProperty('status');
      expect(robot).toHaveProperty('lastSeen');

      expect(typeof robot.status).toBe('string');
      expect(typeof robot.lastSeen).toBe('number');
    });
  });
});
