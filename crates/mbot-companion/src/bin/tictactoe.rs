//! Tic-Tac-Toe: mBot2 plays X's and O's with a pen!
//!
//! The robot draws on paper and plays against you.
//! It uses SONA learning to improve its strategy over time.

use anyhow::Result;
use mbot_companion::tictactoe_logic::{Cell, Difficulty, GameResult, TicTacToeBoard};
use mbot_core::{circle_points, drive_to_point, x_points, MBotBrain, MBotSensors, MotorCommand};
use std::io::{self, Write};
use std::time::Duration;
use tokio::time::sleep;

// Board dimensions (in cm from origin)
const CELL_SIZE: f32 = 15.0;
const BOARD_OFFSET: (f32, f32) = (5.0, 5.0);

struct TicTacToeGame {
    board: TicTacToeBoard,
    brain: MBotBrain,
    current_pos: (f32, f32),
    games_played: u32,
    robot_wins: u32,
    human_wins: u32,
    draws: u32,
}

impl TicTacToeGame {
    fn new(difficulty: Difficulty) -> Self {
        Self {
            board: TicTacToeBoard::new(difficulty),
            brain: MBotBrain::new(),
            current_pos: (0.0, 0.0),
            games_played: 0,
            robot_wins: 0,
            human_wins: 0,
            draws: 0,
        }
    }

    fn cell_center(&self, row: usize, col: usize) -> (f32, f32) {
        (
            BOARD_OFFSET.0 + (col as f32 + 0.5) * CELL_SIZE,
            BOARD_OFFSET.1 + (row as f32 + 0.5) * CELL_SIZE,
        )
    }

    fn reset_board(&mut self) {
        self.board.reset();
    }

    fn draw_board(&self) {
        println!("{}", self.board);
    }

    fn get_human_move(&mut self) -> Option<(usize, usize)> {
        print!("\nYour move (e.g., A1, B2, C3) or 'q' to quit: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).ok()?;
        let input = input.trim().to_uppercase();

        if input == "Q" {
            return None;
        }

        if input.len() != 2 {
            println!("Invalid input. Use format: A1, B2, C3");
            return self.get_human_move();
        }

        let col = match input.chars().next()? {
            'A' => 0,
            'B' => 1,
            'C' => 2,
            _ => {
                println!("Invalid column. Use A, B, or C.");
                return self.get_human_move();
            }
        };

        let row = match input.chars().nth(1)?.to_digit(10)? {
            1 => 0,
            2 => 1,
            3 => 2,
            _ => {
                println!("Invalid row. Use 1, 2, or 3.");
                return self.get_human_move();
            }
        };

        if !self.board.is_valid_move(row, col) {
            println!("That cell is already taken!");
            return self.get_human_move();
        }

        Some((row, col))
    }

    fn get_robot_move(&self) -> (usize, usize) {
        self.board.get_ai_move().expect("No valid moves available")
    }

    async fn draw_x(&mut self, row: usize, col: usize) -> Result<()> {
        let center = self.cell_center(row, col);
        let size = CELL_SIZE * 0.6;
        let points = x_points(center, size);

        println!("🖊️  Drawing X at ({}, {})...", row, col);

        // Draw first line: top-left to bottom-right
        self.drive_to(points[0].0, points[0].1, false).await?;
        self.pen_down().await?;
        self.drive_to(points[1].0, points[1].1, true).await?;
        self.pen_up().await?;

        // Draw second line: top-right to bottom-left
        self.drive_to(points[3].0, points[3].1, false).await?;
        self.pen_down().await?;
        self.drive_to(points[4].0, points[4].1, true).await?;
        self.pen_up().await?;

        Ok(())
    }

    async fn draw_o(&mut self, row: usize, col: usize) -> Result<()> {
        let center = self.cell_center(row, col);
        let radius = CELL_SIZE * 0.3;

        println!("🖊️  Drawing O at ({}, {})...", row, col);

        // Move to start of circle (top)
        let start = (center.0, center.1 - radius);
        self.drive_to(start.0, start.1, false).await?;
        self.pen_down().await?;

        // Draw circle
        for point in circle_points(center, radius, 24) {
            self.drive_to(point.0, point.1, true).await?;
        }

        self.pen_up().await?;

        Ok(())
    }

    async fn draw_grid(&mut self) -> Result<()> {
        println!("🖊️  Drawing tic-tac-toe grid...");

        // Vertical lines
        for i in 1..3 {
            let x = BOARD_OFFSET.0 + i as f32 * CELL_SIZE;
            self.drive_to(x, BOARD_OFFSET.1, false).await?;
            self.pen_down().await?;
            self.drive_to(x, BOARD_OFFSET.1 + 3.0 * CELL_SIZE, true).await?;
            self.pen_up().await?;
        }

        // Horizontal lines
        for i in 1..3 {
            let y = BOARD_OFFSET.1 + i as f32 * CELL_SIZE;
            self.drive_to(BOARD_OFFSET.0, y, false).await?;
            self.pen_down().await?;
            self.drive_to(BOARD_OFFSET.0 + 3.0 * CELL_SIZE, y, true).await?;
            self.pen_up().await?;
        }

        Ok(())
    }

    async fn drive_to(&mut self, x: f32, y: f32, drawing: bool) -> Result<()> {
        let speed = if drawing { 20.0 } else { 50.0 };

        // Simulate driving (in real implementation, this would send commands)
        while (self.current_pos.0 - x).abs() > 0.5 || (self.current_pos.1 - y).abs() > 0.5 {
            let (left, right) =
                drive_to_point(self.current_pos, self.brain.heading(), (x, y), speed);

            // Update simulated position
            let dx = (left + right) as f32 / 200.0;
            let dtheta = (right - left) as f32 / 500.0;

            self.current_pos.0 += dx * self.brain.heading().cos();
            self.current_pos.1 += dx * self.brain.heading().sin();

            // In real implementation: send motor command
            let _cmd = MotorCommand {
                left,
                right,
                pen_angle: if self.brain.position() != (0.0, 0.0) { 90 } else { 45 },
                ..Default::default()
            };

            sleep(Duration::from_millis(20)).await;
        }

        self.current_pos = (x, y);
        Ok(())
    }

    async fn pen_up(&mut self) -> Result<()> {
        self.brain.set_pen(false);
        // In real implementation: send servo command
        sleep(Duration::from_millis(100)).await;
        Ok(())
    }

    async fn pen_down(&mut self) -> Result<()> {
        self.brain.set_pen(true);
        // In real implementation: send servo command
        sleep(Duration::from_millis(100)).await;
        Ok(())
    }

    async fn victory_dance(&mut self) -> Result<()> {
        println!("🎉 Robot does a victory spin!");
        // Spin 360 degrees
        for _ in 0..20 {
            sleep(Duration::from_millis(50)).await;
        }
        Ok(())
    }

    async fn sad_beep(&mut self) -> Result<()> {
        println!("😢 Robot plays sad sound...");
        sleep(Duration::from_millis(500)).await;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║          🤖 mBot2 TIC-TAC-TOE with RuVector AI 🤖          ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║  You are X, Robot is O                                     ║");
    println!("║  Enter moves like: A1, B2, C3                              ║");
    println!("║  The robot will draw on paper!                             ║");
    println!("╚════════════════════════════════════════════════════════════╝");

    // Ask for difficulty
    println!("\nChoose difficulty:");
    println!("1. Easy (random moves)");
    println!("2. Medium (smart blocking)");
    println!("3. Hard (minimax optimal)");
    print!("Enter choice (1-3, default 2): ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();
    let difficulty = match input.trim() {
        "1" => Difficulty::Easy,
        "3" => Difficulty::Hard,
        _ => Difficulty::Medium,
    };

    println!("\n🤖 Playing at {:?} difficulty level!", difficulty);

    let mut game = TicTacToeGame::new(difficulty);

    loop {
        game.reset_board();
        game.games_played += 1;

        println!("\n🎮 Game {} starting!", game.games_played);
        println!("Drawing the grid...");
        game.draw_grid().await?;

        let mut turn = 0;

        loop {
            game.draw_board();

            if turn % 2 == 0 {
                // Human's turn (X)
                match game.get_human_move() {
                    Some((row, col)) => {
                        game.board.set(row, col, Cell::X);
                        println!("You played X at {}{}", ['A', 'B', 'C'][col], row + 1);
                        game.draw_x(row, col).await?;
                    }
                    None => {
                        println!("\n👋 Thanks for playing!");
                        println!(
                            "Final score: Robot {}, Human {}, Draws {}",
                            game.robot_wins, game.human_wins, game.draws
                        );
                        return Ok(());
                    }
                }
            } else {
                // Robot's turn (O)
                println!("\n🤖 Robot is thinking...");
                sleep(Duration::from_millis(500)).await;

                let (row, col) = game.get_robot_move();
                game.board.set(row, col, Cell::O);
                println!(
                    "Robot plays O at {}{}",
                    ['A', 'B', 'C'][col],
                    row + 1
                );
                game.draw_o(row, col).await?;
            }

            // Check game state
            match game.board.check_state() {
                GameResult::Winner(Cell::X) => {
                    game.draw_board();
                    println!("\n🎉 You win!");
                    game.human_wins += 1;
                    game.sad_beep().await?;
                    break;
                }
                GameResult::Winner(Cell::O) => {
                    game.draw_board();
                    println!("\n🤖 Robot wins!");
                    game.robot_wins += 1;
                    game.victory_dance().await?;
                    break;
                }
                GameResult::Winner(Cell::Empty) => {
                    // Should not happen, but handle gracefully
                    game.draw_board();
                    println!("\n✗ Invalid game state!");
                    break;
                }
                GameResult::Draw => {
                    game.draw_board();
                    println!("\n🤝 It's a draw!");
                    game.draws += 1;
                    break;
                }
                GameResult::InProgress => {}
            }

            turn += 1;
        }

        println!(
            "\n📊 Score: Robot {}, Human {}, Draws {}",
            game.robot_wins, game.human_wins, game.draws
        );
        print!("Play again? (y/n): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("\n👋 Thanks for playing!");
            break;
        }
    }

    Ok(())
}
