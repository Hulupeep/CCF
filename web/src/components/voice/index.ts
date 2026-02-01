/**
 * Voice Recognition Components
 * Export all voice-related components and services
 */

export { VoiceEnrollment } from './VoiceEnrollment';
export { VoiceAssistant } from './VoiceAssistant';

export { default as AudioCapture } from '../../services/voice/AudioCapture';
export { default as VoiceProfileService } from '../../services/voice/VoiceProfileService';
export { default as WhisperAPI } from '../../services/voice/WhisperAPI';

export type {
  VoiceProfile,
  VoiceSample,
  VoiceIdentification,
  VoiceSettings,
  VoiceCommand,
  CommandResult,
  VoiceCommandHistory
} from '../../types/voice';
