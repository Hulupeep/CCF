/**
 * Whisper API Integration
 *
 * Provides speech-to-text transcription using OpenAI's Whisper API
 */

export interface WhisperConfig {
  apiKey: string;
  model?: string;
  language?: string;
}

export interface TranscriptionResult {
  text: string;
  duration: number;
  language?: string;
  segments?: Array<{
    text: string;
    start: number;
    end: number;
  }>;
}

export class WhisperAPI {
  private apiKey: string;
  private model: string;
  private language: string;
  private baseUrl: string = 'https://api.openai.com/v1';

  constructor(config: WhisperConfig) {
    this.apiKey = config.apiKey;
    this.model = config.model || 'whisper-1';
    this.language = config.language || 'en';
  }

  /**
   * Transcribe audio to text
   */
  public async transcribeAudio(audioData: ArrayBuffer): Promise<string> {
    try {
      const formData = new FormData();
      const audioBlob = new Blob([audioData], { type: 'audio/webm' });
      formData.append('file', audioBlob, 'audio.webm');
      formData.append('model', this.model);
      formData.append('language', this.language);

      const response = await fetch(`${this.baseUrl}/audio/transcriptions`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${this.apiKey}`
        },
        body: formData
      });

      if (!response.ok) {
        throw new Error(`Whisper API error: ${response.statusText}`);
      }

      const result = await response.json();
      return result.text;
    } catch (error) {
      console.error('Transcription failed:', error);
      throw error;
    }
  }

  /**
   * Transcribe with detailed segments
   */
  public async transcribeDetailed(
    audioData: ArrayBuffer
  ): Promise<TranscriptionResult> {
    try {
      const formData = new FormData();
      const audioBlob = new Blob([audioData], { type: 'audio/webm' });
      formData.append('file', audioBlob, 'audio.webm');
      formData.append('model', this.model);
      formData.append('language', this.language);
      formData.append('response_format', 'verbose_json');

      const response = await fetch(`${this.baseUrl}/audio/transcriptions`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${this.apiKey}`
        },
        body: formData
      });

      if (!response.ok) {
        throw new Error(`Whisper API error: ${response.statusText}`);
      }

      const result = await response.json();

      return {
        text: result.text,
        duration: result.duration,
        language: result.language,
        segments: result.segments
      };
    } catch (error) {
      console.error('Detailed transcription failed:', error);
      throw error;
    }
  }

  /**
   * Translate audio to English
   */
  public async translateToEnglish(audioData: ArrayBuffer): Promise<string> {
    try {
      const formData = new FormData();
      const audioBlob = new Blob([audioData], { type: 'audio/webm' });
      formData.append('file', audioBlob, 'audio.webm');
      formData.append('model', this.model);

      const response = await fetch(`${this.baseUrl}/audio/translations`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${this.apiKey}`
        },
        body: formData
      });

      if (!response.ok) {
        throw new Error(`Whisper API error: ${response.statusText}`);
      }

      const result = await response.json();
      return result.text;
    } catch (error) {
      console.error('Translation failed:', error);
      throw error;
    }
  }

  /**
   * Check if API key is configured
   */
  public isConfigured(): boolean {
    return Boolean(this.apiKey);
  }

  /**
   * Update API key
   */
  public setApiKey(apiKey: string): void {
    this.apiKey = apiKey;
  }
}

export default WhisperAPI;
