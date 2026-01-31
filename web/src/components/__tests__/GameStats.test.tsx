/**
 * GameStats Component Tests
 * Contract: GAME-001, GAME-002, GAME-003
 * Invariant: I-GAME-STAT-001 - Statistics persist across sessions
 */

import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import '@testing-library/jest-dom';
import { GameStats } from '../GameStats';
import { gameStorage } from '../../services/gameStorage';
import { GameSession, GameType } from '../../types/game';

// Mock localStorage
const localStorageMock = (() => {
  let store: Record<string, string> = {};

  return {
    getItem: (key: string) => store[key] || null,
    setItem: (key: string, value: string) => {
      store[key] = value.toString();
    },
    removeItem: (key: string) => {
      delete store[key];
    },
    clear: () => {
      store = {};
    },
  };
})();

Object.defineProperty(window, 'localStorage', {
  value: localStorageMock,
});

describe('GameStats Component', () => {
  beforeEach(() => {
    localStorage.clear();
    gameStorage.clearStatistics();
  });

  describe('Initial Render', () => {
    it('should render the component with data-testid', () => {
      render(<GameStats />);
      expect(screen.getByTestId('game-stats-dashboard')).toBeInTheDocument();
    });

    it('should show zero stats when no games played', () => {
      render(<GameStats />);
      expect(screen.getByTestId('stat-total-games')).toHaveTextContent('0');
      expect(screen.getByTestId('stat-win-rate')).toHaveTextContent('0.0%');
      expect(screen.getByTestId('stat-high-score')).toHaveTextContent('0');
    });

    it('should render all required filter elements', () => {
      render(<GameStats />);
      expect(screen.getByTestId('game-filter-select')).toBeInTheDocument();
      expect(screen.getByTestId('time-filter-select')).toBeInTheDocument();
      expect(screen.getByTestId('export-json-button')).toBeInTheDocument();
      expect(screen.getByTestId('export-csv-button')).toBeInTheDocument();
    });

    it('should render all three tabs', () => {
      render(<GameStats />);
      expect(screen.getByTestId('tab-overview')).toBeInTheDocument();
      expect(screen.getByTestId('tab-leaderboard')).toBeInTheDocument();
      expect(screen.getByTestId('tab-achievements')).toBeInTheDocument();
    });
  });

  describe('Statistics Display', () => {
    beforeEach(() => {
      // Add sample game sessions
      const sessions: GameSession[] = [
        {
          id: '1',
          gameType: 'tictactoe',
          result: 'win',
          score: 1000,
          duration: 60000,
          timestamp: Date.now() - 3600000,
          personality: 'Curious',
        },
        {
          id: '2',
          gameType: 'tictactoe',
          result: 'loss',
          score: 500,
          duration: 45000,
          timestamp: Date.now() - 7200000,
          personality: 'Brave',
        },
        {
          id: '3',
          gameType: 'chase',
          result: 'win',
          score: 2000,
          duration: 90000,
          timestamp: Date.now() - 10800000,
          personality: 'Curious',
        },
      ];

      sessions.forEach(session => gameStorage.recordSession(session));
    });

    it('should display correct total games', () => {
      render(<GameStats />);
      expect(screen.getByTestId('stat-total-games')).toHaveTextContent('3');
    });

    it('should calculate and display win rate correctly', () => {
      render(<GameStats />);
      // 2 wins out of 3 games = 66.7%
      expect(screen.getByTestId('stat-win-rate')).toHaveTextContent('66.7%');
    });

    it('should display high score correctly', () => {
      render(<GameStats />);
      expect(screen.getByTestId('stat-high-score')).toHaveTextContent('2000');
    });

    it('should show results breakdown chart', () => {
      render(<GameStats />);
      expect(screen.getByTestId('results-chart')).toBeInTheDocument();
      expect(screen.getByTestId('bar-wins')).toBeInTheDocument();
      expect(screen.getByTestId('bar-losses')).toBeInTheDocument();
      expect(screen.getByTestId('bar-draws')).toBeInTheDocument();
    });
  });

  describe('Game Filtering', () => {
    beforeEach(() => {
      const sessions: GameSession[] = [
        {
          id: '1',
          gameType: 'tictactoe',
          result: 'win',
          score: 1000,
          duration: 60000,
          timestamp: Date.now(),
          personality: 'Curious',
        },
        {
          id: '2',
          gameType: 'chase',
          result: 'win',
          score: 1500,
          duration: 70000,
          timestamp: Date.now(),
          personality: 'Brave',
        },
        {
          id: '3',
          gameType: 'simon-says',
          result: 'loss',
          score: 800,
          duration: 50000,
          timestamp: Date.now(),
          personality: 'Shy',
        },
      ];

      sessions.forEach(session => gameStorage.recordSession(session));
    });

    it('should filter by game type', () => {
      render(<GameStats />);

      // Select Tic-Tac-Toe
      fireEvent.change(screen.getByTestId('game-filter-select'), {
        target: { value: 'tictactoe' },
      });

      // Should show 1 game
      expect(screen.getByTestId('stat-total-games')).toHaveTextContent('1');
    });

    it('should show all games when "all" is selected', () => {
      render(<GameStats />);

      fireEvent.change(screen.getByTestId('game-filter-select'), {
        target: { value: 'all' },
      });

      expect(screen.getByTestId('stat-total-games')).toHaveTextContent('3');
    });

    it('should show game stats grid when viewing all games', () => {
      render(<GameStats />);
      expect(screen.getByTestId('game-stats-grid')).toBeInTheDocument();
      expect(screen.getByTestId('game-card-tictactoe')).toBeInTheDocument();
      expect(screen.getByTestId('game-card-chase')).toBeInTheDocument();
      expect(screen.getByTestId('game-card-simon-says')).toBeInTheDocument();
    });
  });

  describe('Time Filtering', () => {
    beforeEach(() => {
      const now = Date.now();
      const sessions: GameSession[] = [
        {
          id: '1',
          gameType: 'tictactoe',
          result: 'win',
          score: 1000,
          duration: 60000,
          timestamp: now - 12 * 60 * 60 * 1000, // 12 hours ago
          personality: 'Curious',
        },
        {
          id: '2',
          gameType: 'chase',
          result: 'win',
          score: 1500,
          duration: 70000,
          timestamp: now - 5 * 24 * 60 * 60 * 1000, // 5 days ago
          personality: 'Brave',
        },
        {
          id: '3',
          gameType: 'simon-says',
          result: 'loss',
          score: 800,
          duration: 50000,
          timestamp: now - 60 * 24 * 60 * 60 * 1000, // 60 days ago
          personality: 'Shy',
        },
      ];

      sessions.forEach(session => gameStorage.recordSession(session));
    });

    it('should filter by last 24 hours', () => {
      render(<GameStats />);

      fireEvent.change(screen.getByTestId('time-filter-select'), {
        target: { value: '24h' },
      });

      expect(screen.getByTestId('stat-total-games')).toHaveTextContent('1');
    });

    it('should filter by last 7 days', () => {
      render(<GameStats />);

      fireEvent.change(screen.getByTestId('time-filter-select'), {
        target: { value: '7d' },
      });

      expect(screen.getByTestId('stat-total-games')).toHaveTextContent('2');
    });

    it('should show all games for "all" time filter', () => {
      render(<GameStats />);

      fireEvent.change(screen.getByTestId('time-filter-select'), {
        target: { value: 'all' },
      });

      expect(screen.getByTestId('stat-total-games')).toHaveTextContent('3');
    });
  });

  describe('Leaderboard Tab', () => {
    beforeEach(() => {
      const sessions: GameSession[] = [
        {
          id: '1',
          gameType: 'tictactoe',
          result: 'win',
          score: 5000,
          duration: 60000,
          timestamp: Date.now(),
          personality: 'Curious',
        },
        {
          id: '2',
          gameType: 'chase',
          result: 'win',
          score: 3000,
          duration: 70000,
          timestamp: Date.now(),
          personality: 'Brave',
        },
        {
          id: '3',
          gameType: 'simon-says',
          result: 'win',
          score: 4000,
          duration: 50000,
          timestamp: Date.now(),
          personality: 'Shy',
        },
      ];

      sessions.forEach(session => gameStorage.recordSession(session));
    });

    it('should switch to leaderboard tab', () => {
      render(<GameStats />);

      fireEvent.click(screen.getByTestId('tab-leaderboard'));
      expect(screen.getByTestId('stats-leaderboard-panel')).toBeInTheDocument();
    });

    it('should display leaderboard table with correct testid', () => {
      render(<GameStats />);
      fireEvent.click(screen.getByTestId('tab-leaderboard'));

      expect(screen.getByTestId('leaderboard-table')).toBeInTheDocument();
    });

    it('should show leaderboard entries sorted by score', () => {
      render(<GameStats />);
      fireEvent.click(screen.getByTestId('tab-leaderboard'));

      // First entry should be highest score (5000)
      expect(screen.getByTestId('score-0')).toHaveTextContent('5000');
      expect(screen.getByTestId('score-1')).toHaveTextContent('4000');
      expect(screen.getByTestId('score-2')).toHaveTextContent('3000');
    });

    it('should display all leaderboard columns', () => {
      render(<GameStats />);
      fireEvent.click(screen.getByTestId('tab-leaderboard'));

      expect(screen.getByTestId('rank-0')).toBeInTheDocument();
      expect(screen.getByTestId('game-0')).toBeInTheDocument();
      expect(screen.getByTestId('score-0')).toBeInTheDocument();
      expect(screen.getByTestId('personality-0')).toBeInTheDocument();
      expect(screen.getByTestId('duration-0')).toBeInTheDocument();
      expect(screen.getByTestId('date-0')).toBeInTheDocument();
    });
  });

  describe('Achievements Tab', () => {
    it('should switch to achievements tab', () => {
      render(<GameStats />);

      fireEvent.click(screen.getByTestId('tab-achievements'));
      expect(screen.getByTestId('stats-achievements-panel')).toBeInTheDocument();
    });

    it('should display achievement cards', () => {
      render(<GameStats />);
      fireEvent.click(screen.getByTestId('tab-achievements'));

      // Check for first game achievement
      expect(screen.getByTestId('achievement-first_game')).toBeInTheDocument();
    });

    it('should unlock achievements when criteria met', () => {
      // Play first game
      gameStorage.recordSession({
        id: '1',
        gameType: 'tictactoe',
        result: 'win',
        score: 1000,
        duration: 60000,
        timestamp: Date.now(),
        personality: 'Curious',
      });

      render(<GameStats />);
      fireEvent.click(screen.getByTestId('tab-achievements'));

      const firstGameAchievement = screen.getByTestId('achievement-first_game');
      expect(firstGameAchievement).toHaveClass('unlocked');
    });
  });

  describe('Personality Performance', () => {
    beforeEach(() => {
      const sessions: GameSession[] = [
        {
          id: '1',
          gameType: 'tictactoe',
          result: 'win',
          score: 1000,
          duration: 60000,
          timestamp: Date.now(),
          personality: 'Curious',
        },
        {
          id: '2',
          gameType: 'chase',
          result: 'win',
          score: 1500,
          duration: 70000,
          timestamp: Date.now(),
          personality: 'Curious',
        },
        {
          id: '3',
          gameType: 'simon-says',
          result: 'loss',
          score: 800,
          duration: 50000,
          timestamp: Date.now(),
          personality: 'Brave',
        },
      ];

      sessions.forEach(session => gameStorage.recordSession(session));
    });

    it('should display personality performance section', () => {
      render(<GameStats />);
      expect(screen.getByTestId('personality-stats')).toBeInTheDocument();
    });

    it('should show personality win rates', () => {
      render(<GameStats />);

      // Curious: 2 wins out of 2 games = 100%
      expect(screen.getByTestId('personality-row-0')).toHaveTextContent('Curious');
      expect(screen.getByTestId('personality-row-0')).toHaveTextContent('100.0%');

      // Brave: 0 wins out of 1 game = 0%
      expect(screen.getByTestId('personality-row-1')).toHaveTextContent('Brave');
      expect(screen.getByTestId('personality-row-1')).toHaveTextContent('0.0%');
    });
  });

  describe('Export Functionality', () => {
    beforeEach(() => {
      gameStorage.recordSession({
        id: '1',
        gameType: 'tictactoe',
        result: 'win',
        score: 1000,
        duration: 60000,
        timestamp: Date.now(),
        personality: 'Curious',
      });
    });

    it('should trigger JSON export', () => {
      // Mock URL.createObjectURL
      global.URL.createObjectURL = jest.fn(() => 'mock-url');
      global.URL.revokeObjectURL = jest.fn();

      // Mock document.createElement
      const mockClick = jest.fn();
      const mockAnchor = {
        href: '',
        download: '',
        click: mockClick,
      };
      jest.spyOn(document, 'createElement').mockReturnValue(mockAnchor as any);

      render(<GameStats />);
      fireEvent.click(screen.getByTestId('export-json-button'));

      expect(mockClick).toHaveBeenCalled();
      expect(mockAnchor.download).toContain('.json');
    });

    it('should trigger CSV export', () => {
      // Mock URL.createObjectURL
      global.URL.createObjectURL = jest.fn(() => 'mock-url');
      global.URL.revokeObjectURL = jest.fn();

      // Mock document.createElement
      const mockClick = jest.fn();
      const mockAnchor = {
        href: '',
        download: '',
        click: mockClick,
      };
      jest.spyOn(document, 'createElement').mockReturnValue(mockAnchor as any);

      render(<GameStats />);
      fireEvent.click(screen.getByTestId('export-csv-button'));

      expect(mockClick).toHaveBeenCalled();
      expect(mockAnchor.download).toContain('.csv');
    });
  });

  describe('Invariant I-GAME-STAT-001: Data Persistence', () => {
    it('should persist statistics to localStorage', () => {
      gameStorage.recordSession({
        id: '1',
        gameType: 'tictactoe',
        result: 'win',
        score: 1000,
        duration: 60000,
        timestamp: Date.now(),
        personality: 'Curious',
      });

      const stored = localStorage.getItem('mbot_game_statistics');
      expect(stored).not.toBeNull();

      const parsed = JSON.parse(stored!);
      expect(parsed.totalGames).toBe(1);
      expect(parsed.sessions).toHaveLength(1);
    });

    it('should load persisted statistics on mount', () => {
      // Pre-populate localStorage
      const mockStats = {
        totalGames: 5,
        byGame: {
          tictactoe: {
            totalGames: 5,
            wins: 3,
            losses: 2,
            draws: 0,
            highScore: 2000,
            averageScore: 1000,
            totalDuration: 300000,
            averageDuration: 60000,
            personalityBreakdown: {},
          },
          chase: {
            totalGames: 0,
            wins: 0,
            losses: 0,
            draws: 0,
            highScore: 0,
            averageScore: 0,
            totalDuration: 0,
            averageDuration: 0,
            personalityBreakdown: {},
          },
          'simon-says': {
            totalGames: 0,
            wins: 0,
            losses: 0,
            draws: 0,
            highScore: 0,
            averageScore: 0,
            totalDuration: 0,
            averageDuration: 0,
            personalityBreakdown: {},
          },
        },
        achievements: [],
        leaderboard: [],
        sessions: [],
      };

      localStorage.setItem('mbot_game_statistics', JSON.stringify(mockStats));

      render(<GameStats />);

      // Component should load persisted data
      expect(screen.getByTestId('stat-total-games')).toHaveTextContent('5');
    });
  });
});
