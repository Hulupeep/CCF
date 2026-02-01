/**
 * Contract tests for Multi-Robot Coordination
 * Validates invariants I-MULTI-001 through I-MULTI-006
 */

import { describe, it, expect } from 'vitest';
import * as fs from 'fs';
import * as path from 'path';

const CONTRACTS_PATH = path.join(__dirname, '../../docs/contracts/feature_multi_robot.yml');
const RUST_MULTI_ROBOT_PATH = path.join(__dirname, '../../crates/mbot-core/src/multi_robot');
const WEB_SERVICE_PATH = path.join(__dirname, '../../web/src/services/multiRobotCoordination.ts');

describe('Multi-Robot Coordination Contract Enforcement', () => {
  let rustCode: string;
  let webCode: string;

  beforeAll(() => {
    // Read Rust multi_robot module
    const rustModPath = path.join(RUST_MULTI_ROBOT_PATH, 'mod.rs');
    rustCode = fs.readFileSync(rustModPath, 'utf-8');

    // Read web service
    webCode = fs.readFileSync(WEB_SERVICE_PATH, 'utf-8');
  });

  describe('I-MULTI-001: Discovery timeout must be 5 seconds', () => {
    it('Rust: should define DISCOVERY_TIMEOUT_MS = 5000', () => {
      expect(rustCode).toMatch(/DISCOVERY_TIMEOUT_MS.*5000/);
    });

    it('Web: should define DISCOVERY_TIMEOUT_MS = 5000', () => {
      expect(webCode).toMatch(/DISCOVERY_TIMEOUT_MS\s*=\s*5000/);
    });

    it('Rust: should use discovery timeout in config', () => {
      expect(rustCode).toMatch(/discovery_timeout.*DISCOVERY_TIMEOUT_MS/);
    });

    it('Web: should use discovery timeout in config', () => {
      expect(webCode).toMatch(/discoveryTimeout.*DISCOVERY_TIMEOUT_MS/);
    });
  });

  describe('I-MULTI-002: State synchronization must maintain consistency', () => {
    it('Rust: should use sequence numbers for state updates', () => {
      expect(rustCode).toMatch(/sequence.*u64/);
      expect(rustCode).toMatch(/RobotState.*sequence/);
    });

    it('Web: should use sequence numbers for state updates', () => {
      expect(webCode).toMatch(/sequence.*number/);
      expect(webCode).toMatch(/RobotState.*sequence/);
    });

    it('Rust: should check sequence before updating state', () => {
      expect(rustCode).toMatch(/if.*sequence.*>.*robot\.sequence/);
    });

    it('Web: should check sequence before updating state', () => {
      expect(webCode).toMatch(/if.*sequence.*>.*robot\.sequence/);
    });

    it('Rust: should have heartbeat mechanism', () => {
      expect(rustCode).toMatch(/HEARTBEAT_INTERVAL_MS/);
      expect(rustCode).toMatch(/last_heartbeat/);
    });

    it('Web: should have heartbeat mechanism', () => {
      expect(webCode).toMatch(/HEARTBEAT_INTERVAL_MS/);
      expect(webCode).toMatch(/lastHeartbeat/);
    });
  });

  describe('I-MULTI-003: Leader election must complete within 3 seconds', () => {
    it('Rust: should define ELECTION_TIMEOUT_MS = 3000', () => {
      expect(rustCode).toMatch(/ELECTION_TIMEOUT_MS.*3000/);
    });

    it('Web: should define ELECTION_TIMEOUT_MS = 3000', () => {
      expect(webCode).toMatch(/ELECTION_TIMEOUT_MS\s*=\s*3000/);
    });

    it('Rust: should have election algorithm (Bully)', () => {
      expect(rustCode).toMatch(/election|Election/);
      expect(rustCode).toMatch(/priority|Priority/);
    });

    it('Web: should have election algorithm (Bully)', () => {
      expect(webCode).toMatch(/election|Election/);
      expect(webCode).toMatch(/priority|Priority/);
    });

    it('Rust: should check election timeout', () => {
      expect(rustCode).toMatch(/check.*election.*timeout|election.*elapsed/i);
    });

    it('Web: should check election timeout', () => {
      expect(webCode).toMatch(/checkElectionTimeout|election.*elapsed/i);
    });
  });

  describe('I-MULTI-004: Maximum 4 robots', () => {
    it('Rust: should define MAX_ROBOTS = 4', () => {
      expect(rustCode).toMatch(/MAX_ROBOTS.*=.*4/);
    });

    it('Web: should define MAX_ROBOTS = 4', () => {
      expect(webCode).toMatch(/MAX_ROBOTS\s*=\s*4/);
    });

    it('Rust: should enforce max robots limit', () => {
      expect(rustCode).toMatch(/robots\.len.*>=.*max_robots/);
      expect(rustCode).toMatch(/TooManyRobots/);
    });

    it('Web: should enforce max robots limit', () => {
      expect(webCode).toMatch(/robots\.size.*>=.*maxRobots/);
      expect(webCode).toMatch(/TOO_MANY_ROBOTS/);
    });
  });

  describe('I-MULTI-005: Sync latency must be <100ms', () => {
    it('Rust: should define SYNC_INTERVAL_MS = 100', () => {
      expect(rustCode).toMatch(/SYNC_INTERVAL_MS.*100/);
    });

    it('Web: should define SYNC_INTERVAL_MS = 100', () => {
      expect(webCode).toMatch(/SYNC_INTERVAL_MS\s*=\s*100/);
    });

    it('Rust: should use sync interval in config', () => {
      expect(rustCode).toMatch(/sync_interval.*SYNC_INTERVAL_MS/);
    });

    it('Web: should use sync interval in config', () => {
      expect(webCode).toMatch(/syncInterval.*SYNC_INTERVAL_MS/);
    });
  });

  describe('I-MULTI-006: Graceful disconnect handling', () => {
    it('Rust: should define disconnect timeout', () => {
      expect(rustCode).toMatch(/DISCONNECT_TIMEOUT_MS/);
    });

    it('Web: should define disconnect timeout', () => {
      expect(webCode).toMatch(/DISCONNECT_TIMEOUT_MS/);
    });

    it('Rust: should detect disconnects', () => {
      expect(rustCode).toMatch(/is_disconnected|detect.*disconnect/i);
    });

    it('Web: should detect disconnects', () => {
      expect(webCode).toMatch(/isDisconnected|detectDisconnects/i);
    });

    it('Rust: should trigger election on leader disconnect', () => {
      expect(rustCode).toMatch(/if.*was_leader[\s\S]*start_election/i);
    });

    it('Web: should trigger election on leader disconnect', () => {
      expect(webCode).toMatch(/if.*wasLeader[\s\S]*startElection/i);
    });

    it('Rust: should remove disconnected robots', () => {
      expect(rustCode).toMatch(/remove_robot|robots\.retain/);
    });

    it('Web: should remove disconnected robots', () => {
      expect(webCode).toMatch(/removeRobot|robots\.delete/);
    });
  });

  describe('Data Contract Compliance', () => {
    it('Rust: should have CoordinationMessage struct', () => {
      expect(rustCode).toMatch(/struct CoordinationMessage/);
      expect(rustCode).toMatch(/from_robot.*RobotId/);
      expect(rustCode).toMatch(/to_robots.*Vec<RobotId>/);
      expect(rustCode).toMatch(/action.*MessageAction/);
      expect(rustCode).toMatch(/timestamp.*u64/);
      expect(rustCode).toMatch(/sequence.*u64/);
    });

    it('Web: should have CoordinationMessage interface', () => {
      expect(webCode).toMatch(/interface CoordinationMessage/);
      expect(webCode).toMatch(/fromRobot.*string/);
      expect(webCode).toMatch(/toRobots.*string\[\]/);
      expect(webCode).toMatch(/action.*MessageAction/);
      expect(webCode).toMatch(/timestamp.*number/);
      expect(webCode).toMatch(/sequence.*number/);
    });

    it('Rust: should have RobotState struct', () => {
      expect(rustCode).toMatch(/struct RobotState/);
      expect(rustCode).toMatch(/role.*RobotRole/);
      expect(rustCode).toMatch(/position.*Position/);
      expect(rustCode).toMatch(/status.*RobotStatus/);
      expect(rustCode).toMatch(/last_heartbeat/);
    });

    it('Web: should have RobotState interface', () => {
      expect(webCode).toMatch(/interface RobotState/);
      expect(webCode).toMatch(/role.*RobotRole/);
      expect(webCode).toMatch(/position.*Position/);
      expect(webCode).toMatch(/status.*RobotStatus/);
      expect(webCode).toMatch(/lastHeartbeat/);
    });

    it('Rust: should have CoordinationConfig struct', () => {
      expect(rustCode).toMatch(/struct CoordinationConfig/);
      expect(rustCode).toMatch(/max_robots/);
      expect(rustCode).toMatch(/heartbeat_interval/);
      expect(rustCode).toMatch(/discovery_timeout/);
      expect(rustCode).toMatch(/sync_interval/);
    });

    it('Web: should have CoordinationConfig interface', () => {
      expect(webCode).toMatch(/interface CoordinationConfig/);
      expect(webCode).toMatch(/maxRobots/);
      expect(webCode).toMatch(/heartbeatInterval/);
      expect(webCode).toMatch(/discoveryTimeout/);
      expect(webCode).toMatch(/syncInterval/);
    });
  });

  describe('Architecture Contract Compliance', () => {
    it('Rust: should be no_std compatible (ARCH-001)', () => {
      expect(rustCode).toMatch(/#!\[cfg_attr\(feature = "no_std", no_std\)\]/);
      expect(rustCode).not.toMatch(/use std::/);
    });

    it('Rust: should use bounded values (ARCH-004)', () => {
      expect(rustCode).toMatch(/\.clamp|\.min|\.max/);
    });

    it('Rust: should not have harmful terminology (ARCH-003)', () => {
      expect(rustCode).not.toMatch(/weapon|attack|hurt|kill|destroy|violence/i);
    });

    it('Web: should not have harmful terminology (ARCH-003)', () => {
      expect(webCode).not.toMatch(/weapon|attack|hurt|kill|destroy|violence/i);
    });
  });
});
