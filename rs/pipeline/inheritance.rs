//! Color and Style Inheritance
//!
//! Components can inherit colors from their parents. This module provides
//! utilities for walking up the component tree to find inherited values.

use crate::engine::arrays::{core, visual};
use crate::types::Rgba;

/// Get the effective foreground color for a component, walking up the parent chain.
///
/// Returns the first non-terminal-default fg color found, or TERMINAL_DEFAULT if none.
pub fn get_inherited_fg(index: usize) -> Rgba {
    let mut current = Some(index);

    while let Some(idx) = current {
        let fg = visual::get_fg_color(idx);
        if !fg.is_terminal_default() {
            return fg;
        }
        current = core::get_parent_index(idx);
    }

    Rgba::TERMINAL_DEFAULT
}

/// Get the effective background color for a component, walking up the parent chain.
///
/// Returns the first non-terminal-default bg color found, or TERMINAL_DEFAULT if none.
pub fn get_inherited_bg(index: usize) -> Rgba {
    let mut current = Some(index);

    while let Some(idx) = current {
        let bg = visual::get_bg_color(idx);
        if !bg.is_terminal_default() {
            return bg;
        }
        current = core::get_parent_index(idx);
    }

    Rgba::TERMINAL_DEFAULT
}

/// Get the effective opacity for a component, multiplying up the parent chain.
///
/// Returns the product of all opacities from the component to the root.
/// Opacity is stored as u8 (0-255) but returned as f32 (0.0-1.0).
pub fn get_effective_opacity(index: usize) -> f32 {
    let mut opacity = 1.0f32;
    let mut current = Some(index);

    while let Some(idx) = current {
        let op = visual::get_opacity(idx);
        // Convert u8 (0-255) to f32 (0.0-1.0)
        opacity *= (op as f32) / 255.0;
        current = core::get_parent_index(idx);
    }

    opacity.clamp(0.0, 1.0)
}

/// Apply opacity to a color's alpha channel.
pub fn apply_opacity(color: Rgba, opacity: f32) -> Rgba {
    if opacity >= 1.0 {
        return color;
    }
    if color.is_terminal_default() {
        return color;
    }

    Rgba::new(
        color.r as u8,
        color.g as u8,
        color.b as u8,
        (color.a as f32 * opacity).round() as u8,
    )
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{allocate_index, reset_registry};
    use crate::engine::arrays::core as core_arrays;
    use crate::types::ComponentType;

    fn setup() {
        reset_registry();
    }

    #[test]
    fn test_inherited_fg_from_self() {
        setup();

        let idx = allocate_index(None);
        core_arrays::set_component_type(idx, ComponentType::Box);
        visual::set_fg_color(idx, Rgba::RED);

        assert_eq!(get_inherited_fg(idx), Rgba::RED);
    }

    #[test]
    fn test_inherited_fg_from_parent() {
        setup();

        let parent = allocate_index(None);
        core_arrays::set_component_type(parent, ComponentType::Box);
        visual::set_fg_color(parent, Rgba::GREEN);

        let child = allocate_index(None);
        core_arrays::set_component_type(child, ComponentType::Text);
        core_arrays::set_parent_index(child, Some(parent));
        // Child has no fg set (terminal default)

        assert_eq!(get_inherited_fg(child), Rgba::GREEN);
    }

    #[test]
    fn test_inherited_fg_default() {
        setup();

        let idx = allocate_index(None);
        core_arrays::set_component_type(idx, ComponentType::Box);
        // No fg color set

        assert_eq!(get_inherited_fg(idx), Rgba::TERMINAL_DEFAULT);
    }

    #[test]
    fn test_effective_opacity() {
        setup();

        let parent = allocate_index(None);
        core_arrays::set_component_type(parent, ComponentType::Box);
        visual::set_opacity(parent, 128);  // 50% as u8

        let child = allocate_index(None);
        core_arrays::set_component_type(child, ComponentType::Box);
        core_arrays::set_parent_index(child, Some(parent));
        visual::set_opacity(child, 128);  // 50% as u8

        // (128/255) * (128/255) ≈ 0.25
        assert!((get_effective_opacity(child) - 0.25).abs() < 0.02);
    }

    #[test]
    fn test_apply_opacity() {
        let color = Rgba::new(255, 0, 0, 255);
        let result = apply_opacity(color, 0.5);
        assert_eq!(result.a, 128);

        // Terminal default shouldn't change
        let default = Rgba::TERMINAL_DEFAULT;
        let result2 = apply_opacity(default, 0.5);
        assert_eq!(result2, Rgba::TERMINAL_DEFAULT);
    }
}
