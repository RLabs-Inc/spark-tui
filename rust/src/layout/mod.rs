//! TUI Framework - Layout Module
//!
//! Flexbox layout computation for terminal UI using Taffy.
//!
//! Contains:
//! - `layout_tree`: Taffy trait API on SharedBuffer (512-byte nodes, cache-aligned)
//! - `layout_tree_aos`: Legacy AoS buffer (256-byte nodes) for comparison
//! - `text_measure`: Unicode-aware text measurement for terminal rendering
//! - `types`: Layout-related type definitions

mod types;
pub mod text_measure;
pub mod layout_tree_aos;
pub mod layout_tree;

pub use types::*;
pub use text_measure::*;

// Old AoS buffer layout (256 bytes/node)
pub use layout_tree_aos::compute_layout_aos;

// New cache-aligned layout (512 bytes/node)
pub use layout_tree::compute_layout;
