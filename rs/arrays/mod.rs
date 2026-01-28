//! TUI Framework - Parallel Arrays
//!
//! All component state lives in these parallel arrays.
//! Each array index corresponds to one component.
//!
//! # Array Categories
//!
//! - **core**: Component type, parent, visibility
//! - **visual**: Colors, borders, opacity
//! - **text**: Text content and styling
//! - **interaction**: Scroll, focus, mouse state

pub mod core;
pub mod visual;
pub mod text;
pub mod interaction;
pub mod dirty;

use spark_signals::TrackedSlotArray;

/// Trait to add clear_all functionality to TrackedSlotArray.
/// This restores behavior that was removed from spark-signals v0.1.2.
pub trait ClearAll {
    fn clear_all(&self);
}

impl<T: Clone + PartialEq + 'static> ClearAll for TrackedSlotArray<T> {
    fn clear_all(&self) {
        for i in 0..self.len() {
            self.clear(i);
        }
    }
}

/// Ensure all arrays have capacity for the given index.
pub fn ensure_all_capacity(index: usize) {
    core::ensure_capacity(index);
    visual::ensure_capacity(index);
    text::ensure_capacity(index);
    interaction::ensure_capacity(index);
}

/// Clear all array values at an index.
pub fn clear_all_at_index(index: usize) {
    core::clear_at_index(index);
    visual::clear_at_index(index);
    text::clear_at_index(index);
    interaction::clear_at_index(index);
}

/// Reset all parallel arrays.
pub fn reset_all_arrays() {
    core::reset();
    visual::reset();
    text::reset();
    interaction::reset();
}
