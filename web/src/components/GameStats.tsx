/**
 * Game Statistics Dashboard Component
 * Contract: GAME-001, GAME-002, GAME-003
 * Journey: J-GAME-FIRST-TICTACTOE
 * Issue: #64 - STORY-GAME-006
 */

import React, { useState, useMemo, useEffect } from 'react';
import {
  GameType,
  TimeFilter,
  GameStatistics,
  GameStats as GameStatsType,
  calculateWinRate,
  formatDuration,
} from '../types/game';
import { gameStorage } from '../services/gameStorage';
import './GameStats.css';

interface GameStatsProps {
  className?: string;
}

export const GameStats: React.FC<GameStatsProps> = ({ className = '' }) => {
  const [statistics, setStatistics] = useState<GameStatistics>(gameStorage.getStatistics());
  const [timeFilter, setTimeFilter] = useState<TimeFilter>('all');
  const [selectedGame, setSelectedGame] = useState<GameType | 'all'>('all');
  const [activeTab, setActiveTab] = useState<'overview' | 'leaderboard' | 'achievements'>('overview');

  // Refresh statistics when component mounts or updates
  useEffect(() => {
    const refreshStats = () => {
      setStatistics(gameStorage.getStatistics());
    };

    // Refresh on mount
    refreshStats();

    // Listen for storage events (cross-tab updates)
    window.addEventListener('storage', refreshStats);
    return () => window.removeEventListener('storage', refreshStats);
  }, []);

  // Filter statistics based on selected filters
  const filteredStats = useMemo(() => {
    return gameStorage.getFilteredStatistics(
      timeFilter,
      selectedGame === 'all' ? undefined : selectedGame
    );
  }, [timeFilter, selectedGame]);

  // Aggregate stats for display
  const aggregateStats = useMemo(() => {
    if (selectedGame === 'all') {
      const allGames = Object.values(filteredStats.byGame);
      return {
        totalGames: allGames.reduce((sum, g) => sum + g.totalGames, 0),
        wins: allGames.reduce((sum, g) => sum + g.wins, 0),
        losses: allGames.reduce((sum, g) => sum + g.losses, 0),
        draws: allGames.reduce((sum, g) => sum + g.draws, 0),
        highScore: Math.max(...allGames.map(g => g.highScore), 0),
        averageScore: allGames.reduce((sum, g) => sum + g.averageScore * g.totalGames, 0) /
                      Math.max(allGames.reduce((sum, g) => sum + g.totalGames, 0), 1),
        totalDuration: allGames.reduce((sum, g) => sum + g.totalDuration, 0),
        averageDuration: allGames.reduce((sum, g) => sum + g.averageDuration * g.totalGames, 0) /
                         Math.max(allGames.reduce((sum, g) => sum + g.totalGames, 0), 1),
      };
    }
    return filteredStats.byGame[selectedGame];
  }, [filteredStats, selectedGame]);

  // Win rate calculation
  const winRate = useMemo(() => {
    if (aggregateStats.totalGames === 0) return 0;
    return (aggregateStats.wins / aggregateStats.totalGames) * 100;
  }, [aggregateStats]);

  // Chart data for wins/losses/draws
  const resultChartData = useMemo(() => {
    return [
      { label: 'Wins', value: aggregateStats.wins, color: '#22c55e' },
      { label: 'Losses', value: aggregateStats.losses, color: '#ef4444' },
      { label: 'Draws', value: aggregateStats.draws, color: '#eab308' },
    ];
  }, [aggregateStats]);

  // Personality performance data
  const personalityData = useMemo(() => {
    const allPersonalities = new Map<string, { wins: number; games: number }>();

    Object.values(filteredStats.byGame).forEach(gameStats => {
      Object.entries(gameStats.personalityBreakdown).forEach(([personality, stats]) => {
        const existing = allPersonalities.get(personality) || { wins: 0, games: 0 };
        allPersonalities.set(personality, {
          wins: existing.wins + stats.wins,
          games: existing.games + stats.games,
        });
      });
    });

    return Array.from(allPersonalities.entries())
      .map(([personality, data]) => ({
        personality,
        winRate: (data.wins / data.games) * 100,
        games: data.games,
      }))
      .sort((a, b) => b.winRate - a.winRate);
  }, [filteredStats]);

  // Export handlers
  const handleExportJSON = () => {
    const data = gameStorage.exportJSON();
    const blob = new Blob([data], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `mbot-game-stats-${Date.now()}.json`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const handleExportCSV = () => {
    const data = gameStorage.exportCSV();
    const blob = new Blob([data], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `mbot-game-stats-${Date.now()}.csv`;
    a.click();
    URL.revokeObjectURL(url);
  };

  // Unlocked achievements
  const unlockedAchievements = useMemo(() => {
    return statistics.achievements.filter(a => a.unlocked);
  }, [statistics.achievements]);

  const gameTypeLabels: Record<GameType | 'all', string> = {
    'all': 'All Games',
    'tictactoe': 'Tic-Tac-Toe',
    'chase': 'Chase',
    'simon-says': 'Simon Says',
  };

  return (
    <div className={`game-stats-dashboard ${className}`} data-testid="game-stats-dashboard">
      {/* Header */}
      <div className="stats-header">
        <h1>Game Statistics</h1>

        {/* Filters */}
        <div className="stats-filters">
          <div className="filter-group">
            <label htmlFor="game-select">Game:</label>
            <select
              id="game-select"
              value={selectedGame}
              onChange={(e) => setSelectedGame(e.target.value as GameType | 'all')}
              data-testid="game-filter-select"
            >
              <option value="all">All Games</option>
              <option value="tictactoe">Tic-Tac-Toe</option>
              <option value="chase">Chase</option>
              <option value="simon-says">Simon Says</option>
            </select>
          </div>

          <div className="filter-group">
            <label htmlFor="time-select">Period:</label>
            <select
              id="time-select"
              value={timeFilter}
              onChange={(e) => setTimeFilter(e.target.value as TimeFilter)}
              data-testid="time-filter-select"
            >
              <option value="24h">Last 24 hours</option>
              <option value="7d">Last 7 days</option>
              <option value="30d">Last 30 days</option>
              <option value="all">All time</option>
            </select>
          </div>

          <div className="export-buttons">
            <button onClick={handleExportJSON} data-testid="export-json-button">
              Export JSON
            </button>
            <button onClick={handleExportCSV} data-testid="export-csv-button">
              Export CSV
            </button>
          </div>
        </div>
      </div>

      {/* Tabs */}
      <div className="stats-tabs">
        <button
          className={activeTab === 'overview' ? 'active' : ''}
          onClick={() => setActiveTab('overview')}
          data-testid="tab-overview"
        >
          Overview
        </button>
        <button
          className={activeTab === 'leaderboard' ? 'active' : ''}
          onClick={() => setActiveTab('leaderboard')}
          data-testid="tab-leaderboard"
        >
          Leaderboard
        </button>
        <button
          className={activeTab === 'achievements' ? 'active' : ''}
          onClick={() => setActiveTab('achievements')}
          data-testid="tab-achievements"
        >
          Achievements ({unlockedAchievements.length}/{statistics.achievements.length})
        </button>
      </div>

      {/* Overview Tab */}
      {activeTab === 'overview' && (
        <div className="stats-overview" data-testid="stats-overview-panel">
          {/* Summary Cards */}
          <div className="summary-cards">
            <div className="stat-card" data-testid="stat-total-games">
              <h3>Total Games</h3>
              <p className="stat-value">{aggregateStats.totalGames}</p>
            </div>

            <div className="stat-card" data-testid="stat-win-rate">
              <h3>Win Rate</h3>
              <p className="stat-value">{winRate.toFixed(1)}%</p>
            </div>

            <div className="stat-card" data-testid="stat-high-score">
              <h3>High Score</h3>
              <p className="stat-value">{aggregateStats.highScore}</p>
            </div>

            <div className="stat-card" data-testid="stat-avg-duration">
              <h3>Avg Duration</h3>
              <p className="stat-value">{formatDuration(aggregateStats.averageDuration)}</p>
            </div>
          </div>

          {/* Results Breakdown */}
          <div className="stats-section">
            <h2>Results Breakdown</h2>
            <div className="results-chart" data-testid="results-chart">
              <div className="bar-chart">
                {resultChartData.map(item => (
                  <div key={item.label} className="bar-item">
                    <div
                      className="bar"
                      style={{
                        height: `${(item.value / Math.max(aggregateStats.totalGames, 1)) * 100}%`,
                        backgroundColor: item.color,
                      }}
                      data-testid={`bar-${item.label.toLowerCase()}`}
                    />
                    <div className="bar-label">
                      <span>{item.label}</span>
                      <span className="bar-value">{item.value}</span>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>

          {/* Per-Game Stats */}
          {selectedGame === 'all' && (
            <div className="stats-section">
              <h2>Stats by Game</h2>
              <div className="game-stats-grid" data-testid="game-stats-grid">
                {(['tictactoe', 'chase', 'simon-says'] as GameType[]).map(gameType => {
                  const stats = filteredStats.byGame[gameType];
                  const gameWinRate = calculateWinRate(stats);

                  return (
                    <div key={gameType} className="game-stat-card" data-testid={`game-card-${gameType}`}>
                      <h3>{gameTypeLabels[gameType]}</h3>
                      <div className="game-stat-item">
                        <span>Games:</span>
                        <span data-testid={`${gameType}-total-games`}>{stats.totalGames}</span>
                      </div>
                      <div className="game-stat-item">
                        <span>Win Rate:</span>
                        <span data-testid={`${gameType}-win-rate`}>{gameWinRate.toFixed(1)}%</span>
                      </div>
                      <div className="game-stat-item">
                        <span>High Score:</span>
                        <span data-testid={`${gameType}-high-score`}>{stats.highScore}</span>
                      </div>
                      <div className="game-stat-item">
                        <span>Record:</span>
                        <span data-testid={`${gameType}-record`}>
                          {stats.wins}W-{stats.losses}L-{stats.draws}D
                        </span>
                      </div>
                    </div>
                  );
                })}
              </div>
            </div>
          )}

          {/* Personality Performance */}
          {personalityData.length > 0 && (
            <div className="stats-section">
              <h2>Personality Performance</h2>
              <div className="personality-stats" data-testid="personality-stats">
                {personalityData.map((data, index) => (
                  <div
                    key={data.personality}
                    className="personality-stat-row"
                    data-testid={`personality-row-${index}`}
                  >
                    <span className="personality-name">{data.personality}</span>
                    <div className="personality-bar-container">
                      <div
                        className="personality-bar"
                        style={{ width: `${data.winRate}%` }}
                      />
                    </div>
                    <span className="personality-winrate">{data.winRate.toFixed(1)}%</span>
                    <span className="personality-games">({data.games} games)</span>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      {/* Leaderboard Tab */}
      {activeTab === 'leaderboard' && (
        <div className="stats-leaderboard" data-testid="stats-leaderboard-panel">
          <h2>Top Scores</h2>
          {filteredStats.leaderboard.length === 0 ? (
            <p className="empty-message">No games played yet. Start playing to see leaderboard!</p>
          ) : (
            <table className="leaderboard-table" data-testid="leaderboard-table">
              <thead>
                <tr>
                  <th>Rank</th>
                  <th>Game</th>
                  <th>Score</th>
                  <th>Personality</th>
                  <th>Duration</th>
                  <th>Date</th>
                </tr>
              </thead>
              <tbody>
                {filteredStats.leaderboard.slice(0, 50).map((entry, index) => (
                  <tr key={`${entry.timestamp}-${index}`} data-testid={`leaderboard-row-${index}`}>
                    <td data-testid={`rank-${index}`}>{index + 1}</td>
                    <td data-testid={`game-${index}`}>{gameTypeLabels[entry.gameType]}</td>
                    <td data-testid={`score-${index}`}>{entry.score}</td>
                    <td data-testid={`personality-${index}`}>{entry.personality}</td>
                    <td data-testid={`duration-${index}`}>{formatDuration(entry.duration)}</td>
                    <td data-testid={`date-${index}`}>
                      {new Date(entry.timestamp).toLocaleDateString()}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      )}

      {/* Achievements Tab */}
      {activeTab === 'achievements' && (
        <div className="stats-achievements" data-testid="stats-achievements-panel">
          <h2>Achievements</h2>
          <div className="achievements-grid">
            {statistics.achievements.map((achievement) => (
              <div
                key={achievement.id}
                className={`achievement-card ${achievement.unlocked ? 'unlocked' : 'locked'}`}
                data-testid={`achievement-${achievement.id}`}
              >
                <div className="achievement-icon">{achievement.icon}</div>
                <h3>{achievement.name}</h3>
                <p>{achievement.description}</p>
                <div className="achievement-progress">
                  <div className="progress-bar">
                    <div
                      className="progress-fill"
                      style={{
                        width: `${(achievement.progress / achievement.maxProgress) * 100}%`,
                      }}
                    />
                  </div>
                  <span className="progress-text">
                    {achievement.progress} / {achievement.maxProgress}
                  </span>
                </div>
                {achievement.unlocked && achievement.unlockedAt && (
                  <p className="unlocked-date">
                    Unlocked: {new Date(achievement.unlockedAt).toLocaleDateString()}
                  </p>
                )}
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};

export default GameStats;
