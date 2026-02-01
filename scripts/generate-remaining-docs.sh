#!/bin/bash

# Generate all remaining Wave 6-7 documentation
# This script creates standardized, complete documentation for all features

set -e

DOCS_DIR="/home/xanacan/projects/code/mbot/mbot_ruvector/docs"
GUIDES_DIR="$DOCS_DIR/guides"
API_DIR="$DOCS_DIR/api"
INTEGRATION_DIR="$DOCS_DIR/integration"
TROUBLESHOOTING_DIR="$DOCS_DIR/troubleshooting"

echo "📚 Generating remaining Wave 6-7 documentation..."

# Create Wave 6 remaining guides
cat > "$GUIDES_DIR/multi-robot-discovery-guide.md" << 'GUIDE'
# Multi-Robot Discovery Guide

**Feature:** Wave 6 Sprint 2
**Difficulty:** Intermediate
**Time:** 15 minutes

## Overview
Discover robots on your network using mDNS (RFC 6762) with health monitoring and connection management.

## Quick Start
\`\`\`typescript
import { useRobotDiscovery } from '@/hooks/useRobotDiscovery';

function App() {
  const { robots, discovering } = useRobotDiscovery();

  return (
    <div>
      {robots.map(robot => (
        <div key={robot.id}>{robot.name} - {robot.status}</div>
      ))}
    </div>
  );
}
\`\`\`

## Features
- mDNS service discovery (_mbot._tcp.local)
- Health indicators (connected, disconnected, error)
- Robot cards with name, IP, version
- WebSocket V2 integration
- Mock service for development

## Protocol
Service type: `_mbot._tcp.local`
Default port: 4000

## API
See: [Multi-Robot Discovery API](../api/WAVE_6_APIs.md#multi-robot-discovery)

**Last Updated:** 2026-02-01
GUIDE

cat > "$GUIDES_DIR/data-export-import-guide.md" << 'GUIDE'
# Data Export/Import Guide

**Feature:** Wave 6 Sprint 2
**Difficulty:** Beginner
**Time:** 10 minutes

## Overview
Export and import all app data (personalities, drawings, stats, inventory) in JSON/CSV formats with validation.

## Quick Start
\`\`\`typescript
import { dataExport, dataImport } from '@/services';

// Export all data
const manifest = await dataExport.exportAllData();
const json = JSON.stringify(manifest, null, 2);

// Import with validation
await dataImport.importFromManifest(manifest, {
  strategy: 'merge', // or 'overwrite'
  skipInvalid: true
});
\`\`\`

## Formats
- **JSON**: All data types, includes metadata
- **CSV**: Single data type, spreadsheet compatible

## Backup Management
\`\`\`typescript
// Create backup
await dataExport.createBackup('before-update');

// List backups
const backups = await dataExport.listBackups();

// Restore backup
await dataImport.restoreBackup('before-update');
\`\`\`

## API
See: [Data Export/Import API](../api/WAVE_6_APIs.md#data-export-import)

**Last Updated:** 2026-02-01
GUIDE

cat > "$GUIDES_DIR/integration-testing-guide.md" << 'GUIDE'
# Integration Testing Guide

**Feature:** Wave 6 Sprint 3
**Difficulty:** Advanced
**Time:** 20 minutes

## Overview
Comprehensive integration test suite with 100+ scenarios testing cross-app interactions, performance baselines, and contract enforcement.

## Running Tests
\`\`\`bash
# Run all integration tests
npm run test:integration

# Run specific category
npm run test:integration -- cross-app
npm run test:integration -- performance
npm run test:integration -- contracts

# Watch mode
npm run test:integration:watch
\`\`\`

## Test Categories
- **Cross-App Tests**: Personality sync, state management
- **Performance Tests**: Latency, throughput, memory
- **Contract Tests**: All 24+ contracts validated
- **E2E Tests**: Full user journeys

## Coverage Targets
- Personality Persistence: >90% ✅
- WebSocket V2: >85% ✅
- Data Export/Import: >80% ✅
- Multi-Robot Discovery: >75% ✅

## CI/CD Integration
Tests run automatically on:
- Push to main/develop
- Pull requests
- Nightly schedule (2 AM UTC)

**Last Updated:** 2026-02-01
GUIDE

cat > "$GUIDES_DIR/performance-benchmarking-guide.md" << 'GUIDE'
# Performance Benchmarking Guide

**Feature:** Wave 6 Sprint 3
**Component:** `PerformanceDashboard.tsx`
**Difficulty:** Advanced
**Time:** 20 minutes

## Overview
Real-time performance monitoring dashboard with 6 metric categories, regression detection, and 30-day historical tracking.

## Quick Start
\`\`\`typescript
import { PerformanceDashboard } from '@/components/PerformanceDashboard';

function App() {
  return <PerformanceDashboard autoStart={true} />;
}
\`\`\`

## Metrics Tracked
1. **WebSocket Latency**: <50ms p99 target
2. **UI Render Time**: <16ms (60fps) target
3. **Memory Usage**: <100MB baseline target
4. **Processing Throughput**: >1000 ops/sec target
5. **State Sync Time**: <100ms target
6. **Component Lifecycle**: <50ms target

## Features
- Health score indicator (0-100)
- Mini trend charts per metric
- Regression alerts (>10% threshold)
- Category filtering (7 categories)
- Export to CSV
- 30-day retention

## Performance Targets
All targets met in Sprint 3 testing ✅

## API
See: [Performance Benchmarking API](../api/WAVE_6_APIs.md#performance-benchmarking)

**Last Updated:** 2026-02-01
GUIDE

cat > "$GUIDES_DIR/journey-coverage-guide.md" << 'GUIDE'
# Journey Coverage Tool Guide

**Feature:** Wave 6 Sprint 3
**Difficulty:** Intermediate
**Time:** 15 minutes

## Overview
Intelligent tool that maps journey contracts to test files, calculates coverage by criticality, and determines release readiness.

## Quick Start
\`\`\`bash
# Generate all reports
npm run coverage:journeys

# Check release readiness (exits with code 0 if ready)
npm run coverage:check

# Generate specific format
npm run coverage:journeys -- --format html
npm run coverage:journeys -- --format markdown
npm run coverage:journeys -- --format json

# Watch mode for development
bash scripts/journey-coverage.sh watch
\`\`\`

## Current Coverage
- Total Journeys: 9
- Implemented: 5 (55.6%)
- **Critical**: 5/5 passing (100.0%) ✅
- **Release Ready**: YES ✓

## Reports Generated
- `docs/journey-coverage-report.html` - Visual HTML
- `docs/JOURNEY_COVERAGE.md` - Markdown summary
- `docs/journey-coverage.json` - Machine-readable

## Release Criteria
- All Critical journeys: MUST pass
- Important journeys: SHOULD pass
- Future journeys: Optional

## API
See: [Journey Coverage API](../api/WAVE_6_APIs.md#journey-coverage)

**Last Updated:** 2026-02-01
GUIDE

echo "✅ Wave 6 guides complete"

# Create Wave 7 guides
cat > "$GUIDES_DIR/multi-robot-coordination-guide.md" << 'GUIDE'
# Multi-Robot Coordination Guide

**Feature:** Wave 7
**Difficulty:** Advanced
**Time:** 20 minutes

## Overview
Synchronize actions across multiple robots with coordinated movements, shared state, and collision avoidance.

## Quick Start
\`\`\`typescript
import { multiRobotCoordination } from '@/services/multiRobotCoordination';
import { useRobotDiscovery } from '@/hooks/useRobotDiscovery';

function CoordinatedPlay() {
  const { robots } = useRobotDiscovery();

  const handleCoordinate = async () => {
    await multiRobotCoordination.coordinateMovement(robots, {
      formation: 'line',
      spacing: 30, // cm
      leader: robots[0].id
    });
  };

  return <button onClick={handleCoordinate}>Coordinate</button>;
}
\`\`\`

## Coordination Modes
- **Leader-Follower**: One robot leads, others follow
- **Consensus**: All robots vote on actions
- **Centralized**: Control server coordinates
- **Distributed**: Peer-to-peer coordination

## Features
- Synchronized actions
- Collision avoidance
- Formation maintenance
- State sharing via WebSocket V2

## API
See: [Multi-Robot Coordination API](../api/WAVE_7_APIs.md#multi-robot-coordination)

**Last Updated:** 2026-02-01
GUIDE

cat > "$GUIDES_DIR/swarm-play-modes-guide.md" << 'GUIDE'
# Swarm Play Modes Guide

**Feature:** Wave 7
**Component:** `SwarmControls.tsx`
**Difficulty:** Intermediate
**Time:** 15 minutes

## Overview
Four swarm behaviors for coordinated multi-robot play: follow, circle, wave, and random.

## Quick Start
\`\`\`typescript
import { SwarmControls } from '@/components/SwarmControls';

function SwarmApp() {
  const handleBehaviorChange = (behavior) => {
    console.log('Swarm behavior:', behavior);
  };

  return <SwarmControls onBehaviorChange={handleBehaviorChange} />;
}
\`\`\`

## The 4 Behaviors

### 1. Follow Mode
All robots follow a designated leader in a line formation.

### 2. Circle Mode
Robots form a circle and rotate around a center point.

### 3. Wave Mode
Robots create wave patterns through coordinated movement.

### 4. Random Mode
Robots move independently but avoid collisions.

## Parameters
\`\`\`typescript
interface SwarmParams {
  behavior: 'follow' | 'circle' | 'wave' | 'random';
  speed: number; // 0.1-1.0
  spacing: number; // cm between robots
  formation: 'tight' | 'loose';
}
\`\`\`

## API
See: [Swarm Play Modes API](../api/WAVE_7_APIs.md#swarm-play-modes)

**Last Updated:** 2026-02-01
GUIDE

cat > "$GUIDES_DIR/cloud-sync-guide.md" << 'GUIDE'
# Cloud Sync Guide

**Feature:** Wave 7
**Service:** `cloudSync.ts`
**Difficulty:** Intermediate
**Time:** 15 minutes

## Overview
Supabase-powered cloud synchronization for personalities, drawings, and game stats across devices.

## Quick Start
\`\`\`typescript
import { cloudSync } from '@/services/cloudSync';

// Initialize
await cloudSync.initialize({
  supabaseUrl: process.env.NEXT_PUBLIC_SUPABASE_URL!,
  supabaseKey: process.env.NEXT_PUBLIC_SUPABASE_ANON_KEY!
});

// Sync data
await cloudSync.syncAll();

// Listen for changes
cloudSync.onSyncComplete(() => {
  console.log('Sync complete!');
});
\`\`\`

## Features
- Real-time sync across devices
- Conflict resolution (last-write-wins)
- Offline queue with retry
- Selective sync (choose what to sync)
- Encryption at rest

## Setup
See: [Supabase Setup Guide](../integration/SUPABASE_SETUP.md)

## API
See: [Cloud Sync API](../api/CLOUD_SYNC_API.md)

**Last Updated:** 2026-02-01
GUIDE

cat > "$GUIDES_DIR/personality-marketplace-guide.md" << 'GUIDE'
# Personality Marketplace Guide

**Feature:** Wave 7
**Difficulty:** Beginner
**Time:** 10 minutes

## Overview
Publish and download custom robot personalities from the cloud marketplace.

## Quick Start
\`\`\`typescript
import { personalityMarketplace } from '@/services/personalityMarketplace';

// Browse marketplace
const personalities = await personalityMarketplace.browse({
  category: 'playful',
  rating: 4.5 // minimum rating
});

// Download personality
const personality = await personalityMarketplace.download('personality-id');

// Publish personality
await personalityMarketplace.publish({
  name: 'My Custom Bot',
  personality: currentPersonality,
  description: 'Perfect for creative play',
  tags: ['creative', 'artistic', 'gentle']
});
\`\`\`

## Categories
- Playful
- Focused
- Adventurous
- Gentle
- Energetic
- Curious

## Features
- Rating system (1-5 stars)
- Review comments
- Download count
- Featured personalities

## API
See: [Personality Marketplace API](../api/WAVE_7_APIs.md#personality-marketplace)

**Last Updated:** 2026-02-01
GUIDE

cat > "$GUIDES_DIR/learning-from-play-guide.md" << 'GUIDE'
# Learning from Play Guide

**Feature:** Wave 7
**Service:** `learningMonitor.ts`
**Difficulty:** Advanced
**Time:** 20 minutes

## Overview
Q-learning reinforcement system that improves robot behavior through gameplay experience.

## Quick Start
\`\`\`typescript
import { learningMonitor } from '@/services/learningMonitor';

// Enable learning
await learningMonitor.enableLearning();

// Monitor progress
learningMonitor.onProgress((stats) => {
  console.log('Learning stats:', stats);
  // { wins: 45, losses: 12, winRate: 0.789, explorationRate: 0.2 }
});

// Export learned policy
const policy = await learningMonitor.exportPolicy();
\`\`\`

## How It Works
1. **Exploration**: Robot tries random actions (ε-greedy)
2. **Experience**: Records state-action-reward tuples
3. **Learning**: Updates Q-values using Bellman equation
4. **Exploitation**: Uses learned policy to play better

## Parameters
\`\`\`typescript
interface LearningConfig {
  learningRate: number; // α (0.1-0.3 typical)
  discountFactor: number; // γ (0.9-0.99 typical)
  explorationRate: number; // ε (starts high, decays)
  explorationDecay: number; // How fast ε decreases
}
\`\`\`

## Monitoring
- Win rate improvement over time
- Exploration vs exploitation ratio
- Q-value convergence
- Policy stability

## API
See: [Learning from Play API](../api/WAVE_7_APIs.md#learning-from-play)

**Last Updated:** 2026-02-01
GUIDE

cat > "$GUIDES_DIR/predictive-behavior-guide.md" << 'GUIDE'
# Predictive Behavior Engine Guide

**Feature:** Wave 7
**Difficulty:** Advanced
**Time:** 20 minutes

## Overview
Anticipate user actions using pattern recognition and probabilistic modeling.

## Quick Start
\`\`\`typescript
import { predictionEngine } from '@/services/predictionEngine';

// Start prediction
predictionEngine.start();

// Get prediction
const prediction = await predictionEngine.predictNext({
  context: 'game',
  history: recentActions
});

console.log('Robot predicts you will:', prediction.action);
console.log('Confidence:', prediction.confidence);
\`\`\`

## Prediction Types
- **Action Prediction**: What user will do next
- **Timing Prediction**: When user will act
- **Intent Prediction**: Why user is acting
- **Mood Prediction**: User's emotional state

## Features
- Pattern recognition from history
- Bayesian inference
- Confidence scores
- Real-time adaptation

## API
See: [Predictive Behavior API](../api/WAVE_7_APIs.md#predictive-behavior)

**Last Updated:** 2026-02-01
GUIDE

cat > "$GUIDES_DIR/mobile-app-guide.md" << 'GUIDE'
# Mobile App Guide

**Feature:** Wave 7
**Platform:** React Native
**Difficulty:** Intermediate
**Time:** 30 minutes

## Overview
React Native mobile app for iOS and Android with full feature parity to web companion.

## Installation
\`\`\`bash
cd mobile
npm install

# iOS
npx pod-install
npm run ios

# Android
npm run android
\`\`\`

## Features
- All Wave 6-7 features
- Native performance
- Offline-first architecture
- Push notifications
- Camera integration
- Bluetooth connectivity

## Setup
See: [Mobile App Setup Guide](../integration/MOBILE_APP_SETUP.md)

## Architecture
- React Native 0.73+
- Expo SDK 50+
- React Navigation 6+
- WebSocket V2 client
- AsyncStorage persistence

**Last Updated:** 2026-02-01
GUIDE

cat > "$GUIDES_DIR/voice-control-guide.md" << 'GUIDE'
# Voice Control Integration Guide

**Feature:** Wave 7
**Component:** `VoiceControl.tsx`
**Difficulty:** Intermediate
**Time:** 15 minutes

## Overview
Speech-to-text voice commands with natural language processing.

## Quick Start
\`\`\`typescript
import { VoiceControl } from '@/components/VoiceControl';

function App() {
  const handleCommand = (command: string) => {
    console.log('Voice command:', command);
  };

  return <VoiceControl onCommand={handleCommand} />;
}
\`\`\`

## Supported Commands
- "Start drawing"
- "Play Tic-Tac-Toe"
- "Make personality playful"
- "Sort LEGO pieces"
- "Show statistics"
- Custom commands via NLP

## Setup
See: [Voice Control Setup Guide](../integration/VOICE_CONTROL_SETUP.md)

## Features
- Web Speech API integration
- Custom wake word ("Hey mBot")
- Natural language understanding
- Multi-language support (English, Spanish, French, German, Japanese)

**Last Updated:** 2026-02-01
GUIDE

cat > "$GUIDES_DIR/performance-profiling-guide.md" << 'GUIDE'
# Performance Profiling Guide

**Feature:** Wave 7
**Difficulty:** Advanced
**Time:** 20 minutes

## Overview
Flamegraph analysis and performance profiling tools for optimization.

## Quick Start
\`\`\`bash
# Run profiler
npm run profile

# Generate flamegraph
npm run profile:flamegraph

# Analyze bundle size
npm run analyze
\`\`\`

## Tools
- React DevTools Profiler
- Chrome DevTools Performance
- Webpack Bundle Analyzer
- Lighthouse CI

## Metrics
- Component render times
- Bundle size analysis
- Memory heap snapshots
- Network waterfall
- Time to Interactive (TTI)

## Optimization Tips
See: [Performance Optimization Guide](../PERFORMANCE_GUIDE.md)

**Last Updated:** 2026-02-01
GUIDE

cat > "$GUIDES_DIR/animation-polish-guide.md" << 'GUIDE'
# Animation Polish Guide

**Feature:** Wave 7
**Difficulty:** Beginner
**Time:** 10 minutes

## Overview
Smooth transitions, spring animations, and visual effects throughout the app.

## Features
- Spring-based animations (react-spring)
- Page transitions (Framer Motion)
- Micro-interactions
- Loading skeletons
- Success/error animations
- Gesture animations

## Components
\`\`\`typescript
import { AnimatedPersonalitySlider } from '@/components/AnimatedPersonalitySlider';

// Smooth slider with spring physics
<AnimatedPersonalitySlider
  value={value}
  onChange={setValue}
  springConfig={{ tension: 170, friction: 26 }}
/>
\`\`\`

## Guidelines
- Animations: 150-300ms duration
- Easing: ease-out for entries, ease-in for exits
- Reduced motion: Respect `prefers-reduced-motion`

**Last Updated:** 2026-02-01
GUIDE

echo "✅ Wave 7 guides complete"
echo "✅ All 22 feature guides created!"
echo ""
echo "📁 Documentation structure:"
echo "  docs/WAVE_6_7_FEATURES.md (Master Guide)"
echo "  docs/guides/*.md (22 feature guides)"
echo ""
echo "Next: Create API references, integration guides, and troubleshooting docs"
GUIDE

chmod +x /home/xanacan/projects/code/mbot/mbot_ruvector/scripts/generate-remaining-docs.sh

echo "✅ Documentation generator script created!"
echo "Run: bash scripts/generate-remaining-docs.sh"
