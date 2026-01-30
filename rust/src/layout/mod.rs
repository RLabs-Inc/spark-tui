//! TUI Framework - Layout Module
//!
//! Flexbox layout computation for terminal UI using Taffy.
//!
//! Contains:
//! - `layout_tree_aos`: Low-level Taffy trait API on AoSBuffer (zero-copy, cache-friendly)
//! - `text_measure`: Unicode-aware text measurement for terminal rendering
//! - `types`: Layout-related type definitions

mod types;
pub mod text_measure;
pub mod layout_tree_aos;

pub use types::*;
pub use text_measure::*;

// AoS buffer layout (cache-friendly reads)
pub use layout_tree_aos::compute_layout_aos;
