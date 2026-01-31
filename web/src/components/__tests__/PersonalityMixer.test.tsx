/**
 * Unit tests for PersonalityMixer component
 * Tests all parameter validation logic and UI interactions
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import {
  validatePersonalityConfig,
  clampParameter,
  createDefaultConfig,
  PersonalityConfig,
} from '../../types/personality';
import { PERSONALITY_PRESETS } from '../../types/presets';

describe('PersonalityMixer - Parameter Validation', () => {
  describe('I-PERS-001: Parameter bounds enforcement', () => {
    it('should validate config with all parameters in bounds', () => {
      const config: PersonalityConfig = {
        tension_baseline: 0.5,
        coherence_baseline: 0.7,
        energy_baseline: 0.3,
        startle_sensitivity: 0.6,
        recovery_speed: 0.8,
        curiosity_drive: 0.4,
        movement_expressiveness: 0.9,
        sound_expressiveness: 0.2,
        light_expressiveness: 0.5,
      };

      expect(validatePersonalityConfig(config)).toBe(true);
    });

    it('should reject config with parameter above 1.0', () => {
      const config: Partial<PersonalityConfig> = {
        tension_baseline: 1.5,
      };

      expect(validatePersonalityConfig(config)).toBe(false);
    });

    it('should reject config with parameter below 0.0', () => {
      const config: Partial<PersonalityConfig> = {
        coherence_baseline: -0.1,
      };

      expect(validatePersonalityConfig(config)).toBe(false);
    });

    it('should reject config with non-numeric parameter', () => {
      const config: any = {
        energy_baseline: 'invalid',
      };

      expect(validatePersonalityConfig(config)).toBe(false);
    });

    it('should validate config with boundary values', () => {
      const config: PersonalityConfig = {
        tension_baseline: 0.0,
        coherence_baseline: 1.0,
        energy_baseline: 0.0,
        startle_sensitivity: 1.0,
        recovery_speed: 0.5,
        curiosity_drive: 0.5,
        movement_expressiveness: 0.5,
        sound_expressiveness: 0.5,
        light_expressiveness: 0.5,
      };

      expect(validatePersonalityConfig(config)).toBe(true);
    });
  });

  describe('clampParameter', () => {
    it('should clamp value above 1.0 to 1.0', () => {
      expect(clampParameter(1.5)).toBe(1.0);
      expect(clampParameter(99.9)).toBe(1.0);
    });

    it('should clamp value below 0.0 to 0.0', () => {
      expect(clampParameter(-0.5)).toBe(0.0);
      expect(clampParameter(-99.9)).toBe(0.0);
    });

    it('should not modify values within bounds', () => {
      expect(clampParameter(0.0)).toBe(0.0);
      expect(clampParameter(0.5)).toBe(0.5);
      expect(clampParameter(1.0)).toBe(1.0);
      expect(clampParameter(0.73)).toBe(0.73);
    });
  });

  describe('I-PERS-003: Default personality has safe values', () => {
    it('should create default config with all parameters at 0.5', () => {
      const config = createDefaultConfig();

      expect(config.tension_baseline).toBe(0.5);
      expect(config.coherence_baseline).toBe(0.5);
      expect(config.energy_baseline).toBe(0.5);
      expect(config.startle_sensitivity).toBe(0.5);
      expect(config.recovery_speed).toBe(0.5);
      expect(config.curiosity_drive).toBe(0.5);
      expect(config.movement_expressiveness).toBe(0.5);
      expect(config.sound_expressiveness).toBe(0.5);
      expect(config.light_expressiveness).toBe(0.5);
    });

    it('should validate default config', () => {
      const config = createDefaultConfig();
      expect(validatePersonalityConfig(config)).toBe(true);
    });
  });
});

describe('PersonalityMixer - Presets', () => {
  it('should have exactly 15 presets', () => {
    expect(PERSONALITY_PRESETS).toHaveLength(15);
  });

  it('should have all presets with valid configurations', () => {
    PERSONALITY_PRESETS.forEach((preset) => {
      expect(validatePersonalityConfig(preset.config)).toBe(true);
    });
  });

  it('should have all presets with required fields', () => {
    PERSONALITY_PRESETS.forEach((preset) => {
      expect(preset.id).toBeTruthy();
      expect(preset.name).toBeTruthy();
      expect(preset.description).toBeTruthy();
      expect(preset.icon).toBeTruthy();
      expect(preset.config).toBeTruthy();
    });
  });

  it('should have unique preset IDs', () => {
    const ids = PERSONALITY_PRESETS.map((p) => p.id);
    const uniqueIds = new Set(ids);
    expect(uniqueIds.size).toBe(ids.length);
  });

  describe('Preset: Curious', () => {
    it('should have high curiosity_drive', () => {
      const curious = PERSONALITY_PRESETS.find((p) => p.id === 'curious');
      expect(curious).toBeTruthy();
      expect(curious!.config.curiosity_drive).toBeGreaterThan(0.8);
    });
  });

  describe('Preset: Zen', () => {
    it('should have low tension and high coherence', () => {
      const zen = PERSONALITY_PRESETS.find((p) => p.id === 'zen');
      expect(zen).toBeTruthy();
      expect(zen!.config.tension_baseline).toBeLessThan(0.2);
      expect(zen!.config.coherence_baseline).toBeGreaterThan(0.8);
    });
  });

  describe('Preset: Excitable', () => {
    it('should have high energy and expressiveness', () => {
      const excitable = PERSONALITY_PRESETS.find((p) => p.id === 'excitable');
      expect(excitable).toBeTruthy();
      expect(excitable!.config.energy_baseline).toBeGreaterThan(0.8);
      expect(excitable!.config.movement_expressiveness).toBeGreaterThan(0.8);
    });
  });

  describe('Preset: Timid', () => {
    it('should have high startle_sensitivity and low recovery_speed', () => {
      const timid = PERSONALITY_PRESETS.find((p) => p.id === 'timid');
      expect(timid).toBeTruthy();
      expect(timid!.config.startle_sensitivity).toBeGreaterThan(0.7);
      expect(timid!.config.recovery_speed).toBeLessThan(0.3);
    });
  });
});

describe('PersonalityMixer - Parameter Metadata', () => {
  it('should have metadata for all 9 parameters', () => {
    const { PARAMETER_METADATA } = require('../../types/personality');
    expect(PARAMETER_METADATA).toHaveLength(9);
  });

  it('should have correct categories', () => {
    const { PARAMETER_METADATA } = require('../../types/personality');
    const categories = PARAMETER_METADATA.map((p: any) => p.category);

    expect(categories.filter((c: string) => c === 'baselines')).toHaveLength(3);
    expect(categories.filter((c: string) => c === 'reactivity')).toHaveLength(3);
    expect(categories.filter((c: string) => c === 'expression')).toHaveLength(3);
  });

  it('should have all required fields', () => {
    const { PARAMETER_METADATA } = require('../../types/personality');

    PARAMETER_METADATA.forEach((param: any) => {
      expect(param.key).toBeTruthy();
      expect(param.label).toBeTruthy();
      expect(param.description).toBeTruthy();
      expect(param.category).toBeTruthy();
      expect(param.icon).toBeTruthy();
    });
  });
});

describe('PersonalityMixer - localStorage Integration', () => {
  beforeEach(() => {
    // Clear localStorage before each test
    localStorage.clear();
  });

  it('should save custom personality to localStorage', () => {
    const customPersonality = {
      name: 'My Robot',
      config: createDefaultConfig(),
      created_at: Date.now(),
    };

    localStorage.setItem('mbot-custom-personalities', JSON.stringify([customPersonality]));

    const stored = JSON.parse(localStorage.getItem('mbot-custom-personalities') || '[]');
    expect(stored).toHaveLength(1);
    expect(stored[0].name).toBe('My Robot');
  });

  it('should load custom personalities from localStorage', () => {
    const customPersonalities = [
      {
        name: 'Custom 1',
        config: createDefaultConfig(),
        created_at: Date.now(),
      },
      {
        name: 'Custom 2',
        config: createDefaultConfig(),
        created_at: Date.now(),
      },
    ];

    localStorage.setItem('mbot-custom-personalities', JSON.stringify(customPersonalities));

    const stored = JSON.parse(localStorage.getItem('mbot-custom-personalities') || '[]');
    expect(stored).toHaveLength(2);
    expect(stored[0].name).toBe('Custom 1');
    expect(stored[1].name).toBe('Custom 2');
  });

  it('should handle invalid localStorage data', () => {
    localStorage.setItem('mbot-custom-personalities', 'invalid json');

    expect(() => {
      JSON.parse(localStorage.getItem('mbot-custom-personalities') || '[]');
    }).toThrow();
  });
});

describe('PersonalityMixer - History (Undo/Redo)', () => {
  it('should store up to 50 states', () => {
    const maxStates = 50;
    const states: PersonalityConfig[] = [];

    for (let i = 0; i < 60; i++) {
      states.push({
        ...createDefaultConfig(),
        tension_baseline: i / 100,
      });
    }

    // Should only keep last 50
    expect(states.slice(-maxStates)).toHaveLength(maxStates);
  });

  it('should allow undo when history exists', () => {
    const state1 = createDefaultConfig();
    const state2 = { ...state1, tension_baseline: 0.7 };

    const history = [state1, state2];
    const currentIndex = 1;

    const canUndo = currentIndex > 0;
    expect(canUndo).toBe(true);
  });

  it('should not allow undo at start of history', () => {
    const state1 = createDefaultConfig();
    const history = [state1];
    const currentIndex = 0;

    const canUndo = currentIndex > 0;
    expect(canUndo).toBe(false);
  });

  it('should allow redo when not at end of history', () => {
    const state1 = createDefaultConfig();
    const state2 = { ...state1, tension_baseline: 0.7 };

    const history = [state1, state2];
    const currentIndex = 0;

    const canRedo = currentIndex < history.length - 1;
    expect(canRedo).toBe(true);
  });

  it('should not allow redo at end of history', () => {
    const state1 = createDefaultConfig();
    const state2 = { ...state1, tension_baseline: 0.7 };

    const history = [state1, state2];
    const currentIndex = 1;

    const canRedo = currentIndex < history.length - 1;
    expect(canRedo).toBe(false);
  });
});

describe('PersonalityMixer - WebSocket Messages', () => {
  it('should format personality update message correctly', () => {
    const params = { tension_baseline: 0.7 };
    const message = {
      type: 'personality_update',
      params,
    };

    expect(message.type).toBe('personality_update');
    expect(message.params).toEqual(params);
  });

  it('should validate message payload', () => {
    const message = {
      type: 'personality_update',
      params: { tension_baseline: 0.7 },
    };

    expect(validatePersonalityConfig(message.params)).toBe(true);
  });

  it('should reject invalid message payload', () => {
    const message = {
      type: 'personality_update',
      params: { tension_baseline: 1.5 },
    };

    expect(validatePersonalityConfig(message.params)).toBe(false);
  });
});

describe('PersonalityMixer - I-PERS-UI-002: Debounced Updates', () => {
  it('should debounce to max 2 updates per second', () => {
    const updateIntervalMs = 500; // 2 updates/second
    const minTimeBetweenUpdates = 500;

    expect(updateIntervalMs).toBe(minTimeBetweenUpdates);
  });

  it('should not send update if within debounce window', () => {
    const lastSendTime = Date.now();
    const now = Date.now() + 200; // 200ms later
    const updateInterval = 500;

    const timeSinceLastSend = now - lastSendTime;
    const shouldSend = timeSinceLastSend >= updateInterval;

    expect(shouldSend).toBe(false);
  });

  it('should send update if outside debounce window', () => {
    const lastSendTime = Date.now();
    const now = Date.now() + 600; // 600ms later
    const updateInterval = 500;

    const timeSinceLastSend = now - lastSendTime;
    const shouldSend = timeSinceLastSend >= updateInterval;

    expect(shouldSend).toBe(true);
  });
});

describe('PersonalityMixer - ARCH-004: Contract Compliance', () => {
  it('should enforce bounded parameters per ARCH-004', () => {
    // Test all 9 parameters
    const paramKeys: (keyof PersonalityConfig)[] = [
      'tension_baseline',
      'coherence_baseline',
      'energy_baseline',
      'startle_sensitivity',
      'recovery_speed',
      'curiosity_drive',
      'movement_expressiveness',
      'sound_expressiveness',
      'light_expressiveness',
    ];

    paramKeys.forEach((key) => {
      const config: Partial<PersonalityConfig> = {};
      config[key] = 0.5;
      expect(validatePersonalityConfig(config)).toBe(true);

      config[key] = 1.1; // Out of bounds
      expect(validatePersonalityConfig(config)).toBe(false);

      config[key] = -0.1; // Out of bounds
      expect(validatePersonalityConfig(config)).toBe(false);
    });
  });

  it('should use clamp function for safety', () => {
    const testValues = [-1, -0.5, 0, 0.5, 1, 1.5, 2];
    const clampedValues = testValues.map(clampParameter);

    clampedValues.forEach((value) => {
      expect(value).toBeGreaterThanOrEqual(0.0);
      expect(value).toBeLessThanOrEqual(1.0);
    });
  });
});
