//! TUI Framework - Parallel Arrays
//!
//! All component state lives in these parallel arrays.
//! Each array index corresponds to one component.
//!
//! Components write directly to these arrays using `set_value()` or `set_signal()`.
//! Deriveds read from these arrays directly via `.get()`.
//!
//! All arrays use `TrackedSlotArray` for stable reactive cells with fine-grained
//! per-index tracking. This ensures that deriveds only re-run when the specific
//! indices they access have changed.
//!
//! # Array Categories
//!
//! - **core**: Component type, parent, visibility
//! - **visual**: Colors, borders, opacity
//! - **text**: Text content and styling
//! - **interaction**: Scroll, focus, mouse state
//! - **spacing**: Padding (duplicated for framebuffer reads)
//! - **layout**: Z-index (duplicated for framebuffer reads)

pub mod core;
pub mod visual;
pub mod text;
pub mod interaction;

use self::core as core_arrays;
use self::visual as visual_arrays;
use self::text as text_arrays;
use self::interaction as interaction_arrays;

/// Ensure all arrays have capacity for the given index.
///
/// Called by registry when allocating.
pub fn ensure_all_capacity(index: usize) {
    core_arrays::ensure_capacity(index);
    visual_arrays::ensure_capacity(index);
    text_arrays::ensure_capacity(index);
    interaction_arrays::ensure_capacity(index);
}

/// Clear all array values at an index.
///
/// Called by registry when releasing.
pub fn clear_all_at_index(index: usize) {
    core_arrays::clear_at_index(index);
    visual_arrays::clear_at_index(index);
    text_arrays::clear_at_index(index);
    interaction_arrays::clear_at_index(index);
}

/// Reset all parallel arrays to release memory.
///
/// Called automatically when all components are destroyed (`allocated_indices.size === 0`).
/// This is the "reset on zero" cleanup - no manual API needed!
pub fn reset_all_arrays() {
    core_arrays::reset();
    visual_arrays::reset();
    text_arrays::reset();
    interaction_arrays::reset();
}
