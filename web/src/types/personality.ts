/**
 * Personality type definitions for CCF on RuVector
 * Based on feature_personality.yml contract
 */

/**
 * Core personality configuration with 9 parameters
 * All parameters MUST be in range [0.0, 1.0] per PERS-001 invariant
 */
export interface PersonalityConfig {
  // Baselines (where it wants to settle)
  tension_baseline: number;
  coherence_baseline: number;
  energy_baseline: number;

  // Reactivity (how stimuli affect it)
  startle_sensitivity: number;
  recovery_speed: number;
  curiosity_drive: number;

  // Expression (how it shows feelings)
  movement_expressiveness: number;
  sound_expressiveness: number;
  light_expressiveness: number;
}

/**
 * Full personality with metadata
 */
export interface Personality extends PersonalityConfig {
  id: string;
  name: string;
  icon: string;
  version: number;
  created_at: number;
  modified_at: number;
  quirks?: string[];
  sound_pack?: string;
}

/**
 * Preset personality configuration
 */
export interface PersonalityPreset {
  id: string;
  name: string;
  description: string;
  icon: string;
  config: PersonalityConfig;
}

/**
 * WebSocket message for personality updates
 */
export interface PersonalityUpdateMessage {
  type: 'personality_update';
  params: Partial<PersonalityConfig>;
}

/**
 * WebSocket connection status
 */
export type ConnectionStatus = 'connected' | 'disconnected' | 'connecting';

/**
 * Custom personality saved to localStorage
 */
export interface CustomPersonality {
  name: string;
  config: PersonalityConfig;
  created_at: number;
}

/**
 * Undo/redo history state
 */
export interface HistoryState {
  config: PersonalityConfig;
  timestamp: number;
}

/**
 * Parameter metadata for UI rendering
 */
export interface ParameterMetadata {
  key: keyof PersonalityConfig;
  label: string;
  description: string;
  category: 'baselines' | 'reactivity' | 'expression';
  icon: string;
}

/**
 * Validates that all personality parameters are within bounds [0.0, 1.0]
 * Enforces I-PERS-001 invariant
 */
export function validatePersonalityConfig(config: Partial<PersonalityConfig>): boolean {
  const keys: (keyof PersonalityConfig)[] = [
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

  for (const key of keys) {
    const value = config[key];
    if (value !== undefined) {
      if (typeof value !== 'number' || value < 0.0 || value > 1.0) {
        return false;
      }
    }
  }

  return true;
}

/**
 * Clamps a value to [0.0, 1.0] range
 * Enforces I-PERS-001 invariant
 */
export function clampParameter(value: number): number {
  return Math.max(0.0, Math.min(1.0, value));
}

/**
 * Creates a default personality configuration with safe, neutral values
 * Implements I-PERS-003 invariant
 */
export function createDefaultConfig(): PersonalityConfig {
  return {
    tension_baseline: 0.5,
    coherence_baseline: 0.5,
    energy_baseline: 0.5,
    startle_sensitivity: 0.5,
    recovery_speed: 0.5,
    curiosity_drive: 0.5,
    movement_expressiveness: 0.5,
    sound_expressiveness: 0.5,
    light_expressiveness: 0.5,
  };
}

/**
 * Parameter metadata for rendering
 */
export const PARAMETER_METADATA: ParameterMetadata[] = [
  {
    key: 'tension_baseline',
    label: 'Tension Baseline',
    description: 'How anxious vs. relaxed (0=zen, 1=stressed)',
    category: 'baselines',
    icon: '⚖️',
  },
  {
    key: 'coherence_baseline',
    label: 'Coherence Baseline',
    description: 'How coordinated its movements are (0=clumsy, 1=smooth)',
    category: 'baselines',
    icon: '⚖️',
  },
  {
    key: 'energy_baseline',
    label: 'Energy Baseline',
    description: 'How active it is (0=sleepy, 1=hyperactive)',
    category: 'baselines',
    icon: '⚖️',
  },
  {
    key: 'startle_sensitivity',
    label: 'Startle Sensitivity',
    description: 'How easily it gets scared (0=brave, 1=jumpy)',
    category: 'reactivity',
    icon: '⚡',
  },
  {
    key: 'recovery_speed',
    label: 'Recovery Speed',
    description: 'How quickly it calms down (0=holds grudges, 1=forgets fast)',
    category: 'reactivity',
    icon: '⚡',
  },
  {
    key: 'curiosity_drive',
    label: 'Curiosity Drive',
    description: 'How much it explores (0=cautious, 1=adventurous)',
    category: 'reactivity',
    icon: '⚡',
  },
  {
    key: 'movement_expressiveness',
    label: 'Movement Expressiveness',
    description: 'How much it moves to show emotions (0=subtle, 1=dramatic)',
    category: 'expression',
    icon: '🎭',
  },
  {
    key: 'sound_expressiveness',
    label: 'Sound Expressiveness',
    description: 'How much it uses sounds (0=quiet, 1=chatty)',
    category: 'expression',
    icon: '🎭',
  },
  {
    key: 'light_expressiveness',
    label: 'Light Expressiveness',
    description: 'How much it uses LEDs (0=dim, 1=bright)',
    category: 'expression',
    icon: '🎭',
  },
];
