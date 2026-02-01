/**
 * Learning Engine Integration Tests
 * Tests the full learning workflow: observe → detect → crystallize → execute
 */

import { describe, it, expect, beforeEach } from '@jest/globals';
import { LearningEngine } from '../../web/src/services/learning/LearningEngine';
import { PatternStore } from '../../web/src/services/learning/PatternStore';
import { Overseer } from '../../web/src/services/learning/Overseer';

describe('LearningEngine', () => {
  let engine: LearningEngine;

  beforeEach(() => {
    engine = new LearningEngine();
  });

  describe('Observation', () => {
    it('should observe user actions', () => {
      const obsId = engine.observeAction(
        'user1',
        'send_message',
        { message: 'Hello bot' },
        true,
        100
      );

      expect(obsId).toBeTruthy();
      expect(obsId).toContain('obs-user1-send_message');
    });

    it('should track interactions with satisfaction', () => {
      const obsId = engine.trackInteraction(
        'user1',
        'What is the weather?',
        'The weather is sunny!',
        true
      );

      expect(obsId).toBeTruthy();
      expect(obsId).toContain('user1');
    });
  });

  describe('Pattern Detection', () => {
    it('should not detect patterns with too few observations', () => {
      // Add only 3 observations (below threshold of 5)
      for (let i = 0; i < 3; i++) {
        engine.observeAction(
          'user1',
          'weather_query',
          { query: 'weather' },
          true,
          100
        );
      }

      const patterns = engine.detectPatterns();
      expect(patterns.length).toBe(0);
    });

    it('should detect patterns after 5+ successful uses', () => {
      // Add 6 observations with same pattern
      for (let i = 0; i < 6; i++) {
        engine.observeAction(
          'user1',
          'weather_query',
          { query: 'weather', location: 'NYC' },
          true,
          100
        );
      }

      const patterns = engine.detectPatterns();
      expect(patterns.length).toBeGreaterThan(0);
      expect(patterns[0].usageCount).toBeGreaterThanOrEqual(5);
    });

    it('should require 70%+ success rate for crystallization', () => {
      // Add 10 observations: 6 success, 4 failure = 60% success
      for (let i = 0; i < 6; i++) {
        engine.observeAction('user1', 'test_action', { test: true }, true, 100);
      }
      for (let i = 0; i < 4; i++) {
        engine.observeAction('user1', 'test_action', { test: true }, false, 100);
      }

      const patterns = engine.detectPatterns();
      // Should not crystallize due to low success rate
      expect(patterns.length).toBe(0);
    });

    it('should crystallize patterns with 70%+ success rate', () => {
      // Add 10 observations: 8 success, 2 failure = 80% success
      for (let i = 0; i < 8; i++) {
        engine.observeAction('user1', 'good_action', { test: true }, true, 100);
      }
      for (let i = 0; i < 2; i++) {
        engine.observeAction('user1', 'good_action', { test: true }, false, 100);
      }

      const patterns = engine.detectPatterns();
      expect(patterns.length).toBeGreaterThan(0);
      expect(patterns[0].successRate).toBeGreaterThanOrEqual(0.7);
    });
  });

  describe('Pattern Matching', () => {
    beforeEach(() => {
      // Create and add a test pattern
      for (let i = 0; i < 6; i++) {
        engine.observeAction(
          'user1',
          'weather_query',
          { message: 'what is the weather today' },
          true,
          100
        );
      }

      const patterns = engine.detectPatterns();
      if (patterns.length > 0) {
        engine.addPattern(patterns[0]);
      }
    });

    it('should find matching pattern for similar context', () => {
      const pattern = engine.findMatchingPattern({
        message: 'weather forecast today',
      });

      expect(pattern).toBeTruthy();
    });

    it('should not match dissimilar context', () => {
      const pattern = engine.findMatchingPattern({
        message: 'completely different topic',
      });

      expect(pattern).toBeNull();
    });
  });

  describe('Pattern Execution', () => {
    it('should execute crystallized patterns', async () => {
      // Create pattern
      for (let i = 0; i < 6; i++) {
        engine.observeAction(
          'user1',
          'test_workflow',
          { action: 'test' },
          true,
          100
        );
      }

      const patterns = engine.detectPatterns();
      expect(patterns.length).toBeGreaterThan(0);

      const pattern = patterns[0];
      engine.addPattern(pattern);

      const result = await engine.executePattern(pattern, { action: 'test' });

      expect(result.success).toBe(true);
      expect(result.patternId).toBe(pattern.id);
    });

    it('should update pattern confidence based on execution outcome', async () => {
      // Create and add pattern
      for (let i = 0; i < 6; i++) {
        engine.observeAction('user1', 'test', {}, true, 100);
      }

      const patterns = engine.detectPatterns();
      const pattern = patterns[0];
      engine.addPattern(pattern);

      const initialConfidence = pattern.confidence;

      // Execute successfully
      await engine.executePattern(pattern, {});
      engine.updatePatternConfidence(pattern.id, true);

      expect(pattern.confidence).toBeGreaterThanOrEqual(initialConfidence);

      // Execute unsuccessfully
      engine.updatePatternConfidence(pattern.id, false);
      expect(pattern.confidence).toBeLessThan(initialConfidence);
    });
  });

  describe('Pattern Pruning', () => {
    it('should prune stale patterns', () => {
      // Create old pattern
      for (let i = 0; i < 6; i++) {
        engine.observeAction('user1', 'old_action', {}, true, 100);
      }

      const patterns = engine.detectPatterns();
      const pattern = patterns[0];

      // Make pattern old
      pattern.lastUsed = Date.now() - 35 * 24 * 60 * 60 * 1000; // 35 days ago
      engine.addPattern(pattern);

      const pruned = engine.pruneStalePatterns(30);

      expect(pruned.length).toBe(1);
      expect(pruned[0]).toBe(pattern.id);
    });

    it('should not prune active patterns', () => {
      // Create recent pattern
      for (let i = 0; i < 6; i++) {
        engine.observeAction('user1', 'active_action', {}, true, 100);
      }

      const patterns = engine.detectPatterns();
      const pattern = patterns[0];
      pattern.lastUsed = Date.now(); // Just now
      engine.addPattern(pattern);

      const pruned = engine.pruneStalePatterns(30);

      expect(pruned.length).toBe(0);
    });
  });

  describe('Performance Tracking', () => {
    it('should track execution count and tokens saved', async () => {
      // Create pattern
      for (let i = 0; i < 6; i++) {
        engine.observeAction('user1', 'test', { msg: 'test' }, true, 100);
      }

      const patterns = engine.detectPatterns();
      const pattern = patterns[0];
      engine.addPattern(pattern);

      // Execute pattern multiple times
      await engine.executePattern(pattern, {});
      await engine.executePattern(pattern, {});
      await engine.executePattern(pattern, {});

      const stats = engine.getPerformanceStats();

      expect(stats.executionCount).toBe(3);
      expect(stats.tokensSaved).toBeGreaterThan(0);
    });
  });
});

describe('PatternStore', () => {
  let store: PatternStore;

  beforeEach(async () => {
    store = new PatternStore();
    // Clear all patterns before each test
    await store.clearAll();
  });

  it('should save and load patterns', async () => {
    const pattern = {
      id: 'test-pattern-1',
      name: 'Test Pattern',
      description: 'A test pattern',
      trigger: { type: 'keyword' as const, pattern: 'test' },
      action: {
        type: 'response' as const,
        description: 'Test action',
        responseTemplate: 'Test response',
      },
      confidence: 0.85,
      usageCount: 10,
      successRate: 0.9,
      avgDuration: 100,
      createdAt: Date.now(),
      lastUsed: Date.now(),
      sourceObservations: [],
      crystallizedAt: Date.now(),
      version: 1,
    };

    await store.savePattern(pattern);
    const loaded = await store.loadPatterns();

    expect(loaded.length).toBe(1);
    expect(loaded[0].id).toBe(pattern.id);
  });

  it('should get pattern statistics', async () => {
    // Add multiple patterns
    for (let i = 0; i < 3; i++) {
      await store.savePattern({
        id: `pattern-${i}`,
        name: `Pattern ${i}`,
        description: 'Test',
        trigger: { type: 'keyword' as const, pattern: 'test' },
        action: { type: 'response' as const, description: 'Test' },
        confidence: 0.8 + i * 0.05,
        usageCount: 10 + i * 5,
        successRate: 0.8,
        avgDuration: 100,
        createdAt: Date.now(),
        lastUsed: Date.now(),
        sourceObservations: [],
        crystallizedAt: Date.now(),
        version: 1,
      });
    }

    const stats = await store.getPatternStats();

    expect(stats.totalPatterns).toBe(3);
    expect(stats.avgSuccessRate).toBeCloseTo(0.8);
  });
});

describe('Overseer', () => {
  let engine: LearningEngine;
  let store: PatternStore;
  let overseer: Overseer;

  beforeEach(async () => {
    engine = new LearningEngine();
    store = new PatternStore();
    await store.clearAll();
    overseer = new Overseer(engine, store, {
      intervalMs: 1000,
      enableAutoCrystallization: true,
      requireApproval: false,
    });
  });

  it('should run learning cycle', async () => {
    // Add observations
    for (let i = 0; i < 8; i++) {
      engine.observeAction('user1', 'test_action', { test: true }, true, 100);
    }

    const report = await overseer.runLearningCycle();

    expect(report).toBeTruthy();
    expect(report.observationsAnalyzed).toBeGreaterThan(0);
  });

  it('should detect and crystallize patterns in cycle', async () => {
    // Add enough observations to trigger crystallization
    for (let i = 0; i < 10; i++) {
      engine.observeAction('user1', 'auto_action', { auto: true }, true, 100);
    }

    const report = await overseer.runLearningCycle();

    expect(report.patternsDetected).toBeGreaterThan(0);
    expect(report.actionsTriggered.crystallized.length).toBeGreaterThan(0);
  });

  it('should generate insights in cycle', async () => {
    // Add high-performing patterns
    for (let i = 0; i < 15; i++) {
      engine.observeAction('user1', 'great_action', { great: true }, true, 50);
    }

    const report = await overseer.runLearningCycle();

    expect(report.actionsTriggered.insights.length).toBeGreaterThan(0);
  });

  it('should track learning cycles', async () => {
    await overseer.runLearningCycle();
    await overseer.runLearningCycle();

    const reports = overseer.getReports();
    expect(reports.length).toBe(2);
  });

  it('should start and stop overseer', () => {
    overseer.start();
    const status = overseer.getStatus();
    expect(status.running).toBe(true);

    overseer.stop();
    const statusAfter = overseer.getStatus();
    expect(statusAfter.running).toBe(false);
  });
});

describe('End-to-End Learning Workflow', () => {
  it('should complete full learning cycle: observe → detect → crystallize → execute', async () => {
    const engine = new LearningEngine();
    const store = new PatternStore();
    await store.clearAll();

    // Step 1: Observe user interactions
    for (let i = 0; i < 8; i++) {
      engine.observeAction(
        'user1',
        'weather_query',
        { message: 'what is the weather today' },
        true,
        100
      );
    }

    // Step 2: Detect patterns
    const patterns = engine.detectPatterns();
    expect(patterns.length).toBeGreaterThan(0);

    // Step 3: Crystallize pattern
    const pattern = patterns[0];
    engine.addPattern(pattern);
    await store.savePattern(pattern);

    // Step 4: Execute crystallized pattern
    const match = engine.findMatchingPattern({
      message: 'weather forecast',
    });
    expect(match).toBeTruthy();

    const result = await engine.executePattern(match!, { message: 'weather' });
    expect(result.success).toBe(true);

    // Step 5: Update confidence
    engine.updatePatternConfidence(pattern.id, true);
    expect(pattern.usageCount).toBeGreaterThan(8);
  });
});
