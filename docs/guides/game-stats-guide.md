# Game Statistics Dashboard Guide

**Feature:** Wave 6 Sprint 1
**Component:** `GameStats.tsx`
**Difficulty:** Beginner
**Time to Learn:** 10 minutes

## Overview

The Game Statistics Dashboard tracks all gameplay metrics including wins, losses, streaks, and 20 achievements across Tic-Tac-Toe, Chase, and Simon Says games.

### Key Features

- 3-tab interface: Overview, Leaderboard, Achievements
- 20-achievement system across 4 categories
- Leaderboard with top 100 scores
- Personality performance analysis
- Win/loss/draw charts
- Time-based filtering (24h, 7d, 30d, all time)
- Export to JSON/CSV
- LocalStorage persistence
- Cross-tab synchronization

---

## Quick Start

```typescript
import { GameStats } from '@/components/GameStats';

function App() {
  return <GameStats />;
}
```

---

## Achievement System

### Categories

#### Games Played
- **First Steps**: Play your first game
- **Getting Started**: Play 10 games
- **Dedicated Player**: Play 50 games
- **Master Player**: Play 100 games

#### High Scores
- **Tic-Tac-Toe Master**: Win a Tic-Tac-Toe game
- **Chase Champion**: Win 5 Chase games
- **Simon Savant**: Complete 10-pattern Simon Says

#### Streaks
- **Hot Streak**: Win 3 games in a row
- **Unstoppable**: Win 5 games in a row
- **Legendary**: Win 10 games in a row

#### Special
- **Perfect Game**: Win without losing any rounds
- **Comeback King**: Win after being behind 2-0
- **Speedrun**: Win a game in under 30 seconds
- **Perfectionist**: 100% win rate (min 10 games)

---

## Leaderboard

### Ranking System

```typescript
interface LeaderboardEntry {
  rank: number;
  playerName: string; // Or "Robot + Personality"
  game: 'tictactoe' | 'chase' | 'simon';
  score: number;
  personality: string; // Which personality was active
  timestamp: number;
}
```

### Filtering

```typescript
<GameStats
  gameFilter="tictactoe" // or "chase", "simon", "all"
  timeRange="7d" // or "24h", "30d", "all"
/>
```

---

## Personality Performance

See which personalities perform best:

```typescript
const personalityStats = {
  'Scientist': { wins: 45, losses: 12, winRate: 0.789 },
  'Playful': { wins: 32, losses: 28, winRate: 0.533 },
  'Focused': { wins: 67, losses: 8, winRate: 0.893 },
};
```

---

## Export

```typescript
import { gameStorage } from '@/services/gameStorage';

// Export JSON
const stats = await gameStorage.exportStats();
const json = JSON.stringify(stats, null, 2);

// Export CSV
const csv = await gameStorage.exportCSV();
```

---

## Related Features

- [Personality Mixer](personality-mixer-guide.md)
- [Data Export/Import](data-export-import-guide.md)
- [Cloud Sync](cloud-sync-guide.md)

---

**Last Updated:** 2026-02-01
**Status:** Production Ready ✅
