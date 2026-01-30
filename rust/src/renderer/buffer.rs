//! FrameBuffer and drawing primitives.
//!
//! The FrameBuffer is a 2D grid of Cells that represents what should be displayed
//! on the terminal. All drawing operations work on this buffer.
//!
//! # Design Decisions
//!
//! - **Flat storage**: Uses `Vec<Cell>` with row-major indexing for cache efficiency.
//! - **Clipping**: All drawing functions accept an optional `ClipRect` for overflow:hidden.
//! - **Alpha blending**: Transparent backgrounds blend with existing cells.
//! - **Wide characters**: Emoji and CJK characters use continuation markers.

use crate::types::{Attr, BorderStyle, Cell, ClipRect, Rgba};

// =============================================================================
// FrameBuffer
// =============================================================================

/// A 2D buffer of terminal cells.
///
/// Uses flat storage with row-major indexing: `index = y * width + x`
#[derive(Debug, Clone, PartialEq)]
pub struct FrameBuffer {
    width: u16,
    height: u16,
    cells: Vec<Cell>,
}

impl FrameBuffer {
    /// Create a new buffer filled with default cells.
    pub fn new(width: u16, height: u16) -> Self {
        let size = width as usize * height as usize;
        Self {
            width,
            height,
            cells: vec![Cell::default(); size],
        }
    }

    /// Create a new buffer with a specific background color.
    pub fn with_background(width: u16, height: u16, bg: Rgba) -> Self {
        let size = width as usize * height as usize;
        let cell = Cell {
            char: b' ' as u32,
            fg: Rgba::TERMINAL_DEFAULT,
            bg,
            attrs: Attr::NONE,
        };
        Self {
            width,
            height,
            cells: vec![cell; size],
        }
    }

    /// Get buffer width.
    #[inline]
    pub fn width(&self) -> u16 {
        self.width
    }

    /// Get buffer height.
    #[inline]
    pub fn height(&self) -> u16 {
        self.height
    }

    /// Get the full buffer bounds as a ClipRect.
    #[inline]
    pub fn bounds(&self) -> ClipRect {
        ClipRect::new(0, 0, self.width, self.height)
    }

    /// Convert (x, y) to flat index.
    #[inline]
    fn index(&self, x: u16, y: u16) -> usize {
        y as usize * self.width as usize + x as usize
    }

    /// Check if coordinates are in bounds.
    #[inline]
    pub fn in_bounds(&self, x: u16, y: u16) -> bool {
        x < self.width && y < self.height
    }

    /// Get a cell reference (returns None if out of bounds).
    #[inline]
    pub fn get(&self, x: u16, y: u16) -> Option<&Cell> {
        if self.in_bounds(x, y) {
            Some(&self.cells[self.index(x, y)])
        } else {
            None
        }
    }

    /// Get a mutable cell reference (returns None if out of bounds).
    #[inline]
    pub fn get_mut(&mut self, x: u16, y: u16) -> Option<&mut Cell> {
        if self.in_bounds(x, y) {
            let idx = self.index(x, y);
            Some(&mut self.cells[idx])
        } else {
            None
        }
    }

    /// Get raw cells slice (for iteration during rendering).
    #[inline]
    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    /// Get cell at index (for diff rendering).
    #[inline]
    pub fn cell_at_index(&self, index: usize) -> Option<&Cell> {
        self.cells.get(index)
    }

    /// Iterate over cells with their coordinates.
    pub fn iter(&self) -> impl Iterator<Item = (u16, u16, &Cell)> {
        self.cells.iter().enumerate().map(move |(i, cell)| {
            let x = (i % self.width as usize) as u16;
            let y = (i / self.width as usize) as u16;
            (x, y, cell)
        })
    }

    /// Clear the entire buffer to default cells.
    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            *cell = Cell::default();
        }
    }

    /// Clear with a specific background color.
    pub fn clear_with_bg(&mut self, bg: Rgba) {
        for cell in &mut self.cells {
            cell.char = b' ' as u32;
            cell.fg = Rgba::TERMINAL_DEFAULT;
            cell.bg = bg;
            cell.attrs = Attr::NONE;
        }
    }

    /// Resize the buffer (clears content).
    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        let size = width as usize * height as usize;
        self.cells.resize(size, Cell::default());
        self.clear();
    }

    // =========================================================================
    // Drawing Primitives
    // =========================================================================

    /// Set a single cell with optional clipping.
    ///
    /// Returns true if the cell was set.
    pub fn set_cell(
        &mut self,
        x: u16,
        y: u16,
        char: u32,
        fg: Rgba,
        bg: Rgba,
        attrs: Attr,
        clip: Option<&ClipRect>,
    ) -> bool {
        // Bounds check
        if !self.in_bounds(x, y) {
            return false;
        }

        // Clip check
        if let Some(clip) = clip {
            if !clip.contains(x, y) {
                return false;
            }
        }

        let idx = self.index(x, y);
        let cell = &mut self.cells[idx];

        // Alpha blend background if not opaque
        let blended_bg = if bg.is_opaque() || bg.is_terminal_default() || bg.is_ansi() {
            bg
        } else {
            Rgba::blend(bg, cell.bg)
        };

        cell.char = char;
        cell.fg = fg;
        cell.bg = blended_bg;
        cell.attrs = attrs;

        true
    }

    /// Fill a rectangle with a background color.
    pub fn fill_rect(&mut self, x: u16, y: u16, width: u16, height: u16, bg: Rgba, clip: Option<&ClipRect>) {
        // Compute effective bounds
        let x1 = x;
        let y1 = y;
        let x2 = x.saturating_add(width).min(self.width);
        let y2 = y.saturating_add(height).min(self.height);

        // Apply clipping
        let (x1, y1, x2, y2) = if let Some(clip) = clip {
            let cx2 = clip.x.saturating_add(clip.width);
            let cy2 = clip.y.saturating_add(clip.height);
            (x1.max(clip.x), y1.max(clip.y), x2.min(cx2), y2.min(cy2))
        } else {
            (x1, y1, x2, y2)
        };

        if x2 <= x1 || y2 <= y1 {
            return;
        }

        // Fast path for opaque fill
        let is_opaque = bg.is_opaque() || bg.is_terminal_default() || bg.is_ansi();

        for row in y1..y2 {
            let row_start = self.index(x1, row);
            let row_end = self.index(x2, row);
            for cell in &mut self.cells[row_start..row_end] {
                if is_opaque {
                    cell.bg = bg;
                } else {
                    cell.bg = Rgba::blend(bg, cell.bg);
                }
                cell.char = b' ' as u32;
                cell.attrs = Attr::NONE;
            }
        }
    }

    /// Draw a single character.
    pub fn draw_char(
        &mut self,
        x: u16,
        y: u16,
        char: char,
        fg: Rgba,
        bg: Option<Rgba>,
        attrs: Attr,
        clip: Option<&ClipRect>,
    ) -> bool {
        let bg = bg.unwrap_or(Rgba::TRANSPARENT);
        self.set_cell(x, y, char as u32, fg, bg, attrs, clip)
    }

    /// Draw text at a position.
    ///
    /// Returns the number of cells used (handles wide characters).
    pub fn draw_text(
        &mut self,
        x: u16,
        y: u16,
        text: &str,
        fg: Rgba,
        bg: Option<Rgba>,
        attrs: Attr,
        clip: Option<&ClipRect>,
    ) -> u16 {
        let bg = bg.unwrap_or(Rgba::TRANSPARENT);
        let mut col = x;

        for ch in text.chars() {
            if col >= self.width {
                break;
            }

            let char_width = char_width(ch);

            if char_width == 0 {
                continue; // Skip zero-width characters
            }

            // Draw main character
            if self.set_cell(col, y, ch as u32, fg, bg, attrs, clip) {
                // Handle wide characters (emoji, CJK)
                if char_width == 2 && col + 1 < self.width {
                    // Mark next cell as continuation (char = 0)
                    if let Some(next) = self.get_mut(col + 1, y) {
                        if clip.map_or(true, |c| c.contains(col + 1, y)) {
                            next.char = 0; // Continuation marker
                            next.fg = fg;
                            if !bg.is_transparent() {
                                next.bg = Rgba::blend(bg, next.bg);
                            }
                            next.attrs = attrs;
                        }
                    }
                }
            }

            col += char_width as u16;
        }

        col.saturating_sub(x)
    }

    /// Draw text centered within a width.
    pub fn draw_text_centered(
        &mut self,
        x: u16,
        y: u16,
        width: u16,
        text: &str,
        fg: Rgba,
        bg: Option<Rgba>,
        attrs: Attr,
        clip: Option<&ClipRect>,
    ) -> u16 {
        let text_width = string_width(text);
        if text_width >= width as usize {
            return self.draw_text(x, y, text, fg, bg, attrs, clip);
        }
        let offset = ((width as usize - text_width) / 2) as u16;
        self.draw_text(x + offset, y, text, fg, bg, attrs, clip)
    }

    /// Draw text right-aligned within a width.
    pub fn draw_text_right(
        &mut self,
        x: u16,
        y: u16,
        width: u16,
        text: &str,
        fg: Rgba,
        bg: Option<Rgba>,
        attrs: Attr,
        clip: Option<&ClipRect>,
    ) -> u16 {
        let text_width = string_width(text);
        if text_width >= width as usize {
            return self.draw_text(x, y, text, fg, bg, attrs, clip);
        }
        let offset = (width as usize - text_width) as u16;
        self.draw_text(x + offset, y, text, fg, bg, attrs, clip)
    }

    /// Draw a border around a rectangle.
    pub fn draw_border(
        &mut self,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        style: BorderStyle,
        color: Rgba,
        bg: Option<Rgba>,
        clip: Option<&ClipRect>,
    ) {
        if width < 2 || height < 2 || style == BorderStyle::None {
            return;
        }

        let (horiz, vert, tl, tr, br, bl) = style.chars();
        let bg = bg.unwrap_or(Rgba::TRANSPARENT);

        let x2 = x + width - 1;
        let y2 = y + height - 1;

        // Draw corners
        self.draw_char(x, y, tl.chars().next().unwrap(), color, Some(bg), Attr::NONE, clip);
        self.draw_char(x2, y, tr.chars().next().unwrap(), color, Some(bg), Attr::NONE, clip);
        self.draw_char(x2, y2, br.chars().next().unwrap(), color, Some(bg), Attr::NONE, clip);
        self.draw_char(x, y2, bl.chars().next().unwrap(), color, Some(bg), Attr::NONE, clip);

        let horiz_char = horiz.chars().next().unwrap();
        let vert_char = vert.chars().next().unwrap();

        // Draw horizontal edges
        for col in (x + 1)..x2 {
            self.draw_char(col, y, horiz_char, color, Some(bg), Attr::NONE, clip);
            self.draw_char(col, y2, horiz_char, color, Some(bg), Attr::NONE, clip);
        }

        // Draw vertical edges
        for row in (y + 1)..y2 {
            self.draw_char(x, row, vert_char, color, Some(bg), Attr::NONE, clip);
            self.draw_char(x2, row, vert_char, color, Some(bg), Attr::NONE, clip);
        }
    }

    /// Draw a border with per-side styles.
    pub fn draw_border_sides(
        &mut self,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        styles: BorderSides,
        colors: BorderColors,
        bg: Option<Rgba>,
        clip: Option<&ClipRect>,
    ) {
        if width < 2 || height < 2 {
            return;
        }

        let bg = bg.unwrap_or(Rgba::TRANSPARENT);
        let x2 = x + width - 1;
        let y2 = y + height - 1;

        // Get characters for each side
        let top_chars = styles.top.chars();
        let bottom_chars = styles.bottom.chars();
        let left_chars = styles.left.chars();
        let right_chars = styles.right.chars();

        // Draw top edge
        if styles.top != BorderStyle::None {
            let horiz = top_chars.0.chars().next().unwrap();
            for col in (x + 1)..x2 {
                self.draw_char(col, y, horiz, colors.top, Some(bg), Attr::NONE, clip);
            }
        }

        // Draw bottom edge
        if styles.bottom != BorderStyle::None {
            let horiz = bottom_chars.0.chars().next().unwrap();
            for col in (x + 1)..x2 {
                self.draw_char(col, y2, horiz, colors.bottom, Some(bg), Attr::NONE, clip);
            }
        }

        // Draw left edge
        if styles.left != BorderStyle::None {
            let vert = left_chars.1.chars().next().unwrap();
            for row in (y + 1)..y2 {
                self.draw_char(x, row, vert, colors.left, Some(bg), Attr::NONE, clip);
            }
        }

        // Draw right edge
        if styles.right != BorderStyle::None {
            let vert = right_chars.1.chars().next().unwrap();
            for row in (y + 1)..y2 {
                self.draw_char(x2, row, vert, colors.right, Some(bg), Attr::NONE, clip);
            }
        }

        // Draw corners (use top style preference, then left)
        let corner_style = if styles.top != BorderStyle::None {
            styles.top
        } else if styles.left != BorderStyle::None {
            styles.left
        } else {
            BorderStyle::None
        };

        if corner_style != BorderStyle::None {
            let (_, _, tl, tr, br, bl) = corner_style.chars();

            if styles.top != BorderStyle::None || styles.left != BorderStyle::None {
                self.draw_char(x, y, tl.chars().next().unwrap(), colors.top, Some(bg), Attr::NONE, clip);
            }
            if styles.top != BorderStyle::None || styles.right != BorderStyle::None {
                self.draw_char(x2, y, tr.chars().next().unwrap(), colors.top, Some(bg), Attr::NONE, clip);
            }
            if styles.bottom != BorderStyle::None || styles.right != BorderStyle::None {
                self.draw_char(x2, y2, br.chars().next().unwrap(), colors.bottom, Some(bg), Attr::NONE, clip);
            }
            if styles.bottom != BorderStyle::None || styles.left != BorderStyle::None {
                self.draw_char(x, y2, bl.chars().next().unwrap(), colors.bottom, Some(bg), Attr::NONE, clip);
            }
        }
    }

    /// Draw a horizontal line.
    pub fn draw_hline(
        &mut self,
        x: u16,
        y: u16,
        length: u16,
        char: char,
        fg: Rgba,
        bg: Option<Rgba>,
        clip: Option<&ClipRect>,
    ) {
        for col in x..x.saturating_add(length).min(self.width) {
            self.draw_char(col, y, char, fg, bg, Attr::NONE, clip);
        }
    }

    /// Draw a vertical line.
    pub fn draw_vline(
        &mut self,
        x: u16,
        y: u16,
        length: u16,
        char: char,
        fg: Rgba,
        bg: Option<Rgba>,
        clip: Option<&ClipRect>,
    ) {
        for row in y..y.saturating_add(length).min(self.height) {
            self.draw_char(x, row, char, fg, bg, Attr::NONE, clip);
        }
    }

    /// Draw a progress bar.
    pub fn draw_progress(
        &mut self,
        x: u16,
        y: u16,
        width: u16,
        progress: f32,
        filled_char: char,
        empty_char: char,
        filled_fg: Rgba,
        empty_fg: Rgba,
        bg: Option<Rgba>,
        clip: Option<&ClipRect>,
    ) {
        let progress = progress.clamp(0.0, 1.0);
        let filled = (progress * width as f32).round() as u16;

        for col in 0..width {
            let actual_x = x + col;
            if col < filled {
                self.draw_char(actual_x, y, filled_char, filled_fg, bg, Attr::NONE, clip);
            } else {
                self.draw_char(actual_x, y, empty_char, empty_fg, bg, Attr::NONE, clip);
            }
        }
    }

    /// Draw a vertical scrollbar.
    pub fn draw_scrollbar_v(
        &mut self,
        x: u16,
        y: u16,
        height: u16,
        scroll_position: f32,
        viewport_ratio: f32,
        track_fg: Rgba,
        thumb_fg: Rgba,
        bg: Option<Rgba>,
        clip: Option<&ClipRect>,
    ) {
        if height == 0 {
            return;
        }

        let thumb_height = (height as f32 * viewport_ratio).max(1.0) as u16;
        let track_space = height.saturating_sub(thumb_height);
        let thumb_start = (track_space as f32 * scroll_position.clamp(0.0, 1.0)) as u16;

        for row in 0..height {
            let actual_y = y + row;
            let is_thumb = row >= thumb_start && row < thumb_start + thumb_height;
            let (char, fg) = if is_thumb {
                ('█', thumb_fg)
            } else {
                ('░', track_fg)
            };
            self.draw_char(x, actual_y, char, fg, bg, Attr::NONE, clip);
        }
    }

    /// Draw a horizontal scrollbar.
    pub fn draw_scrollbar_h(
        &mut self,
        x: u16,
        y: u16,
        width: u16,
        scroll_position: f32,
        viewport_ratio: f32,
        track_fg: Rgba,
        thumb_fg: Rgba,
        bg: Option<Rgba>,
        clip: Option<&ClipRect>,
    ) {
        if width == 0 {
            return;
        }

        let thumb_width = (width as f32 * viewport_ratio).max(1.0) as u16;
        let track_space = width.saturating_sub(thumb_width);
        let thumb_start = (track_space as f32 * scroll_position.clamp(0.0, 1.0)) as u16;

        for col in 0..width {
            let actual_x = x + col;
            let is_thumb = col >= thumb_start && col < thumb_start + thumb_width;
            let (char, fg) = if is_thumb {
                ('█', thumb_fg)
            } else {
                ('░', track_fg)
            };
            self.draw_char(actual_x, y, char, fg, bg, Attr::NONE, clip);
        }
    }
}

// =============================================================================
// Border Configuration
// =============================================================================

/// Border styles for each side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BorderSides {
    pub top: BorderStyle,
    pub right: BorderStyle,
    pub bottom: BorderStyle,
    pub left: BorderStyle,
}

impl BorderSides {
    /// All sides with the same style.
    pub const fn all(style: BorderStyle) -> Self {
        Self {
            top: style,
            right: style,
            bottom: style,
            left: style,
        }
    }

    /// No borders.
    pub const NONE: Self = Self::all(BorderStyle::None);
}

impl Default for BorderSides {
    fn default() -> Self {
        Self::NONE
    }
}

/// Border colors for each side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BorderColors {
    pub top: Rgba,
    pub right: Rgba,
    pub bottom: Rgba,
    pub left: Rgba,
}

impl BorderColors {
    /// All sides with the same color.
    pub const fn all(color: Rgba) -> Self {
        Self {
            top: color,
            right: color,
            bottom: color,
            left: color,
        }
    }
}

impl Default for BorderColors {
    fn default() -> Self {
        Self::all(Rgba::TERMINAL_DEFAULT)
    }
}

// =============================================================================
// Text Width Utilities
// =============================================================================

/// Get the display width of a character.
///
/// - ASCII printable: 1
/// - Control characters: 0
/// - Wide characters (CJK, emoji): 2
pub fn char_width(c: char) -> usize {
    let cp = c as u32;

    // Control characters
    if cp < 32 || (0x7F..=0x9F).contains(&cp) {
        return 0;
    }

    // ASCII printable
    if cp < 127 {
        return 1;
    }

    // Wide characters (simplified heuristic)
    // Full ranges would need unicode-width crate for accuracy
    if is_wide_char(cp) {
        return 2;
    }

    1
}

/// Check if a codepoint is typically double-width.
fn is_wide_char(cp: u32) -> bool {
    // CJK ranges
    (0x1100..=0x115F).contains(&cp)   // Hangul Jamo
        || (0x2E80..=0x9FFF).contains(&cp)   // CJK
        || (0xAC00..=0xD7A3).contains(&cp)   // Hangul Syllables
        || (0xF900..=0xFAFF).contains(&cp)   // CJK Compatibility Ideographs
        || (0xFE10..=0xFE1F).contains(&cp)   // Vertical forms
        || (0xFE30..=0xFE6F).contains(&cp)   // CJK Compatibility Forms
        || (0xFF00..=0xFF60).contains(&cp)   // Fullwidth forms
        || (0xFFE0..=0xFFE6).contains(&cp)   // Fullwidth symbols
        || (0x20000..=0x2FFFF).contains(&cp) // CJK Extension B+
        || (0x30000..=0x3FFFF).contains(&cp) // CJK Extension G+
        // Symbols (includes ✨ sparkles, ⚡ zap, etc.)
        || (0x2600..=0x27BF).contains(&cp)   // Misc Symbols, Dingbats
        // Emoji (simplified - many are actually width 2)
        || (0x1F300..=0x1F9FF).contains(&cp) // Misc Symbols and Pictographs, Emoticons, etc.
        || (0x1FA00..=0x1FAFF).contains(&cp) // Chess, Extended-A
}

/// Calculate the display width of a string.
pub fn string_width(s: &str) -> usize {
    s.chars().map(char_width).sum()
}

// Note: Higher-level text utilities (truncate_text, wrap_text, measure_text_height)
// are in layout/text_measure.rs - they're layout concerns, not renderer concerns.

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clip_rect_contains() {
        let clip = ClipRect::new(10, 10, 20, 20);
        assert!(clip.contains(10, 10));
        assert!(clip.contains(29, 29));
        assert!(!clip.contains(9, 10));
        assert!(!clip.contains(30, 10));
    }

    #[test]
    fn test_clip_rect_intersect() {
        let a = ClipRect::new(0, 0, 20, 20);
        let b = ClipRect::new(10, 10, 20, 20);

        let intersect = a.intersect(&b).unwrap();
        assert_eq!(intersect.x, 10);
        assert_eq!(intersect.y, 10);
        assert_eq!(intersect.width, 10);
        assert_eq!(intersect.height, 10);

        // Non-overlapping
        let c = ClipRect::new(100, 100, 10, 10);
        assert!(a.intersect(&c).is_none());
    }

    #[test]
    fn test_framebuffer_creation() {
        let buffer = FrameBuffer::new(80, 24);
        assert_eq!(buffer.width(), 80);
        assert_eq!(buffer.height(), 24);
    }

    #[test]
    fn test_framebuffer_set_cell() {
        let mut buffer = FrameBuffer::new(10, 10);
        buffer.set_cell(5, 5, 'X' as u32, Rgba::RED, Rgba::BLACK, Attr::BOLD, None);

        let cell = buffer.get(5, 5).unwrap();
        assert_eq!(cell.char, 'X' as u32);
        assert_eq!(cell.fg, Rgba::RED);
        assert_eq!(cell.bg, Rgba::BLACK);
        assert_eq!(cell.attrs, Attr::BOLD);
    }

    #[test]
    fn test_framebuffer_fill_rect() {
        let mut buffer = FrameBuffer::new(20, 20);
        buffer.fill_rect(5, 5, 10, 10, Rgba::BLUE, None);

        // Inside
        assert_eq!(buffer.get(5, 5).unwrap().bg, Rgba::BLUE);
        assert_eq!(buffer.get(14, 14).unwrap().bg, Rgba::BLUE);

        // Outside
        assert_eq!(buffer.get(4, 5).unwrap().bg, Rgba::TERMINAL_DEFAULT);
        assert_eq!(buffer.get(15, 5).unwrap().bg, Rgba::TERMINAL_DEFAULT);
    }

    #[test]
    fn test_draw_text() {
        let mut buffer = FrameBuffer::new(20, 5);
        buffer.draw_text(0, 0, "Hello", Rgba::WHITE, None, Attr::NONE, None);

        assert_eq!(buffer.get(0, 0).unwrap().char, 'H' as u32);
        assert_eq!(buffer.get(1, 0).unwrap().char, 'e' as u32);
        assert_eq!(buffer.get(4, 0).unwrap().char, 'o' as u32);
    }

    #[test]
    fn test_char_width() {
        assert_eq!(char_width('a'), 1);
        assert_eq!(char_width(' '), 1);
        assert_eq!(char_width('\n'), 0);
        assert_eq!(char_width('中'), 2);
    }

    #[test]
    fn test_string_width() {
        assert_eq!(string_width("hello"), 5);
        assert_eq!(string_width("中文"), 4);
        assert_eq!(string_width("a中b"), 4);
    }
}
