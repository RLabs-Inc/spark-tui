//! # spark-tui
//!
//! Reactive Terminal UI Framework for Rust.
//!
//! Built on [spark-signals](https://github.com/RLabs-Inc/spark-signals) for fine-grained reactivity.
//!
//! ## Architecture
//!
//! spark-tui uses a parallel arrays (ECS-style) architecture where components are
//! indices into columnar arrays rather than objects. Each array cell is a reactive
//! `Slot` that can be bound to signals, getters, or static values.
//!
//! The rendering pipeline is purely derived-based:
//! ```text
//! Component Tree → FlexNode Slots → layoutDerived → frameBufferDerived → render effect
//! ```
//!
//! ## Reference Implementation
//!
//! The TypeScript version at `/Users/rusty/Documents/Projects/TUI/tui` serves as
//! the reference implementation. All features should be ported with identical
//! patterns and ergonomics.
//!
//! ## Modules
//!
//! - [`types`] - Core types (Dimension, RGBA, ComponentType, etc.)
//! - [`engine`] - Component registry, FlexNode, parallel arrays
//! - [`layout`] - Taffy/TITAN layout engine for flexbox computation
//! - [`renderer`] - Terminal renderer (ANSI output, diff rendering)

pub mod engine;
pub mod layout;
pub mod pipeline;
pub mod primitives;
pub mod renderer;
pub mod state;
pub mod theme;
pub mod types;

// Re-export commonly used items
pub use types::*;

pub use engine::{
    allocate_index, create_flex_node, destroy_flex_node, get_allocated_indices,
    get_current_parent_index, get_flex_node, get_id, get_index, is_allocated, on_destroy,
    pop_parent_context, push_parent_context, release_index, reset_registry, FlexNode,
};

pub use layout::{
    compute_layout, reset_titan_arrays, string_width, measure_text_height,
    truncate_text, wrap_text, ComputedLayout,
};

pub use renderer::{
    AppendRenderer, ClipRect, DiffRenderer, FrameBuffer, InlineRenderer, OutputBuffer,
};

pub use pipeline::{
    create_frame_buffer_derived, create_layout_derived, mount, unmount,
    render_mode, set_render_mode, set_terminal_size, terminal_height, terminal_width,
    FrameBufferResult, HitRegion, MountHandle, RenderMode,
};

pub use primitives::{
    box_primitive, text, BoxProps, TextProps, PropValue, Cleanup,
};

pub use state::{
    // Focus
    focus, focus_first, focus_last, focus_next, focus_previous, blur,
    get_focused_index, has_focus, is_focused, get_focusable_indices,
    register_callbacks, FocusCallbacks,
    push_focus_trap, pop_focus_trap, is_focus_trapped, get_focus_trap_container,
    save_focus_to_history, restore_focus_from_history,
    reset_focus_state,
    // Keyboard
    KeyboardEvent, KeyState, Modifiers, KeyHandler,
    dispatch as dispatch_keyboard, dispatch_focused,
    on as on_keyboard, on_key, on_keys, on_focused,
    last_event, last_key, cleanup_index, reset_keyboard_state,
};

pub use theme::{
    // Types
    Theme, ThemeColor, ReactiveTheme, ThemeAccessor,
    // Variants
    Variant, VariantStyle, get_variant_style, variant_style,
    // Presets
    get_preset, preset_names,
    terminal, dracula, nord, monokai, solarized, catppuccin, gruvbox,
    tokyo_night, one_dark, rose_pine, kanagawa, everforest, night_owl,
    // Reactive state
    t, active_theme, get_reactive_theme, set_theme, set_custom_theme,
    reset_theme_state, reset_accessor,
};
