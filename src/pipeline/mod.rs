//! Reactive Pipeline
//!
//! This module implements the reactive rendering pipeline that connects
//! the component system to the terminal output.
//!
//! # Pipeline Architecture
//!
//! ```text
//! Component Tree → FlexNode Slots → layoutDerived → frameBufferDerived → render effect
//! ```
//!
//! ## Data Flow
//!
//! 1. **layoutDerived** - Reads FlexNode slots, calls Taffy, returns ComputedLayout
//! 2. **frameBufferDerived** - Reads layout + visual arrays, fills FrameBuffer
//! 3. **render effect** - Monitors frameBufferDerived, calls blind renderer
//!
//! ## Key Design Principles
//!
//! - **Pure Deriveds**: layoutDerived and frameBufferDerived are pure computations
//! - **Side Effects in Effect**: Only the render effect mutates state (hitGrid, terminal I/O)
//! - **Reactive Dependencies**: Reads from signals/slots auto-track dependencies

pub mod terminal;
pub mod layout_derived;
pub mod frame_buffer_derived;
pub mod inheritance;
pub mod mount;

// Re-exports
pub use terminal::{terminal_width, terminal_height, set_terminal_size, RenderMode, render_mode, set_render_mode};
pub use layout_derived::{create_layout_derived, get_layout, try_get_layout, set_layout, clear_layout};
pub use frame_buffer_derived::{create_frame_buffer_derived, FrameBufferResult, HitRegion};
pub use mount::{mount, unmount, MountHandle};
