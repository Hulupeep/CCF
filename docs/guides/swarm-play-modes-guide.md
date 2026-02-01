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
