//! Drawing mode: mBot2 draws based on its mood (emotional art!)
//!
//! The robot creates spirograph-like patterns that change based on
//! its tension/coherence state reported by the RuVector OS platform.

use anyhow::Result;
use mbot_core::{MBotBrain, MBotSensors, ReflexMode};
use std::f32::consts::PI;
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Spirograph parameters - modified by emotional state
struct SpirographParams {
    outer_radius: f32,
    inner_radius: f32,
    pen_offset: f32,
    speed: f32,
}

impl SpirographParams {
    fn from_reflex(mode: ReflexMode, tension: f32, coherence: f32) -> Self {
        match mode {
            ReflexMode::Calm => Self {
                outer_radius: 80.0,
                inner_radius: 30.0 + coherence * 20.0,
                pen_offset: 20.0,
                speed: 0.5,
            },
            ReflexMode::Active => Self {
                outer_radius: 70.0,
                inner_radius: 45.0,
                pen_offset: 35.0,
                speed: 1.0,
            },
            ReflexMode::Spike => Self {
                outer_radius: 60.0 + tension * 20.0,
                inner_radius: 15.0 + tension * 10.0,
                pen_offset: 40.0 + tension * 20.0,
                speed: 2.0,
            },
            ReflexMode::Protect => Self {
                outer_radius: 30.0,
                inner_radius: 25.0,
                pen_offset: 5.0,
                speed: 0.3,
            },
        }
    }

    fn point(&self, t: f32) -> (f32, f32) {
        let r_diff = self.outer_radius - self.inner_radius;
        let ratio = r_diff / self.inner_radius;

        let x = r_diff * t.cos() + self.pen_offset * (ratio * t).cos();
        let y = r_diff * t.sin() - self.pen_offset * (ratio * t).sin();

        (x, y)
    }
}

struct EmotionalDrawer {
    brain: MBotBrain,
    center: (f32, f32),
    current_pos: (f32, f32),
    pen_down: bool,
    path: Vec<(f32, f32)>,
}

impl EmotionalDrawer {
    fn new(center: (f32, f32)) -> Self {
        Self {
            brain: MBotBrain::new(),
            center,
            current_pos: center,
            pen_down: false,
            path: Vec::new(),
        }
    }

    async fn draw_emotional_art(&mut self, duration_secs: u32) -> Result<()> {
        let start = Instant::now();
        let mut t: f32 = 0.0;

        println!("🎨 Starting emotional art session for {} seconds...", duration_secs);
        println!("   Watch the pattern change based on the robot's mood!\n");

        // Move to starting position
        self.drive_to(self.center.0, self.center.1).await?;
        self.set_pen(true).await?;

        while start.elapsed().as_secs() < duration_secs as u64 {
            // Simulate sensor input (in real use, this comes from hardware)
            let sensors = self.simulate_sensors();

            // Process through brain
            let (state, _cmd) = self.brain.tick(&sensors);

            // Get spirograph parameters based on emotional state
            let params = SpirographParams::from_reflex(
                state.reflex,
                state.tension,
                state.coherence,
            );

            // Calculate next point
            let (dx, dy) = params.point(t);
            let target = (self.center.0 + dx, self.center.1 + dy);

            // Draw to that point
            self.drive_to(target.0, target.1).await?;
            self.path.push(target);

            // Advance time
            t += 0.05 * params.speed;

            // Print status periodically
            if (t * 100.0) as u32 % 100 == 0 {
                let mode_icon = match state.reflex {
                    ReflexMode::Calm => "😌",
                    ReflexMode::Active => "🔍",
                    ReflexMode::Spike => "⚡",
                    ReflexMode::Protect => "🛡️",
                };
                println!(
                    "{} Mode: {:?} | Tension: {:.2} | Coherence: {:.2} | Points: {}",
                    mode_icon,
                    state.reflex,
                    state.tension,
                    state.coherence,
                    self.path.len()
                );
            }

            // Occasional pen lift for dramatic effect (in Spike mode)
            if state.reflex == ReflexMode::Spike && rand::random::<f32>() < 0.02 {
                self.set_pen(false).await?;
                sleep(Duration::from_millis(100)).await;
                self.set_pen(true).await?;
            }

            sleep(Duration::from_millis(20)).await;
        }

        self.set_pen(false).await?;

        // Sign the artwork
        self.sign_artwork().await?;

        println!("\n✅ Art complete! {} points drawn.", self.path.len());
        self.print_ascii_preview();

        Ok(())
    }

    async fn sign_artwork(&mut self) -> Result<()> {
        println!("✍️  Signing artwork...");

        // Move to bottom right corner
        let sign_pos = (self.center.0 + 60.0, self.center.1 + 60.0);
        self.drive_to(sign_pos.0, sign_pos.1).await?;

        // Draw a small heart as signature
        self.set_pen(true).await?;

        let heart_size = 5.0;
        for i in 0..=20 {
            let t = (i as f32 / 20.0) * 2.0 * PI;
            let x = sign_pos.0 + heart_size * (16.0 * t.sin().powi(3)) / 16.0;
            let y = sign_pos.1 - heart_size * (13.0 * t.cos() - 5.0 * (2.0 * t).cos()
                - 2.0 * (3.0 * t).cos() - (4.0 * t).cos()) / 16.0;
            self.drive_to(x, y).await?;
        }

        self.set_pen(false).await?;
        Ok(())
    }

    fn simulate_sensors(&self) -> MBotSensors {
        // Generate varying sensor data to create interesting patterns
        let tick = self.brain.tick_count();
        let wave1 = ((tick as f32) * 0.01).sin();
        let wave2 = ((tick as f32) * 0.023).sin();

        // Occasionally simulate "interesting" events
        let event = if tick % 500 > 450 {
            0.8 // Something approaching
        } else {
            0.0
        };

        MBotSensors {
            timestamp_us: tick * 20_000,
            ultrasonic_cm: 50.0 + wave1 * 30.0 - event * 40.0,
            encoder_left: (tick * 3) as i32,
            encoder_right: (tick * 3) as i32,
            gyro_z: wave2 * 20.0,
            accel: [wave1 * 2.0, wave2 * 1.5, 9.8],
            sound_level: 0.1 + (wave1 * 0.2).abs(),
            light_level: 0.5 + wave2 * 0.2,
            quad_rgb: [[200, 200, 200]; 4],
        }
    }

    async fn drive_to(&mut self, x: f32, y: f32) -> Result<()> {
        // In simulation, just update position
        // In real implementation, send motor commands
        self.current_pos = (x, y);
        Ok(())
    }

    async fn set_pen(&mut self, down: bool) -> Result<()> {
        self.pen_down = down;
        self.brain.set_pen(down);
        sleep(Duration::from_millis(50)).await;
        Ok(())
    }

    fn print_ascii_preview(&self) {
        if self.path.is_empty() {
            return;
        }

        // Find bounds
        let min_x = self.path.iter().map(|p| p.0).fold(f32::MAX, f32::min);
        let max_x = self.path.iter().map(|p| p.0).fold(f32::MIN, f32::max);
        let min_y = self.path.iter().map(|p| p.1).fold(f32::MAX, f32::min);
        let max_y = self.path.iter().map(|p| p.1).fold(f32::MIN, f32::max);

        let width = 60;
        let height = 30;
        let mut canvas = vec![vec![' '; width]; height];

        // Plot points
        for (x, y) in &self.path {
            let px = ((x - min_x) / (max_x - min_x) * (width - 1) as f32) as usize;
            let py = ((y - min_y) / (max_y - min_y) * (height - 1) as f32) as usize;
            if px < width && py < height {
                canvas[py][px] = '█';
            }
        }

        println!("\n┌{}┐", "─".repeat(width));
        for row in canvas.iter().rev() {
            println!("│{}│", row.iter().collect::<String>());
        }
        println!("└{}┘", "─".repeat(width));
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║        🎨 CCF Emotional Art on RuVector OS 🎨          ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║  The robot draws spirograph patterns that change based     ║");
    println!("║  on its emotional state (tension/coherence).               ║");
    println!("║                                                            ║");
    println!("║  😌 Calm    = Smooth, flowing curves                       ║");
    println!("║  🔍 Active  = Tighter, more complex spirals                ║");
    println!("║  ⚡ Spike   = Chaotic, high-energy patterns                ║");
    println!("║  🛡️  Protect = Small, tight defensive circles              ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    let mut drawer = EmotionalDrawer::new((0.0, 0.0));

    // Draw for 30 seconds
    drawer.draw_emotional_art(30).await?;

    println!("\n🖼️  Artwork complete! Remove paper and admire your creation.");

    Ok(())
}
