//! Layout Types
//!
//! Output types for the layout computation.

/// Computed layout result.
///
/// Contains parallel arrays indexed by component index.
/// Each index maps to the computed position and size of that component.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ComputedLayout {
    /// X position (column) of each component.
    pub x: Vec<u16>,

    /// Y position (row) of each component.
    pub y: Vec<u16>,

    /// Width of each component.
    pub width: Vec<u16>,

    /// Height of each component.
    pub height: Vec<u16>,

    /// Whether component is scrollable (0 = no, 1 = yes).
    pub scrollable: Vec<u8>,

    /// Maximum scroll offset X for each component.
    pub max_scroll_x: Vec<u16>,

    /// Maximum scroll offset Y for each component.
    pub max_scroll_y: Vec<u16>,

    /// Total content width (rightmost edge of any component).
    pub content_width: u16,

    /// Total content height (bottommost edge of any component).
    pub content_height: u16,
}

impl ComputedLayout {
    /// Create a new empty computed layout.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the position and size of a component.
    ///
    /// Returns (x, y, width, height) or defaults if index is out of bounds.
    pub fn get(&self, index: usize) -> (u16, u16, u16, u16) {
        (
            self.x.get(index).copied().unwrap_or(0),
            self.y.get(index).copied().unwrap_or(0),
            self.width.get(index).copied().unwrap_or(0),
            self.height.get(index).copied().unwrap_or(0),
        )
    }

    /// Check if a component is scrollable.
    pub fn is_scrollable(&self, index: usize) -> bool {
        self.scrollable.get(index).copied().unwrap_or(0) != 0
    }

    /// Get scroll limits for a component.
    ///
    /// Returns (max_scroll_x, max_scroll_y).
    pub fn scroll_limits(&self, index: usize) -> (u16, u16) {
        (
            self.max_scroll_x.get(index).copied().unwrap_or(0),
            self.max_scroll_y.get(index).copied().unwrap_or(0),
        )
    }
}

/// Overflow behavior for containers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Overflow {
    /// Content can overflow container bounds (default).
    #[default]
    Visible = 0,
    /// Content is clipped at container bounds.
    Hidden = 1,
    /// Content is scrollable.
    Scroll = 2,
    /// Scroll only if content overflows.
    Auto = 3,
}

impl From<u8> for Overflow {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Visible,
            1 => Self::Hidden,
            2 => Self::Scroll,
            3 => Self::Auto,
            _ => Self::Visible,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_computed_layout_get() {
        let mut layout = ComputedLayout::new();
        layout.x.push(10);
        layout.y.push(20);
        layout.width.push(30);
        layout.height.push(40);

        let (x, y, w, h) = layout.get(0);
        assert_eq!(x, 10);
        assert_eq!(y, 20);
        assert_eq!(w, 30);
        assert_eq!(h, 40);

        // Out of bounds returns zeros
        let (x, y, w, h) = layout.get(999);
        assert_eq!(x, 0);
        assert_eq!(y, 0);
        assert_eq!(w, 0);
        assert_eq!(h, 0);
    }

    #[test]
    fn test_overflow_from_u8() {
        assert_eq!(Overflow::from(0), Overflow::Visible);
        assert_eq!(Overflow::from(1), Overflow::Hidden);
        assert_eq!(Overflow::from(2), Overflow::Scroll);
        assert_eq!(Overflow::from(3), Overflow::Auto);
        assert_eq!(Overflow::from(99), Overflow::Visible);
    }
}
