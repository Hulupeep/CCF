/**
 * Performance Metrics Type Definitions
 * Contract: Issue #80 - Performance Benchmarking Dashboard
 *
 * Invariants:
 * - I-PERF-001: WebSocket latency p99 < 50ms
 * - I-PERF-002: UI render time < 16ms for 60fps
 * - I-PERF-003: Memory baseline < 100MB
 * - I-PERF-004: Regression detection threshold 10%
 */

/**
 * Metric categories for performance tracking
 */
export type MetricCategory =
  | 'websocket'
  | 'ui'
  | 'memory'
  | 'processing'
  | 'sync'
  | 'component';

/**
 * Performance metric data point
 */
export interface MetricDataPoint {
  /** Timestamp (milliseconds since epoch) */
  timestamp: number;

  /** Metric value */
  value: number;

  /** Optional label for the data point */
  label?: string;
}

/**
 * Statistical summary of metrics
 */
export interface MetricStats {
  /** Mean/average value */
  mean: number;

  /** Median value (50th percentile) */
  median: number;

  /** 90th percentile */
  p90: number;

  /** 95th percentile */
  p95: number;

  /** 99th percentile */
  p99: number;

  /** Minimum value */
  min: number;

  /** Maximum value */
  max: number;

  /** Standard deviation */
  stdDev: number;

  /** Sample count */
  count: number;
}

/**
 * Individual performance metric
 */
export interface PerformanceMetric {
  /** Metric identifier */
  id: string;

  /** Human-readable name */
  name: string;

  /** Metric category */
  category: MetricCategory;

  /** Unit of measurement (ms, MB, fps, etc.) */
  unit: string;

  /** Target/baseline value */
  target: number;

  /** Current value */
  current: number;

  /** Statistical summary */
  stats: MetricStats;

  /** Historical data points (last 30 days) */
  history: MetricDataPoint[];

  /** Is this metric in a regression state? */
  isRegressed: boolean;

  /** Regression percentage (if any) */
  regressionPercent?: number;
}

/**
 * Regression alert
 */
export interface RegressionAlert {
  /** Alert ID */
  id: string;

  /** Metric that regressed */
  metricId: string;

  /** Metric name */
  metricName: string;

  /** Timestamp when regression was detected */
  detectedAt: number;

  /** Baseline value */
  baseline: number;

  /** Current value */
  current: number;

  /** Regression percentage */
  percent: number;

  /** Alert severity */
  severity: 'warning' | 'critical';

  /** Has alert been acknowledged? */
  acknowledged: boolean;
}

/**
 * Performance snapshot for export
 */
export interface PerformanceSnapshot {
  /** Snapshot timestamp */
  timestamp: number;

  /** All metrics at snapshot time */
  metrics: PerformanceMetric[];

  /** Active alerts */
  alerts: RegressionAlert[];

  /** System information */
  systemInfo: {
    userAgent: string;
    memory?: MemoryInfo;
    connection?: ConnectionInfo;
  };
}

/**
 * Memory information
 */
export interface MemoryInfo {
  /** Used JS heap size (bytes) */
  usedJSHeapSize: number;

  /** Total JS heap size (bytes) */
  totalJSHeapSize: number;

  /** JS heap size limit (bytes) */
  jsHeapSizeLimit: number;
}

/**
 * Connection information
 */
export interface ConnectionInfo {
  /** Effective connection type */
  effectiveType?: string;

  /** Round trip time (ms) */
  rtt?: number;

  /** Downlink speed (Mbps) */
  downlink?: number;
}

/**
 * Benchmark result
 */
export interface BenchmarkResult {
  /** Benchmark name */
  name: string;

  /** Category */
  category: MetricCategory;

  /** Duration in milliseconds */
  duration: number;

  /** Operations per second */
  opsPerSec: number;

  /** Timestamp */
  timestamp: number;

  /** Additional metadata */
  metadata?: Record<string, any>;
}

/**
 * Performance dashboard state
 */
export interface PerformanceDashboardState {
  /** All tracked metrics */
  metrics: Record<string, PerformanceMetric>;

  /** Active regression alerts */
  alerts: RegressionAlert[];

  /** Is dashboard collecting metrics? */
  isCollecting: boolean;

  /** Last update timestamp */
  lastUpdate: number;

  /** Time range for historical data (days) */
  historyDays: number;
}

/**
 * CSV export configuration
 */
export interface CSVExportConfig {
  /** Include historical data? */
  includeHistory: boolean;

  /** Include alerts? */
  includeAlerts: boolean;

  /** Time range in days */
  dayRange: number;

  /** Metrics to include (empty = all) */
  metricIds?: string[];
}

/**
 * Predefined metric targets (Issue #80 requirements)
 */
export const METRIC_TARGETS = {
  websocketLatencyP99: 50, // ms
  uiRenderTime: 16, // ms (60fps)
  memoryBaseline: 100, // MB
  processingThroughput: 1000, // ops/sec
  stateSyncTime: 100, // ms
  componentMountTime: 50, // ms
} as const;

/**
 * Regression detection threshold (10% per requirements)
 */
export const REGRESSION_THRESHOLD = 0.1; // 10%

/**
 * Calculate percentile from sorted array
 */
export function calculatePercentile(values: number[], percentile: number): number {
  if (values.length === 0) return 0;
  const sorted = [...values].sort((a, b) => a - b);
  const index = Math.ceil((percentile / 100) * sorted.length) - 1;
  return sorted[Math.max(0, index)];
}

/**
 * Calculate standard deviation
 */
export function calculateStdDev(values: number[], mean: number): number {
  if (values.length === 0) return 0;
  const variance = values.reduce((sum, val) => sum + Math.pow(val - mean, 2), 0) / values.length;
  return Math.sqrt(variance);
}

/**
 * Generate metric stats from data points
 */
export function generateMetricStats(dataPoints: MetricDataPoint[]): MetricStats {
  if (dataPoints.length === 0) {
    return {
      mean: 0,
      median: 0,
      p90: 0,
      p95: 0,
      p99: 0,
      min: 0,
      max: 0,
      stdDev: 0,
      count: 0,
    };
  }

  const values = dataPoints.map(dp => dp.value);
  const mean = values.reduce((sum, val) => sum + val, 0) / values.length;

  return {
    mean,
    median: calculatePercentile(values, 50),
    p90: calculatePercentile(values, 90),
    p95: calculatePercentile(values, 95),
    p99: calculatePercentile(values, 99),
    min: Math.min(...values),
    max: Math.max(...values),
    stdDev: calculateStdDev(values, mean),
    count: values.length,
  };
}

/**
 * Detect regression in metric
 */
export function detectRegression(
  current: number,
  baseline: number,
  threshold: number = REGRESSION_THRESHOLD
): { isRegressed: boolean; percent: number } {
  const percent = (current - baseline) / baseline;
  return {
    isRegressed: percent > threshold,
    percent,
  };
}

/**
 * Format bytes to human readable
 */
export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

/**
 * Format duration to human readable
 */
export function formatDuration(ms: number): string {
  if (ms < 1) return `${(ms * 1000).toFixed(2)}μs`;
  if (ms < 1000) return `${ms.toFixed(2)}ms`;
  return `${(ms / 1000).toFixed(2)}s`;
}
