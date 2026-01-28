//! Terminal renderer - the "blind" output layer.
//!
//! The renderer knows only about cells. It doesn't understand components,
//! layout, or reactivity. It simply takes a filled FrameBuffer and outputs
//! optimized ANSI escape sequences to the terminal.
//!
//! # Rendering Modes
//!
//! - **Fullscreen** ([`DiffRenderer`]): Uses alternate screen buffer,
//!   differential rendering (only outputs changed cells)
//!
//! - **Inline** ([`InlineRenderer`]): Renders to normal buffer,
//!   clears and redraws each frame
//!
//! - **Append** ([`AppendRenderer`]): Two regions - frozen history
//!   above, active updating region below

pub mod ansi;
pub mod append;
pub mod buffer;
pub mod diff;
pub mod inline;
pub mod output;

// Re-exports for convenience
pub use append::AppendRenderer;
pub use buffer::{char_width, string_width, BorderColors, BorderSides, FrameBuffer};
pub use crate::types::ClipRect;
pub use diff::DiffRenderer;
pub use inline::InlineRenderer;
pub use output::{OutputBuffer, StatefulCellRenderer};
