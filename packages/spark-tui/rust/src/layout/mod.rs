//! TUI Framework - Layout Module
//!
//! Flexbox and Grid layout computation for terminal UI using Taffy 0.9.
//!
//! Contains:
//! - `layout_tree`: Taffy 0.9 trait API directly on SharedBuffer (1024-byte nodes)
//! - `text_measure`: Unicode-aware text measurement for terminal rendering

pub mod layout_tree;
pub mod text_measure;

pub use layout_tree::compute_layout;
pub use text_measure::*;
