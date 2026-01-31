/**
 * Undo/redo history hook for personality configurations
 * Implements undo/redo stack with max 50 states
 */

import { useState, useCallback } from 'react';
import { PersonalityConfig } from '../types/personality';

const MAX_HISTORY_SIZE = 50;

export interface UsePersonalityHistoryOptions {
  initialConfig: PersonalityConfig;
  maxSize?: number;
}

export function usePersonalityHistory(options: UsePersonalityHistoryOptions) {
  const { initialConfig, maxSize = MAX_HISTORY_SIZE } = options;

  const [history, setHistory] = useState<PersonalityConfig[]>([initialConfig]);
  const [currentIndex, setCurrentIndex] = useState(0);

  const canUndo = currentIndex > 0;
  const canRedo = currentIndex < history.length - 1;

  // Add new state to history
  const pushState = useCallback(
    (config: PersonalityConfig) => {
      setHistory((prev) => {
        // Remove any states after current index (when adding after undo)
        const newHistory = prev.slice(0, currentIndex + 1);

        // Add new state
        newHistory.push(config);

        // Trim if exceeds max size (keep most recent)
        if (newHistory.length > maxSize) {
          return newHistory.slice(newHistory.length - maxSize);
        }

        return newHistory;
      });

      setCurrentIndex((prev) => {
        const newIndex = prev + 1;
        return Math.min(newIndex, maxSize - 1);
      });
    },
    [currentIndex, maxSize]
  );

  // Undo to previous state
  const undo = useCallback(() => {
    if (canUndo) {
      setCurrentIndex((prev) => prev - 1);
      return history[currentIndex - 1];
    }
    return null;
  }, [canUndo, currentIndex, history]);

  // Redo to next state
  const redo = useCallback(() => {
    if (canRedo) {
      setCurrentIndex((prev) => prev + 1);
      return history[currentIndex + 1];
    }
    return null;
  }, [canRedo, currentIndex, history]);

  // Get current state
  const getCurrentState = useCallback(() => {
    return history[currentIndex];
  }, [history, currentIndex]);

  // Clear history
  const clearHistory = useCallback((newInitial: PersonalityConfig) => {
    setHistory([newInitial]);
    setCurrentIndex(0);
  }, []);

  return {
    canUndo,
    canRedo,
    pushState,
    undo,
    redo,
    getCurrentState,
    clearHistory,
    historySize: history.length,
    currentIndex,
  };
}
