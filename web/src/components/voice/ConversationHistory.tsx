/**
 * Conversation History Component
 *
 * Displays past conversations with the voice assistant
 * data-testid: conversation-history
 */

import React, { useState, useEffect } from 'react';
import { Conversation } from '../../types/voice';
import { ConversationMemoryService } from '../../services/memory/ConversationMemoryService';

const memoryService = new ConversationMemoryService();

interface ConversationHistoryProps {
  userId: string;
}

export function ConversationHistory({ userId }: ConversationHistoryProps) {
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadConversations();
  }, [userId]);

  const loadConversations = async () => {
    setLoading(true);
    try {
      const conv = await memoryService.getConversations(userId, 10);
      setConversations(conv);
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return <div>Loading conversations...</div>;
  }

  return (
    <div data-testid="conversation-history" className="conversation-history">
      <h2>Conversation History</h2>

      {conversations.length === 0 && (
        <div className="empty-state">
          <p>No conversations yet.</p>
        </div>
      )}

      <div className="conversations-list">
        {conversations.map(conv => (
          <div key={conv.id} className="conversation-card">
            <div className="conversation-header">
              <span className="conversation-topic">{conv.topic}</span>
              <span className="conversation-date">
                {new Date(conv.timestamp).toLocaleDateString()}
              </span>
              <span className={`sentiment-badge sentiment-${conv.sentiment}`}>
                {conv.sentiment}
              </span>
            </div>

            <div className="conversation-turns">
              {conv.turns.map((turn, idx) => (
                <div
                  key={idx}
                  className={`turn turn-${turn.speaker}`}
                >
                  <span className="speaker">{turn.speaker}:</span>
                  <span className="text">{turn.text}</span>
                </div>
              ))}
            </div>

            {conv.keyPoints.length > 0 && (
              <div className="key-points">
                <strong>Key Points:</strong>
                <ul>
                  {conv.keyPoints.map((point, idx) => (
                    <li key={idx}>{point}</li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
