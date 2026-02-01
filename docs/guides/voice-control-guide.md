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
