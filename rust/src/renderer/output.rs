//! Output buffering and stateful cell rendering.
//!
//! These components optimize terminal output by:
//! - Batching writes into a single syscall
//! - Tracking terminal state to avoid redundant escape codes
//! - Only emitting changes (colors, attributes, cursor position)

use crate::types::{Attr, Cell, Rgba};
use std::io::{self, Write};

use super::ansi;

// =============================================================================
// OutputBuffer
// =============================================================================

/// A buffer that accumulates output for batch writing.
///
/// Instead of many small writes to stdout, we accumulate everything
/// and flush once. This reduces syscall overhead significantly.
#[derive(Debug, Default)]
pub struct OutputBuffer {
    data: Vec<u8>,
}

impl OutputBuffer {
    /// Create a new output buffer with default capacity.
    pub fn new() -> Self {
        Self::with_capacity(16384) // 16KB default
    }

    /// Create a buffer with specific capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    /// Get current buffer length.
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if buffer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Clear the buffer without deallocating.
    #[inline]
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Write bytes directly.
    #[inline]
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    /// Write a string.
    #[inline]
    pub fn write_str(&mut self, s: &str) {
        self.data.extend_from_slice(s.as_bytes());
    }

    /// Write a single character.
    #[inline]
    pub fn write_char(&mut self, c: char) {
        let mut buf = [0u8; 4];
        let s = c.encode_utf8(&mut buf);
        self.data.extend_from_slice(s.as_bytes());
    }

    /// Write a unicode codepoint.
    #[inline]
    pub fn write_codepoint(&mut self, cp: u32) {
        if let Some(c) = char::from_u32(cp) {
            self.write_char(c);
        }
    }

    /// Flush buffer to stdout (blocking).
    pub fn flush_stdout(&mut self) -> io::Result<()> {
        if self.data.is_empty() {
            return Ok(());
        }
        let mut stdout = io::stdout().lock();
        stdout.write_all(&self.data)?;
        stdout.flush()?;
        self.data.clear();
        Ok(())
    }

    /// Flush buffer to a writer.
    pub fn flush_to<W: Write>(&mut self, writer: &mut W) -> io::Result<()> {
        if self.data.is_empty() {
            return Ok(());
        }
        writer.write_all(&self.data)?;
        self.data.clear();
        Ok(())
    }

    /// Get the accumulated data as bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Get the accumulated data as a string (lossy).
    pub fn as_str(&self) -> std::borrow::Cow<'_, str> {
        String::from_utf8_lossy(&self.data)
    }
}

impl Write for OutputBuffer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.data.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(()) // Buffering only - real flush via flush_stdout
    }
}

// =============================================================================
// StatefulCellRenderer
// =============================================================================

/// Renders cells while tracking terminal state to minimize output.
///
/// This is the core optimization engine. It tracks:
/// - Last cursor position (to skip redundant moves)
/// - Last foreground color
/// - Last background color
/// - Last text attributes
///
/// When rendering a cell, it only emits escape codes for changed state.
#[derive(Debug)]
pub struct StatefulCellRenderer {
    last_x: i32,
    last_y: i32,
    last_fg: Option<Rgba>,
    last_bg: Option<Rgba>,
    last_attrs: Attr,
}

impl StatefulCellRenderer {
    /// Create a new renderer with no state.
    pub fn new() -> Self {
        Self {
            last_x: -1,
            last_y: -1,
            last_fg: None,
            last_bg: None,
            last_attrs: Attr::NONE,
        }
    }

    /// Reset all tracked state.
    ///
    /// Call this at the start of each frame to ensure clean state.
    pub fn reset(&mut self) {
        self.last_x = -1;
        self.last_y = -1;
        self.last_fg = None;
        self.last_bg = None;
        self.last_attrs = Attr::NONE;
    }

    /// Render a single cell to the output buffer.
    ///
    /// Only emits escape codes for state that has changed.
    pub fn render_cell(&mut self, output: &mut OutputBuffer, x: u16, y: u16, cell: &Cell) {
        // Skip continuation cells (wide character placeholders)
        if cell.char == 0 {
            // Update position tracking but don't output
            self.last_x = x as i32;
            self.last_y = y as i32;
            return;
        }

        // 1. Cursor movement (only if not sequential)
        if y as i32 != self.last_y || x as i32 != self.last_x + 1 {
            ansi::cursor_to(output, x, y).ok();
        }

        // 2. Attributes (reset if changed, then apply new)
        if cell.attrs != self.last_attrs {
            // Reset all then apply new attrs
            ansi::reset(output).ok();
            if !cell.attrs.is_empty() {
                ansi::attrs(output, cell.attrs).ok();
            }
            // Force color re-emit after reset
            self.last_fg = None;
            self.last_bg = None;
            self.last_attrs = cell.attrs;
        }

        // 3. Foreground color
        if self.last_fg.map_or(true, |c| c != cell.fg) {
            ansi::fg(output, cell.fg).ok();
            self.last_fg = Some(cell.fg);
        }

        // 4. Background color
        if self.last_bg.map_or(true, |c| c != cell.bg) {
            ansi::bg(output, cell.bg).ok();
            self.last_bg = Some(cell.bg);
        }

        // 5. Output the character
        output.write_codepoint(cell.char);

        // Update position
        self.last_x = x as i32;
        self.last_y = y as i32;
    }

    /// Render a cell for inline mode (always outputs, no cursor positioning).
    ///
    /// Used by InlineRenderer where we write sequentially with newlines.
    pub fn render_cell_inline(&mut self, output: &mut OutputBuffer, cell: &Cell) {
        // Skip continuation cells - output space for grid alignment
        if cell.char == 0 {
            output.write_char(' ');
            return;
        }

        // Attributes
        if cell.attrs != self.last_attrs {
            ansi::reset(output).ok();
            if !cell.attrs.is_empty() {
                ansi::attrs(output, cell.attrs).ok();
            }
            self.last_fg = None;
            self.last_bg = None;
            self.last_attrs = cell.attrs;
        }

        // Colors
        if self.last_fg.map_or(true, |c| c != cell.fg) {
            ansi::fg(output, cell.fg).ok();
            self.last_fg = Some(cell.fg);
        }
        if self.last_bg.map_or(true, |c| c != cell.bg) {
            ansi::bg(output, cell.bg).ok();
            self.last_bg = Some(cell.bg);
        }

        // Character
        output.write_codepoint(cell.char);
    }
}

impl Default for StatefulCellRenderer {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_buffer_write() {
        let mut buf = OutputBuffer::new();
        buf.write_str("hello");
        buf.write_char(' ');
        buf.write_str("world");
        assert_eq!(buf.as_str().as_ref(), "hello world");
    }

    #[test]
    fn test_output_buffer_clear() {
        let mut buf = OutputBuffer::new();
        buf.write_str("test");
        assert!(!buf.is_empty());
        buf.clear();
        assert!(buf.is_empty());
    }

    #[test]
    fn test_stateful_renderer_skips_sequential() {
        let mut renderer = StatefulCellRenderer::new();
        let mut output = OutputBuffer::new();

        let cell = Cell {
            char: 'A' as u32,
            fg: Rgba::WHITE,
            bg: Rgba::BLACK,
            attrs: Attr::NONE,
        };

        // First cell at (0, 0) - needs cursor move
        renderer.render_cell(&mut output, 0, 0, &cell);
        let first_len = output.len();

        // Second cell at (1, 0) - should skip cursor move
        output.clear();
        renderer.render_cell(&mut output, 1, 0, &cell);
        let second_len = output.len();

        // Second should be shorter (no cursor move, no color change)
        assert!(second_len < first_len, "Sequential cell should skip cursor move");
    }

    #[test]
    fn test_stateful_renderer_skips_same_colors() {
        let mut renderer = StatefulCellRenderer::new();
        let mut output = OutputBuffer::new();

        let cell = Cell {
            char: 'X' as u32,
            fg: Rgba::rgb(255, 0, 0),
            bg: Rgba::rgb(0, 0, 255),
            attrs: Attr::NONE,
        };

        // First cell
        renderer.render_cell(&mut output, 0, 0, &cell);
        let first_len = output.len();

        // Third cell at (5, 0) with same colors - needs cursor but not colors
        output.clear();
        renderer.render_cell(&mut output, 5, 0, &cell);
        let third_len = output.len();

        // Should only have cursor move + character, not colors
        assert!(third_len < first_len);
    }

    #[test]
    fn test_continuation_cell_skipped() {
        let mut renderer = StatefulCellRenderer::new();
        let mut output = OutputBuffer::new();

        let continuation = Cell {
            char: 0, // Continuation marker
            fg: Rgba::WHITE,
            bg: Rgba::BLACK,
            attrs: Attr::NONE,
        };

        renderer.render_cell(&mut output, 0, 0, &continuation);

        // Should output nothing for continuation cells
        assert!(output.is_empty());
    }
}
