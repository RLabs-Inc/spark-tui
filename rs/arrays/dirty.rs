//! Global Dirty Sets for TUI Arrays
//!
//! These sets track which components have modified properties in each category.
//! This enables incremental updates for layout and rendering.

use spark_signals::{DirtySet, dirty_set};

thread_local! {
    /// Dirty set for Core arrays (component type, parent, visibility).
    /// Changes here often affect both layout and rendering.
    pub static CORE_DIRTY_SET: DirtySet = dirty_set();

    /// Dirty set for Visual arrays (colors, borders, opacity).
    /// Changes here usually only affect rendering.
    pub static VISUAL_DIRTY_SET: DirtySet = dirty_set();

    /// Dirty set for Text arrays (content, style).
    /// Changes here affect layout (measure) and rendering.
    pub static TEXT_DIRTY_SET: DirtySet = dirty_set();

    /// Dirty set for Interaction arrays (scroll, focus, mouse).
    /// Changes here affect rendering (scroll position, focus state).
    pub static INTERACTION_DIRTY_SET: DirtySet = dirty_set();
}

/// Get the core dirty set.
pub fn get_core_dirty_set() -> DirtySet {
    CORE_DIRTY_SET.with(|s| s.clone())
}

/// Get the visual dirty set.
pub fn get_visual_dirty_set() -> DirtySet {
    VISUAL_DIRTY_SET.with(|s| s.clone())
}

/// Get the text dirty set.
pub fn get_text_dirty_set() -> DirtySet {
    TEXT_DIRTY_SET.with(|s| s.clone())
}

/// Get the interaction dirty set.
pub fn get_interaction_dirty_set() -> DirtySet {
    INTERACTION_DIRTY_SET.with(|s| s.clone())
}

/// Clear all dirty sets.
pub fn clear_all_dirty_sets() {
    CORE_DIRTY_SET.with(|s| s.borrow_mut().clear());
    VISUAL_DIRTY_SET.with(|s| s.borrow_mut().clear());
    TEXT_DIRTY_SET.with(|s| s.borrow_mut().clear());
    INTERACTION_DIRTY_SET.with(|s| s.borrow_mut().clear());
}
