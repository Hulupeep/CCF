/**
 * Performance Metrics Service
 * Contract: Issue #80 - Performance Benchmarking Dashboard
 *
 * Features:
 * - Real-time metric collection
 * - Historical data storage (30 days)
 * - Regression detection
 * - CSV export
 * - Memory and WebSocket monitoring
 */

import {
  PerformanceMetric,
  MetricDataPoint,
  MetricCategory,
  RegressionAlert,
  PerformanceSnapshot,
  BenchmarkResult,
  generateMetricStats,
  detectRegression,
  METRIC_TARGETS,
  REGRESSION_THRESHOLD,
  MemoryInfo,
  ConnectionInfo,
} from '../types/performance';

/**
 * Storage key for persisting metrics
 */
const STORAGE_KEY = 'mbot_performance_metrics';
const STORAGE_KEY_ALERTS = 'mbot_performance_alerts';

/**
 * Maximum age of historical data (30 days in milliseconds)
 */
const MAX_HISTORY_AGE = 30 * 24 * 60 * 60 * 1000;

/**
 * Performance Metrics Service
 */
export class PerformanceMetricsService {
  private metrics: Map<string, PerformanceMetric> = new Map();
  private alerts: Map<string, RegressionAlert> = new Map();
  private observers: Set<() => void> = new Set();
  private collectInterval: number | null = null;

  constructor() {
    this.loadFromStorage();
    this.initializeMetrics();
  }

  /**
   * Initialize default metrics
   */
  private initializeMetrics(): void {
    const defaultMetrics: Omit<PerformanceMetric, 'current' | 'stats' | 'history' | 'isRegressed'>[] = [
      {
        id: 'websocket_latency',
        name: 'WebSocket Message Latency',
        category: 'websocket',
        unit: 'ms',
        target: METRIC_TARGETS.websocketLatencyP99,
      },
      {
        id: 'ui_render_time',
        name: 'UI Render Time',
        category: 'ui',
        unit: 'ms',
        target: METRIC_TARGETS.uiRenderTime,
      },
      {
        id: 'memory_usage',
        name: 'Memory Usage',
        category: 'memory',
        unit: 'MB',
        target: METRIC_TARGETS.memoryBaseline,
      },
      {
        id: 'processing_throughput',
        name: 'Data Processing Throughput',
        category: 'processing',
        unit: 'ops/sec',
        target: METRIC_TARGETS.processingThroughput,
      },
      {
        id: 'state_sync_time',
        name: 'State Synchronization Time',
        category: 'sync',
        unit: 'ms',
        target: METRIC_TARGETS.stateSyncTime,
      },
      {
        id: 'component_mount_time',
        name: 'Component Mount/Unmount Time',
        category: 'component',
        unit: 'ms',
        target: METRIC_TARGETS.componentMountTime,
      },
    ];

    for (const metric of defaultMetrics) {
      if (!this.metrics.has(metric.id)) {
        this.metrics.set(metric.id, {
          ...metric,
          current: 0,
          stats: {
            mean: 0,
            median: 0,
            p90: 0,
            p95: 0,
            p99: 0,
            min: 0,
            max: 0,
            stdDev: 0,
            count: 0,
          },
          history: [],
          isRegressed: false,
        });
      }
    }
  }

  /**
   * Load metrics from localStorage
   */
  private loadFromStorage(): void {
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (stored) {
        const data = JSON.parse(stored);
        this.metrics = new Map(Object.entries(data));
        this.cleanOldHistory();
      }

      const storedAlerts = localStorage.getItem(STORAGE_KEY_ALERTS);
      if (storedAlerts) {
        const data = JSON.parse(storedAlerts);
        this.alerts = new Map(Object.entries(data));
      }
    } catch (error) {
      console.error('Failed to load performance metrics from storage:', error);
    }
  }

  /**
   * Save metrics to localStorage
   */
  private saveToStorage(): void {
    try {
      const data = Object.fromEntries(this.metrics);
      localStorage.setItem(STORAGE_KEY, JSON.stringify(data));

      const alertData = Object.fromEntries(this.alerts);
      localStorage.setItem(STORAGE_KEY_ALERTS, JSON.stringify(alertData));
    } catch (error) {
      console.error('Failed to save performance metrics to storage:', error);
    }
  }

  /**
   * Clean old historical data (keep last 30 days)
   */
  private cleanOldHistory(): void {
    const cutoff = Date.now() - MAX_HISTORY_AGE;

    for (const [id, metric] of this.metrics.entries()) {
      metric.history = metric.history.filter(dp => dp.timestamp >= cutoff);
      this.metrics.set(id, metric);
    }
  }

  /**
   * Record a metric value
   */
  recordMetric(metricId: string, value: number, label?: string): void {
    const metric = this.metrics.get(metricId);
    if (!metric) {
      console.warn(`Unknown metric: ${metricId}`);
      return;
    }

    // Add data point
    const dataPoint: MetricDataPoint = {
      timestamp: Date.now(),
      value,
      label,
    };

    metric.history.push(dataPoint);
    metric.current = value;

    // Recalculate stats
    metric.stats = generateMetricStats(metric.history);

    // Check for regression (use p95 for comparison)
    const regression = detectRegression(metric.stats.p95, metric.target, REGRESSION_THRESHOLD);
    metric.isRegressed = regression.isRegressed;
    metric.regressionPercent = regression.percent;

    // Create alert if regressed
    if (regression.isRegressed) {
      this.createRegressionAlert(metric, regression.percent);
    }

    this.metrics.set(metricId, metric);
    this.saveToStorage();
    this.notifyObservers();
  }

  /**
   * Create a regression alert
   */
  private createRegressionAlert(metric: PerformanceMetric, percent: number): void {
    const alertId = `${metric.id}_${Date.now()}`;

    // Don't create duplicate alerts for same metric
    const existingAlert = Array.from(this.alerts.values()).find(
      a => a.metricId === metric.id && !a.acknowledged
    );

    if (existingAlert) {
      return;
    }

    const alert: RegressionAlert = {
      id: alertId,
      metricId: metric.id,
      metricName: metric.name,
      detectedAt: Date.now(),
      baseline: metric.target,
      current: metric.stats.p95,
      percent,
      severity: percent > 0.25 ? 'critical' : 'warning',
      acknowledged: false,
    };

    this.alerts.set(alertId, alert);
    this.saveToStorage();
  }

  /**
   * Acknowledge an alert
   */
  acknowledgeAlert(alertId: string): void {
    const alert = this.alerts.get(alertId);
    if (alert) {
      alert.acknowledged = true;
      this.alerts.set(alertId, alert);
      this.saveToStorage();
      this.notifyObservers();
    }
  }

  /**
   * Get all metrics
   */
  getMetrics(): PerformanceMetric[] {
    return Array.from(this.metrics.values());
  }

  /**
   * Get metrics by category
   */
  getMetricsByCategory(category: MetricCategory): PerformanceMetric[] {
    return this.getMetrics().filter(m => m.category === category);
  }

  /**
   * Get single metric
   */
  getMetric(metricId: string): PerformanceMetric | undefined {
    return this.metrics.get(metricId);
  }

  /**
   * Get all alerts
   */
  getAlerts(): RegressionAlert[] {
    return Array.from(this.alerts.values());
  }

  /**
   * Get unacknowledged alerts
   */
  getActiveAlerts(): RegressionAlert[] {
    return this.getAlerts().filter(a => !a.acknowledged);
  }

  /**
   * Record a benchmark result
   */
  recordBenchmark(result: BenchmarkResult): void {
    // Map benchmark to appropriate metric
    const metricMap: Record<string, string> = {
      websocket_latency: 'websocket_latency',
      ui_render: 'ui_render_time',
      memory: 'memory_usage',
      processing: 'processing_throughput',
      state_sync: 'state_sync_time',
      component_lifecycle: 'component_mount_time',
    };

    const metricId = metricMap[result.name];
    if (metricId) {
      this.recordMetric(metricId, result.duration, result.name);
    }
  }

  /**
   * Get current memory info
   */
  getMemoryInfo(): MemoryInfo | undefined {
    if ('memory' in performance && (performance as any).memory) {
      const mem = (performance as any).memory;
      return {
        usedJSHeapSize: mem.usedJSHeapSize,
        totalJSHeapSize: mem.totalJSHeapSize,
        jsHeapSizeLimit: mem.jsHeapSizeLimit,
      };
    }
    return undefined;
  }

  /**
   * Get connection info
   */
  getConnectionInfo(): ConnectionInfo | undefined {
    if ('connection' in navigator) {
      const conn = (navigator as any).connection;
      return {
        effectiveType: conn.effectiveType,
        rtt: conn.rtt,
        downlink: conn.downlink,
      };
    }
    return undefined;
  }

  /**
   * Create performance snapshot for export
   */
  createSnapshot(): PerformanceSnapshot {
    return {
      timestamp: Date.now(),
      metrics: this.getMetrics(),
      alerts: this.getAlerts(),
      systemInfo: {
        userAgent: navigator.userAgent,
        memory: this.getMemoryInfo(),
        connection: this.getConnectionInfo(),
      },
    };
  }

  /**
   * Export metrics to CSV
   */
  exportToCSV(includeHistory: boolean = true): string {
    const lines: string[] = [];

    // Header
    lines.push('Metric ID,Name,Category,Unit,Target,Current,Mean,P95,P99,Is Regressed');

    // Metrics
    for (const metric of this.getMetrics()) {
      lines.push(
        [
          metric.id,
          metric.name,
          metric.category,
          metric.unit,
          metric.target,
          metric.current,
          metric.stats.mean.toFixed(2),
          metric.stats.p95.toFixed(2),
          metric.stats.p99.toFixed(2),
          metric.isRegressed ? 'Yes' : 'No',
        ].join(',')
      );
    }

    // Historical data
    if (includeHistory) {
      lines.push('');
      lines.push('Historical Data');
      lines.push('Metric ID,Timestamp,Value,Label');

      for (const metric of this.getMetrics()) {
        for (const dp of metric.history) {
          lines.push(
            [
              metric.id,
              new Date(dp.timestamp).toISOString(),
              dp.value,
              dp.label || '',
            ].join(',')
          );
        }
      }
    }

    return lines.join('\n');
  }

  /**
   * Start automatic metric collection
   */
  startCollection(intervalMs: number = 5000): void {
    if (this.collectInterval !== null) {
      return;
    }

    this.collectInterval = window.setInterval(() => {
      // Collect memory metrics
      const memInfo = this.getMemoryInfo();
      if (memInfo) {
        this.recordMetric('memory_usage', memInfo.usedJSHeapSize / (1024 * 1024)); // Convert to MB
      }
    }, intervalMs);
  }

  /**
   * Stop automatic metric collection
   */
  stopCollection(): void {
    if (this.collectInterval !== null) {
      clearInterval(this.collectInterval);
      this.collectInterval = null;
    }
  }

  /**
   * Subscribe to metric updates
   */
  subscribe(callback: () => void): () => void {
    this.observers.add(callback);
    return () => this.observers.delete(callback);
  }

  /**
   * Notify all observers of changes
   */
  private notifyObservers(): void {
    this.observers.forEach(callback => callback());
  }

  /**
   * Clear all metrics and alerts
   */
  clear(): void {
    this.metrics.clear();
    this.alerts.clear();
    this.initializeMetrics();
    this.saveToStorage();
    this.notifyObservers();
  }
}

/**
 * Singleton instance
 */
export const performanceMetrics = new PerformanceMetricsService();
