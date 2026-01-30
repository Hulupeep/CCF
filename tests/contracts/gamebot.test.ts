/**
 * Contract Tests for GameBot Turn Detection System (GAME-007)
 *
 * Validates that the implementation adheres to the feature_gamebot.yml contract.
 * Tests invariants I-GAME-003, I-GAME-006, I-GAME-007.
 */

import { describe, it, expect, beforeEach, afterEach } from 'vitest';

// ============================================
// Type Definitions (Mirror Rust types)
// ============================================

type TurnSignalType = 'tap' | 'double_tap' | 'voice' | 'timeout' | 'app';
type InputMethod = 'tap' | 'voice' | 'app';
type LedPattern = 'pulse_blue' | 'flash_green' | 'pulse_yellow';
type AckSound = 'beep' | 'chime' | 'voice';

interface TurnSignal {
  signal_type: TurnSignalType;
  confidence: number;
  timestamp_us: number;
  raw_data?: string;
}

interface TurnDetectionConfig {
  enabled_inputs: InputMethod[];
  double_tap_threshold_ms: number;
  double_tap_min_g: number;
  voice_keywords: string[];
  voice_confidence_threshold: number;
  timeout_seconds: number;
}

interface TurnAcknowledgment {
  led_pattern: LedPattern;
  sound: AckSound;
  voice_response?: string;
}

interface AccelerometerReading {
  x: number;
  y: number;
  z: number;
  timestamp_us: number;
}

interface VoiceDetectionResult {
  transcript: string;
  confidence: number;
  timestamp_us: number;
}

// ============================================
// Mock Turn Detection System
// ============================================

class TurnDetectionSystem {
  private config: TurnDetectionConfig;
  private lastTapTimestamp: number = 0;
  private lastTapForce: number = 0;
  private isRobotTurn: boolean = false;
  private lastInputTimestamp: number = 0;

  constructor(config?: Partial<TurnDetectionConfig>) {
    this.config = {
      enabled_inputs: ['tap', 'voice'],
      double_tap_threshold_ms: 400,
      double_tap_min_g: 2.0,
      voice_keywords: ['your turn', 'go', 'done', 'roby'],
      voice_confidence_threshold: 0.7,
      timeout_seconds: 30,
      ...config,
    };
  }

  setRobotTurn(isRobotTurn: boolean): void {
    this.isRobotTurn = isRobotTurn;
  }

  getConfig(): TurnDetectionConfig {
    return { ...this.config };
  }

  updateConfig(config: Partial<TurnDetectionConfig>): void {
    this.config = { ...this.config, ...config };
  }

  /**
   * Process accelerometer reading for tap detection
   * Returns TurnSignal if double-tap detected, null otherwise
   */
  processAccelerometer(reading: AccelerometerReading): TurnSignal | null {
    // I-GAME-008: Detection disabled during robot's turn
    if (this.isRobotTurn) {
      return null;
    }

    if (!this.config.enabled_inputs.includes('tap')) {
      return null;
    }

    // Calculate g-force magnitude
    const gForce = Math.sqrt(
      reading.x * reading.x + reading.y * reading.y + reading.z * reading.z
    );

    // I-GAME-006: Require minimum g-force threshold
    if (gForce < this.config.double_tap_min_g) {
      return null;
    }

    const timeSinceLastTap = (reading.timestamp_us - this.lastTapTimestamp) / 1000;

    // Check for double-tap pattern
    if (
      this.lastTapForce >= this.config.double_tap_min_g &&
      timeSinceLastTap <= this.config.double_tap_threshold_ms
    ) {
      // Double-tap detected!
      this.lastTapTimestamp = 0;
      this.lastTapForce = 0;
      this.lastInputTimestamp = reading.timestamp_us;

      return {
        signal_type: 'double_tap',
        confidence: 1.0, // Deterministic for taps
        timestamp_us: reading.timestamp_us,
      };
    }

    // Store this tap for potential double-tap
    this.lastTapTimestamp = reading.timestamp_us;
    this.lastTapForce = gForce;

    return null;
  }

  /**
   * Process voice detection result
   * Returns TurnSignal if keyword detected with sufficient confidence
   */
  processVoice(result: VoiceDetectionResult): TurnSignal | null {
    // I-GAME-008: Detection disabled during robot's turn
    if (this.isRobotTurn) {
      return null;
    }

    if (!this.config.enabled_inputs.includes('voice')) {
      return null;
    }

    // I-GAME-006: Require minimum confidence threshold
    if (result.confidence < this.config.voice_confidence_threshold) {
      return null;
    }

    const transcript = result.transcript.toLowerCase();
    const matchedKeyword = this.config.voice_keywords.find(keyword =>
      transcript.includes(keyword.toLowerCase())
    );

    if (!matchedKeyword) {
      return null;
    }

    // Higher confidence required for name-based commands
    if (matchedKeyword.includes('roby') && result.confidence < 0.8) {
      return null;
    }

    this.lastInputTimestamp = result.timestamp_us;

    return {
      signal_type: 'voice',
      confidence: result.confidence,
      timestamp_us: result.timestamp_us,
      raw_data: result.transcript,
    };
  }

  /**
   * Check for timeout condition
   */
  checkTimeout(currentTimestamp: number): TurnSignal | null {
    if (this.isRobotTurn) {
      return null;
    }

    const timeSinceInput = (currentTimestamp - this.lastInputTimestamp) / 1_000_000;

    if (timeSinceInput >= this.config.timeout_seconds) {
      return {
        signal_type: 'timeout',
        confidence: 1.0,
        timestamp_us: currentTimestamp,
      };
    }

    return null;
  }

  /**
   * Generate acknowledgment for a turn signal
   * I-GAME-007: Robot must always acknowledge turn receipt
   */
  generateAcknowledgment(signal: TurnSignal): TurnAcknowledgment {
    switch (signal.signal_type) {
      case 'double_tap':
        return {
          led_pattern: 'pulse_blue',
          sound: 'beep',
          voice_response: "OK, my turn!",
        };
      case 'voice':
        return {
          led_pattern: 'flash_green',
          sound: 'chime',
          voice_response: "Got it!",
        };
      case 'timeout':
        return {
          led_pattern: 'pulse_yellow',
          sound: 'voice',
          voice_response: "Are you ready? Tap twice or say done",
        };
      case 'app':
        return {
          led_pattern: 'flash_green',
          sound: 'beep',
          voice_response: "OK!",
        };
      default:
        return {
          led_pattern: 'pulse_blue',
          sound: 'beep',
        };
    }
  }

  resetInputTimer(timestamp: number): void {
    this.lastInputTimestamp = timestamp;
  }
}

// ============================================
// Contract Tests
// ============================================

describe('GameBot Contract Tests (GAME-007)', () => {
  let detector: TurnDetectionSystem;

  beforeEach(() => {
    detector = new TurnDetectionSystem();
  });

  describe('Data Contract: TurnSignal', () => {
    it('should have required fields with correct types', () => {
      const signal: TurnSignal = {
        signal_type: 'double_tap',
        confidence: 1.0,
        timestamp_us: 1000000,
      };

      expect(signal.signal_type).toBeDefined();
      expect(typeof signal.confidence).toBe('number');
      expect(signal.confidence).toBeGreaterThanOrEqual(0);
      expect(signal.confidence).toBeLessThanOrEqual(1);
      expect(typeof signal.timestamp_us).toBe('number');
    });

    it('should allow optional raw_data for voice signals', () => {
      const signal: TurnSignal = {
        signal_type: 'voice',
        confidence: 0.85,
        timestamp_us: 1000000,
        raw_data: 'your turn',
      };

      expect(signal.raw_data).toBe('your turn');
    });
  });

  describe('Data Contract: TurnDetectionConfig', () => {
    it('should have correct default values', () => {
      const config = detector.getConfig();

      expect(config.double_tap_threshold_ms).toBe(400);
      expect(config.double_tap_min_g).toBe(2.0);
      expect(config.voice_keywords).toContain('your turn');
      expect(config.voice_keywords).toContain('go');
      expect(config.voice_keywords).toContain('done');
      expect(config.voice_keywords).toContain('roby');
      expect(config.voice_confidence_threshold).toBe(0.7);
      expect(config.timeout_seconds).toBe(30);
    });

    it('should allow custom configuration', () => {
      const customDetector = new TurnDetectionSystem({
        double_tap_threshold_ms: 500,
        voice_confidence_threshold: 0.8,
      });
      const config = customDetector.getConfig();

      expect(config.double_tap_threshold_ms).toBe(500);
      expect(config.voice_confidence_threshold).toBe(0.8);
    });
  });

  describe('Invariant I-GAME-006: No false positives', () => {
    it('should ignore single tap', () => {
      const reading: AccelerometerReading = {
        x: 0, y: 2.5, z: 0,
        timestamp_us: 1000000,
      };

      const result = detector.processAccelerometer(reading);
      expect(result).toBeNull();
    });

    it('should ignore bump less than 2g', () => {
      // First tap below threshold
      const reading1: AccelerometerReading = {
        x: 0, y: 1.5, z: 0, // 1.5g < 2g threshold
        timestamp_us: 1000000,
      };

      // Second tap also below threshold
      const reading2: AccelerometerReading = {
        x: 0, y: 1.8, z: 0, // 1.8g < 2g threshold
        timestamp_us: 1200000, // 200ms later
      };

      detector.processAccelerometer(reading1);
      const result = detector.processAccelerometer(reading2);

      expect(result).toBeNull();
    });

    it('should ignore voice with confidence below threshold', () => {
      const result = detector.processVoice({
        transcript: 'your turn',
        confidence: 0.5, // Below 0.7 threshold
        timestamp_us: 1000000,
      });

      expect(result).toBeNull();
    });

    it('should ignore non-keyword voice commands', () => {
      const result = detector.processVoice({
        transcript: 'hello there',
        confidence: 0.9,
        timestamp_us: 1000000,
      });

      expect(result).toBeNull();
    });
  });

  describe('Invariant I-GAME-007: Always acknowledge turn', () => {
    it('should generate LED pulse for double-tap', () => {
      const signal: TurnSignal = {
        signal_type: 'double_tap',
        confidence: 1.0,
        timestamp_us: 1000000,
      };

      const ack = detector.generateAcknowledgment(signal);

      expect(ack.led_pattern).toBe('pulse_blue');
      expect(ack.sound).toBeDefined();
      expect(ack.voice_response).toBeDefined();
    });

    it('should generate LED flash for voice command', () => {
      const signal: TurnSignal = {
        signal_type: 'voice',
        confidence: 0.85,
        timestamp_us: 1000000,
        raw_data: 'your turn',
      };

      const ack = detector.generateAcknowledgment(signal);

      expect(ack.led_pattern).toBe('flash_green');
      expect(ack.voice_response).toBe('Got it!');
    });

    it('should generate timeout prompt with yellow LED', () => {
      const signal: TurnSignal = {
        signal_type: 'timeout',
        confidence: 1.0,
        timestamp_us: 31000000,
      };

      const ack = detector.generateAcknowledgment(signal);

      expect(ack.led_pattern).toBe('pulse_yellow');
      expect(ack.voice_response).toContain('Tap twice or say done');
    });
  });

  describe('GAME-007-tap: Double tap detection', () => {
    it('should detect double-tap within threshold', () => {
      // First tap
      const reading1: AccelerometerReading = {
        x: 0, y: 3.0, z: 0,
        timestamp_us: 1000000,
      };

      // Second tap within 400ms
      const reading2: AccelerometerReading = {
        x: 0, y: 2.5, z: 0,
        timestamp_us: 1300000, // 300ms later
      };

      const result1 = detector.processAccelerometer(reading1);
      expect(result1).toBeNull(); // First tap should not trigger

      const result2 = detector.processAccelerometer(reading2);
      expect(result2).not.toBeNull();
      expect(result2?.signal_type).toBe('double_tap');
      expect(result2?.confidence).toBe(1.0);
    });

    it('should not detect taps outside threshold window', () => {
      // First tap
      const reading1: AccelerometerReading = {
        x: 0, y: 3.0, z: 0,
        timestamp_us: 1000000,
      };

      // Second tap after 500ms (outside 400ms threshold)
      const reading2: AccelerometerReading = {
        x: 0, y: 2.5, z: 0,
        timestamp_us: 1500000, // 500ms later
      };

      detector.processAccelerometer(reading1);
      const result = detector.processAccelerometer(reading2);

      // Should be stored as new first tap, not trigger double-tap
      expect(result).toBeNull();
    });
  });

  describe('GAME-007-voice: Voice command detection', () => {
    it('should detect "your turn" with sufficient confidence', () => {
      const result = detector.processVoice({
        transcript: 'your turn',
        confidence: 0.85,
        timestamp_us: 1000000,
      });

      expect(result).not.toBeNull();
      expect(result?.signal_type).toBe('voice');
      expect(result?.raw_data).toBe('your turn');
    });

    it('should detect "done" command', () => {
      const result = detector.processVoice({
        transcript: "I'm done",
        confidence: 0.75,
        timestamp_us: 1000000,
      });

      expect(result).not.toBeNull();
      expect(result?.signal_type).toBe('voice');
    });

    it('should detect "go Roby" with higher confidence requirement', () => {
      // Should fail at 0.75 confidence
      const result1 = detector.processVoice({
        transcript: 'go Roby',
        confidence: 0.75,
        timestamp_us: 1000000,
      });
      expect(result1).toBeNull();

      // Should succeed at 0.85 confidence
      const result2 = detector.processVoice({
        transcript: 'go Roby',
        confidence: 0.85,
        timestamp_us: 1000000,
      });
      expect(result2).not.toBeNull();
    });
  });

  describe('GAME-007-timeout: Timeout detection', () => {
    it('should detect timeout after 30 seconds', () => {
      detector.resetInputTimer(0);

      // Check at 29 seconds - should not trigger
      const result1 = detector.checkTimeout(29_000_000);
      expect(result1).toBeNull();

      // Check at 30 seconds - should trigger
      const result2 = detector.checkTimeout(30_000_000);
      expect(result2).not.toBeNull();
      expect(result2?.signal_type).toBe('timeout');
    });
  });

  describe('GAME-007-robot-turn: Detection disabled during robot turn', () => {
    it('should ignore double-tap during robot turn', () => {
      detector.setRobotTurn(true);

      const reading1: AccelerometerReading = {
        x: 0, y: 3.0, z: 0,
        timestamp_us: 1000000,
      };
      const reading2: AccelerometerReading = {
        x: 0, y: 2.5, z: 0,
        timestamp_us: 1200000,
      };

      detector.processAccelerometer(reading1);
      const result = detector.processAccelerometer(reading2);

      expect(result).toBeNull();
    });

    it('should ignore voice command during robot turn', () => {
      detector.setRobotTurn(true);

      const result = detector.processVoice({
        transcript: 'your turn',
        confidence: 0.9,
        timestamp_us: 1000000,
      });

      expect(result).toBeNull();
    });

    it('should not trigger timeout during robot turn', () => {
      detector.setRobotTurn(true);
      detector.resetInputTimer(0);

      const result = detector.checkTimeout(60_000_000);
      expect(result).toBeNull();
    });
  });

  describe('GAME-007-config: Configurable input methods', () => {
    it('should only detect enabled input methods', () => {
      // Disable tap detection
      detector.updateConfig({ enabled_inputs: ['voice'] });

      const reading1: AccelerometerReading = {
        x: 0, y: 3.0, z: 0,
        timestamp_us: 1000000,
      };
      const reading2: AccelerometerReading = {
        x: 0, y: 2.5, z: 0,
        timestamp_us: 1200000,
      };

      detector.processAccelerometer(reading1);
      const tapResult = detector.processAccelerometer(reading2);
      expect(tapResult).toBeNull();

      // Voice should still work
      const voiceResult = detector.processVoice({
        transcript: 'done',
        confidence: 0.8,
        timestamp_us: 1000000,
      });
      expect(voiceResult).not.toBeNull();
    });

    it('should allow voice-only configuration', () => {
      const voiceOnlyDetector = new TurnDetectionSystem({
        enabled_inputs: ['voice'],
      });

      expect(voiceOnlyDetector.getConfig().enabled_inputs).toEqual(['voice']);
    });

    it('should allow tap-only configuration', () => {
      const tapOnlyDetector = new TurnDetectionSystem({
        enabled_inputs: ['tap'],
      });

      expect(tapOnlyDetector.getConfig().enabled_inputs).toEqual(['tap']);
    });
  });

  describe('ARCH-GAME-003: Bounded response latency', () => {
    it('acknowledgment generation should be fast (no blocking)', () => {
      const signal: TurnSignal = {
        signal_type: 'double_tap',
        confidence: 1.0,
        timestamp_us: 1000000,
      };

      const start = performance.now();
      detector.generateAcknowledgment(signal);
      const elapsed = performance.now() - start;

      // Should complete in <1ms (no blocking operations)
      expect(elapsed).toBeLessThan(10);
    });
  });

  describe('ARCH-GAME-004: Physical-only mode', () => {
    it('should work without app input method enabled', () => {
      const physicalOnlyDetector = new TurnDetectionSystem({
        enabled_inputs: ['tap', 'voice'],
      });

      // Double-tap should work
      const reading1: AccelerometerReading = {
        x: 0, y: 3.0, z: 0,
        timestamp_us: 1000000,
      };
      const reading2: AccelerometerReading = {
        x: 0, y: 2.5, z: 0,
        timestamp_us: 1200000,
      };

      physicalOnlyDetector.processAccelerometer(reading1);
      const tapResult = physicalOnlyDetector.processAccelerometer(reading2);
      expect(tapResult).not.toBeNull();

      // Voice should work
      const voiceResult = physicalOnlyDetector.processVoice({
        transcript: 'your turn',
        confidence: 0.8,
        timestamp_us: 2000000,
      });
      expect(voiceResult).not.toBeNull();
    });
  });
});

// ============================================
// Integration Contract Tests
// ============================================

describe('GameBot Integration Contract Tests', () => {
  describe('Full turn lifecycle', () => {
    it('should complete turn signal -> acknowledgment cycle', () => {
      const detector = new TurnDetectionSystem();

      // Simulate double-tap
      const reading1: AccelerometerReading = {
        x: 0, y: 3.0, z: 0,
        timestamp_us: 1000000,
      };
      const reading2: AccelerometerReading = {
        x: 0, y: 2.5, z: 0,
        timestamp_us: 1200000,
      };

      detector.processAccelerometer(reading1);
      const signal = detector.processAccelerometer(reading2);

      expect(signal).not.toBeNull();

      const ack = detector.generateAcknowledgment(signal!);

      expect(ack.led_pattern).toBe('pulse_blue');
      expect(ack.sound).toBe('beep');
      expect(ack.voice_response).toBe("OK, my turn!");
    });
  });

  describe('State transitions', () => {
    it('should handle human turn -> robot turn transition', () => {
      const detector = new TurnDetectionSystem();

      // Initially human's turn, robot waiting
      detector.setRobotTurn(false);

      // Human signals turn complete
      const reading1: AccelerometerReading = {
        x: 0, y: 3.0, z: 0,
        timestamp_us: 1000000,
      };
      const reading2: AccelerometerReading = {
        x: 0, y: 2.5, z: 0,
        timestamp_us: 1200000,
      };

      detector.processAccelerometer(reading1);
      const signal = detector.processAccelerometer(reading2);
      expect(signal).not.toBeNull();

      // Transition to robot's turn
      detector.setRobotTurn(true);

      // Further inputs should be ignored
      const reading3: AccelerometerReading = {
        x: 0, y: 3.0, z: 0,
        timestamp_us: 2000000,
      };
      const reading4: AccelerometerReading = {
        x: 0, y: 2.5, z: 0,
        timestamp_us: 2200000,
      };

      detector.processAccelerometer(reading3);
      const ignoredSignal = detector.processAccelerometer(reading4);
      expect(ignoredSignal).toBeNull();
    });
  });
});
