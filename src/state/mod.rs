//! State Module - Runtime state management systems
//!
//! This module contains the reactive state systems that power TUI interactivity:
//!
//! - **Focus** - Tab cycling, focus trap, callbacks, history
//! - **Keyboard** - Event types, dispatch, handler registry
//! - **Mouse** - HitGrid, event dispatch, hover tracking
//! - **Input** - Event conversion and polling from crossterm
//! - **Global Keys** - Global keyboard shortcuts (Ctrl+C, Tab navigation)
//! - **Clipboard** - Text copy/paste with internal buffer
//! - **Scroll** - Scrollable containers, scroll chaining
//! - **Cursor** - Drawn cursor for inputs, blink animation (future)

pub mod clipboard;
pub mod focus;
pub mod global_keys;
pub mod input;
pub mod keyboard;
pub mod mouse;
pub mod scroll;

pub use clipboard::{copy, paste, cut, clear as clear_clipboard, has_content as clipboard_has_content};
pub use focus::*;
pub use global_keys::*;
pub use input::*;
pub use keyboard::*;
pub use scroll::{
    is_scrollable, get_scroll_offset, get_max_scroll,
    set_scroll_offset, scroll_by, scroll_to_top, scroll_to_bottom,
    scroll_to_start, scroll_to_end, scroll_by_with_chaining,
    get_focused_scrollable, handle_arrow_scroll, handle_page_scroll,
    handle_home_end, handle_wheel_scroll, scroll_into_view,
    set_current_layout, with_current_layout, clear_current_layout,
    LINE_SCROLL, WHEEL_SCROLL, PAGE_SCROLL_FACTOR,
};
