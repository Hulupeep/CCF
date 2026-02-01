/**
 * Voice Assistant Component
 *
 * Main interface for voice-activated personal assistant
 * Integrates speaker identification, speech recognition, and personalized responses
 */

import React, { useState, useEffect } from 'react';
import VoiceProfileService from '../../services/voice/VoiceProfileService';
import AudioCapture from '../../services/voice/AudioCapture';
import { VoiceIdentification } from '../../types/voice';
import VoiceEnrollment from './VoiceEnrollment';

export function VoiceAssistant() {
  const [listening, setListening] = useState(false);
  const [currentUser, setCurrentUser] = useState<string | null>(null);
  const [identification, setIdentification] = useState<VoiceIdentification | null>(null);
  const [transcript, setTranscript] = useState<string>('');
  const [showEnrollment, setShowEnrollment] = useState(false);
  const [profiles, setProfiles] = useState<any[]>([]);
  const [error, setError] = useState<string | null>(null);

  const [voiceService] = useState(() => new VoiceProfileService());
  const [audioCapture] = useState(() => new AudioCapture());

  useEffect(() => {
    loadProfiles();

    return () => {
      audioCapture.dispose();
    };
  }, []);

  const loadProfiles = async () => {
    const allProfiles = await voiceService.getAllProfiles();
    setProfiles(allProfiles);
  };

  const handleStartListening = async () => {
    try {
      setError(null);

      if (!audioCapture.isSupported()) {
        setError('Voice recognition not supported in your browser');
        return;
      }

      const hasPermission = await audioCapture.requestPermission();
      if (!hasPermission) {
        setError('Microphone permission denied');
        return;
      }

      await audioCapture.startRecording();
      setListening(true);
      setTranscript('Listening...');
    } catch (err) {
      setError(`Failed to start: ${err}`);
    }
  };

  const handleStopListening = async () => {
    try {
      setListening(false);
      const audioData = await audioCapture.stopRecording();

      // Identify user
      const id = await voiceService.identifyUser(audioData);
      setIdentification(id);

      if (id.confidence >= 0.85) {
        const profile = await voiceService.getProfile(id.userId!);
        setCurrentUser(profile?.name || null);
        setTranscript(`Welcome back, ${profile?.name}!`);
      } else {
        setCurrentUser(null);
        setTranscript('Hello! I don\'t think we\'ve met. Would you like to enroll?');
      }

      await loadProfiles(); // Refresh stats
    } catch (err) {
      setError(`Failed to process: ${err}`);
      setTranscript('');
    }
  };

  const handleEnrollmentComplete = async (userId: string) => {
    setShowEnrollment(false);
    const profile = await voiceService.getProfile(userId);
    setCurrentUser(profile?.name || null);
    await loadProfiles();
  };

  const stats = voiceService.getStats();

  if (showEnrollment) {
    return (
      <VoiceEnrollment
        onComplete={handleEnrollmentComplete}
        onCancel={() => setShowEnrollment(false)}
      />
    );
  }

  return (
    <div data-testid="voice-assistant-dashboard" className="voice-assistant">
      <header>
        <h1>Voice Assistant</h1>
        {currentUser && (
          <div className="current-user" data-testid="current-user">
            <span className="user-icon">👤</span>
            <span>{currentUser}</span>
          </div>
        )}
      </header>

      <div className="main-controls">
        <div className="listening-area">
          {!listening ? (
            <button
              onClick={handleStartListening}
              className="listen-btn"
              data-testid="start-listening-btn"
            >
              <span className="mic-icon">🎤</span>
              <span>Start Listening</span>
            </button>
          ) : (
            <button
              onClick={handleStopListening}
              className="stop-btn listening"
              data-testid="stop-listening-btn"
            >
              <span className="recording-pulse">⏺</span>
              <span>Stop</span>
            </button>
          )}

          {transcript && (
            <div className="transcript" data-testid="transcript">
              {transcript}
            </div>
          )}

          {identification && (
            <div className="identification-result" data-testid="identification-result">
              <div className="confidence-bar">
                <div
                  className="confidence-fill"
                  style={{
                    width: `${identification.confidence * 100}%`,
                    backgroundColor: identification.confidence >= 0.85 ? '#4CAF50' : '#FF9800'
                  }}
                />
              </div>
              <div className="confidence-text" data-testid={`voice-confidence-${identification.userId}`}>
                Confidence: {(identification.confidence * 100).toFixed(1)}%
              </div>

              {identification.isAnonymous && (
                <div className="anonymous-mode" data-testid="anonymous-mode">
                  <p>⚠️ Anonymous Mode</p>
                  <button onClick={() => setShowEnrollment(true)}>
                    Create Voice Profile
                  </button>
                </div>
              )}

              {identification.alternativeMatches.length > 0 && (
                <div className="alternatives">
                  <p>Alternative matches:</p>
                  {identification.alternativeMatches.map(match => (
                    <div key={match.userId} className="alt-match">
                      {match.userId}: {(match.confidence * 100).toFixed(1)}%
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>

        {error && (
          <div className="error" data-testid="error-message">
            {error}
          </div>
        )}
      </div>

      <div className="voice-profiles">
        <div className="profiles-header">
          <h2>Voice Profiles ({stats.totalProfiles}/{stats.maxProfiles})</h2>
          <button
            onClick={() => setShowEnrollment(true)}
            disabled={stats.totalProfiles >= stats.maxProfiles}
            data-testid="voice-enroll-btn"
          >
            + Add Profile
          </button>
        </div>

        <div className="profiles-list">
          {profiles.map(profile => (
            <div
              key={profile.userId}
              className="profile-card"
              data-testid={`voice-profile-${profile.userId}`}
            >
              <div className="profile-header">
                <h3>{profile.name}</h3>
                {profile.metadata.isChild && (
                  <span className="child-badge">Child</span>
                )}
              </div>

              <div className="profile-stats">
                <div className="stat">
                  <span className="stat-label">Uses:</span>
                  <span className="stat-value">{profile.useCount}</span>
                </div>
                <div className="stat">
                  <span className="stat-label">Samples:</span>
                  <span className="stat-value">{profile.voiceSamples.length}</span>
                </div>
                <div className="stat">
                  <span className="stat-label">Last used:</span>
                  <span className="stat-value">
                    {profile.lastUsed ? new Date(profile.lastUsed).toLocaleDateString() : 'Never'}
                  </span>
                </div>
              </div>

              <div className="profile-actions">
                <button
                  onClick={async () => {
                    if (confirm(`Delete profile for ${profile.name}?`)) {
                      await voiceService.deleteProfile(profile.userId);
                      await loadProfiles();
                      if (currentUser === profile.name) {
                        setCurrentUser(null);
                      }
                    }
                  }}
                  className="delete-btn"
                >
                  Delete
                </button>
              </div>
            </div>
          ))}

          {profiles.length === 0 && (
            <div className="no-profiles">
              <p>No voice profiles yet</p>
              <button onClick={() => setShowEnrollment(true)}>
                Create Your First Profile
              </button>
            </div>
          )}
        </div>
      </div>

      <div className="stats-panel">
        <h3>Statistics</h3>
        <div className="stats-grid">
          <div className="stat-item">
            <div className="stat-number">{stats.totalProfiles}</div>
            <div className="stat-label">Active Profiles</div>
          </div>
          <div className="stat-item">
            <div className="stat-number">{stats.totalUses}</div>
            <div className="stat-label">Total Uses</div>
          </div>
          <div className="stat-item">
            <div className="stat-number">
              {(stats.averageConfidence * 100).toFixed(0)}%
            </div>
            <div className="stat-label">Avg Confidence</div>
          </div>
        </div>
      </div>
    </div>
  );
}

export default VoiceAssistant;
