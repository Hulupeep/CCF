/**
 * Memory Timeline Component - Display daily activities timeline
 *
 * Contract: I-MEMORY-001
 * Shows user's activities across days with date picker
 */

import React, { useState, useEffect } from 'react';
import { DailyActivity } from '../../types/voice';
import { conversationMemoryService } from '../../services/memory/ConversationMemoryService';

interface MemoryTimelineProps {
  userId: string;
}

export function MemoryTimeline({ userId }: MemoryTimelineProps) {
  const [activities, setActivities] = useState<DailyActivity[]>([]);
  const [selectedDate, setSelectedDate] = useState<string>(getTodayString());
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadActivities();
  }, [userId, selectedDate]);

  const loadActivities = async () => {
    try {
      setLoading(true);
      setError(null);

      // Load 7 days of activities
      const endDate = selectedDate;
      const startDate = getDateString(
        new Date(new Date(selectedDate).getTime() - 6 * 24 * 60 * 60 * 1000)
      );

      const loadedActivities = await conversationMemoryService.getActivitiesInRange(
        userId,
        startDate,
        endDate
      );

      setActivities(loadedActivities);
    } catch (err) {
      console.error('Failed to load activities:', err);
      setError('Failed to load activities');
    } finally {
      setLoading(false);
    }
  };

  const handleDateChange = (date: string) => {
    setSelectedDate(date);
  };

  const handleMarkComplete = async (date: string, activity: string) => {
    try {
      await conversationMemoryService.markActivityCompleted(userId, date, activity);
      await loadActivities();
    } catch (err) {
      console.error('Failed to mark activity complete:', err);
    }
  };

  if (loading) {
    return (
      <div data-testid="activity-timeline" className="memory-timeline loading">
        <div className="spinner">Loading activities...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div data-testid="activity-timeline" className="memory-timeline error">
        <div className="error-message">{error}</div>
      </div>
    );
  }

  return (
    <div data-testid="activity-timeline" className="memory-timeline">
      <div className="timeline-header">
        <h2>Activity Timeline</h2>
        <DatePicker value={selectedDate} onChange={handleDateChange} />
      </div>

      <div className="timeline-content">
        {activities.length === 0 ? (
          <div className="empty-state">
            <p>No activities recorded for this period</p>
          </div>
        ) : (
          <ActivityList
            activities={activities}
            onMarkComplete={handleMarkComplete}
            selectedDate={selectedDate}
          />
        )}
      </div>
    </div>
  );
}

interface DatePickerProps {
  value: string;
  onChange: (date: string) => void;
}

function DatePicker({ value, onChange }: DatePickerProps) {
  const goBack = () => {
    const date = new Date(value);
    date.setDate(date.getDate() - 1);
    onChange(getDateString(date));
  };

  const goForward = () => {
    const date = new Date(value);
    date.setDate(date.getDate() + 1);
    const today = new Date();
    if (date <= today) {
      onChange(getDateString(date));
    }
  };

  const goToToday = () => {
    onChange(getTodayString());
  };

  const formatDisplayDate = (dateStr: string): string => {
    const date = new Date(dateStr);
    const today = new Date();
    const yesterday = new Date(today);
    yesterday.setDate(yesterday.getDate() - 1);

    if (dateStr === getDateString(today)) return 'Today';
    if (dateStr === getDateString(yesterday)) return 'Yesterday';

    return date.toLocaleDateString('en-US', {
      weekday: 'short',
      month: 'short',
      day: 'numeric',
    });
  };

  const isToday = value === getTodayString();

  return (
    <div className="date-picker" data-testid="date-picker">
      <button onClick={goBack} className="nav-button" aria-label="Previous day">
        ←
      </button>
      <div className="date-display">{formatDisplayDate(value)}</div>
      <button
        onClick={goForward}
        className="nav-button"
        disabled={isToday}
        aria-label="Next day"
      >
        →
      </button>
      {!isToday && (
        <button onClick={goToToday} className="today-button">
          Today
        </button>
      )}
    </div>
  );
}

interface ActivityListProps {
  activities: DailyActivity[];
  onMarkComplete: (date: string, activity: string) => void;
  selectedDate: string;
}

function ActivityList({ activities, onMarkComplete, selectedDate }: ActivityListProps) {
  return (
    <div className="activity-list">
      {activities.map((dailyActivity) => (
        <ActivityCard
          key={dailyActivity.date}
          activity={dailyActivity}
          onMarkComplete={onMarkComplete}
          isSelected={dailyActivity.date === selectedDate}
        />
      ))}
    </div>
  );
}

interface ActivityCardProps {
  activity: DailyActivity;
  onMarkComplete: (date: string, activity: string) => void;
  isSelected: boolean;
}

function ActivityCard({ activity, onMarkComplete, isSelected }: ActivityCardProps) {
  const formatDate = (dateStr: string): string => {
    const date = new Date(dateStr);
    return date.toLocaleDateString('en-US', {
      weekday: 'short',
      month: 'short',
      day: 'numeric',
    });
  };

  const getMoodEmoji = (mood?: string): string => {
    if (!mood) return '';
    const moodMap: Record<string, string> = {
      happy: '😊',
      excited: '🎉',
      sad: '😢',
      tired: '😴',
      focused: '🎯',
      relaxed: '😌',
    };
    return moodMap[mood.toLowerCase()] || '';
  };

  return (
    <div
      className={`activity-card ${isSelected ? 'selected' : ''}`}
      data-testid={`activity-timeline-${activity.date}`}
    >
      <div className="activity-header">
        <h3>{formatDate(activity.date)}</h3>
        {activity.mood && (
          <span className="mood-indicator" title={activity.mood}>
            {getMoodEmoji(activity.mood)}
          </span>
        )}
      </div>

      {activity.plannedActivities.length > 0 && (
        <div className="planned-activities">
          <h4>Planned:</h4>
          <ul>
            {activity.plannedActivities.map((act, index) => {
              const isCompleted = activity.completedActivities.includes(act);
              return (
                <li key={index} className={isCompleted ? 'completed' : 'pending'}>
                  <input
                    type="checkbox"
                    checked={isCompleted}
                    onChange={() => onMarkComplete(activity.date, act)}
                    id={`activity-${activity.date}-${index}`}
                  />
                  <label htmlFor={`activity-${activity.date}-${index}`}>{act}</label>
                </li>
              );
            })}
          </ul>
        </div>
      )}

      {activity.notes && (
        <div className="activity-notes">
          <h4>Notes:</h4>
          <p>{activity.notes}</p>
        </div>
      )}

      {activity.plannedActivities.length === 0 && !activity.notes && (
        <div className="empty-day">
          <p>No activities recorded</p>
        </div>
      )}
    </div>
  );
}

// Helper functions
function getTodayString(): string {
  return getDateString(new Date());
}

function getDateString(date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
}
