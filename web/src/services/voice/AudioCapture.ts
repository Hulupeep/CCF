/**
 * Audio Capture Service
 *
 * Contract: I-VOICE-002 (Privacy Protection)
 * Handles microphone access, audio recording, and processing for voice recognition.
 */

export class AudioCapture {
  private mediaRecorder: MediaRecorder | null = null;
  private audioChunks: Blob[] = [];
  private stream: MediaStream | null = null;
  private audioContext: AudioContext | null = null;

  /**
   * Check if browser supports audio recording
   */
  public isSupported(): boolean {
    return !!(
      navigator.mediaDevices &&
      navigator.mediaDevices.getUserMedia &&
      window.MediaRecorder
    );
  }

  /**
   * Request microphone permission and get available devices
   */
  public async requestPermission(): Promise<boolean> {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      // Stop the stream immediately after permission check
      stream.getTracks().forEach(track => track.stop());
      return true;
    } catch (error) {
      console.error('Microphone permission denied:', error);
      return false;
    }
  }

  /**
   * Get list of available audio input devices
   */
  public async getAudioDevices(): Promise<MediaDeviceInfo[]> {
    try {
      const devices = await navigator.mediaDevices.enumerateDevices();
      return devices.filter(device => device.kind === 'audioinput');
    } catch (error) {
      console.error('Failed to enumerate audio devices:', error);
      return [];
    }
  }

  /**
   * Start recording audio from microphone
   */
  public async startRecording(deviceId?: string): Promise<void> {
    if (!this.isSupported()) {
      throw new Error('Audio recording not supported in this browser');
    }

    try {
      // Stop any existing recording
      await this.stopRecording();

      // Get audio stream
      const constraints: MediaStreamConstraints = {
        audio: deviceId ? { deviceId: { exact: deviceId } } : true
      };

      this.stream = await navigator.mediaDevices.getUserMedia(constraints);
      this.audioChunks = [];

      // Create MediaRecorder
      const mimeType = this.getSupportedMimeType();
      this.mediaRecorder = new MediaRecorder(this.stream, { mimeType });

      this.mediaRecorder.ondataavailable = (event) => {
        if (event.data.size > 0) {
          this.audioChunks.push(event.data);
        }
      };

      this.mediaRecorder.start(100); // Collect data every 100ms
    } catch (error) {
      throw new Error(`Failed to start recording: ${error}`);
    }
  }

  /**
   * Stop recording and return audio data
   */
  public async stopRecording(): Promise<ArrayBuffer> {
    return new Promise((resolve, reject) => {
      if (!this.mediaRecorder || this.mediaRecorder.state === 'inactive') {
        resolve(new ArrayBuffer(0));
        return;
      }

      this.mediaRecorder.onstop = async () => {
        try {
          // Stop all tracks
          if (this.stream) {
            this.stream.getTracks().forEach(track => track.stop());
            this.stream = null;
          }

          // Create audio blob
          const audioBlob = new Blob(this.audioChunks, {
            type: this.getSupportedMimeType()
          });

          // Process and return audio data
          const audioData = await this.processAudio(audioBlob);
          resolve(audioData);
        } catch (error) {
          reject(error);
        }
      };

      this.mediaRecorder.stop();
    });
  }

  /**
   * Check if currently recording
   */
  public isRecording(): boolean {
    return this.mediaRecorder !== null && this.mediaRecorder.state === 'recording';
  }

  /**
   * Get recording duration in milliseconds
   */
  public getRecordingDuration(): number {
    if (!this.audioChunks.length) return 0;

    // Estimate duration based on chunks (rough estimate)
    const totalSize = this.audioChunks.reduce((sum, chunk) => sum + chunk.size, 0);
    const bytesPerSecond = 16000; // Rough estimate for typical audio
    return (totalSize / bytesPerSecond) * 1000;
  }

  /**
   * Get audio level (for visualization)
   */
  public async getAudioLevel(): Promise<number> {
    if (!this.stream || !this.audioContext) {
      return 0;
    }

    try {
      const analyser = this.audioContext.createAnalyser();
      const source = this.audioContext.createMediaStreamSource(this.stream);
      source.connect(analyser);

      const dataArray = new Uint8Array(analyser.frequencyBinCount);
      analyser.getByteFrequencyData(dataArray);

      // Calculate average volume
      const average = dataArray.reduce((sum, value) => sum + value, 0) / dataArray.length;
      return average / 255; // Normalize to 0-1
    } catch (error) {
      console.error('Failed to get audio level:', error);
      return 0;
    }
  }

  /**
   * Convert audio blob to ArrayBuffer and apply processing
   */
  private async processAudio(audioBlob: Blob): Promise<ArrayBuffer> {
    return await audioBlob.arrayBuffer();
  }

  /**
   * Get supported MIME type for MediaRecorder
   */
  private getSupportedMimeType(): string {
    const types = [
      'audio/webm;codecs=opus',
      'audio/webm',
      'audio/ogg;codecs=opus',
      'audio/mp4',
      'audio/wav'
    ];

    for (const type of types) {
      if (MediaRecorder.isTypeSupported(type)) {
        return type;
      }
    }

    return 'audio/webm'; // Fallback
  }

  /**
   * Clean up resources
   */
  public dispose(): void {
    if (this.mediaRecorder && this.mediaRecorder.state !== 'inactive') {
      this.mediaRecorder.stop();
    }

    if (this.stream) {
      this.stream.getTracks().forEach(track => track.stop());
      this.stream = null;
    }

    if (this.audioContext) {
      this.audioContext.close();
      this.audioContext = null;
    }

    this.mediaRecorder = null;
    this.audioChunks = [];
  }
}

export default AudioCapture;
