/**
 * Privacy Settings Component
 *
 * Configure privacy and data retention settings
 * data-testid: voice-privacy-settings
 */

import React, { useState, useEffect } from 'react';
import { PrivacySettings as IPrivacySettings } from '../../types/voice';
import { ConversationMemoryService } from '../../services/memory/ConversationMemoryService';

const memoryService = new ConversationMemoryService();

interface PrivacySettingsProps {
  userId: string;
}

export function PrivacySettings({ userId }: PrivacySettingsProps) {
  const [settings, setSettings] = useState<IPrivacySettings | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    loadSettings();
  }, [userId]);

  const loadSettings = async () => {
    setLoading(true);
    try {
      const prefs = await memoryService.getUserPreferences(userId);
      setSettings(prefs.privacySettings);
    } finally {
      setLoading(false);
    }
  };

  const handleToggle = (key: keyof IPrivacySettings) => {
    if (!settings || typeof settings[key] !== 'boolean') return;
    setSettings({ ...settings, [key]: !settings[key] });
  };

  const handleRetentionChange = (days: number) => {
    if (!settings) return;
    setSettings({ ...settings, retentionDays: days });
  };

  const handleSave = async () => {
    if (!settings) return;

    setSaving(true);
    try {
      await memoryService.updatePrivacySettings(userId, settings);
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return <div>Loading privacy settings...</div>;
  }

  if (!settings) {
    return <div>Failed to load settings.</div>;
  }

  return (
    <div data-testid="voice-privacy-settings" className="privacy-settings">
      <h2>Privacy Settings</h2>

      <div className="settings-section">
        <div className="setting-item">
          <label>
            <input
              type="checkbox"
              checked={settings.allowVoiceRecording}
              onChange={() => handleToggle('allowVoiceRecording')}
            />
            Allow Voice Recording
          </label>
          <p className="setting-description">
            Enable voice recording for speaker identification
          </p>
        </div>

        <div className="setting-item">
          <label>
            <input
              type="checkbox"
              checked={settings.allowEmailAccess}
              onChange={() => handleToggle('allowEmailAccess')}
            />
            Allow Email Access
          </label>
          <p className="setting-description">
            Allow mBot to access your email for summaries
          </p>
        </div>

        <div className="setting-item">
          <label>
            <input
              type="checkbox"
              checked={settings.allowNewsPersonalization}
              onChange={() => handleToggle('allowNewsPersonalization')}
            />
            Allow News Personalization
          </label>
          <p className="setting-description">
            Personalize news based on your interests
          </p>
        </div>

        <div className="setting-item">
          <label>
            <input
              type="checkbox"
              checked={settings.shareDataWithFamily}
              onChange={() => handleToggle('shareDataWithFamily')}
            />
            Share Data with Family
          </label>
          <p className="setting-description">
            Allow family members to see shared activities
          </p>
        </div>
      </div>

      <div className="settings-section">
        <h3>Data Retention</h3>
        <div className="retention-options">
          {[7, 30, 90, 365].map(days => (
            <button
              key={days}
              className={`retention-btn ${settings.retentionDays === days ? 'selected' : ''}`}
              onClick={() => handleRetentionChange(days)}
            >
              {days} days
            </button>
          ))}
        </div>
        <p className="setting-description">
          Conversation history will be deleted after {settings.retentionDays} days
        </p>
      </div>

      <button
        className="save-btn"
        onClick={handleSave}
        disabled={saving}
      >
        {saving ? 'Saving...' : 'Save Settings'}
      </button>
    </div>
  );
}
