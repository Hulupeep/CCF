/**
 * News Preferences Component
 *
 * Allows users to configure news preferences
 * data-testid: news-preferences
 */

import React, { useState, useEffect } from 'react';
import { NewsPreferences as INewsPreferences, NEWS_TOPICS, NEWS_SOURCES } from '../../types/voice';
import { NewsService } from '../../services/news/NewsService';

const newsService = new NewsService();

interface NewsPreferencesProps {
  userId: string;
}

export function NewsPreferences({ userId }: NewsPreferencesProps) {
  const [preferences, setPreferences] = useState<INewsPreferences | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    loadPreferences();
  }, [userId]);

  const loadPreferences = async () => {
    setLoading(true);
    try {
      const prefs = await newsService.getPreferences(userId);
      setPreferences(prefs);
    } finally {
      setLoading(false);
    }
  };

  const handleToggleTopic = (topic: string) => {
    if (!preferences) return;

    const topics = preferences.topics.includes(topic)
      ? preferences.topics.filter(t => t !== topic)
      : [...preferences.topics, topic];

    setPreferences({ ...preferences, topics });
  };

  const handleSave = async () => {
    if (!preferences) return;

    setSaving(true);
    try {
      await newsService.updatePreferences(userId, preferences);
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return <div>Loading preferences...</div>;
  }

  if (!preferences) {
    return <div>Failed to load preferences.</div>;
  }

  return (
    <div data-testid="news-preferences" className="news-preferences">
      <h2>News Preferences</h2>

      <div className="preferences-section">
        <h3>Topics of Interest</h3>
        <div className="topic-grid">
          {NEWS_TOPICS.map(topic => (
            <button
              key={topic}
              className={`topic-btn ${preferences.topics.includes(topic) ? 'selected' : ''}`}
              onClick={() => handleToggleTopic(topic)}
              data-topic={topic}
            >
              {topic}
            </button>
          ))}
        </div>
      </div>

      <div className="preferences-section">
        <h3>Maximum Articles</h3>
        <input
          type="number"
          min="1"
          max="10"
          value={preferences.maxArticles}
          onChange={(e) => setPreferences({ ...preferences, maxArticles: parseInt(e.target.value) })}
        />
      </div>

      <div className="preferences-section">
        <h3>Reading Level</h3>
        <select
          value={preferences.readingLevel || 'adult'}
          onChange={(e) => setPreferences({ ...preferences, readingLevel: e.target.value as any })}
        >
          <option value="child">Child-Friendly</option>
          <option value="teen">Teen</option>
          <option value="adult">Adult</option>
        </select>
      </div>

      <button
        className="save-btn"
        onClick={handleSave}
        disabled={saving}
      >
        {saving ? 'Saving...' : 'Save Preferences'}
      </button>
    </div>
  );
}
