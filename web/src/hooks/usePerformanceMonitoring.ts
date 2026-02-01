/**
 * Performance Monitoring Hook
 * Contract: Issue #80 - Performance Benchmarking Dashboard
 *
 * Integrates performance metrics collection with application lifecycle
 * and WebSocket V2 monitoring.
 */

import { useEffect, useRef, useCallback } from 'react';
import { performanceMetrics } from '../services/performanceMetrics';
import { UseWebSocketV2Return } from '../types/websocketV2';

/**
 * Hook options
 */
export interface UsePerformanceMonitoringOptions {
  /** Enable automatic metric collection */
  enabled?: boolean;

  /** WebSocket connection to monitor */
  websocket?: UseWebSocketV2Return;

  /** Interval for automatic collection (ms) */
  collectionInterval?: number;

  /** Component name for lifecycle tracking */
  componentName?: string;
}

/**
 * Performance monitoring hook
 */
export function usePerformanceMonitoring(options: UsePerformanceMonitoringOptions = {}) {
  const {
    enabled = true,
    websocket,
    collectionInterval = 5000,
    componentName,
  } = options;

  const mountTimeRef = useRef<number>(0);
  const renderCountRef = useRef<number>(0);

  /**
   * Track component mount time
   */
  useEffect(() => {
    if (!enabled || !componentName) return;

    const mountStart = performance.now();
    mountTimeRef.current = mountStart;

    return () => {
      // Track unmount time
      const unmountTime = performance.now() - mountStart;
      performanceMetrics.recordMetric(
        'component_mount_time',
        unmountTime,
        `${componentName}-unmount`
      );
    };
  }, [enabled, componentName]);

  /**
   * Track render count and time
   */
  useEffect(() => {
    if (!enabled) return;

    renderCountRef.current++;

    if (renderCountRef.current > 1 && mountTimeRef.current > 0) {
      const renderTime = performance.now() - mountTimeRef.current;
      performanceMetrics.recordMetric(
        'ui_render_time',
        renderTime,
        componentName ? `${componentName}-render-${renderCountRef.current}` : undefined
      );
    }
  });

  /**
   * Monitor WebSocket latency
   */
  useEffect(() => {
    if (!enabled || !websocket) return;

    // Track latency from stats
    const intervalId = setInterval(() => {
      if (websocket.stats.latency > 0) {
        performanceMetrics.recordMetric('websocket_latency', websocket.stats.latency);
      }
    }, collectionInterval);

    return () => clearInterval(intervalId);
  }, [enabled, websocket, collectionInterval]);

  /**
   * Start automatic collection
   */
  useEffect(() => {
    if (!enabled) return;

    performanceMetrics.startCollection(collectionInterval);

    return () => {
      performanceMetrics.stopCollection();
    };
  }, [enabled, collectionInterval]);

  /**
   * Measure async operation performance
   */
  const measureAsync = useCallback(
    async <T,>(
      metricId: string,
      operation: () => Promise<T>,
      label?: string
    ): Promise<T> => {
      const start = performance.now();
      try {
        const result = await operation();
        const duration = performance.now() - start;
        performanceMetrics.recordMetric(metricId, duration, label);
        return result;
      } catch (error) {
        const duration = performance.now() - start;
        performanceMetrics.recordMetric(metricId, duration, `${label}-error`);
        throw error;
      }
    },
    []
  );

  /**
   * Measure sync operation performance
   */
  const measureSync = useCallback(
    <T,>(metricId: string, operation: () => T, label?: string): T => {
      const start = performance.now();
      try {
        const result = operation();
        const duration = performance.now() - start;
        performanceMetrics.recordMetric(metricId, duration, label);
        return result;
      } catch (error) {
        const duration = performance.now() - start;
        performanceMetrics.recordMetric(metricId, duration, `${label}-error`);
        throw error;
      }
    },
    []
  );

  /**
   * Mark a performance milestone
   */
  const mark = useCallback((name: string) => {
    performance.mark(name);
  }, []);

  /**
   * Measure between two marks
   */
  const measure = useCallback((metricId: string, startMark: string, endMark: string) => {
    try {
      performance.measure(`${metricId}-measure`, startMark, endMark);
      const measure = performance.getEntriesByName(`${metricId}-measure`)[0];
      if (measure) {
        performanceMetrics.recordMetric(metricId, measure.duration, `${startMark}-${endMark}`);
      }
    } catch (error) {
      console.warn('Failed to measure performance:', error);
    }
  }, []);

  return {
    measureAsync,
    measureSync,
    mark,
    measure,
    renderCount: renderCountRef.current,
  };
}

/**
 * WebSocket-specific performance monitoring
 */
export function useWebSocketPerformance(websocket: UseWebSocketV2Return | null) {
  useEffect(() => {
    if (!websocket) return;

    const intervalId = setInterval(() => {
      // Record WebSocket latency
      if (websocket.stats.latency > 0) {
        performanceMetrics.recordMetric('websocket_latency', websocket.stats.latency);
      }

      // Calculate throughput
      const throughput =
        websocket.stats.messagesSent + websocket.stats.messagesReceived;
      if (throughput > 0) {
        performanceMetrics.recordMetric('processing_throughput', throughput);
      }
    }, 5000);

    return () => clearInterval(intervalId);
  }, [websocket]);
}

/**
 * Component lifecycle performance monitoring
 */
export function useComponentPerformance(componentName: string) {
  const mountTimeRef = useRef<number>(0);

  useEffect(() => {
    const mountStart = performance.now();
    mountTimeRef.current = mountStart;

    // Record mount time
    requestIdleCallback(() => {
      const mountDuration = performance.now() - mountStart;
      performanceMetrics.recordMetric(
        'component_mount_time',
        mountDuration,
        `${componentName}-mount`
      );
    });

    return () => {
      // Record unmount time
      const unmountStart = performance.now();
      requestIdleCallback(() => {
        const unmountDuration = performance.now() - unmountStart;
        performanceMetrics.recordMetric(
          'component_mount_time',
          unmountDuration,
          `${componentName}-unmount`
        );
      });
    };
  }, [componentName]);

  return mountTimeRef.current;
}

/**
 * Memory usage monitoring
 */
export function useMemoryMonitoring(intervalMs: number = 10000) {
  useEffect(() => {
    if (typeof (performance as any).memory === 'undefined') {
      console.warn('Memory API not available');
      return;
    }

    const intervalId = setInterval(() => {
      const memInfo = (performance as any).memory;
      const usedMemoryMB = memInfo.usedJSHeapSize / (1024 * 1024);
      performanceMetrics.recordMetric('memory_usage', usedMemoryMB);
    }, intervalMs);

    return () => clearInterval(intervalId);
  }, [intervalMs]);
}

/**
 * State synchronization performance monitoring
 */
export function useStateSyncPerformance() {
  const lastSyncRef = useRef<number>(0);

  const measureSync = useCallback((stateSize: number) => {
    const now = performance.now();

    if (lastSyncRef.current > 0) {
      const syncDuration = now - lastSyncRef.current;
      performanceMetrics.recordMetric(
        'state_sync_time',
        syncDuration,
        `size-${stateSize}`
      );
    }

    lastSyncRef.current = now;
  }, []);

  return { measureSync };
}
