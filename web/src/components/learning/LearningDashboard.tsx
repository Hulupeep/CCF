/**
 * Learning Dashboard - Visual interface for learned patterns
 * Shows active patterns, performance metrics, and learning insights
 */

import React, { useEffect, useState } from 'react';
import { LearningEngine } from '../../services/learning/LearningEngine';
import { PatternStore } from '../../services/learning/PatternStore';
import { Overseer } from '../../services/learning/Overseer';
import {
  CrystallizedPattern,
  PatternStatistics,
  LearningReport,
  LearningInsight,
} from '../../types/learning';

interface LearningDashboardProps {
  learningEngine: LearningEngine;
  patternStore: PatternStore;
  overseer: Overseer;
}

export function LearningDashboard({
  learningEngine,
  patternStore,
  overseer,
}: LearningDashboardProps) {
  const [patterns, setPatterns] = useState<CrystallizedPattern[]>([]);
  const [stats, setStats] = useState<PatternStatistics | null>(null);
  const [insights, setInsights] = useState<LearningInsight[]>([]);
  const [latestReport, setLatestReport] = useState<LearningReport | null>(null);
  const [overseerStatus, setOverseerStatus] = useState<any>(null);

  useEffect(() => {
    loadData();

    // Refresh every 10 seconds
    const interval = setInterval(loadData, 10000);
    return () => clearInterval(interval);
  }, [learningEngine, patternStore, overseer]);

  const loadData = async () => {
    // Load patterns
    const loadedPatterns = await patternStore.loadPatterns();
    setPatterns(loadedPatterns);

    // Load stats
    const patternStats = await patternStore.getPatternStats();
    setStats(patternStats);

    // Load insights
    const engineInsights = learningEngine.getInsights();
    setInsights(engineInsights);

    // Load latest report
    const report = overseer.getLatestReport();
    setLatestReport(report);

    // Load overseer status
    const status = overseer.getStatus();
    setOverseerStatus(status);
  };

  return (
    <div data-testid="learning-dashboard" className="learning-dashboard">
      <h1>Learning Dashboard</h1>

      {/* Overseer Status */}
      <section className="overseer-status" data-testid="overseer-status">
        <h2>Overseer Status</h2>
        <div className="status-card">
          <div className="status-item">
            <span className="label">Status:</span>
            <span className={`value ${overseerStatus?.running ? 'active' : 'inactive'}`}>
              {overseerStatus?.running ? '✅ Running' : '❌ Stopped'}
            </span>
          </div>
          <div className="status-item">
            <span className="label">Total Cycles:</span>
            <span className="value">{overseerStatus?.totalCycles || 0}</span>
          </div>
          <div className="status-item">
            <span className="label">Last Cycle:</span>
            <span className="value">
              {latestReport
                ? new Date(latestReport.timestamp).toLocaleString()
                : 'Never'}
            </span>
          </div>
        </div>
      </section>

      {/* Performance Metrics */}
      <section className="performance-metrics" data-testid="learning-metrics">
        <h2>Performance Metrics</h2>
        <div className="metrics-grid">
          <div className="metric-card">
            <div className="metric-value">{stats?.totalPatterns || 0}</div>
            <div className="metric-label">Total Patterns</div>
          </div>
          <div className="metric-card">
            <div className="metric-value" data-testid="active-patterns-count">{stats?.activePatterns || 0}</div>
            <div className="metric-label">Active Patterns</div>
          </div>
          <div className="metric-card">
            <div className="metric-value">{stats?.totalUsages || 0}</div>
            <div className="metric-label">Total Uses</div>
          </div>
          <div className="metric-card">
            <div className="metric-value">
              {stats ? (stats.avgSuccessRate * 100).toFixed(1) : 0}%
            </div>
            <div className="metric-label">Avg Success Rate</div>
          </div>
        </div>
      </section>

      {/* Active Patterns */}
      <section className="patterns-list">
        <h2>Active Patterns</h2>
        <div className="patterns-grid">
          {patterns.length === 0 ? (
            <div className="empty-state">
              <p>No patterns learned yet.</p>
              <p>Interact with the bot to start building patterns!</p>
            </div>
          ) : (
            patterns.map((pattern) => (
              <PatternCard key={pattern.id} pattern={pattern} />
            ))
          )}
        </div>
      </section>

      {/* Learning Insights */}
      <section className="insights">
        <h2>Learning Insights</h2>
        <div className="insights-list">
          {insights.length === 0 ? (
            <div className="empty-state">
              <p>No insights yet.</p>
            </div>
          ) : (
            insights.slice(0, 5).map((insight) => (
              <InsightCard key={insight.id} insight={insight} />
            ))
          )}
        </div>
      </section>

      {/* Latest Report */}
      {latestReport && (
        <section className="latest-report">
          <h2>Latest Learning Cycle</h2>
          <ReportCard report={latestReport} />
        </section>
      )}
    </div>
  );
}

/**
 * Pattern Card Component
 */
function PatternCard({ pattern }: { pattern: CrystallizedPattern }) {
  const [expanded, setExpanded] = useState(false);

  return (
    <div
      className="pattern-card"
      data-testid={`pattern-${pattern.id}`}
      onClick={() => setExpanded(!expanded)}
    >
      <div className="pattern-header">
        <h3>{pattern.name}</h3>
        <div className="pattern-badges">
          <span className="badge success-rate" data-testid={`pattern-success-${pattern.id}`}>
            {(pattern.successRate * 100).toFixed(0)}% success
          </span>
          <span className="badge usage-count" data-testid={`pattern-usage-${pattern.id}`}>{pattern.usageCount} uses</span>
        </div>
      </div>

      <p className="pattern-description">{pattern.description}</p>

      {expanded && (
        <div className="pattern-details">
          <div className="detail-row">
            <span className="detail-label">Confidence:</span>
            <span className="detail-value">
              {(pattern.confidence * 100).toFixed(1)}%
            </span>
          </div>
          <div className="detail-row">
            <span className="detail-label">Avg Duration:</span>
            <span className="detail-value">{pattern.avgDuration.toFixed(0)}ms</span>
          </div>
          <div className="detail-row">
            <span className="detail-label">Created:</span>
            <span className="detail-value">
              {new Date(pattern.createdAt).toLocaleDateString()}
            </span>
          </div>
          <div className="detail-row">
            <span className="detail-label">Last Used:</span>
            <span className="detail-value">
              {new Date(pattern.lastUsed).toLocaleString()}
            </span>
          </div>

          {pattern.improvementTrajectory && pattern.improvementTrajectory.length > 0 && (
            <div className="trajectory">
              <span className="detail-label">Trajectory:</span>
              <div className="trajectory-chart">
                {pattern.improvementTrajectory.map((value, i) => (
                  <div
                    key={i}
                    className="trajectory-bar"
                    style={{ height: `${value * 100}%` }}
                    title={`${(value * 100).toFixed(1)}%`}
                  />
                ))}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

/**
 * Insight Card Component
 */
function InsightCard({ insight }: { insight: LearningInsight }) {
  const getTypeIcon = (type: string) => {
    switch (type) {
      case 'pattern_detected':
        return '🔍';
      case 'high_success':
        return '✨';
      case 'frequent_failure':
        return '⚠️';
      case 'optimization':
        return '⚡';
      default:
        return '💡';
    }
  };

  return (
    <div className="insight-card" data-testid={`insight-${insight.id}`}>
      <div className="insight-header">
        <span className="insight-icon">{getTypeIcon(insight.type)}</span>
        <span className="insight-type">{insight.type.replace('_', ' ')}</span>
        <span className="insight-confidence">
          {(insight.confidence * 100).toFixed(0)}% confidence
        </span>
      </div>
      <p className="insight-message">{insight.message}</p>
      <span className="insight-timestamp">
        {new Date(insight.createdAt).toLocaleString()}
      </span>
    </div>
  );
}

/**
 * Report Card Component
 */
function ReportCard({ report }: { report: LearningReport }) {
  return (
    <div className="report-card" data-testid="learning-report">
      <div className="report-summary">
        <div className="summary-item">
          <span className="label">Observations Analyzed:</span>
          <span className="value">{report.observationsAnalyzed}</span>
        </div>
        <div className="summary-item">
          <span className="label">Patterns Detected:</span>
          <span className="value">{report.patternsDetected}</span>
        </div>
        <div className="summary-item">
          <span className="label">Tokens Saved:</span>
          <span className="value" data-testid="tokens-saved">{report.performanceMetrics.tokensSaved}</span>
        </div>
      </div>

      <div className="report-actions">
        <h3>Actions Triggered</h3>
        <ul>
          {report.actionsTriggered.crystallized.length > 0 && (
            <li>
              ✨ Crystallized {report.actionsTriggered.crystallized.length} patterns
            </li>
          )}
          {report.actionsTriggered.pruned.length > 0 && (
            <li>🗑️ Pruned {report.actionsTriggered.pruned.length} stale patterns</li>
          )}
          {report.actionsTriggered.insights.length > 0 && (
            <li>💡 Generated {report.actionsTriggered.insights.length} insights</li>
          )}
        </ul>
      </div>

      {report.recommendations.length > 0 && (
        <div className="report-recommendations">
          <h3>Recommendations</h3>
          <ul>
            {report.recommendations.slice(0, 5).map((rec, i) => (
              <li key={i}>
                {rec.type === 'crystallize' && '✨'}
                {rec.type === 'prune' && '🗑️'}
                {rec.type === 'monitor' && '👁️'}
                {rec.type === 'update' && '🔄'} {rec.reason}
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}
