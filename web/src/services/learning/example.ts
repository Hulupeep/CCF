/**
 * Self-Learning System - Usage Examples
 *
 * This file demonstrates how to use the learning system in different scenarios.
 */

import { LearningEngine } from './LearningEngine';
import { PatternStore } from './PatternStore';
import { Overseer } from './Overseer';

// ============================================================================
// Example 1: Basic Setup
// ============================================================================

export async function basicSetup() {
  console.log('=== Example 1: Basic Setup ===\n');

  // Create learning engine
  const engine = new LearningEngine({
    minObservations: 5,
    minSuccessRate: 0.7,
    minSimilarity: 0.8,
  });

  // Create pattern store
  const store = new PatternStore();

  // Create overseer
  const overseer = new Overseer(engine, store, {
    intervalMs: 3600000, // 1 hour
    enableAutoCrystallization: true,
    requireApproval: false,
  });

  // Start overseer
  overseer.start();
  console.log('✅ Learning system started');

  return { engine, store, overseer };
}

// ============================================================================
// Example 2: Observing User Interactions
// ============================================================================

export async function observeInteractions(engine: LearningEngine) {
  console.log('\n=== Example 2: Observing Interactions ===\n');

  // Observe a successful message interaction
  const obsId1 = engine.observeAction(
    'user123',
    'weather_query',
    {
      message: 'What is the weather today?',
      location: 'San Francisco',
    },
    true, // success
    120   // duration in ms
  );
  console.log(`Observation recorded: ${obsId1}`);

  // Observe multiple similar interactions
  for (let i = 0; i < 5; i++) {
    engine.observeAction(
      'user123',
      'weather_query',
      {
        message: `Weather in ${['NYC', 'LA', 'Chicago', 'Miami', 'Seattle'][i]}?`,
        location: ['NYC', 'LA', 'Chicago', 'Miami', 'Seattle'][i],
      },
      true,
      100 + i * 10
    );
  }
  console.log('✅ Observed 6 weather queries');

  // Use trackInteraction helper
  engine.trackInteraction(
    'user123',
    'Tell me a joke',
    'Why did the robot go to therapy? It had too many bugs!',
    true // user satisfied
  );
  console.log('✅ Tracked joke interaction');

  const stats = engine.getPerformanceStats();
  console.log(`\nTotal observations: ${stats.totalObservations}`);
}

// ============================================================================
// Example 3: Pattern Detection
// ============================================================================

export async function detectPatterns(engine: LearningEngine) {
  console.log('\n=== Example 3: Pattern Detection ===\n');

  // Detect patterns from observations
  const patterns = engine.detectPatterns();
  console.log(`Found ${patterns.length} pattern candidates`);

  // Examine each pattern
  patterns.forEach((pattern, i) => {
    console.log(`\nPattern ${i + 1}:`);
    console.log(`  Name: ${pattern.name}`);
    console.log(`  Confidence: ${(pattern.confidence * 100).toFixed(1)}%`);
    console.log(`  Usage Count: ${pattern.usageCount}`);
    console.log(`  Success Rate: ${(pattern.successRate * 100).toFixed(1)}%`);
    console.log(`  Keywords: ${pattern.trigger.keywords?.join(', ') || 'none'}`);
  });

  return patterns;
}

// ============================================================================
// Example 4: Crystallizing and Using Patterns
// ============================================================================

export async function crystallizeAndUsePatterns(
  engine: LearningEngine,
  store: PatternStore
) {
  console.log('\n=== Example 4: Crystallizing and Using Patterns ===\n');

  // Detect patterns
  const patterns = engine.detectPatterns();

  if (patterns.length === 0) {
    console.log('⚠️  No patterns ready for crystallization yet');
    return;
  }

  // Crystallize first pattern
  const pattern = patterns[0];
  console.log(`Crystallizing pattern: ${pattern.name}`);

  // Add to engine
  engine.addPattern(pattern);

  // Persist to storage
  await store.savePattern(pattern);
  console.log('✅ Pattern crystallized and saved');

  // Find matching pattern for new context
  const match = engine.findMatchingPattern({
    message: 'weather forecast today',
  });

  if (match) {
    console.log(`\n✨ Found matching pattern: ${match.name}`);

    // Execute pattern
    const result = await engine.executePattern(match, {
      message: 'weather forecast',
    });

    console.log(`Execution result:`);
    console.log(`  Success: ${result.success}`);
    console.log(`  Duration: ${result.duration}ms`);

    // Update confidence
    engine.updatePatternConfidence(match.id, result.success);
    console.log('✅ Pattern confidence updated');
  } else {
    console.log('❌ No matching pattern found');
  }
}

// ============================================================================
// Example 5: Running Overseer Learning Cycle
// ============================================================================

export async function runLearningCycle(overseer: Overseer) {
  console.log('\n=== Example 5: Running Learning Cycle ===\n');

  // Run a single learning cycle manually
  const report = await overseer.runLearningCycle();

  console.log('Learning Cycle Report:');
  console.log(`  Observations Analyzed: ${report.observationsAnalyzed}`);
  console.log(`  Patterns Detected: ${report.patternsDetected}`);
  console.log(`  Patterns Crystallized: ${report.actionsTriggered.crystallized.length}`);
  console.log(`  Patterns Pruned: ${report.actionsTriggered.pruned.length}`);
  console.log(`  Insights Generated: ${report.actionsTriggered.insights.length}`);
  console.log(`  Tokens Saved: ${report.performanceMetrics.tokensSaved}`);

  // Show recommendations
  if (report.recommendations.length > 0) {
    console.log('\nRecommendations:');
    report.recommendations.forEach((rec, i) => {
      console.log(`  ${i + 1}. [${rec.type}] ${rec.reason}`);
    });
  }

  return report;
}

// ============================================================================
// Example 6: Viewing Pattern Statistics
// ============================================================================

export async function viewStatistics(store: PatternStore) {
  console.log('\n=== Example 6: Pattern Statistics ===\n');

  const stats = await store.getPatternStats();

  console.log('Pattern Statistics:');
  console.log(`  Total Patterns: ${stats.totalPatterns}`);
  console.log(`  Active Patterns: ${stats.activePatterns}`);
  console.log(`  Total Uses: ${stats.totalUsages}`);
  console.log(`  Avg Success Rate: ${(stats.avgSuccessRate * 100).toFixed(1)}%`);
  console.log(`  Avg Confidence: ${(stats.avgConfidence * 100).toFixed(1)}%`);

  // Show top patterns
  if (stats.topPatterns.length > 0) {
    console.log('\nTop Patterns:');
    stats.topPatterns.forEach((pattern, i) => {
      console.log(`  ${i + 1}. ${pattern.name}`);
      console.log(`     Uses: ${pattern.usageCount}`);
      console.log(`     Success: ${(pattern.successRate * 100).toFixed(1)}%`);
    });
  }

  // Show recent activity
  if (stats.recentActivity.length > 0) {
    console.log('\nRecent Activity:');
    stats.recentActivity.slice(0, 5).forEach((activity) => {
      const date = new Date(activity.timestamp).toLocaleString();
      const status = activity.success ? '✅' : '❌';
      console.log(`  ${status} ${date} - Pattern: ${activity.patternId}`);
    });
  }
}

// ============================================================================
// Example 7: Export and Import Patterns
// ============================================================================

export async function exportImportPatterns(store: PatternStore) {
  console.log('\n=== Example 7: Export and Import ===\n');

  // Export patterns
  const json = await store.exportPatterns();
  console.log(`Exported ${json.length} bytes of pattern data`);

  // Save to localStorage (or file in Node.js)
  localStorage.setItem('mbot-patterns-backup', json);
  console.log('✅ Backup saved to localStorage');

  // Later: Import patterns
  const backup = localStorage.getItem('mbot-patterns-backup');
  if (backup) {
    const imported = await store.importPatterns(backup);
    console.log(`✅ Imported ${imported} patterns`);
  }
}

// ============================================================================
// Example 8: Pruning Stale Patterns
// ============================================================================

export async function pruneStalePatterns(engine: LearningEngine) {
  console.log('\n=== Example 8: Pruning Stale Patterns ===\n');

  // Prune patterns not used in 30 days
  const pruned = engine.pruneStalePatterns(30);

  console.log(`Pruned ${pruned.length} stale patterns`);
  if (pruned.length > 0) {
    console.log('Pruned pattern IDs:');
    pruned.forEach((id) => console.log(`  - ${id}`));
  }
}

// ============================================================================
// Example 9: Real-time Pattern Suggestion
// ============================================================================

export async function realtimePatternSuggestion(engine: LearningEngine) {
  console.log('\n=== Example 9: Real-time Pattern Suggestion ===\n');

  // User types a message
  const userMessage = 'What is the weather like today?';
  console.log(`User: ${userMessage}`);

  // Check for matching pattern
  const pattern = engine.findMatchingPattern({ message: userMessage });

  if (pattern) {
    console.log(`\n💡 Suggestion: Use learned pattern "${pattern.name}"`);
    console.log(`   Confidence: ${(pattern.confidence * 100).toFixed(1)}%`);
    console.log(`   Used ${pattern.usageCount} times before`);

    // Execute pattern
    const result = await engine.executePattern(pattern, { message: userMessage });

    if (result.success) {
      console.log(`\n✅ Pattern executed successfully in ${result.duration}ms`);
      console.log(`   Tokens saved: ~${Math.floor(userMessage.length / 4)}`);
    }
  } else {
    console.log('\nℹ️  No matching pattern found - will generate new response');
  }
}

// ============================================================================
// Example 10: Full Integration Demo
// ============================================================================

export async function fullIntegrationDemo() {
  console.log('\n' + '='.repeat(70));
  console.log('FULL INTEGRATION DEMO');
  console.log('='.repeat(70) + '\n');

  // Step 1: Setup
  const { engine, store, overseer } = await basicSetup();

  // Step 2: Simulate user interactions
  console.log('\n--- Step 2: Simulating User Interactions ---');
  for (let i = 0; i < 8; i++) {
    engine.observeAction(
      'user1',
      'weather_query',
      { message: `weather query ${i}`, location: 'NYC' },
      true,
      100
    );
  }
  console.log('✅ Simulated 8 weather queries');

  // Step 3: Run learning cycle
  console.log('\n--- Step 3: Running Learning Cycle ---');
  const report = await runLearningCycle(overseer);

  // Step 4: Use learned pattern
  console.log('\n--- Step 4: Using Learned Pattern ---');
  const pattern = engine.findMatchingPattern({ message: 'weather forecast' });
  if (pattern) {
    const result = await engine.executePattern(pattern, { message: 'weather' });
    console.log(`✅ Pattern executed: ${result.success}`);
  }

  // Step 5: View statistics
  console.log('\n--- Step 5: Viewing Statistics ---');
  await viewStatistics(store);

  // Cleanup
  overseer.stop();
  console.log('\n✅ Demo complete!');
}

// ============================================================================
// Run Examples
// ============================================================================

if (require.main === module) {
  // Run full demo
  fullIntegrationDemo().catch(console.error);
}
