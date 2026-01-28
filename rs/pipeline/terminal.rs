//! Terminal state signals.
//!
//! Provides reactive signals for terminal dimensions and render mode.
//! These are the root signals that trigger the entire rendering pipeline.

use spark_signals::signal;
use std::cell::RefCell;

// =============================================================================
// Terminal Size Signals
// =============================================================================

thread_local! {
    static TERMINAL_WIDTH: RefCell<spark_signals::Signal<u16>> = RefCell::new(signal(80));
    static TERMINAL_HEIGHT: RefCell<spark_signals::Signal<u16>> = RefCell::new(signal(24));
    static RENDER_MODE: RefCell<spark_signals::Signal<RenderMode>> = RefCell::new(signal(RenderMode::Fullscreen));
}

/// Get the current terminal width.
pub fn terminal_width() -> u16 {
    TERMINAL_WIDTH.with(|w| w.borrow().get())
}

/// Get the current terminal height.
pub fn terminal_height() -> u16 {
    TERMINAL_HEIGHT.with(|h| h.borrow().get())
}

/// Set the terminal size (called on resize events).
pub fn set_terminal_size(width: u16, height: u16) {
    TERMINAL_WIDTH.with(|w| w.borrow().set(width));
    TERMINAL_HEIGHT.with(|h| h.borrow().set(height));
}

/// Get the terminal width signal for reactive tracking.
pub fn terminal_width_signal() -> spark_signals::Signal<u16> {
    TERMINAL_WIDTH.with(|w| w.borrow().clone())
}

/// Get the terminal height signal for reactive tracking.
pub fn terminal_height_signal() -> spark_signals::Signal<u16> {
    TERMINAL_HEIGHT.with(|h| h.borrow().clone())
}

// =============================================================================
// Render Mode
// =============================================================================

/// Rendering mode determines how content is displayed on the terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderMode {
    /// Fullscreen mode - uses alternate screen buffer, differential rendering.
    #[default]
    Fullscreen,
    /// Inline mode - renders to normal buffer, clears and redraws each frame.
    Inline,
    /// Append mode - two regions: frozen history above, active updating below.
    Append,
}

/// Get the current render mode.
pub fn render_mode() -> RenderMode {
    RENDER_MODE.with(|m| m.borrow().get())
}

/// Set the render mode.
pub fn set_render_mode(mode: RenderMode) {
    RENDER_MODE.with(|m| m.borrow().set(mode));
}

/// Get the render mode signal for reactive tracking.
pub fn render_mode_signal() -> spark_signals::Signal<RenderMode> {
    RENDER_MODE.with(|m| m.borrow().clone())
}

// =============================================================================
// Terminal Detection
// =============================================================================

/// Detect and set the actual terminal size from the environment.
///
/// Uses crossterm to query the terminal dimensions.
pub fn detect_terminal_size() {
    if let Ok((width, height)) = crossterm::terminal::size() {
        set_terminal_size(width, height);
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_size() {
        set_terminal_size(120, 40);
        assert_eq!(terminal_width(), 120);
        assert_eq!(terminal_height(), 40);
    }

    #[test]
    fn test_render_mode() {
        set_render_mode(RenderMode::Inline);
        assert_eq!(render_mode(), RenderMode::Inline);

        set_render_mode(RenderMode::Fullscreen);
        assert_eq!(render_mode(), RenderMode::Fullscreen);
    }
}
