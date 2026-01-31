/**
 * Personality Mixer Component
 * Issue #58 - STORY-PERS-008
 *
 * Implements:
 * - 9 personality parameter sliders (I-PERS-UI-001: bounded 0.0-1.0)
 * - 15 preset personality buttons with hover previews
 * - Real-time WebSocket updates (I-PERS-UI-002: debounced 20Hz display, 2Hz send)
 * - Save/load custom personalities to localStorage
 * - Undo/redo stack (max 50 states)
 * - Connection state handling (I-PERS-UI-003: disable when disconnected)
 * - ARCH-004 contract compliance (bounded parameters)
 */

import React, { useState, useCallback, useEffect, useMemo } from 'react';
import {
  PersonalityConfig,
  createDefaultConfig,
  clampParameter,
  validatePersonalityConfig,
  PARAMETER_METADATA,
  CustomPersonality,
} from '../types/personality';
import { PERSONALITY_PRESETS } from '../types/presets';
import { usePersonalityWebSocket } from '../hooks/usePersonalityWebSocket';
import { usePersonalityHistory } from '../hooks/usePersonalityHistory';
import { useLocalStorage } from '../hooks/useLocalStorage';

interface PersonalityMixerProps {
  wsUrl?: string;
  onConfigChange?: (config: PersonalityConfig) => void;
  className?: string;
}

export const PersonalityMixer: React.FC<PersonalityMixerProps> = ({
  wsUrl = 'ws://localhost:8081',
  onConfigChange,
  className = '',
}) => {
  const initialConfig = useMemo(() => createDefaultConfig(), []);
  const [currentConfig, setCurrentConfig] = useState<PersonalityConfig>(initialConfig);
  const [activePreset, setActivePreset] = useState<string | null>(null);
  const [hoveredPreset, setHoveredPreset] = useState<string | null>(null);
  const [customPersonalities, setCustomPersonalities] = useLocalStorage<CustomPersonality[]>(
    'mbot-custom-personalities',
    []
  );
  const [saveDialogOpen, setSaveDialogOpen] = useState(false);
  const [customName, setCustomName] = useState('');

  const { connectionStatus, sendUpdate, sendImmediate } = usePersonalityWebSocket({
    url: wsUrl,
    autoConnect: true,
  });

  const {
    canUndo,
    canRedo,
    pushState,
    undo,
    redo,
    getCurrentState,
  } = usePersonalityHistory({
    initialConfig,
  });

  const isConnected = connectionStatus === 'connected';

  // Handle parameter change from slider
  const handleParameterChange = useCallback(
    (key: keyof PersonalityConfig, rawValue: number) => {
      const value = clampParameter(rawValue);

      setCurrentConfig((prev) => {
        const newConfig = { ...prev, [key]: value };

        // Validate before applying
        if (validatePersonalityConfig(newConfig)) {
          // Send update via WebSocket (debounced)
          sendUpdate({ [key]: value });

          // Notify parent
          onConfigChange?.(newConfig);

          return newConfig;
        }

        return prev;
      });

      setActivePreset(null); // Clear active preset when manually adjusting
    },
    [sendUpdate, onConfigChange]
  );

  // Handle parameter change complete (for undo/redo)
  const handleParameterChangeComplete = useCallback(
    (config: PersonalityConfig) => {
      pushState(config);
    },
    [pushState]
  );

  // Load preset personality
  const handleLoadPreset = useCallback(
    (presetId: string) => {
      const preset = PERSONALITY_PRESETS.find((p) => p.id === presetId);
      if (!preset) return;

      const newConfig = preset.config;
      setCurrentConfig(newConfig);
      setActivePreset(presetId);

      // Send immediately (not debounced)
      sendImmediate(newConfig);

      // Add to history
      pushState(newConfig);

      // Notify parent
      onConfigChange?.(newConfig);
    },
    [sendImmediate, pushState, onConfigChange]
  );

  // Load custom personality
  const handleLoadCustom = useCallback(
    (custom: CustomPersonality) => {
      const newConfig = custom.config;
      setCurrentConfig(newConfig);
      setActivePreset(null);

      sendImmediate(newConfig);
      pushState(newConfig);
      onConfigChange?.(newConfig);
    },
    [sendImmediate, pushState, onConfigChange]
  );

  // Save custom personality
  const handleSaveCustom = useCallback(() => {
    if (!customName.trim()) return;

    const custom: CustomPersonality = {
      name: customName.trim(),
      config: currentConfig,
      created_at: Date.now(),
    };

    setCustomPersonalities((prev) => [...prev, custom]);
    setSaveDialogOpen(false);
    setCustomName('');
  }, [customName, currentConfig, setCustomPersonalities]);

  // Delete custom personality
  const handleDeleteCustom = useCallback(
    (index: number) => {
      setCustomPersonalities((prev) => prev.filter((_, i) => i !== index));
    },
    [setCustomPersonalities]
  );

  // Reset to default
  const handleReset = useCallback(() => {
    const defaultConfig = createDefaultConfig();
    setCurrentConfig(defaultConfig);
    setActivePreset(null);
    sendImmediate(defaultConfig);
    pushState(defaultConfig);
    onConfigChange?.(defaultConfig);
  }, [sendImmediate, pushState, onConfigChange]);

  // Randomize parameters
  const handleRandomize = useCallback(() => {
    const randomConfig: PersonalityConfig = {
      tension_baseline: Math.random(),
      coherence_baseline: Math.random(),
      energy_baseline: Math.random(),
      startle_sensitivity: Math.random(),
      recovery_speed: Math.random(),
      curiosity_drive: Math.random(),
      movement_expressiveness: Math.random(),
      sound_expressiveness: Math.random(),
      light_expressiveness: Math.random(),
    };

    setCurrentConfig(randomConfig);
    setActivePreset(null);
    sendImmediate(randomConfig);
    pushState(randomConfig);
    onConfigChange?.(randomConfig);
  }, [sendImmediate, pushState, onConfigChange]);

  // Undo
  const handleUndo = useCallback(() => {
    const prevState = undo();
    if (prevState) {
      setCurrentConfig(prevState);
      setActivePreset(null);
      sendImmediate(prevState);
      onConfigChange?.(prevState);
    }
  }, [undo, sendImmediate, onConfigChange]);

  // Redo
  const handleRedo = useCallback(() => {
    const nextState = redo();
    if (nextState) {
      setCurrentConfig(nextState);
      setActivePreset(null);
      sendImmediate(nextState);
      onConfigChange?.(nextState);
    }
  }, [redo, sendImmediate, onConfigChange]);

  // Group parameters by category
  const parametersByCategory = useMemo(() => {
    const grouped: Record<string, typeof PARAMETER_METADATA> = {
      baselines: [],
      reactivity: [],
      expression: [],
    };

    PARAMETER_METADATA.forEach((param) => {
      grouped[param.category].push(param);
    });

    return grouped;
  }, []);

  // Preview config for hovered preset
  const previewConfig = useMemo(() => {
    if (!hoveredPreset) return null;
    const preset = PERSONALITY_PRESETS.find((p) => p.id === hoveredPreset);
    return preset?.config || null;
  }, [hoveredPreset]);

  return (
    <div className={`personality-mixer ${className}`} data-testid="personality-mixer">
      {/* Connection Status */}
      <div className="connection-status" data-testid="connection-status">
        <span className={`status-dot status-${connectionStatus}`} />
        <span className="status-text">
          {connectionStatus === 'connected' && 'Connected to robot'}
          {connectionStatus === 'connecting' && 'Connecting...'}
          {connectionStatus === 'disconnected' && 'Disconnected'}
        </span>
      </div>

      <div className="mixer-grid">
        {/* Parameters Panel */}
        <div className="parameters-panel">
          <h2>Personality Parameters</h2>

          {/* Undo/Redo Controls */}
          <div className="history-controls">
            <button
              onClick={handleUndo}
              disabled={!canUndo || !isConnected}
              data-testid="undo-button"
              title="Undo (Ctrl+Z)"
            >
              ↶ Undo
            </button>
            <button
              onClick={handleRedo}
              disabled={!canRedo || !isConnected}
              data-testid="redo-button"
              title="Redo (Ctrl+Shift+Z)"
            >
              ↷ Redo
            </button>
          </div>

          {/* Baselines */}
          <div className="parameter-category">
            <h3>⚖️ Baselines (where it wants to be)</h3>
            {parametersByCategory.baselines.map((param) => (
              <ParameterSlider
                key={param.key}
                param={param}
                value={currentConfig[param.key]}
                previewValue={previewConfig?.[param.key]}
                onChange={handleParameterChange}
                onChangeComplete={() => handleParameterChangeComplete(currentConfig)}
                disabled={!isConnected}
              />
            ))}
          </div>

          {/* Reactivity */}
          <div className="parameter-category">
            <h3>⚡ Reactivity (how stimuli affect it)</h3>
            {parametersByCategory.reactivity.map((param) => (
              <ParameterSlider
                key={param.key}
                param={param}
                value={currentConfig[param.key]}
                previewValue={previewConfig?.[param.key]}
                onChange={handleParameterChange}
                onChangeComplete={() => handleParameterChangeComplete(currentConfig)}
                disabled={!isConnected}
              />
            ))}
          </div>

          {/* Expression */}
          <div className="parameter-category">
            <h3>🎭 Expression (how it shows feelings)</h3>
            {parametersByCategory.expression.map((param) => (
              <ParameterSlider
                key={param.key}
                param={param}
                value={currentConfig[param.key]}
                previewValue={previewConfig?.[param.key]}
                onChange={handleParameterChange}
                onChangeComplete={() => handleParameterChangeComplete(currentConfig)}
                disabled={!isConnected}
              />
            ))}
          </div>

          {/* Action Buttons */}
          <div className="action-buttons">
            <button
              onClick={handleReset}
              disabled={!isConnected}
              data-testid="reset-button"
              className="btn-secondary"
            >
              ↺ Reset
            </button>
            <button
              onClick={handleRandomize}
              disabled={!isConnected}
              data-testid="randomize-button"
              className="btn-secondary"
            >
              🎲 Randomize
            </button>
            <button
              onClick={() => setSaveDialogOpen(true)}
              disabled={!isConnected}
              data-testid="save-custom-button"
              className="btn-primary"
            >
              💾 Save Custom
            </button>
          </div>
        </div>

        {/* Presets Panel */}
        <div className="presets-panel">
          <h2>Personality Presets</h2>
          <div className="preset-grid">
            {PERSONALITY_PRESETS.map((preset) => (
              <button
                key={preset.id}
                onClick={() => handleLoadPreset(preset.id)}
                onMouseEnter={() => setHoveredPreset(preset.id)}
                onMouseLeave={() => setHoveredPreset(null)}
                disabled={!isConnected}
                data-testid={`preset-button-${preset.id}`}
                className={`preset-button ${activePreset === preset.id ? 'active' : ''}`}
              >
                <div className="preset-icon">{preset.icon}</div>
                <div className="preset-name">{preset.name}</div>
                <div className="preset-description">{preset.description}</div>
              </button>
            ))}
          </div>

          {/* Custom Personalities */}
          {customPersonalities.length > 0 && (
            <div className="custom-personalities">
              <h3>Custom Personalities</h3>
              {customPersonalities.map((custom, index) => (
                <div key={index} className="custom-item" data-testid={`custom-personality-${index}`}>
                  <button
                    onClick={() => handleLoadCustom(custom)}
                    disabled={!isConnected}
                    className="custom-load"
                  >
                    {custom.name}
                  </button>
                  <button
                    onClick={() => handleDeleteCustom(index)}
                    data-testid={`delete-custom-${index}`}
                    className="custom-delete"
                  >
                    ✕
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Save Custom Dialog */}
      {saveDialogOpen && (
        <div className="dialog-overlay" data-testid="save-dialog">
          <div className="dialog">
            <h3>Save Custom Personality</h3>
            <input
              type="text"
              value={customName}
              onChange={(e) => setCustomName(e.target.value)}
              placeholder="Enter personality name"
              data-testid="custom-name-input"
              autoFocus
            />
            <div className="dialog-buttons">
              <button onClick={handleSaveCustom} className="btn-primary" data-testid="save-confirm">
                Save
              </button>
              <button
                onClick={() => {
                  setSaveDialogOpen(false);
                  setCustomName('');
                }}
                className="btn-secondary"
                data-testid="save-cancel"
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

// Parameter Slider Component
interface ParameterSliderProps {
  param: typeof PARAMETER_METADATA[0];
  value: number;
  previewValue?: number;
  onChange: (key: keyof PersonalityConfig, value: number) => void;
  onChangeComplete: () => void;
  disabled: boolean;
}

const ParameterSlider: React.FC<ParameterSliderProps> = ({
  param,
  value,
  previewValue,
  onChange,
  onChangeComplete,
  disabled,
}) => {
  const [isDragging, setIsDragging] = useState(false);
  const displayValue = previewValue !== undefined && !isDragging ? previewValue : value;

  return (
    <div className="parameter" data-testid={`parameter-${param.key}`}>
      <div className="param-header">
        <label className="param-label">{param.label}</label>
        <span className="param-value" data-testid={`value-${param.key}`}>
          {displayValue.toFixed(2)}
        </span>
      </div>
      <div className="param-description">{param.description}</div>
      <input
        type="range"
        min="0"
        max="1"
        step="0.01"
        value={value}
        onChange={(e) => onChange(param.key, parseFloat(e.target.value))}
        onMouseDown={() => setIsDragging(true)}
        onMouseUp={() => {
          setIsDragging(false);
          onChangeComplete();
        }}
        onTouchStart={() => setIsDragging(true)}
        onTouchEnd={() => {
          setIsDragging(false);
          onChangeComplete();
        }}
        disabled={disabled}
        data-testid={`slider-${param.key.replace(/_/g, '-')}`}
        className="parameter-slider"
      />
    </div>
  );
};

export default PersonalityMixer;
