/**
 * Game Statistics Storage Service
 * Implements I-GAME-STAT-001: Statistics persist across sessions
 * Uses localStorage for client-side persistence
 */

import {
  GameSession,
  GameStatistics,
  GameType,
  Achievement,
  LeaderboardEntry,
  calculateGameStats,
  filterSessionsByTime,
  TimeFilter,
} from '../types/game';

const STORAGE_KEY = 'mbot_game_statistics';
const LEADERBOARD_MAX_ENTRIES = 100;

/**
 * Achievement definitions (20 achievements)
 */
const ACHIEVEMENT_DEFINITIONS: Omit<Achievement, 'unlocked' | 'unlockedAt' | 'progress'>[] = [
  // Games category
  { id: 'first_game', name: 'First Game', description: 'Play your first game', icon: '🎮', maxProgress: 1, category: 'games' },
  { id: 'ten_games', name: 'Getting Started', description: 'Play 10 games', icon: '🎯', maxProgress: 10, category: 'games' },
  { id: 'hundred_games', name: 'Century Club', description: 'Play 100 games', icon: '💯', maxProgress: 100, category: 'games' },
  { id: 'all_games', name: 'Variety Player', description: 'Play all 3 game types', icon: '🎪', maxProgress: 3, category: 'games' },
  { id: 'marathon', name: 'Marathon', description: 'Play for 1 hour total', icon: '⏱️', maxProgress: 3600000, category: 'games' },

  // Scores category
  { id: 'first_win', name: 'First Victory', description: 'Win your first game', icon: '🏆', maxProgress: 1, category: 'scores' },
  { id: 'perfect_10', name: 'Perfect 10', description: 'Score 1000+ points', icon: '⭐', maxProgress: 1000, category: 'scores' },
  { id: 'high_roller', name: 'High Roller', description: 'Score 5000+ points', icon: '💎', maxProgress: 5000, category: 'scores' },
  { id: 'legendary', name: 'Legendary', description: 'Score 10000+ points', icon: '👑', maxProgress: 10000, category: 'scores' },

  // Streaks category
  { id: 'win_streak_3', name: 'Hat Trick', description: 'Win 3 games in a row', icon: '🎩', maxProgress: 3, category: 'streaks' },
  { id: 'win_streak_5', name: 'Unstoppable', description: 'Win 5 games in a row', icon: '🔥', maxProgress: 5, category: 'streaks' },
  { id: 'win_streak_10', name: 'Domination', description: 'Win 10 games in a row', icon: '⚡', maxProgress: 10, category: 'streaks' },
  { id: 'no_loss_10', name: 'Undefeated', description: 'Go 10 games without losing', icon: '🛡️', maxProgress: 10, category: 'streaks' },

  // Special category
  { id: 'speed_demon', name: 'Speed Demon', description: 'Win a game in under 30 seconds', icon: '💨', maxProgress: 1, category: 'special' },
  { id: 'personality_master', name: 'Personality Master', description: 'Win with all 6 personalities', icon: '🎭', maxProgress: 6, category: 'special' },
  { id: 'comeback_kid', name: 'Comeback Kid', description: 'Win after losing 3 in a row', icon: '🎯', maxProgress: 1, category: 'special' },
  { id: 'tictactoe_master', name: 'Tic-Tac-Toe Master', description: 'Win 50 Tic-Tac-Toe games', icon: '❌', maxProgress: 50, category: 'special' },
  { id: 'chase_champion', name: 'Chase Champion', description: 'Win 50 Chase games', icon: '🏃', maxProgress: 50, category: 'special' },
  { id: 'simon_savant', name: 'Simon Says Savant', description: 'Win 50 Simon Says games', icon: '🧠', maxProgress: 50, category: 'special' },
  { id: 'perfectionist', name: 'Perfectionist', description: 'Achieve 100% win rate over 20 games', icon: '✨', maxProgress: 20, category: 'special' },
];

class GameStorageService {
  private statistics: GameStatistics | null = null;

  constructor() {
    this.loadStatistics();
  }

  /**
   * Load statistics from localStorage
   */
  private loadStatistics(): GameStatistics {
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (stored) {
        this.statistics = JSON.parse(stored);
      }
    } catch (error) {
      console.error('Failed to load game statistics:', error);
    }

    if (!this.statistics) {
      this.statistics = this.createEmptyStatistics();
    }

    return this.statistics;
  }

  /**
   * Save statistics to localStorage
   */
  private saveStatistics(): void {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(this.statistics));
    } catch (error) {
      console.error('Failed to save game statistics:', error);
    }
  }

  /**
   * Create empty statistics object
   */
  private createEmptyStatistics(): GameStatistics {
    return {
      totalGames: 0,
      byGame: {
        'tictactoe': this.createEmptyGameStats(),
        'chase': this.createEmptyGameStats(),
        'simon-says': this.createEmptyGameStats(),
      },
      achievements: ACHIEVEMENT_DEFINITIONS.map(def => ({
        ...def,
        unlocked: false,
        progress: 0,
      })),
      leaderboard: [],
      sessions: [],
    };
  }

  /**
   * Create empty game stats
   */
  private createEmptyGameStats() {
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

  /**
   * Record a new game session
   */
  recordSession(session: GameSession): void {
    if (!this.statistics) {
      this.statistics = this.createEmptyStatistics();
    }

    // Add session
    this.statistics.sessions.push(session);
    this.statistics.totalGames++;

    // Update game-specific stats
    const gameSessions = this.statistics.sessions.filter(s => s.gameType === session.gameType);
    this.statistics.byGame[session.gameType] = calculateGameStats(gameSessions);

    // Update leaderboard
    this.updateLeaderboard(session);

    // Check achievements
    this.updateAchievements();

    // Save to localStorage
    this.saveStatistics();
  }

  /**
   * Update leaderboard with new session
   */
  private updateLeaderboard(session: GameSession): void {
    if (!this.statistics) return;

    const entry: LeaderboardEntry = {
      gameType: session.gameType,
      score: session.score,
      timestamp: session.timestamp,
      personality: session.personality,
      duration: session.duration,
    };

    this.statistics.leaderboard.push(entry);

    // Sort by score (descending) and keep top entries
    this.statistics.leaderboard.sort((a, b) => b.score - a.score);
    this.statistics.leaderboard = this.statistics.leaderboard.slice(0, LEADERBOARD_MAX_ENTRIES);
  }

  /**
   * Update achievement progress
   */
  private updateAchievements(): void {
    if (!this.statistics) return;

    const sessions = this.statistics.sessions;
    const achievements = this.statistics.achievements;

    // First game
    this.checkAchievement('first_game', sessions.length >= 1 ? 1 : 0);

    // Ten games
    this.checkAchievement('ten_games', Math.min(sessions.length, 10));

    // Hundred games
    this.checkAchievement('hundred_games', Math.min(sessions.length, 100));

    // All games variety
    const uniqueGames = new Set(sessions.map(s => s.gameType)).size;
    this.checkAchievement('all_games', uniqueGames);

    // Marathon (1 hour total play time)
    const totalTime = sessions.reduce((sum, s) => sum + s.duration, 0);
    this.checkAchievement('marathon', Math.min(totalTime, 3600000));

    // First win
    const hasWin = sessions.some(s => s.result === 'win');
    this.checkAchievement('first_win', hasWin ? 1 : 0);

    // High scores
    const maxScore = Math.max(...sessions.map(s => s.score), 0);
    this.checkAchievement('perfect_10', Math.min(maxScore, 1000));
    this.checkAchievement('high_roller', Math.min(maxScore, 5000));
    this.checkAchievement('legendary', Math.min(maxScore, 10000));

    // Win streaks
    const currentStreak = this.calculateCurrentWinStreak();
    const maxStreak = this.calculateMaxWinStreak();
    this.checkAchievement('win_streak_3', Math.min(maxStreak, 3));
    this.checkAchievement('win_streak_5', Math.min(maxStreak, 5));
    this.checkAchievement('win_streak_10', Math.min(maxStreak, 10));

    // No loss streak
    const maxNoLossStreak = this.calculateMaxNoLossStreak();
    this.checkAchievement('no_loss_10', Math.min(maxNoLossStreak, 10));

    // Speed demon (win in under 30 seconds)
    const hasSpeedWin = sessions.some(s => s.result === 'win' && s.duration < 30000);
    this.checkAchievement('speed_demon', hasSpeedWin ? 1 : 0);

    // Personality master
    const uniqueWinPersonalities = new Set(sessions.filter(s => s.result === 'win').map(s => s.personality)).size;
    this.checkAchievement('personality_master', uniqueWinPersonalities);

    // Comeback kid
    const hasComebackWin = this.checkComebackWin();
    this.checkAchievement('comeback_kid', hasComebackWin ? 1 : 0);

    // Game-specific masters
    const tictactoeWins = sessions.filter(s => s.gameType === 'tictactoe' && s.result === 'win').length;
    const chaseWins = sessions.filter(s => s.gameType === 'chase' && s.result === 'win').length;
    const simonWins = sessions.filter(s => s.gameType === 'simon-says' && s.result === 'win').length;
    this.checkAchievement('tictactoe_master', Math.min(tictactoeWins, 50));
    this.checkAchievement('chase_champion', Math.min(chaseWins, 50));
    this.checkAchievement('simon_savant', Math.min(simonWins, 50));

    // Perfectionist
    if (sessions.length >= 20) {
      const last20 = sessions.slice(-20);
      const allWins = last20.every(s => s.result === 'win');
      this.checkAchievement('perfectionist', allWins ? 20 : 0);
    }
  }

  /**
   * Check and update achievement
   */
  private checkAchievement(id: string, progress: number): void {
    if (!this.statistics) return;

    const achievement = this.statistics.achievements.find(a => a.id === id);
    if (!achievement) return;

    achievement.progress = progress;

    if (!achievement.unlocked && progress >= achievement.maxProgress) {
      achievement.unlocked = true;
      achievement.unlockedAt = Date.now();
    }
  }

  /**
   * Calculate current win streak
   */
  private calculateCurrentWinStreak(): number {
    if (!this.statistics) return 0;

    let streak = 0;
    for (let i = this.statistics.sessions.length - 1; i >= 0; i--) {
      if (this.statistics.sessions[i].result === 'win') {
        streak++;
      } else {
        break;
      }
    }
    return streak;
  }

  /**
   * Calculate maximum win streak
   */
  private calculateMaxWinStreak(): number {
    if (!this.statistics) return 0;

    let maxStreak = 0;
    let currentStreak = 0;

    for (const session of this.statistics.sessions) {
      if (session.result === 'win') {
        currentStreak++;
        maxStreak = Math.max(maxStreak, currentStreak);
      } else {
        currentStreak = 0;
      }
    }

    return maxStreak;
  }

  /**
   * Calculate maximum no-loss streak
   */
  private calculateMaxNoLossStreak(): number {
    if (!this.statistics) return 0;

    let maxStreak = 0;
    let currentStreak = 0;

    for (const session of this.statistics.sessions) {
      if (session.result !== 'loss') {
        currentStreak++;
        maxStreak = Math.max(maxStreak, currentStreak);
      } else {
        currentStreak = 0;
      }
    }

    return maxStreak;
  }

  /**
   * Check if there's a comeback win (win after 3 losses)
   */
  private checkComebackWin(): boolean {
    if (!this.statistics || this.statistics.sessions.length < 4) return false;

    for (let i = 3; i < this.statistics.sessions.length; i++) {
      const last4 = this.statistics.sessions.slice(i - 3, i + 1);
      const firstThreeLosses = last4.slice(0, 3).every(s => s.result === 'loss');
      const fourthWin = last4[3].result === 'win';

      if (firstThreeLosses && fourthWin) {
        return true;
      }
    }

    return false;
  }

  /**
   * Get all statistics
   */
  getStatistics(): GameStatistics {
    if (!this.statistics) {
      this.statistics = this.loadStatistics();
    }
    return this.statistics;
  }

  /**
   * Get filtered statistics
   */
  getFilteredStatistics(timeFilter: TimeFilter, gameType?: GameType): GameStatistics {
    const stats = this.getStatistics();
    let filteredSessions = filterSessionsByTime(stats.sessions, timeFilter);

    if (gameType) {
      filteredSessions = filteredSessions.filter(s => s.gameType === gameType);
    }

    // Recalculate stats for filtered sessions
    const byGame: Record<GameType, any> = {
      'tictactoe': calculateGameStats(filteredSessions.filter(s => s.gameType === 'tictactoe')),
      'chase': calculateGameStats(filteredSessions.filter(s => s.gameType === 'chase')),
      'simon-says': calculateGameStats(filteredSessions.filter(s => s.gameType === 'simon-says')),
    };

    return {
      ...stats,
      totalGames: filteredSessions.length,
      byGame,
      sessions: filteredSessions,
    };
  }

  /**
   * Export statistics as JSON
   */
  exportJSON(): string {
    const stats = this.getStatistics();
    return JSON.stringify(stats, null, 2);
  }

  /**
   * Export statistics as CSV
   */
  exportCSV(): string {
    const stats = this.getStatistics();
    const headers = ['Game Type', 'Result', 'Score', 'Duration (s)', 'Timestamp', 'Personality'];
    const rows = stats.sessions.map(s => [
      s.gameType,
      s.result,
      s.score.toString(),
      (s.duration / 1000).toFixed(1),
      new Date(s.timestamp).toISOString(),
      s.personality,
    ]);

    return [headers, ...rows].map(row => row.join(',')).join('\n');
  }

  /**
   * Clear all statistics (for testing)
   */
  clearStatistics(): void {
    this.statistics = this.createEmptyStatistics();
    this.saveStatistics();
  }
}

// Export singleton instance
export const gameStorage = new GameStorageService();
