/**
 * Neural Visualizer Component
 * Real-time visualization of robot's nervous system
 *
 * Features:
 * - Live neural state meters (tension, energy, coherence, curiosity)
 * - Animated timeline with 60-second history
 * - Mode transition indicators
 * - Zoom/pan controls
 * - Timeline scrubber for historical review
 * - Export functionality (CSV/JSON)
 * - 20Hz WebSocket updates with smooth transitions
 *
 * Satisfies invariants:
 * - I-LEARN-VIZ-001: Update at minimum 10Hz (100ms)
 * - I-LEARN-VIZ-002: Store last 300 seconds (5 min)
 */

import React, { useEffect, useRef, useState, useCallback } from 'react';
import { useWebSocket } from '../hooks/useWebSocket';
import { NeuralState } from '../types/neural';
import {
  setupCanvas,
  renderMeter,
  renderTimeline,
  renderModeIndicator,
  renderStimulusFlash,
  getTensionColor,
  getEnergyColor,
} from '../utils/canvasRenderer';

interface NeuralVisualizerProps {
  wsUrl?: string;
  maxHistorySeconds?: number;
}

const NeuralVisualizer: React.FC<NeuralVisualizerProps> = ({
  wsUrl = 'ws://localhost:8081',
  maxHistorySeconds = 300, // I-LEARN-VIZ-002: 5 minutes
}) => {
  // WebSocket connection
  const { state, isConnected, error, reconnect } = useWebSocket({ url: wsUrl });

  // State management
  const [history, setHistory] = useState<NeuralState[]>([]);
  const [isPaused, setIsPaused] = useState(false);
  const [scrubPosition, setScrubPosition] = useState<number | null>(null);
  const [zoom, setZoom] = useState(1);
  const [stimulusFlash, setStimulusFlash] = useState(0);

  // Canvas refs
  const timelineCanvasRef = useRef<HTMLCanvasElement>(null);
  const metersCanvasRef = useRef<HTMLCanvasElement>(null);
  const modeCanvasRef = useRef<HTMLCanvasElement>(null);

  // Add new state to history
  useEffect(() => {
    if (!state || isPaused) return;

    setHistory((prev) => {
      const newHistory = [...prev, state];
      const cutoffTime = Date.now() - maxHistorySeconds * 1000;
      return newHistory.filter((s) => s.timestamp > cutoffTime);
    });

    // Trigger stimulus flash on mode change
    if (history.length > 0 && history[history.length - 1].mode !== state.mode) {
      setStimulusFlash(1);
    }
  }, [state, isPaused, maxHistorySeconds, history.length]);

  // Decay stimulus flash
  useEffect(() => {
    if (stimulusFlash > 0) {
      const timer = setTimeout(() => {
        setStimulusFlash((prev) => Math.max(0, prev - 0.05));
      }, 16);
      return () => clearTimeout(timer);
    }
  }, [stimulusFlash]);

  // Get current display state (scrubbed or live)
  const displayState = useCallback((): NeuralState | undefined => {
    if (scrubPosition !== null && history.length > 0) {
      const index = Math.floor((history.length - 1) * scrubPosition);
      return history[index];
    }
    return state || undefined;
  }, [scrubPosition, history, state]);

  // Render meters
  useEffect(() => {
    const canvas = metersCanvasRef.current;
    if (!canvas) return;

    const renderContext = setupCanvas(canvas);
    const { ctx, width, height } = renderContext;

    const currentState = displayState();
    if (!currentState) return;

    ctx.clearRect(0, 0, width, height);

    const meterWidth = width * 0.8;
    const meterHeight = 30;
    const startX = (width - meterWidth) / 2;
    let yOffset = 40;

    // Tension meter (red intensity)
    renderMeter(ctx, {
      x: startX,
      y: yOffset,
      width: meterWidth,
      height: meterHeight,
      label: 'Tension',
      color: getTensionColor(currentState.tension),
      value: currentState.tension,
    });
    yOffset += meterHeight + 40;

    // Energy meter (blue intensity)
    renderMeter(ctx, {
      x: startX,
      y: yOffset,
      width: meterWidth,
      height: meterHeight,
      label: 'Energy',
      color: getEnergyColor(currentState.energy),
      value: currentState.energy,
    });
    yOffset += meterHeight + 40;

    // Coherence meter
    renderMeter(ctx, {
      x: startX,
      y: yOffset,
      width: meterWidth,
      height: meterHeight,
      label: 'Coherence',
      color: '#10b981',
      value: currentState.coherence,
    });
    yOffset += meterHeight + 40;

    // Curiosity meter
    renderMeter(ctx, {
      x: startX,
      y: yOffset,
      width: meterWidth,
      height: meterHeight,
      label: 'Curiosity',
      color: '#8b5cf6',
      value: currentState.curiosity,
    });

    // Stimulus flash effect
    if (stimulusFlash > 0) {
      renderStimulusFlash(ctx, width / 2, height / 2, 200, stimulusFlash * 0.3);
    }
  }, [displayState, stimulusFlash]);

  // Render timeline
  useEffect(() => {
    const canvas = timelineCanvasRef.current;
    if (!canvas) return;

    const renderContext = setupCanvas(canvas);
    const { ctx, width, height } = renderContext;

    const timelineDuration = 60; // 60 seconds visible
    const currentTime = scrubPosition !== null ? scrubPosition * maxHistorySeconds : 0;

    renderTimeline(
      ctx,
      history,
      {
        duration: timelineDuration,
        width,
        height,
        padding: 40,
      },
      currentTime
    );

    // Draw scrub indicator
    if (scrubPosition !== null) {
      const x = 40 + (width - 80) * scrubPosition;
      ctx.strokeStyle = '#fbbf24';
      ctx.lineWidth = 3;
      ctx.beginPath();
      ctx.moveTo(x, 40);
      ctx.lineTo(x, height - 40);
      ctx.stroke();
    }
  }, [history, scrubPosition, maxHistorySeconds]);

  // Render mode indicator
  useEffect(() => {
    const canvas = modeCanvasRef.current;
    if (!canvas) return;

    const renderContext = setupCanvas(canvas);
    const { ctx, width, height } = renderContext;

    const currentState = displayState();
    if (!currentState) return;

    ctx.clearRect(0, 0, width, height);

    renderModeIndicator(ctx, currentState.mode, width / 2, height / 2, 80);
  }, [displayState]);

  // Export data functions
  const exportToCSV = useCallback(() => {
    const csv = [
      'timestamp,mode,tension,coherence,energy,curiosity',
      ...history.map(
        (s) =>
          `${s.timestamp},${s.mode},${s.tension},${s.coherence},${s.energy},${s.curiosity}`
      ),
    ].join('\n');

    const blob = new Blob([csv], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `neural-data-${Date.now()}.csv`;
    a.click();
    URL.revokeObjectURL(url);
  }, [history]);

  const exportToJSON = useCallback(() => {
    const json = JSON.stringify(history, null, 2);
    const blob = new Blob([json], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `neural-data-${Date.now()}.json`;
    a.click();
    URL.revokeObjectURL(url);
  }, [history]);

  // Timeline scrubbing handlers
  const handleTimelineMouseDown = useCallback(
    (e: React.MouseEvent<HTMLCanvasElement>) => {
      const canvas = timelineCanvasRef.current;
      if (!canvas) return;

      const rect = canvas.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const position = (x - 40) / (rect.width - 80);
      setScrubPosition(Math.max(0, Math.min(1, position)));
      setIsPaused(true);
    },
    []
  );

  const handleTimelineMouseMove = useCallback(
    (e: React.MouseEvent<HTMLCanvasElement>) => {
      if (scrubPosition === null) return;

      const canvas = timelineCanvasRef.current;
      if (!canvas) return;

      const rect = canvas.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const position = (x - 40) / (rect.width - 80);
      setScrubPosition(Math.max(0, Math.min(1, position)));
    },
    [scrubPosition]
  );

  const handleTimelineMouseUp = useCallback(() => {
    // Keep paused until user clicks play
  }, []);

  // Zoom controls
  const handleZoomIn = () => setZoom((prev) => Math.min(prev + 0.25, 3));
  const handleZoomOut = () => setZoom((prev) => Math.max(prev - 0.25, 0.5));
  const handleZoomReset = () => setZoom(1);

  // Play/Pause
  const handlePlayPause = () => {
    setIsPaused((prev) => !prev);
    if (isPaused) {
      setScrubPosition(null);
    }
  };

  return (
    <div
      style={{
        width: '100%',
        height: '100vh',
        backgroundColor: '#0f172a',
        color: '#ffffff',
        fontFamily: 'monospace',
        overflow: 'auto',
      }}
    >
      {/* Header */}
      <div
        style={{
          padding: '20px',
          borderBottom: '2px solid #1e293b',
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
        }}
      >
        <h1 style={{ margin: 0 }}>Neural Visualizer</h1>
        <div style={{ display: 'flex', gap: '10px', alignItems: 'center' }}>
          <div
            style={{
              width: '10px',
              height: '10px',
              borderRadius: '50%',
              backgroundColor: isConnected ? '#10b981' : '#ef4444',
            }}
            title={isConnected ? 'Connected' : 'Disconnected'}
          />
          <span>{isConnected ? 'Connected' : 'Disconnected'}</span>
          {error && (
            <button
              onClick={reconnect}
              style={{
                padding: '5px 10px',
                backgroundColor: '#3b82f6',
                color: '#ffffff',
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
              }}
            >
              Reconnect
            </button>
          )}
        </div>
      </div>

      {/* Main content */}
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '20px', padding: '20px' }}>
        {/* Mode Indicator */}
        <div>
          <h2>Current Mode</h2>
          <canvas
            ref={modeCanvasRef}
            data-testid="neural-mode-indicator"
            width={400}
            height={200}
            style={{ width: '100%', height: 'auto', backgroundColor: '#1e293b', borderRadius: '8px' }}
          />
        </div>

        {/* Meters */}
        <div>
          <h2>Neural State</h2>
          <canvas
            ref={metersCanvasRef}
            data-testid="neural-tension-meter"
            width={400}
            height={400}
            style={{ width: '100%', height: 'auto', backgroundColor: '#1e293b', borderRadius: '8px' }}
          />
        </div>
      </div>

      {/* Timeline */}
      <div style={{ padding: '20px' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '10px' }}>
          <h2>Timeline (Last 60 seconds)</h2>
          <div style={{ display: 'flex', gap: '10px' }}>
            <button
              onClick={handlePlayPause}
              style={{
                padding: '8px 16px',
                backgroundColor: '#3b82f6',
                color: '#ffffff',
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
              }}
            >
              {isPaused ? '▶ Play' : '⏸ Pause'}
            </button>
            <button
              onClick={handleZoomIn}
              style={{
                padding: '8px 16px',
                backgroundColor: '#6366f1',
                color: '#ffffff',
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
              }}
            >
              🔍+
            </button>
            <button
              onClick={handleZoomOut}
              style={{
                padding: '8px 16px',
                backgroundColor: '#6366f1',
                color: '#ffffff',
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
              }}
            >
              🔍-
            </button>
            <button
              onClick={handleZoomReset}
              style={{
                padding: '8px 16px',
                backgroundColor: '#6366f1',
                color: '#ffffff',
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
              }}
            >
              Reset Zoom
            </button>
          </div>
        </div>
        <canvas
          ref={timelineCanvasRef}
          data-testid="neural-timeline-chart"
          width={1200}
          height={300}
          onMouseDown={handleTimelineMouseDown}
          onMouseMove={handleTimelineMouseMove}
          onMouseUp={handleTimelineMouseUp}
          onMouseLeave={handleTimelineMouseUp}
          style={{
            width: '100%',
            height: 'auto',
            backgroundColor: '#1e293b',
            borderRadius: '8px',
            cursor: 'crosshair',
            transform: `scale(${zoom})`,
            transformOrigin: 'top left',
          }}
        />
        <div style={{ marginTop: '10px', fontSize: '12px', color: '#9ca3af' }}>
          {scrubPosition !== null && (
            <span>
              Viewing: {((1 - scrubPosition) * 60).toFixed(1)}s ago
            </span>
          )}
        </div>
      </div>

      {/* Export Controls */}
      <div style={{ padding: '20px', borderTop: '2px solid #1e293b' }}>
        <h2>Export Data</h2>
        <div style={{ display: 'flex', gap: '10px' }}>
          <button
            onClick={exportToCSV}
            data-testid="export-data-button"
            style={{
              padding: '10px 20px',
              backgroundColor: '#10b981',
              color: '#ffffff',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
            }}
          >
            Export CSV
          </button>
          <button
            onClick={exportToJSON}
            style={{
              padding: '10px 20px',
              backgroundColor: '#10b981',
              color: '#ffffff',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
            }}
          >
            Export JSON
          </button>
        </div>
        <div style={{ marginTop: '10px', fontSize: '12px', color: '#9ca3af' }}>
          {history.length} data points • {Math.floor(history.length / 20)} seconds recorded
        </div>
      </div>
    </div>
  );
};

export default NeuralVisualizer;
