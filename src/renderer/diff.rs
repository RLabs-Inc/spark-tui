//! Differential renderer for fullscreen mode.
//!
//! The DiffRenderer compares the current frame to the previous frame and only
//! outputs cells that have changed. This dramatically reduces terminal I/O
//! and provides smooth, flicker-free updates.
//!
//! # Algorithm
//!
//! 1. Wrap output in synchronized block (beginSync/endSync)
//! 2. For each cell in the new frame:
//!    - If previous frame exists and cell is unchanged: skip
//!    - Otherwise: render cell with StatefulCellRenderer
//! 3. Flush output buffer (single syscall)
//! 4. Store current frame as previous for next comparison

use std::io;

use super::ansi;
use super::buffer::FrameBuffer;
use super::output::{OutputBuffer, StatefulCellRenderer};
use crate::types::Cell;

/// Differential renderer for fullscreen mode.
///
/// Keeps track of the previous frame to enable diff-based rendering.
/// Only cells that have changed since the last frame are output.
pub struct DiffRenderer {
    output: OutputBuffer,
    cell_renderer: StatefulCellRenderer,
    previous: Option<FrameBuffer>,
}

impl DiffRenderer {
    /// Create a new diff renderer.
    pub fn new() -> Self {
        Self {
            output: OutputBuffer::new(),
            cell_renderer: StatefulCellRenderer::new(),
            previous: None,
        }
    }

    /// Render a frame, outputting only changed cells.
    ///
    /// Returns true if any cells were changed.
    pub fn render(&mut self, buffer: &FrameBuffer) -> io::Result<bool> {
        let mut has_changes = false;

        // Begin synchronized output
        ansi::begin_sync(&mut self.output)?;

        // Reset renderer state for new frame
        self.cell_renderer.reset();

        let width = buffer.width();
        let height = buffer.height();

        // Differential rendering
        for y in 0..height {
            for x in 0..width {
                let cell = buffer.get(x, y).unwrap();

                // Check if cell changed from previous frame
                let changed = match &self.previous {
                    Some(prev) if prev.width() == width && prev.height() == height => {
                        match prev.get(x, y) {
                            Some(prev_cell) => !cells_equal(cell, prev_cell),
                            None => true,
                        }
                    }
                    _ => true, // No previous or size changed
                };

                if changed {
                    has_changes = true;
                    self.cell_renderer.render_cell(&mut self.output, x, y, cell);
                }
            }
        }

        // End synchronized output
        ansi::end_sync(&mut self.output)?;

        // Flush to terminal
        self.output.flush_stdout()?;

        // Store for next frame comparison
        self.previous = Some(buffer.clone());

        Ok(has_changes)
    }

    /// Force a full redraw (no diffing).
    ///
    /// Use this after terminal resize or when the screen is corrupted.
    pub fn render_full(&mut self, buffer: &FrameBuffer) -> io::Result<()> {
        // Begin synchronized output
        ansi::begin_sync(&mut self.output)?;

        // Move to home position
        ansi::cursor_to(&mut self.output, 0, 0)?;

        // Reset renderer state
        self.cell_renderer.reset();

        // Render all cells
        let width = buffer.width();
        let height = buffer.height();

        for y in 0..height {
            for x in 0..width {
                if let Some(cell) = buffer.get(x, y) {
                    self.cell_renderer.render_cell(&mut self.output, x, y, cell);
                }
            }
        }

        // End synchronized output
        ansi::end_sync(&mut self.output)?;

        // Flush
        self.output.flush_stdout()?;

        // Store for next frame
        self.previous = Some(buffer.clone());

        Ok(())
    }

    /// Invalidate the previous frame.
    ///
    /// Next render will be a full redraw.
    pub fn invalidate(&mut self) {
        self.previous = None;
    }

    /// Check if we have a previous frame to diff against.
    pub fn has_previous(&self) -> bool {
        self.previous.is_some()
    }

    /// Enter fullscreen mode (alternate screen buffer).
    pub fn enter_fullscreen(&mut self) -> io::Result<()> {
        ansi::enter_alt_screen(&mut self.output)?;
        ansi::cursor_hide(&mut self.output)?;
        ansi::clear_screen(&mut self.output)?;
        self.output.flush_stdout()?;
        self.invalidate();
        Ok(())
    }

    /// Exit fullscreen mode.
    pub fn exit_fullscreen(&mut self) -> io::Result<()> {
        ansi::reset(&mut self.output)?;
        ansi::cursor_show(&mut self.output)?;
        ansi::exit_alt_screen(&mut self.output)?;
        self.output.flush_stdout()?;
        Ok(())
    }

    /// Enable mouse tracking.
    pub fn enable_mouse(&mut self) -> io::Result<()> {
        ansi::enable_mouse(&mut self.output)?;
        self.output.flush_stdout()
    }

    /// Disable mouse tracking.
    pub fn disable_mouse(&mut self) -> io::Result<()> {
        ansi::disable_mouse(&mut self.output)?;
        self.output.flush_stdout()
    }
}

impl Default for DiffRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Fast cell equality check.
#[inline]
fn cells_equal(a: &Cell, b: &Cell) -> bool {
    a.char == b.char && a.attrs == b.attrs && a.fg == b.fg && a.bg == b.bg
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Attr, Rgba};

    #[test]
    fn test_diff_renderer_creation() {
        let renderer = DiffRenderer::new();
        assert!(!renderer.has_previous());
    }

    #[test]
    fn test_cells_equal() {
        let a = Cell {
            char: 'X' as u32,
            fg: Rgba::WHITE,
            bg: Rgba::BLACK,
            attrs: Attr::BOLD,
        };
        let b = a;
        assert!(cells_equal(&a, &b));

        let c = Cell {
            char: 'Y' as u32,
            ..a
        };
        assert!(!cells_equal(&a, &c));
    }

    #[test]
    fn test_invalidate() {
        let mut renderer = DiffRenderer::new();
        let buffer = FrameBuffer::new(10, 10);

        // Can't test actual rendering without terminal, but can test state
        renderer.previous = Some(buffer);
        assert!(renderer.has_previous());

        renderer.invalidate();
        assert!(!renderer.has_previous());
    }
}
