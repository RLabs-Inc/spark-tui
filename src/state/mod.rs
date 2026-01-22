//! State Module - Runtime state management systems
//!
//! This module contains the reactive state systems that power TUI interactivity:
//!
//! - **Focus** - Tab cycling, focus trap, callbacks, history
//! - **Keyboard** - Event types, dispatch, handler registry
//! - **Mouse** - HitGrid, event dispatch, hover tracking
//! - **Input** - Event conversion and polling from crossterm
//! - **Scroll** - Scrollable containers, scroll chaining (future)
//! - **Cursor** - Drawn cursor for inputs, blink animation (future)

mod focus;
pub mod input;
pub mod keyboard;
pub mod mouse;

pub use focus::*;
pub use input::*;
pub use keyboard::*;
