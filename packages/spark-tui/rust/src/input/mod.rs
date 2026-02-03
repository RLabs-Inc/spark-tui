//! Rust input system — stdin → state → SharedBuffer.
//!
//! Rust owns ALL input. Reads stdin, parses escape sequences,
//! manages focus/keyboard/mouse/scroll state, and writes results
//! to SharedBuffer interaction arrays and event ring buffer.
//!
//! # Architecture
//!
//! ```text
//! stdin bytes → parser → parsed events
//!                              │
//!                    ┌─────────┴──────────┐
//!                    │                    │
//!               Keyboard               Mouse
//!                    │                    │
//!          ┌─────────┼──────────┐         │
//!          │         │          │         │
//!        Focus    Text Edit   Scroll    HitGrid → Hover/Click
//!          │         │          │         │
//!          └─────────┴──────────┴─────────┘
//!                         │
//!                  SharedBuffer + Event Ring
//! ```

pub mod parser;
pub mod focus;
pub mod keyboard;
pub mod mouse;
pub mod scroll;
pub mod text_edit;
pub mod reader;

pub use parser::{ParsedEvent, KeyEvent, MouseEvent, KeyCode, Modifier};
