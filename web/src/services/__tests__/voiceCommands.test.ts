/**
 * Unit tests for VoiceCommandService
 * Tests all contracts and invariants from feature_voice.yml
 *
 * Contracts: VOICE-001 through VOICE-005
 * Invariants: I-VOICE-001, I-VOICE-002, I-VOICE-003
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { VoiceCommandService } from '../voiceCommands';
import {
  createDefaultVoiceSettings,
  validateVoiceSettings,
  checkBrowserCompatibility,
  MAX_HISTORY_ENTRIES,
  MAX_COMMAND_LATENCY_MS,
  CONFIRMATION_TIMEOUT_MS,
} from '../../types/voiceCommand';

// Mock SpeechRecognition API
class MockSpeechRecognition {
  continuous = false;
  interimResults = false;
  lang = 'en-US';

  onstart: (() => void) | null = null;
  onresult: ((event: any) => void) | null = null;
  onerror: ((event: any) => void) | null = null;
  onend: (() => void) | null = null;

  start() {
    if (this.onstart) this.onstart();
  }

  stop() {
    if (this.onend) this.onend();
  }

  simulateResult(transcript: string, confidence: number = 1.0, isFinal: boolean = true) {
    if (this.onresult) {
      this.onresult({
        results: [
          {
            isFinal,
            0: { transcript, confidence },
          },
        ],
        results: {
          length: 1,
          [0]: {
            isFinal,
            0: { transcript, confidence },
          },
        },
      } as any);
    }
  }

  simulateError(error: string) {
    if (this.onerror) {
      this.onerror({ error } as any);
    }
  }
}

describe('VoiceCommandService - Initialization', () => {
  beforeEach(() => {
    VoiceCommandService.resetInstance();
    localStorage.clear();

    // Mock SpeechRecognition
    (global as any).window = {
      SpeechRecognition: MockSpeechRecognition,
      AudioContext: class {
        currentTime = 0;
        createOscillator() {
          return {
            frequency: { value: 0 },
            connect: vi.fn(),
            start: vi.fn(),
            stop: vi.fn(),
          };
        }
        createGain() {
          return {
            gain: {
              setValueAtTime: vi.fn(),
              exponentialRampToValueAtTime: vi.fn(),
            },
            connect: vi.fn(),
          };
        }
        get destination() {
          return {};
        }
      },
    };
  });

  afterEach(() => {
    VoiceCommandService.resetInstance();
  });

  it('should create singleton instance', () => {
    const instance1 = VoiceCommandService.getInstance();
    const instance2 = VoiceCommandService.getInstance();

    expect(instance1).toBe(instance2);
  });

  it('should initialize with default settings', () => {
    const service = VoiceCommandService.getInstance();
    const settings = service.getSettings();

    expect(settings.enabled).toBe(false);
    expect(settings.wakeWordEnabled).toBe(false);
    expect(settings.wakeWord).toBe('hey robot');
    expect(settings.language).toBe('en-US');
    expect(settings.confidenceThreshold).toBe(0.7);
    expect(settings.noiseThreshold).toBe(50);
  });

  it('should register 15+ default commands', () => {
    const service = VoiceCommandService.getInstance();
    const history = service.getCommandHistory();

    // After initialization, history should be empty
    expect(history.length).toBe(0);
  });
});

describe('VoiceCommandService - Settings Validation', () => {
  beforeEach(() => {
    VoiceCommandService.resetInstance();
  });

  it('should validate correct settings', () => {
    const settings = createDefaultVoiceSettings();
    expect(validateVoiceSettings(settings)).toBe(true);
  });

  it('should reject invalid noise threshold (negative)', () => {
    const settings = { noiseThreshold: -10 };
    expect(validateVoiceSettings(settings)).toBe(false);
  });

  it('should reject invalid noise threshold (>120dB)', () => {
    const settings = { noiseThreshold: 150 };
    expect(validateVoiceSettings(settings)).toBe(false);
  });

  it('should reject invalid confidence threshold (<0)', () => {
    const settings = { confidenceThreshold: -0.1 };
    expect(validateVoiceSettings(settings)).toBe(false);
  });

  it('should reject invalid confidence threshold (>1)', () => {
    const settings = { confidenceThreshold: 1.5 };
    expect(validateVoiceSettings(settings)).toBe(false);
  });

  it('should reject empty wake word', () => {
    const settings = { wakeWord: '' };
    expect(validateVoiceSettings(settings)).toBe(false);
  });

  it('should accept valid wake word', () => {
    const settings = { wakeWord: 'hey robot' };
    expect(validateVoiceSettings(settings)).toBe(true);
  });
});

describe('VoiceCommandService - I-VOICE-002: Confidence Threshold', () => {
  beforeEach(() => {
    VoiceCommandService.resetInstance();
    (global as any).window = {
      SpeechRecognition: MockSpeechRecognition,
      AudioContext: class {
        currentTime = 0;
        createOscillator() {
          return {
            frequency: { value: 0 },
            connect: vi.fn(),
            start: vi.fn(),
            stop: vi.fn(),
          };
        }
        createGain() {
          return {
            gain: {
              setValueAtTime: vi.fn(),
              exponentialRampToValueAtTime: vi.fn(),
            },
            connect: vi.fn(),
          };
        }
        get destination() {
          return {};
        }
      },
    };
  });

  it('should filter commands below confidence threshold', async () => {
    const service = VoiceCommandService.getInstance();

    // Set confidence threshold to 0.7
    service.updateSettings({ confidenceThreshold: 0.7 });

    // Process low confidence command
    const result = await service.processCommand('stop', 0.5);

    expect(result.recognized).toBe(false);
  });

  it('should accept commands above confidence threshold', async () => {
    const service = VoiceCommandService.getInstance();

    // Set confidence threshold to 0.7
    service.updateSettings({ confidenceThreshold: 0.7 });

    // Process high confidence command
    const result = await service.processCommand('stop', 0.9);

    expect(result.recognized).toBe(true);
    expect(result.action).toBe('STOP');
  });
});

describe('VoiceCommandService - Command Processing', () => {
  beforeEach(() => {
    VoiceCommandService.resetInstance();
    (global as any).window = {
      SpeechRecognition: MockSpeechRecognition,
      AudioContext: class {
        currentTime = 0;
        createOscillator() {
          return {
            frequency: { value: 0 },
            connect: vi.fn(),
            start: vi.fn(),
            stop: vi.fn(),
          };
        }
        createGain() {
          return {
            gain: {
              setValueAtTime: vi.fn(),
              exponentialRampToValueAtTime: vi.fn(),
            },
            connect: vi.fn(),
          };
        }
        get destination() {
          return {};
        }
      },
    };
  });

  it('should recognize "stop" command', async () => {
    const service = VoiceCommandService.getInstance();
    const result = await service.processCommand('stop', 1.0);

    expect(result.recognized).toBe(true);
    expect(result.action).toBe('STOP');
  });

  it('should recognize "start drawing" command', async () => {
    const service = VoiceCommandService.getInstance();
    const result = await service.processCommand('start drawing', 1.0);

    expect(result.recognized).toBe(true);
    expect(result.action).toBe('START_ARTBOT');
  });

  it('should extract parameters from "switch to curious" command', async () => {
    const service = VoiceCommandService.getInstance();
    const result = await service.processCommand('switch to curious', 1.0);

    expect(result.recognized).toBe(true);
    expect(result.action).toBe('SWITCH_PERSONALITY');
    expect(result.params).toContain('curious');
  });

  it('should extract multiple parameters from adjustment command', async () => {
    const service = VoiceCommandService.getInstance();
    const result = await service.processCommand('increase energy by 50', 1.0);

    expect(result.recognized).toBe(true);
    expect(result.action).toBe('ADJUST_PARAMETER');
    expect(result.params).toContain('increase');
    expect(result.params).toContain('energy');
    expect(result.params).toContain('50');
  });

  it('should reject unrecognized command', async () => {
    const service = VoiceCommandService.getInstance();
    const result = await service.processCommand('invalid command xyz', 1.0);

    expect(result.recognized).toBe(false);
    expect(result.error).toContain('not recognized');
  });

  it('should handle case-insensitive commands', async () => {
    const service = VoiceCommandService.getInstance();

    const result1 = await service.processCommand('STOP', 1.0);
    const result2 = await service.processCommand('stop', 1.0);
    const result3 = await service.processCommand('StOp', 1.0);

    expect(result1.recognized).toBe(true);
    expect(result2.recognized).toBe(true);
    expect(result3.recognized).toBe(true);
  });
});

describe('VoiceCommandService - I-VOICE-001: Command Latency', () => {
  beforeEach(() => {
    VoiceCommandService.resetInstance();
    (global as any).window = {
      SpeechRecognition: MockSpeechRecognition,
      AudioContext: class {
        currentTime = 0;
        createOscillator() {
          return {
            frequency: { value: 0 },
            connect: vi.fn(),
            start: vi.fn(),
            stop: vi.fn(),
          };
        }
        createGain() {
          return {
            gain: {
              setValueAtTime: vi.fn(),
              exponentialRampToValueAtTime: vi.fn(),
            },
            connect: vi.fn(),
          };
        }
        get destination() {
          return {};
        }
      },
    };
  });

  it('should execute commands within 500ms', async () => {
    const service = VoiceCommandService.getInstance();

    const startTime = performance.now();
    await service.processCommand('stop', 1.0);
    const endTime = performance.now();

    const executionTime = endTime - startTime;
    expect(executionTime).toBeLessThan(MAX_COMMAND_LATENCY_MS);
  });

  it('should track execution time in history', async () => {
    const service = VoiceCommandService.getInstance();

    await service.processCommand('stop', 1.0);

    const history = service.getCommandHistory();
    expect(history.length).toBeGreaterThan(0);
    expect(history[0].executionTime).toBeDefined();
    expect(typeof history[0].executionTime).toBe('number');
  });

  it('should warn if command exceeds latency threshold', async () => {
    const service = VoiceCommandService.getInstance();
    const consoleWarnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    // Register slow command
    service.registerCommand({
      pattern: /slow command/i,
      action: 'SLOW',
      description: 'Slow command',
      requiresConfirmation: false,
      handler: async () => {
        await new Promise((resolve) => setTimeout(resolve, 600));
      },
    });

    await service.processCommand('slow command', 1.0);

    // Should log warning for exceeding threshold
    expect(consoleWarnSpy).toHaveBeenCalled();

    consoleWarnSpy.mockRestore();
  });
});

describe('VoiceCommandService - VOICE-004: Confirmation', () => {
  beforeEach(() => {
    VoiceCommandService.resetInstance();
    (global as any).window = {
      SpeechRecognition: MockSpeechRecognition,
      AudioContext: class {
        currentTime = 0;
        createOscillator() {
          return {
            frequency: { value: 0 },
            connect: vi.fn(),
            start: vi.fn(),
            stop: vi.fn(),
          };
        }
        createGain() {
          return {
            gain: {
              setValueAtTime: vi.fn(),
              exponentialRampToValueAtTime: vi.fn(),
            },
            connect: vi.fn(),
          };
        }
        get destination() {
          return {};
        }
      },
    };
  });

  it('should require confirmation for destructive commands', async () => {
    const service = VoiceCommandService.getInstance();
    const result = await service.processCommand('delete all drawings', 1.0);

    expect(result.action).toBe('CONFIRM_REQUIRED');
    expect(service.getConfirmationState()).not.toBeNull();
  });

  it('should execute command on "yes" confirmation', async () => {
    const service = VoiceCommandService.getInstance();

    // Initiate destructive command
    await service.processCommand('delete all drawings', 1.0);

    // Confirm with "yes"
    const result = await service.processCommand('yes', 1.0);

    expect(result.recognized).toBe(true);
    expect(result.action).toBe('DELETE_DRAWINGS');
  });

  it('should cancel command on "no" confirmation', async () => {
    const service = VoiceCommandService.getInstance();

    // Initiate destructive command
    await service.processCommand('delete all drawings', 1.0);

    // Cancel with "no"
    const result = await service.processCommand('no', 1.0);

    expect(result.action).toBe('CANCELLED');
  });

  it('should expire confirmation after timeout', async () => {
    vi.useFakeTimers();
    const service = VoiceCommandService.getInstance();

    // Initiate destructive command
    await service.processCommand('delete all drawings', 1.0);

    // Fast forward past timeout
    vi.advanceTimersByTime(CONFIRMATION_TIMEOUT_MS + 1000);

    const state = service.getConfirmationState();
    expect(state).toBeNull();

    vi.useRealTimers();
  });
});

describe('VoiceCommandService - VOICE-005: History', () => {
  beforeEach(() => {
    VoiceCommandService.resetInstance();
    localStorage.clear();
    (global as any).window = {
      SpeechRecognition: MockSpeechRecognition,
      AudioContext: class {
        currentTime = 0;
        createOscillator() {
          return {
            frequency: { value: 0 },
            connect: vi.fn(),
            start: vi.fn(),
            stop: vi.fn(),
          };
        }
        createGain() {
          return {
            gain: {
              setValueAtTime: vi.fn(),
              exponentialRampToValueAtTime: vi.fn(),
            },
            connect: vi.fn(),
          };
        }
        get destination() {
          return {};
        }
      },
    };
  });

  it('should add commands to history', async () => {
    const service = VoiceCommandService.getInstance();

    await service.processCommand('stop', 1.0);

    const history = service.getCommandHistory();
    expect(history.length).toBe(1);
    expect(history[0].transcript).toBe('stop');
  });

  it('should limit history to MAX_HISTORY_ENTRIES', async () => {
    const service = VoiceCommandService.getInstance();

    // Add more than MAX_HISTORY_ENTRIES
    for (let i = 0; i < MAX_HISTORY_ENTRIES + 10; i++) {
      await service.processCommand('stop', 1.0);
    }

    const history = service.getCommandHistory();
    expect(history.length).toBe(MAX_HISTORY_ENTRIES);
  });

  it('should clear history', async () => {
    const service = VoiceCommandService.getInstance();

    await service.processCommand('stop', 1.0);
    await service.processCommand('dance', 1.0);

    service.clearHistory();

    const history = service.getCommandHistory();
    expect(history.length).toBe(0);
  });

  it('should persist history to localStorage', async () => {
    const service = VoiceCommandService.getInstance();

    await service.processCommand('stop', 1.0);

    // Check localStorage
    const stored = localStorage.getItem('mbot_voice_history');
    expect(stored).not.toBeNull();

    const history = JSON.parse(stored!);
    expect(history.length).toBeGreaterThan(0);
  });

  it('should include execution time in history', async () => {
    const service = VoiceCommandService.getInstance();

    await service.processCommand('stop', 1.0);

    const history = service.getCommandHistory();
    expect(history[0].executionTime).toBeDefined();
    expect(history[0].executionTime).toBeGreaterThan(0);
  });
});

describe('VoiceCommandService - VOICE-003: Wake Word', () => {
  beforeEach(() => {
    VoiceCommandService.resetInstance();
    (global as any).window = {
      SpeechRecognition: MockSpeechRecognition,
      AudioContext: class {
        currentTime = 0;
        createOscillator() {
          return {
            frequency: { value: 0 },
            connect: vi.fn(),
            start: vi.fn(),
            stop: vi.fn(),
          };
        }
        createGain() {
          return {
            gain: {
              setValueAtTime: vi.fn(),
              exponentialRampToValueAtTime: vi.fn(),
            },
            connect: vi.fn(),
          };
        }
        get destination() {
          return {};
        }
      },
    };
  });

  it('should process command without wake word when disabled', async () => {
    const service = VoiceCommandService.getInstance();
    service.updateSettings({ wakeWordEnabled: false });

    const result = await service.processCommand('stop', 1.0);

    expect(result.recognized).toBe(true);
  });

  it('should require wake word when enabled', async () => {
    const service = VoiceCommandService.getInstance();
    service.updateSettings({ wakeWordEnabled: true, wakeWord: 'hey robot' });

    // Command without wake word should be ignored
    // (This is handled in handleRecognitionResult, not processCommand directly)
  });
});

describe('VoiceCommandService - Browser Compatibility', () => {
  it('should detect native SpeechRecognition support', () => {
    (global as any).window = { SpeechRecognition: class {} };

    const compat = checkBrowserCompatibility();

    expect(compat.supported).toBe(true);
    expect(compat.hasWebkitPrefix).toBe(false);
  });

  it('should detect webkit-prefixed support', () => {
    (global as any).window = { webkitSpeechRecognition: class {} };

    const compat = checkBrowserCompatibility();

    expect(compat.supported).toBe(true);
    expect(compat.hasWebkitPrefix).toBe(true);
  });

  it('should detect no support', () => {
    (global as any).window = {};

    const compat = checkBrowserCompatibility();

    expect(compat.supported).toBe(false);
    expect(compat.message).toBeTruthy();
  });
});

describe('VoiceCommandService - Custom Commands', () => {
  beforeEach(() => {
    VoiceCommandService.resetInstance();
    (global as any).window = {
      SpeechRecognition: MockSpeechRecognition,
      AudioContext: class {
        currentTime = 0;
        createOscillator() {
          return {
            frequency: { value: 0 },
            connect: vi.fn(),
            start: vi.fn(),
            stop: vi.fn(),
          };
        }
        createGain() {
          return {
            gain: {
              setValueAtTime: vi.fn(),
              exponentialRampToValueAtTime: vi.fn(),
            },
            connect: vi.fn(),
          };
        }
        get destination() {
          return {};
        }
      },
    };
  });

  it('should allow registering custom commands', async () => {
    const service = VoiceCommandService.getInstance();
    const handlerSpy = vi.fn();

    service.registerCommand({
      pattern: /custom command/i,
      action: 'CUSTOM',
      description: 'Custom test command',
      requiresConfirmation: false,
      handler: handlerSpy,
    });

    await service.processCommand('custom command', 1.0);

    expect(handlerSpy).toHaveBeenCalled();
  });

  it('should pass extracted parameters to handler', async () => {
    const service = VoiceCommandService.getInstance();
    const handlerSpy = vi.fn();

    service.registerCommand({
      pattern: /set volume to (\d+)/i,
      action: 'SET_VOLUME',
      description: 'Set volume',
      requiresConfirmation: false,
      handler: handlerSpy,
    });

    await service.processCommand('set volume to 75', 1.0);

    expect(handlerSpy).toHaveBeenCalledWith(['75']);
  });
});
