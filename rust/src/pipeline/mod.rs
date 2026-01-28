//! Reactive pipeline — the core of SparkTUI's architecture.
//!
//! Pure reactive propagation. No loops. No polling. No fixed FPS.
//!
//! ```text
//! TS writes props to SharedBuffer → wakes Rust (single byte notification)
//!     → generation signal increments
//!         → layout (IF dirty: run Taffy, write output)
//!             → framebuffer (build 2D cell grid + hit regions)
//!                 → render (diff → ANSI → terminal)
//! ```
//!
//! Rust stdin input → updates state in SharedBuffer → same propagation → terminal
//! Rust writes events to ring buffer → wakes TS → TS dispatches callbacks

pub mod setup;
pub mod terminal;
pub mod wake;

pub use setup::Engine;
pub use terminal::TerminalSetup;
