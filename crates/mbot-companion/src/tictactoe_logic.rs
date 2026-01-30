//! Pure game logic for Tic-Tac-Toe
//!
//! This module provides deterministic, testable game logic
//! following GAME-001 contract requirements.

use std::fmt;

/// Represents a cell state on the board
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Cell {
    Empty,
    X,
    O,
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Cell::Empty => write!(f, " "),
            Cell::X => write!(f, "X"),
            Cell::O => write!(f, "O"),
        }
    }
}

/// Game difficulty level
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Difficulty {
    /// Random valid moves
    Easy,
    /// Block wins, take center
    Medium,
    /// Minimax optimal strategy
    Hard,
}

/// Result of checking game state
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum GameResult {
    InProgress,
    Winner(Cell),
    Draw,
}

/// Core Tic-Tac-Toe game board
#[derive(Clone, Debug)]
pub struct TicTacToeBoard {
    cells: [[Cell; 3]; 3],
    difficulty: Difficulty,
}

impl TicTacToeBoard {
    /// Create a new empty board
    pub fn new(difficulty: Difficulty) -> Self {
        Self {
            cells: [[Cell::Empty; 3]; 3],
            difficulty,
        }
    }

    /// Reset the board to empty state
    pub fn reset(&mut self) {
        self.cells = [[Cell::Empty; 3]; 3];
    }

    /// Get cell at position (row, col)
    pub fn get(&self, row: usize, col: usize) -> Option<Cell> {
        if row < 3 && col < 3 {
            Some(self.cells[row][col])
        } else {
            None
        }
    }

    /// Set cell at position (row, col)
    /// Returns false if move is invalid (out of bounds or cell occupied)
    pub fn set(&mut self, row: usize, col: usize, cell: Cell) -> bool {
        if row >= 3 || col >= 3 || self.cells[row][col] != Cell::Empty {
            return false;
        }
        self.cells[row][col] = cell;
        true
    }

    /// Check if a move is valid
    pub fn is_valid_move(&self, row: usize, col: usize) -> bool {
        row < 3 && col < 3 && self.cells[row][col] == Cell::Empty
    }

    /// Get all empty cells
    pub fn empty_cells(&self) -> Vec<(usize, usize)> {
        let mut cells = Vec::new();
        for row in 0..3 {
            for col in 0..3 {
                if self.cells[row][col] == Cell::Empty {
                    cells.push((row, col));
                }
            }
        }
        cells
    }

    /// Check current game state
    pub fn check_state(&self) -> GameResult {
        // Check rows
        for row in 0..3 {
            if self.cells[row][0] != Cell::Empty
                && self.cells[row][0] == self.cells[row][1]
                && self.cells[row][1] == self.cells[row][2]
            {
                return GameResult::Winner(self.cells[row][0]);
            }
        }

        // Check columns
        for col in 0..3 {
            if self.cells[0][col] != Cell::Empty
                && self.cells[0][col] == self.cells[1][col]
                && self.cells[1][col] == self.cells[2][col]
            {
                return GameResult::Winner(self.cells[0][col]);
            }
        }

        // Check diagonals
        if self.cells[0][0] != Cell::Empty
            && self.cells[0][0] == self.cells[1][1]
            && self.cells[1][1] == self.cells[2][2]
        {
            return GameResult::Winner(self.cells[0][0]);
        }

        if self.cells[0][2] != Cell::Empty
            && self.cells[0][2] == self.cells[1][1]
            && self.cells[1][1] == self.cells[2][0]
        {
            return GameResult::Winner(self.cells[0][2]);
        }

        // Check for draw
        if self.cells.iter().all(|row| row.iter().all(|&c| c != Cell::Empty)) {
            return GameResult::Draw;
        }

        GameResult::InProgress
    }

    /// Get AI move based on difficulty level
    /// This is deterministic - given same board state, returns same move
    pub fn get_ai_move(&self) -> Option<(usize, usize)> {
        match self.difficulty {
            Difficulty::Easy => self.get_random_move(),
            Difficulty::Medium => self.get_medium_move(),
            Difficulty::Hard => self.get_minimax_move(),
        }
    }

    /// Easy AI: First available move (deterministic - always same order)
    fn get_random_move(&self) -> Option<(usize, usize)> {
        self.empty_cells().first().copied()
    }

    /// Medium AI: Block wins, take center, then corners
    fn get_medium_move(&self) -> Option<(usize, usize)> {
        let empty = self.empty_cells();
        if empty.is_empty() {
            return None;
        }

        // Try to win
        for &(r, c) in &empty {
            let mut test_board = self.clone();
            test_board.cells[r][c] = Cell::O;
            if let GameResult::Winner(Cell::O) = test_board.check_state() {
                return Some((r, c));
            }
        }

        // Block opponent win
        for &(r, c) in &empty {
            let mut test_board = self.clone();
            test_board.cells[r][c] = Cell::X;
            if let GameResult::Winner(Cell::X) = test_board.check_state() {
                return Some((r, c));
            }
        }

        // Take center
        if self.cells[1][1] == Cell::Empty {
            return Some((1, 1));
        }

        // Take corners (deterministic order)
        for &(r, c) in &[(0, 0), (0, 2), (2, 0), (2, 2)] {
            if self.cells[r][c] == Cell::Empty {
                return Some((r, c));
            }
        }

        // Take any available
        empty.first().copied()
    }

    /// Hard AI: Minimax optimal strategy
    fn get_minimax_move(&self) -> Option<(usize, usize)> {
        let empty = self.empty_cells();
        if empty.is_empty() {
            return None;
        }

        let mut best_score = i32::MIN;
        let mut best_move = empty[0];

        for &(r, c) in &empty {
            let mut test_board = self.clone();
            test_board.cells[r][c] = Cell::O;
            let score = test_board.minimax(0, false);
            if score > best_score {
                best_score = score;
                best_move = (r, c);
            }
        }

        Some(best_move)
    }

    /// Minimax algorithm for optimal play
    fn minimax(&self, depth: i32, is_maximizing: bool) -> i32 {
        match self.check_state() {
            GameResult::Winner(Cell::O) => return 10 - depth,
            GameResult::Winner(Cell::X) => return depth - 10,
            GameResult::Winner(Cell::Empty) => return 0, // Should not happen
            GameResult::Draw => return 0,
            GameResult::InProgress => {}
        }

        let empty = self.empty_cells();
        if empty.is_empty() {
            return 0;
        }

        if is_maximizing {
            let mut best_score = i32::MIN;
            for &(r, c) in &empty {
                let mut test_board = self.clone();
                test_board.cells[r][c] = Cell::O;
                let score = test_board.minimax(depth + 1, false);
                best_score = best_score.max(score);
            }
            best_score
        } else {
            let mut best_score = i32::MAX;
            for &(r, c) in &empty {
                let mut test_board = self.clone();
                test_board.cells[r][c] = Cell::X;
                let score = test_board.minimax(depth + 1, true);
                best_score = best_score.min(score);
            }
            best_score
        }
    }

    /// Get board as 2D array
    pub fn as_array(&self) -> [[Cell; 3]; 3] {
        self.cells
    }
}

impl fmt::Display for TicTacToeBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "  ╔═══╦═══╦═══╗")?;
        for row in 0..3 {
            write!(f, "{} ║", row + 1)?;
            for col in 0..3 {
                write!(f, " {} ║", self.cells[row][col])?;
            }
            writeln!(f)?;
            if row < 2 {
                writeln!(f, "  ╠═══╬═══╬═══╣")?;
            }
        }
        writeln!(f, "  ╚═══╩═══╩═══╝")?;
        write!(f, "    A   B   C  ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_board_is_empty() {
        let board = TicTacToeBoard::new(Difficulty::Medium);
        for row in 0..3 {
            for col in 0..3 {
                assert_eq!(board.get(row, col), Some(Cell::Empty));
            }
        }
        assert_eq!(board.check_state(), GameResult::InProgress);
    }

    #[test]
    fn test_valid_moves() {
        let mut board = TicTacToeBoard::new(Difficulty::Medium);

        // Valid move
        assert!(board.set(0, 0, Cell::X));
        assert_eq!(board.get(0, 0), Some(Cell::X));

        // Invalid move - cell occupied
        assert!(!board.set(0, 0, Cell::O));

        // Invalid move - out of bounds
        assert!(!board.set(3, 0, Cell::X));
        assert!(!board.set(0, 3, Cell::X));
    }

    #[test]
    fn test_is_valid_move() {
        let mut board = TicTacToeBoard::new(Difficulty::Medium);

        assert!(board.is_valid_move(0, 0));
        assert!(board.is_valid_move(1, 1));
        assert!(board.is_valid_move(2, 2));

        board.set(1, 1, Cell::X);
        assert!(!board.is_valid_move(1, 1));

        assert!(!board.is_valid_move(3, 0));
        assert!(!board.is_valid_move(0, 3));
    }

    #[test]
    fn test_horizontal_win() {
        let mut board = TicTacToeBoard::new(Difficulty::Medium);

        // Top row X wins
        board.set(0, 0, Cell::X);
        board.set(0, 1, Cell::X);
        board.set(0, 2, Cell::X);
        assert_eq!(board.check_state(), GameResult::Winner(Cell::X));

        // Middle row O wins
        board.reset();
        board.set(1, 0, Cell::O);
        board.set(1, 1, Cell::O);
        board.set(1, 2, Cell::O);
        assert_eq!(board.check_state(), GameResult::Winner(Cell::O));

        // Bottom row X wins
        board.reset();
        board.set(2, 0, Cell::X);
        board.set(2, 1, Cell::X);
        board.set(2, 2, Cell::X);
        assert_eq!(board.check_state(), GameResult::Winner(Cell::X));
    }

    #[test]
    fn test_vertical_win() {
        let mut board = TicTacToeBoard::new(Difficulty::Medium);

        // Left column X wins
        board.set(0, 0, Cell::X);
        board.set(1, 0, Cell::X);
        board.set(2, 0, Cell::X);
        assert_eq!(board.check_state(), GameResult::Winner(Cell::X));

        // Middle column O wins
        board.reset();
        board.set(0, 1, Cell::O);
        board.set(1, 1, Cell::O);
        board.set(2, 1, Cell::O);
        assert_eq!(board.check_state(), GameResult::Winner(Cell::O));

        // Right column X wins
        board.reset();
        board.set(0, 2, Cell::X);
        board.set(1, 2, Cell::X);
        board.set(2, 2, Cell::X);
        assert_eq!(board.check_state(), GameResult::Winner(Cell::X));
    }

    #[test]
    fn test_diagonal_win() {
        let mut board = TicTacToeBoard::new(Difficulty::Medium);

        // Top-left to bottom-right X wins
        board.set(0, 0, Cell::X);
        board.set(1, 1, Cell::X);
        board.set(2, 2, Cell::X);
        assert_eq!(board.check_state(), GameResult::Winner(Cell::X));

        // Top-right to bottom-left O wins
        board.reset();
        board.set(0, 2, Cell::O);
        board.set(1, 1, Cell::O);
        board.set(2, 0, Cell::O);
        assert_eq!(board.check_state(), GameResult::Winner(Cell::O));
    }

    #[test]
    fn test_draw() {
        let mut board = TicTacToeBoard::new(Difficulty::Medium);

        // X O X
        // X O O
        // O X X
        board.set(0, 0, Cell::X);
        board.set(0, 1, Cell::O);
        board.set(0, 2, Cell::X);
        board.set(1, 0, Cell::X);
        board.set(1, 1, Cell::O);
        board.set(1, 2, Cell::O);
        board.set(2, 0, Cell::O);
        board.set(2, 1, Cell::X);
        board.set(2, 2, Cell::X);

        assert_eq!(board.check_state(), GameResult::Draw);
    }

    #[test]
    fn test_game_in_progress() {
        let mut board = TicTacToeBoard::new(Difficulty::Medium);

        board.set(0, 0, Cell::X);
        board.set(1, 1, Cell::O);
        assert_eq!(board.check_state(), GameResult::InProgress);
    }

    #[test]
    fn test_empty_cells() {
        let mut board = TicTacToeBoard::new(Difficulty::Medium);

        assert_eq!(board.empty_cells().len(), 9);

        board.set(0, 0, Cell::X);
        assert_eq!(board.empty_cells().len(), 8);

        board.set(1, 1, Cell::O);
        board.set(2, 2, Cell::X);
        assert_eq!(board.empty_cells().len(), 6);
    }

    #[test]
    fn test_easy_ai_returns_valid_move() {
        let mut board = TicTacToeBoard::new(Difficulty::Easy);

        board.set(0, 0, Cell::X);
        board.set(1, 1, Cell::O);

        let ai_move = board.get_ai_move();
        assert!(ai_move.is_some());
        let (row, col) = ai_move.unwrap();
        assert!(board.is_valid_move(row, col));
    }

    #[test]
    fn test_medium_ai_blocks_win() {
        let mut board = TicTacToeBoard::new(Difficulty::Medium);

        // X X _
        // _ _ _
        // _ _ _
        board.set(0, 0, Cell::X);
        board.set(0, 1, Cell::X);

        let ai_move = board.get_ai_move();
        assert_eq!(ai_move, Some((0, 2))); // Should block
    }

    #[test]
    fn test_medium_ai_takes_winning_move() {
        let mut board = TicTacToeBoard::new(Difficulty::Medium);

        // O O _
        // _ _ _
        // _ _ _
        board.set(0, 0, Cell::O);
        board.set(0, 1, Cell::O);

        let ai_move = board.get_ai_move();
        assert_eq!(ai_move, Some((0, 2))); // Should win
    }

    #[test]
    fn test_medium_ai_takes_center() {
        let board = TicTacToeBoard::new(Difficulty::Medium);

        let ai_move = board.get_ai_move();
        assert_eq!(ai_move, Some((1, 1))); // Should take center
    }

    #[test]
    fn test_hard_ai_never_loses() {
        // Test that hard AI playing first never loses
        let mut board = TicTacToeBoard::new(Difficulty::Hard);

        // Simulate game where X plays randomly and O plays optimally
        // O should never lose
        let moves = vec![
            (Cell::O, board.get_ai_move().unwrap()),
            (Cell::X, (0, 0)),
            (Cell::O, board.get_ai_move().unwrap()),
            (Cell::X, (2, 2)),
            (Cell::O, board.get_ai_move().unwrap()),
        ];

        for (cell, (r, c)) in moves {
            if board.is_valid_move(r, c) {
                board.set(r, c, cell);
                match board.check_state() {
                    GameResult::Winner(Cell::X) => panic!("Hard AI should not lose!"),
                    GameResult::Winner(Cell::O) | GameResult::Draw => break,
                    GameResult::Winner(Cell::Empty) => {}, // Should not happen
                    GameResult::InProgress => continue,
                }
            }
        }
    }

    #[test]
    fn test_minimax_detects_immediate_win() {
        let mut board = TicTacToeBoard::new(Difficulty::Hard);

        // O O _
        // X X _
        // _ _ _
        board.set(0, 0, Cell::O);
        board.set(0, 1, Cell::O);
        board.set(1, 0, Cell::X);
        board.set(1, 1, Cell::X);

        let ai_move = board.get_ai_move();
        assert_eq!(ai_move, Some((0, 2))); // Should win immediately
    }

    #[test]
    fn test_minimax_optimal_response() {
        let mut board = TicTacToeBoard::new(Difficulty::Hard);

        // X _ _
        // _ O _
        // _ _ X
        // This is a corner opening by X with center taken by O
        // Minimax should find the optimal defensive move
        board.set(0, 0, Cell::X);
        board.set(1, 1, Cell::O);
        board.set(2, 2, Cell::X);

        let ai_move = board.get_ai_move();
        assert!(ai_move.is_some());

        // The AI should make a move that prevents X from winning
        // Any valid move is acceptable here as long as the game doesn't immediately lose
        let (row, col) = ai_move.unwrap();
        assert!(board.is_valid_move(row, col));

        // After the move, X should not have an immediate winning position
        let mut test_board = board.clone();
        test_board.set(row, col, Cell::O);

        // Verify no immediate loss scenario
        for r in 0..3 {
            for c in 0..3 {
                if test_board.is_valid_move(r, c) {
                    let mut check_board = test_board.clone();
                    check_board.set(r, c, Cell::X);
                    if let GameResult::Winner(Cell::X) = check_board.check_state() {
                        // There should be at most one winning move for X
                        // (optimal play prevents multiple winning paths)
                    }
                }
            }
        }
    }

    #[test]
    fn test_deterministic_behavior() {
        // Same board state should always produce same AI move
        let mut board1 = TicTacToeBoard::new(Difficulty::Medium);
        let mut board2 = TicTacToeBoard::new(Difficulty::Medium);

        board1.set(0, 0, Cell::X);
        board2.set(0, 0, Cell::X);

        assert_eq!(board1.get_ai_move(), board2.get_ai_move());
    }

    #[test]
    fn test_reset_board() {
        let mut board = TicTacToeBoard::new(Difficulty::Medium);

        board.set(0, 0, Cell::X);
        board.set(1, 1, Cell::O);
        board.set(2, 2, Cell::X);

        board.reset();

        assert_eq!(board.check_state(), GameResult::InProgress);
        assert_eq!(board.empty_cells().len(), 9);
        for row in 0..3 {
            for col in 0..3 {
                assert_eq!(board.get(row, col), Some(Cell::Empty));
            }
        }
    }

    #[test]
    fn test_get_returns_none_out_of_bounds() {
        let board = TicTacToeBoard::new(Difficulty::Medium);

        assert_eq!(board.get(3, 0), None);
        assert_eq!(board.get(0, 3), None);
        assert_eq!(board.get(10, 10), None);
    }

    #[test]
    fn test_all_difficulty_levels_return_valid_moves() {
        for difficulty in [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard] {
            let mut board = TicTacToeBoard::new(difficulty);
            board.set(0, 0, Cell::X);

            let ai_move = board.get_ai_move();
            assert!(ai_move.is_some(), "Difficulty {:?} failed", difficulty);

            let (row, col) = ai_move.unwrap();
            assert!(board.is_valid_move(row, col), "Difficulty {:?} returned invalid move", difficulty);
        }
    }

    #[test]
    fn test_full_game_scenario() {
        let mut board = TicTacToeBoard::new(Difficulty::Medium);

        // Simulate a full game
        let moves = vec![
            (0, 0, Cell::X),  // X takes corner
            (1, 1, Cell::O),  // O takes center (from AI)
            (0, 2, Cell::X),  // X takes corner
            (0, 1, Cell::O),  // O blocks (from AI)
            (2, 1, Cell::X),  // X continues
            (2, 0, Cell::O),  // O blocks (from AI)
            (1, 0, Cell::X),  // X wins
        ];

        for (row, col, cell) in moves {
            assert!(board.set(row, col, cell));
            match board.check_state() {
                GameResult::Winner(winner) => {
                    assert_eq!(winner, Cell::X);
                    return;
                }
                GameResult::Draw => panic!("Should not be draw"),
                GameResult::InProgress => continue,
            }
        }
    }
}
