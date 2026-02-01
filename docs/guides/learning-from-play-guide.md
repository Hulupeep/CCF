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
