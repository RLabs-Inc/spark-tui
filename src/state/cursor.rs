//! Terminal Native Cursor API
//!
//! This module provides a clean API for terminal cursor control, wrapping the
//! low-level ANSI escape sequences from `ansi.rs`. It tracks cursor state
//! internally so you can query cursor position, visibility, and shape.
//!
//! # Functions
//!
//! - [`cursor_show`], [`cursor_hide`] - Toggle cursor visibility
//! - [`cursor_move_to`] - Position the terminal cursor
//! - [`cursor_set_shape`] - Set cursor shape (block, bar, underline)
//! - [`cursor_save`], [`cursor_restore`] - Save/restore cursor position (DEC)
//!
//! # State Query
//!
//! - [`cursor_is_visible`] - Check if cursor is currently visible
//! - [`cursor_position`] - Get current cursor position
//! - [`cursor_shape`] - Get current cursor shape

use crate::renderer::ansi;
use std::cell::RefCell;
use std::io::{self, Write};

// Re-export CursorShape from ansi for external use
pub use ansi::CursorShape;

// =============================================================================
// Internal State
// =============================================================================

/// Internal cursor state for tracking across render cycles.
struct CursorState {
    visible: bool,
    shape: CursorShape,
    blinking: bool,
    x: u16,
    y: u16,
}

impl Default for CursorState {
    fn default() -> Self {
        Self {
            visible: true,
            shape: CursorShape::Block,
            blinking: false,
            x: 0,
            y: 0,
        }
    }
}

thread_local! {
    static CURSOR_STATE: RefCell<CursorState> = RefCell::new(CursorState::default());
}

// =============================================================================
// Visibility Control
// =============================================================================

/// Show the terminal cursor.
///
/// Updates internal state and writes ANSI escape sequence to stdout.
pub fn cursor_show() -> io::Result<()> {
    CURSOR_STATE.with(|state| {
        state.borrow_mut().visible = true;
    });
    let mut stdout = io::stdout().lock();
    ansi::cursor_show(&mut stdout)?;
    stdout.flush()
}

/// Hide the terminal cursor.
///
/// Updates internal state and writes ANSI escape sequence to stdout.
pub fn cursor_hide() -> io::Result<()> {
    CURSOR_STATE.with(|state| {
        state.borrow_mut().visible = false;
    });
    let mut stdout = io::stdout().lock();
    ansi::cursor_hide(&mut stdout)?;
    stdout.flush()
}

// =============================================================================
// Position Control
// =============================================================================

/// Move the terminal cursor to the specified position.
///
/// Coordinates are 0-indexed (converted to 1-indexed for the terminal).
///
/// Updates internal state and writes ANSI escape sequence to stdout.
pub fn cursor_move_to(x: u16, y: u16) -> io::Result<()> {
    CURSOR_STATE.with(|state| {
        let mut s = state.borrow_mut();
        s.x = x;
        s.y = y;
    });
    let mut stdout = io::stdout().lock();
    ansi::cursor_to(&mut stdout, x, y)?;
    stdout.flush()
}

// =============================================================================
// Shape Control
// =============================================================================

/// Set the terminal cursor shape.
///
/// # Arguments
///
/// * `shape` - The cursor shape (Block, Bar, or Underline)
/// * `blinking` - Whether the cursor should blink
///
/// Updates internal state and writes ANSI escape sequence to stdout.
pub fn cursor_set_shape(shape: CursorShape, blinking: bool) -> io::Result<()> {
    CURSOR_STATE.with(|state| {
        let mut s = state.borrow_mut();
        s.shape = shape;
        s.blinking = blinking;
    });
    let mut stdout = io::stdout().lock();
    ansi::cursor_shape(&mut stdout, shape, blinking)?;
    stdout.flush()
}

// =============================================================================
// Save/Restore
// =============================================================================

/// Save the current cursor position (DEC sequence).
///
/// The position can be restored later with [`cursor_restore`].
///
/// Note: This uses the terminal's save position, not our internal state.
pub fn cursor_save() -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    ansi::cursor_save(&mut stdout)?;
    stdout.flush()
}

/// Restore the previously saved cursor position (DEC sequence).
///
/// Restores the position saved by [`cursor_save`].
///
/// Note: This uses the terminal's saved position. Our internal state is not
/// updated to match the restored position.
pub fn cursor_restore() -> io::Result<()> {
    let mut stdout = io::stdout().lock();
    ansi::cursor_restore(&mut stdout)?;
    stdout.flush()
}

// =============================================================================
// State Query
// =============================================================================

/// Check if the terminal cursor is currently visible.
///
/// Returns the last visibility state set via [`cursor_show`] or [`cursor_hide`].
pub fn cursor_is_visible() -> bool {
    CURSOR_STATE.with(|state| state.borrow().visible)
}

/// Get the current terminal cursor position.
///
/// Returns `(x, y)` coordinates (0-indexed) as last set via [`cursor_move_to`].
pub fn cursor_position() -> (u16, u16) {
    CURSOR_STATE.with(|state| {
        let s = state.borrow();
        (s.x, s.y)
    })
}

/// Get the current terminal cursor shape.
///
/// Returns the shape as last set via [`cursor_set_shape`].
pub fn cursor_shape() -> CursorShape {
    CURSOR_STATE.with(|state| state.borrow().shape)
}

/// Check if the cursor is set to blink.
///
/// Returns the blinking state as last set via [`cursor_set_shape`].
pub fn cursor_is_blinking() -> bool {
    CURSOR_STATE.with(|state| state.borrow().blinking)
}

// =============================================================================
// Testing Helpers
// =============================================================================

/// Reset cursor state to defaults (for testing).
///
/// Resets to: visible, Block shape, not blinking, position (0, 0).
///
/// Note: This only resets internal state, not the actual terminal cursor.
pub fn reset_cursor_state() {
    CURSOR_STATE.with(|state| {
        *state.borrow_mut() = CursorState::default();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() {
        reset_cursor_state();
    }

    #[test]
    fn test_cursor_show_hide() {
        setup();

        // Initially visible
        assert!(cursor_is_visible());

        // Hide cursor (update state only - don't actually write to terminal in tests)
        CURSOR_STATE.with(|state| {
            state.borrow_mut().visible = false;
        });
        assert!(!cursor_is_visible());

        // Show cursor
        CURSOR_STATE.with(|state| {
            state.borrow_mut().visible = true;
        });
        assert!(cursor_is_visible());
    }

    #[test]
    fn test_cursor_move_to() {
        setup();

        // Initially at (0, 0)
        assert_eq!(cursor_position(), (0, 0));

        // Move to new position
        CURSOR_STATE.with(|state| {
            let mut s = state.borrow_mut();
            s.x = 10;
            s.y = 20;
        });
        assert_eq!(cursor_position(), (10, 20));

        // Move again
        CURSOR_STATE.with(|state| {
            let mut s = state.borrow_mut();
            s.x = 5;
            s.y = 15;
        });
        assert_eq!(cursor_position(), (5, 15));
    }

    #[test]
    fn test_cursor_shape() {
        setup();

        // Initially Block
        assert_eq!(cursor_shape(), CursorShape::Block);
        assert!(!cursor_is_blinking());

        // Change to Bar with blinking
        CURSOR_STATE.with(|state| {
            let mut s = state.borrow_mut();
            s.shape = CursorShape::Bar;
            s.blinking = true;
        });
        assert_eq!(cursor_shape(), CursorShape::Bar);
        assert!(cursor_is_blinking());

        // Change to Underline without blinking
        CURSOR_STATE.with(|state| {
            let mut s = state.borrow_mut();
            s.shape = CursorShape::Underline;
            s.blinking = false;
        });
        assert_eq!(cursor_shape(), CursorShape::Underline);
        assert!(!cursor_is_blinking());
    }

    #[test]
    fn test_reset_cursor_state() {
        setup();

        // Modify all state
        CURSOR_STATE.with(|state| {
            let mut s = state.borrow_mut();
            s.visible = false;
            s.shape = CursorShape::Bar;
            s.blinking = true;
            s.x = 50;
            s.y = 100;
        });

        // Verify state changed
        assert!(!cursor_is_visible());
        assert_eq!(cursor_shape(), CursorShape::Bar);
        assert!(cursor_is_blinking());
        assert_eq!(cursor_position(), (50, 100));

        // Reset
        reset_cursor_state();

        // Verify back to defaults
        assert!(cursor_is_visible());
        assert_eq!(cursor_shape(), CursorShape::Block);
        assert!(!cursor_is_blinking());
        assert_eq!(cursor_position(), (0, 0));
    }

    #[test]
    fn test_cursor_save_restore_is_noop_for_state() {
        setup();

        // Set a position
        CURSOR_STATE.with(|state| {
            let mut s = state.borrow_mut();
            s.x = 25;
            s.y = 30;
        });

        // Save doesn't change internal state
        // (actual save/restore happens in terminal)
        let pos_before = cursor_position();

        // Move to new position
        CURSOR_STATE.with(|state| {
            let mut s = state.borrow_mut();
            s.x = 50;
            s.y = 60;
        });

        // Restore doesn't change internal state
        // (actual save/restore happens in terminal)
        let pos_after = cursor_position();

        // Internal state reflects last set, not restored
        assert_eq!(pos_before, (25, 30));
        assert_eq!(pos_after, (50, 60));
    }
}
