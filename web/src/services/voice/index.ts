/**
 * Voice Services Index
 * Central export for all voice recognition services
 */

export { default as VoiceProfileService } from './VoiceProfileService';
export { default as AudioCapture } from './AudioCapture';
export { default as WhisperAPI } from './WhisperAPI';

export type { WhisperConfig, TranscriptionResult } from './WhisperAPI';
