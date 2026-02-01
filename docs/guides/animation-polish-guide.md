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
