/**
 * Briefing Panel Component
 *
 * UI for generating and playing personalized daily briefings
 * data-testid: morning-briefing
 */

import React, { useState, useEffect } from 'react';
import { PersonalBriefing, BriefingSection as IBriefingSection } from '../../types/voice';
import { PersonalBriefingService } from '../../services/briefing/PersonalBriefingService';
import { TTSService } from '../../services/briefing/TTSService';

const briefingService = new PersonalBriefingService();
const ttsService = new TTSService();

interface BriefingPanelProps {
  userId: string;
}

export function BriefingPanel({ userId }: BriefingPanelProps) {
  const [briefing, setBriefing] = useState<PersonalBriefing | null>(null);
  const [playing, setPlaying] = useState(false);
  const [currentSection, setCurrentSection] = useState<number>(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleGenerateBriefing = async () => {
    setLoading(true);
    setError(null);

    try {
      const generated = await briefingService.generateBriefing(userId);
      setBriefing(generated);
    } catch (err) {
      setError('Failed to generate briefing');
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  const handlePlayBriefing = async () => {
    if (!briefing) return;

    setPlaying(true);
    setError(null);

    try {
      for (let i = 0; i < briefing.sections.length; i++) {
        const section = briefing.sections[i];
        setCurrentSection(i);

        // Speak each section
        await ttsService.synthesizeSpeech(section.content);

        // Small pause between sections
        await new Promise(resolve => setTimeout(resolve, 500));
      }
    } catch (err) {
      setError('Failed to play briefing');
      console.error(err);
    } finally {
      setPlaying(false);
      setCurrentSection(0);
    }
  };

  const handleStopBriefing = () => {
    ttsService.stop();
    setPlaying(false);
    setCurrentSection(0);
  };

  useEffect(() => {
    // Auto-generate on mount
    if (userId) {
      handleGenerateBriefing();
    }
  }, [userId]);

  return (
    <div data-testid="morning-briefing" className="briefing-panel">
      <div className="briefing-header">
        <h1>Your Daily Briefing</h1>
        {briefing && (
          <span className="briefing-duration">
            ~{briefing.duration} seconds
          </span>
        )}
      </div>

      <div className="briefing-controls">
        <button
          onClick={handleGenerateBriefing}
          disabled={loading || playing}
          data-testid="generate-briefing-btn"
        >
          {loading ? 'Generating...' : 'Regenerate Briefing'}
        </button>

        <button
          onClick={handlePlayBriefing}
          disabled={!briefing || playing || loading}
          data-testid="play-briefing-btn"
        >
          {playing ? 'Playing...' : 'Play Briefing'}
        </button>

        {playing && (
          <button
            onClick={handleStopBriefing}
            data-testid="stop-briefing-btn"
          >
            Stop
          </button>
        )}
      </div>

      {error && (
        <div className="error-message" data-testid="briefing-error">
          {error}
        </div>
      )}

      {briefing && (
        <div className="briefing-sections">
          {briefing.sections.map((section, idx) => (
            <BriefingSection
              key={idx}
              section={section}
              active={playing && currentSection === idx}
              index={idx}
            />
          ))}
        </div>
      )}

      {!briefing && !loading && (
        <div className="empty-state">
          <p>No briefing available. Click "Generate Briefing" to create one.</p>
        </div>
      )}
    </div>
  );
}

interface BriefingSectionProps {
  section: IBriefingSection;
  active: boolean;
  index: number;
}

function BriefingSection({ section, active, index }: BriefingSectionProps) {
  const getIcon = (type: string) => {
    switch (type) {
      case 'greeting': return '👋';
      case 'news': return '📰';
      case 'email': return '📧';
      case 'calendar': return '📅';
      case 'memory': return '💭';
      case 'question': return '❓';
      case 'weather': return '☀️';
      default: return '📌';
    }
  };

  return (
    <div
      className={`briefing-section ${active ? 'active' : ''}`}
      data-testid={`briefing-${section.type}`}
    >
      <div className="section-header">
        <span className="section-icon">{getIcon(section.type)}</span>
        <h3>{section.title}</h3>
        <span className={`section-priority priority-${section.priority}`}>
          {section.priority}
        </span>
      </div>

      <div className="section-content">
        <p>{section.content}</p>
      </div>

      {section.articles && section.articles.length > 0 && (
        <div className="section-articles">
          {section.articles.map((article, idx) => (
            <div key={idx} className="article-item">
              <a href={article.url} target="_blank" rel="noopener noreferrer">
                {article.headline}
              </a>
            </div>
          ))}
        </div>
      )}

      {active && (
        <div className="playing-indicator" data-testid="playing-indicator">
          🔊 Playing...
        </div>
      )}
    </div>
  );
}
