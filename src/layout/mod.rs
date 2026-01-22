//! TUI Framework - Layout Module
//!
//! Flexbox layout computation for terminal UI using Taffy.
//!
//! # Architecture
//!
//! The layout module uses [Taffy](https://github.com/DioxusLabs/taffy) for
//! W3C-compliant flexbox computation. The bridge:
//!
//! 1. Converts FlexNode properties → Taffy styles
//! 2. Builds Taffy tree from parent relationships
//! 3. Provides measure functions for text intrinsic sizing
//! 4. Extracts computed layout back to our parallel arrays
//!
//! # Reactivity
//!
//! When called from a derived, reading FlexNode.*.get() creates dependencies.
//! The layout derived re-runs when any layout property changes.
//!
//! # Example
//!
//! ```ignore
//! use spark_tui::layout::{compute_layout, ComputedLayout};
//! use spark_signals::derived;
//!
//! let layout = derived(|| {
//!     compute_layout(80, 24, true)
//! });
//! ```

mod types;
mod text_measure;
mod taffy_bridge;
mod titan;

pub use types::*;
pub use text_measure::*;
pub use taffy_bridge::compute_layout_taffy;

// Re-export compute_layout_taffy as the primary compute_layout
pub use taffy_bridge::compute_layout_taffy as compute_layout;

// Keep TITAN available for reference/fallback
pub use titan::reset_titan_arrays;
