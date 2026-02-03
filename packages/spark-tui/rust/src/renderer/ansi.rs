//! ANSI escape sequences for terminal control.
//!
//! This module provides all the escape sequences needed for terminal rendering:
//! - Cursor movement and visibility
//! - Screen clearing and scrolling
//! - Colors (ANSI 16, 256, and TrueColor)
//! - Text attributes (bold, italic, underline, etc.)
//! - Mouse and keyboard protocol control
//! - Synchronized output for flicker-free rendering

use crate::utils::{Attr, Rgba};
use std::io::Write;

// =============================================================================
// Constants
// =============================================================================

/// Escape character.
pub const ESC: &str = "\x1b";

/// Control Sequence Introducer.
pub const CSI: &str = "\x1b[";

/// Operating System Command.
pub const OSC: &str = "\x1b]";

/// Bell character (OSC terminator).
pub const BEL: &str = "\x07";

/// String Terminator.
pub const ST: &str = "\x1b\\";

// =============================================================================
// Cursor Movement
// =============================================================================

/// Move cursor to absolute position (1-indexed).
#[inline]
pub fn cursor_to<W: Write>(w: &mut W, x: u16, y: u16) -> std::io::Result<()> {
    write!(w, "\x1b[{};{}H", y + 1, x + 1)
}

/// Move cursor up by n rows.
#[inline]
pub fn cursor_up<W: Write>(w: &mut W, n: u16) -> std::io::Result<()> {
    if n > 0 {
        write!(w, "\x1b[{}A", n)
    } else {
        Ok(())
    }
}

/// Move cursor down by n rows.
#[inline]
pub fn cursor_down<W: Write>(w: &mut W, n: u16) -> std::io::Result<()> {
    if n > 0 {
        write!(w, "\x1b[{}B", n)
    } else {
        Ok(())
    }
}

/// Move cursor forward (right) by n columns.
#[inline]
pub fn cursor_forward<W: Write>(w: &mut W, n: u16) -> std::io::Result<()> {
    if n > 0 {
        write!(w, "\x1b[{}C", n)
    } else {
        Ok(())
    }
}

/// Move cursor backward (left) by n columns.
#[inline]
pub fn cursor_backward<W: Write>(w: &mut W, n: u16) -> std::io::Result<()> {
    if n > 0 {
        write!(w, "\x1b[{}D", n)
    } else {
        Ok(())
    }
}

/// Move cursor to beginning of line.
#[inline]
pub fn cursor_column_zero<W: Write>(w: &mut W) -> std::io::Result<()> {
    write!(w, "\x1b[G")
}

/// Move cursor to next line start.
#[inline]
pub fn cursor_next_line<W: Write>(w: &mut W) -> std::io::Result<()> {
    write!(w, "\x1b[E")
}

/// Move cursor to previous line start.
#[inline]
pub fn cursor_prev_line<W: Write>(w: &mut W) -> std::io::Result<()> {
    write!(w, "\x1b[F")
}

/// Save cursor position (DEC).
#[inline]
pub fn cursor_save<W: Write>(w: &mut W) -> std::io::Result<()> {
    write!(w, "\x1b7")
}

/// Restore cursor position (DEC).
#[inline]
pub fn cursor_restore<W: Write>(w: &mut W) -> std::io::Result<()> {
    write!(w, "\x1b8")
}

/// Hide cursor.
#[inline]
pub fn cursor_hide<W: Write>(w: &mut W) -> std::io::Result<()> {
    write!(w, "\x1b[?25l")
}

/// Show cursor.
#[inline]
pub fn cursor_show<W: Write>(w: &mut W) -> std::io::Result<()> {
    write!(w, "\x1b[?25h")
}

/// Cursor shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorShape {
    Block,
    Underline,
    Bar,
}

/// Set cursor shape.
#[inline]
pub fn cursor_shape<W: Write>(w: &mut W, shape: CursorShape, blinking: bool) -> std::io::Result<()> {
    let n = match (shape, blinking) {
        (CursorShape::Block, true) => 1,
        (CursorShape::Block, false) => 2,
        (CursorShape::Underline, true) => 3,
        (CursorShape::Underline, false) => 4,
        (CursorShape::Bar, true) => 5,
        (CursorShape::Bar, false) => 6,
    };
    write!(w, "\x1b[{} q", n)
}

// =============================================================================
// Screen Control
// =============================================================================

/// Clear from cursor to end of line.
#[inline]
pub fn erase_to_eol<W: Write>(w: &mut W) -> std::io::Result<()> {
    write!(w, "\x1b[K")
}

/// Clear from start of line to cursor.
#[inline]
pub fn erase_from_sol<W: Write>(w: &mut W) -> std::io::Result<()> {
    write!(w, "\x1b[1K")
}

/// Clear entire line.
#[inline]
pub fn erase_line<W: Write>(w: &mut W) -> std::io::Result<()> {
    write!(w, "\x1b[2K")
}

/// Clear from cursor to end of screen.
#[inline]
pub fn erase_down<W: Write>(w: &mut W) -> std::io::Result<()> {
    write!(w, "\x1b[J")
}

/// Clear from start of screen to cursor.
#[inline]
pub fn erase_up<W: Write>(w: &mut W) -> std::io::Result<()> {
    write!(w, "\x1b[1J")
}

/// Clear entire screen (viewport only).
#[inline]
pub fn erase_screen<W: Write>(w: &mut W) -> std::io::Result<()> {
    write!(w, "\x1b[2J")
}

/// Clear screen and scrollback buffer.
#[inline]
pub fn clear_screen<W: Write>(w: &mut W) -> std::io::Result<()> {
    write!(w, "\x1b[2J\x1b[3J\x1b[H")
}

/// Clear only scrollback buffer.
#[inline]
pub fn clear_scrollback<W: Write>(w: &mut W) -> std::io::Result<()> {
    write!(w, "\x1b[3J")
}

/// Erase n lines upward from cursor (for inline mode).
pub fn erase_lines<W: Write>(w: &mut W, count: u16) -> std::io::Result<()> {
    for _ in 0..count {
        erase_line(w)?;
        cursor_up(w, 1)?;
    }
    erase_line(w)?;
    cursor_column_zero(w)
}

/// Enter alternate screen buffer (fullscreen mode).
#[inline]
pub fn enter_alt_screen<W: Write>(w: &mut W) -> std::io::Result<()> {
    write!(w, "\x1b[?1049h")
}

/// Exit alternate screen buffer.
#[inline]
pub fn exit_alt_screen<W: Write>(w: &mut W) -> std::io::Result<()> {
    write!(w, "\x1b[?1049l")
}

/// Scroll screen up by n lines.
#[inline]
pub fn scroll_up<W: Write>(w: &mut W, n: u16) -> std::io::Result<()> {
    write!(w, "\x1b[{}S", n)
}

/// Scroll screen down by n lines.
#[inline]
pub fn scroll_down<W: Write>(w: &mut W, n: u16) -> std::io::Result<()> {
    write!(w, "\x1b[{}T", n)
}

// =============================================================================
// Synchronized Output (Flicker Prevention)
// =============================================================================

/// Begin synchronized output (terminal buffers until end_sync).
#[inline]
pub fn begin_sync<W: Write>(w: &mut W) -> std::io::Result<()> {
    write!(w, "\x1b[?2026h")
}

/// End synchronized output (terminal flushes buffer).
#[inline]
pub fn end_sync<W: Write>(w: &mut W) -> std::io::Result<()> {
    write!(w, "\x1b[?2026l")
}

// =============================================================================
// Colors
// =============================================================================

/// Reset all attributes and colors.
#[inline]
pub fn reset<W: Write>(w: &mut W) -> std::io::Result<()> {
    write!(w, "\x1b[0m")
}

/// Set foreground color.
#[inline]
pub fn fg<W: Write>(w: &mut W, color: Rgba) -> std::io::Result<()> {
    if color.is_terminal_default() {
        // Reset to terminal default foreground
        write!(w, "\x1b[39m")
    } else if color.is_ansi() {
        let index = color.ansi_index();
        if index < 8 {
            // Standard colors: 30-37
            write!(w, "\x1b[{}m", 30 + index)
        } else if index < 16 {
            // Bright colors: 90-97
            write!(w, "\x1b[{}m", 90 + index - 8)
        } else {
            // Extended palette: 38;5;n
            write!(w, "\x1b[38;5;{}m", index)
        }
    } else {
        // TrueColor: 38;2;r;g;b
        write!(w, "\x1b[38;2;{};{};{}m", color.r, color.g, color.b)
    }
}

/// Set background color.
#[inline]
pub fn bg<W: Write>(w: &mut W, color: Rgba) -> std::io::Result<()> {
    if color.is_terminal_default() {
        // Reset to terminal default background
        write!(w, "\x1b[49m")
    } else if color.is_ansi() {
        let index = color.ansi_index();
        if index < 8 {
            // Standard colors: 40-47
            write!(w, "\x1b[{}m", 40 + index)
        } else if index < 16 {
            // Bright colors: 100-107
            write!(w, "\x1b[{}m", 100 + index - 8)
        } else {
            // Extended palette: 48;5;n
            write!(w, "\x1b[48;5;{}m", index)
        }
    } else {
        // TrueColor: 48;2;r;g;b
        write!(w, "\x1b[48;2;{};{};{}m", color.r, color.g, color.b)
    }
}

// =============================================================================
// Text Attributes
// =============================================================================

/// Set text attributes from bitflags.
#[allow(unused_assignments)]
pub fn attrs<W: Write>(w: &mut W, attr: Attr) -> std::io::Result<()> {
    if attr.is_empty() {
        return Ok(());
    }

    let mut first = true;
    write!(w, "\x1b[")?;

    macro_rules! emit {
        ($flag:expr, $code:expr) => {
            if attr.contains($flag) {
                if !first {
                    write!(w, ";")?;
                }
                write!(w, "{}", $code)?;
                first = false;
            }
        };
    }

    emit!(Attr::BOLD, 1);
    emit!(Attr::DIM, 2);
    emit!(Attr::ITALIC, 3);
    emit!(Attr::UNDERLINE, 4);
    emit!(Attr::BLINK, 5);
    emit!(Attr::INVERSE, 7);
    emit!(Attr::HIDDEN, 8);
    emit!(Attr::STRIKETHROUGH, 9);

    write!(w, "m")
}

/// Reset specific attribute.
#[inline]
pub fn reset_bold<W: Write>(w: &mut W) -> std::io::Result<()> {
    write!(w, "\x1b[22m")
}

#[inline]
pub fn reset_dim<W: Write>(w: &mut W) -> std::io::Result<()> {
    write!(w, "\x1b[22m")
}

#[inline]
pub fn reset_italic<W: Write>(w: &mut W) -> std::io::Result<()> {
    write!(w, "\x1b[23m")
}

#[inline]
pub fn reset_underline<W: Write>(w: &mut W) -> std::io::Result<()> {
    write!(w, "\x1b[24m")
}

#[inline]
pub fn reset_blink<W: Write>(w: &mut W) -> std::io::Result<()> {
    write!(w, "\x1b[25m")
}

#[inline]
pub fn reset_inverse<W: Write>(w: &mut W) -> std::io::Result<()> {
    write!(w, "\x1b[27m")
}

#[inline]
pub fn reset_hidden<W: Write>(w: &mut W) -> std::io::Result<()> {
    write!(w, "\x1b[28m")
}

#[inline]
pub fn reset_strikethrough<W: Write>(w: &mut W) -> std::io::Result<()> {
    write!(w, "\x1b[29m")
}

// =============================================================================
// Mouse Support
// =============================================================================

/// Enable mouse tracking (SGR extended mode - best compatibility).
#[inline]
pub fn enable_mouse<W: Write>(w: &mut W) -> std::io::Result<()> {
    // Enable button events + any events + SGR extended mode
    write!(w, "\x1b[?1000h\x1b[?1002h\x1b[?1006h")
}

/// Disable mouse tracking.
#[inline]
pub fn disable_mouse<W: Write>(w: &mut W) -> std::io::Result<()> {
    write!(w, "\x1b[?1006l\x1b[?1002l\x1b[?1000l")
}

// =============================================================================
// Keyboard Protocols
// =============================================================================

/// Enable Kitty keyboard protocol (enhanced key reporting).
#[inline]
pub fn enable_kitty_keyboard<W: Write>(w: &mut W) -> std::io::Result<()> {
    // Flags: 1=disambiguate, 2=report events, 4=alternate keys, 8=all keys
    write!(w, "\x1b[>1u")
}

/// Disable Kitty keyboard protocol.
#[inline]
pub fn disable_kitty_keyboard<W: Write>(w: &mut W) -> std::io::Result<()> {
    write!(w, "\x1b[<u")
}

/// Enable bracketed paste mode.
#[inline]
pub fn enable_bracketed_paste<W: Write>(w: &mut W) -> std::io::Result<()> {
    write!(w, "\x1b[?2004h")
}

/// Disable bracketed paste mode.
#[inline]
pub fn disable_bracketed_paste<W: Write>(w: &mut W) -> std::io::Result<()> {
    write!(w, "\x1b[?2004l")
}

/// Enable focus reporting.
#[inline]
pub fn enable_focus_reporting<W: Write>(w: &mut W) -> std::io::Result<()> {
    write!(w, "\x1b[?1004h")
}

/// Disable focus reporting.
#[inline]
pub fn disable_focus_reporting<W: Write>(w: &mut W) -> std::io::Result<()> {
    write!(w, "\x1b[?1004l")
}

// =============================================================================
// Window/Title
// =============================================================================

/// Set terminal window title.
#[inline]
pub fn set_title<W: Write>(w: &mut W, title: &str) -> std::io::Result<()> {
    write!(w, "\x1b]0;{}\x07", title)
}

// =============================================================================
// Hyperlinks
// =============================================================================

/// Create a hyperlink (OSC 8).
pub fn link<W: Write>(w: &mut W, text: &str, url: &str) -> std::io::Result<()> {
    write!(w, "\x1b]8;;{}\x07{}\x1b]8;;\x07", url, text)
}

// =============================================================================
// Testing Helpers
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn to_string<F: FnOnce(&mut Vec<u8>) -> std::io::Result<()>>(f: F) -> String {
        let mut buf = Vec::new();
        f(&mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }

    #[test]
    fn test_cursor_to() {
        assert_eq!(to_string(|w| cursor_to(w, 0, 0)), "\x1b[1;1H");
        assert_eq!(to_string(|w| cursor_to(w, 5, 10)), "\x1b[11;6H");
    }

    #[test]
    fn test_cursor_movement() {
        assert_eq!(to_string(|w| cursor_up(w, 5)), "\x1b[5A");
        assert_eq!(to_string(|w| cursor_down(w, 3)), "\x1b[3B");
        assert_eq!(to_string(|w| cursor_forward(w, 2)), "\x1b[2C");
        assert_eq!(to_string(|w| cursor_backward(w, 4)), "\x1b[4D");
    }

    #[test]
    fn test_cursor_visibility() {
        assert_eq!(to_string(cursor_hide), "\x1b[?25l");
        assert_eq!(to_string(cursor_show), "\x1b[?25h");
    }

    #[test]
    fn test_screen_control() {
        assert_eq!(to_string(erase_line), "\x1b[2K");
        assert_eq!(to_string(erase_screen), "\x1b[2J");
        assert_eq!(to_string(enter_alt_screen), "\x1b[?1049h");
        assert_eq!(to_string(exit_alt_screen), "\x1b[?1049l");
    }

    #[test]
    fn test_sync_output() {
        assert_eq!(to_string(begin_sync), "\x1b[?2026h");
        assert_eq!(to_string(end_sync), "\x1b[?2026l");
    }

    #[test]
    fn test_fg_colors() {
        // Terminal default
        assert_eq!(to_string(|w| fg(w, Rgba::TERMINAL_DEFAULT)), "\x1b[39m");

        // ANSI standard (0-7)
        assert_eq!(to_string(|w| fg(w, Rgba::ansi(0))), "\x1b[30m"); // black
        assert_eq!(to_string(|w| fg(w, Rgba::ansi(1))), "\x1b[31m"); // red
        assert_eq!(to_string(|w| fg(w, Rgba::ansi(7))), "\x1b[37m"); // white

        // ANSI bright (8-15)
        assert_eq!(to_string(|w| fg(w, Rgba::ansi(8))), "\x1b[90m"); // bright black
        assert_eq!(to_string(|w| fg(w, Rgba::ansi(15))), "\x1b[97m"); // bright white

        // Extended palette (16-255)
        assert_eq!(to_string(|w| fg(w, Rgba::ansi(196))), "\x1b[38;5;196m");

        // TrueColor
        assert_eq!(
            to_string(|w| fg(w, Rgba::rgb(255, 128, 64))),
            "\x1b[38;2;255;128;64m"
        );
    }

    #[test]
    fn test_bg_colors() {
        assert_eq!(to_string(|w| bg(w, Rgba::TERMINAL_DEFAULT)), "\x1b[49m");
        assert_eq!(to_string(|w| bg(w, Rgba::ansi(1))), "\x1b[41m");
        assert_eq!(to_string(|w| bg(w, Rgba::ansi(9))), "\x1b[101m");
        assert_eq!(
            to_string(|w| bg(w, Rgba::rgb(0, 128, 255))),
            "\x1b[48;2;0;128;255m"
        );
    }

    #[test]
    fn test_attrs() {
        assert_eq!(to_string(|w| attrs(w, Attr::BOLD)), "\x1b[1m");
        assert_eq!(
            to_string(|w| attrs(w, Attr::BOLD | Attr::UNDERLINE)),
            "\x1b[1;4m"
        );
        assert_eq!(
            to_string(|w| attrs(w, Attr::BOLD | Attr::ITALIC | Attr::STRIKETHROUGH)),
            "\x1b[1;3;9m"
        );
    }

    #[test]
    fn test_reset() {
        assert_eq!(to_string(reset), "\x1b[0m");
    }
}
