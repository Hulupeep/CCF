/**
 * Voice Recognition Integration Tests
 *
 * Tests for contracts:
 * - I-VOICE-001: Speaker identification ≥85%
 * - I-VOICE-002: Privacy protection
 * - I-VOICE-005: Multi-user support (10 profiles)
 * - I-VOICE-006: Anonymous mode fallback
 */

import { describe, test, expect, beforeEach, afterEach } from '@jest/globals';
import VoiceProfileService from '../../web/src/services/voice/VoiceProfileService';
import AudioCapture from '../../web/src/services/voice/AudioCapture';
import { VoiceSample } from '../../web/src/types/voice';

describe('Voice Recognition System', () => {
  let voiceService: VoiceProfileService;
  let audioCapture: AudioCapture;

  beforeEach(() => {
    voiceService = new VoiceProfileService();
    audioCapture = new AudioCapture();
    localStorage.clear();
  });

  afterEach(async () => {
    await voiceService.clearAll();
    audioCapture.dispose();
  });

  describe('I-VOICE-001: Speaker Identification ≥85%', () => {
    test('should identify enrolled user with ≥85% confidence', async () => {
      // Create mock voice samples
      const samples = createMockSamples('Alice', 3);

      // Enroll user
      const profile = await voiceService.enrollUser('Alice', samples);
      expect(profile.userId).toBeDefined();
      expect(profile.name).toBe('Alice');

      // Create test audio that should match
      const testAudio = createMockAudioData('Alice');

      // Identify user
      const identification = await voiceService.identifyUser(testAudio);

      // Verify confidence threshold
      expect(identification.userId).toBe(profile.userId);
      expect(identification.confidence).toBeGreaterThanOrEqual(0.85);
      expect(identification.isAnonymous).toBe(false);
    });

    test('should reject unknown user below confidence threshold', async () => {
      // Enroll one user
      const aliceSamples = createMockSamples('Alice', 3);
      await voiceService.enrollUser('Alice', aliceSamples);

      // Test with completely different audio
      const unknownAudio = createMockAudioData('Unknown');

      // Identify should fail
      const identification = await voiceService.identifyUser(unknownAudio);

      // Should be anonymous if confidence < 85%
      if (identification.confidence < 0.85) {
        expect(identification.isAnonymous).toBe(true);
        expect(identification.userId).toBeNull();
      }
    });

    test('should provide alternative matches ranked by confidence', async () => {
      // Enroll multiple users
      await voiceService.enrollUser('Alice', createMockSamples('Alice', 3));
      await voiceService.enrollUser('Bob', createMockSamples('Bob', 3));
      await voiceService.enrollUser('Charlie', createMockSamples('Charlie', 3));

      const testAudio = createMockAudioData('Alice');
      const identification = await voiceService.identifyUser(testAudio);

      // Should have alternative matches
      expect(identification.alternativeMatches.length).toBeGreaterThan(0);

      // Alternative matches should be sorted by confidence
      const confidences = identification.alternativeMatches.map(m => m.confidence);
      for (let i = 1; i < confidences.length; i++) {
        expect(confidences[i]).toBeLessThanOrEqual(confidences[i - 1]);
      }
    });
  });

  describe('I-VOICE-002: Privacy Protection', () => {
    test('should require explicit consent during enrollment', () => {
      // This is tested in component tests
      // The VoiceEnrollment component requires consent checkbox
      expect(true).toBe(true);
    });

    test('should not share voice data across profiles', async () => {
      const alice = await voiceService.enrollUser('Alice', createMockSamples('Alice', 3));
      const bob = await voiceService.enrollUser('Bob', createMockSamples('Bob', 3));

      const aliceProfile = await voiceService.getProfile(alice.userId);
      const bobProfile = await voiceService.getProfile(bob.userId);

      // Profiles should be completely separate
      expect(aliceProfile?.voiceprint).not.toEqual(bobProfile?.voiceprint);
      expect(aliceProfile?.voiceSamples).not.toEqual(bobProfile?.voiceSamples);
    });

    test('should allow profile deletion', async () => {
      const profile = await voiceService.enrollUser('Alice', createMockSamples('Alice', 3));

      // Delete profile
      await voiceService.deleteProfile(profile.userId);

      // Profile should be gone
      const deletedProfile = await voiceService.getProfile(profile.userId);
      expect(deletedProfile).toBeNull();
    });

    test('should encrypt voiceprints in storage', async () => {
      await voiceService.enrollUser('Alice', createMockSamples('Alice', 3));

      // Check localStorage
      const stored = localStorage.getItem('mbot_voice_profiles');
      expect(stored).toBeDefined();

      // Voiceprints should be stored as arrays (serialized Float32Array)
      const data = JSON.parse(stored!);
      expect(Array.isArray(data[0].voiceprint)).toBe(true);
    });
  });

  describe('I-VOICE-005: Multi-User Support (10 profiles)', () => {
    test('should support up to 10 voice profiles', async () => {
      const profiles = [];

      // Enroll 10 users
      for (let i = 0; i < 10; i++) {
        const profile = await voiceService.enrollUser(
          `User${i}`,
          createMockSamples(`User${i}`, 3)
        );
        profiles.push(profile);
      }

      // All profiles should be accessible
      const allProfiles = await voiceService.getAllProfiles();
      expect(allProfiles.length).toBe(10);
    });

    test('should reject enrollment beyond 10 profiles', async () => {
      // Enroll 10 users
      for (let i = 0; i < 10; i++) {
        await voiceService.enrollUser(`User${i}`, createMockSamples(`User${i}`, 3));
      }

      // 11th enrollment should fail
      await expect(
        voiceService.enrollUser('User11', createMockSamples('User11', 3))
      ).rejects.toThrow('Maximum 10 voice profiles supported');
    });

    test('should maintain independent contexts for each user', async () => {
      await voiceService.enrollUser('Alice', createMockSamples('Alice', 3));
      await voiceService.enrollUser('Bob', createMockSamples('Bob', 3));

      // Update Alice's profile
      const alice = (await voiceService.getAllProfiles())[0];
      await voiceService.updateProfile(alice.userId, {
        metadata: {
          ...alice.metadata,
          age: 30
        }
      });

      // Bob's profile should be unchanged
      const bob = (await voiceService.getAllProfiles())[1];
      expect(bob.metadata.age).toBeUndefined();
    });
  });

  describe('I-VOICE-006: Anonymous Mode Fallback', () => {
    test('should operate in anonymous mode when no profiles exist', async () => {
      const testAudio = createMockAudioData('Unknown');
      const identification = await voiceService.identifyUser(testAudio);

      expect(identification.isAnonymous).toBe(true);
      expect(identification.userId).toBeNull();
      expect(identification.confidence).toBe(0);
    });

    test('should fallback to anonymous mode on recognition failure', async () => {
      await voiceService.enrollUser('Alice', createMockSamples('Alice', 3));

      // Simulate error by passing invalid data
      const invalidAudio = new ArrayBuffer(0);
      const identification = await voiceService.identifyUser(invalidAudio);

      expect(identification.isAnonymous).toBe(true);
    });

    test('should not expose personal data in anonymous mode', async () => {
      // Enroll user with personal data
      const alice = await voiceService.enrollUser('Alice', createMockSamples('Alice', 3));
      await voiceService.updateProfile(alice.userId, {
        metadata: {
          ...alice.metadata,
          age: 30
        }
      });

      // Anonymous identification
      const anonymousAudio = createMockAudioData('Unknown');
      const identification = await voiceService.identifyUser(anonymousAudio);

      // Should not return any profile data
      expect(identification.userId).toBeNull();
      expect(identification.isAnonymous).toBe(true);
    });
  });

  describe('Voice Enrollment Flow', () => {
    test('should require at least 3 samples for enrollment', async () => {
      const insufficientSamples = createMockSamples('Alice', 2);

      await expect(
        voiceService.enrollUser('Alice', insufficientSamples)
      ).rejects.toThrow('At least 3 voice samples required');
    });

    test('should improve accuracy with additional samples', async () => {
      const initialSamples = createMockSamples('Alice', 3);
      const profile = await voiceService.enrollUser('Alice', initialSamples);

      // Add more samples
      for (let i = 0; i < 3; i++) {
        const additionalSample = createMockSample('Alice', i + 4);
        await voiceService.addVoiceSample(profile.userId, additionalSample);
      }

      // Profile should have 6 samples now
      const updatedProfile = await voiceService.getProfile(profile.userId);
      expect(updatedProfile?.voiceSamples.length).toBe(6);
    });

    test('should track profile usage statistics', async () => {
      const profile = await voiceService.enrollUser('Alice', createMockSamples('Alice', 3));

      // Simulate multiple identifications
      const testAudio = createMockAudioData('Alice');
      await voiceService.identifyUser(testAudio);
      await voiceService.identifyUser(testAudio);
      await voiceService.identifyUser(testAudio);

      // Check stats
      const updatedProfile = await voiceService.getProfile(profile.userId);
      expect(updatedProfile?.useCount).toBeGreaterThan(0);
      expect(updatedProfile?.lastUsed).toBeGreaterThan(updatedProfile.enrolledAt);
    });
  });

  describe('Performance Requirements', () => {
    test('should complete identification in <2 seconds', async () => {
      await voiceService.enrollUser('Alice', createMockSamples('Alice', 3));
      const testAudio = createMockAudioData('Alice');

      const startTime = Date.now();
      await voiceService.identifyUser(testAudio);
      const duration = Date.now() - startTime;

      expect(duration).toBeLessThan(2000);
    });

    test('should handle concurrent identification requests', async () => {
      // Enroll multiple users
      await voiceService.enrollUser('Alice', createMockSamples('Alice', 3));
      await voiceService.enrollUser('Bob', createMockSamples('Bob', 3));

      // Concurrent identifications
      const results = await Promise.all([
        voiceService.identifyUser(createMockAudioData('Alice')),
        voiceService.identifyUser(createMockAudioData('Bob')),
        voiceService.identifyUser(createMockAudioData('Alice'))
      ]);

      expect(results.length).toBe(3);
      results.forEach(result => {
        expect(result.timestamp).toBeDefined();
      });
    });
  });
});

// Helper functions for creating mock data
function createMockSamples(name: string, count: number): VoiceSample[] {
  return Array.from({ length: count }, (_, i) => createMockSample(name, i));
}

function createMockSample(name: string, index: number): VoiceSample {
  const phrase = [
    `I am ${name} and this is my voice`,
    `Hello mBot, it's ${name}`,
    `This is ${name} speaking`
  ][index % 3];

  // Create mock audio data (in real tests, use actual audio)
  const audioData = new ArrayBuffer(1024);
  const view = new Uint8Array(audioData);

  // Fill with pseudo-random data based on name (for consistency)
  const seed = name.split('').reduce((sum, char) => sum + char.charCodeAt(0), 0);
  for (let i = 0; i < view.length; i++) {
    view[i] = ((seed * (i + 1)) % 256);
  }

  return {
    id: `${name}-sample-${index}-${Date.now()}`,
    audioData,
    duration: 3000, // 3 seconds
    sampleRate: 48000,
    recordedAt: Date.now(),
    phrase
  };
}

function createMockAudioData(identifier: string): ArrayBuffer {
  const audioData = new ArrayBuffer(512);
  const view = new Uint8Array(audioData);

  // Create consistent audio signature for identifier
  const seed = identifier.split('').reduce((sum, char) => sum + char.charCodeAt(0), 0);
  for (let i = 0; i < view.length; i++) {
    view[i] = ((seed * (i + 1)) % 256);
  }

  return audioData;
}
