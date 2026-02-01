/**
 * Follow-Up Questions Component - Display and respond to follow-up questions
 *
 * Contract: I-VOICE-004
 * Shows intelligent follow-up questions based on conversation context
 */

import React, { useState, useEffect } from 'react';
import { FollowUpQuestion } from '../../types/voice';
import { followUpGenerator } from '../../services/memory/FollowUpGenerator';

interface FollowUpQuestionsProps {
  userId: string;
  onAnswer?: (questionId: string, answer: string) => void;
}

export function FollowUpQuestions({ userId, onAnswer }: FollowUpQuestionsProps) {
  const [questions, setQuestions] = useState<FollowUpQuestion[]>([]);
  const [loading, setLoading] = useState(true);
  const [answering, setAnswering] = useState<string | null>(null);
  const [answerText, setAnswerText] = useState('');

  useEffect(() => {
    loadQuestions();
  }, [userId]);

  const loadQuestions = async () => {
    try {
      setLoading(true);
      const active = await followUpGenerator.getActiveQuestions(userId);
      setQuestions(active);
    } catch (err) {
      console.error('Failed to load follow-up questions:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleStartAnswering = (questionId: string) => {
    setAnswering(questionId);
    setAnswerText('');
  };

  const handleCancelAnswering = () => {
    setAnswering(null);
    setAnswerText('');
  };

  const handleSubmitAnswer = async (questionId: string) => {
    if (!answerText.trim()) return;

    try {
      await followUpGenerator.markQuestionAnswered(questionId, answerText);

      if (onAnswer) {
        onAnswer(questionId, answerText);
      }

      setAnswering(null);
      setAnswerText('');
      await loadQuestions();
    } catch (err) {
      console.error('Failed to submit answer:', err);
    }
  };

  const handleDismiss = async (questionId: string) => {
    try {
      await followUpGenerator.markQuestionAnswered(questionId, '[dismissed]');
      await loadQuestions();
    } catch (err) {
      console.error('Failed to dismiss question:', err);
    }
  };

  if (loading) {
    return (
      <div data-testid="followup-questions" className="followup-questions loading">
        <div className="spinner">Loading questions...</div>
      </div>
    );
  }

  if (questions.length === 0) {
    return (
      <div data-testid="followup-questions" className="followup-questions empty">
        <div className="empty-state">
          <p>No follow-up questions at the moment</p>
          <p className="subtitle">mBot will ask you questions based on your conversations</p>
        </div>
      </div>
    );
  }

  return (
    <div data-testid="followup-questions" className="followup-questions">
      <div className="questions-header">
        <h2>Follow-Up Questions</h2>
        <span className="question-count">{questions.length} pending</span>
      </div>

      <div className="questions-list">
        {questions.map((question) => (
          <QuestionCard
            key={question.id}
            question={question}
            answering={answering === question.id}
            answerText={answerText}
            onStartAnswering={() => handleStartAnswering(question.id)}
            onCancelAnswering={handleCancelAnswering}
            onAnswerChange={setAnswerText}
            onSubmitAnswer={() => handleSubmitAnswer(question.id)}
            onDismiss={() => handleDismiss(question.id)}
          />
        ))}
      </div>
    </div>
  );
}

interface QuestionCardProps {
  question: FollowUpQuestion;
  answering: boolean;
  answerText: string;
  onStartAnswering: () => void;
  onCancelAnswering: () => void;
  onAnswerChange: (text: string) => void;
  onSubmitAnswer: () => void;
  onDismiss: () => void;
}

function QuestionCard({
  question,
  answering,
  answerText,
  onStartAnswering,
  onCancelAnswering,
  onAnswerChange,
  onSubmitAnswer,
  onDismiss,
}: QuestionCardProps) {
  const getTimeRemaining = (): string => {
    const now = Date.now();
    const remaining = question.validUntil - now;
    const hours = Math.floor(remaining / (1000 * 60 * 60));
    const days = Math.floor(hours / 24);

    if (days > 1) return `${days} days`;
    if (hours > 1) return `${hours} hours`;
    return 'Soon';
  };

  const getPriorityClass = (): string => {
    if (question.priority >= 80) return 'high';
    if (question.priority >= 60) return 'medium';
    return 'low';
  };

  const getContextLabel = (): string => {
    const [type] = question.context.split(':');
    const labels: Record<string, string> = {
      yesterday_activity: 'Yesterday',
      ongoing_project: 'Project',
      previous_goal: 'Goal',
      upcoming_event: 'Event',
    };
    return labels[type] || 'Memory';
  };

  return (
    <div
      className={`question-card priority-${getPriorityClass()}`}
      data-testid={`followup-question-${question.id}`}
    >
      <div className="question-header">
        <div className="question-meta">
          <span className="context-badge">{getContextLabel()}</span>
          <span className="time-remaining">Expires in {getTimeRemaining()}</span>
        </div>
        <div className="question-priority">
          <PriorityIndicator priority={question.priority} />
        </div>
      </div>

      <div className="question-content">
        <p className="question-text">{question.question}</p>
      </div>

      {!answering ? (
        <div className="question-actions">
          <button onClick={onStartAnswering} className="answer-button primary">
            Answer
          </button>
          <button onClick={onDismiss} className="dismiss-button secondary">
            Not now
          </button>
        </div>
      ) : (
        <div className="answer-form">
          <textarea
            value={answerText}
            onChange={(e) => onAnswerChange(e.target.value)}
            placeholder="Type your answer..."
            className="answer-input"
            rows={3}
            autoFocus
          />
          <div className="form-actions">
            <button
              onClick={onSubmitAnswer}
              className="submit-button primary"
              disabled={!answerText.trim()}
            >
              Submit
            </button>
            <button onClick={onCancelAnswering} className="cancel-button secondary">
              Cancel
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

interface PriorityIndicatorProps {
  priority: number;
}

function PriorityIndicator({ priority }: PriorityIndicatorProps) {
  const getColor = (): string => {
    if (priority >= 80) return '#ef4444'; // red
    if (priority >= 60) return '#f59e0b'; // orange
    return '#10b981'; // green
  };

  const getLabel = (): string => {
    if (priority >= 80) return 'High';
    if (priority >= 60) return 'Medium';
    return 'Low';
  };

  return (
    <div className="priority-indicator" style={{ color: getColor() }}>
      <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
        <circle cx="8" cy="8" r="4" />
      </svg>
      <span>{getLabel()}</span>
    </div>
  );
}
