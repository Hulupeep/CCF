/**
 * Voice Profile Service
 *
 * Contract: I-VOICE-001 (Speaker Identification ≥85%)
 * Contract: I-VOICE-005 (Multi-User Support - 10 profiles)
 * Contract: I-VOICE-006 (Graceful Fallback - Anonymous mode)
 *
 * Manages voice enrollment, speaker identification, and profile management.
 */

import { VoiceProfile, VoiceSample, VoiceIdentification } from '../../types/voice';

const CONFIDENCE_THRESHOLD = 0.85; // I-VOICE-001: ≥85% confidence
const MAX_PROFILES = 10; // I-VOICE-005: Support 10 profiles
const STORAGE_KEY = 'mbot_voice_profiles';

export class VoiceProfileService {
  private profiles: Map<string, VoiceProfile> = new Map();
  private initialized: boolean = false;

  constructor() {
    this.loadProfiles();
  }

  /**
   * Enroll a new user with voice samples
   * Requires at least 3 samples for reliable voiceprint
   */
  public async enrollUser(
    name: string,
    samples: VoiceSample[]
  ): Promise<VoiceProfile> {
    if (samples.length < 3) {
      throw new Error('At least 3 voice samples required for enrollment');
    }

    if (this.profiles.size >= MAX_PROFILES) {
      throw new Error(`Maximum ${MAX_PROFILES} voice profiles supported`);
    }

    const userId = this.generateUserId(name);

    // Extract voiceprint from samples
    const voiceprint = await this.extractVoiceprint(samples);

    const profile: VoiceProfile = {
      userId,
      name,
      voiceSamples: samples,
      voiceprint,
      confidence: 1.0,
      enrolledAt: Date.now(),
      lastUsed: Date.now(),
      useCount: 0,
      metadata: {
        isChild: false,
        language: 'en-US'
      }
    };

    this.profiles.set(userId, profile);
    await this.saveProfiles();

    return profile;
  }

  /**
   * Add additional voice sample to existing profile
   * Improves recognition accuracy over time
   */
  public async addVoiceSample(
    userId: string,
    sample: VoiceSample
  ): Promise<void> {
    const profile = this.profiles.get(userId);
    if (!profile) {
      throw new Error(`Profile not found: ${userId}`);
    }

    profile.voiceSamples.push(sample);

    // Re-extract voiceprint with new sample
    profile.voiceprint = await this.extractVoiceprint(profile.voiceSamples);

    await this.saveProfiles();
  }

  /**
   * Identify user from audio data
   * Returns identification with confidence score
   */
  public async identifyUser(
    audioData: ArrayBuffer
  ): Promise<VoiceIdentification> {
    if (this.profiles.size === 0) {
      // No profiles enrolled - anonymous mode (I-VOICE-006)
      return {
        userId: null,
        confidence: 0,
        alternativeMatches: [],
        isAnonymous: true,
        timestamp: Date.now()
      };
    }

    try {
      // Extract voiceprint from input audio
      const inputVoiceprint = await this.extractVoiceprintFromAudio(audioData);

      // Find best match
      const identification = this.findBestMatch(inputVoiceprint);

      // Update profile if match found
      if (identification.userId && identification.confidence >= CONFIDENCE_THRESHOLD) {
        const profile = this.profiles.get(identification.userId);
        if (profile) {
          profile.lastUsed = Date.now();
          profile.useCount++;
          await this.saveProfiles();
        }
      }

      return identification;
    } catch (error) {
      console.error('Voice identification failed:', error);

      // Fallback to anonymous mode
      return {
        userId: null,
        confidence: 0,
        alternativeMatches: [],
        isAnonymous: true,
        timestamp: Date.now()
      };
    }
  }

  /**
   * Verify if audio matches a specific user
   */
  public async verifyUser(
    userId: string,
    audioData: ArrayBuffer
  ): Promise<boolean> {
    const profile = this.profiles.get(userId);
    if (!profile) {
      return false;
    }

    try {
      const inputVoiceprint = await this.extractVoiceprintFromAudio(audioData);
      const similarity = this.calculateSimilarity(
        inputVoiceprint,
        profile.voiceprint
      );

      return similarity >= CONFIDENCE_THRESHOLD;
    } catch (error) {
      console.error('Voice verification failed:', error);
      return false;
    }
  }

  /**
   * Get profile by user ID
   */
  public async getProfile(userId: string): Promise<VoiceProfile | null> {
    return this.profiles.get(userId) || null;
  }

  /**
   * Get all enrolled profiles
   */
  public async getAllProfiles(): Promise<VoiceProfile[]> {
    return Array.from(this.profiles.values());
  }

  /**
   * Update profile information
   */
  public async updateProfile(
    userId: string,
    updates: Partial<VoiceProfile>
  ): Promise<void> {
    const profile = this.profiles.get(userId);
    if (!profile) {
      throw new Error(`Profile not found: ${userId}`);
    }

    Object.assign(profile, updates);
    await this.saveProfiles();
  }

  /**
   * Delete profile
   */
  public async deleteProfile(userId: string): Promise<void> {
    this.profiles.delete(userId);
    await this.saveProfiles();
  }

  /**
   * Extract voiceprint from multiple samples
   * This is a simplified implementation - in production, use a proper
   * speaker recognition model like Resemblyzer or SpeechBrain
   */
  private async extractVoiceprint(samples: VoiceSample[]): Promise<Float32Array> {
    // For now, create a simple feature vector
    // In production, this should use a proper voice embedding model
    const features = new Float32Array(128); // 128-dimensional embedding

    // Placeholder: Extract simple audio features
    // Real implementation would use MFCC, spectral features, etc.
    for (let i = 0; i < samples.length && i < features.length; i++) {
      const sample = samples[i];
      // Simple hash of audio characteristics
      features[i] = (sample.duration * sample.sampleRate) % 256 / 256;
    }

    return features;
  }

  /**
   * Extract voiceprint from single audio buffer
   */
  private async extractVoiceprintFromAudio(
    audioData: ArrayBuffer
  ): Promise<Float32Array> {
    // Simplified implementation
    // Real implementation would:
    // 1. Decode audio to PCM
    // 2. Extract MFCC or other features
    // 3. Run through neural network for embedding
    const features = new Float32Array(128);

    // Placeholder feature extraction
    const view = new Uint8Array(audioData);
    for (let i = 0; i < Math.min(view.length, features.length); i++) {
      features[i] = view[i] / 255;
    }

    return features;
  }

  /**
   * Calculate similarity between two voiceprints
   * Uses cosine similarity
   */
  private calculateSimilarity(
    print1: Float32Array,
    print2: Float32Array
  ): number {
    if (print1.length !== print2.length) {
      throw new Error('Voiceprint dimensions must match');
    }

    let dotProduct = 0;
    let norm1 = 0;
    let norm2 = 0;

    for (let i = 0; i < print1.length; i++) {
      dotProduct += print1[i] * print2[i];
      norm1 += print1[i] * print1[i];
      norm2 += print2[i] * print2[i];
    }

    norm1 = Math.sqrt(norm1);
    norm2 = Math.sqrt(norm2);

    if (norm1 === 0 || norm2 === 0) {
      return 0;
    }

    return dotProduct / (norm1 * norm2);
  }

  /**
   * Find best matching profile for voiceprint
   */
  private findBestMatch(voiceprint: Float32Array): VoiceIdentification {
    const matches: Array<{ userId: string; confidence: number }> = [];

    for (const [userId, profile] of this.profiles) {
      const confidence = this.calculateSimilarity(voiceprint, profile.voiceprint);
      matches.push({ userId, confidence });
    }

    // Sort by confidence descending
    matches.sort((a, b) => b.confidence - a.confidence);

    const bestMatch = matches[0];
    const alternativeMatches = matches.slice(1, 4); // Top 3 alternatives

    // Check if best match meets threshold
    const isAnonymous = !bestMatch || bestMatch.confidence < CONFIDENCE_THRESHOLD;

    return {
      userId: isAnonymous ? null : bestMatch.userId,
      confidence: bestMatch ? bestMatch.confidence : 0,
      alternativeMatches,
      isAnonymous,
      timestamp: Date.now()
    };
  }

  /**
   * Generate unique user ID
   */
  private generateUserId(name: string): string {
    const timestamp = Date.now();
    const random = Math.floor(Math.random() * 1000);
    return `${name.toLowerCase().replace(/\s+/g, '-')}-${timestamp}-${random}`;
  }

  /**
   * Load profiles from localStorage
   */
  private loadProfiles(): void {
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (stored) {
        const data = JSON.parse(stored);
        for (const profile of data) {
          // Reconstruct Float32Array from stored data
          profile.voiceprint = new Float32Array(profile.voiceprint);
          this.profiles.set(profile.userId, profile);
        }
      }
      this.initialized = true;
    } catch (error) {
      console.error('Failed to load voice profiles:', error);
      this.initialized = true;
    }
  }

  /**
   * Save profiles to localStorage
   */
  private async saveProfiles(): Promise<void> {
    try {
      const data = Array.from(this.profiles.values()).map(profile => ({
        ...profile,
        // Convert Float32Array to regular array for storage
        voiceprint: Array.from(profile.voiceprint)
      }));

      localStorage.setItem(STORAGE_KEY, JSON.stringify(data));
    } catch (error) {
      console.error('Failed to save voice profiles:', error);
      throw error;
    }
  }

  /**
   * Clear all profiles (for testing/reset)
   */
  public async clearAll(): Promise<void> {
    this.profiles.clear();
    localStorage.removeItem(STORAGE_KEY);
  }

  /**
   * Get service statistics
   */
  public getStats(): {
    totalProfiles: number;
    maxProfiles: number;
    totalUses: number;
    averageConfidence: number;
  } {
    const profiles = Array.from(this.profiles.values());

    return {
      totalProfiles: profiles.length,
      maxProfiles: MAX_PROFILES,
      totalUses: profiles.reduce((sum, p) => sum + p.useCount, 0),
      averageConfidence: profiles.length > 0
        ? profiles.reduce((sum, p) => sum + p.confidence, 0) / profiles.length
        : 0
    };
  }
}

export default VoiceProfileService;
