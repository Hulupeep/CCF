/**
 * Text-to-Speech Service
 *
 * Handles speech synthesis for briefing delivery
 * Supports multiple TTS providers (Web Speech API, ElevenLabs, Google TTS)
 */

export interface Voice {
  id: string;
  name: string;
  language: string;
  gender?: 'male' | 'female';
  provider: 'browser' | 'elevenlabs' | 'google' | 'azure';
}

export interface TTSOptions {
  voice?: string;
  rate?: number; // 0.1 to 10
  pitch?: number; // 0 to 2
  volume?: number; // 0 to 1
}

export class TTSService {
  private provider: 'browser' | 'elevenlabs' | 'google' | 'azure';
  private currentAudio: HTMLAudioElement | null = null;
  private synthesis: SpeechSynthesis | null = null;

  constructor(provider: 'browser' | 'elevenlabs' | 'google' | 'azure' = 'browser') {
    this.provider = provider;

    // Initialize Web Speech API if using browser
    if (provider === 'browser' && typeof window !== 'undefined' && 'speechSynthesis' in window) {
      this.synthesis = window.speechSynthesis;
    }
  }

  /**
   * Synthesize speech from text
   */
  async synthesizeSpeech(text: string, options: TTSOptions = {}): Promise<void> {
    if (this.provider === 'browser') {
      return this.synthesizeWithBrowser(text, options);
    } else {
      return this.synthesizeWithExternal(text, options);
    }
  }

  /**
   * Synthesize using Web Speech API
   */
  private async synthesizeWithBrowser(text: string, options: TTSOptions): Promise<void> {
    if (!this.synthesis) {
      console.error('Speech synthesis not available');
      return;
    }

    return new Promise((resolve, reject) => {
      const utterance = new SpeechSynthesisUtterance(text);

      // Apply options
      if (options.rate) utterance.rate = options.rate;
      if (options.pitch) utterance.pitch = options.pitch;
      if (options.volume) utterance.volume = options.volume;

      // Get voice if specified
      if (options.voice) {
        const voices = this.synthesis!.getVoices();
        const voice = voices.find(v => v.name === options.voice || v.voiceURI === options.voice);
        if (voice) {
          utterance.voice = voice;
        }
      }

      utterance.onend = () => resolve();
      utterance.onerror = (error) => reject(error);

      this.synthesis!.speak(utterance);
    });
  }

  /**
   * Synthesize using external service (mock for now)
   */
  private async synthesizeWithExternal(text: string, options: TTSOptions): Promise<void> {
    // Mock implementation - would call ElevenLabs, Google TTS, or Azure
    console.log(`[TTS ${this.provider}] Synthesizing: ${text.substring(0, 50)}...`);

    // Simulate API call
    await new Promise(resolve => setTimeout(resolve, 100));
  }

  /**
   * Play audio from buffer
   */
  async playAudio(audioData: ArrayBuffer): Promise<void> {
    return new Promise((resolve, reject) => {
      const blob = new Blob([audioData], { type: 'audio/wav' });
      const url = URL.createObjectURL(blob);

      this.currentAudio = new Audio(url);
      this.currentAudio.onended = () => {
        URL.revokeObjectURL(url);
        resolve();
      };
      this.currentAudio.onerror = (error) => {
        URL.revokeObjectURL(url);
        reject(error);
      };

      this.currentAudio.play();
    });
  }

  /**
   * Get available voices
   */
  async getAvailableVoices(): Promise<Voice[]> {
    if (this.provider === 'browser' && this.synthesis) {
      const voices = this.synthesis.getVoices();
      return voices.map(v => ({
        id: v.voiceURI,
        name: v.name,
        language: v.lang,
        provider: 'browser'
      }));
    }

    // Mock voices for other providers
    return [
      { id: 'elevenlabs-rachel', name: 'Rachel', language: 'en-US', gender: 'female', provider: 'elevenlabs' },
      { id: 'elevenlabs-adam', name: 'Adam', language: 'en-US', gender: 'male', provider: 'elevenlabs' },
      { id: 'google-en-us-wavenet-a', name: 'Google en-US A', language: 'en-US', gender: 'female', provider: 'google' },
      { id: 'azure-jenny', name: 'Jenny', language: 'en-US', gender: 'female', provider: 'azure' }
    ];
  }

  /**
   * Set voice for user
   */
  async setVoiceForUser(userId: string, voiceId: string): Promise<void> {
    localStorage.setItem(`tts_voice_${userId}`, voiceId);
  }

  /**
   * Get user's preferred voice
   */
  async getUserVoice(userId: string): Promise<string | null> {
    return localStorage.getItem(`tts_voice_${userId}`);
  }

  /**
   * Stop current speech
   */
  stop(): void {
    if (this.synthesis) {
      this.synthesis.cancel();
    }
    if (this.currentAudio) {
      this.currentAudio.pause();
      this.currentAudio = null;
    }
  }

  /**
   * Check if speaking
   */
  isSpeaking(): boolean {
    return this.synthesis ? this.synthesis.speaking : false;
  }
}
