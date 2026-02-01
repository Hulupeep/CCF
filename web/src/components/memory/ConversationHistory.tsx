/**
 * Conversation History Component - Display and search past conversations
 *
 * Contract: I-VOICE-004
 * Shows conversation history with search functionality
 */

import React, { useState, useEffect } from 'react';
import { Conversation, ConversationTurn } from '../../types/voice';
import { conversationMemoryService } from '../../services/memory/ConversationMemoryService';

interface ConversationHistoryProps {
  userId: string;
  days?: number;
}

export function ConversationHistory({ userId, days = 7 }: ConversationHistoryProps) {
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [expandedConversations, setExpandedConversations] = useState<Set<string>>(
    new Set()
  );

  useEffect(() => {
    loadConversations();
  }, [userId, days]);

  useEffect(() => {
    if (searchQuery) {
      searchConversations();
    } else {
      loadConversations();
    }
  }, [searchQuery]);

  const loadConversations = async () => {
    try {
      setLoading(true);
      setError(null);
      const loaded = await conversationMemoryService.getConversations(userId, days);
      setConversations(loaded);
    } catch (err) {
      console.error('Failed to load conversations:', err);
      setError('Failed to load conversations');
    } finally {
      setLoading(false);
    }
  };

  const searchConversations = async () => {
    try {
      setLoading(true);
      setError(null);
      const results = await conversationMemoryService.searchConversations(
        userId,
        searchQuery
      );
      setConversations(results);
    } catch (err) {
      console.error('Failed to search conversations:', err);
      setError('Search failed');
    } finally {
      setLoading(false);
    }
  };

  const toggleConversation = (id: string) => {
    const newExpanded = new Set(expandedConversations);
    if (newExpanded.has(id)) {
      newExpanded.delete(id);
    } else {
      newExpanded.add(id);
    }
    setExpandedConversations(newExpanded);
  };

  if (loading) {
    return (
      <div data-testid="conversation-history" className="conversation-history loading">
        <div className="spinner">Loading conversations...</div>
      </div>
    );
  }

  return (
    <div data-testid="conversation-history" className="conversation-history">
      <div className="history-header">
        <h2>Conversation History</h2>
        <p className="subtitle">Last {days} days</p>
      </div>

      <SearchBar
        value={searchQuery}
        onChange={setSearchQuery}
        placeholder="Search conversations..."
      />

      {error && <div className="error-message">{error}</div>}

      <div className="conversations-list">
        {conversations.length === 0 ? (
          <div className="empty-state">
            <p>
              {searchQuery
                ? 'No conversations match your search'
                : 'No conversations yet'}
            </p>
          </div>
        ) : (
          conversations.map((conv) => (
            <ConversationCard
              key={conv.id}
              conversation={conv}
              expanded={expandedConversations.has(conv.id)}
              onToggle={() => toggleConversation(conv.id)}
            />
          ))
        )}
      </div>
    </div>
  );
}

interface SearchBarProps {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
}

function SearchBar({ value, onChange, placeholder = 'Search...' }: SearchBarProps) {
  return (
    <div className="search-bar" data-testid="conversation-search">
      <input
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        className="search-input"
      />
      {value && (
        <button onClick={() => onChange('')} className="clear-button" aria-label="Clear">
          ✕
        </button>
      )}
    </div>
  );
}

interface ConversationCardProps {
  conversation: Conversation;
  expanded: boolean;
  onToggle: () => void;
}

function ConversationCard({ conversation, expanded, onToggle }: ConversationCardProps) {
  const formatTimestamp = (timestamp: number): string => {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - timestamp;
    const diffHours = diffMs / (1000 * 60 * 60);

    if (diffHours < 1) {
      const diffMins = Math.floor(diffMs / (1000 * 60));
      return `${diffMins} minute${diffMins !== 1 ? 's' : ''} ago`;
    }

    if (diffHours < 24) {
      const hours = Math.floor(diffHours);
      return `${hours} hour${hours !== 1 ? 's' : ''} ago`;
    }

    const diffDays = Math.floor(diffHours / 24);
    if (diffDays < 7) {
      return `${diffDays} day${diffDays !== 1 ? 's' : ''} ago`;
    }

    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
      year: date.getFullYear() !== now.getFullYear() ? 'numeric' : undefined,
    });
  };

  const getSentimentEmoji = (sentiment: string): string => {
    const emojiMap: Record<string, string> = {
      positive: '😊',
      neutral: '😐',
      negative: '😕',
    };
    return emojiMap[sentiment] || '😐';
  };

  const getPreview = (): string => {
    const firstUserTurn = conversation.turns.find((turn) => turn.speaker === 'user');
    if (!firstUserTurn) return 'No user messages';
    return firstUserTurn.text.length > 100
      ? firstUserTurn.text.substring(0, 100) + '...'
      : firstUserTurn.text;
  };

  return (
    <div
      className={`conversation-card ${expanded ? 'expanded' : ''}`}
      data-testid={`conversation-${conversation.id}`}
    >
      <div className="card-header" onClick={onToggle}>
        <div className="header-left">
          <span className="sentiment-indicator">
            {getSentimentEmoji(conversation.sentiment)}
          </span>
          <div className="conversation-info">
            <h3 className="conversation-topic">{conversation.topic}</h3>
            <p className="conversation-time">{formatTimestamp(conversation.timestamp)}</p>
          </div>
        </div>
        <div className="header-right">
          <span className="turn-count">{conversation.turns.length} messages</span>
          <button className="expand-button" aria-label={expanded ? 'Collapse' : 'Expand'}>
            {expanded ? '▼' : '▶'}
          </button>
        </div>
      </div>

      {!expanded && (
        <div className="conversation-preview">
          <p>{getPreview()}</p>
        </div>
      )}

      {expanded && (
        <div className="conversation-details">
          <div className="conversation-turns">
            {conversation.turns.map((turn, index) => (
              <ConversationTurnView key={index} turn={turn} />
            ))}
          </div>

          {conversation.keyPoints.length > 0 && (
            <div className="key-points">
              <h4>Key Points:</h4>
              <ul>
                {conversation.keyPoints.map((point, index) => (
                  <li key={index}>{point}</li>
                ))}
              </ul>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

interface ConversationTurnViewProps {
  turn: ConversationTurn;
}

function ConversationTurnView({ turn }: ConversationTurnViewProps) {
  const formatTime = (timestamp: number): string => {
    const date = new Date(timestamp);
    return date.toLocaleTimeString('en-US', {
      hour: 'numeric',
      minute: '2-digit',
    });
  };

  return (
    <div className={`conversation-turn ${turn.speaker}`}>
      <div className="turn-header">
        <span className="speaker-label">
          {turn.speaker === 'user' ? 'You' : 'mBot'}
        </span>
        <span className="turn-time">{formatTime(turn.timestamp)}</span>
      </div>
      <div className="turn-text">{turn.text}</div>
      {turn.entities && Object.keys(turn.entities).length > 0 && (
        <div className="turn-entities">
          {Object.entries(turn.entities).map(([key, value]) => (
            <span key={key} className="entity-badge">
              {key}: {JSON.stringify(value)}
            </span>
          ))}
        </div>
      )}
    </div>
  );
}
