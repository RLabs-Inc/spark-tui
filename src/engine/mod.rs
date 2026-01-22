//! TUI Engine - Component registry and parallel arrays.
//!
//! The engine manages the core data structures:
//! - Registry: Index allocation, ID mapping, parent context
//! - FlexNode: Persistent layout object with 33 Slot properties
//! - FlexNodeRegistry: FlexNode lifecycle management
//! - Arrays: Parallel SlotArrays for component state
//!
//! # Architecture
//!
//! Components are NOT objects. They are indices into parallel arrays:
//!
//! ```text
//! Index 0: Box  (parent=-1, width=80, visible=true,  fg=white, ...)
//! Index 1: Text (parent=0,  width=auto, visible=true, fg=blue,  ...)
//! Index 2: Box  (parent=0,  width=40,  visible=true, fg=white, ...)
//! ```
//!
//! This enables cache-friendly iteration, efficient reactivity (each cell is a
//! stable Slot that never moves), and no object allocation overhead.

mod registry;
mod flex_node;
mod flex_node_registry;
pub mod arrays;

pub use registry::*;
pub use flex_node::*;
pub use flex_node_registry::*;
