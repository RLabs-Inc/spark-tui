//! TUI Framework - Layout Module
//!
//! Flexbox layout computation for terminal UI using Taffy.
//!
//! Contains:
//! - `layout_tree`: Low-level Taffy trait API integration (zero-copy, zero-translation)
//! - `text_measure`: Unicode-aware text measurement for terminal rendering
//! - `types`: Layout-related type definitions

mod types;
pub mod text_measure;
pub mod layout_tree;

pub use types::*;
pub use text_measure::*;

// Low-level Taffy trait API (zero-copy, zero-translation)
pub use layout_tree::compute_layout_direct;

// Legacy modules depend on crate::engine which only exists in the
// signals-based crate root. They are kept for reference but not compiled
// from lib.rs (the SharedBuffer-based crate root).
#[cfg(feature = "engine")]
mod taffy_bridge;
#[cfg(feature = "engine")]
mod titan;

#[cfg(feature = "engine")]
pub use taffy_bridge::compute_layout_taffy;
#[cfg(feature = "engine")]
pub use taffy_bridge::{invalidate_taffy_cache, reset_taffy_cache, is_taffy_cache_dirty};
#[cfg(feature = "engine")]
pub use taffy_bridge::compute_layout_taffy as compute_layout;
#[cfg(feature = "engine")]
pub use titan::reset_titan_arrays;
