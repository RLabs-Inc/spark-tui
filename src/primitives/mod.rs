//! TUI Primitives - Component building blocks.
//!
//! This module provides the core UI primitives:
//! - [`box_primitive`] - Container with flexbox layout, borders, and background
//! - [`text`] - Text display with styling and wrapping
//!
//! # Architecture
//!
//! Components are indices into parallel arrays (ECS pattern). Each component:
//! 1. Allocates an index from the registry
//! 2. Creates a FlexNode with reactive Slot properties
//! 3. Binds props directly to slots (preserving reactivity!)
//! 4. Returns a cleanup function
//!
//! # Reactivity
//!
//! Props can be:
//! - Static values: `width: 50`
//! - Signals: `width: my_signal` (stays connected!)
//! - Getters: `width: || compute_width()`
//!
//! The key is to pass props directly - don't extract values before binding!
//!
//! ```ignore
//! // CORRECT - signal stays connected
//! box_primitive(BoxProps { width: Some(PropValue::Signal(width_signal)), ..default });
//!
//! // WRONG - extracts value, breaks reactivity
//! box_primitive(BoxProps { width: Some(PropValue::Static(width_signal.get())), ..default });
//! ```

mod types;
mod box_primitive;
mod text;

pub use types::*;
pub use box_primitive::box_primitive;
pub use text::text;
