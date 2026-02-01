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
