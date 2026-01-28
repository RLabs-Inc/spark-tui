//! Terminal renderer - the "blind" output layer.
//!
//! The renderer knows only about cells. It doesn't understand components,
//! layout, or reactivity. It simply takes a filled FrameBuffer and outputs
//! optimized ANSI escape sequences to the terminal.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    Reactive Pipeline                             │
//! │  Components → FlexNodes → layoutDerived → frameBufferDerived    │
//! └─────────────────────────────────────────────────────────────────┘
//!                                   │
//!                                   ▼
//!                          ┌───────────────┐
//!                          │  FrameBuffer  │  ← 2D grid of Cells
//!                          └───────────────┘
//!                                   │
//!                                   ▼
//!                          ┌───────────────┐
//!                          │   Renderer    │  ← This module
//!                          └───────────────┘
//!                                   │
//!                                   ▼
//!                              Terminal
//! ```
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
//!
//! # Key Optimizations
//!
//! 1. **Differential rendering**: Only output cells that changed
//! 2. **Stateful rendering**: Track colors/attrs to skip redundant codes
//! 3. **Output batching**: Single syscall per frame
//! 4. **Synchronized output**: Flicker-free with terminal sync protocol
//!
//! # Example
//!
//! ```no_run
//! use spark_tui::renderer::{FrameBuffer, DiffRenderer};
//! use spark_tui::types::Rgba;
//!
//! // Create a buffer
//! let mut buffer = FrameBuffer::new(80, 24);
//!
//! // Draw something
//! buffer.fill_rect(0, 0, 80, 24, Rgba::rgb(32, 32, 64), None);
//! buffer.draw_text(10, 10, "Hello, TUI!", Rgba::WHITE, None, Default::default(), None);
//!
//! // Render to terminal
//! let mut renderer = DiffRenderer::new();
//! renderer.enter_fullscreen().unwrap();
//! renderer.render(&buffer).unwrap();
//!
//! // Later, modify buffer and render again - only changes are output
//! buffer.draw_text(10, 12, "Updated!", Rgba::GREEN, None, Default::default(), None);
//! renderer.render(&buffer).unwrap();
//!
//! // Cleanup
//! renderer.exit_fullscreen().unwrap();
//! ```

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

// Re-export text utilities from layout (where they belong)
pub use crate::layout::{measure_text_height, string_width as layout_string_width, truncate_text, wrap_text};
