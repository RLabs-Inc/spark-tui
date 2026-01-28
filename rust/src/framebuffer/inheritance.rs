//! Color and opacity inheritance via SharedBuffer parent chain.
//!
//! Components inherit fg/bg colors from ancestors. Opacity cascades
//! (multiplies) down the tree.

use crate::shared_buffer::SharedBuffer;
use crate::types::Rgba;

/// Get effective foreground color, walking up the parent chain.
/// Returns the first non-terminal-default fg, or TERMINAL_DEFAULT if none.
pub fn get_inherited_fg(buf: &SharedBuffer, node: usize) -> Rgba {
    let mut current = Some(node);
    while let Some(idx) = current {
        let fg = buf.fg_rgba(idx);
        if !fg.is_terminal_default() {
            return fg;
        }
        current = buf.parent_index(idx);
    }
    Rgba::TERMINAL_DEFAULT
}

/// Get effective background color, walking up the parent chain.
/// Returns the first non-terminal-default bg, or TERMINAL_DEFAULT if none.
pub fn get_inherited_bg(buf: &SharedBuffer, node: usize) -> Rgba {
    let mut current = Some(node);
    while let Some(idx) = current {
        let bg = buf.bg_rgba(idx);
        if !bg.is_terminal_default() {
            return bg;
        }
        current = buf.parent_index(idx);
    }
    Rgba::TERMINAL_DEFAULT
}

/// Get effective opacity, multiplying up the parent chain.
/// Opacity stored as u8 (0-255), returned as f32 (0.0-1.0).
pub fn get_effective_opacity(buf: &SharedBuffer, node: usize) -> f32 {
    let mut opacity = 1.0f32;
    let mut current = Some(node);
    while let Some(idx) = current {
        let op = buf.opacity(idx);
        opacity *= op as f32 / 255.0;
        current = buf.parent_index(idx);
    }
    opacity.clamp(0.0, 1.0)
}

/// Apply opacity to a color's alpha channel.
pub fn apply_opacity(color: Rgba, opacity: f32) -> Rgba {
    if opacity >= 1.0 || color.is_terminal_default() {
        return color;
    }
    Rgba::new(
        color.r as u8,
        color.g as u8,
        color.b as u8,
        (color.a as f32 * opacity).round() as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_opacity_full() {
        let color = Rgba::new(255, 0, 0, 255);
        let result = apply_opacity(color, 1.0);
        assert_eq!(result.a, 255);
    }

    #[test]
    fn test_apply_opacity_half() {
        let color = Rgba::new(255, 0, 0, 255);
        let result = apply_opacity(color, 0.5);
        assert_eq!(result.a, 128);
    }

    #[test]
    fn test_apply_opacity_terminal_default() {
        let result = apply_opacity(Rgba::TERMINAL_DEFAULT, 0.5);
        assert_eq!(result, Rgba::TERMINAL_DEFAULT);
    }
}
