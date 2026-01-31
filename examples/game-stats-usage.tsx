/**
 * Example: Using the Game Statistics Dashboard
 * Issue: #64 - STORY-GAME-006
 */

import React from 'react';
import { GameStats } from '../web/src/components/GameStats';
import { gameStorage } from '../web/src/services/gameStorage';
import { GameSession } from '../web/src/types/game';

/**
 * Example 1: Record a game session
 */
export function recordGameExample() {
  // After a Tic-Tac-Toe game completes
  const session: GameSession = {
    id: crypto.randomUUID(),
    gameType: 'tictactoe',
    result: 'win',
    score: 1000,
    duration: 60000, // 60 seconds
    timestamp: Date.now(),
    personality: 'Curious',
    metadata: {
      moves: 9,
      difficulty: 'medium',
      playerFirst: true,
    },
  };

  gameStorage.recordSession(session);
  console.log('Game recorded! Statistics updated.');
}

/**
 * Example 2: Display the statistics dashboard
 */
export function StatsPage() {
  return (
    <div className="stats-page">
      <header>
        <h1>mBot Game Statistics</h1>
      </header>
      <GameStats />
    </div>
  );
}

/**
 * Example 3: Export statistics
 */
export function exportStatsExample() {
  // Export as JSON
  const jsonData = gameStorage.exportJSON();
  console.log('JSON Export:', jsonData);

  // Export as CSV
  const csvData = gameStorage.exportCSV();
  console.log('CSV Export:', csvData);
}

/**
 * Example 4: Get filtered statistics
 */
export function getFilteredStatsExample() {
  // Get last 7 days of Tic-Tac-Toe games
  const stats = gameStorage.getFilteredStatistics('7d', 'tictactoe');

  console.log(`Total games: ${stats.totalGames}`);
  console.log(`Win rate: ${(stats.byGame.tictactoe.wins / stats.byGame.tictactoe.totalGames * 100).toFixed(1)}%`);
  console.log(`High score: ${stats.byGame.tictactoe.highScore}`);
}

/**
 * Example 5: Check achievements
 */
export function checkAchievementsExample() {
  const stats = gameStorage.getStatistics();

  const unlockedAchievements = stats.achievements.filter(a => a.unlocked);
  console.log(`Unlocked ${unlockedAchievements.length} / ${stats.achievements.length} achievements`);

  unlockedAchievements.forEach(achievement => {
    console.log(`✓ ${achievement.name}: ${achievement.description}`);
  });
}

/**
 * Example 6: Integration with game components
 */
export function TicTacToeWithStats() {
  const [gameActive, setGameActive] = React.useState(false);
  const [currentScore, setCurrentScore] = React.useState(0);
  const [startTime, setStartTime] = React.useState(0);
  const [personality, setPersonality] = React.useState('Curious');

  const startGame = () => {
    setGameActive(true);
    setCurrentScore(0);
    setStartTime(Date.now());
  };

  const endGame = (result: 'win' | 'loss' | 'draw') => {
    setGameActive(false);

    // Record the game session
    const session: GameSession = {
      id: crypto.randomUUID(),
      gameType: 'tictactoe',
      result,
      score: currentScore,
      duration: Date.now() - startTime,
      timestamp: Date.now(),
      personality,
      metadata: {
        difficulty: 'medium',
      },
    };

    gameStorage.recordSession(session);

    // Show achievement notifications
    const stats = gameStorage.getStatistics();
    const recentlyUnlocked = stats.achievements.filter(
      a => a.unlocked && a.unlockedAt && a.unlockedAt > Date.now() - 5000
    );

    recentlyUnlocked.forEach(achievement => {
      showNotification(`🏆 Achievement Unlocked: ${achievement.name}`);
    });
  };

  return (
    <div>
      <h2>Tic-Tac-Toe</h2>
      {!gameActive ? (
        <button onClick={startGame}>Start Game</button>
      ) : (
        <>
          <div>Score: {currentScore}</div>
          <button onClick={() => endGame('win')}>Win</button>
          <button onClick={() => endGame('loss')}>Loss</button>
          <button onClick={() => endGame('draw')}>Draw</button>
        </>
      )}
    </div>
  );
}

function showNotification(message: string) {
  console.log(message);
  // In real app: toast notification or modal
}

/**
 * Example 7: Personality performance analysis
 */
export function analyzePersonalityPerformance() {
  const stats = gameStorage.getStatistics();

  // Analyze which personality performs best
  const personalityStats = new Map<string, { wins: number; games: number }>();

  Object.values(stats.byGame).forEach(gameStats => {
    Object.entries(gameStats.personalityBreakdown).forEach(([personality, data]) => {
      const existing = personalityStats.get(personality) || { wins: 0, games: 0 };
      personalityStats.set(personality, {
        wins: existing.wins + data.wins,
        games: existing.games + data.games,
      });
    });
  });

  const rankings = Array.from(personalityStats.entries())
    .map(([personality, data]) => ({
      personality,
      winRate: (data.wins / data.games) * 100,
      games: data.games,
    }))
    .sort((a, b) => b.winRate - a.winRate);

  console.log('Personality Rankings:');
  rankings.forEach((rank, index) => {
    console.log(`${index + 1}. ${rank.personality}: ${rank.winRate.toFixed(1)}% (${rank.games} games)`);
  });
}

/**
 * Example 8: Leaderboard display
 */
export function displayLeaderboard() {
  const stats = gameStorage.getStatistics();

  console.log('Top 10 Scores:');
  stats.leaderboard.slice(0, 10).forEach((entry, index) => {
    console.log(
      `${index + 1}. ${entry.score} points - ${entry.gameType} - ${entry.personality} - ${new Date(entry.timestamp).toLocaleDateString()}`
    );
  });
}

/**
 * Example 9: Clear statistics (for testing)
 */
export function clearStatsExample() {
  if (confirm('Are you sure you want to clear all statistics?')) {
    gameStorage.clearStatistics();
    console.log('All statistics cleared.');
  }
}

/**
 * Example 10: Real-time statistics updates
 */
export function RealTimeStatsWidget() {
  const [stats, setStats] = React.useState(gameStorage.getStatistics());

  React.useEffect(() => {
    // Update stats when localStorage changes (cross-tab sync)
    const handleStorageChange = () => {
      setStats(gameStorage.getStatistics());
    };

    window.addEventListener('storage', handleStorageChange);
    return () => window.removeEventListener('storage', handleStorageChange);
  }, []);

  return (
    <div className="stats-widget">
      <h3>Quick Stats</h3>
      <p>Total Games: {stats.totalGames}</p>
      <p>
        Win Rate:{' '}
        {stats.totalGames > 0
          ? (
              (Object.values(stats.byGame).reduce((sum, g) => sum + g.wins, 0) /
                stats.totalGames) *
              100
            ).toFixed(1)
          : 0}
        %
      </p>
      <p>
        High Score:{' '}
        {Math.max(...Object.values(stats.byGame).map(g => g.highScore), 0)}
      </p>
      <p>Achievements: {stats.achievements.filter(a => a.unlocked).length}/{stats.achievements.length}</p>
    </div>
  );
}
