//! CCF embedded architecture running on RuVector OS
//!
//! This crate provides a minimal implementation that can run
//! directly on the mBot2's CyberPi (ESP32) without a companion laptop.
//!
//! ## Building for ESP32
//!
//! ```bash
//! # Install ESP32 toolchain
//! cargo install espup
//! espup install
//!
//! # Add target
//! rustup target add xtensa-esp32-espidf
//!
//! # Build
//! cargo build --target xtensa-esp32-espidf --release
//! ```

#![no_std]

pub use mbot_core::*;

/// Embedded-specific utilities
pub mod embedded {
    /// Busy-wait delay (when no OS available)
    #[inline]
    pub fn delay_cycles(cycles: u32) {
        for _ in 0..cycles {
            core::hint::spin_loop();
        }
    }

    /// Approximate microsecond delay at 240MHz
    #[inline]
    pub fn delay_us(us: u32) {
        delay_cycles(us * 240);
    }

    /// Approximate millisecond delay
    #[inline]
    pub fn delay_ms(ms: u32) {
        for _ in 0..ms {
            delay_us(1000);
        }
    }
}

// Re-export core brain for embedded use
pub use mbot_core::MBotBrain as Brain;
pub use mbot_core::MBotSensors as Sensors;
pub use mbot_core::MotorCommand as Command;
pub use mbot_core::ReflexMode as Mode;
