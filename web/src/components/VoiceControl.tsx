/**
 * VoiceControl Component - Voice command UI
 * Contract: feature_voice.yml (all contracts)
 * Invariants:
 * - I-VOICE-001: Visual feedback for command execution
 * - I-VOICE-002: Display confidence levels
 * - I-VOICE-003: Non-blocking UI feedback
 *
 * Issue: #89 - Voice Command System
 */

import React, { useState, useEffect, useCallback } from 'react';
import { voiceCommandService } from '../services/voiceCommands';
import {
  ListeningState,
  VoiceCommandHistory,
  VoiceSettings,
  ConfirmationState,
  checkBrowserCompatibility,
} from '../types/voiceCommand';

interface VoiceControlProps {
  className?: string;
}

export const VoiceControl: React.FC<VoiceControlProps> = ({ className = '' }) => {
  const [isSupported, setIsSupported] = useState(true);
  const [listeningState, setListeningState] = useState<ListeningState>('idle');
  const [settings, setSettings] = useState<VoiceSettings>(
    voiceCommandService.getSettings()
  );
  const [history, setHistory] = useState<VoiceCommandHistory[]>([]);
  const [confirmationState, setConfirmationState] = useState<ConfirmationState | null>(
    null
  );
  const [lastTranscript, setLastTranscript] = useState<string>('');
  const [showSettings, setShowSettings] = useState(false);
  const [showHistory, setShowHistory] = useState(false);

  // Check browser compatibility on mount
  useEffect(() => {
    const compatibility = checkBrowserCompatibility();
    setIsSupported(compatibility.supported);

    if (!compatibility.supported) {
      console.warn(compatibility.message);
    }
  }, []);

  // Subscribe to state changes
  useEffect(() => {
    const unsubscribe = voiceCommandService.subscribeToStateChanges(
      (state: ListeningState) => {
        setListeningState(state);
      }
    );

    return unsubscribe;
  }, []);

  // Subscribe to history changes
  useEffect(() => {
    const unsubscribe = voiceCommandService.subscribeToHistoryChanges(() => {
      setHistory(voiceCommandService.getCommandHistory());
    });

    // Initial load
    setHistory(voiceCommandService.getCommandHistory());

    return unsubscribe;
  }, []);

  // Poll confirmation state (since it's not event-based)
  useEffect(() => {
    const interval = setInterval(() => {
      setConfirmationState(voiceCommandService.getConfirmationState());
    }, 100);

    return () => clearInterval(interval);
  }, []);

  // Handle toggle voice recognition
  const handleToggleVoice = useCallback(async () => {
    if (!isSupported) {
      alert('Voice commands are not supported in this browser');
      return;
    }

    try {
      if (voiceCommandService.isListening()) {
        voiceCommandService.stopListening();
      } else {
        await voiceCommandService.startListening();
      }
    } catch (error) {
      console.error('Failed to toggle voice recognition:', error);
      alert('Failed to access microphone. Please check permissions.');
    }
  }, [isSupported]);

  // Handle settings update
  const handleUpdateSettings = useCallback(
    (updates: Partial<VoiceSettings>) => {
      try {
        voiceCommandService.updateSettings(updates);
        setSettings(voiceCommandService.getSettings());
      } catch (error) {
        console.error('Failed to update settings:', error);
        alert('Invalid settings');
      }
    },
    []
  );

  // Handle clear history
  const handleClearHistory = useCallback(() => {
    if (confirm('Are you sure you want to clear command history?')) {
      voiceCommandService.clearHistory();
    }
  }, []);

  // Update last transcript from history
  useEffect(() => {
    if (history.length > 0) {
      setLastTranscript(history[0].transcript);
    }
  }, [history]);

  // Get listening indicator color
  const getIndicatorColor = (): string => {
    switch (listeningState) {
      case 'listening':
        return 'bg-green-500';
      case 'processing':
        return 'bg-yellow-500';
      case 'error':
        return 'bg-red-500';
      default:
        return 'bg-gray-400';
    }
  };

  // Get listening indicator animation
  const getIndicatorAnimation = (): string => {
    return listeningState === 'listening' ? 'animate-pulse' : '';
  };

  if (!isSupported) {
    return (
      <div className={`voice-control ${className}`}>
        <div className="text-red-500 text-sm p-4 bg-red-50 rounded">
          Voice commands are not supported in this browser. Please use Chrome, Edge, or Safari.
        </div>
      </div>
    );
  }

  return (
    <div className={`voice-control ${className}`} data-testid="voice-control">
      {/* Main Control */}
      <div className="flex items-center gap-4 p-4 bg-white rounded-lg shadow">
        {/* Microphone Button */}
        <button
          onClick={handleToggleVoice}
          data-testid="voice-toggle"
          className={`p-3 rounded-full transition-colors ${
            voiceCommandService.isListening()
              ? 'bg-green-500 hover:bg-green-600'
              : 'bg-gray-300 hover:bg-gray-400'
          }`}
          aria-label={voiceCommandService.isListening() ? 'Stop listening' : 'Start listening'}
        >
          <svg
            data-testid="microphone-icon"
            className="w-6 h-6 text-white"
            fill="currentColor"
            viewBox="0 0 20 20"
          >
            <path d="M7 4a3 3 0 016 0v6a3 3 0 11-6 0V4zm4 10.93A7.001 7.001 0 0017 8a1 1 0 10-2 0A5 5 0 015 8a1 1 0 00-2 0 7.001 7.001 0 006 6.93V17H6a1 1 0 100 2h8a1 1 0 100-2h-3v-2.07z" />
          </svg>
        </button>

        {/* Listening Indicator */}
        <div className="flex items-center gap-2">
          <div
            data-testid="listening-indicator"
            className={`w-3 h-3 rounded-full ${getIndicatorColor()} ${getIndicatorAnimation()}`}
          />
          <span className="text-sm text-gray-600">
            {listeningState === 'listening' && 'Listening...'}
            {listeningState === 'processing' && 'Processing...'}
            {listeningState === 'error' && 'Error'}
            {listeningState === 'idle' && 'Ready'}
          </span>
        </div>

        {/* Settings Toggle */}
        <button
          onClick={() => setShowSettings(!showSettings)}
          className="ml-auto p-2 text-gray-600 hover:text-gray-800"
          aria-label="Settings"
        >
          <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
            <path d="M10 6a2 2 0 110-4 2 2 0 010 4zM10 12a2 2 0 110-4 2 2 0 010 4zM10 18a2 2 0 110-4 2 2 0 010 4z" />
          </svg>
        </button>

        {/* History Toggle */}
        <button
          onClick={() => setShowHistory(!showHistory)}
          className="p-2 text-gray-600 hover:text-gray-800"
          aria-label="History"
        >
          <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
            <path d="M9 2a1 1 0 000 2h2a1 1 0 100-2H9z" />
            <path
              fillRule="evenodd"
              d="M4 5a2 2 0 012-2 3 3 0 003 3h2a3 3 0 003-3 2 2 0 012 2v11a2 2 0 01-2 2H6a2 2 0 01-2-2V5zm3 4a1 1 0 000 2h.01a1 1 0 100-2H7zm3 0a1 1 0 000 2h3a1 1 0 100-2h-3zm-3 4a1 1 0 100 2h.01a1 1 0 100-2H7zm3 0a1 1 0 100 2h3a1 1 0 100-2h-3z"
              clipRule="evenodd"
            />
          </svg>
        </button>
      </div>

      {/* Transcript Display */}
      {lastTranscript && (
        <div className="mt-4 p-3 bg-blue-50 rounded" data-testid="recognized-text">
          <div className="text-xs text-blue-600 font-semibold mb-1">Last Command:</div>
          <div className="text-sm text-gray-800">{lastTranscript}</div>
        </div>
      )}

      {/* Confirmation Prompt */}
      {confirmationState?.active && (
        <div className="mt-4 p-4 bg-yellow-50 border-2 border-yellow-400 rounded" data-testid="confirmation-prompt">
          <div className="text-sm font-semibold text-yellow-800 mb-2">
            Confirmation Required
          </div>
          <div className="text-sm text-gray-700 mb-3">
            Are you sure you want to: "{confirmationState.command}"?
          </div>
          <div className="text-xs text-gray-600">
            Say "yes" to confirm or "no" to cancel
          </div>
        </div>
      )}

      {/* Confidence Meter */}
      {history.length > 0 && history[0].confidence && (
        <div className="mt-4" data-testid="voice-confidence">
          <div className="text-xs text-gray-600 mb-1">
            Confidence: {Math.round(history[0].confidence * 100)}%
          </div>
          <div className="w-full bg-gray-200 rounded-full h-2">
            <div
              className={`h-2 rounded-full ${
                history[0].confidence >= 0.8
                  ? 'bg-green-500'
                  : history[0].confidence >= 0.5
                  ? 'bg-yellow-500'
                  : 'bg-red-500'
              }`}
              style={{ width: `${history[0].confidence * 100}%` }}
            />
          </div>
        </div>
      )}

      {/* Settings Panel */}
      {showSettings && (
        <div className="mt-4 p-4 bg-gray-50 rounded" data-testid="settings-panel">
          <h3 className="text-sm font-semibold mb-3">Voice Settings</h3>

          {/* Wake Word Toggle */}
          <label className="flex items-center gap-2 mb-3">
            <input
              type="checkbox"
              checked={settings.wakeWordEnabled}
              onChange={(e) =>
                handleUpdateSettings({ wakeWordEnabled: e.target.checked })
              }
              data-testid="wake-word-toggle"
              className="rounded"
            />
            <span className="text-sm">Enable Wake Word</span>
          </label>

          {/* Wake Word Input */}
          {settings.wakeWordEnabled && (
            <div className="mb-3">
              <label className="block text-xs text-gray-600 mb-1">Wake Word:</label>
              <input
                type="text"
                value={settings.wakeWord}
                onChange={(e) => handleUpdateSettings({ wakeWord: e.target.value })}
                data-testid="wake-word-input"
                className="w-full px-3 py-1 text-sm border rounded"
                placeholder="e.g., hey robot"
              />
            </div>
          )}

          {/* Audio Feedback Toggle */}
          <label className="flex items-center gap-2 mb-3">
            <input
              type="checkbox"
              checked={settings.audioFeedback}
              onChange={(e) =>
                handleUpdateSettings({ audioFeedback: e.target.checked })
              }
              data-testid="audio-feedback-toggle"
              className="rounded"
            />
            <span className="text-sm">Audio Feedback (Beeps)</span>
          </label>

          {/* Language Selector */}
          <div className="mb-3">
            <label className="block text-xs text-gray-600 mb-1">Language:</label>
            <select
              value={settings.language}
              onChange={(e) => handleUpdateSettings({ language: e.target.value })}
              data-testid="language-selector"
              className="w-full px-3 py-1 text-sm border rounded"
            >
              <option value="en-US">English (US)</option>
              <option value="en-GB">English (UK)</option>
              <option value="es-ES">Spanish</option>
              <option value="fr-FR">French</option>
              <option value="de-DE">German</option>
            </select>
          </div>

          {/* Confidence Threshold */}
          <div className="mb-3">
            <label className="block text-xs text-gray-600 mb-1">
              Confidence Threshold: {settings.confidenceThreshold.toFixed(2)}
            </label>
            <input
              type="range"
              min="0"
              max="1"
              step="0.05"
              value={settings.confidenceThreshold}
              onChange={(e) =>
                handleUpdateSettings({
                  confidenceThreshold: parseFloat(e.target.value),
                })
              }
              className="w-full"
            />
          </div>

          {/* Noise Threshold */}
          <div>
            <label className="block text-xs text-gray-600 mb-1">
              Noise Threshold: {settings.noiseThreshold} dB
            </label>
            <input
              type="range"
              min="0"
              max="120"
              step="5"
              value={settings.noiseThreshold}
              onChange={(e) =>
                handleUpdateSettings({
                  noiseThreshold: parseInt(e.target.value, 10),
                })
              }
              className="w-full"
            />
          </div>
        </div>
      )}

      {/* History Panel */}
      {showHistory && (
        <div className="mt-4 p-4 bg-gray-50 rounded" data-testid="command-history">
          <div className="flex items-center justify-between mb-3">
            <h3 className="text-sm font-semibold">Command History</h3>
            <button
              onClick={handleClearHistory}
              data-testid="clear-history-btn"
              className="text-xs text-red-600 hover:text-red-800"
            >
              Clear
            </button>
          </div>

          {history.length === 0 ? (
            <div className="text-sm text-gray-500">No commands yet</div>
          ) : (
            <div className="space-y-2 max-h-64 overflow-y-auto">
              {history.map((entry) => (
                <div
                  key={entry.id}
                  data-testid={`history-item-${entry.id}`}
                  className={`p-2 rounded text-sm ${
                    entry.recognized ? 'bg-green-50' : 'bg-red-50'
                  }`}
                >
                  <div className="flex items-center justify-between mb-1">
                    <span className="font-medium">{entry.transcript}</span>
                    <span className="text-xs text-gray-500">
                      {entry.executionTime}ms
                    </span>
                  </div>
                  <div className="text-xs text-gray-600">
                    {entry.recognized ? (
                      <>
                        ✓ {entry.action} ({Math.round(entry.confidence * 100)}%)
                      </>
                    ) : (
                      <>✗ Not recognized</>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      {/* Feedback Message */}
      {listeningState === 'processing' && (
        <div
          className="mt-4 p-3 bg-yellow-50 text-yellow-800 text-sm rounded animate-pulse"
          data-testid="voice-feedback"
        >
          Processing command...
        </div>
      )}

      {listeningState === 'error' && (
        <div
          className="mt-4 p-3 bg-red-50 text-red-800 text-sm rounded"
          data-testid="voice-feedback"
        >
          Error: Please try again
        </div>
      )}
    </div>
  );
};

export default VoiceControl;
