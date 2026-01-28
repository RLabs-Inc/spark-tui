//! Framebuffer computation from SharedBuffer.
//!
//! This module reads the SharedBuffer (layout output + visual + text + interaction)
//! and produces a filled FrameBuffer ready for the renderer.
//!
//! # Pipeline Position
//!
//! ```text
//! SharedBuffer → layout (Taffy) → OUTPUT section
//!                                      │
//!                                      ▼
//!                              THIS MODULE
//!                    (reads output + visual + text + interaction)
//!                                      │
//!                                      ▼
//!                              FrameBuffer (2D cell grid)
//!                                      │
//!                                      ▼
//!                              Renderer (diff → ANSI → terminal)
//! ```

mod render_tree;
mod inheritance;

pub use render_tree::{compute_framebuffer, HitRegion};

// Re-export FrameBuffer from renderer for convenience
pub use crate::renderer::FrameBuffer;
