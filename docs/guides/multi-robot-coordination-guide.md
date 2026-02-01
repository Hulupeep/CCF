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
