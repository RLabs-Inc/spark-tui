//! SparkTUI Shared Buffer - Rust Implementation
//!
//! This module implements the shared memory contract defined in SHARED-BUFFER-SPEC.md.
//! Both TypeScript and Rust MUST match this spec exactly.
//!
//! Memory Layout:
//!   - Header (256 bytes): Global state, wake flags, config
//!   - Nodes (1024 bytes × MAX_NODES): Per-component data
//!   - Text Pool (configurable): UTF-8 text content
//!   - Event Ring (5,132 bytes): Rust → TS event queue
//!
//! @version 3.0
//! @date 2026-01-31

use std::ptr;
use std::sync::atomic::{AtomicU32, Ordering};

use bitflags::bitflags;

// =============================================================================
// CONSTANTS
// =============================================================================

/// Header size in bytes
pub const HEADER_SIZE: usize = 256;

/// Bytes per node (16 cache lines × 64 bytes)
pub const NODE_STRIDE: usize = 1024;

/// Default maximum nodes
pub const DEFAULT_MAX_NODES: usize = 10_000;

/// Default text pool size (10 MB)
pub const DEFAULT_TEXT_POOL_SIZE: usize = 10 * 1024 * 1024;

/// Event ring header size
pub const EVENT_RING_HEADER_SIZE: usize = 12;

/// Bytes per event slot
pub const EVENT_SLOT_SIZE: usize = 20;

/// Maximum events in ring buffer
pub const MAX_EVENTS: usize = 256;

/// Total event ring size
pub const EVENT_RING_SIZE: usize = EVENT_RING_HEADER_SIZE + MAX_EVENTS * EVENT_SLOT_SIZE;

// =============================================================================
// BUFFER SIZE CALCULATION
// =============================================================================

/// Calculate total buffer size for given configuration.
///
/// Use this to determine how many bytes TypeScript should allocate.
#[inline]
pub const fn calculate_buffer_size(max_nodes: usize, text_pool_size: usize) -> usize {
    HEADER_SIZE + (max_nodes * NODE_STRIDE) + text_pool_size + EVENT_RING_SIZE
}

/// Default buffer size (10,000 nodes, 10MB text pool).
///
/// ≈ 20.7 MB total.
pub const DEFAULT_BUFFER_SIZE: usize = calculate_buffer_size(DEFAULT_MAX_NODES, DEFAULT_TEXT_POOL_SIZE);

/// Maximum grid tracks per axis
pub const MAX_GRID_TRACKS: usize = 32;

/// Bytes per grid track (type u8 + padding u8 + value f32)
pub const GRID_TRACK_SIZE: usize = 6;

// =============================================================================
// HEADER OFFSETS (256 bytes total)
// =============================================================================

// --- Bytes 0-63: Core ---
pub const H_VERSION: usize = 0;
pub const H_NODE_COUNT: usize = 4;
pub const H_MAX_NODES: usize = 8;
pub const H_TERMINAL_WIDTH: usize = 12;
pub const H_TERMINAL_HEIGHT: usize = 16;
pub const H_GENERATION: usize = 20;
pub const H_TEXT_POOL_SIZE: usize = 24;
pub const H_TEXT_POOL_WRITE_PTR: usize = 28;
// 32-63: reserved

// --- Bytes 64-95: Wake & Sync (4-byte aligned for Atomics) ---
pub const H_WAKE_RUST: usize = 64;
pub const H_WAKE_TS: usize = 68;
// 72-95: reserved

// --- Bytes 96-127: State (Rust writes, TS reads) ---
pub const H_FOCUSED_INDEX: usize = 96;
pub const H_HOVERED_INDEX: usize = 100;
pub const H_PRESSED_INDEX: usize = 104;
pub const H_MOUSE_X: usize = 108;
pub const H_MOUSE_Y: usize = 110;
// 112-127: reserved

// --- Bytes 128-159: Config (TS writes, Rust reads) ---
pub const H_CONFIG_FLAGS: usize = 128;
pub const H_RENDER_MODE: usize = 132;
pub const H_CURSOR_CONFIG: usize = 136;
pub const H_SCROLL_SPEED: usize = 140;
// 144-159: reserved

// --- Bytes 160-191: Events ---
pub const H_EVENT_WRITE_IDX: usize = 160;
pub const H_EVENT_READ_IDX: usize = 164;
pub const H_EXIT_REQUESTED: usize = 168;
// 169-191: reserved

// --- Bytes 192-255: Stats & Debug ---
pub const H_RENDER_COUNT: usize = 192;
pub const H_LAYOUT_COUNT: usize = 196;
// 200-255: reserved

// =============================================================================
// NODE FIELD OFFSETS (1024 bytes per node)
// =============================================================================
// Organized in 16 cache lines (64 bytes each)

// --- Cache Line 1 (0-63): Core Layout Dimensions ---
pub const N_WIDTH: usize = 0;
pub const N_HEIGHT: usize = 4;
pub const N_MIN_WIDTH: usize = 8;
pub const N_MIN_HEIGHT: usize = 12;
pub const N_MAX_WIDTH: usize = 16;
pub const N_MAX_HEIGHT: usize = 20;
pub const N_ASPECT_RATIO: usize = 24;
pub const N_COMPONENT_TYPE: usize = 28;
pub const N_DISPLAY: usize = 29;
pub const N_POSITION: usize = 30;
pub const N_OVERFLOW: usize = 31;
pub const N_VISIBLE: usize = 32;
pub const N_BOX_SIZING: usize = 33;
pub const N_DIRTY_FLAGS: usize = 34;
// 35: reserved
// 36-63: reserved

// --- Cache Line 2 (64-127): Flexbox Properties ---
pub const N_FLEX_DIRECTION: usize = 64;
pub const N_FLEX_WRAP: usize = 65;
pub const N_JUSTIFY_CONTENT: usize = 66;
pub const N_ALIGN_ITEMS: usize = 67;
pub const N_ALIGN_CONTENT: usize = 68;
pub const N_ALIGN_SELF: usize = 69;
// 70-71: reserved (alignment)
pub const N_FLEX_GROW: usize = 72;
pub const N_FLEX_SHRINK: usize = 76;
pub const N_FLEX_BASIS: usize = 80;
pub const N_GAP: usize = 84;
pub const N_ROW_GAP: usize = 88;
pub const N_COLUMN_GAP: usize = 92;
// 96-127: reserved

// --- Cache Line 3 (128-191): Spacing Properties ---
pub const N_PADDING_TOP: usize = 128;
pub const N_PADDING_RIGHT: usize = 132;
pub const N_PADDING_BOTTOM: usize = 136;
pub const N_PADDING_LEFT: usize = 140;
pub const N_MARGIN_TOP: usize = 144;
pub const N_MARGIN_RIGHT: usize = 148;
pub const N_MARGIN_BOTTOM: usize = 152;
pub const N_MARGIN_LEFT: usize = 156;
pub const N_INSET_TOP: usize = 160;
pub const N_INSET_RIGHT: usize = 164;
pub const N_INSET_BOTTOM: usize = 168;
pub const N_INSET_LEFT: usize = 172;
pub const N_BORDER_WIDTH_TOP: usize = 176;
pub const N_BORDER_WIDTH_RIGHT: usize = 177;
pub const N_BORDER_WIDTH_BOTTOM: usize = 178;
pub const N_BORDER_WIDTH_LEFT: usize = 179;
pub const N_PARENT_INDEX: usize = 180;
pub const N_TAB_INDEX: usize = 184;
// 188-191: reserved

// --- Cache Line 4 (192-255): Grid Container Properties ---
pub const N_GRID_AUTO_FLOW: usize = 192;
pub const N_JUSTIFY_ITEMS: usize = 193;
pub const N_GRID_COLUMN_COUNT: usize = 194;
pub const N_GRID_ROW_COUNT: usize = 195;
pub const N_GRID_AUTO_COLUMNS_TYPE: usize = 196;
pub const N_GRID_AUTO_ROWS_TYPE: usize = 197;
// 198-199: reserved (alignment)
pub const N_GRID_AUTO_COLUMNS_VALUE: usize = 200;
pub const N_GRID_AUTO_ROWS_VALUE: usize = 204;
pub const N_GRID_COLUMN_START: usize = 208;
pub const N_GRID_COLUMN_END: usize = 210;
pub const N_GRID_ROW_START: usize = 212;
pub const N_GRID_ROW_END: usize = 214;
pub const N_JUSTIFY_SELF: usize = 216;
// 217-255: reserved

// --- Cache Lines 5-7 (256-447): Grid Column Tracks ---
// 32 tracks × 6 bytes each = 192 bytes
pub const N_GRID_COLUMN_TRACKS: usize = 256;

// --- Cache Lines 8-10 (448-639): Grid Row Tracks ---
// 32 tracks × 6 bytes each = 192 bytes
pub const N_GRID_ROW_TRACKS: usize = 448;

// --- Cache Line 11 (640-703): Computed Output ---
pub const N_COMPUTED_X: usize = 640;
pub const N_COMPUTED_Y: usize = 644;
pub const N_COMPUTED_WIDTH: usize = 648;
pub const N_COMPUTED_HEIGHT: usize = 652;
pub const N_CONTENT_WIDTH: usize = 656;
pub const N_CONTENT_HEIGHT: usize = 660;
pub const N_MAX_SCROLL_X: usize = 664;
pub const N_MAX_SCROLL_Y: usize = 668;
pub const N_IS_SCROLLABLE: usize = 672;
// 673-703: reserved

// --- Cache Line 12 (704-767): Visual Properties ---
pub const N_OPACITY: usize = 704;
pub const N_Z_INDEX: usize = 708;
pub const N_BORDER_STYLE: usize = 712;
pub const N_BORDER_STYLE_TOP: usize = 713;
pub const N_BORDER_STYLE_RIGHT: usize = 714;
pub const N_BORDER_STYLE_BOTTOM: usize = 715;
pub const N_BORDER_STYLE_LEFT: usize = 716;
pub const N_SCROLLBAR_VISIBILITY: usize = 717;
pub const N_BORDER_CHAR_H: usize = 718;
pub const N_BORDER_CHAR_V: usize = 720;
pub const N_BORDER_CHAR_TL: usize = 722;
pub const N_BORDER_CHAR_TR: usize = 724;
pub const N_BORDER_CHAR_BL: usize = 726;
pub const N_BORDER_CHAR_BR: usize = 728;
pub const N_FOCUS_INDICATOR_CHAR: usize = 730;
pub const N_FOCUS_INDICATOR_ENABLED: usize = 731;
// 732-767: reserved

// --- Cache Line 13 (768-831): Colors ---
pub const N_FG_COLOR: usize = 768;
pub const N_BG_COLOR: usize = 772;
pub const N_BORDER_COLOR: usize = 776;
pub const N_BORDER_TOP_COLOR: usize = 780;
pub const N_BORDER_RIGHT_COLOR: usize = 784;
pub const N_BORDER_BOTTOM_COLOR: usize = 788;
pub const N_BORDER_LEFT_COLOR: usize = 792;
pub const N_FOCUS_RING_COLOR: usize = 796;
pub const N_CURSOR_FG_COLOR: usize = 800;
pub const N_CURSOR_BG_COLOR: usize = 804;
pub const N_SELECTION_COLOR: usize = 808;
// 812-831: reserved

// --- Cache Line 14 (832-895): Text Properties ---
pub const N_TEXT_OFFSET: usize = 832;
pub const N_TEXT_LENGTH: usize = 836;
pub const N_TEXT_ALIGN: usize = 840;
pub const N_TEXT_WRAP: usize = 841;
pub const N_TEXT_OVERFLOW: usize = 842;
pub const N_TEXT_ATTRS: usize = 843;
pub const N_TEXT_DECORATION: usize = 844;
pub const N_TEXT_DECORATION_STYLE: usize = 845;
// 846-847: reserved (alignment)
pub const N_TEXT_DECORATION_COLOR: usize = 848;
pub const N_LINE_HEIGHT: usize = 852;
pub const N_LETTER_SPACING: usize = 853;
pub const N_MAX_LINES: usize = 854;
// 855-895: reserved

// --- Cache Line 15 (896-959): Interaction State ---
pub const N_SCROLL_X: usize = 896;
pub const N_SCROLL_Y: usize = 900;
pub const N_CURSOR_POSITION: usize = 904;
pub const N_SELECTION_START: usize = 908;
pub const N_SELECTION_END: usize = 912;
pub const N_CURSOR_CHAR: usize = 916;
pub const N_CURSOR_ALT_CHAR: usize = 920;
pub const N_INTERACTION_FLAGS: usize = 924;
pub const N_CURSOR_FLAGS: usize = 925;
pub const N_CURSOR_STYLE: usize = 926;
pub const N_CURSOR_BLINK_RATE: usize = 927;
pub const N_MAX_LENGTH: usize = 928;
pub const N_INPUT_TYPE: usize = 929;
// 930-959: reserved

// --- Cache Line 16 (960-1023): Reserved (Animation, Effects, Transforms) ---
// Reserved for future animation/effects/physics

// =============================================================================
// LEGACY OFFSET ALIASES (for layout_tree.rs compatibility)
// =============================================================================
// These will be removed after layout_tree.rs is updated

pub const F_WIDTH: usize = N_WIDTH;
pub const F_HEIGHT: usize = N_HEIGHT;
pub const F_MIN_WIDTH: usize = N_MIN_WIDTH;
pub const F_MIN_HEIGHT: usize = N_MIN_HEIGHT;
pub const F_MAX_WIDTH: usize = N_MAX_WIDTH;
pub const F_MAX_HEIGHT: usize = N_MAX_HEIGHT;
pub const F_FLEX_BASIS: usize = N_FLEX_BASIS;
pub const F_FLEX_GROW: usize = N_FLEX_GROW;
pub const F_FLEX_SHRINK: usize = N_FLEX_SHRINK;
pub const F_PADDING_TOP: usize = N_PADDING_TOP;
pub const F_PADDING_RIGHT: usize = N_PADDING_RIGHT;
pub const F_PADDING_BOTTOM: usize = N_PADDING_BOTTOM;
pub const F_PADDING_LEFT: usize = N_PADDING_LEFT;
pub const F_MARGIN_TOP: usize = N_MARGIN_TOP;
pub const F_MARGIN_RIGHT: usize = N_MARGIN_RIGHT;
pub const F_MARGIN_BOTTOM: usize = N_MARGIN_BOTTOM;
pub const F_MARGIN_LEFT: usize = N_MARGIN_LEFT;
pub const F_GAP: usize = N_GAP;
pub const F_ROW_GAP: usize = N_ROW_GAP;
pub const F_COLUMN_GAP: usize = N_COLUMN_GAP;
pub const F_INSET_TOP: usize = N_INSET_TOP;
pub const F_INSET_RIGHT: usize = N_INSET_RIGHT;
pub const F_INSET_BOTTOM: usize = N_INSET_BOTTOM;
pub const F_INSET_LEFT: usize = N_INSET_LEFT;
pub const F_COMPUTED_X: usize = N_COMPUTED_X;
pub const F_COMPUTED_Y: usize = N_COMPUTED_Y;
pub const F_COMPUTED_WIDTH: usize = N_COMPUTED_WIDTH;
pub const F_COMPUTED_HEIGHT: usize = N_COMPUTED_HEIGHT;
pub const F_SCROLL_WIDTH: usize = N_CONTENT_WIDTH;
pub const F_SCROLL_HEIGHT: usize = N_CONTENT_HEIGHT;
pub const F_MAX_SCROLL_X: usize = N_MAX_SCROLL_X;
pub const F_MAX_SCROLL_Y: usize = N_MAX_SCROLL_Y;
pub const U_FLEX_DIRECTION: usize = N_FLEX_DIRECTION;
pub const U_FLEX_WRAP: usize = N_FLEX_WRAP;
pub const U_JUSTIFY_CONTENT: usize = N_JUSTIFY_CONTENT;
pub const U_ALIGN_ITEMS: usize = N_ALIGN_ITEMS;
pub const U_ALIGN_CONTENT: usize = N_ALIGN_CONTENT;
pub const U_ALIGN_SELF: usize = N_ALIGN_SELF;
pub const U_POSITION: usize = N_POSITION;
pub const U_OVERFLOW: usize = N_OVERFLOW;
pub const U_DISPLAY: usize = N_DISPLAY;
pub const U_BORDER_WIDTH_TOP: usize = N_BORDER_WIDTH_TOP;
pub const U_BORDER_WIDTH_RIGHT: usize = N_BORDER_WIDTH_RIGHT;
pub const U_BORDER_WIDTH_BOTTOM: usize = N_BORDER_WIDTH_BOTTOM;
pub const U_BORDER_WIDTH_LEFT: usize = N_BORDER_WIDTH_LEFT;
pub const U_COMPONENT_TYPE: usize = N_COMPONENT_TYPE;
pub const U_VISIBLE: usize = N_VISIBLE;
pub const I_PARENT_INDEX: usize = N_PARENT_INDEX;
pub const I_TAB_INDEX: usize = N_TAB_INDEX;
pub const C_FG_COLOR: usize = N_FG_COLOR;
pub const C_BG_COLOR: usize = N_BG_COLOR;
pub const C_BORDER_COLOR: usize = N_BORDER_COLOR;
pub const C_BORDER_TOP_COLOR: usize = N_BORDER_TOP_COLOR;
pub const C_BORDER_RIGHT_COLOR: usize = N_BORDER_RIGHT_COLOR;
pub const C_BORDER_BOTTOM_COLOR: usize = N_BORDER_BOTTOM_COLOR;
pub const C_BORDER_LEFT_COLOR: usize = N_BORDER_LEFT_COLOR;
pub const C_FOCUS_RING_COLOR: usize = N_FOCUS_RING_COLOR;
pub const C_CURSOR_FG_COLOR: usize = N_CURSOR_FG_COLOR;
pub const C_CURSOR_BG_COLOR: usize = N_CURSOR_BG_COLOR;
pub const C_SELECTION_COLOR: usize = N_SELECTION_COLOR;
pub const U_OPACITY: usize = N_OPACITY;
pub const I_Z_INDEX: usize = N_Z_INDEX;
pub const U_BORDER_STYLE: usize = N_BORDER_STYLE;
pub const U_BORDER_STYLE_TOP: usize = N_BORDER_STYLE_TOP;
pub const U_BORDER_STYLE_RIGHT: usize = N_BORDER_STYLE_RIGHT;
pub const U_BORDER_STYLE_BOTTOM: usize = N_BORDER_STYLE_BOTTOM;
pub const U_BORDER_STYLE_LEFT: usize = N_BORDER_STYLE_LEFT;
pub const U_SCROLLABLE_FLAGS: usize = N_IS_SCROLLABLE;
pub const U_BORDER_CHAR_H: usize = N_BORDER_CHAR_H;
pub const U_BORDER_CHAR_V: usize = N_BORDER_CHAR_V;
pub const U_BORDER_CHAR_TL: usize = N_BORDER_CHAR_TL;
pub const U_BORDER_CHAR_TR: usize = N_BORDER_CHAR_TR;
pub const U_BORDER_CHAR_BL: usize = N_BORDER_CHAR_BL;
pub const U_BORDER_CHAR_BR: usize = N_BORDER_CHAR_BR;
pub const U_FOCUS_INDICATOR_CHAR: usize = N_FOCUS_INDICATOR_CHAR;
pub const U_FOCUS_INDICATOR_ENABLED: usize = N_FOCUS_INDICATOR_ENABLED;
pub const U_TEXT_OFFSET: usize = N_TEXT_OFFSET;
pub const U_TEXT_LENGTH: usize = N_TEXT_LENGTH;
pub const U_TEXT_ALIGN: usize = N_TEXT_ALIGN;
pub const U_TEXT_WRAP: usize = N_TEXT_WRAP;
pub const U_TEXT_OVERFLOW: usize = N_TEXT_OVERFLOW;
pub const U_TEXT_ATTRS: usize = N_TEXT_ATTRS;
pub const U_TEXT_DECORATION: usize = N_TEXT_DECORATION;
pub const U_TEXT_DECORATION_STYLE: usize = N_TEXT_DECORATION_STYLE;
pub const C_TEXT_DECORATION_COLOR: usize = N_TEXT_DECORATION_COLOR;
pub const U_LINE_HEIGHT: usize = N_LINE_HEIGHT;
pub const U_LETTER_SPACING: usize = N_LETTER_SPACING;
pub const U_MAX_LINES: usize = N_MAX_LINES;
pub const I_SCROLL_X: usize = N_SCROLL_X;
pub const I_SCROLL_Y: usize = N_SCROLL_Y;
pub const I_CURSOR_POSITION: usize = N_CURSOR_POSITION;
pub const I_SELECTION_START: usize = N_SELECTION_START;
pub const I_SELECTION_END: usize = N_SELECTION_END;
pub const U_CURSOR_CHAR: usize = N_CURSOR_CHAR;
pub const U_CURSOR_ALT_CHAR: usize = N_CURSOR_ALT_CHAR;
pub const U_DIRTY_FLAGS: usize = N_DIRTY_FLAGS;
pub const U_INTERACTION_FLAGS: usize = N_INTERACTION_FLAGS;
pub const U_CURSOR_FLAGS: usize = N_CURSOR_FLAGS;
pub const U_CURSOR_STYLE: usize = N_CURSOR_STYLE;
pub const U_CURSOR_BLINK_RATE: usize = N_CURSOR_BLINK_RATE;
pub const U_MAX_LENGTH: usize = N_MAX_LENGTH;
pub const U_INPUT_TYPE: usize = N_INPUT_TYPE;

// Legacy field that doesn't exist in v3 - map to reserved area
pub const I_CHILD_COUNT: usize = 188; // Use reserved space in line 3

// =============================================================================
// CONFIG FLAGS
// =============================================================================

bitflags! {
    /// Configuration flags (TS writes, Rust reads)
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ConfigFlags: u32 {
        const EXIT_ON_CTRL_C = 1 << 0;
        const TAB_NAVIGATION = 1 << 1;
        const ARROW_SCROLL = 1 << 2;
        const PAGE_SCROLL = 1 << 3;
        const HOME_END_SCROLL = 1 << 4;
        const WHEEL_SCROLL = 1 << 5;
        const FOCUS_ON_CLICK = 1 << 6;
        const MOUSE_ENABLED = 1 << 7;
        const KITTY_KEYBOARD = 1 << 8;
    }
}

impl Default for ConfigFlags {
    fn default() -> Self {
        // Default: bits 0-7 enabled (0x00FF)
        Self::EXIT_ON_CTRL_C
            | Self::TAB_NAVIGATION
            | Self::ARROW_SCROLL
            | Self::PAGE_SCROLL
            | Self::HOME_END_SCROLL
            | Self::WHEEL_SCROLL
            | Self::FOCUS_ON_CLICK
            | Self::MOUSE_ENABLED
    }
}

// =============================================================================
// DIRTY FLAGS
// =============================================================================

pub const DIRTY_LAYOUT: u8 = 1 << 0;
pub const DIRTY_VISUAL: u8 = 1 << 1;
pub const DIRTY_TEXT: u8 = 1 << 2;
pub const DIRTY_HIERARCHY: u8 = 1 << 3;

// =============================================================================
// INTERACTION FLAGS
// =============================================================================

pub const FLAG_FOCUSABLE: u8 = 1 << 0;
pub const FLAG_FOCUSED: u8 = 1 << 1;
pub const FLAG_HOVERED: u8 = 1 << 2;
pub const FLAG_PRESSED: u8 = 1 << 3;
pub const FLAG_DISABLED: u8 = 1 << 4;

// =============================================================================
// TEXT ATTRIBUTES
// =============================================================================

pub const ATTR_BOLD: u8 = 1 << 0;
pub const ATTR_ITALIC: u8 = 1 << 1;
pub const ATTR_UNDERLINE: u8 = 1 << 2;
pub const ATTR_STRIKETHROUGH: u8 = 1 << 3;
pub const ATTR_DIM: u8 = 1 << 4;
pub const ATTR_BLINK: u8 = 1 << 5;
pub const ATTR_REVERSE: u8 = 1 << 6;
pub const ATTR_HIDDEN: u8 = 1 << 7;

// =============================================================================
// COMPONENT TYPES
// =============================================================================

pub const COMPONENT_NONE: u8 = 0;
pub const COMPONENT_BOX: u8 = 1;
pub const COMPONENT_TEXT: u8 = 2;
pub const COMPONENT_INPUT: u8 = 3;

// =============================================================================
// BORDER STYLES
// =============================================================================

pub const BORDER_NONE: u8 = 0;
pub const BORDER_SINGLE: u8 = 1;
pub const BORDER_DOUBLE: u8 = 2;
pub const BORDER_ROUNDED: u8 = 3;
pub const BORDER_THICK: u8 = 4;
pub const BORDER_DASHED: u8 = 5;
pub const BORDER_DOTTED: u8 = 6;
pub const BORDER_ASCII: u8 = 7;

// =============================================================================
// EVENT TYPES
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EventType {
    None = 0,
    Key = 1,
    MouseDown = 2,
    MouseUp = 3,
    Click = 4,
    MouseEnter = 5,
    MouseLeave = 6,
    MouseMove = 7,
    Scroll = 8,
    Focus = 9,
    Blur = 10,
    ValueChange = 11,
    Submit = 12,
    Cancel = 13,
    Exit = 14,
    Resize = 15,
}

impl From<u8> for EventType {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Key,
            2 => Self::MouseDown,
            3 => Self::MouseUp,
            4 => Self::Click,
            5 => Self::MouseEnter,
            6 => Self::MouseLeave,
            7 => Self::MouseMove,
            8 => Self::Scroll,
            9 => Self::Focus,
            10 => Self::Blur,
            11 => Self::ValueChange,
            12 => Self::Submit,
            13 => Self::Cancel,
            14 => Self::Exit,
            15 => Self::Resize,
            _ => Self::None,
        }
    }
}

// =============================================================================
// ENUMS
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum FlexDirection {
    #[default]
    Row = 0,
    Column = 1,
    RowReverse = 2,
    ColumnReverse = 3,
}

impl From<u8> for FlexDirection {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Column,
            2 => Self::RowReverse,
            3 => Self::ColumnReverse,
            _ => Self::Row,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum FlexWrap {
    #[default]
    NoWrap = 0,
    Wrap = 1,
    WrapReverse = 2,
}

impl From<u8> for FlexWrap {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Wrap,
            2 => Self::WrapReverse,
            _ => Self::NoWrap,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum JustifyContent {
    #[default]
    Start = 0,
    End = 1,
    Center = 2,
    SpaceBetween = 3,
    SpaceAround = 4,
    SpaceEvenly = 5,
}

impl From<u8> for JustifyContent {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::End,
            2 => Self::Center,
            3 => Self::SpaceBetween,
            4 => Self::SpaceAround,
            5 => Self::SpaceEvenly,
            _ => Self::Start,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum AlignItems {
    Start = 0,
    End = 1,
    Center = 2,
    Baseline = 3,
    #[default]
    Stretch = 4,
}

impl From<u8> for AlignItems {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Start,
            1 => Self::End,
            2 => Self::Center,
            3 => Self::Baseline,
            _ => Self::Stretch,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum AlignContent {
    #[default]
    Start = 0,
    End = 1,
    Center = 2,
    SpaceBetween = 3,
    SpaceAround = 4,
    SpaceEvenly = 5,
}

impl From<u8> for AlignContent {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::End,
            2 => Self::Center,
            3 => Self::SpaceBetween,
            4 => Self::SpaceAround,
            5 => Self::SpaceEvenly,
            _ => Self::Start,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum AlignSelf {
    #[default]
    Auto = 0,
    Start = 1,
    End = 2,
    Center = 3,
    Baseline = 4,
    Stretch = 5,
}

impl From<u8> for AlignSelf {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Start,
            2 => Self::End,
            3 => Self::Center,
            4 => Self::Baseline,
            5 => Self::Stretch,
            _ => Self::Auto,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Position {
    #[default]
    Relative = 0,
    Absolute = 1,
}

impl From<u8> for Position {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Absolute,
            _ => Self::Relative,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Overflow {
    #[default]
    Visible = 0,
    Hidden = 1,
    Scroll = 2,
}

impl From<u8> for Overflow {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Hidden,
            2 => Self::Scroll,
            _ => Self::Visible,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Display {
    None = 0,
    #[default]
    Flex = 1,
    Grid = 2,
}

impl From<u8> for Display {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::None,
            2 => Self::Grid,
            _ => Self::Flex,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum TextAlign {
    #[default]
    Left = 0,
    Center = 1,
    Right = 2,
}

impl From<u8> for TextAlign {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Center,
            2 => Self::Right,
            _ => Self::Left,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum TextWrap {
    #[default]
    NoWrap = 0,
    Wrap = 1,
    Truncate = 2,
}

impl From<u8> for TextWrap {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Wrap,
            2 => Self::Truncate,
            _ => Self::NoWrap,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum TextOverflow {
    #[default]
    Clip = 0,
    Ellipsis = 1,
    Fade = 2,
}

impl From<u8> for TextOverflow {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Ellipsis,
            2 => Self::Fade,
            _ => Self::Clip,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum CursorStyle {
    #[default]
    Block = 0,
    Bar = 1,
    Underline = 2,
}

impl From<u8> for CursorStyle {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Bar,
            2 => Self::Underline,
            _ => Self::Block,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum RenderMode {
    #[default]
    Diff = 0,
    Inline = 1,
    Append = 2,
}

impl From<u8> for RenderMode {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Inline,
            2 => Self::Append,
            _ => Self::Diff,
        }
    }
}

// =============================================================================
// GRID ENUMS
// =============================================================================

/// Grid track sizing type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum TrackType {
    /// Track not used (sentinel for unused slots)
    #[default]
    None = 0,
    /// Auto sizing
    Auto = 1,
    /// Minimum content size
    MinContent = 2,
    /// Maximum content size
    MaxContent = 3,
    /// Fixed size in terminal cells
    Length = 4,
    /// Percentage of container (0.0-1.0)
    Percent = 5,
    /// Fractional unit
    Fr = 6,
    /// FitContent (maximum size clamped to content)
    FitContent = 7,
}

impl From<u8> for TrackType {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Auto,
            2 => Self::MinContent,
            3 => Self::MaxContent,
            4 => Self::Length,
            5 => Self::Percent,
            6 => Self::Fr,
            7 => Self::FitContent,
            _ => Self::None,
        }
    }
}

/// Grid auto flow direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum GridAutoFlow {
    #[default]
    Row = 0,
    Column = 1,
    RowDense = 2,
    ColumnDense = 3,
}

impl From<u8> for GridAutoFlow {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Column,
            2 => Self::RowDense,
            3 => Self::ColumnDense,
            _ => Self::Row,
        }
    }
}

/// Justify items (grid container property)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum JustifyItems {
    #[default]
    Start = 0,
    End = 1,
    Center = 2,
    Stretch = 3,
}

impl From<u8> for JustifyItems {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::End,
            2 => Self::Center,
            3 => Self::Stretch,
            _ => Self::Start,
        }
    }
}

/// Justify self (grid item property)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum JustifySelf {
    #[default]
    Auto = 0,
    Start = 1,
    End = 2,
    Center = 3,
    Stretch = 4,
}

impl From<u8> for JustifySelf {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Start,
            2 => Self::End,
            3 => Self::Center,
            4 => Self::Stretch,
            _ => Self::Auto,
        }
    }
}

/// A grid track definition (type + value)
#[derive(Debug, Clone, Copy, Default)]
pub struct GridTrack {
    pub track_type: TrackType,
    pub value: f32,
}

/// Border drawing style for components.
///
/// SparkTUI provides 14 predefined border styles using Unicode box-drawing characters,
/// plus a `Custom` variant for user-defined borders.
///
/// # Predefined Styles
///
/// ```text
/// None          (space)     Single        ─│┌┐└┘     Double        ═║╔╗╚╝
/// ┌───────┐                 ┌───────┐                 ╔═══════╗
/// │       │                 │       │                 ║       ║
/// └───────┘                 └───────┘                 ╚═══════╝
///
/// Rounded       ─│╭╮╰╯     Thick         ━┃┏┓┗┛     Bold          ━┃┏┓┗┛
/// ╭───────╮                 ┏━━━━━━━┓                 ┏━━━━━━━┓
/// │       │                 ┃       ┃                 ┃       ┃
/// ╰───────╯                 ┗━━━━━━━┛                 ┗━━━━━━━┛
///
/// Dashed        ╌╎┌┐└┘     Dotted        ┄┆┌┐└┘     Ascii         -|++++
/// ┌╌╌╌╌╌╌╌┐                 ┌┄┄┄┄┄┄┄┐                 +-------+
/// ╎       ╎                 ┆       ┆                 |       |
/// └╌╌╌╌╌╌╌┘                 └┄┄┄┄┄┄┄┘                 +-------+
///
/// Block         ████       DoubleHorz    ═│╒╕╘╛     DoubleVert    ─║╓╖╙╜
/// █████████                 ╒═══════╕                 ╓───────╖
/// █       █                 │       │                 ║       ║
/// █████████                 ╘═══════╛                 ╙───────╜
///
/// HeavyDashed   ╍╏┏┓┗┛     HeavyDotted   ┅┇┏┓┗┛
/// ┏╍╍╍╍╍╍╍┓                 ┏┅┅┅┅┅┅┅┓
/// ╏       ╏                 ┇       ┇
/// ┗╍╍╍╍╍╍╍┛                 ┗┅┅┅┅┅┅┅┛
/// ```
///
/// # Custom Borders
///
/// Use `BorderStyle::Custom` (value 255) to define your own border characters.
/// Set the characters via `N_BORDER_CHAR_*` fields in the SharedBuffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum BorderStyle {
    /// No border (invisible, uses space characters)
    #[default]
    None = 0,

    /// Light single lines: ─ │ ┌ ┐ └ ┘
    Single = 1,

    /// Double lines: ═ ║ ╔ ╗ ╚ ╝
    Double = 2,

    /// Single lines with rounded corners: ─ │ ╭ ╮ ╰ ╯
    Rounded = 3,

    /// Heavy/thick lines: ━ ┃ ┏ ┓ ┗ ┛
    Thick = 4,

    /// Light dashed lines: ╌ ╎ ┌ ┐ └ ┘
    Dashed = 5,

    /// Light dotted lines: ┄ ┆ ┌ ┐ └ ┘
    Dotted = 6,

    /// ASCII-only for maximum compatibility: - | + + + +
    Ascii = 7,

    /// Solid block characters: █ █ █ █ █ █
    Block = 8,

    /// Double horizontal, single vertical: ═ │ ╒ ╕ ╘ ╛
    DoubleHorz = 9,

    /// Single horizontal, double vertical: ─ ║ ╓ ╖ ╙ ╜
    DoubleVert = 10,

    /// Heavy/thick dashed lines: ╍ ╏ ┏ ┓ ┗ ┛
    HeavyDashed = 11,

    /// Heavy/thick dotted lines: ┅ ┇ ┏ ┓ ┗ ┛
    HeavyDotted = 12,

    /// Alias for Thick (semantic alternative)
    Bold = 13,

    /// User-defined characters from N_BORDER_CHAR_* fields
    Custom = 255,
}

impl From<u8> for BorderStyle {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::Single,
            2 => Self::Double,
            3 => Self::Rounded,
            4 => Self::Thick,
            5 => Self::Dashed,
            6 => Self::Dotted,
            7 => Self::Ascii,
            8 => Self::Block,
            9 => Self::DoubleHorz,
            10 => Self::DoubleVert,
            11 => Self::HeavyDashed,
            12 => Self::HeavyDotted,
            13 => Self::Bold,
            255 => Self::Custom,
            _ => Self::None,
        }
    }
}

impl BorderStyle {
    /// Get the characters for this predefined border style.
    ///
    /// Returns `(horizontal, vertical, top_left, top_right, bottom_left, bottom_right)`.
    ///
    /// For `BorderStyle::Custom`, this returns spaces. Use `SharedBuffer::border_chars()`
    /// instead to read the user-defined characters from the buffer.
    pub const fn chars(&self) -> (char, char, char, char, char, char) {
        match self {
            Self::None      => (' ', ' ', ' ', ' ', ' ', ' '),
            Self::Single    => ('─', '│', '┌', '┐', '└', '┘'),
            Self::Double    => ('═', '║', '╔', '╗', '╚', '╝'),
            Self::Rounded   => ('─', '│', '╭', '╮', '╰', '╯'),
            Self::Thick     => ('━', '┃', '┏', '┓', '┗', '┛'),
            Self::Dashed    => ('╌', '╎', '┌', '┐', '└', '┘'),
            Self::Dotted    => ('┄', '┆', '┌', '┐', '└', '┘'),
            Self::Ascii     => ('-', '|', '+', '+', '+', '+'),
            Self::Block     => ('█', '█', '█', '█', '█', '█'),
            Self::DoubleHorz=> ('═', '│', '╒', '╕', '╘', '╛'),
            Self::DoubleVert=> ('─', '║', '╓', '╖', '╙', '╜'),
            Self::HeavyDashed=>('╍', '╏', '┏', '┓', '┗', '┛'),
            Self::HeavyDotted=>('┅', '┇', '┏', '┓', '┗', '┛'),
            Self::Bold      => ('━', '┃', '┏', '┓', '┗', '┛'),
            Self::Custom    => (' ', ' ', ' ', ' ', ' ', ' '),
        }
    }

    /// Returns `true` if this is a predefined style (not Custom).
    #[inline]
    pub const fn is_predefined(&self) -> bool {
        !matches!(self, Self::Custom)
    }

    /// Returns `true` if this style uses heavy/thick lines.
    #[inline]
    pub const fn is_heavy(&self) -> bool {
        matches!(self, Self::Thick | Self::Bold | Self::HeavyDashed | Self::HeavyDotted)
    }

    /// Returns `true` if this style uses dashed or dotted lines.
    #[inline]
    pub const fn is_dashed(&self) -> bool {
        matches!(self, Self::Dashed | Self::Dotted | Self::HeavyDashed | Self::HeavyDotted)
    }
}

// =============================================================================
// RGBA COLOR
// =============================================================================

/// RGBA color with i16 components for blending math.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rgba {
    pub r: i16,
    pub g: i16,
    pub b: i16,
    pub a: i16,
}

impl Rgba {
    pub const TRANSPARENT: Self = Self { r: 0, g: 0, b: 0, a: 0 };

    /// Create from packed ARGB u32
    #[inline]
    pub fn from_packed(packed: u32) -> Self {
        Self {
            r: ((packed >> 16) & 0xFF) as i16,
            g: ((packed >> 8) & 0xFF) as i16,
            b: (packed & 0xFF) as i16,
            a: ((packed >> 24) & 0xFF) as i16,
        }
    }

    /// Pack to ARGB u32
    #[inline]
    pub fn to_packed(&self) -> u32 {
        ((self.a as u32 & 0xFF) << 24)
            | ((self.r as u32 & 0xFF) << 16)
            | ((self.g as u32 & 0xFF) << 8)
            | (self.b as u32 & 0xFF)
    }

    /// Check if transparent (alpha = 0)
    #[inline]
    pub fn is_transparent(&self) -> bool {
        self.a == 0
    }

    /// Check if fully opaque (alpha = 255)
    #[inline]
    pub fn is_opaque(&self) -> bool {
        self.a == 255
    }
}

// =============================================================================
// SHARED BUFFER
// =============================================================================

/// Shared buffer wrapper for zero-copy access to SharedArrayBuffer.
pub struct SharedBuffer {
    ptr: *mut u8,
    len: usize,
    max_nodes: usize,
    text_pool_size: usize,
    text_pool_offset: usize,
    event_ring_offset: usize,
}

// SAFETY: The buffer is shared with JS via SharedArrayBuffer.
// Access is coordinated through atomic operations on wake flags.
unsafe impl Send for SharedBuffer {}
unsafe impl Sync for SharedBuffer {}

impl SharedBuffer {
    /// Create from raw pointer (from FFI).
    ///
    /// # Safety
    /// - `ptr` must point to a valid SharedArrayBuffer of at least `len` bytes
    /// - The buffer must remain valid for the lifetime of this struct
    pub unsafe fn from_raw(ptr: *mut u8, len: usize) -> Self {
        // Read configuration from header
        // SAFETY: caller guarantees ptr is valid and len is sufficient
        let max_nodes = unsafe { ptr::read_unaligned(ptr.add(H_MAX_NODES) as *const u32) } as usize;
        let text_pool_size = unsafe { ptr::read_unaligned(ptr.add(H_TEXT_POOL_SIZE) as *const u32) } as usize;
        let text_pool_offset = HEADER_SIZE + max_nodes * NODE_STRIDE;
        let event_ring_offset = text_pool_offset + text_pool_size;

        Self {
            ptr,
            len,
            max_nodes,
            text_pool_size,
            text_pool_offset,
            event_ring_offset,
        }
    }

    /// Get raw pointer
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.ptr
    }

    /// Get mutable raw pointer
    #[inline]
    pub fn as_mut_ptr(&self) -> *mut u8 {
        self.ptr
    }

    /// Get buffer length
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if buffer is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get maximum nodes
    #[inline]
    pub fn max_nodes(&self) -> usize {
        self.max_nodes
    }

    // =========================================================================
    // LOW-LEVEL HEADER ACCESS
    // =========================================================================

    #[inline]
    fn read_header_u32(&self, offset: usize) -> u32 {
        unsafe { ptr::read_unaligned(self.ptr.add(offset) as *const u32) }
    }

    #[inline]
    fn write_header_u32(&self, offset: usize, value: u32) {
        unsafe { ptr::write_unaligned(self.ptr.add(offset) as *mut u32, value) }
    }

    #[inline]
    fn read_header_i32(&self, offset: usize) -> i32 {
        unsafe { ptr::read_unaligned(self.ptr.add(offset) as *const i32) }
    }

    #[inline]
    fn write_header_i32(&self, offset: usize, value: i32) {
        unsafe { ptr::write_unaligned(self.ptr.add(offset) as *mut i32, value) }
    }

    #[inline]
    fn read_header_u16(&self, offset: usize) -> u16 {
        unsafe { ptr::read_unaligned(self.ptr.add(offset) as *const u16) }
    }

    #[inline]
    fn write_header_u16(&self, offset: usize, value: u16) {
        unsafe { ptr::write_unaligned(self.ptr.add(offset) as *mut u16, value) }
    }

    #[inline]
    fn read_header_u8(&self, offset: usize) -> u8 {
        unsafe { *self.ptr.add(offset) }
    }

    #[inline]
    fn write_header_u8(&self, offset: usize, value: u8) {
        unsafe { *self.ptr.add(offset) = value }
    }

    // =========================================================================
    // HEADER ACCESSORS
    // =========================================================================

    /// Get buffer version
    #[inline]
    pub fn version(&self) -> u32 {
        self.read_header_u32(H_VERSION)
    }

    /// Get active node count
    #[inline]
    pub fn node_count(&self) -> usize {
        self.read_header_u32(H_NODE_COUNT) as usize
    }

    /// Get terminal dimensions
    #[inline]
    pub fn terminal_size(&self) -> (u32, u32) {
        (
            self.read_header_u32(H_TERMINAL_WIDTH),
            self.read_header_u32(H_TERMINAL_HEIGHT),
        )
    }

    /// Get terminal width
    #[inline]
    pub fn terminal_width(&self) -> u32 {
        self.read_header_u32(H_TERMINAL_WIDTH)
    }

    /// Get terminal height
    #[inline]
    pub fn terminal_height(&self) -> u32 {
        self.read_header_u32(H_TERMINAL_HEIGHT)
    }

    /// Get generation counter
    #[inline]
    pub fn generation(&self) -> u32 {
        self.read_header_u32(H_GENERATION)
    }

    /// Get config flags
    #[inline]
    pub fn config_flags(&self) -> ConfigFlags {
        ConfigFlags::from_bits_truncate(self.read_header_u32(H_CONFIG_FLAGS))
    }

    /// Get render mode
    #[inline]
    pub fn render_mode(&self) -> RenderMode {
        RenderMode::from(self.read_header_u32(H_RENDER_MODE) as u8)
    }

    /// Get scroll speed
    #[inline]
    pub fn scroll_speed(&self) -> u32 {
        self.read_header_u32(H_SCROLL_SPEED)
    }

    // =========================================================================
    // STATE (Rust writes, TS reads)
    // =========================================================================

    /// Get focused component index (-1 = none)
    #[inline]
    pub fn focused_index(&self) -> i32 {
        self.read_header_i32(H_FOCUSED_INDEX)
    }

    /// Set focused component index
    #[inline]
    pub fn set_focused_index(&self, idx: i32) {
        self.write_header_i32(H_FOCUSED_INDEX, idx)
    }

    /// Get hovered component index (-1 = none)
    #[inline]
    pub fn hovered_index(&self) -> i32 {
        self.read_header_i32(H_HOVERED_INDEX)
    }

    /// Set hovered component index
    #[inline]
    pub fn set_hovered_index(&self, idx: i32) {
        self.write_header_i32(H_HOVERED_INDEX, idx)
    }

    /// Get pressed component index (-1 = none)
    #[inline]
    pub fn pressed_index(&self) -> i32 {
        self.read_header_i32(H_PRESSED_INDEX)
    }

    /// Set pressed component index
    #[inline]
    pub fn set_pressed_index(&self, idx: i32) {
        self.write_header_i32(H_PRESSED_INDEX, idx)
    }

    /// Get mouse position
    #[inline]
    pub fn mouse_position(&self) -> (u16, u16) {
        (
            self.read_header_u16(H_MOUSE_X),
            self.read_header_u16(H_MOUSE_Y),
        )
    }

    /// Set mouse position
    #[inline]
    pub fn set_mouse_position(&self, x: u16, y: u16) {
        self.write_header_u16(H_MOUSE_X, x);
        self.write_header_u16(H_MOUSE_Y, y);
    }

    // =========================================================================
    // STATS
    // =========================================================================

    /// Increment render count
    #[inline]
    pub fn increment_render_count(&self) {
        let count = self.read_header_u32(H_RENDER_COUNT);
        self.write_header_u32(H_RENDER_COUNT, count.wrapping_add(1));
    }

    /// Increment layout count
    #[inline]
    pub fn increment_layout_count(&self) {
        let count = self.read_header_u32(H_LAYOUT_COUNT);
        self.write_header_u32(H_LAYOUT_COUNT, count.wrapping_add(1));
    }

    /// Check if exit has been requested
    #[inline]
    pub fn exit_requested(&self) -> bool {
        self.read_header_u8(H_EXIT_REQUESTED) != 0
    }

    /// Set exit requested flag
    #[inline]
    pub fn set_exit_requested(&self, value: bool) {
        self.write_header_u8(H_EXIT_REQUESTED, if value { 1 } else { 0 });
    }

    // =========================================================================
    // WAKE MECHANISM
    // =========================================================================

    /// Consume wake flag (read and clear atomically)
    #[inline]
    pub fn consume_wake(&self) -> bool {
        unsafe {
            let wake_ptr = self.ptr.add(H_WAKE_RUST) as *const AtomicU32;
            (*wake_ptr).swap(0, Ordering::AcqRel) != 0
        }
    }

    /// Set wake flag (TS calls this via Atomics)
    #[inline]
    pub fn set_wake_flag(&self) {
        unsafe {
            let wake_ptr = self.ptr.add(H_WAKE_RUST) as *const AtomicU32;
            (*wake_ptr).store(1, Ordering::Release);
        }
    }

    /// Wake the TypeScript side
    pub fn notify_ts(&self) {
        unsafe {
            let wake_ptr = self.ptr.add(H_WAKE_TS) as *const AtomicU32;
            (*wake_ptr).store(1, Ordering::SeqCst);
            atomic_wait::wake_one(wake_ptr);
        }
    }

    // =========================================================================
    // LOW-LEVEL NODE ACCESS
    // =========================================================================

    /// Get pointer to node data (with bounds check)
    #[inline]
    fn node_ptr(&self, index: usize) -> *const u8 {
        debug_assert!(index < self.max_nodes, "Node index {} out of bounds (max {})", index, self.max_nodes);
        unsafe { self.ptr.add(HEADER_SIZE + index * NODE_STRIDE) }
    }

    /// Get mutable pointer to node data (with bounds check)
    #[inline]
    fn node_ptr_mut(&self, index: usize) -> *mut u8 {
        debug_assert!(index < self.max_nodes, "Node index {} out of bounds (max {})", index, self.max_nodes);
        unsafe { self.ptr.add(HEADER_SIZE + index * NODE_STRIDE) }
    }

    #[inline]
    fn read_node_f32(&self, index: usize, field: usize) -> f32 {
        unsafe { ptr::read_unaligned(self.node_ptr(index).add(field) as *const f32) }
    }

    #[inline]
    fn write_node_f32(&self, index: usize, field: usize, value: f32) {
        unsafe { ptr::write_unaligned(self.node_ptr_mut(index).add(field) as *mut f32, value) }
    }

    #[inline]
    fn read_node_u8(&self, index: usize, field: usize) -> u8 {
        unsafe { *self.node_ptr(index).add(field) }
    }

    #[inline]
    fn write_node_u8(&self, index: usize, field: usize, value: u8) {
        unsafe { *self.node_ptr_mut(index).add(field) = value }
    }

    #[inline]
    fn read_node_i16(&self, index: usize, field: usize) -> i16 {
        unsafe { ptr::read_unaligned(self.node_ptr(index).add(field) as *const i16) }
    }

    #[inline]
    fn read_node_u16(&self, index: usize, field: usize) -> u16 {
        unsafe { ptr::read_unaligned(self.node_ptr(index).add(field) as *const u16) }
    }

    #[inline]
    fn read_node_u32(&self, index: usize, field: usize) -> u32 {
        unsafe { ptr::read_unaligned(self.node_ptr(index).add(field) as *const u32) }
    }

    #[inline]
    fn write_node_u32(&self, index: usize, field: usize, value: u32) {
        unsafe { ptr::write_unaligned(self.node_ptr_mut(index).add(field) as *mut u32, value) }
    }

    #[inline]
    fn read_node_i32(&self, index: usize, field: usize) -> i32 {
        unsafe { ptr::read_unaligned(self.node_ptr(index).add(field) as *const i32) }
    }

    #[inline]
    fn write_node_i32(&self, index: usize, field: usize, value: i32) {
        unsafe { ptr::write_unaligned(self.node_ptr_mut(index).add(field) as *mut i32, value) }
    }

    // =========================================================================
    // LAYOUT PROPERTIES (Cache Lines 1-4)
    // =========================================================================

    // Core dimensions
    #[inline] pub fn width(&self, i: usize) -> f32 { self.read_node_f32(i, N_WIDTH) }
    #[inline] pub fn height(&self, i: usize) -> f32 { self.read_node_f32(i, N_HEIGHT) }
    #[inline] pub fn min_width(&self, i: usize) -> f32 { self.read_node_f32(i, N_MIN_WIDTH) }
    #[inline] pub fn min_height(&self, i: usize) -> f32 { self.read_node_f32(i, N_MIN_HEIGHT) }
    #[inline] pub fn max_width(&self, i: usize) -> f32 { self.read_node_f32(i, N_MAX_WIDTH) }
    #[inline] pub fn max_height(&self, i: usize) -> f32 { self.read_node_f32(i, N_MAX_HEIGHT) }
    #[inline] pub fn aspect_ratio(&self, i: usize) -> f32 { self.read_node_f32(i, N_ASPECT_RATIO) }

    // Flex properties
    #[inline] pub fn flex_basis(&self, i: usize) -> f32 { self.read_node_f32(i, N_FLEX_BASIS) }
    #[inline] pub fn flex_grow(&self, i: usize) -> f32 { self.read_node_f32(i, N_FLEX_GROW) }
    #[inline] pub fn flex_shrink(&self, i: usize) -> f32 { self.read_node_f32(i, N_FLEX_SHRINK) }
    #[inline] pub fn gap(&self, i: usize) -> f32 { self.read_node_f32(i, N_GAP) }
    #[inline] pub fn row_gap(&self, i: usize) -> f32 { self.read_node_f32(i, N_ROW_GAP) }
    #[inline] pub fn column_gap(&self, i: usize) -> f32 { self.read_node_f32(i, N_COLUMN_GAP) }

    // Spacing
    #[inline] pub fn padding_top(&self, i: usize) -> f32 { self.read_node_f32(i, N_PADDING_TOP) }
    #[inline] pub fn padding_right(&self, i: usize) -> f32 { self.read_node_f32(i, N_PADDING_RIGHT) }
    #[inline] pub fn padding_bottom(&self, i: usize) -> f32 { self.read_node_f32(i, N_PADDING_BOTTOM) }
    #[inline] pub fn padding_left(&self, i: usize) -> f32 { self.read_node_f32(i, N_PADDING_LEFT) }
    #[inline] pub fn margin_top(&self, i: usize) -> f32 { self.read_node_f32(i, N_MARGIN_TOP) }
    #[inline] pub fn margin_right(&self, i: usize) -> f32 { self.read_node_f32(i, N_MARGIN_RIGHT) }
    #[inline] pub fn margin_bottom(&self, i: usize) -> f32 { self.read_node_f32(i, N_MARGIN_BOTTOM) }
    #[inline] pub fn margin_left(&self, i: usize) -> f32 { self.read_node_f32(i, N_MARGIN_LEFT) }
    #[inline] pub fn inset_top(&self, i: usize) -> f32 { self.read_node_f32(i, N_INSET_TOP) }
    #[inline] pub fn inset_right(&self, i: usize) -> f32 { self.read_node_f32(i, N_INSET_RIGHT) }
    #[inline] pub fn inset_bottom(&self, i: usize) -> f32 { self.read_node_f32(i, N_INSET_BOTTOM) }
    #[inline] pub fn inset_left(&self, i: usize) -> f32 { self.read_node_f32(i, N_INSET_LEFT) }

    // Layout enums - return raw u8 for direct Taffy conversion
    #[inline] pub fn flex_direction(&self, i: usize) -> u8 { self.read_node_u8(i, N_FLEX_DIRECTION) }
    #[inline] pub fn flex_wrap(&self, i: usize) -> u8 { self.read_node_u8(i, N_FLEX_WRAP) }
    #[inline] pub fn justify_content(&self, i: usize) -> u8 { self.read_node_u8(i, N_JUSTIFY_CONTENT) }
    #[inline] pub fn align_items(&self, i: usize) -> u8 { self.read_node_u8(i, N_ALIGN_ITEMS) }
    #[inline] pub fn align_content(&self, i: usize) -> u8 { self.read_node_u8(i, N_ALIGN_CONTENT) }
    #[inline] pub fn align_self(&self, i: usize) -> u8 { self.read_node_u8(i, N_ALIGN_SELF) }
    #[inline] pub fn position(&self, i: usize) -> u8 { self.read_node_u8(i, N_POSITION) }
    #[inline] pub fn overflow(&self, i: usize) -> u8 { self.read_node_u8(i, N_OVERFLOW) }
    #[inline] pub fn display(&self, i: usize) -> u8 { self.read_node_u8(i, N_DISPLAY) }
    #[inline] pub fn box_sizing(&self, i: usize) -> u8 { self.read_node_u8(i, N_BOX_SIZING) }

    // Border widths
    #[inline] pub fn border_width_top(&self, i: usize) -> u8 { self.read_node_u8(i, N_BORDER_WIDTH_TOP) }
    #[inline] pub fn border_width_right(&self, i: usize) -> u8 { self.read_node_u8(i, N_BORDER_WIDTH_RIGHT) }
    #[inline] pub fn border_width_bottom(&self, i: usize) -> u8 { self.read_node_u8(i, N_BORDER_WIDTH_BOTTOM) }
    #[inline] pub fn border_width_left(&self, i: usize) -> u8 { self.read_node_u8(i, N_BORDER_WIDTH_LEFT) }

    // Border width aliases (for layout compatibility)
    #[inline] pub fn border_top(&self, i: usize) -> u8 { self.border_width_top(i) }
    #[inline] pub fn border_right(&self, i: usize) -> u8 { self.border_width_right(i) }
    #[inline] pub fn border_bottom(&self, i: usize) -> u8 { self.border_width_bottom(i) }
    #[inline] pub fn border_left(&self, i: usize) -> u8 { self.border_width_left(i) }

    // Component type and visibility
    #[inline] pub fn component_type(&self, i: usize) -> u8 { self.read_node_u8(i, N_COMPONENT_TYPE) }
    #[inline] pub fn visible(&self, i: usize) -> bool { self.read_node_u8(i, N_VISIBLE) != 0 }

    // Hierarchy
    #[inline]
    pub fn parent_index(&self, i: usize) -> Option<usize> {
        let idx = self.read_node_i32(i, N_PARENT_INDEX);
        if idx < 0 { None } else { Some(idx as usize) }
    }

    #[inline] pub fn tab_index(&self, i: usize) -> i32 { self.read_node_i32(i, N_TAB_INDEX) }

    // Legacy child_count - reads from reserved space
    #[inline] pub fn child_count(&self, i: usize) -> i32 { self.read_node_i32(i, I_CHILD_COUNT) }

    // =========================================================================
    // GRID PROPERTIES (Cache Line 4 + Lines 5-10)
    // =========================================================================

    #[inline] pub fn grid_auto_flow(&self, i: usize) -> GridAutoFlow { GridAutoFlow::from(self.read_node_u8(i, N_GRID_AUTO_FLOW)) }
    #[inline] pub fn justify_items(&self, i: usize) -> JustifyItems { JustifyItems::from(self.read_node_u8(i, N_JUSTIFY_ITEMS)) }
    #[inline] pub fn grid_column_count(&self, i: usize) -> u8 { self.read_node_u8(i, N_GRID_COLUMN_COUNT) }
    #[inline] pub fn grid_row_count(&self, i: usize) -> u8 { self.read_node_u8(i, N_GRID_ROW_COUNT) }
    #[inline] pub fn grid_auto_columns_type(&self, i: usize) -> TrackType { TrackType::from(self.read_node_u8(i, N_GRID_AUTO_COLUMNS_TYPE)) }
    #[inline] pub fn grid_auto_rows_type(&self, i: usize) -> TrackType { TrackType::from(self.read_node_u8(i, N_GRID_AUTO_ROWS_TYPE)) }
    #[inline] pub fn grid_auto_columns_value(&self, i: usize) -> f32 { self.read_node_f32(i, N_GRID_AUTO_COLUMNS_VALUE) }
    #[inline] pub fn grid_auto_rows_value(&self, i: usize) -> f32 { self.read_node_f32(i, N_GRID_AUTO_ROWS_VALUE) }
    #[inline] pub fn grid_column_start(&self, i: usize) -> i16 { self.read_node_i16(i, N_GRID_COLUMN_START) }
    #[inline] pub fn grid_column_end(&self, i: usize) -> i16 { self.read_node_i16(i, N_GRID_COLUMN_END) }
    #[inline] pub fn grid_row_start(&self, i: usize) -> i16 { self.read_node_i16(i, N_GRID_ROW_START) }
    #[inline] pub fn grid_row_end(&self, i: usize) -> i16 { self.read_node_i16(i, N_GRID_ROW_END) }
    #[inline] pub fn justify_self(&self, i: usize) -> JustifySelf { JustifySelf::from(self.read_node_u8(i, N_JUSTIFY_SELF)) }

    /// Read a grid column track at the given index (0-31)
    pub fn grid_column_track(&self, node: usize, track_idx: usize) -> GridTrack {
        debug_assert!(track_idx < MAX_GRID_TRACKS, "Track index {} out of bounds", track_idx);
        let offset = N_GRID_COLUMN_TRACKS + track_idx * GRID_TRACK_SIZE;
        let track_type = TrackType::from(self.read_node_u8(node, offset));
        let value = self.read_node_f32(node, offset + 2);
        GridTrack { track_type, value }
    }

    /// Read a grid row track at the given index (0-31)
    pub fn grid_row_track(&self, node: usize, track_idx: usize) -> GridTrack {
        debug_assert!(track_idx < MAX_GRID_TRACKS, "Track index {} out of bounds", track_idx);
        let offset = N_GRID_ROW_TRACKS + track_idx * GRID_TRACK_SIZE;
        let track_type = TrackType::from(self.read_node_u8(node, offset));
        let value = self.read_node_f32(node, offset + 2);
        GridTrack { track_type, value }
    }

    /// Get all column tracks as a Vec (up to grid_column_count)
    pub fn grid_column_tracks(&self, node: usize) -> Vec<GridTrack> {
        let count = self.grid_column_count(node) as usize;
        (0..count.min(MAX_GRID_TRACKS))
            .map(|i| self.grid_column_track(node, i))
            .collect()
    }

    /// Get all row tracks as a Vec (up to grid_row_count)
    pub fn grid_row_tracks(&self, node: usize) -> Vec<GridTrack> {
        let count = self.grid_row_count(node) as usize;
        (0..count.min(MAX_GRID_TRACKS))
            .map(|i| self.grid_row_track(node, i))
            .collect()
    }

    // =========================================================================
    // OUTPUT (Rust writes, Cache Line 11)
    // =========================================================================

    #[inline] pub fn computed_x(&self, i: usize) -> f32 { self.read_node_f32(i, N_COMPUTED_X) }
    #[inline] pub fn computed_y(&self, i: usize) -> f32 { self.read_node_f32(i, N_COMPUTED_Y) }
    #[inline] pub fn computed_width(&self, i: usize) -> f32 { self.read_node_f32(i, N_COMPUTED_WIDTH) }
    #[inline] pub fn computed_height(&self, i: usize) -> f32 { self.read_node_f32(i, N_COMPUTED_HEIGHT) }
    #[inline] pub fn content_width(&self, i: usize) -> f32 { self.read_node_f32(i, N_CONTENT_WIDTH) }
    #[inline] pub fn content_height(&self, i: usize) -> f32 { self.read_node_f32(i, N_CONTENT_HEIGHT) }
    #[inline] pub fn max_scroll_x(&self, i: usize) -> f32 { self.read_node_f32(i, N_MAX_SCROLL_X) }
    #[inline] pub fn max_scroll_y(&self, i: usize) -> f32 { self.read_node_f32(i, N_MAX_SCROLL_Y) }
    #[inline] pub fn is_scrollable(&self, i: usize) -> bool { self.read_node_u8(i, N_IS_SCROLLABLE) != 0 }

    // Legacy aliases
    #[inline] pub fn scroll_width(&self, i: usize) -> f32 { self.content_width(i) }
    #[inline] pub fn scroll_height(&self, i: usize) -> f32 { self.content_height(i) }

    #[inline] pub fn set_computed_x(&self, i: usize, v: f32) { self.write_node_f32(i, N_COMPUTED_X, v) }
    #[inline] pub fn set_computed_y(&self, i: usize, v: f32) { self.write_node_f32(i, N_COMPUTED_Y, v) }
    #[inline] pub fn set_computed_width(&self, i: usize, v: f32) { self.write_node_f32(i, N_COMPUTED_WIDTH, v) }
    #[inline] pub fn set_computed_height(&self, i: usize, v: f32) { self.write_node_f32(i, N_COMPUTED_HEIGHT, v) }
    #[inline] pub fn set_content_width(&self, i: usize, v: f32) { self.write_node_f32(i, N_CONTENT_WIDTH, v) }
    #[inline] pub fn set_content_height(&self, i: usize, v: f32) { self.write_node_f32(i, N_CONTENT_HEIGHT, v) }
    #[inline] pub fn set_max_scroll_x(&self, i: usize, v: f32) { self.write_node_f32(i, N_MAX_SCROLL_X, v) }
    #[inline] pub fn set_max_scroll_y(&self, i: usize, v: f32) { self.write_node_f32(i, N_MAX_SCROLL_Y, v) }

    // Legacy aliases
    #[inline] pub fn set_scroll_width(&self, i: usize, v: f32) { self.set_content_width(i, v) }
    #[inline] pub fn set_scroll_height(&self, i: usize, v: f32) { self.set_content_height(i, v) }

    /// Set all scroll-related output (called by layout engine)
    #[inline]
    pub fn set_output_scroll(&self, i: usize, scrollable: bool, max_x: f32, max_y: f32) {
        self.write_node_u8(i, N_IS_SCROLLABLE, if scrollable { 1 } else { 0 });
        self.write_node_f32(i, N_MAX_SCROLL_X, max_x);
        self.write_node_f32(i, N_MAX_SCROLL_Y, max_y);
    }

    // =========================================================================
    // VISUAL PROPERTIES (Cache Line 12)
    // =========================================================================

    #[inline] pub fn opacity(&self, i: usize) -> f32 { self.read_node_f32(i, N_OPACITY) }
    #[inline] pub fn z_index(&self, i: usize) -> i32 { self.read_node_i32(i, N_Z_INDEX) }
    #[inline] pub fn border_style(&self, i: usize) -> BorderStyle { BorderStyle::from(self.read_node_u8(i, N_BORDER_STYLE)) }

    /// Get border style for top (falls back to border_style if 0)
    #[inline]
    pub fn border_style_top(&self, i: usize) -> BorderStyle {
        let s = self.read_node_u8(i, N_BORDER_STYLE_TOP);
        if s == 0 { self.border_style(i) } else { BorderStyle::from(s) }
    }

    /// Get border style for right (falls back to border_style if 0)
    #[inline]
    pub fn border_style_right(&self, i: usize) -> BorderStyle {
        let s = self.read_node_u8(i, N_BORDER_STYLE_RIGHT);
        if s == 0 { self.border_style(i) } else { BorderStyle::from(s) }
    }

    /// Get border style for bottom (falls back to border_style if 0)
    #[inline]
    pub fn border_style_bottom(&self, i: usize) -> BorderStyle {
        let s = self.read_node_u8(i, N_BORDER_STYLE_BOTTOM);
        if s == 0 { self.border_style(i) } else { BorderStyle::from(s) }
    }

    /// Get border style for left (falls back to border_style if 0)
    #[inline]
    pub fn border_style_left(&self, i: usize) -> BorderStyle {
        let s = self.read_node_u8(i, N_BORDER_STYLE_LEFT);
        if s == 0 { self.border_style(i) } else { BorderStyle::from(s) }
    }

    // Custom border chars
    #[inline] pub fn border_char_h(&self, i: usize) -> u16 { self.read_node_u16(i, N_BORDER_CHAR_H) }
    #[inline] pub fn border_char_v(&self, i: usize) -> u16 { self.read_node_u16(i, N_BORDER_CHAR_V) }
    #[inline] pub fn border_char_tl(&self, i: usize) -> u16 { self.read_node_u16(i, N_BORDER_CHAR_TL) }
    #[inline] pub fn border_char_tr(&self, i: usize) -> u16 { self.read_node_u16(i, N_BORDER_CHAR_TR) }
    #[inline] pub fn border_char_bl(&self, i: usize) -> u16 { self.read_node_u16(i, N_BORDER_CHAR_BL) }
    #[inline] pub fn border_char_br(&self, i: usize) -> u16 { self.read_node_u16(i, N_BORDER_CHAR_BR) }

    /// Get all border characters for a node, handling both predefined and custom styles.
    pub fn border_chars(&self, i: usize) -> (char, char, char, char, char, char) {
        let style = self.border_style(i);
        if style == BorderStyle::Custom {
            let h  = char::from_u32(self.border_char_h(i) as u32).unwrap_or('─');
            let v  = char::from_u32(self.border_char_v(i) as u32).unwrap_or('│');
            let tl = char::from_u32(self.border_char_tl(i) as u32).unwrap_or('┌');
            let tr = char::from_u32(self.border_char_tr(i) as u32).unwrap_or('┐');
            let bl = char::from_u32(self.border_char_bl(i) as u32).unwrap_or('└');
            let br = char::from_u32(self.border_char_br(i) as u32).unwrap_or('┘');
            (h, v, tl, tr, bl, br)
        } else {
            style.chars()
        }
    }

    // Focus indicator
    #[inline]
    pub fn focus_indicator_char(&self, i: usize) -> char {
        let ch = self.read_node_u8(i, N_FOCUS_INDICATOR_CHAR);
        if ch == 0 { '*' } else { ch as char }
    }

    #[inline]
    pub fn focus_indicator_enabled(&self, i: usize) -> bool {
        self.read_node_u8(i, N_FOCUS_INDICATOR_ENABLED) != 0
    }

    // =========================================================================
    // COLORS (Cache Line 13)
    // =========================================================================

    #[inline] pub fn fg_color(&self, i: usize) -> u32 { self.read_node_u32(i, N_FG_COLOR) }
    #[inline] pub fn bg_color(&self, i: usize) -> u32 { self.read_node_u32(i, N_BG_COLOR) }
    #[inline] pub fn border_color(&self, i: usize) -> u32 { self.read_node_u32(i, N_BORDER_COLOR) }
    #[inline] pub fn focus_ring_color(&self, i: usize) -> u32 { self.read_node_u32(i, N_FOCUS_RING_COLOR) }
    #[inline] pub fn cursor_fg_color(&self, i: usize) -> u32 { self.read_node_u32(i, N_CURSOR_FG_COLOR) }
    #[inline] pub fn cursor_bg_color(&self, i: usize) -> u32 { self.read_node_u32(i, N_CURSOR_BG_COLOR) }
    #[inline] pub fn selection_color(&self, i: usize) -> u32 { self.read_node_u32(i, N_SELECTION_COLOR) }

    /// Get border top color (falls back to border_color if 0)
    #[inline]
    pub fn border_top_color(&self, i: usize) -> u32 {
        let c = self.read_node_u32(i, N_BORDER_TOP_COLOR);
        if c == 0 { self.border_color(i) } else { c }
    }

    /// Get border right color (falls back to border_color if 0)
    #[inline]
    pub fn border_right_color(&self, i: usize) -> u32 {
        let c = self.read_node_u32(i, N_BORDER_RIGHT_COLOR);
        if c == 0 { self.border_color(i) } else { c }
    }

    /// Get border bottom color (falls back to border_color if 0)
    #[inline]
    pub fn border_bottom_color(&self, i: usize) -> u32 {
        let c = self.read_node_u32(i, N_BORDER_BOTTOM_COLOR);
        if c == 0 { self.border_color(i) } else { c }
    }

    /// Get border left color (falls back to border_color if 0)
    #[inline]
    pub fn border_left_color(&self, i: usize) -> u32 {
        let c = self.read_node_u32(i, N_BORDER_LEFT_COLOR);
        if c == 0 { self.border_color(i) } else { c }
    }

    // Rgba helpers
    #[inline] pub fn fg_rgba(&self, i: usize) -> Rgba { Rgba::from_packed(self.fg_color(i)) }
    #[inline] pub fn bg_rgba(&self, i: usize) -> Rgba { Rgba::from_packed(self.bg_color(i)) }
    #[inline] pub fn border_rgba(&self, i: usize) -> Rgba { Rgba::from_packed(self.border_color(i)) }

    // =========================================================================
    // TEXT PROPERTIES (Cache Line 14)
    // =========================================================================

    #[inline] pub fn text_offset(&self, i: usize) -> u32 { self.read_node_u32(i, N_TEXT_OFFSET) }
    #[inline] pub fn text_length(&self, i: usize) -> u32 { self.read_node_u32(i, N_TEXT_LENGTH) }
    #[inline] pub fn text_align(&self, i: usize) -> TextAlign { TextAlign::from(self.read_node_u8(i, N_TEXT_ALIGN)) }
    #[inline] pub fn text_wrap(&self, i: usize) -> TextWrap { TextWrap::from(self.read_node_u8(i, N_TEXT_WRAP)) }
    #[inline] pub fn text_overflow(&self, i: usize) -> TextOverflow { TextOverflow::from(self.read_node_u8(i, N_TEXT_OVERFLOW)) }
    #[inline] pub fn text_attrs(&self, i: usize) -> u8 { self.read_node_u8(i, N_TEXT_ATTRS) }
    #[inline] pub fn line_height(&self, i: usize) -> u8 { self.read_node_u8(i, N_LINE_HEIGHT) }
    #[inline] pub fn letter_spacing(&self, i: usize) -> u8 { self.read_node_u8(i, N_LETTER_SPACING) }
    #[inline] pub fn max_lines(&self, i: usize) -> u8 { self.read_node_u8(i, N_MAX_LINES) }

    /// Read text content from text pool
    pub fn text(&self, i: usize) -> &str {
        let offset = self.text_offset(i) as usize;
        let length = self.text_length(i) as usize;

        if length == 0 {
            return "";
        }

        let text_end = self.text_pool_offset + offset + length;
        if text_end > self.len {
            return "";
        }

        unsafe {
            let ptr = self.ptr.add(self.text_pool_offset + offset);
            let slice = std::slice::from_raw_parts(ptr, length);
            std::str::from_utf8_unchecked(slice)
        }
    }

    /// Get text pool write pointer
    #[inline]
    pub fn text_pool_write_ptr(&self) -> u32 {
        self.read_header_u32(H_TEXT_POOL_WRITE_PTR)
    }

    /// Set text pool write pointer
    #[inline]
    pub fn set_text_pool_write_ptr(&self, ptr: u32) {
        self.write_header_u32(H_TEXT_POOL_WRITE_PTR, ptr)
    }

    /// Write text content to text pool (bump allocation).
    /// Allocates new space in the text pool and updates the node's offset/length.
    /// Returns true if successful, false if pool is full.
    pub fn set_text(&self, i: usize, text: &str) -> bool {
        let bytes = text.as_bytes();
        let len = bytes.len();

        if len == 0 {
            // Empty text - just set length to 0
            self.write_node_u32(i, N_TEXT_LENGTH, 0);
            return true;
        }

        let write_ptr = self.text_pool_write_ptr() as usize;
        let text_end = write_ptr + len;

        // Check if we have space in the text pool
        if text_end > self.text_pool_size {
            return false; // Pool is full
        }

        // Write bytes to text pool
        unsafe {
            let ptr = self.ptr.add(self.text_pool_offset + write_ptr);
            ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, len);
        }

        // Update node's text offset and length
        self.write_node_u32(i, N_TEXT_OFFSET, write_ptr as u32);
        self.write_node_u32(i, N_TEXT_LENGTH, len as u32);

        // Advance write pointer
        self.set_text_pool_write_ptr(text_end as u32);

        true
    }

    // =========================================================================
    // INTERACTION STATE (Cache Line 15)
    // =========================================================================

    #[inline] pub fn scroll_x(&self, i: usize) -> i32 { self.read_node_i32(i, N_SCROLL_X) }
    #[inline] pub fn scroll_y(&self, i: usize) -> i32 { self.read_node_i32(i, N_SCROLL_Y) }
    #[inline] pub fn cursor_position(&self, i: usize) -> i32 { self.read_node_i32(i, N_CURSOR_POSITION) }
    #[inline] pub fn selection_start(&self, i: usize) -> i32 { self.read_node_i32(i, N_SELECTION_START) }
    #[inline] pub fn selection_end(&self, i: usize) -> i32 { self.read_node_i32(i, N_SELECTION_END) }
    #[inline] pub fn cursor_char(&self, i: usize) -> u32 { self.read_node_u32(i, N_CURSOR_CHAR) }
    #[inline] pub fn cursor_alt_char(&self, i: usize) -> u32 { self.read_node_u32(i, N_CURSOR_ALT_CHAR) }
    #[inline] pub fn cursor_style(&self, i: usize) -> CursorStyle { CursorStyle::from(self.read_node_u8(i, N_CURSOR_STYLE)) }
    #[inline] pub fn cursor_blink_rate(&self, i: usize) -> u8 { self.read_node_u8(i, N_CURSOR_BLINK_RATE) }
    #[inline] pub fn max_length(&self, i: usize) -> u8 { self.read_node_u8(i, N_MAX_LENGTH) }

    #[inline] pub fn set_scroll(&self, i: usize, x: i32, y: i32) {
        self.write_node_i32(i, N_SCROLL_X, x);
        self.write_node_i32(i, N_SCROLL_Y, y);
    }

    #[inline] pub fn set_cursor_position(&self, i: usize, pos: i32) {
        self.write_node_i32(i, N_CURSOR_POSITION, pos);
    }

    #[inline] pub fn set_selection(&self, i: usize, start: i32, end: i32) {
        self.write_node_i32(i, N_SELECTION_START, start);
        self.write_node_i32(i, N_SELECTION_END, end);
    }

    // Dirty flags
    #[inline] pub fn dirty_flags(&self, i: usize) -> u8 { self.read_node_u8(i, N_DIRTY_FLAGS) }
    #[inline] pub fn is_dirty(&self, i: usize, flag: u8) -> bool { (self.dirty_flags(i) & flag) != 0 }
    #[inline] pub fn clear_dirty(&self, i: usize) { self.write_node_u8(i, N_DIRTY_FLAGS, 0) }

    // Interaction flags
    #[inline] pub fn interaction_flags(&self, i: usize) -> u8 { self.read_node_u8(i, N_INTERACTION_FLAGS) }
    #[inline] pub fn focusable(&self, i: usize) -> bool { (self.interaction_flags(i) & FLAG_FOCUSABLE) != 0 }
    #[inline] pub fn is_focused(&self, i: usize) -> bool { (self.interaction_flags(i) & FLAG_FOCUSED) != 0 }
    #[inline] pub fn is_hovered(&self, i: usize) -> bool { (self.interaction_flags(i) & FLAG_HOVERED) != 0 }
    #[inline] pub fn is_pressed(&self, i: usize) -> bool { (self.interaction_flags(i) & FLAG_PRESSED) != 0 }
    #[inline] pub fn is_disabled(&self, i: usize) -> bool { (self.interaction_flags(i) & FLAG_DISABLED) != 0 }

    #[inline]
    pub fn set_focused(&self, i: usize, val: bool) {
        let flags = self.interaction_flags(i);
        let new_flags = if val { flags | FLAG_FOCUSED } else { flags & !FLAG_FOCUSED };
        self.write_node_u8(i, N_INTERACTION_FLAGS, new_flags);
    }

    #[inline]
    pub fn set_hovered(&self, i: usize, val: bool) {
        let flags = self.interaction_flags(i);
        let new_flags = if val { flags | FLAG_HOVERED } else { flags & !FLAG_HOVERED };
        self.write_node_u8(i, N_INTERACTION_FLAGS, new_flags);
    }

    #[inline]
    pub fn set_pressed(&self, i: usize, val: bool) {
        let flags = self.interaction_flags(i);
        let new_flags = if val { flags | FLAG_PRESSED } else { flags & !FLAG_PRESSED };
        self.write_node_u8(i, N_INTERACTION_FLAGS, new_flags);
    }

    // Cursor flags
    #[inline]
    pub fn cursor_visible(&self, i: usize) -> bool {
        (self.read_node_u8(i, N_CURSOR_FLAGS) & 0x01) != 0
    }

    #[inline]
    pub fn set_cursor_visible(&self, i: usize, val: bool) {
        let flags = self.read_node_u8(i, N_CURSOR_FLAGS);
        let new_flags = if val { flags | 0x01 } else { flags & !0x01 };
        self.write_node_u8(i, N_CURSOR_FLAGS, new_flags);
    }

    // =========================================================================
    // EVENT RING
    // =========================================================================

    /// Get event write index
    #[inline]
    pub fn event_write_idx(&self) -> u32 {
        self.read_header_u32(H_EVENT_WRITE_IDX)
    }

    /// Set event write index
    #[inline]
    pub fn set_event_write_idx(&self, idx: u32) {
        self.write_header_u32(H_EVENT_WRITE_IDX, idx)
    }

    /// Get event read index
    #[inline]
    pub fn event_read_idx(&self) -> u32 {
        self.read_header_u32(H_EVENT_READ_IDX)
    }

    /// Push an event to the ring buffer
    pub fn push_event(&self, event_type: EventType, component_index: u16, data: &[u8; 16]) {
        let write_idx = self.event_write_idx() as usize;
        let slot = write_idx % MAX_EVENTS;
        let offset = self.event_ring_offset + EVENT_RING_HEADER_SIZE + slot * EVENT_SLOT_SIZE;

        unsafe {
            let ptr = self.ptr.add(offset);
            *ptr = event_type as u8;
            // ptr[1] is padding
            ptr::write_unaligned(ptr.add(2) as *mut u16, component_index);
            ptr::copy_nonoverlapping(data.as_ptr(), ptr.add(4), 16);
        }

        // Set exit flag if this is an exit event
        if event_type == EventType::Exit {
            self.set_exit_requested(true);
        }

        self.set_event_write_idx((write_idx + 1) as u32);
        self.notify_ts();
    }

    /// Push a focus event
    pub fn push_focus_event(&self, component_index: u16) {
        self.push_event(EventType::Focus, component_index, &[0; 16]);
    }

    /// Push a blur event
    pub fn push_blur_event(&self, component_index: u16) {
        self.push_event(EventType::Blur, component_index, &[0; 16]);
    }

    /// Push a resize event
    pub fn push_resize_event(&self, width: u16, height: u16) {
        let mut data = [0u8; 16];
        data[0..2].copy_from_slice(&width.to_le_bytes());
        data[2..4].copy_from_slice(&height.to_le_bytes());
        self.push_event(EventType::Resize, 0xFFFF, &data);
    }

    /// Push an exit event
    pub fn push_exit_event(&self, exit_code: u8) {
        let mut data = [0u8; 16];
        data[0] = exit_code;
        self.push_event(EventType::Exit, 0xFFFF, &data);
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_buffer(max_nodes: usize, text_pool_size: usize) -> (Vec<u8>, SharedBuffer) {
        let text_pool_offset = HEADER_SIZE + max_nodes * NODE_STRIDE;
        let event_ring_offset = text_pool_offset + text_pool_size;
        let total_size = event_ring_offset + EVENT_RING_SIZE;

        let mut data = vec![0u8; total_size];
        let ptr = data.as_mut_ptr();

        // Initialize header
        unsafe {
            ptr::write_unaligned(ptr.add(H_VERSION) as *mut u32, 3);
            ptr::write_unaligned(ptr.add(H_MAX_NODES) as *mut u32, max_nodes as u32);
            ptr::write_unaligned(ptr.add(H_TEXT_POOL_SIZE) as *mut u32, text_pool_size as u32);
        }

        let buf = unsafe { SharedBuffer::from_raw(ptr, total_size) };
        (data, buf)
    }

    #[test]
    fn test_constants_alignment() {
        // Verify cache line boundaries (64-byte aligned)
        assert_eq!(N_WIDTH, 0);
        assert_eq!(N_FLEX_DIRECTION, 64);
        assert_eq!(N_PADDING_TOP, 128);
        assert_eq!(N_GRID_AUTO_FLOW, 192);
        assert_eq!(N_GRID_COLUMN_TRACKS, 256);
        assert_eq!(N_GRID_ROW_TRACKS, 448);
        assert_eq!(N_COMPUTED_X, 640);
        assert_eq!(N_OPACITY, 704);
        assert_eq!(N_FG_COLOR, 768);
        assert_eq!(N_TEXT_OFFSET, 832);
        assert_eq!(N_SCROLL_X, 896);

        // Verify stride
        assert_eq!(NODE_STRIDE, 1024);

        // Verify wake flags are 4-byte aligned
        assert_eq!(H_WAKE_RUST % 4, 0);
        assert_eq!(H_WAKE_TS % 4, 0);

        // Verify grid track regions
        assert_eq!(N_GRID_ROW_TRACKS - N_GRID_COLUMN_TRACKS, 192); // 32 tracks × 6 bytes
    }

    #[test]
    fn test_buffer_creation() {
        let (_data, buf) = create_test_buffer(100, 1024);

        assert_eq!(buf.version(), 3);
        assert_eq!(buf.max_nodes(), 100);
    }

    #[test]
    fn test_header_accessors() {
        let (_data, buf) = create_test_buffer(100, 1024);

        buf.set_focused_index(5);
        assert_eq!(buf.focused_index(), 5);

        buf.set_focused_index(-1);
        assert_eq!(buf.focused_index(), -1);

        buf.set_mouse_position(100, 50);
        assert_eq!(buf.mouse_position(), (100, 50));
    }

    #[test]
    fn test_node_layout_fields() {
        let (mut data, buf) = create_test_buffer(100, 1024);

        // Write directly to buffer
        let node_base = HEADER_SIZE + 0 * NODE_STRIDE;
        unsafe {
            let ptr = data.as_mut_ptr();
            ptr::write_unaligned(ptr.add(node_base + N_WIDTH) as *mut f32, 100.0);
            ptr::write_unaligned(ptr.add(node_base + N_HEIGHT) as *mut f32, 50.0);
            *ptr.add(node_base + N_FLEX_DIRECTION) = 1;
            *ptr.add(node_base + N_COMPONENT_TYPE) = COMPONENT_BOX;
        }

        assert_eq!(buf.width(0), 100.0);
        assert_eq!(buf.height(0), 50.0);
        assert_eq!(buf.flex_direction(0), 1); // 1 = Column
        assert_eq!(buf.component_type(0), COMPONENT_BOX);
    }

    #[test]
    fn test_output_writes() {
        let (_data, buf) = create_test_buffer(100, 1024);

        buf.set_computed_x(0, 10.0);
        buf.set_computed_y(0, 20.0);
        buf.set_computed_width(0, 100.0);
        buf.set_computed_height(0, 50.0);

        assert_eq!(buf.computed_x(0), 10.0);
        assert_eq!(buf.computed_y(0), 20.0);
        assert_eq!(buf.computed_width(0), 100.0);
        assert_eq!(buf.computed_height(0), 50.0);
    }

    #[test]
    fn test_color_accessors() {
        let (mut data, buf) = create_test_buffer(100, 1024);

        let packed = 0xFF804020u32;
        let node_base = HEADER_SIZE + 0 * NODE_STRIDE;
        unsafe {
            ptr::write_unaligned(data.as_mut_ptr().add(node_base + N_FG_COLOR) as *mut u32, packed);
        }

        assert_eq!(buf.fg_color(0), packed);

        let rgba = buf.fg_rgba(0);
        assert_eq!(rgba.r, 128);
        assert_eq!(rgba.g, 64);
        assert_eq!(rgba.b, 32);
        assert_eq!(rgba.a, 255);
    }

    #[test]
    fn test_border_fallback() {
        let (mut data, buf) = create_test_buffer(100, 1024);

        let base_color = 0xFFFF0000u32;
        let node_base = HEADER_SIZE + 0 * NODE_STRIDE;
        unsafe {
            ptr::write_unaligned(data.as_mut_ptr().add(node_base + N_BORDER_COLOR) as *mut u32, base_color);
        }

        // Per-side colors are 0, should fall back to base
        assert_eq!(buf.border_top_color(0), base_color);
        assert_eq!(buf.border_right_color(0), base_color);

        // Set specific side
        let top_color = 0xFF00FF00u32;
        unsafe {
            ptr::write_unaligned(data.as_mut_ptr().add(node_base + N_BORDER_TOP_COLOR) as *mut u32, top_color);
        }

        assert_eq!(buf.border_top_color(0), top_color);
        assert_eq!(buf.border_right_color(0), base_color);
    }

    #[test]
    fn test_interaction_flags() {
        let (mut data, buf) = create_test_buffer(100, 1024);

        let node_base = HEADER_SIZE + 0 * NODE_STRIDE;
        unsafe {
            *data.as_mut_ptr().add(node_base + N_INTERACTION_FLAGS) = FLAG_FOCUSABLE | FLAG_FOCUSED;
        }

        assert!(buf.focusable(0));
        assert!(buf.is_focused(0));
        assert!(!buf.is_hovered(0));

        buf.set_hovered(0, true);
        assert!(buf.is_hovered(0));

        buf.set_hovered(0, false);
        assert!(!buf.is_hovered(0));
    }

    #[test]
    fn test_scroll_position() {
        let (_data, buf) = create_test_buffer(100, 1024);

        buf.set_scroll(0, 100, 200);
        assert_eq!(buf.scroll_x(0), 100);
        assert_eq!(buf.scroll_y(0), 200);
    }

    #[test]
    fn test_enum_conversions() {
        assert_eq!(FlexDirection::from(0), FlexDirection::Row);
        assert_eq!(FlexDirection::from(1), FlexDirection::Column);
        assert_eq!(FlexDirection::from(255), FlexDirection::Row); // Invalid -> default

        assert_eq!(Display::from(0), Display::None);
        assert_eq!(Display::from(1), Display::Flex);
        assert_eq!(Display::from(2), Display::Grid);
        assert_eq!(Display::from(255), Display::Flex); // Invalid -> default Flex

        assert_eq!(BorderStyle::from(1), BorderStyle::Single);
        assert_eq!(BorderStyle::from(3), BorderStyle::Rounded);
        assert_eq!(BorderStyle::from(255), BorderStyle::Custom);
        assert_eq!(BorderStyle::from(200), BorderStyle::None); // Unknown -> None

        assert_eq!(EventType::from(9), EventType::Focus);
        assert_eq!(EventType::from(255), EventType::None);
    }

    #[test]
    fn test_grid_enums() {
        assert_eq!(TrackType::from(0), TrackType::None);
        assert_eq!(TrackType::from(1), TrackType::Auto);
        assert_eq!(TrackType::from(4), TrackType::Length);
        assert_eq!(TrackType::from(6), TrackType::Fr);
        assert_eq!(TrackType::from(255), TrackType::None);

        assert_eq!(GridAutoFlow::from(0), GridAutoFlow::Row);
        assert_eq!(GridAutoFlow::from(1), GridAutoFlow::Column);
        assert_eq!(GridAutoFlow::from(2), GridAutoFlow::RowDense);
        assert_eq!(GridAutoFlow::from(3), GridAutoFlow::ColumnDense);

        assert_eq!(JustifyItems::from(0), JustifyItems::Start);
        assert_eq!(JustifyItems::from(3), JustifyItems::Stretch);

        assert_eq!(JustifySelf::from(0), JustifySelf::Auto);
        assert_eq!(JustifySelf::from(4), JustifySelf::Stretch);
    }

    #[test]
    fn test_grid_track_access() {
        let (mut data, buf) = create_test_buffer(100, 1024);

        let node_base = HEADER_SIZE + 0 * NODE_STRIDE;

        // Set up grid column count
        unsafe {
            *data.as_mut_ptr().add(node_base + N_GRID_COLUMN_COUNT) = 3;
        }

        // Write track 0: 1fr
        let track0_offset = node_base + N_GRID_COLUMN_TRACKS;
        unsafe {
            let ptr = data.as_mut_ptr();
            *ptr.add(track0_offset) = TrackType::Fr as u8;
            ptr::write_unaligned(ptr.add(track0_offset + 2) as *mut f32, 1.0);
        }

        // Write track 1: 2fr
        let track1_offset = node_base + N_GRID_COLUMN_TRACKS + GRID_TRACK_SIZE;
        unsafe {
            let ptr = data.as_mut_ptr();
            *ptr.add(track1_offset) = TrackType::Fr as u8;
            ptr::write_unaligned(ptr.add(track1_offset + 2) as *mut f32, 2.0);
        }

        // Write track 2: auto
        let track2_offset = node_base + N_GRID_COLUMN_TRACKS + 2 * GRID_TRACK_SIZE;
        unsafe {
            let ptr = data.as_mut_ptr();
            *ptr.add(track2_offset) = TrackType::Auto as u8;
        }

        // Read tracks
        let t0 = buf.grid_column_track(0, 0);
        assert_eq!(t0.track_type, TrackType::Fr);
        assert_eq!(t0.value, 1.0);

        let t1 = buf.grid_column_track(0, 1);
        assert_eq!(t1.track_type, TrackType::Fr);
        assert_eq!(t1.value, 2.0);

        let t2 = buf.grid_column_track(0, 2);
        assert_eq!(t2.track_type, TrackType::Auto);

        // Get all tracks
        let tracks = buf.grid_column_tracks(0);
        assert_eq!(tracks.len(), 3);
    }

    #[test]
    fn test_rgba() {
        let rgba = Rgba::from_packed(0x80FF8040);
        assert_eq!(rgba.a, 128);
        assert_eq!(rgba.r, 255);
        assert_eq!(rgba.g, 128);
        assert_eq!(rgba.b, 64);

        assert_eq!(rgba.to_packed(), 0x80FF8040);
        assert!(!rgba.is_transparent());
        assert!(!rgba.is_opaque());

        assert!(Rgba::TRANSPARENT.is_transparent());
    }

    #[test]
    fn test_border_style_chars() {
        let (h, v, tl, tr, bl, br) = BorderStyle::Single.chars();
        assert_eq!(h, '─');
        assert_eq!(v, '│');
        assert_eq!(tl, '┌');
        assert_eq!(tr, '┐');
        assert_eq!(bl, '└');
        assert_eq!(br, '┘');

        let (h, _, tl, _, _, _) = BorderStyle::Double.chars();
        assert_eq!(h, '═');
        assert_eq!(tl, '╔');

        let (_, _, tl, tr, bl, br) = BorderStyle::Rounded.chars();
        assert_eq!(tl, '╭');
        assert_eq!(tr, '╮');
        assert_eq!(bl, '╰');
        assert_eq!(br, '╯');
    }

    #[test]
    fn test_border_style_helpers() {
        assert!(BorderStyle::Single.is_predefined());
        assert!(!BorderStyle::Custom.is_predefined());

        assert!(BorderStyle::Thick.is_heavy());
        assert!(BorderStyle::Bold.is_heavy());
        assert!(!BorderStyle::Single.is_heavy());

        assert!(BorderStyle::Dashed.is_dashed());
        assert!(BorderStyle::HeavyDotted.is_dashed());
        assert!(!BorderStyle::Single.is_dashed());
    }

    #[test]
    fn test_stats() {
        let (_data, buf) = create_test_buffer(100, 1024);

        assert!(!buf.exit_requested());
        buf.set_exit_requested(true);
        assert!(buf.exit_requested());

        buf.increment_render_count();
        buf.increment_render_count();
    }

    #[test]
    fn test_spec_checksums() {
        // These must match SHARED-BUFFER-SPEC.md checksums
        assert_eq!(HEADER_SIZE, 256, "Header size mismatch");
        assert_eq!(NODE_STRIDE, 1024, "Node stride mismatch");
        assert_eq!(N_GRID_COLUMN_TRACKS, 256, "Grid column tracks offset mismatch");
        assert_eq!(N_GRID_ROW_TRACKS, 448, "Grid row tracks offset mismatch");
        assert_eq!(N_COMPUTED_X, 640, "Output offset mismatch");
        assert_eq!(N_FG_COLOR, 768, "Colors offset mismatch");
        assert_eq!(N_TEXT_OFFSET, 832, "Text offset mismatch");
        assert_eq!(N_SCROLL_X, 896, "Scroll offset mismatch");
        assert_eq!(EVENT_SLOT_SIZE, 20, "Event slot size mismatch");
    }
}
