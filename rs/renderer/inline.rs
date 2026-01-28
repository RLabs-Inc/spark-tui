//! Inline renderer for normal terminal mode.
//!
//! Unlike DiffRenderer which uses the alternate screen buffer,
//! InlineRenderer writes to the normal terminal buffer. It:
//!
//! - Clears and rewrites the entire content each frame
//! - Respects terminal scrollback
//! - Suitable for CLI tools that want to show updating content
//!   without taking over the full screen

use std::io;

use super::ansi;
use super::buffer::FrameBuffer;
use super::output::{OutputBuffer, StatefulCellRenderer};

/// Inline renderer for normal terminal mode.
///
/// Renders content inline (not fullscreen). Each render clears
/// the previous content and writes new content.
pub struct InlineRenderer {
    output: OutputBuffer,
    cell_renderer: StatefulCellRenderer,
    previous_height: u16,
}

impl InlineRenderer {
    /// Create a new inline renderer.
    pub fn new() -> Self {
        Self {
            output: OutputBuffer::new(),
            cell_renderer: StatefulCellRenderer::new(),
            previous_height: 0,
        }
    }

    /// Render a frame inline.
    ///
    /// Clears previous content and writes new content.
    pub fn render(&mut self, buffer: &FrameBuffer) -> io::Result<()> {
        // Begin synchronized output
        ansi::begin_sync(&mut self.output)?;

        // Erase previous content by moving up and clearing
        if self.previous_height > 0 {
            ansi::cursor_up(&mut self.output, self.previous_height)?;
            ansi::cursor_column_zero(&mut self.output)?;
            ansi::erase_down(&mut self.output)?;
        }

        // Reset renderer state
        self.cell_renderer.reset();

        // Render all cells line by line
        let width = buffer.width();
        let height = buffer.height();

        for y in 0..height {
            for x in 0..width {
                if let Some(cell) = buffer.get(x, y) {
                    self.cell_renderer.render_cell_inline(&mut self.output, cell);
                }
            }
            // Newline after each row (except last)
            if y < height - 1 {
                self.output.write_char('\n');
            }
        }

        // Reset attributes at end
        ansi::reset(&mut self.output)?;

        // End synchronized output
        ansi::end_sync(&mut self.output)?;

        // Flush to terminal
        self.output.flush_stdout()?;

        // Track height for next erase
        self.previous_height = height;

        Ok(())
    }

    /// Clear any rendered content and reset state.
    pub fn clear(&mut self) -> io::Result<()> {
        if self.previous_height > 0 {
            ansi::cursor_up(&mut self.output, self.previous_height)?;
            ansi::cursor_column_zero(&mut self.output)?;
            ansi::erase_down(&mut self.output)?;
            self.output.flush_stdout()?;
            self.previous_height = 0;
        }
        Ok(())
    }

    /// Get the height of the previously rendered content.
    pub fn previous_height(&self) -> u16 {
        self.previous_height
    }

    /// Reset the renderer state.
    pub fn reset(&mut self) {
        self.previous_height = 0;
        self.cell_renderer.reset();
    }
}

impl Default for InlineRenderer {
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
    fn test_inline_renderer_creation() {
        let renderer = InlineRenderer::new();
        assert_eq!(renderer.previous_height(), 0);
    }

    #[test]
    fn test_inline_renderer_reset() {
        let mut renderer = InlineRenderer::new();
        renderer.previous_height = 10;
        renderer.reset();
        assert_eq!(renderer.previous_height(), 0);
    }
}
