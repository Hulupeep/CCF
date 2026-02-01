/**
 * Memory Timeline Component
 *
 * Displays daily activities timeline
 * data-testid: activity-timeline-{date}
 */

import React, { useState, useEffect } from 'react';
import { DailyActivity } from '../../types/voice';
import { ConversationMemoryService } from '../../services/memory/ConversationMemoryService';

const memoryService = new ConversationMemoryService();

interface MemoryTimelineProps {
  userId: string;
}

export function MemoryTimeline({ userId }: MemoryTimelineProps) {
  const [activities, setActivities] = useState<DailyActivity[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadActivities();
  }, [userId]);

  const loadActivities = async () => {
    setLoading(true);
    try {
      const acts = await memoryService.getActivityHistory(userId, 7);
      setActivities(acts);
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return <div>Loading memory timeline...</div>;
  }

  return (
    <div className="memory-timeline">
      <h2>Activity Timeline</h2>

      {activities.length === 0 && (
        <div className="empty-state">
          <p>No activity history yet.</p>
        </div>
      )}

      <div className="timeline">
        {activities.map(activity => (
          <div
            key={activity.date}
            className="timeline-item"
            data-testid={`activity-timeline-${activity.date}`}
          >
            <div className="timeline-date">
              <span className="date-label">
                {formatDate(activity.date)}
              </span>
            </div>

            <div className="timeline-content">
              {activity.plannedActivities.length > 0 && (
                <div className="planned-activities">
                  <strong>Planned:</strong>
                  <ul>
                    {activity.plannedActivities.map((act, idx) => (
                      <li key={idx}>{act}</li>
                    ))}
                  </ul>
                </div>
              )}

              {activity.completedActivities.length > 0 && (
                <div className="completed-activities">
                  <strong>Completed:</strong>
                  <ul>
                    {activity.completedActivities.map((act, idx) => (
                      <li key={idx}>✅ {act}</li>
                    ))}
                  </ul>
                </div>
              )}

              {activity.mood && (
                <div className="mood">
                  Mood: <span className="mood-indicator">{activity.mood}</span>
                </div>
              )}

              {activity.notes && (
                <div className="notes">
                  <em>{activity.notes}</em>
                </div>
              )}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

function formatDate(dateStr: string): string {
  const date = new Date(dateStr);
  const today = new Date();
  const yesterday = new Date(today);
  yesterday.setDate(yesterday.getDate() - 1);

  if (date.toDateString() === today.toDateString()) {
    return 'Today';
  } else if (date.toDateString() === yesterday.toDateString()) {
    return 'Yesterday';
  } else {
    return date.toLocaleDateString('en-US', { weekday: 'short', month: 'short', day: 'numeric' });
  }
}
