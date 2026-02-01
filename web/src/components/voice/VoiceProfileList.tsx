/**
 * Voice Profile List Component
 *
 * Displays list of enrolled voice profiles
 */

import React, { useState, useEffect } from 'react';
import { VoiceProfile } from '../../types/voice';
import { VoiceProfileService } from '../../services/voice/VoiceProfileService';

const voiceService = new VoiceProfileService();

export function VoiceProfileList() {
  const [profiles, setProfiles] = useState<VoiceProfile[]>([]);
  const [enrolling, setEnrolling] = useState(false);

  useEffect(() => {
    loadProfiles();
  }, []);

  const loadProfiles = async () => {
    const allProfiles = await voiceService.getAllProfiles();
    setProfiles(allProfiles);
  };

  const handleEnroll = () => {
    setEnrolling(true);
    // Would open enrollment modal
  };

  return (
    <div className="voice-profile-list">
      <div className="list-header">
        <h2>Voice Profiles</h2>
        <button
          onClick={handleEnroll}
          data-testid="voice-enroll-btn"
        >
          Enroll New Voice
        </button>
      </div>

      {profiles.length === 0 && (
        <div className="empty-state">
          <p>No voice profiles yet. Enroll your voice to get started!</p>
        </div>
      )}

      <div className="profiles-grid">
        {profiles.map(profile => (
          <div
            key={profile.userId}
            className="profile-card"
            data-testid={`voice-profile-${profile.userId}`}
          >
            <div className="profile-avatar">
              {profile.name.charAt(0).toUpperCase()}
            </div>
            <div className="profile-info">
              <h3>{profile.name}</h3>
              <div className="profile-stats">
                <span
                  className="confidence-indicator"
                  data-testid={`voice-confidence-${profile.userId}`}
                >
                  Confidence: {(profile.confidence * 100).toFixed(0)}%
                </span>
                <span className="use-count">
                  Used {profile.useCount} times
                </span>
              </div>
              <div className="profile-meta">
                {profile.metadata.isChild && (
                  <span className="child-badge">Child</span>
                )}
                <span>Language: {profile.metadata.language}</span>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
