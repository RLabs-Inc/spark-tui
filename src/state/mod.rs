//! State Module - Runtime state management systems
//!
//! This module contains the reactive state systems that power TUI interactivity:
//!
//! - **Focus** - Tab cycling, focus trap, callbacks, history
//! - **Keyboard** - Event types, dispatch, handler registry
//! - **Mouse** - HitGrid, event dispatch, hover tracking (future)
//! - **Scroll** - Scrollable containers, scroll chaining (future)
//! - **Cursor** - Drawn cursor for inputs, blink animation (future)

mod focus;
mod keyboard;

pub use focus::*;
pub use keyboard::*;
