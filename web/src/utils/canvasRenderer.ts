/**
 * Canvas Rendering Utilities
 * High-performance rendering for neural visualizer
 */

import { NeuralState, NeuralMode } from '../types/neural';

export interface RenderContext {
  ctx: CanvasRenderingContext2D;
  width: number;
  height: number;
  pixelRatio: number;
}

export interface TimelineConfig {
  duration: number; // seconds
  width: number;
  height: number;
  padding: number;
}

export interface MeterConfig {
  x: number;
  y: number;
  width: number;
  height: number;
  label: string;
  color: string;
  value: number;
}

/**
 * Setup canvas for high-DPI displays
 */
export function setupCanvas(canvas: HTMLCanvasElement): RenderContext {
  const pixelRatio = window.devicePixelRatio || 1;
  const rect = canvas.getBoundingClientRect();

  canvas.width = rect.width * pixelRatio;
  canvas.height = rect.height * pixelRatio;
  canvas.style.width = `${rect.width}px`;
  canvas.style.height = `${rect.height}px`;

  const ctx = canvas.getContext('2d');
  if (!ctx) {
    throw new Error('Could not get 2D context');
  }

  ctx.scale(pixelRatio, pixelRatio);

  return {
    ctx,
    width: rect.width,
    height: rect.height,
    pixelRatio,
  };
}

/**
 * Get color for neural mode
 */
export function getModeColor(mode: NeuralMode): string {
  const colors: Record<NeuralMode, string> = {
    Calm: '#4ade80',
    Active: '#60a5fa',
    Spike: '#f59e0b',
    Protect: '#ef4444',
  };
  return colors[mode];
}

/**
 * Get color for tension level (red intensity)
 */
export function getTensionColor(tension: number): string {
  const intensity = Math.floor(tension * 255);
  return `rgb(${intensity}, 0, 0)`;
}

/**
 * Get color for energy level (blue intensity)
 */
export function getEnergyColor(energy: number): string {
  const intensity = Math.floor(energy * 255);
  return `rgb(0, 0, ${intensity})`;
}

/**
 * Easing function for smooth transitions
 */
export function easeOutCubic(t: number): number {
  return 1 - Math.pow(1 - t, 3);
}

export function easeInOutCubic(t: number): number {
  return t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;
}

/**
 * Render a meter bar
 */
export function renderMeter(
  ctx: CanvasRenderingContext2D,
  config: MeterConfig
): void {
  const { x, y, width, height, label, color, value } = config;

  // Background
  ctx.fillStyle = '#1f2937';
  ctx.fillRect(x, y, width, height);

  // Value bar
  ctx.fillStyle = color;
  ctx.fillRect(x, y, width * Math.max(0, Math.min(1, value)), height);

  // Border
  ctx.strokeStyle = '#4b5563';
  ctx.lineWidth = 2;
  ctx.strokeRect(x, y, width, height);

  // Label
  ctx.fillStyle = '#ffffff';
  ctx.font = '14px monospace';
  ctx.textAlign = 'left';
  ctx.textBaseline = 'top';
  ctx.fillText(label, x, y - 20);

  // Value text
  ctx.textAlign = 'right';
  ctx.fillText(`${(value * 100).toFixed(0)}%`, x + width, y - 20);
}

/**
 * Render timeline chart
 */
export function renderTimeline(
  ctx: CanvasRenderingContext2D,
  history: NeuralState[],
  config: TimelineConfig,
  _currentTime: number
): void {
  const { duration, width, height, padding } = config;

  if (history.length === 0) return;

  // Clear area
  ctx.clearRect(0, 0, width, height);

  // Background
  ctx.fillStyle = '#111827';
  ctx.fillRect(padding, padding, width - 2 * padding, height - 2 * padding);

  // Grid lines
  ctx.strokeStyle = '#374151';
  ctx.lineWidth = 1;
  const gridLines = 5;
  for (let i = 0; i <= gridLines; i++) {
    const y = padding + (height - 2 * padding) * (i / gridLines);
    ctx.beginPath();
    ctx.moveTo(padding, y);
    ctx.lineTo(width - padding, y);
    ctx.stroke();
  }

  // Time axis labels
  ctx.fillStyle = '#9ca3af';
  ctx.font = '12px monospace';
  ctx.textAlign = 'center';
  for (let i = 0; i <= 4; i++) {
    const x = padding + (width - 2 * padding) * (i / 4);
    const time = -duration + (duration * i / 4);
    ctx.fillText(`${time.toFixed(0)}s`, x, height - padding + 15);
  }

  // Plot tension line
  ctx.strokeStyle = getTensionColor(0.8);
  ctx.lineWidth = 2;
  ctx.beginPath();

  const now = Date.now();
  const chartWidth = width - 2 * padding;
  const chartHeight = height - 2 * padding;

  history.forEach((state, i) => {
    const age = (now - state.timestamp) / 1000; // seconds ago
    if (age > duration) return;

    const x = padding + chartWidth * (1 - age / duration);
    const y = padding + chartHeight * (1 - state.tension);

    if (i === 0) {
      ctx.moveTo(x, y);
    } else {
      ctx.lineTo(x, y);
    }
  });

  ctx.stroke();

  // Plot energy line
  ctx.strokeStyle = getEnergyColor(0.8);
  ctx.lineWidth = 2;
  ctx.beginPath();

  history.forEach((state, i) => {
    const age = (now - state.timestamp) / 1000;
    if (age > duration) return;

    const x = padding + chartWidth * (1 - age / duration);
    const y = padding + chartHeight * (1 - state.energy);

    if (i === 0) {
      ctx.moveTo(x, y);
    } else {
      ctx.lineTo(x, y);
    }
  });

  ctx.stroke();

  // Mode transition markers
  let lastMode: NeuralMode | null = null;
  history.forEach((state) => {
    const age = (now - state.timestamp) / 1000;
    if (age > duration) return;

    if (lastMode && state.mode !== lastMode) {
      const x = padding + chartWidth * (1 - age / duration);
      ctx.strokeStyle = getModeColor(state.mode);
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.moveTo(x, padding);
      ctx.lineTo(x, height - padding);
      ctx.stroke();
    }

    lastMode = state.mode;
  });
}

/**
 * Render stimulus flash effect
 */
export function renderStimulusFlash(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  radius: number,
  intensity: number
): void {
  const gradient = ctx.createRadialGradient(x, y, 0, x, y, radius);
  gradient.addColorStop(0, `rgba(255, 255, 0, ${intensity})`);
  gradient.addColorStop(1, 'rgba(255, 255, 0, 0)');

  ctx.fillStyle = gradient;
  ctx.beginPath();
  ctx.arc(x, y, radius, 0, Math.PI * 2);
  ctx.fill();
}

/**
 * Render mode indicator
 */
export function renderModeIndicator(
  ctx: CanvasRenderingContext2D,
  mode: NeuralMode,
  x: number,
  y: number,
  size: number
): void {
  const color = getModeColor(mode);

  // Circle background
  ctx.fillStyle = color;
  ctx.beginPath();
  ctx.arc(x, y, size / 2, 0, Math.PI * 2);
  ctx.fill();

  // Mode icon
  const icons: Record<NeuralMode, string> = {
    Calm: '😌',
    Active: '🤔',
    Spike: '⚡',
    Protect: '🛡️',
  };

  ctx.fillStyle = '#ffffff';
  ctx.font = `${size * 0.6}px sans-serif`;
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillText(icons[mode], x, y);

  // Mode label
  ctx.font = '16px monospace';
  ctx.fillStyle = '#ffffff';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'top';
  ctx.fillText(mode, x, y + size / 2 + 10);
}
