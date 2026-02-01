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
