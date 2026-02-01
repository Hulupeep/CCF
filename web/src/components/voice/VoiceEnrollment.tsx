/**
 * Voice Enrollment Component
 *
 * Contract: I-VOICE-002 (Privacy Protection - explicit consent)
 * Guides users through voice profile enrollment process
 */

import React, { useState, useEffect } from 'react';
import AudioCapture from '../../services/voice/AudioCapture';
import VoiceProfileService from '../../services/voice/VoiceProfileService';
import { VoiceSample, EnrollmentPhrase } from '../../types/voice';

interface VoiceEnrollmentProps {
  onComplete: (userId: string) => void;
  onCancel: () => void;
}

export function VoiceEnrollment({ onComplete, onCancel }: VoiceEnrollmentProps) {
  const [step, setStep] = useState(1);
  const [name, setName] = useState('');
  const [age, setAge] = useState<number | undefined>();
  const [samples, setSamples] = useState<VoiceSample[]>([]);
  const [recording, setRecording] = useState(false);
  const [currentPhrase, setCurrentPhrase] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const [audioCapture] = useState(() => new AudioCapture());
  const [voiceService] = useState(() => new VoiceProfileService());

  const phrases: EnrollmentPhrase[] = [
    { text: "I am {name} and this is my voice", recorded: false, attempt: 0 },
    { text: "Hello mBot, it's {name}", recorded: false, attempt: 0 },
    { text: "This is {name} speaking", recorded: false, attempt: 0 }
  ];

  // Replace {name} placeholder with actual name
  const getPhrase = (index: number): string => {
    return phrases[index].text.replace('{name}', name);
  };

  const handleStartRecording = async () => {
    try {
      setError(null);

      // Check browser support
      if (!audioCapture.isSupported()) {
        setError('Voice recording is not supported in your browser');
        return;
      }

      // Request permission
      const hasPermission = await audioCapture.requestPermission();
      if (!hasPermission) {
        setError('Microphone permission denied. Please allow access to continue.');
        return;
      }

      // Start recording
      await audioCapture.startRecording();
      setRecording(true);
    } catch (err) {
      setError(`Failed to start recording: ${err}`);
    }
  };

  const handleStopRecording = async () => {
    try {
      setRecording(false);
      const audioData = await audioCapture.stopRecording();

      // Create voice sample
      const sample: VoiceSample = {
        id: `sample-${Date.now()}`,
        audioData,
        duration: audioCapture.getRecordingDuration(),
        sampleRate: 48000, // Standard sample rate
        recordedAt: Date.now(),
        phrase: getPhrase(currentPhrase)
      };

      setSamples([...samples, sample]);

      // Move to next phrase
      if (currentPhrase < phrases.length - 1) {
        setCurrentPhrase(currentPhrase + 1);
      } else {
        // All samples recorded, move to next step
        setStep(3);
      }
    } catch (err) {
      setError(`Failed to save recording: ${err}`);
    }
  };

  const handleEnroll = async () => {
    try {
      setError(null);

      if (samples.length < 3) {
        setError('At least 3 voice samples required');
        return;
      }

      const profile = await voiceService.enrollUser(name, samples);

      // Update profile with age if provided
      if (age) {
        await voiceService.updateProfile(profile.userId, {
          metadata: {
            age,
            isChild: age < 13,
            language: 'en-US'
          }
        });
      }

      setStep(4);
      setTimeout(() => onComplete(profile.userId), 2000);
    } catch (err) {
      setError(`Enrollment failed: ${err}`);
    }
  };

  const handleRetryPhrase = () => {
    // Remove last sample and allow re-recording
    if (samples.length > 0) {
      setSamples(samples.slice(0, -1));
      if (currentPhrase > 0) {
        setCurrentPhrase(currentPhrase - 1);
      }
    }
  };

  useEffect(() => {
    // Cleanup on unmount
    return () => {
      audioCapture.dispose();
    };
  }, [audioCapture]);

  return (
    <div data-testid="voice-enrollment" className="voice-enrollment">
      {step === 1 && (
        <div className="enrollment-step" data-testid="enrollment-step-1">
          <h2>Voice Enrollment</h2>
          <p>Let's set up your voice profile so mBot can recognize you!</p>

          <div className="form-group">
            <label htmlFor="name">What's your name?</label>
            <input
              id="name"
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Enter your name"
              data-testid="name-input"
            />
          </div>

          <div className="form-group">
            <label htmlFor="age">Age (optional)</label>
            <input
              id="age"
              type="number"
              value={age || ''}
              onChange={(e) => setAge(e.target.value ? parseInt(e.target.value) : undefined)}
              placeholder="Your age"
              data-testid="age-input"
            />
          </div>

          <div className="privacy-notice">
            <h3>Privacy Notice</h3>
            <p>
              Your voice samples will be stored locally on your device.
              We will not share your voice data with anyone.
              You can delete your profile at any time.
            </p>
            <label>
              <input
                type="checkbox"
                id="consent"
                data-testid="privacy-consent"
              />
              I understand and consent to voice data collection
            </label>
          </div>

          <div className="actions">
            <button onClick={onCancel} data-testid="cancel-btn">
              Cancel
            </button>
            <button
              onClick={() => setStep(2)}
              disabled={!name || !document.getElementById('consent')?.['checked']}
              data-testid="next-btn"
            >
              Next
            </button>
          </div>

          {error && <div className="error" data-testid="error-message">{error}</div>}
        </div>
      )}

      {step === 2 && (
        <div className="enrollment-step" data-testid="enrollment-step-2">
          <h2>Record Voice Samples</h2>
          <p>
            Please record yourself saying the following phrases clearly.
            We need 3 samples to create your voice profile.
          </p>

          <div className="progress">
            <div className="progress-bar">
              <div
                className="progress-fill"
                style={{ width: `${(samples.length / 3) * 100}%` }}
              />
            </div>
            <p>Sample {samples.length + 1} of 3</p>
          </div>

          <div className="phrase-display" data-testid="current-phrase">
            <h3>Say:</h3>
            <p className="phrase-text">"{getPhrase(currentPhrase)}"</p>
          </div>

          <div className="recording-controls">
            {!recording ? (
              <button
                onClick={handleStartRecording}
                className="record-btn"
                data-testid="start-recording-btn"
              >
                <span className="record-icon">🎤</span>
                Start Recording
              </button>
            ) : (
              <button
                onClick={handleStopRecording}
                className="stop-btn"
                data-testid="stop-recording-btn"
              >
                <span className="recording-indicator">⏹</span>
                Stop Recording
              </button>
            )}
          </div>

          {recording && (
            <div className="recording-status" data-testid="recording-status">
              <div className="recording-animation">🔴 Recording...</div>
            </div>
          )}

          {samples.length > 0 && (
            <div className="samples-list">
              <h4>Recorded Samples:</h4>
              {samples.map((sample, idx) => (
                <div key={sample.id} className="sample-item" data-testid={`sample-${idx}`}>
                  ✓ Sample {idx + 1}: {sample.phrase}
                </div>
              ))}
              <button onClick={handleRetryPhrase} className="retry-btn">
                Redo Last Sample
              </button>
            </div>
          )}

          <div className="actions">
            <button onClick={() => setStep(1)} data-testid="back-btn">
              Back
            </button>
          </div>

          {error && <div className="error" data-testid="error-message">{error}</div>}
        </div>
      )}

      {step === 3 && (
        <div className="enrollment-step" data-testid="enrollment-step-3">
          <h2>Review & Complete</h2>
          <p>Great! You've recorded all voice samples.</p>

          <div className="review-summary">
            <div className="summary-item">
              <strong>Name:</strong> {name}
            </div>
            {age && (
              <div className="summary-item">
                <strong>Age:</strong> {age}
              </div>
            )}
            <div className="summary-item">
              <strong>Voice Samples:</strong> {samples.length}
            </div>
          </div>

          <div className="actions">
            <button onClick={() => setStep(2)} data-testid="back-btn">
              Back
            </button>
            <button onClick={handleEnroll} data-testid="enroll-btn">
              Complete Enrollment
            </button>
          </div>

          {error && <div className="error" data-testid="error-message">{error}</div>}
        </div>
      )}

      {step === 4 && (
        <div className="enrollment-step" data-testid="enrollment-step-4">
          <h2>Enrollment Complete!</h2>
          <div className="success-message">
            <div className="success-icon">✓</div>
            <p>
              All set, {name}! mBot will now recognize your voice.
            </p>
          </div>
        </div>
      )}
    </div>
  );
}

export default VoiceEnrollment;
