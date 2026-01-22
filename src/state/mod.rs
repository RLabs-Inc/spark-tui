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
//! - **Scroll** - Scrollable containers, scroll chaining (future)
//! - **Cursor** - Drawn cursor for inputs, blink animation (future)

pub mod clipboard;
pub mod focus;
pub mod global_keys;
pub mod input;
pub mod keyboard;
pub mod mouse;

pub use clipboard::{copy, paste, cut, clear as clear_clipboard, has_content as clipboard_has_content};
pub use focus::*;
pub use global_keys::*;
pub use input::*;
pub use keyboard::*;
