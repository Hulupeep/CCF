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
