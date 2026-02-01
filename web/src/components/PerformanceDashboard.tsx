/**
 * Performance Benchmarking Dashboard
 * Contract: Issue #80 - IMPORTANT (should pass before release)
 *
 * Features:
 * - Real-time metrics display
 * - Historical trend charts (30 days)
 * - Regression detection with alerts
 * - CSV export functionality
 * - Performance targets monitoring
 */

import React, { useEffect, useState, useMemo } from 'react';
import { performanceMetrics } from '../services/performanceMetrics';
import {
  PerformanceMetric,
  RegressionAlert,
  MetricCategory,
  formatBytes,
  formatDuration,
} from '../types/performance';

/**
 * Dashboard component
 */
export function PerformanceDashboard() {
  const [metrics, setMetrics] = useState<PerformanceMetric[]>([]);
  const [alerts, setAlerts] = useState<RegressionAlert[]>([]);
  const [selectedCategory, setSelectedCategory] = useState<MetricCategory | 'all'>('all');
  const [isCollecting, setIsCollecting] = useState(false);

  // Subscribe to metric updates
  useEffect(() => {
    const updateMetrics = () => {
      setMetrics(performanceMetrics.getMetrics());
      setAlerts(performanceMetrics.getActiveAlerts());
    };

    updateMetrics();
    const unsubscribe = performanceMetrics.subscribe(updateMetrics);

    return unsubscribe;
  }, []);

  // Start/stop collection
  useEffect(() => {
    if (isCollecting) {
      performanceMetrics.startCollection(5000);
    } else {
      performanceMetrics.stopCollection();
    }

    return () => performanceMetrics.stopCollection();
  }, [isCollecting]);

  // Filter metrics by category
  const filteredMetrics = useMemo(() => {
    if (selectedCategory === 'all') {
      return metrics;
    }
    return metrics.filter(m => m.category === selectedCategory);
  }, [metrics, selectedCategory]);

  // Export to CSV
  const handleExport = () => {
    const csv = performanceMetrics.exportToCSV(true);
    const blob = new Blob([csv], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `mbot-performance-${Date.now()}.csv`;
    a.click();
    URL.revokeObjectURL(url);
  };

  // Acknowledge alert
  const handleAcknowledgeAlert = (alertId: string) => {
    performanceMetrics.acknowledgeAlert(alertId);
  };

  // Clear all data
  const handleClear = () => {
    if (confirm('Clear all performance data? This cannot be undone.')) {
      performanceMetrics.clear();
    }
  };

  // Calculate overall health score
  const healthScore = useMemo(() => {
    if (metrics.length === 0) return 100;
    const passedMetrics = metrics.filter(m => !m.isRegressed).length;
    return Math.round((passedMetrics / metrics.length) * 100);
  }, [metrics]);

  return (
    <div className="performance-dashboard" data-testid="performance-dashboard">
      {/* Header */}
      <div className="dashboard-header">
        <h1>Performance Benchmarking Dashboard</h1>
        <div className="header-actions">
          <button
            onClick={() => setIsCollecting(!isCollecting)}
            className={isCollecting ? 'collecting' : ''}
            data-testid="toggle-collection"
          >
            {isCollecting ? '⏸ Pause' : '▶ Start'} Collection
          </button>
          <button onClick={handleExport} data-testid="export-csv">
            📊 Export CSV
          </button>
          <button onClick={handleClear} data-testid="clear-data" className="danger">
            🗑 Clear Data
          </button>
        </div>
      </div>

      {/* Health Score */}
      <div className="health-score" data-testid="health-score">
        <div className="score-circle">
          <svg viewBox="0 0 100 100">
            <circle cx="50" cy="50" r="45" fill="none" stroke="#e0e0e0" strokeWidth="10" />
            <circle
              cx="50"
              cy="50"
              r="45"
              fill="none"
              stroke={healthScore >= 80 ? '#4caf50' : healthScore >= 60 ? '#ff9800' : '#f44336'}
              strokeWidth="10"
              strokeDasharray={`${(healthScore / 100) * 283} 283`}
              strokeDashoffset="0"
              transform="rotate(-90 50 50)"
            />
          </svg>
          <div className="score-text">{healthScore}%</div>
        </div>
        <div className="score-label">Overall Health</div>
      </div>

      {/* Alerts */}
      {alerts.length > 0 && (
        <div className="alerts-section" data-testid="alerts-section">
          <h2>⚠️ Regression Alerts ({alerts.length})</h2>
          <div className="alerts-list">
            {alerts.map(alert => (
              <div
                key={alert.id}
                className={`alert alert-${alert.severity}`}
                data-testid={`alert-${alert.metricId}`}
              >
                <div className="alert-content">
                  <div className="alert-title">{alert.metricName}</div>
                  <div className="alert-details">
                    Baseline: {alert.baseline.toFixed(2)} → Current: {alert.current.toFixed(2)}
                    {' '}({(alert.percent * 100).toFixed(1)}% regression)
                  </div>
                  <div className="alert-time">
                    Detected: {new Date(alert.detectedAt).toLocaleString()}
                  </div>
                </div>
                <button
                  onClick={() => handleAcknowledgeAlert(alert.id)}
                  className="alert-acknowledge"
                  data-testid={`acknowledge-${alert.metricId}`}
                >
                  ✓ Acknowledge
                </button>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Category Filter */}
      <div className="category-filter" data-testid="category-filter">
        <button
          onClick={() => setSelectedCategory('all')}
          className={selectedCategory === 'all' ? 'active' : ''}
        >
          All
        </button>
        <button
          onClick={() => setSelectedCategory('websocket')}
          className={selectedCategory === 'websocket' ? 'active' : ''}
        >
          WebSocket
        </button>
        <button
          onClick={() => setSelectedCategory('ui')}
          className={selectedCategory === 'ui' ? 'active' : ''}
        >
          UI
        </button>
        <button
          onClick={() => setSelectedCategory('memory')}
          className={selectedCategory === 'memory' ? 'active' : ''}
        >
          Memory
        </button>
        <button
          onClick={() => setSelectedCategory('processing')}
          className={selectedCategory === 'processing' ? 'active' : ''}
        >
          Processing
        </button>
        <button
          onClick={() => setSelectedCategory('sync')}
          className={selectedCategory === 'sync' ? 'active' : ''}
        >
          Sync
        </button>
        <button
          onClick={() => setSelectedCategory('component')}
          className={selectedCategory === 'component' ? 'active' : ''}
        >
          Components
        </button>
      </div>

      {/* Metrics Grid */}
      <div className="metrics-grid" data-testid="metrics-grid">
        {filteredMetrics.map(metric => (
          <MetricCard key={metric.id} metric={metric} />
        ))}
      </div>
    </div>
  );
}

/**
 * Individual metric card component
 */
function MetricCard({ metric }: { metric: PerformanceMetric }) {
  const formatValue = (value: number) => {
    if (metric.unit === 'MB') return formatBytes(value * 1024 * 1024);
    if (metric.unit === 'ms') return formatDuration(value);
    return `${value.toFixed(2)} ${metric.unit}`;
  };

  const getStatusColor = () => {
    if (metric.isRegressed) return '#f44336';
    if (metric.stats.p95 > metric.target * 0.9) return '#ff9800';
    return '#4caf50';
  };

  return (
    <div
      className={`metric-card ${metric.isRegressed ? 'regressed' : ''}`}
      data-testid={`metric-${metric.id}`}
      style={{ borderLeftColor: getStatusColor() }}
    >
      <div className="metric-header">
        <h3>{metric.name}</h3>
        <span className="metric-category">{metric.category}</span>
      </div>

      <div className="metric-current">
        <div className="metric-value" data-testid={`${metric.id}-current`}>
          {formatValue(metric.current)}
        </div>
        <div className="metric-label">Current</div>
      </div>

      <div className="metric-target">
        <span>Target: {formatValue(metric.target)}</span>
        {metric.isRegressed && metric.regressionPercent && (
          <span className="regression-badge">
            +{(metric.regressionPercent * 100).toFixed(1)}%
          </span>
        )}
      </div>

      <div className="metric-stats">
        <div className="stat">
          <span className="stat-label">P50</span>
          <span className="stat-value" data-testid={`${metric.id}-p50`}>
            {formatValue(metric.stats.median)}
          </span>
        </div>
        <div className="stat">
          <span className="stat-label">P95</span>
          <span className="stat-value" data-testid={`${metric.id}-p95`}>
            {formatValue(metric.stats.p95)}
          </span>
        </div>
        <div className="stat">
          <span className="stat-label">P99</span>
          <span className="stat-value" data-testid={`${metric.id}-p99`}>
            {formatValue(metric.stats.p99)}
          </span>
        </div>
        <div className="stat">
          <span className="stat-label">Samples</span>
          <span className="stat-value">{metric.stats.count}</span>
        </div>
      </div>

      {/* Mini chart */}
      {metric.history.length > 0 && (
        <div className="metric-chart" data-testid={`${metric.id}-chart`}>
          <MiniChart data={metric.history} target={metric.target} />
        </div>
      )}
    </div>
  );
}

/**
 * Mini line chart component
 */
function MiniChart({
  data,
  target,
}: {
  data: Array<{ timestamp: number; value: number }>;
  target: number;
}) {
  const width = 300;
  const height = 60;
  const padding = 5;

  const values = data.map(d => d.value);
  const minVal = Math.min(...values, target) * 0.9;
  const maxVal = Math.max(...values, target) * 1.1;

  const points = data.map((d, i) => {
    const x = padding + (i / (data.length - 1)) * (width - 2 * padding);
    const y = height - padding - ((d.value - minVal) / (maxVal - minVal)) * (height - 2 * padding);
    return `${x},${y}`;
  });

  const targetY =
    height - padding - ((target - minVal) / (maxVal - minVal)) * (height - 2 * padding);

  return (
    <svg viewBox={`0 0 ${width} ${height}`} className="mini-chart-svg">
      {/* Target line */}
      <line
        x1={padding}
        y1={targetY}
        x2={width - padding}
        y2={targetY}
        stroke="#ff9800"
        strokeWidth="1"
        strokeDasharray="2,2"
      />

      {/* Data line */}
      <polyline
        points={points.join(' ')}
        fill="none"
        stroke="#2196f3"
        strokeWidth="2"
      />

      {/* Data points */}
      {points.map((point, i) => (
        <circle
          key={i}
          cx={point.split(',')[0]}
          cy={point.split(',')[1]}
          r="2"
          fill="#2196f3"
        />
      ))}
    </svg>
  );
}

/**
 * Dashboard styles (inline for simplicity)
 */
const styles = `
.performance-dashboard {
  padding: 20px;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
}

.dashboard-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 30px;
}

.dashboard-header h1 {
  margin: 0;
  font-size: 24px;
  color: #333;
}

.header-actions {
  display: flex;
  gap: 10px;
}

.header-actions button {
  padding: 8px 16px;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  background: #2196f3;
  color: white;
  transition: background 0.2s;
}

.header-actions button:hover {
  background: #1976d2;
}

.header-actions button.collecting {
  background: #4caf50;
}

.header-actions button.danger {
  background: #f44336;
}

.health-score {
  text-align: center;
  margin-bottom: 30px;
}

.score-circle {
  position: relative;
  width: 150px;
  height: 150px;
  margin: 0 auto;
}

.score-circle svg {
  transform: rotate(-90deg);
}

.score-text {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  font-size: 36px;
  font-weight: bold;
  color: #333;
}

.score-label {
  margin-top: 10px;
  font-size: 18px;
  color: #666;
}

.alerts-section {
  margin-bottom: 30px;
  padding: 20px;
  background: #fff3cd;
  border-left: 4px solid #ff9800;
  border-radius: 4px;
}

.alerts-section h2 {
  margin: 0 0 15px 0;
  font-size: 18px;
  color: #856404;
}

.alerts-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.alert {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 15px;
  background: white;
  border-radius: 4px;
  border-left: 4px solid;
}

.alert-warning {
  border-left-color: #ff9800;
}

.alert-critical {
  border-left-color: #f44336;
}

.alert-content {
  flex: 1;
}

.alert-title {
  font-weight: bold;
  margin-bottom: 5px;
}

.alert-details {
  color: #666;
  font-size: 14px;
}

.alert-time {
  color: #999;
  font-size: 12px;
  margin-top: 5px;
}

.alert-acknowledge {
  padding: 6px 12px;
  border: none;
  border-radius: 4px;
  background: #4caf50;
  color: white;
  cursor: pointer;
}

.category-filter {
  display: flex;
  gap: 10px;
  margin-bottom: 20px;
  flex-wrap: wrap;
}

.category-filter button {
  padding: 8px 16px;
  border: 1px solid #ddd;
  border-radius: 4px;
  background: white;
  cursor: pointer;
  transition: all 0.2s;
}

.category-filter button.active {
  background: #2196f3;
  color: white;
  border-color: #2196f3;
}

.metrics-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
  gap: 20px;
}

.metric-card {
  background: white;
  padding: 20px;
  border-radius: 8px;
  border-left: 4px solid #4caf50;
  box-shadow: 0 2px 4px rgba(0,0,0,0.1);
}

.metric-card.regressed {
  background: #ffebee;
}

.metric-header {
  display: flex;
  justify-content: space-between;
  align-items: start;
  margin-bottom: 15px;
}

.metric-header h3 {
  margin: 0;
  font-size: 16px;
  color: #333;
}

.metric-category {
  font-size: 12px;
  padding: 4px 8px;
  background: #e3f2fd;
  color: #1976d2;
  border-radius: 4px;
}

.metric-current {
  text-align: center;
  margin-bottom: 15px;
}

.metric-value {
  font-size: 32px;
  font-weight: bold;
  color: #2196f3;
}

.metric-label {
  font-size: 12px;
  color: #999;
  margin-top: 5px;
}

.metric-target {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px;
  background: #f5f5f5;
  border-radius: 4px;
  margin-bottom: 15px;
  font-size: 14px;
}

.regression-badge {
  background: #f44336;
  color: white;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 12px;
}

.metric-stats {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 10px;
  margin-bottom: 15px;
}

.stat {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 8px;
  background: #fafafa;
  border-radius: 4px;
}

.stat-label {
  font-size: 11px;
  color: #999;
  margin-bottom: 4px;
}

.stat-value {
  font-size: 14px;
  font-weight: bold;
  color: #333;
}

.metric-chart {
  margin-top: 15px;
}

.mini-chart-svg {
  width: 100%;
  height: 60px;
}
`;

// Inject styles
if (typeof document !== 'undefined') {
  const styleSheet = document.createElement('style');
  styleSheet.textContent = styles;
  document.head.appendChild(styleSheet);
}
