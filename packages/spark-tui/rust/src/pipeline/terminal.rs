//! Terminal setup and teardown.
//!
//! Handles entering/exiting raw mode, alternate screen, mouse tracking,
//! Kitty keyboard protocol, and other terminal configuration.
//!
//! All terminal protocol writes are done via ANSI escape sequences.

use std::io;
use crate::renderer::ansi;
use crate::renderer::OutputBuffer;

/// Terminal setup/teardown handle.
pub struct TerminalSetup {
    is_fullscreen: bool,
    is_raw: bool,
    mouse_enabled: bool,
    kitty_keyboard: bool,
    bracketed_paste: bool,
    focus_reporting: bool,
}

impl TerminalSetup {
    pub fn new() -> Self {
        Self {
            is_fullscreen: false,
            is_raw: false,
            mouse_enabled: false,
            kitty_keyboard: false,
            bracketed_paste: false,
            focus_reporting: false,
        }
    }

    /// Enter fullscreen mode with all terminal features enabled.
    pub fn enter_fullscreen(&mut self) -> io::Result<()> {
        let mut out = OutputBuffer::new();

        // Enable raw mode (platform-specific)
        self.enable_raw_mode()?;

        // Enter alternate screen
        ansi::enter_alt_screen(&mut out)?;

        // Hide cursor
        ansi::cursor_hide(&mut out)?;

        // Clear screen
        ansi::clear_screen(&mut out)?;

        // Enable SGR mouse tracking
        out.write_str("\x1b[?1000h"); // Enable mouse clicks
        out.write_str("\x1b[?1002h"); // Enable mouse motion
        out.write_str("\x1b[?1003h"); // Enable all mouse tracking
        out.write_str("\x1b[?1006h"); // SGR mouse protocol
        self.mouse_enabled = true;

        // Enable Kitty keyboard protocol (progressive enhancement level 1)
        out.write_str("\x1b[>1u");
        self.kitty_keyboard = true;

        // Enable bracketed paste
        out.write_str("\x1b[?2004h");
        self.bracketed_paste = true;

        // Enable focus reporting
        out.write_str("\x1b[?1004h");
        self.focus_reporting = true;

        // Synchronized output start
        out.write_str("\x1b[?2026h");

        out.flush_stdout()?;
        self.is_fullscreen = true;
        Ok(())
    }

    /// Enter inline mode (no alternate screen, no mouse).
    /// For inline/append render modes where terminal scroll should work.
    pub fn enter_inline(&mut self) -> io::Result<()> {
        let mut out = OutputBuffer::new();

        // Enable raw mode (needed for keyboard input)
        self.enable_raw_mode()?;

        // Hide cursor during renders
        ansi::cursor_hide(&mut out)?;

        // NO alternate screen - stay in normal buffer
        // NO mouse tracking - let terminal handle scroll

        // Enable Kitty keyboard protocol for better key detection
        out.write_str("\x1b[>1u");
        self.kitty_keyboard = true;

        // Enable bracketed paste
        out.write_str("\x1b[?2004h");
        self.bracketed_paste = true;

        out.flush_stdout()?;
        // Note: is_fullscreen stays false for inline mode
        Ok(())
    }

    /// Exit inline mode and restore terminal.
    pub fn exit_inline(&mut self) -> io::Result<()> {
        let mut out = OutputBuffer::new();

        // Disable bracketed paste
        if self.bracketed_paste {
            out.write_str("\x1b[?2004l");
            self.bracketed_paste = false;
        }

        // Disable Kitty keyboard
        if self.kitty_keyboard {
            out.write_str("\x1b[<u");
            self.kitty_keyboard = false;
        }

        // Reset terminal state
        ansi::reset(&mut out)?;

        // Show cursor
        ansi::cursor_show(&mut out)?;

        out.flush_stdout()?;

        // Disable raw mode
        self.disable_raw_mode()?;
        Ok(())
    }

    /// Exit fullscreen mode and restore terminal.
    pub fn exit_fullscreen(&mut self) -> io::Result<()> {
        let mut out = OutputBuffer::new();

        // Disable focus reporting
        if self.focus_reporting {
            out.write_str("\x1b[?1004l");
            self.focus_reporting = false;
        }

        // Disable bracketed paste
        if self.bracketed_paste {
            out.write_str("\x1b[?2004l");
            self.bracketed_paste = false;
        }

        // Disable Kitty keyboard
        if self.kitty_keyboard {
            out.write_str("\x1b[<u");
            self.kitty_keyboard = false;
        }

        // Disable mouse tracking
        if self.mouse_enabled {
            out.write_str("\x1b[?1006l");
            out.write_str("\x1b[?1003l");
            out.write_str("\x1b[?1002l");
            out.write_str("\x1b[?1000l");
            self.mouse_enabled = false;
        }

        // End synchronized output
        out.write_str("\x1b[?2026l");

        // Reset terminal state
        ansi::reset(&mut out)?;

        // Show cursor
        ansi::cursor_show(&mut out)?;

        // Exit alternate screen
        ansi::exit_alt_screen(&mut out)?;

        out.flush_stdout()?;

        // Disable raw mode
        self.disable_raw_mode()?;
        self.is_fullscreen = false;
        Ok(())
    }

    /// Enable raw mode (platform-specific).
    fn enable_raw_mode(&mut self) -> io::Result<()> {
        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            let fd = io::stdin().as_raw_fd();

            // Check if stdin is a TTY — skip raw mode if not (e.g., piped input, testing)
            if unsafe { libc::isatty(fd) } == 0 {
                // Not a TTY — can't enable raw mode, but continue anyway
                // (engine won't receive keyboard input but will render)
                return Ok(());
            }

            // Use libc termios to enable raw mode
            unsafe {
                let mut termios: libc::termios = std::mem::zeroed();
                if libc::tcgetattr(fd, &mut termios) != 0 {
                    return Err(io::Error::last_os_error());
                }
                // Save original for restore
                ORIGINAL_TERMIOS = Some(termios);

                // Raw mode flags
                termios.c_iflag &= !(libc::IGNBRK | libc::BRKINT | libc::PARMRK | libc::ISTRIP
                    | libc::INLCR | libc::IGNCR | libc::ICRNL | libc::IXON);
                termios.c_oflag &= !libc::OPOST;
                termios.c_lflag &= !(libc::ECHO | libc::ECHONL | libc::ICANON | libc::ISIG | libc::IEXTEN);
                termios.c_cflag &= !(libc::CSIZE | libc::PARENB);
                termios.c_cflag |= libc::CS8;
                termios.c_cc[libc::VMIN] = 1;
                termios.c_cc[libc::VTIME] = 0;

                if libc::tcsetattr(fd, libc::TCSAFLUSH, &termios) != 0 {
                    return Err(io::Error::last_os_error());
                }
            }
            self.is_raw = true;
        }
        Ok(())
    }

    /// Disable raw mode.
    fn disable_raw_mode(&mut self) -> io::Result<()> {
        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            let fd = io::stdin().as_raw_fd();
            unsafe {
                if let Some(ref original) = ORIGINAL_TERMIOS {
                    if libc::tcsetattr(fd, libc::TCSAFLUSH, original) != 0 {
                        return Err(io::Error::last_os_error());
                    }
                }
            }
            self.is_raw = false;
        }
        Ok(())
    }
}

impl Default for TerminalSetup {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TerminalSetup {
    fn drop(&mut self) {
        if self.is_fullscreen {
            let _ = self.exit_fullscreen();
        }
    }
}

/// Saved original terminal settings for restore.
#[cfg(unix)]
static mut ORIGINAL_TERMIOS: Option<libc::termios> = None;
