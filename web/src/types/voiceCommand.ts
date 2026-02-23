/**
 * Voice Command type definitions for CCF on RuVector
 * Based on feature_voice.yml contract
 * Invariants: I-VOICE-001, I-VOICE-002, I-VOICE-003
 */

/**
 * Voice command definition with pattern matching and handler
 */
export interface VoiceCommand {
  pattern: RegExp;
  action: string;
  description: string;
  requiresConfirmation: boolean;
  handler: (params: any) => Promise<void>;
}

/**
 * Result of command processing
 */
export interface CommandResult {
  recognized: boolean;
  confidence: number; // 0-1
  command?: string;
  params?: any;
  action?: string;
  error?: string;
}

/**
 * Voice recognition settings
 * Contract: VOICE-001, VOICE-003
 */
export interface VoiceSettings {
  enabled: boolean;
  wakeWordEnabled: boolean;
  wakeWord: string; // Default: "hey robot"
  language: string; // Default: "en-US"
  continuous: boolean;
  interimResults: boolean;
  audioFeedback: boolean;
  visualFeedback: boolean; // Always true
  noiseThreshold: number; // dB, default: 50
  confidenceThreshold: number; // 0-1, default: 0.7
}

/**
 * Voice command history entry
 * Contract: VOICE-005
 */
export interface VoiceCommandHistory {
  id: string;
  transcript: string;
  recognized: boolean;
  confidence: number;
  action?: string;
  timestamp: number;
  executionTime: number; // ms, must be <= 500ms per I-VOICE-001
}

/**
 * Voice command service interface
 */
export interface IVoiceCommandService {
  // Recognition control
  startListening(): Promise<void>;
  stopListening(): void;
  isListening(): boolean;

  // Command processing
  processCommand(transcript: string): Promise<CommandResult>;
  registerCommand(command: VoiceCommand): void;

  // Settings management
  getSettings(): VoiceSettings;
  updateSettings(settings: Partial<VoiceSettings>): void;

  // History
  getCommandHistory(): VoiceCommandHistory[];
  clearHistory(): void;
}

/**
 * Listening state for UI feedback
 */
export type ListeningState = 'idle' | 'listening' | 'processing' | 'error';

/**
 * Confirmation prompt state
 * Contract: VOICE-004
 */
export interface ConfirmationState {
  active: boolean;
  command: string;
  action: string;
  params?: any;
  expiresAt: number; // Timeout after 5 seconds
}

/**
 * Browser compatibility check result
 */
export interface BrowserCompatibility {
  supported: boolean;
  hasWebkitPrefix: boolean;
  message?: string;
}

/**
 * Create default voice settings
 * Enforces I-VOICE-002 (noise threshold) and contract defaults
 */
export function createDefaultVoiceSettings(): VoiceSettings {
  return {
    enabled: false,
    wakeWordEnabled: false,
    wakeWord: 'hey robot',
    language: 'en-US',
    continuous: true,
    interimResults: true,
    audioFeedback: false,
    visualFeedback: true,
    noiseThreshold: 50, // dB per I-VOICE-002
    confidenceThreshold: 0.7, // Per I-VOICE-002
  };
}

/**
 * Validate voice settings
 */
export function validateVoiceSettings(settings: Partial<VoiceSettings>): boolean {
  if (settings.noiseThreshold !== undefined) {
    if (settings.noiseThreshold < 0 || settings.noiseThreshold > 120) {
      return false;
    }
  }

  if (settings.confidenceThreshold !== undefined) {
    if (
      settings.confidenceThreshold < 0.0 ||
      settings.confidenceThreshold > 1.0
    ) {
      return false;
    }
  }

  if (settings.wakeWord !== undefined) {
    if (typeof settings.wakeWord !== 'string' || settings.wakeWord.trim() === '') {
      return false;
    }
  }

  return true;
}

/**
 * Check browser compatibility for Web Speech API
 * Contract: VOICE-001
 */
export function checkBrowserCompatibility(): BrowserCompatibility {
  const hasNativeSupport = 'SpeechRecognition' in window;
  const hasWebkitSupport = 'webkitSpeechRecognition' in window;

  if (hasNativeSupport) {
    return {
      supported: true,
      hasWebkitPrefix: false,
    };
  }

  if (hasWebkitSupport) {
    return {
      supported: true,
      hasWebkitPrefix: true,
      message: 'Using webkit-prefixed API',
    };
  }

  return {
    supported: false,
    hasWebkitPrefix: false,
    message: 'Voice commands not supported in this browser',
  };
}

/**
 * Storage keys for voice settings and history
 */
export const VOICE_STORAGE_KEYS = {
  SETTINGS: 'mbot_voice_settings',
  HISTORY: 'mbot_voice_history',
  VERSION: 'mbot_voice_version',
} as const;

export const VOICE_STORAGE_VERSION = 1;

/**
 * Maximum command history entries to keep
 */
export const MAX_HISTORY_ENTRIES = 50;

/**
 * Command timeout for confirmation prompts (5 seconds)
 */
export const CONFIRMATION_TIMEOUT_MS = 5000;

/**
 * Maximum command execution time per I-VOICE-001 (500ms)
 */
export const MAX_COMMAND_LATENCY_MS = 500;
