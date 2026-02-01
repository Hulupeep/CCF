# Performance Benchmarking Dashboard

**Contract**: Issue #80 - Performance Benchmarking Dashboard (IMPORTANT)

## Overview

The Performance Benchmarking Dashboard provides comprehensive monitoring and regression detection for mBot RuVector's real-time performance metrics.

## Features

### 1. Real-time Metrics Collection
- **WebSocket Message Latency**: Tracks round-trip time for WebSocket messages
- **UI Render Time**: Measures component render performance
- **Memory Usage**: Monitors JavaScript heap usage
- **Data Processing Throughput**: Tracks data transformation operations per second
- **State Synchronization Time**: Measures full state sync duration
- **Component Lifecycle**: Tracks mount/unmount times

### 2. Historical Trend Charts
- 30-day rolling history for all metrics
- Mini line charts showing trends over time
- Target baseline visualization

### 3. Regression Detection
- Automatic detection of >10% performance degradation
- Alert system with severity levels (warning/critical)
- Alert acknowledgment workflow
- Historical regression tracking

### 4. CSV Export
- Export all metrics and historical data
- Include/exclude options for history and alerts
- Timestamp-based file naming
- Full system information in export

### 5. Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| WebSocket Latency (p99) | < 50ms | Round-trip message time |
| UI Render Time | < 16ms | 60fps target |
| Memory Baseline | < 100MB | JS heap usage |
| Processing Throughput | > 1000 ops/sec | Data operations |
| State Sync Time | < 100ms | Full state snapshot |
| Component Lifecycle | < 50ms | Mount/unmount time |

## Usage

### Basic Setup

```typescript
import { PerformanceDashboard } from './components/PerformanceDashboard';

function App() {
  return <PerformanceDashboard />;
}
```

### With Performance Monitoring Hook

```typescript
import { usePerformanceMonitoring } from './hooks/usePerformanceMonitoring';
import { useWebSocketV2 } from './hooks/useWebSocketV2';

function MyComponent() {
  const websocket = useWebSocketV2({ url: 'ws://localhost:8080' });

  const { measureAsync, measureSync } = usePerformanceMonitoring({
    enabled: true,
    websocket,
    componentName: 'MyComponent',
  });

  // Measure async operations
  const fetchData = async () => {
    return await measureAsync('data_fetch', async () => {
      const response = await fetch('/api/data');
      return response.json();
    }, 'api-data');
  };

  // Measure sync operations
  const processData = (data: any) => {
    return measureSync('data_processing', () => {
      return data.map(item => ({ ...item, processed: true }));
    }, 'transform');
  };

  return <div>...</div>;
}
```

### WebSocket Performance Monitoring

```typescript
import { useWebSocketPerformance } from './hooks/usePerformanceMonitoring';

function WebSocketComponent() {
  const websocket = useWebSocketV2();
  useWebSocketPerformance(websocket);

  // WebSocket latency is automatically tracked
  return <div>...</div>;
}
```

### Memory Monitoring

```typescript
import { useMemoryMonitoring } from './hooks/usePerformanceMonitoring';

function App() {
  // Monitor memory every 10 seconds
  useMemoryMonitoring(10000);

  return <div>...</div>;
}
```

## API Reference

### PerformanceMetricsService

```typescript
import { performanceMetrics } from './services/performanceMetrics';

// Record a metric
performanceMetrics.recordMetric('websocket_latency', 25, 'optional-label');

// Get all metrics
const metrics = performanceMetrics.getMetrics();

// Get metrics by category
const websocketMetrics = performanceMetrics.getMetricsByCategory('websocket');

// Get active alerts
const alerts = performanceMetrics.getActiveAlerts();

// Acknowledge an alert
performanceMetrics.acknowledgeAlert('alert-id');

// Export to CSV
const csv = performanceMetrics.exportToCSV(true);

// Start/stop automatic collection
performanceMetrics.startCollection(5000); // 5 second interval
performanceMetrics.stopCollection();

// Subscribe to updates
const unsubscribe = performanceMetrics.subscribe(() => {
  console.log('Metrics updated');
});
```

### usePerformanceMonitoring Hook

```typescript
const {
  measureAsync,  // Measure async operations
  measureSync,   // Measure sync operations
  mark,          // Create performance mark
  measure,       // Measure between marks
  renderCount,   // Current render count
} = usePerformanceMonitoring({
  enabled: true,
  websocket: websocketInstance,
  collectionInterval: 5000,
  componentName: 'MyComponent',
});
```

## Dashboard Controls

### Collection Toggle
- **Start**: Begin automatic metric collection (5-second intervals)
- **Pause**: Stop automatic collection

### Category Filter
- **All**: Show all metrics
- **WebSocket**: WebSocket-specific metrics
- **UI**: User interface performance
- **Memory**: Memory usage metrics
- **Processing**: Data processing metrics
- **Sync**: State synchronization metrics
- **Components**: Component lifecycle metrics

### Export Button
- Exports all metrics and historical data to CSV
- Includes system information
- Timestamp-based filename

### Clear Data Button
- Removes all metrics and historical data
- Requires confirmation
- Cannot be undone

## Regression Detection

### Algorithm
1. Calculate p95 percentile for current metrics
2. Compare against target baseline
3. If p95 > baseline * 1.10 (>10% regression), trigger alert
4. Create alert with severity:
   - **Warning**: 10-25% regression
   - **Critical**: >25% regression

### Alert Workflow
1. Alert appears in dashboard header
2. Shows baseline vs. current value
3. Displays regression percentage
4. User can acknowledge alert
5. Acknowledged alerts are hidden but stored

## Testing

### Running Benchmarks

```bash
# Run all performance benchmarks
npm run test:benchmarks

# Run specific benchmark suite
npm run test:benchmarks -- --grep "WebSocket"
```

### Running Component Tests

```bash
# Run dashboard tests
npm test -- PerformanceDashboard.test

# Run with coverage
npm test -- --coverage PerformanceDashboard.test
```

## Benchmark Results

Example output from performance benchmarks:

```
WebSocket Latency:
  Mean: 8.45ms
  P50:  7.23ms
  P95:  12.87ms
  P99:  15.42ms ✓ < 50ms target
  Min:  5.11ms
  Max:  18.93ms

UI Render Time:
  Mean: 9.32ms
  P50:  8.67ms
  P95:  13.21ms
  P99:  14.89ms ✓ < 16ms target
  Min:  6.45ms
  Max:  16.78ms

Memory Usage: 75.42MB ✓ < 100MB target
```

## Integration with Existing Features

### WebSocket V2 Integration
- Automatic latency tracking via ping/pong
- Message count and throughput monitoring
- Connection state performance

### Component Integration
- Mount/unmount time tracking
- Render count and timing
- Lifecycle performance metrics

### State Management Integration
- State sync time measurement
- Incremental update performance
- Full snapshot performance

## Troubleshooting

### Memory API Not Available
Some browsers don't expose `performance.memory`. Memory metrics will be skipped in these browsers.

### High Memory Usage
1. Check for memory leaks in components
2. Review data caching strategies
3. Analyze component lifecycle

### High Latency
1. Check network conditions
2. Review WebSocket batching configuration
3. Analyze server response times

### Regressions Not Detected
1. Ensure sufficient data points (>100 samples recommended)
2. Check regression threshold configuration
3. Verify target baselines are realistic

## File Locations

```
web/src/
├── components/
│   ├── PerformanceDashboard.tsx           # Main dashboard component
│   └── __tests__/
│       └── PerformanceDashboard.test.tsx  # Component tests
├── services/
│   └── performanceMetrics.ts              # Metrics service
├── hooks/
│   └── usePerformanceMonitoring.ts        # Performance monitoring hooks
└── types/
    └── performance.ts                      # Type definitions

tests/
└── benchmarks/
    └── performance.bench.ts                # Benchmark suite
```

## Contract Compliance

**Issue #80 Requirements**:
- ✅ Performance metrics collection system
- ✅ Dashboard UI showing real-time metrics
- ✅ Historical trend charts (30 days)
- ✅ Regression detection (alert if >10% degradation)
- ✅ Export to CSV functionality
- ✅ Automated benchmarking integration

**DOD**: IMPORTANT - should pass before release

All performance targets are validated through automated benchmark tests.
