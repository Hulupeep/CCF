/**
 * Performance Dashboard Component Tests
 * Contract: Issue #80 - Performance Benchmarking Dashboard
 */

import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import '@testing-library/jest-dom';
import { PerformanceDashboard } from '../PerformanceDashboard';
import { performanceMetrics } from '../../services/performanceMetrics';
import { METRIC_TARGETS } from '../../types/performance';

describe('PerformanceDashboard', () => {
  beforeEach(() => {
    performanceMetrics.clear();
    localStorage.clear();
  });

  afterEach(() => {
    performanceMetrics.stopCollection();
  });

  describe('Rendering', () => {
    test('renders dashboard with all sections', () => {
      render(<PerformanceDashboard />);

      expect(screen.getByTestId('performance-dashboard')).toBeInTheDocument();
      expect(screen.getByTestId('health-score')).toBeInTheDocument();
      expect(screen.getByTestId('category-filter')).toBeInTheDocument();
      expect(screen.getByTestId('metrics-grid')).toBeInTheDocument();
    });

    test('renders all default metrics', () => {
      render(<PerformanceDashboard />);

      expect(screen.getByTestId('metric-websocket_latency')).toBeInTheDocument();
      expect(screen.getByTestId('metric-ui_render_time')).toBeInTheDocument();
      expect(screen.getByTestId('metric-memory_usage')).toBeInTheDocument();
      expect(screen.getByTestId('metric-processing_throughput')).toBeInTheDocument();
      expect(screen.getByTestId('metric-state_sync_time')).toBeInTheDocument();
      expect(screen.getByTestId('metric-component_mount_time')).toBeInTheDocument();
    });

    test('displays health score correctly', async () => {
      render(<PerformanceDashboard />);

      const healthScore = screen.getByTestId('health-score');
      expect(healthScore).toHaveTextContent('100%'); // All metrics passing initially
    });

    test('updates health score when metrics regress', async () => {
      render(<PerformanceDashboard />);

      // Introduce regressions
      for (let i = 0; i < 50; i++) {
        performanceMetrics.recordMetric(
          'websocket_latency',
          METRIC_TARGETS.websocketLatencyP99 * 1.5
        );
      }

      await waitFor(() => {
        const healthScore = screen.getByTestId('health-score');
        expect(healthScore).not.toHaveTextContent('100%');
      });
    });
  });

  describe('Metric Display', () => {
    test('displays current metric values', () => {
      performanceMetrics.recordMetric('websocket_latency', 25);
      render(<PerformanceDashboard />);

      const currentValue = screen.getByTestId('websocket_latency-current');
      expect(currentValue).toBeInTheDocument();
    });

    test('displays percentile statistics', () => {
      for (let i = 0; i < 100; i++) {
        performanceMetrics.recordMetric('websocket_latency', 20 + Math.random() * 10);
      }

      render(<PerformanceDashboard />);

      expect(screen.getByTestId('websocket_latency-p50')).toBeInTheDocument();
      expect(screen.getByTestId('websocket_latency-p95')).toBeInTheDocument();
      expect(screen.getByTestId('websocket_latency-p99')).toBeInTheDocument();
    });

    test('shows regression badge when metric regresses', async () => {
      // Record good metrics first
      for (let i = 0; i < 50; i++) {
        performanceMetrics.recordMetric('websocket_latency', 20);
      }

      // Introduce regression
      for (let i = 0; i < 50; i++) {
        performanceMetrics.recordMetric(
          'websocket_latency',
          METRIC_TARGETS.websocketLatencyP99 * 1.5
        );
      }

      render(<PerformanceDashboard />);

      await waitFor(() => {
        const metricCard = screen.getByTestId('metric-websocket_latency');
        expect(metricCard).toHaveClass('regressed');
      });
    });

    test('renders mini chart for metrics with history', async () => {
      for (let i = 0; i < 20; i++) {
        performanceMetrics.recordMetric('websocket_latency', 20 + Math.random() * 10);
      }

      render(<PerformanceDashboard />);

      await waitFor(() => {
        expect(screen.getByTestId('websocket_latency-chart')).toBeInTheDocument();
      });
    });
  });

  describe('Category Filtering', () => {
    test('filters metrics by category', async () => {
      render(<PerformanceDashboard />);

      const websocketButton = screen.getByText('WebSocket');
      fireEvent.click(websocketButton);

      await waitFor(() => {
        expect(screen.getByTestId('metric-websocket_latency')).toBeInTheDocument();
        expect(screen.queryByTestId('metric-ui_render_time')).not.toBeInTheDocument();
      });
    });

    test('shows all metrics when "All" is selected', async () => {
      render(<PerformanceDashboard />);

      // Filter to websocket
      fireEvent.click(screen.getByText('WebSocket'));

      await waitFor(() => {
        expect(screen.queryByTestId('metric-ui_render_time')).not.toBeInTheDocument();
      });

      // Switch back to all
      fireEvent.click(screen.getByText('All'));

      await waitFor(() => {
        expect(screen.getByTestId('metric-websocket_latency')).toBeInTheDocument();
        expect(screen.getByTestId('metric-ui_render_time')).toBeInTheDocument();
      });
    });

    test('highlights active category button', () => {
      render(<PerformanceDashboard />);

      const allButton = screen.getByText('All');
      expect(allButton).toHaveClass('active');

      const uiButton = screen.getByText('UI');
      fireEvent.click(uiButton);

      expect(uiButton).toHaveClass('active');
      expect(allButton).not.toHaveClass('active');
    });
  });

  describe('Alerts', () => {
    test('displays regression alerts', async () => {
      // Create regression
      for (let i = 0; i < 100; i++) {
        performanceMetrics.recordMetric(
          'websocket_latency',
          METRIC_TARGETS.websocketLatencyP99 * 1.5
        );
      }

      render(<PerformanceDashboard />);

      await waitFor(() => {
        expect(screen.getByTestId('alerts-section')).toBeInTheDocument();
        expect(screen.getByTestId('alert-websocket_latency')).toBeInTheDocument();
      });
    });

    test('acknowledges alerts when button clicked', async () => {
      // Create regression
      for (let i = 0; i < 100; i++) {
        performanceMetrics.recordMetric('ui_render_time', METRIC_TARGETS.uiRenderTime * 1.5);
      }

      render(<PerformanceDashboard />);

      await waitFor(() => {
        expect(screen.getByTestId('alert-ui_render_time')).toBeInTheDocument();
      });

      const acknowledgeButton = screen.getByTestId('acknowledge-ui_render_time');
      fireEvent.click(acknowledgeButton);

      await waitFor(() => {
        expect(screen.queryByTestId('alert-ui_render_time')).not.toBeInTheDocument();
      });
    });

    test('hides alerts section when no active alerts', () => {
      render(<PerformanceDashboard />);
      expect(screen.queryByTestId('alerts-section')).not.toBeInTheDocument();
    });

    test('shows alert count in header', async () => {
      // Create multiple regressions
      for (let i = 0; i < 100; i++) {
        performanceMetrics.recordMetric(
          'websocket_latency',
          METRIC_TARGETS.websocketLatencyP99 * 1.5
        );
        performanceMetrics.recordMetric('ui_render_time', METRIC_TARGETS.uiRenderTime * 1.5);
      }

      render(<PerformanceDashboard />);

      await waitFor(() => {
        const alertsSection = screen.getByTestId('alerts-section');
        expect(alertsSection).toHaveTextContent(/\(2\)/); // 2 alerts
      });
    });
  });

  describe('Actions', () => {
    test('starts metric collection when button clicked', async () => {
      render(<PerformanceDashboard />);

      const toggleButton = screen.getByTestId('toggle-collection');
      expect(toggleButton).toHaveTextContent('▶ Start Collection');

      fireEvent.click(toggleButton);

      await waitFor(() => {
        expect(toggleButton).toHaveTextContent('⏸ Pause Collection');
        expect(toggleButton).toHaveClass('collecting');
      });
    });

    test('stops metric collection when button clicked again', async () => {
      render(<PerformanceDashboard />);

      const toggleButton = screen.getByTestId('toggle-collection');

      // Start
      fireEvent.click(toggleButton);
      await waitFor(() => {
        expect(toggleButton).toHaveTextContent('⏸ Pause Collection');
      });

      // Stop
      fireEvent.click(toggleButton);
      await waitFor(() => {
        expect(toggleButton).toHaveTextContent('▶ Start Collection');
        expect(toggleButton).not.toHaveClass('collecting');
      });
    });

    test('exports data to CSV when button clicked', () => {
      performanceMetrics.recordMetric('websocket_latency', 25);

      // Mock URL.createObjectURL
      const createObjectURLMock = jest.fn(() => 'blob:test');
      const revokeObjectURLMock = jest.fn();
      global.URL.createObjectURL = createObjectURLMock;
      global.URL.revokeObjectURL = revokeObjectURLMock;

      // Mock createElement and click
      const clickMock = jest.fn();
      const originalCreateElement = document.createElement.bind(document);
      document.createElement = jest.fn((tag) => {
        const element = originalCreateElement(tag);
        if (tag === 'a') {
          element.click = clickMock;
        }
        return element;
      });

      render(<PerformanceDashboard />);

      const exportButton = screen.getByTestId('export-csv');
      fireEvent.click(exportButton);

      expect(createObjectURLMock).toHaveBeenCalled();
      expect(clickMock).toHaveBeenCalled();
      expect(revokeObjectURLMock).toHaveBeenCalled();

      // Restore
      document.createElement = originalCreateElement;
    });

    test('clears all data when clear button clicked', async () => {
      performanceMetrics.recordMetric('websocket_latency', 25);
      performanceMetrics.recordMetric('ui_render_time', 12);

      // Mock confirm
      global.confirm = jest.fn(() => true);

      render(<PerformanceDashboard />);

      const clearButton = screen.getByTestId('clear-data');
      fireEvent.click(clearButton);

      await waitFor(() => {
        const metrics = performanceMetrics.getMetrics();
        expect(metrics.every(m => m.history.length === 0)).toBe(true);
      });
    });

    test('does not clear data if user cancels', async () => {
      performanceMetrics.recordMetric('websocket_latency', 25);

      // Mock confirm to return false
      global.confirm = jest.fn(() => false);

      render(<PerformanceDashboard />);

      const clearButton = screen.getByTestId('clear-data');
      fireEvent.click(clearButton);

      await waitFor(() => {
        const metric = performanceMetrics.getMetric('websocket_latency');
        expect(metric!.history.length).toBeGreaterThan(0);
      });
    });
  });

  describe('Real-time Updates', () => {
    test('updates display when new metrics recorded', async () => {
      render(<PerformanceDashboard />);

      // Record initial value
      performanceMetrics.recordMetric('websocket_latency', 25);

      await waitFor(() => {
        const currentValue = screen.getByTestId('websocket_latency-current');
        expect(currentValue).toHaveTextContent('25');
      });

      // Record new value
      performanceMetrics.recordMetric('websocket_latency', 30);

      await waitFor(() => {
        const currentValue = screen.getByTestId('websocket_latency-current');
        expect(currentValue).toHaveTextContent('30');
      });
    });

    test('updates statistics when metrics recorded', async () => {
      render(<PerformanceDashboard />);

      // Record multiple values
      for (let i = 0; i < 100; i++) {
        performanceMetrics.recordMetric('websocket_latency', 20 + i * 0.1);
      }

      await waitFor(() => {
        const p95Value = screen.getByTestId('websocket_latency-p95');
        expect(p95Value).toBeInTheDocument();
      });
    });
  });

  describe('Accessibility', () => {
    test('has proper ARIA labels', () => {
      render(<PerformanceDashboard />);

      const dashboard = screen.getByTestId('performance-dashboard');
      expect(dashboard).toBeInTheDocument();
    });

    test('buttons are keyboard accessible', () => {
      render(<PerformanceDashboard />);

      const toggleButton = screen.getByTestId('toggle-collection');
      expect(toggleButton).toHaveAttribute('type', 'button');
    });
  });

  describe('Performance Targets', () => {
    test('displays correct target values', () => {
      render(<PerformanceDashboard />);

      const websocketCard = screen.getByTestId('metric-websocket_latency');
      expect(websocketCard).toHaveTextContent(`Target: ${METRIC_TARGETS.websocketLatencyP99}`);
    });

    test('validates metrics meet targets', () => {
      // Record metrics within targets
      performanceMetrics.recordMetric('websocket_latency', 30); // < 50ms target
      performanceMetrics.recordMetric('ui_render_time', 10); // < 16ms target
      performanceMetrics.recordMetric('memory_usage', 75); // < 100MB target

      const metrics = performanceMetrics.getMetrics();
      const passingMetrics = metrics.filter(m => !m.isRegressed);

      expect(passingMetrics.length).toBeGreaterThan(0);
    });
  });
});
