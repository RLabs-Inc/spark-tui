//! Drawn Cursor Module - Cursor management for input components
//!
//! Manages cursor state for input components (Input, custom editors).
//! Unlike the terminal native cursor (cursor.rs), this cursor is drawn into
//! the frameBuffer and supports full customization.
//!
//! # Features
//!
//! - Style presets (block, bar, underline) or custom characters
//! - Blink animation with configurable FPS (default: 2 FPS)
//! - Alt character for blink "off" phase
//! - Integrates with animate module for efficient shared clocks
//!
//! # Pattern
//!
//! The drawn cursor uses focus callbacks to manage blink subscription:
//! - On focus: subscribe to blink clock
//! - On blur: unsubscribe from blink clock
//!
//! The cursor_visible getter is a closure that evaluates:
//! 1. Manual override (show/hide) - highest priority
//! 2. Focus state - not focused = always visible (no blink)
//! 3. Blink phase - when focused and blink enabled
//!
//! # Example
//!
//! ```ignore
//! use spark_tui::state::drawn_cursor::{create_cursor, dispose_cursor, DrawnCursorConfig};
//! use spark_tui::types::CursorStyle;
//!
//! // In input component:
//! let cursor = create_cursor(index, DrawnCursorConfig {
//!     style: CursorStyle::Bar,
//!     blink: true,
//!     fps: 2,
//!     ..Default::default()
//! });
//!
//! // Update position
//! cursor.set_position(5);
//!
//! // Cleanup on unmount
//! dispose_cursor(index);
//! ```

use std::cell::RefCell;
use std::collections::HashMap;

use spark_signals::{signal, Signal};

use crate::engine::arrays::interaction;
use crate::state::{animate, focus::{self, FocusCallbacks}};
use crate::types::CursorStyle;

// =============================================================================
// CURSOR CHARACTER CODEPOINTS
// =============================================================================

/// Block cursor - special case: inverse rendering (swap fg/bg)
pub const CURSOR_CHAR_BLOCK: u32 = 0;
/// Bar cursor - vertical line (|)
pub const CURSOR_CHAR_BAR: u32 = 0x2502;
/// Underline cursor - underscore (_)
pub const CURSOR_CHAR_UNDERLINE: u32 = 0x5F;

/// Get the cursor character codepoint for a style
fn cursor_char_for_style(style: CursorStyle) -> u32 {
    match style {
        CursorStyle::Block => CURSOR_CHAR_BLOCK,
        CursorStyle::Bar => CURSOR_CHAR_BAR,
        CursorStyle::Underline => CURSOR_CHAR_UNDERLINE,
    }
}

// =============================================================================
// CONFIGURATION
// =============================================================================

/// Configuration for creating a drawn cursor.
#[derive(Clone)]
pub struct DrawnCursorConfig {
    /// Cursor style preset (default: Block)
    pub style: CursorStyle,
    /// Custom cursor character (overrides style if Some)
    pub char: Option<char>,
    /// Enable blink animation (default: true)
    pub blink: bool,
    /// Blink FPS - 2 = 500ms on/off cycle (default: 2)
    pub fps: u8,
    /// Alt character for blink "off" phase (default: None = show original text)
    pub alt_char: Option<char>,
}

impl Default for DrawnCursorConfig {
    fn default() -> Self {
        Self {
            style: CursorStyle::Block,
            char: None,
            blink: true,
            fps: 2,
            alt_char: None,
        }
    }
}

// =============================================================================
// DRAWN CURSOR CONTROL OBJECT
// =============================================================================

/// Control object for a drawn cursor.
///
/// Returned by `create_cursor()`. Provides methods to control cursor visibility
/// and position.
pub struct DrawnCursor {
    index: usize,
}

impl DrawnCursor {
    /// Set cursor position in text.
    pub fn set_position(&self, pos: u16) {
        interaction::set_cursor_position(self.index, pos);
    }

    /// Get current cursor position.
    pub fn get_position(&self) -> u16 {
        interaction::get_cursor_position(self.index)
    }

    /// Manually show cursor (override blink).
    pub fn show(&self) {
        ACTIVE_CURSORS.with(|cursors| {
            if let Some(entry) = cursors.borrow().get(&self.index) {
                entry.manual_visible.set(Some(true));
            }
        });
    }

    /// Manually hide cursor (override blink).
    pub fn hide(&self) {
        ACTIVE_CURSORS.with(|cursors| {
            if let Some(entry) = cursors.borrow().get(&self.index) {
                entry.manual_visible.set(Some(false));
            }
        });
    }

    /// Clear manual override, return to blink-controlled visibility.
    pub fn clear_override(&self) {
        ACTIVE_CURSORS.with(|cursors| {
            if let Some(entry) = cursors.borrow().get(&self.index) {
                entry.manual_visible.set(None);
            }
        });
    }

    /// Check if cursor is currently visible.
    pub fn is_visible(&self) -> bool {
        interaction::get_cursor_visible(self.index)
    }

    /// Dispose this cursor (alias for `dispose_cursor`).
    pub fn dispose(self) {
        dispose_cursor(self.index);
    }
}

// =============================================================================
// ACTIVE CURSORS REGISTRY
// =============================================================================

/// Internal registry entry for an active cursor
struct CursorEntry {
    /// Unsubscribe function for blink (Some when focused, None when not)
    unsubscribe_blink: RefCell<Option<Box<dyn FnOnce()>>>,
    /// Unsubscribe function for focus callbacks
    unsubscribe_focus: RefCell<Option<Box<dyn FnOnce()>>>,
    /// Manual visibility override: None = use blink, Some(true/false) = override
    manual_visible: Signal<Option<bool>>,
    /// Blink enabled for this cursor
    blink_enabled: bool,
    /// Blink FPS for this cursor
    fps: u8,
}

thread_local! {
    /// Registry of active cursors by component index
    static ACTIVE_CURSORS: RefCell<HashMap<usize, CursorEntry>> = RefCell::new(HashMap::new());
}

// =============================================================================
// CREATE CURSOR
// =============================================================================

/// Create a drawn cursor for a component.
///
/// Sets up cursor arrays, registers focus callbacks, and creates the
/// cursor_visible getter that handles blink animation.
///
/// # Arguments
///
/// * `index` - Component index
/// * `config` - Cursor configuration
///
/// # Returns
///
/// A `DrawnCursor` control object for manipulating the cursor.
///
/// # Example
///
/// ```ignore
/// let cursor = create_cursor(index, DrawnCursorConfig {
///     style: CursorStyle::Bar,
///     blink: true,
///     fps: 2,
///     ..Default::default()
/// });
///
/// cursor.set_position(5);
/// ```
pub fn create_cursor(index: usize, config: DrawnCursorConfig) -> DrawnCursor {
    // Determine cursor character codepoint
    let char_code = if let Some(ch) = config.char {
        ch as u32
    } else {
        cursor_char_for_style(config.style)
    };

    // Determine alt character for blink "off" phase
    let alt_char_code = config.alt_char.map(|c| c as u32).unwrap_or(0);

    // Set cursor arrays
    interaction::set_cursor_char(index, char_code);
    interaction::set_cursor_alt_char(index, alt_char_code);
    interaction::set_cursor_style(index, config.style as u8);
    interaction::set_cursor_blink_fps(index, if config.blink { config.fps } else { 0 });

    // Manual visibility override signal
    let manual_visible = signal(None::<bool>);
    let manual_visible_clone = manual_visible.clone();

    // Blink configuration
    let blink_enabled = config.blink && config.fps > 0;
    let fps = config.fps;

    // Create cursor_visible getter closure
    // This is set in the interaction array and evaluated by the renderer
    let manual_visible_for_getter = manual_visible.clone();
    interaction::set_cursor_visible_getter(index, move || {
        // 1. Check manual override first
        if let Some(manual) = manual_visible_for_getter.get() {
            return manual;
        }

        // 2. Check focus state
        let focused = focus::is_focused(index);
        if !focused {
            // Not focused = always show cursor (no blink)
            return true;
        }

        // 3. If focused but blink disabled, always visible
        if !blink_enabled {
            return true;
        }

        // 4. If focused and blink enabled, return blink phase
        animate::get_blink_phase(fps)
    });

    // Create entry for registry
    let entry = CursorEntry {
        unsubscribe_blink: RefCell::new(None),
        unsubscribe_focus: RefCell::new(None),
        manual_visible: manual_visible_clone,
        blink_enabled,
        fps,
    };

    // Store in registry
    ACTIVE_CURSORS.with(|cursors| {
        cursors.borrow_mut().insert(index, entry);
    });

    // Register focus callbacks to start/stop blink
    // These fire imperatively at the source of focus change
    let focus_cleanup = focus::register_callbacks(index, FocusCallbacks {
        on_focus: Some(Box::new(move || {
            if blink_enabled {
                let unsub = animate::subscribe_to_blink(fps);
                ACTIVE_CURSORS.with(|cursors| {
                    if let Some(entry) = cursors.borrow().get(&index) {
                        *entry.unsubscribe_blink.borrow_mut() = Some(unsub);
                    }
                });
            }
        })),
        on_blur: Some(Box::new(move || {
            ACTIVE_CURSORS.with(|cursors| {
                if let Some(entry) = cursors.borrow().get(&index) {
                    if let Some(unsub) = entry.unsubscribe_blink.borrow_mut().take() {
                        unsub();
                    }
                }
            });
        })),
    });

    // Store focus cleanup
    ACTIVE_CURSORS.with(|cursors| {
        if let Some(entry) = cursors.borrow().get(&index) {
            *entry.unsubscribe_focus.borrow_mut() = Some(Box::new(focus_cleanup));
        }
    });

    DrawnCursor { index }
}

// =============================================================================
// DISPOSE CURSOR
// =============================================================================

/// Dispose a cursor and clean up its resources.
///
/// - Unsubscribes from focus callbacks
/// - Unsubscribes from blink (if active)
/// - Clears cursor arrays
/// - Removes from registry
///
/// This is idempotent - safe to call multiple times.
///
/// # Arguments
///
/// * `index` - Component index
pub fn dispose_cursor(index: usize) {
    ACTIVE_CURSORS.with(|cursors| {
        if let Some(entry) = cursors.borrow_mut().remove(&index) {
            // Unsubscribe from focus callbacks
            if let Some(unsub) = entry.unsubscribe_focus.borrow_mut().take() {
                unsub();
            }

            // Unsubscribe from blink (if active)
            if let Some(unsub) = entry.unsubscribe_blink.borrow_mut().take() {
                unsub();
            }
        }
    });

    // Clear cursor arrays (always, even if not in registry)
    interaction::set_cursor_char(index, 0);
    interaction::set_cursor_alt_char(index, 0);
    interaction::set_cursor_style(index, 0);
    interaction::set_cursor_blink_fps(index, 0);
    interaction::set_cursor_visible(index, true); // Reset to default
}

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

/// Check if a component has an active cursor.
pub fn has_cursor(index: usize) -> bool {
    ACTIVE_CURSORS.with(|cursors| cursors.borrow().contains_key(&index))
}

/// Get the number of active cursors (for testing).
pub fn active_cursor_count() -> usize {
    ACTIVE_CURSORS.with(|cursors| cursors.borrow().len())
}

/// Reset all active cursors (for testing).
pub fn reset_cursors() {
    // Collect indices to dispose
    let indices: Vec<usize> = ACTIVE_CURSORS.with(|cursors| {
        cursors.borrow().keys().copied().collect()
    });

    // Dispose each cursor
    for index in indices {
        dispose_cursor(index);
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::reset_registry;
    use crate::engine::arrays::interaction as int_arrays;
    use crate::state::focus::reset_focus_state;
    use crate::state::animate::reset_blink_registries;

    fn setup() {
        reset_registry();
        reset_focus_state();
        reset_blink_registries();
        reset_cursors();
        int_arrays::reset();
    }

    #[test]
    fn test_create_cursor_sets_arrays() {
        setup();

        let _cursor = create_cursor(0, DrawnCursorConfig {
            style: CursorStyle::Bar,
            blink: true,
            fps: 2,
            ..Default::default()
        });

        // Verify arrays are set
        assert_eq!(int_arrays::get_cursor_char(0), CURSOR_CHAR_BAR);
        assert_eq!(int_arrays::get_cursor_style(0), CursorStyle::Bar as u8);
        assert_eq!(int_arrays::get_cursor_blink_fps(0), 2);
        assert_eq!(int_arrays::get_cursor_alt_char(0), 0);
    }

    #[test]
    fn test_cursor_visible_when_not_focused() {
        setup();

        let _cursor = create_cursor(0, DrawnCursorConfig::default());

        // Not focused = always visible (no blink)
        assert!(int_arrays::get_cursor_visible(0));
    }

    #[test]
    fn test_cursor_blinks_when_focused() {
        setup();

        // Create a focusable component first
        use crate::primitives::{input, InputProps};
        let value = spark_signals::signal("test".to_string());
        let _cleanup = input(InputProps::new(value));

        let _cursor = create_cursor(0, DrawnCursorConfig {
            blink: true,
            fps: 2,
            ..Default::default()
        });

        // Focus the component
        focus::focus(0);

        // Check that blink is now running
        assert!(animate::is_blink_running(2));
        assert_eq!(animate::get_subscriber_count(2), 1);
    }

    #[test]
    fn test_cursor_stops_blink_on_blur() {
        setup();

        // Create two focusable components
        use crate::primitives::{input, InputProps};
        let value1 = spark_signals::signal("test1".to_string());
        let value2 = spark_signals::signal("test2".to_string());
        let _cleanup1 = input(InputProps::new(value1));
        let _cleanup2 = input(InputProps::new(value2));

        let _cursor = create_cursor(0, DrawnCursorConfig {
            blink: true,
            fps: 2,
            ..Default::default()
        });

        // Focus component 0
        focus::focus(0);
        assert!(animate::is_blink_running(2));

        // Focus component 1 (blurs component 0)
        focus::focus(1);

        // Blink should stop (no subscribers at fps=2 for component 0's cursor)
        // Note: The blink may still be running if component 1 also has a cursor
        // For this test, component 1 doesn't have a drawn cursor
        assert!(!animate::is_blink_running(2));
    }

    #[test]
    fn test_dispose_cursor_cleans_up() {
        setup();

        let cursor = create_cursor(0, DrawnCursorConfig {
            style: CursorStyle::Underline,
            ..Default::default()
        });

        assert!(has_cursor(0));
        assert_eq!(int_arrays::get_cursor_char(0), CURSOR_CHAR_UNDERLINE);

        cursor.dispose();

        assert!(!has_cursor(0));
        assert_eq!(int_arrays::get_cursor_char(0), 0);
        assert_eq!(int_arrays::get_cursor_style(0), 0);
    }

    #[test]
    fn test_manual_show_hide_override() {
        setup();

        // Create a focusable component
        use crate::primitives::{input, InputProps};
        let value = spark_signals::signal("test".to_string());
        let _cleanup = input(InputProps::new(value));

        let cursor = create_cursor(0, DrawnCursorConfig::default());

        // Default: visible (not focused)
        assert!(cursor.is_visible());

        // Manual hide
        cursor.hide();
        assert!(!int_arrays::get_cursor_visible(0));

        // Manual show
        cursor.show();
        assert!(int_arrays::get_cursor_visible(0));

        // Clear override
        cursor.clear_override();
        // Back to default behavior
        assert!(int_arrays::get_cursor_visible(0)); // Not focused = visible
    }

    #[test]
    fn test_cursor_styles() {
        setup();

        // Block
        let _cursor1 = create_cursor(0, DrawnCursorConfig {
            style: CursorStyle::Block,
            ..Default::default()
        });
        assert_eq!(int_arrays::get_cursor_char(0), CURSOR_CHAR_BLOCK);
        assert_eq!(int_arrays::get_cursor_style(0), 0); // Block = 0

        dispose_cursor(0);

        // Bar
        let _cursor2 = create_cursor(0, DrawnCursorConfig {
            style: CursorStyle::Bar,
            ..Default::default()
        });
        assert_eq!(int_arrays::get_cursor_char(0), CURSOR_CHAR_BAR);
        assert_eq!(int_arrays::get_cursor_style(0), 1); // Bar = 1

        dispose_cursor(0);

        // Underline
        let _cursor3 = create_cursor(0, DrawnCursorConfig {
            style: CursorStyle::Underline,
            ..Default::default()
        });
        assert_eq!(int_arrays::get_cursor_char(0), CURSOR_CHAR_UNDERLINE);
        assert_eq!(int_arrays::get_cursor_style(0), 2); // Underline = 2
    }

    #[test]
    fn test_custom_cursor_char() {
        setup();

        // Custom character overrides style
        let _cursor = create_cursor(0, DrawnCursorConfig {
            style: CursorStyle::Block, // This should be ignored
            char: Some('X'),           // Custom char takes precedence
            ..Default::default()
        });

        assert_eq!(int_arrays::get_cursor_char(0), 'X' as u32);
    }
}
