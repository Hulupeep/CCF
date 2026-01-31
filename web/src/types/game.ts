/**
 * Game Statistics Type Definitions
 * Contract: GAME-001, GAME-002, GAME-003
 * Journey: J-GAME-FIRST-TICTACTOE
 * Invariant: I-GAME-STAT-001 - Statistics persist across sessions
 */

export type GameType = 'tictactoe' | 'chase' | 'simon-says';

export type GameResult = 'win' | 'loss' | 'draw';

export interface GameSession {
  id: string;
  gameType: GameType;
  result: GameResult;
  score: number;
  duration: number; // milliseconds
  timestamp: number;
  personality: string;
  metadata?: {
    moves?: number;
    difficulty?: string;
    playerFirst?: boolean;
    [key: string]: any;
  };
}

export interface GameStats {
  totalGames: number;
  wins: number;
  losses: number;
  draws: number;
  highScore: number;
  averageScore: number;
  totalDuration: number;
  averageDuration: number;
  lastPlayed?: number;
  personalityBreakdown: Record<string, PersonalityStats>;
}

export interface PersonalityStats {
  games: number;
  wins: number;
  losses: number;
  draws: number;
  averageScore: number;
}

export interface GameStatistics {
  totalGames: number;
  byGame: Record<GameType, GameStats>;
  achievements: Achievement[];
  leaderboard: LeaderboardEntry[];
  sessions: GameSession[];
}

export interface Achievement {
  id: string;
  name: string;
  description: string;
  icon: string;
  unlocked: boolean;
  unlockedAt?: number;
  progress: number;
  maxProgress: number;
  category: 'games' | 'scores' | 'streaks' | 'special';
}

export interface LeaderboardEntry {
  gameType: GameType;
  score: number;
  timestamp: number;
  personality: string;
  duration: number;
}

export type TimeFilter = '24h' | '7d' | '30d' | 'all';

export interface StatsFilter {
  gameType?: GameType;
  timeFilter: TimeFilter;
  personality?: string;
}

export interface ChartDataPoint {
  label: string;
  value: number;
  color?: string;
}

export interface TimeSeriesDataPoint {
  timestamp: number;
  value: number;
  label?: string;
}

/**
 * Calculates game statistics from sessions
 */
export function calculateGameStats(sessions: GameSession[]): GameStats {
  if (sessions.length === 0) {
    return {
      totalGames: 0,
      wins: 0,
      losses: 0,
      draws: 0,
      highScore: 0,
      averageScore: 0,
      totalDuration: 0,
      averageDuration: 0,
      personalityBreakdown: {},
    };
  }

  const wins = sessions.filter(s => s.result === 'win').length;
  const losses = sessions.filter(s => s.result === 'loss').length;
  const draws = sessions.filter(s => s.result === 'draw').length;
  const highScore = Math.max(...sessions.map(s => s.score));
  const totalScore = sessions.reduce((sum, s) => sum + s.score, 0);
  const totalDuration = sessions.reduce((sum, s) => sum + s.duration, 0);

  // Calculate personality breakdown
  const personalityBreakdown: Record<string, PersonalityStats> = {};
  sessions.forEach(session => {
    if (!personalityBreakdown[session.personality]) {
      personalityBreakdown[session.personality] = {
        games: 0,
        wins: 0,
        losses: 0,
        draws: 0,
        averageScore: 0,
      };
    }
    const stats = personalityBreakdown[session.personality];
    stats.games++;
    if (session.result === 'win') stats.wins++;
    if (session.result === 'loss') stats.losses++;
    if (session.result === 'draw') stats.draws++;
  });

  // Calculate average scores per personality
  Object.keys(personalityBreakdown).forEach(personality => {
    const personalitySessions = sessions.filter(s => s.personality === personality);
    const totalScore = personalitySessions.reduce((sum, s) => sum + s.score, 0);
    personalityBreakdown[personality].averageScore = totalScore / personalitySessions.length;
  });

  return {
    totalGames: sessions.length,
    wins,
    losses,
    draws,
    highScore,
    averageScore: totalScore / sessions.length,
    totalDuration,
    averageDuration: totalDuration / sessions.length,
    lastPlayed: Math.max(...sessions.map(s => s.timestamp)),
    personalityBreakdown,
  };
}

/**
 * Filters sessions by time period
 */
export function filterSessionsByTime(sessions: GameSession[], filter: TimeFilter): GameSession[] {
  if (filter === 'all') return sessions;

  const now = Date.now();
  const timeRanges: Record<TimeFilter, number> = {
    '24h': 24 * 60 * 60 * 1000,
    '7d': 7 * 24 * 60 * 60 * 1000,
    '30d': 30 * 24 * 60 * 60 * 1000,
    'all': Infinity,
  };

  const cutoff = now - timeRanges[filter];
  return sessions.filter(s => s.timestamp >= cutoff);
}

/**
 * Formats duration in human-readable format
 */
export function formatDuration(milliseconds: number): string {
  const seconds = Math.floor(milliseconds / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);

  if (hours > 0) {
    return `${hours}h ${minutes % 60}m`;
  }
  if (minutes > 0) {
    return `${minutes}m ${seconds % 60}s`;
  }
  return `${seconds}s`;
}

/**
 * Calculates win rate percentage
 */
export function calculateWinRate(stats: GameStats): number {
  if (stats.totalGames === 0) return 0;
  return (stats.wins / stats.totalGames) * 100;
}
