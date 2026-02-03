//! Append-mode renderer for CLI applications with history.
//!
//! The AppendRegionRenderer supports a two-region model:
//!
//! 1. **History** - Frozen content that scrolls up into terminal scrollback
//! 2. **Active** - Dynamic content at the bottom that updates in place
//!
//! This is ideal for CLI tools like build systems, test runners, or
//! any application that wants to show updating status while preserving
//! a log of previous output.
//!
//! # Example Use Case
//!
//! ```text
//! ┌─────────────────────────┐
//! │ [PASS] test_foo         │  ← History (frozen, scrollable)
//! │ [PASS] test_bar         │
//! │ [FAIL] test_baz         │
//! ├─────────────────────────┤
//! │ Running: test_qux...    │  ← Active (updates in place)
//! │ Progress: ████░░░░ 50%  │
//! └─────────────────────────┘
//! ```

use std::io::{self, Write};

use super::ansi;
use super::buffer::FrameBuffer;
use super::output::{OutputBuffer, StatefulCellRenderer};

/// Append-mode renderer with history and active regions.
pub struct AppendRenderer {
    output: OutputBuffer,
    cell_renderer: StatefulCellRenderer,
    previous_active_height: u16,
}

impl AppendRenderer {
    /// Create a new append renderer.
    pub fn new() -> Self {
        Self {
            output: OutputBuffer::new(),
            cell_renderer: StatefulCellRenderer::new(),
            previous_active_height: 0,
        }
    }

    /// Render the active region (updates in place).
    ///
    /// This erases the previous active content and renders new content.
    /// History above remains untouched.
    pub fn render_active(&mut self, buffer: &FrameBuffer) -> io::Result<()> {
        // Begin synchronized output
        ansi::begin_sync(&mut self.output)?;

        // Erase previous active region
        self.erase_active_internal()?;

        // Reset renderer state
        self.cell_renderer.reset();

        // Render buffer content
        let width = buffer.width();
        let height = buffer.height();

        for y in 0..height {
            for x in 0..width {
                if let Some(cell) = buffer.get(x, y) {
                    self.cell_renderer.render_cell_inline(&mut self.output, cell);
                }
            }
            // Newline after each row
            self.output.write_char('\n');
        }

        // Reset attributes
        ansi::reset(&mut self.output)?;

        // End synchronized output
        ansi::end_sync(&mut self.output)?;

        // Flush
        self.output.flush_stdout()?;

        // Track height for next erase
        self.previous_active_height = height;

        Ok(())
    }

    /// Write a line to history (above active region).
    ///
    /// The line is written and the active region redrawn below it.
    pub fn write_history(&mut self, line: &str) -> io::Result<()> {
        // Erase active region first
        self.erase_active()?;

        // Write history line
        let mut stdout = io::stdout().lock();
        writeln!(stdout, "{}", line)?;
        stdout.flush()?;

        Ok(())
    }

    /// Write multiple lines to history.
    pub fn write_history_lines(&mut self, lines: &[&str]) -> io::Result<()> {
        // Erase active region first
        self.erase_active()?;

        // Write all history lines
        let mut stdout = io::stdout().lock();
        for line in lines {
            writeln!(stdout, "{}", line)?;
        }
        stdout.flush()?;

        Ok(())
    }

    /// Erase the active region without rendering new content.
    pub fn erase_active(&mut self) -> io::Result<()> {
        if self.previous_active_height > 0 {
            ansi::begin_sync(&mut self.output)?;
            self.erase_active_internal()?;
            ansi::end_sync(&mut self.output)?;
            self.output.flush_stdout()?;
            self.previous_active_height = 0;
        }
        Ok(())
    }

    /// Internal erase without sync block (for use within render).
    fn erase_active_internal(&mut self) -> io::Result<()> {
        if self.previous_active_height > 0 {
            // Move cursor up by previous height
            ansi::cursor_up(&mut self.output, self.previous_active_height)?;
            ansi::cursor_column_zero(&mut self.output)?;
            // Erase from cursor down
            ansi::erase_down(&mut self.output)?;
        }
        Ok(())
    }

    /// Get the height of the current active region.
    pub fn active_height(&self) -> u16 {
        self.previous_active_height
    }

    /// Clear everything and reset state.
    pub fn reset(&mut self) -> io::Result<()> {
        self.erase_active()?;
        self.previous_active_height = 0;
        self.cell_renderer.reset();
        Ok(())
    }

    /// Finalize - clean up and show cursor.
    ///
    /// Call this when done with the renderer.
    pub fn finalize(&mut self) -> io::Result<()> {
        self.erase_active()?;
        ansi::reset(&mut self.output)?;
        ansi::cursor_show(&mut self.output)?;
        self.output.flush_stdout()?;
        Ok(())
    }
}

impl Default for AppendRenderer {
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
    fn test_append_renderer_creation() {
        let renderer = AppendRenderer::new();
        assert_eq!(renderer.active_height(), 0);
    }
}
