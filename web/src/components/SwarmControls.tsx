/**
 * SwarmControls Component
 *
 * UI for controlling multi-robot swarm play modes.
 *
 * **Issue**: #83 - Swarm Play Modes
 * **Contract Compliance**: I-MULTI-004, I-MULTI-006, I-MULTI-007
 *
 * Features:
 * - Mode selection (dance, collaborative drawing, tag team, patrol)
 * - Formation visualizer
 * - Sync status indicator
 * - Robot position markers
 * - Safety status display
 */

import React, { useState, useEffect } from 'react';
import './SwarmControls.css';

// ============================================================================
// Types (matches Rust data structures)
// ============================================================================

export type SwarmModeType =
  | 'follow-leader'
  | 'circle'
  | 'wave'
  | 'random-walk'
  | 'dance'
  | 'collaborative-draw'
  | 'tag-team'
  | 'patrol';

export type PlayStatus = 'idle' | 'active' | 'paused' | 'completed' | 'failed';

export type FormationType = 'line' | 'diamond' | 'circle' | 'square';

export interface RobotPosition {
  robotId: string;
  x: number;  // cm
  y: number;  // cm
  heading: number;  // radians
  status: 'idle' | 'moving' | 'executing' | 'offline';
  batteryLevel: number;  // 0-1
}

export interface SwarmConfig {
  modeType: SwarmModeType;
  participants: string[];
  status: PlayStatus;
  startTime: number;
  currentStep: number;
  params: SwarmParams;
}

export type SwarmParams =
  | { type: 'follow-leader'; leaderId: string; spacing: number }
  | { type: 'circle'; centerX: number; centerY: number; radius: number; rotationSpeed: number }
  | { type: 'wave'; amplitude: number; frequency: number; phaseOffset: number }
  | { type: 'random-walk'; boundsMinX: number; boundsMinY: number; boundsMaxX: number; boundsMaxY: number; duration: number }
  | { type: 'dance'; choreographyName: string; syncTolerance: number }
  | { type: 'collaborative-draw'; turnDuration: number; canvasWidth: number; canvasHeight: number }
  | { type: 'tag-team'; game: string; strategy: string }
  | { type: 'patrol'; formation: FormationType; spacing: number; speed: number };

export interface SwarmStatus {
  config: SwarmConfig | null;
  robots: RobotPosition[];
  syncIndicator: {
    inSync: boolean;
    maxDeviation: number;  // ms
  };
  safetyStatus: {
    safe: boolean;
    warnings: string[];
  };
}

// ============================================================================
// Props
// ============================================================================

export interface SwarmControlsProps {
  /** Available robots */
  availableRobots: RobotPosition[];
  /** Current swarm status */
  status: SwarmStatus | null;
  /** Callback when swarm mode is started */
  onStartMode: (config: SwarmConfig) => void;
  /** Callback when swarm mode is stopped */
  onStopMode: () => void;
  /** Callback when formation is changed */
  onUpdateFormation?: (params: SwarmParams) => void;
}

// ============================================================================
// Component
// ============================================================================

export const SwarmControls: React.FC<SwarmControlsProps> = ({
  availableRobots,
  status,
  onStartMode,
  onStopMode,
  onUpdateFormation,
}) => {
  // State
  const [selectedMode, setSelectedMode] = useState<SwarmModeType>('dance');
  const [selectedRobots, setSelectedRobots] = useState<string[]>([]);
  const [modeParams, setModeParams] = useState<Partial<SwarmParams>>({});

  // Auto-select available robots (up to 4)
  useEffect(() => {
    if (selectedRobots.length === 0 && availableRobots.length > 0) {
      setSelectedRobots(
        availableRobots.slice(0, Math.min(4, availableRobots.length)).map(r => r.robotId)
      );
    }
  }, [availableRobots]);

  // ============================================================================
  // Handlers
  // ============================================================================

  const handleStartMode = () => {
    // TODO (#82): Integrate with coordination protocol once available
    const config: SwarmConfig = {
      modeType: selectedMode,
      participants: selectedRobots,
      status: 'idle',
      startTime: Date.now(),
      currentStep: 0,
      params: buildParamsForMode(selectedMode),
    };

    onStartMode(config);
  };

  const handleStopMode = () => {
    onStopMode();
  };

  const handleRobotToggle = (robotId: string) => {
    if (selectedRobots.includes(robotId)) {
      setSelectedRobots(selectedRobots.filter(id => id !== robotId));
    } else {
      // Max 4 robots (I-MULTI-004)
      if (selectedRobots.length < 4) {
        setSelectedRobots([...selectedRobots, robotId]);
      }
    }
  };

  const buildParamsForMode = (mode: SwarmModeType): SwarmParams => {
    switch (mode) {
      case 'follow-leader':
        return {
          type: 'follow-leader',
          leaderId: selectedRobots[0] || '',
          spacing: 30,  // cm
        };
      case 'circle':
        return {
          type: 'circle',
          centerX: 0,
          centerY: 0,
          radius: 50,  // cm
          rotationSpeed: 0.1,  // rad/s
        };
      case 'wave':
        return {
          type: 'wave',
          amplitude: 20,  // cm
          frequency: 0.5,  // Hz
          phaseOffset: Math.PI / 2,
        };
      case 'random-walk':
        return {
          type: 'random-walk',
          boundsMinX: -100,
          boundsMinY: -100,
          boundsMaxX: 100,
          boundsMaxY: 100,
          duration: 60,  // seconds
        };
      case 'dance':
        return {
          type: 'dance',
          choreographyName: 'default-routine',
          syncTolerance: 100,  // ms (I-MULTI-007)
        };
      case 'collaborative-draw':
        return {
          type: 'collaborative-draw',
          turnDuration: 5,  // seconds
          canvasWidth: 200,  // cm
          canvasHeight: 200,  // cm
        };
      case 'tag-team':
        return {
          type: 'tag-team',
          game: 'tic-tac-toe',
          strategy: 'minimax',
        };
      case 'patrol':
        return {
          type: 'patrol',
          formation: 'diamond',
          spacing: 50,  // cm
          speed: 20,  // cm/s
        };
      default:
        throw new Error(`Unknown mode: ${mode}`);
    }
  };

  // ============================================================================
  // Render Helpers
  // ============================================================================

  const renderModeSelector = () => (
    <div className="mode-selector" data-testid="swarm-mode-selector">
      <h3>Select Swarm Mode</h3>
      <div className="mode-buttons">
        <button
          data-testid="swarm-dance-btn"
          className={selectedMode === 'dance' ? 'active' : ''}
          onClick={() => setSelectedMode('dance')}
          disabled={status?.config?.status === 'active'}
        >
          🕺 Dance
        </button>
        <button
          data-testid="swarm-draw-btn"
          className={selectedMode === 'collaborative-draw' ? 'active' : ''}
          onClick={() => setSelectedMode('collaborative-draw')}
          disabled={status?.config?.status === 'active'}
        >
          🎨 Collaborative Draw
        </button>
        <button
          data-testid="swarm-tag-team-btn"
          className={selectedMode === 'tag-team' ? 'active' : ''}
          onClick={() => setSelectedMode('tag-team')}
          disabled={status?.config?.status === 'active'}
        >
          🎮 Tag Team
        </button>
        <button
          data-testid="swarm-patrol-btn"
          className={selectedMode === 'patrol' ? 'active' : ''}
          onClick={() => setSelectedMode('patrol')}
          disabled={status?.config?.status === 'active'}
        >
          🛡️ Patrol
        </button>
      </div>
      <div className="mode-buttons">
        <button
          className={selectedMode === 'follow-leader' ? 'active' : ''}
          onClick={() => setSelectedMode('follow-leader')}
          disabled={status?.config?.status === 'active'}
        >
          👑 Follow Leader
        </button>
        <button
          className={selectedMode === 'circle' ? 'active' : ''}
          onClick={() => setSelectedMode('circle')}
          disabled={status?.config?.status === 'active'}
        >
          ⭕ Circle
        </button>
        <button
          className={selectedMode === 'wave' ? 'active' : ''}
          onClick={() => setSelectedMode('wave')}
          disabled={status?.config?.status === 'active'}
        >
          🌊 Wave
        </button>
        <button
          className={selectedMode === 'random-walk' ? 'active' : ''}
          onClick={() => setSelectedMode('random-walk')}
          disabled={status?.config?.status === 'active'}
        >
          🎲 Random Walk
        </button>
      </div>
    </div>
  );

  const renderRobotSelector = () => (
    <div className="robot-selector">
      <h3>Select Robots (2-4)</h3>
      <div className="robot-list">
        {availableRobots.map(robot => (
          <label key={robot.robotId} className="robot-checkbox">
            <input
              type="checkbox"
              checked={selectedRobots.includes(robot.robotId)}
              onChange={() => handleRobotToggle(robot.robotId)}
              disabled={
                status?.config?.status === 'active' ||
                (!selectedRobots.includes(robot.robotId) && selectedRobots.length >= 4)
              }
            />
            <span className={`robot-status ${robot.status}`}>
              {robot.robotId}
            </span>
            <span className="battery-indicator">
              🔋 {Math.round(robot.batteryLevel * 100)}%
            </span>
          </label>
        ))}
      </div>
      {selectedRobots.length < 2 && (
        <p className="warning">⚠️ Select at least 2 robots</p>
      )}
      {selectedRobots.length > 4 && (
        <p className="error">❌ Maximum 4 robots allowed (I-MULTI-004)</p>
      )}
    </div>
  );

  const renderFormationVisualizer = () => {
    if (!status || !status.robots || status.robots.length === 0) {
      return (
        <div className="formation-visualizer" data-testid="formation-visualizer">
          <p className="placeholder">Start swarm mode to see formation</p>
        </div>
      );
    }

    // Calculate bounds for visualization
    const positions = status.robots.map(r => ({ x: r.x, y: r.y }));
    const minX = Math.min(...positions.map(p => p.x), -100);
    const maxX = Math.max(...positions.map(p => p.x), 100);
    const minY = Math.min(...positions.map(p => p.y), -100);
    const maxY = Math.max(...positions.map(p => p.y), 100);

    const scale = (val: number, min: number, max: number) => {
      const range = max - min;
      return ((val - min) / range) * 280 + 10; // 10px padding
    };

    return (
      <div className="formation-visualizer" data-testid="formation-visualizer">
        <svg width="300" height="300" viewBox="0 0 300 300">
          {/* Grid */}
          <g className="grid">
            {[0, 1, 2, 3, 4].map(i => (
              <React.Fragment key={i}>
                <line
                  x1={i * 60 + 30}
                  y1={30}
                  x2={i * 60 + 30}
                  y2={270}
                  stroke="#eee"
                  strokeWidth="1"
                />
                <line
                  x1={30}
                  y1={i * 60 + 30}
                  x2={270}
                  y2={i * 60 + 30}
                  stroke="#eee"
                  strokeWidth="1"
                />
              </React.Fragment>
            ))}
          </g>

          {/* Robot markers */}
          {status.robots.map(robot => {
            const x = scale(robot.x, minX, maxX);
            const y = scale(robot.y, minY, maxY);

            return (
              <g
                key={robot.robotId}
                data-testid={`robot-marker-${robot.robotId}`}
                transform={`translate(${x}, ${y})`}
              >
                {/* Robot body */}
                <circle
                  r="15"
                  fill={robot.status === 'offline' ? '#ccc' : '#4CAF50'}
                  stroke="#333"
                  strokeWidth="2"
                />
                {/* Heading indicator */}
                <line
                  x1="0"
                  y1="0"
                  x2={Math.cos(robot.heading) * 20}
                  y2={Math.sin(robot.heading) * 20}
                  stroke="#333"
                  strokeWidth="2"
                />
                {/* Robot ID */}
                <text
                  y="30"
                  textAnchor="middle"
                  fontSize="10"
                  fill="#333"
                >
                  {robot.robotId}
                </text>
              </g>
            );
          })}
        </svg>
      </div>
    );
  };

  const renderSyncIndicator = () => {
    if (!status) return null;

    const { syncIndicator } = status;
    const syncClass = syncIndicator.inSync ? 'in-sync' : 'out-of-sync';

    return (
      <div className={`sync-indicator ${syncClass}`} data-testid="swarm-sync-indicator">
        <span className="sync-icon">{syncIndicator.inSync ? '✓' : '⚠'}</span>
        <span className="sync-text">
          {syncIndicator.inSync
            ? `Synchronized (${syncIndicator.maxDeviation}ms)`
            : `Out of sync (${syncIndicator.maxDeviation}ms > 100ms)`}
        </span>
      </div>
    );
  };

  const renderSwarmStatus = () => {
    if (!status || !status.config) {
      return (
        <div className="swarm-status" data-testid="swarm-status">
          <p>No active swarm mode</p>
        </div>
      );
    }

    const { config, safetyStatus } = status;

    return (
      <div className="swarm-status" data-testid="swarm-status">
        <h3>Status: {config.status.toUpperCase()}</h3>
        <div className="status-details">
          <p><strong>Mode:</strong> {config.modeType}</p>
          <p><strong>Robots:</strong> {config.participants.length}</p>
          <p><strong>Step:</strong> {config.currentStep}</p>
          {renderSyncIndicator()}
          <div className={`safety-status ${safetyStatus.safe ? 'safe' : 'warning'}`}>
            <strong>Safety:</strong> {safetyStatus.safe ? '✓ All clear' : '⚠ Warnings'}
            {!safetyStatus.safe && (
              <ul className="safety-warnings">
                {safetyStatus.warnings.map((warning, i) => (
                  <li key={i}>{warning}</li>
                ))}
              </ul>
            )}
          </div>
        </div>
      </div>
    );
  };

  const renderControls = () => {
    const isActive = status?.config?.status === 'active';

    return (
      <div className="swarm-controls-buttons">
        {!isActive ? (
          <button
            className="start-btn"
            onClick={handleStartMode}
            disabled={selectedRobots.length < 2 || selectedRobots.length > 4}
          >
            ▶️ Start Swarm Mode
          </button>
        ) : (
          <button
            className="stop-btn"
            data-testid="stop-swarm-btn"
            onClick={handleStopMode}
          >
            ⏹️ Stop Swarm Mode
          </button>
        )}
      </div>
    );
  };

  // ============================================================================
  // Render
  // ============================================================================

  return (
    <div className="swarm-controls">
      <h2>Multi-Robot Swarm Control</h2>

      {/* TODO (#82): Show loading state while coordination protocol initializes */}
      <div className="swarm-panel">
        <div className="swarm-config">
          {renderModeSelector()}
          {renderRobotSelector()}
          {renderControls()}
        </div>

        <div className="swarm-visualization">
          {renderFormationVisualizer()}
          {renderSwarmStatus()}
        </div>
      </div>

      {/* Contract compliance notice */}
      <div className="contract-notice">
        <small>
          <strong>Safety:</strong> 20cm buffer maintained (I-MULTI-004) |{' '}
          <strong>Sync:</strong> 100ms tolerance (I-MULTI-007)
        </small>
      </div>
    </div>
  );
};

export default SwarmControls;
