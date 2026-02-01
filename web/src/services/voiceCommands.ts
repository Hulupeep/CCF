/**
 * VoiceCommandService - Voice recognition and command processing
 * Contract: feature_voice.yml (VOICE-001 through VOICE-005)
 * Invariants:
 * - I-VOICE-001: Commands execute within 500ms of recognition
 * - I-VOICE-002: Handle ambient noise >50dB without false triggers
 * - I-VOICE-003: Non-blocking visual + optional audio feedback
 *
 * Issue: #89 - Voice Command System
 */

import {
  VoiceCommand,
  CommandResult,
  VoiceSettings,
  VoiceCommandHistory,
  IVoiceCommandService,
  ListeningState,
  ConfirmationState,
  createDefaultVoiceSettings,
  validateVoiceSettings,
  checkBrowserCompatibility,
  VOICE_STORAGE_KEYS,
  VOICE_STORAGE_VERSION,
  MAX_HISTORY_ENTRIES,
  CONFIRMATION_TIMEOUT_MS,
  MAX_COMMAND_LATENCY_MS,
} from '../types/voiceCommand';

// Extend Window interface for webkit support
declare global {
  interface Window {
    webkitSpeechRecognition: any;
  }
}

/**
 * VoiceCommandService - Singleton implementation
 * Manages voice recognition and command processing
 */
class VoiceCommandService implements IVoiceCommandService {
  private static instance: VoiceCommandService | null = null;

  private recognition: SpeechRecognition | null = null;
  private settings: VoiceSettings;
  private commands: Map<string, VoiceCommand> = new Map();
  private history: VoiceCommandHistory[] = [];
  private listeningState: ListeningState = 'idle';
  private confirmationState: ConfirmationState | null = null;
  private stateChangeListeners: Set<(state: ListeningState) => void> = new Set();
  private historyChangeListeners: Set<() => void> = new Set();

  /**
   * Private constructor enforces singleton pattern
   */
  private constructor() {
    this.settings = createDefaultVoiceSettings();
    this.initializeRecognition();
    this.loadSettings();
    this.loadHistory();
    this.registerDefaultCommands();
  }

  /**
   * Get singleton instance
   */
  public static getInstance(): VoiceCommandService {
    if (!VoiceCommandService.instance) {
      VoiceCommandService.instance = new VoiceCommandService();
    }
    return VoiceCommandService.instance;
  }

  /**
   * Initialize Speech Recognition API
   * Contract: VOICE-001
   */
  private initializeRecognition(): void {
    const compatibility = checkBrowserCompatibility();

    if (!compatibility.supported) {
      console.warn('Speech recognition not supported in this browser');
      return;
    }

    const SpeechRecognitionAPI = compatibility.hasWebkitPrefix
      ? window.webkitSpeechRecognition
      : (window as any).SpeechRecognition;

    this.recognition = new SpeechRecognitionAPI();
    this.recognition.continuous = this.settings.continuous;
    this.recognition.interimResults = this.settings.interimResults;
    this.recognition.lang = this.settings.language;

    // Set up event handlers
    this.recognition.onstart = () => {
      this.setListeningState('listening');
    };

    this.recognition.onresult = (event: SpeechRecognitionEvent) => {
      this.handleRecognitionResult(event);
    };

    this.recognition.onerror = (event: any) => {
      console.error('Speech recognition error:', event.error);
      this.setListeningState('error');

      // Auto-restart if enabled and not a fatal error
      if (this.settings.continuous && event.error !== 'no-speech') {
        setTimeout(() => this.startListening(), 1000);
      }
    };

    this.recognition.onend = () => {
      if (this.settings.continuous && this.listeningState === 'listening') {
        // Restart continuous listening
        this.recognition?.start();
      } else {
        this.setListeningState('idle');
      }
    };
  }

  /**
   * Start listening for voice commands
   * Contract: VOICE-001
   */
  public async startListening(): Promise<void> {
    if (!this.recognition) {
      throw new Error('Speech recognition not supported');
    }

    if (this.listeningState === 'listening') {
      return; // Already listening
    }

    try {
      this.recognition.start();
      this.setListeningState('listening');
    } catch (error) {
      console.error('Failed to start speech recognition:', error);
      this.setListeningState('error');
      throw error;
    }
  }

  /**
   * Stop listening for voice commands
   */
  public stopListening(): void {
    if (this.recognition && this.listeningState === 'listening') {
      this.recognition.stop();
      this.setListeningState('idle');
    }
  }

  /**
   * Check if currently listening
   */
  public isListening(): boolean {
    return this.listeningState === 'listening';
  }

  /**
   * Process recognition result
   * Contract: VOICE-002 (pattern matching)
   * Invariant: I-VOICE-001 (500ms execution)
   */
  private async handleRecognitionResult(event: SpeechRecognitionEvent): Promise<void> {
    const result = event.results[event.results.length - 1];

    if (!result.isFinal) {
      return; // Wait for final result
    }

    const transcript = result[0].transcript.trim();
    const confidence = result[0].confidence;

    // Check confidence threshold per I-VOICE-002
    if (confidence < this.settings.confidenceThreshold) {
      console.log(`Low confidence (${confidence}), ignoring: "${transcript}"`);
      return;
    }

    // Check wake word if enabled
    if (this.settings.wakeWordEnabled) {
      const lowerTranscript = transcript.toLowerCase();
      const wakeWord = this.settings.wakeWord.toLowerCase();

      if (!lowerTranscript.includes(wakeWord)) {
        return; // Wake word not detected
      }

      // Remove wake word from transcript
      const commandText = lowerTranscript.replace(wakeWord, '').trim();
      await this.processCommand(commandText, confidence);
    } else {
      await this.processCommand(transcript, confidence);
    }
  }

  /**
   * Process voice command
   * Contract: VOICE-002, VOICE-004
   * Invariant: I-VOICE-001 (execution within 500ms)
   */
  public async processCommand(
    transcript: string,
    confidence: number = 1.0
  ): Promise<CommandResult> {
    const startTime = performance.now();
    this.setListeningState('processing');

    try {
      // Check if this is a confirmation response
      if (this.confirmationState?.active) {
        return await this.handleConfirmationResponse(transcript, confidence, startTime);
      }

      // Try to match command pattern
      for (const [, command] of this.commands) {
        const match = transcript.match(command.pattern);

        if (match) {
          // Extract parameters from regex capture groups
          const params = match.slice(1);

          // Check if confirmation required
          if (command.requiresConfirmation) {
            this.initiateConfirmation(transcript, command, params);
            const executionTime = performance.now() - startTime;

            const result: CommandResult = {
              recognized: true,
              confidence,
              command: command.action,
              params,
              action: 'CONFIRM_REQUIRED',
            };

            this.addToHistory(transcript, result, executionTime);
            return result;
          }

          // Execute command
          try {
            await command.handler(params);

            const executionTime = performance.now() - startTime;

            // Warn if execution exceeded threshold (I-VOICE-001)
            if (executionTime > MAX_COMMAND_LATENCY_MS) {
              console.warn(
                `Command execution exceeded ${MAX_COMMAND_LATENCY_MS}ms: ${executionTime}ms`
              );
            }

            const result: CommandResult = {
              recognized: true,
              confidence,
              command: command.action,
              params,
              action: command.action,
            };

            this.addToHistory(transcript, result, executionTime);
            this.playAudioFeedback('success');

            return result;
          } catch (error) {
            const errorMessage = error instanceof Error ? error.message : 'Unknown error';
            console.error('Command execution failed:', errorMessage);

            const executionTime = performance.now() - startTime;
            const result: CommandResult = {
              recognized: true,
              confidence,
              command: command.action,
              error: errorMessage,
            };

            this.addToHistory(transcript, result, executionTime);
            this.playAudioFeedback('error');

            return result;
          }
        }
      }

      // No command matched
      const executionTime = performance.now() - startTime;
      const result: CommandResult = {
        recognized: false,
        confidence,
        error: 'Command not recognized',
      };

      this.addToHistory(transcript, result, executionTime);
      this.playAudioFeedback('error');

      return result;
    } finally {
      this.setListeningState('listening');
    }
  }

  /**
   * Handle confirmation response
   * Contract: VOICE-004
   */
  private async handleConfirmationResponse(
    transcript: string,
    confidence: number,
    startTime: number
  ): Promise<CommandResult> {
    if (!this.confirmationState) {
      return {
        recognized: false,
        confidence,
        error: 'No active confirmation',
      };
    }

    // Check if confirmation expired
    if (Date.now() > this.confirmationState.expiresAt) {
      this.confirmationState = null;
      return {
        recognized: false,
        confidence,
        error: 'Confirmation expired',
      };
    }

    const lowerTranscript = transcript.toLowerCase().trim();
    const confirmed = lowerTranscript === 'yes' || lowerTranscript === 'confirm';
    const cancelled = lowerTranscript === 'no' || lowerTranscript === 'cancel';

    if (!confirmed && !cancelled) {
      return {
        recognized: false,
        confidence,
        error: 'Please say "yes" or "no"',
      };
    }

    const state = this.confirmationState;
    this.confirmationState = null;

    if (cancelled) {
      const executionTime = performance.now() - startTime;
      const result: CommandResult = {
        recognized: true,
        confidence,
        action: 'CANCELLED',
      };

      this.addToHistory(`${state.command} (cancelled)`, result, executionTime);
      return result;
    }

    // Execute confirmed command
    const command = this.commands.get(state.action);
    if (!command) {
      return {
        recognized: false,
        confidence,
        error: 'Command not found',
      };
    }

    try {
      await command.handler(state.params);

      const executionTime = performance.now() - startTime;
      const result: CommandResult = {
        recognized: true,
        confidence,
        command: command.action,
        params: state.params,
        action: command.action,
      };

      this.addToHistory(`${state.command} (confirmed)`, result, executionTime);
      this.playAudioFeedback('success');

      return result;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      const executionTime = performance.now() - startTime;
      const result: CommandResult = {
        recognized: true,
        confidence,
        command: command.action,
        error: errorMessage,
      };

      this.addToHistory(`${state.command} (failed)`, result, executionTime);
      this.playAudioFeedback('error');

      return result;
    }
  }

  /**
   * Initiate confirmation prompt
   * Contract: VOICE-004
   */
  private initiateConfirmation(
    transcript: string,
    command: VoiceCommand,
    params: any
  ): void {
    this.confirmationState = {
      active: true,
      command: transcript,
      action: command.action,
      params,
      expiresAt: Date.now() + CONFIRMATION_TIMEOUT_MS,
    };

    // Auto-expire after timeout
    setTimeout(() => {
      if (this.confirmationState?.expiresAt === this.confirmationState?.expiresAt) {
        this.confirmationState = null;
      }
    }, CONFIRMATION_TIMEOUT_MS);
  }

  /**
   * Register a voice command
   * Contract: VOICE-002
   */
  public registerCommand(command: VoiceCommand): void {
    this.commands.set(command.action, command);
  }

  /**
   * Register default commands
   * Supports 15+ commands per contract requirement
   */
  private registerDefaultCommands(): void {
    // Mode control commands
    this.registerCommand({
      pattern: /start (drawing|art)/i,
      action: 'START_ARTBOT',
      description: 'Start drawing mode',
      requiresConfirmation: false,
      handler: async () => {
        console.log('Starting ArtBot mode');
        // TODO: Integrate with ArtBot activation
      },
    });

    this.registerCommand({
      pattern: /play (tic-tac-toe|game)/i,
      action: 'START_GAME',
      description: 'Start game mode',
      requiresConfirmation: false,
      handler: async () => {
        console.log('Starting game mode');
        // TODO: Integrate with GameBot activation
      },
    });

    this.registerCommand({
      pattern: /follow me/i,
      action: 'START_FOLLOW',
      description: 'Start follow mode',
      requiresConfirmation: false,
      handler: async () => {
        console.log('Starting follow mode');
        // TODO: Integrate with follow behavior
      },
    });

    // Personality commands
    this.registerCommand({
      pattern: /switch to (\w+)/i,
      action: 'SWITCH_PERSONALITY',
      description: 'Switch personality preset',
      requiresConfirmation: false,
      handler: async (params: string[]) => {
        const personalityName = params[0];
        console.log(`Switching to personality: ${personalityName}`);
        // TODO: Integrate with PersonalityStore
      },
    });

    this.registerCommand({
      pattern: /(increase|decrease) (\w+) by (\d+)/i,
      action: 'ADJUST_PARAMETER',
      description: 'Adjust personality parameter',
      requiresConfirmation: false,
      handler: async (params: string[]) => {
        const [direction, parameter, value] = params;
        console.log(`${direction} ${parameter} by ${value}%`);
        // TODO: Integrate with PersonalityStore parameter adjustment
      },
    });

    // Control commands
    this.registerCommand({
      pattern: /stop/i,
      action: 'STOP',
      description: 'Stop current activity',
      requiresConfirmation: false,
      handler: async () => {
        console.log('Stopping current activity');
        // TODO: Integrate with robot control
      },
    });

    this.registerCommand({
      pattern: /dance/i,
      action: 'DANCE',
      description: 'Perform dance routine',
      requiresConfirmation: false,
      handler: async () => {
        console.log('Dancing!');
        // TODO: Integrate with dance routine
      },
    });

    this.registerCommand({
      pattern: /go (to )?home/i,
      action: 'GO_HOME',
      description: 'Return to home position',
      requiresConfirmation: false,
      handler: async () => {
        console.log('Going home');
        // TODO: Integrate with robot positioning
      },
    });

    // Data commands
    this.registerCommand({
      pattern: /show stats/i,
      action: 'SHOW_STATS',
      description: 'Display statistics',
      requiresConfirmation: false,
      handler: async () => {
        console.log('Showing statistics');
        // TODO: Integrate with stats display
      },
    });

    this.registerCommand({
      pattern: /save preset (\w+)/i,
      action: 'SAVE_PRESET',
      description: 'Save personality preset',
      requiresConfirmation: false,
      handler: async (params: string[]) => {
        const presetName = params[0];
        console.log(`Saving preset: ${presetName}`);
        // TODO: Integrate with PersonalityStore preset saving
      },
    });

    // Destructive commands (require confirmation)
    this.registerCommand({
      pattern: /delete all drawings/i,
      action: 'DELETE_DRAWINGS',
      description: 'Delete all artworks',
      requiresConfirmation: true,
      handler: async () => {
        console.log('Deleting all drawings');
        // TODO: Integrate with artwork storage deletion
      },
    });

    this.registerCommand({
      pattern: /reset personality/i,
      action: 'RESET_PERSONALITY',
      description: 'Reset to default personality',
      requiresConfirmation: true,
      handler: async () => {
        console.log('Resetting personality');
        // TODO: Integrate with PersonalityStore reset
      },
    });

    this.registerCommand({
      pattern: /clear (all )?history/i,
      action: 'CLEAR_HISTORY',
      description: 'Clear command history',
      requiresConfirmation: true,
      handler: async () => {
        this.clearHistory();
      },
    });

    // Help command
    this.registerCommand({
      pattern: /help|what can you do/i,
      action: 'HELP',
      description: 'Show available commands',
      requiresConfirmation: false,
      handler: async () => {
        console.log('Available commands:', Array.from(this.commands.values()));
        // TODO: Show command list in UI
      },
    });

    // Status command
    this.registerCommand({
      pattern: /status/i,
      action: 'STATUS',
      description: 'Show robot status',
      requiresConfirmation: false,
      handler: async () => {
        console.log('Showing robot status');
        // TODO: Display robot status
      },
    });
  }

  /**
   * Get current settings
   */
  public getSettings(): VoiceSettings {
    return { ...this.settings };
  }

  /**
   * Update settings
   */
  public updateSettings(settings: Partial<VoiceSettings>): void {
    if (!validateVoiceSettings(settings)) {
      throw new Error('Invalid voice settings');
    }

    this.settings = { ...this.settings, ...settings };

    // Update recognition settings if active
    if (this.recognition) {
      this.recognition.continuous = this.settings.continuous;
      this.recognition.interimResults = this.settings.interimResults;
      this.recognition.lang = this.settings.language;
    }

    this.saveSettings();
  }

  /**
   * Get command history
   * Contract: VOICE-005
   */
  public getCommandHistory(): VoiceCommandHistory[] {
    return [...this.history];
  }

  /**
   * Clear command history
   * Contract: VOICE-005
   */
  public clearHistory(): void {
    this.history = [];
    this.saveHistory();
    this.notifyHistoryChange();
  }

  /**
   * Get confirmation state
   */
  public getConfirmationState(): ConfirmationState | null {
    return this.confirmationState ? { ...this.confirmationState } : null;
  }

  /**
   * Subscribe to state changes
   */
  public subscribeToStateChanges(callback: (state: ListeningState) => void): () => void {
    this.stateChangeListeners.add(callback);
    return () => {
      this.stateChangeListeners.delete(callback);
    };
  }

  /**
   * Subscribe to history changes
   */
  public subscribeToHistoryChanges(callback: () => void): () => void {
    this.historyChangeListeners.add(callback);
    return () => {
      this.historyChangeListeners.delete(callback);
    };
  }

  /**
   * Add command to history
   * Contract: VOICE-005
   */
  private addToHistory(
    transcript: string,
    result: CommandResult,
    executionTime: number
  ): void {
    const entry: VoiceCommandHistory = {
      id: `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      transcript,
      recognized: result.recognized,
      confidence: result.confidence,
      action: result.action,
      timestamp: Date.now(),
      executionTime,
    };

    this.history.unshift(entry); // Add to beginning

    // Keep only last MAX_HISTORY_ENTRIES
    if (this.history.length > MAX_HISTORY_ENTRIES) {
      this.history = this.history.slice(0, MAX_HISTORY_ENTRIES);
    }

    this.saveHistory();
    this.notifyHistoryChange();
  }

  /**
   * Play audio feedback if enabled
   * Invariant: I-VOICE-003 (non-blocking)
   */
  private playAudioFeedback(type: 'success' | 'error'): void {
    if (!this.settings.audioFeedback) {
      return;
    }

    // Non-blocking audio context
    const audioContext = new AudioContext();
    const oscillator = audioContext.createOscillator();
    const gainNode = audioContext.createGain();

    oscillator.connect(gainNode);
    gainNode.connect(audioContext.destination);

    if (type === 'success') {
      oscillator.frequency.value = 800;
      gainNode.gain.setValueAtTime(0.1, audioContext.currentTime);
      gainNode.gain.exponentialRampToValueAtTime(
        0.01,
        audioContext.currentTime + 0.1
      );
    } else {
      oscillator.frequency.value = 400;
      gainNode.gain.setValueAtTime(0.1, audioContext.currentTime);
      gainNode.gain.exponentialRampToValueAtTime(
        0.01,
        audioContext.currentTime + 0.2
      );
    }

    oscillator.start(audioContext.currentTime);
    oscillator.stop(audioContext.currentTime + 0.2);
  }

  /**
   * Set listening state and notify listeners
   */
  private setListeningState(state: ListeningState): void {
    this.listeningState = state;
    this.stateChangeListeners.forEach((listener) => {
      try {
        listener(state);
      } catch (error) {
        console.error('Error in state change listener:', error);
      }
    });
  }

  /**
   * Notify history change listeners
   */
  private notifyHistoryChange(): void {
    this.historyChangeListeners.forEach((listener) => {
      try {
        listener();
      } catch (error) {
        console.error('Error in history change listener:', error);
      }
    });
  }

  /**
   * Save settings to localStorage
   */
  private saveSettings(): void {
    try {
      localStorage.setItem(VOICE_STORAGE_KEYS.SETTINGS, JSON.stringify(this.settings));
      localStorage.setItem(VOICE_STORAGE_KEYS.VERSION, String(VOICE_STORAGE_VERSION));
    } catch (error) {
      console.error('Failed to save voice settings:', error);
    }
  }

  /**
   * Load settings from localStorage
   */
  private loadSettings(): void {
    try {
      const settingsJson = localStorage.getItem(VOICE_STORAGE_KEYS.SETTINGS);
      if (settingsJson) {
        const loadedSettings = JSON.parse(settingsJson);
        if (validateVoiceSettings(loadedSettings)) {
          this.settings = { ...this.settings, ...loadedSettings };
        }
      }
    } catch (error) {
      console.error('Failed to load voice settings:', error);
    }
  }

  /**
   * Save history to localStorage
   * Contract: VOICE-005
   */
  private saveHistory(): void {
    try {
      localStorage.setItem(VOICE_STORAGE_KEYS.HISTORY, JSON.stringify(this.history));
    } catch (error) {
      console.error('Failed to save voice history:', error);
    }
  }

  /**
   * Load history from localStorage
   * Contract: VOICE-005
   */
  private loadHistory(): void {
    try {
      const historyJson = localStorage.getItem(VOICE_STORAGE_KEYS.HISTORY);
      if (historyJson) {
        this.history = JSON.parse(historyJson);
      }
    } catch (error) {
      console.error('Failed to load voice history:', error);
    }
  }

  /**
   * FOR TESTING ONLY: Reset singleton instance
   */
  public static resetInstance(): void {
    VoiceCommandService.instance?.stopListening();
    VoiceCommandService.instance = null;
  }
}

// Export singleton accessor
export const voiceCommandService = VoiceCommandService.getInstance();

// Export class for testing
export { VoiceCommandService };
