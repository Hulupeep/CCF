/**
 * News Preferences Component
 * UI for managing user news preferences
 *
 * Contract: I-NEWS-001 - User-controlled personalization
 */

import React, { useState, useEffect } from 'react';
import type { NewsPreferences, NewsTopic } from '../../types/voice';
import { NEWS_TOPICS, NEWS_SOURCES, createDefaultNewsPreferences } from '../../types/voice';
import { getNewsPreferencesManager } from '../../services/news';

interface NewsPreferencesProps {
  userId: string;
  onSave?: (preferences: NewsPreferences) => void;
}

export function NewsPreferences({ userId, onSave }: NewsPreferencesProps) {
  const [preferences, setPreferences] = useState<NewsPreferences | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);

  const preferencesManager = getNewsPreferencesManager();

  // Load preferences on mount
  useEffect(() => {
    loadPreferences();
  }, [userId]);

  const loadPreferences = async () => {
    try {
      setLoading(true);
      const prefs = await preferencesManager.getPreferences(userId);
      setPreferences(prefs);
    } catch (error) {
      console.error('Failed to load preferences:', error);
      setMessage({ type: 'error', text: 'Failed to load preferences' });
      setPreferences(createDefaultNewsPreferences(userId));
    } finally {
      setLoading(false);
    }
  };

  const handleTopicToggle = (topic: string) => {
    if (!preferences) return;

    const topics = preferences.topics.includes(topic)
      ? preferences.topics.filter(t => t !== topic)
      : [...preferences.topics, topic];

    setPreferences({
      ...preferences,
      topics
    });
  };

  const handleExcludeToggle = (topic: string) => {
    if (!preferences) return;

    const excludeTopics = preferences.excludeTopics.includes(topic)
      ? preferences.excludeTopics.filter(t => t !== topic)
      : [...preferences.excludeTopics, topic];

    setPreferences({
      ...preferences,
      excludeTopics
    });
  };

  const handleMaxArticlesChange = (value: number) => {
    if (!preferences) return;

    setPreferences({
      ...preferences,
      maxArticles: Math.max(1, Math.min(20, value))
    });
  };

  const handleReadingLevelChange = (level: 'child' | 'teen' | 'adult') => {
    if (!preferences) return;

    setPreferences({
      ...preferences,
      readingLevel: level
    });
  };

  const handleSave = async () => {
    if (!preferences) return;

    try {
      setSaving(true);
      await preferencesManager.updatePreferences(userId, preferences);
      setMessage({ type: 'success', text: 'Preferences saved successfully!' });

      if (onSave) {
        onSave(preferences);
      }

      // Clear message after 3 seconds
      setTimeout(() => setMessage(null), 3000);
    } catch (error) {
      console.error('Failed to save preferences:', error);
      setMessage({ type: 'error', text: 'Failed to save preferences' });
    } finally {
      setSaving(false);
    }
  };

  const handleReset = async () => {
    if (!confirm('Are you sure you want to reset to default preferences?')) {
      return;
    }

    try {
      setSaving(true);
      await preferencesManager.resetPreferences(userId);
      await loadPreferences();
      setMessage({ type: 'success', text: 'Preferences reset to defaults' });
    } catch (error) {
      console.error('Failed to reset preferences:', error);
      setMessage({ type: 'error', text: 'Failed to reset preferences' });
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return (
      <div data-testid="news-preferences" className="news-preferences loading">
        <div className="spinner">Loading preferences...</div>
      </div>
    );
  }

  if (!preferences) {
    return (
      <div data-testid="news-preferences" className="news-preferences error">
        <p>Failed to load preferences</p>
        <button onClick={loadPreferences}>Retry</button>
      </div>
    );
  }

  return (
    <div data-testid="news-preferences" className="news-preferences">
      <h2>News Preferences</h2>

      {message && (
        <div className={`message message-${message.type}`} role="alert">
          {message.text}
        </div>
      )}

      {/* Topic Selection */}
      <section className="preference-section">
        <h3>Topics of Interest</h3>
        <p className="description">
          Select the topics you want to see in your news briefing
        </p>

        <div className="topic-grid" data-testid="topic-selector">
          {NEWS_TOPICS.map(topic => (
            <label
              key={topic}
              className={`topic-chip ${preferences.topics.includes(topic) ? 'selected' : ''}`}
            >
              <input
                type="checkbox"
                checked={preferences.topics.includes(topic)}
                onChange={() => handleTopicToggle(topic)}
              />
              <span className="topic-name">
                {topic.charAt(0).toUpperCase() + topic.slice(1)}
              </span>
            </label>
          ))}
        </div>
      </section>

      {/* Excluded Topics */}
      <section className="preference-section">
        <h3>Excluded Topics</h3>
        <p className="description">
          Topics you don't want to see (overrides interests)
        </p>

        <div className="topic-grid" data-testid="excluded-topics">
          {NEWS_TOPICS.map(topic => (
            <label
              key={topic}
              className={`topic-chip exclude ${preferences.excludeTopics.includes(topic) ? 'selected' : ''}`}
            >
              <input
                type="checkbox"
                checked={preferences.excludeTopics.includes(topic)}
                onChange={() => handleExcludeToggle(topic)}
              />
              <span className="topic-name">
                {topic.charAt(0).toUpperCase() + topic.slice(1)}
              </span>
            </label>
          ))}
        </div>
      </section>

      {/* Max Articles */}
      <section className="preference-section">
        <h3>Briefing Length</h3>
        <p className="description">
          Maximum number of articles in your daily briefing
        </p>

        <div className="max-articles-control">
          <input
            type="range"
            min="1"
            max="20"
            value={preferences.maxArticles}
            onChange={(e) => handleMaxArticlesChange(parseInt(e.target.value))}
            className="slider"
          />
          <span className="value">{preferences.maxArticles} articles</span>
        </div>
      </section>

      {/* Reading Level */}
      <section className="preference-section">
        <h3>Reading Level</h3>
        <p className="description">
          Adjust content complexity
        </p>

        <div className="reading-level-selector" data-testid="reading-level-selector">
          {(['child', 'teen', 'adult'] as const).map(level => (
            <label key={level} className="radio-option">
              <input
                type="radio"
                name="readingLevel"
                value={level}
                checked={preferences.readingLevel === level}
                onChange={() => handleReadingLevelChange(level)}
              />
              <span className="label">
                {level.charAt(0).toUpperCase() + level.slice(1)}
              </span>
            </label>
          ))}
        </div>
      </section>

      {/* Topic Weights (Advanced) */}
      <section className="preference-section">
        <h3>Learned Preferences</h3>
        <p className="description">
          The system learns from your reading habits. These weights are automatically adjusted.
        </p>

        <div className="topic-weights">
          {Object.entries(preferences.topicWeights)
            .filter(([topic]) => !topic.startsWith('source:'))
            .sort(([, a], [, b]) => b - a)
            .slice(0, 10)
            .map(([topic, weight]) => (
              <div key={topic} className="weight-bar">
                <span className="topic">{topic}</span>
                <div className="bar">
                  <div
                    className="fill"
                    style={{ width: `${(weight / 2.0) * 100}%` }}
                  />
                </div>
                <span className="value">{weight.toFixed(2)}</span>
              </div>
            ))}

          {Object.keys(preferences.topicWeights).length === 0 && (
            <p className="empty-state">
              No learned preferences yet. Start reading to build your profile!
            </p>
          )}
        </div>
      </section>

      {/* Actions */}
      <div className="actions">
        <button
          onClick={handleSave}
          disabled={saving}
          className="btn btn-primary"
        >
          {saving ? 'Saving...' : 'Save Preferences'}
        </button>

        <button
          onClick={handleReset}
          disabled={saving}
          className="btn btn-secondary"
        >
          Reset to Defaults
        </button>
      </div>

      <style jsx>{`
        .news-preferences {
          max-width: 800px;
          margin: 0 auto;
          padding: 20px;
        }

        .preference-section {
          margin-bottom: 30px;
          padding: 20px;
          background: #f5f5f5;
          border-radius: 8px;
        }

        .preference-section h3 {
          margin-top: 0;
          color: #333;
        }

        .description {
          color: #666;
          font-size: 14px;
          margin-bottom: 15px;
        }

        .topic-grid {
          display: grid;
          grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
          gap: 10px;
        }

        .topic-chip {
          display: flex;
          align-items: center;
          padding: 10px 15px;
          border: 2px solid #ddd;
          border-radius: 20px;
          cursor: pointer;
          transition: all 0.2s;
          background: white;
        }

        .topic-chip:hover {
          border-color: #4CAF50;
        }

        .topic-chip.selected {
          border-color: #4CAF50;
          background: #E8F5E9;
        }

        .topic-chip.exclude.selected {
          border-color: #f44336;
          background: #FFEBEE;
        }

        .topic-chip input {
          margin-right: 8px;
        }

        .max-articles-control {
          display: flex;
          align-items: center;
          gap: 15px;
        }

        .slider {
          flex: 1;
        }

        .value {
          min-width: 100px;
          font-weight: bold;
        }

        .reading-level-selector {
          display: flex;
          gap: 15px;
        }

        .radio-option {
          display: flex;
          align-items: center;
          padding: 10px 20px;
          border: 2px solid #ddd;
          border-radius: 8px;
          cursor: pointer;
          background: white;
        }

        .radio-option input {
          margin-right: 8px;
        }

        .topic-weights {
          display: flex;
          flex-direction: column;
          gap: 10px;
        }

        .weight-bar {
          display: flex;
          align-items: center;
          gap: 10px;
        }

        .weight-bar .topic {
          min-width: 120px;
          font-size: 14px;
        }

        .weight-bar .bar {
          flex: 1;
          height: 20px;
          background: #e0e0e0;
          border-radius: 10px;
          overflow: hidden;
        }

        .weight-bar .fill {
          height: 100%;
          background: linear-gradient(90deg, #4CAF50, #8BC34A);
          transition: width 0.3s;
        }

        .weight-bar .value {
          min-width: 50px;
          text-align: right;
          font-size: 12px;
          color: #666;
        }

        .actions {
          display: flex;
          gap: 15px;
          margin-top: 30px;
        }

        .btn {
          padding: 12px 24px;
          border: none;
          border-radius: 8px;
          font-size: 16px;
          cursor: pointer;
          transition: all 0.2s;
        }

        .btn:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }

        .btn-primary {
          background: #4CAF50;
          color: white;
        }

        .btn-primary:hover:not(:disabled) {
          background: #45a049;
        }

        .btn-secondary {
          background: #f5f5f5;
          color: #333;
          border: 1px solid #ddd;
        }

        .btn-secondary:hover:not(:disabled) {
          background: #e0e0e0;
        }

        .message {
          padding: 12px;
          border-radius: 8px;
          margin-bottom: 20px;
        }

        .message-success {
          background: #E8F5E9;
          color: #2E7D32;
          border: 1px solid #4CAF50;
        }

        .message-error {
          background: #FFEBEE;
          color: #C62828;
          border: 1px solid #f44336;
        }

        .empty-state {
          text-align: center;
          color: #999;
          font-style: italic;
        }

        .loading,
        .error {
          text-align: center;
          padding: 40px;
        }

        .spinner {
          font-size: 18px;
          color: #666;
        }
      `}</style>
    </div>
  );
}
