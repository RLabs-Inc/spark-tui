//! SparkTUI Shared Buffer - Rust Implementation
//!
//! This module implements the shared memory contract defined in SHARED-BUFFER-SPEC.md.
//! Both TypeScript and Rust MUST match this spec exactly.
//!
//! Memory Layout:
//!   - Header (256 bytes): Global state, wake flags, config
//!   - Nodes (512 bytes × MAX_NODES): Per-component data
//!   - Text Pool (configurable): UTF-8 text content
//!   - Event Ring (5,132 bytes): Rust → TS event queue
//!
//! @version 2.0
//! @date 2026-01-30

use std::ptr;
use std::sync::atomic::{AtomicU32, Ordering};

use bitflags::bitflags;

// =============================================================================
// CONSTANTS
// =============================================================================

/// Header size in bytes
pub const HEADER_SIZE: usize = 256;

/// Bytes per node (8 cache lines × 64 bytes)
pub const NODE_STRIDE: usize = 512;

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
// NODE FIELD OFFSETS (512 bytes per node)
// =============================================================================

// --- Cache Line 1 (0-63): Layout Dimensions ---
pub const F_WIDTH: usize = 0;
pub const F_HEIGHT: usize = 4;
pub const F_MIN_WIDTH: usize = 8;
pub const F_MIN_HEIGHT: usize = 12;
pub const F_MAX_WIDTH: usize = 16;
pub const F_MAX_HEIGHT: usize = 20;
pub const F_FLEX_BASIS: usize = 24;
pub const F_FLEX_GROW: usize = 28;
pub const F_FLEX_SHRINK: usize = 32;
pub const F_PADDING_TOP: usize = 36;
pub const F_PADDING_RIGHT: usize = 40;
pub const F_PADDING_BOTTOM: usize = 44;
pub const F_PADDING_LEFT: usize = 48;
pub const F_MARGIN_TOP: usize = 52;
pub const F_MARGIN_RIGHT: usize = 56;
pub const F_MARGIN_BOTTOM: usize = 60;

// --- Cache Line 2 (64-127): Layout Spacing & Enums ---
pub const F_MARGIN_LEFT: usize = 64;
pub const F_GAP: usize = 68;
pub const F_ROW_GAP: usize = 72;
pub const F_COLUMN_GAP: usize = 76;
pub const F_INSET_TOP: usize = 80;
pub const F_INSET_RIGHT: usize = 84;
pub const F_INSET_BOTTOM: usize = 88;
pub const F_INSET_LEFT: usize = 92;
pub const U_FLEX_DIRECTION: usize = 96;
pub const U_FLEX_WRAP: usize = 97;
pub const U_JUSTIFY_CONTENT: usize = 98;
pub const U_ALIGN_ITEMS: usize = 99;
pub const U_ALIGN_CONTENT: usize = 100;
pub const U_ALIGN_SELF: usize = 101;
pub const U_POSITION: usize = 102;
pub const U_OVERFLOW: usize = 103;
pub const U_DISPLAY: usize = 104;
pub const U_BORDER_WIDTH_TOP: usize = 105;
pub const U_BORDER_WIDTH_RIGHT: usize = 106;
pub const U_BORDER_WIDTH_BOTTOM: usize = 107;
pub const U_BORDER_WIDTH_LEFT: usize = 108;
pub const U_COMPONENT_TYPE: usize = 109;
pub const U_VISIBLE: usize = 110;
// 111: reserved
pub const I_PARENT_INDEX: usize = 112;
pub const I_TAB_INDEX: usize = 116;
pub const I_CHILD_COUNT: usize = 120;
// 124-127: reserved

// --- Cache Line 3 (128-191): Output & Colors ---
pub const F_COMPUTED_X: usize = 128;
pub const F_COMPUTED_Y: usize = 132;
pub const F_COMPUTED_WIDTH: usize = 136;
pub const F_COMPUTED_HEIGHT: usize = 140;
pub const F_SCROLL_WIDTH: usize = 144;
pub const F_SCROLL_HEIGHT: usize = 148;
pub const F_MAX_SCROLL_X: usize = 152;
pub const F_MAX_SCROLL_Y: usize = 156;
pub const C_FG_COLOR: usize = 160;
pub const C_BG_COLOR: usize = 164;
pub const C_BORDER_COLOR: usize = 168;
pub const C_BORDER_TOP_COLOR: usize = 172;
pub const C_BORDER_RIGHT_COLOR: usize = 176;
pub const C_BORDER_BOTTOM_COLOR: usize = 180;
pub const C_BORDER_LEFT_COLOR: usize = 184;
pub const C_FOCUS_RING_COLOR: usize = 188;

// --- Cache Line 4 (192-255): Visual Properties ---
pub const C_CURSOR_FG_COLOR: usize = 192;
pub const C_CURSOR_BG_COLOR: usize = 196;
pub const C_SELECTION_COLOR: usize = 200;
pub const U_OPACITY: usize = 204;
pub const I_Z_INDEX: usize = 205;
pub const U_BORDER_STYLE: usize = 206;
pub const U_BORDER_STYLE_TOP: usize = 207;
pub const U_BORDER_STYLE_RIGHT: usize = 208;
pub const U_BORDER_STYLE_BOTTOM: usize = 209;
pub const U_BORDER_STYLE_LEFT: usize = 210;
pub const U_SCROLLABLE_FLAGS: usize = 211;
pub const U_BORDER_CHAR_H: usize = 212;
pub const U_BORDER_CHAR_V: usize = 214;
pub const U_BORDER_CHAR_TL: usize = 216;
pub const U_BORDER_CHAR_TR: usize = 218;
pub const U_BORDER_CHAR_BL: usize = 220;
pub const U_BORDER_CHAR_BR: usize = 222;
pub const U_FOCUS_INDICATOR_CHAR: usize = 224;
pub const U_FOCUS_INDICATOR_ENABLED: usize = 225;
// 226-255: reserved

// --- Cache Line 5 (256-319): Text Properties ---
pub const U_TEXT_OFFSET: usize = 256;
pub const U_TEXT_LENGTH: usize = 260;
pub const U_TEXT_ALIGN: usize = 264;
pub const U_TEXT_WRAP: usize = 265;
pub const U_TEXT_OVERFLOW: usize = 266;
pub const U_TEXT_ATTRS: usize = 267;
pub const U_TEXT_DECORATION: usize = 268;
pub const U_TEXT_DECORATION_STYLE: usize = 269;
pub const C_TEXT_DECORATION_COLOR: usize = 270;
pub const U_LINE_HEIGHT: usize = 274;
pub const U_LETTER_SPACING: usize = 275;
pub const U_MAX_LINES: usize = 276;
// 277-319: reserved

// --- Cache Line 6 (320-383): Interaction State ---
pub const I_SCROLL_X: usize = 320;
pub const I_SCROLL_Y: usize = 324;
pub const I_CURSOR_POSITION: usize = 328;
pub const I_SELECTION_START: usize = 332;
pub const I_SELECTION_END: usize = 336;
pub const U_CURSOR_CHAR: usize = 340;
pub const U_CURSOR_ALT_CHAR: usize = 344;
pub const U_DIRTY_FLAGS: usize = 348;
pub const U_INTERACTION_FLAGS: usize = 349;
pub const U_CURSOR_FLAGS: usize = 350;
pub const U_CURSOR_STYLE: usize = 351;
pub const U_CURSOR_BLINK_RATE: usize = 352;
pub const U_MAX_LENGTH: usize = 353;
pub const U_INPUT_TYPE: usize = 354;
// 356-383: reserved

// --- Cache Line 7 (384-447): Animation (Reserved) ---
// Reserved for future animation system

// --- Cache Line 8 (448-511): Effects & Transforms (Reserved) ---
// Reserved for future effects and physics

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
}

impl From<u8> for Display {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::None,
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
/// Set the characters via `U_BORDER_CHAR_*` fields in the SharedBuffer:
/// - `U_BORDER_CHAR_H` - horizontal line
/// - `U_BORDER_CHAR_V` - vertical line
/// - `U_BORDER_CHAR_TL` - top-left corner
/// - `U_BORDER_CHAR_TR` - top-right corner
/// - `U_BORDER_CHAR_BL` - bottom-left corner
/// - `U_BORDER_CHAR_BR` - bottom-right corner
///
/// # Example
///
/// ```rust,ignore
/// let style = BorderStyle::from(buffer.read_u8(node, U_BORDER_STYLE));
/// let (h, v, tl, tr, bl, br) = style.chars();
/// ```
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

    /// User-defined characters from U_BORDER_CHAR_* fields
    /// When this variant is used, call `SharedBuffer::border_chars()` instead
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
    /// # Note
    ///
    /// For `BorderStyle::Custom`, this returns spaces. Use `SharedBuffer::border_chars()`
    /// instead to read the user-defined characters from the buffer.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let (h, v, tl, tr, bl, br) = BorderStyle::Single.chars();
    /// assert_eq!(h, '─');
    /// assert_eq!(tl, '┌');
    /// ```
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
            Self::Custom    => (' ', ' ', ' ', ' ', ' ', ' '), // Use SharedBuffer::border_chars()
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
        let max_nodes = ptr::read_unaligned(ptr.add(H_MAX_NODES) as *const u32) as usize;
        let text_pool_size = ptr::read_unaligned(ptr.add(H_TEXT_POOL_SIZE) as *const u32) as usize;
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
    fn read_node_i8(&self, index: usize, field: usize) -> i8 {
        unsafe { *self.node_ptr(index).add(field) as i8 }
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
    // LAYOUT PROPERTIES (Cache Line 1-2)
    // =========================================================================

    #[inline] pub fn width(&self, i: usize) -> f32 { self.read_node_f32(i, F_WIDTH) }
    #[inline] pub fn height(&self, i: usize) -> f32 { self.read_node_f32(i, F_HEIGHT) }
    #[inline] pub fn min_width(&self, i: usize) -> f32 { self.read_node_f32(i, F_MIN_WIDTH) }
    #[inline] pub fn min_height(&self, i: usize) -> f32 { self.read_node_f32(i, F_MIN_HEIGHT) }
    #[inline] pub fn max_width(&self, i: usize) -> f32 { self.read_node_f32(i, F_MAX_WIDTH) }
    #[inline] pub fn max_height(&self, i: usize) -> f32 { self.read_node_f32(i, F_MAX_HEIGHT) }
    #[inline] pub fn flex_basis(&self, i: usize) -> f32 { self.read_node_f32(i, F_FLEX_BASIS) }
    #[inline] pub fn flex_grow(&self, i: usize) -> f32 { self.read_node_f32(i, F_FLEX_GROW) }
    #[inline] pub fn flex_shrink(&self, i: usize) -> f32 { self.read_node_f32(i, F_FLEX_SHRINK) }
    #[inline] pub fn padding_top(&self, i: usize) -> f32 { self.read_node_f32(i, F_PADDING_TOP) }
    #[inline] pub fn padding_right(&self, i: usize) -> f32 { self.read_node_f32(i, F_PADDING_RIGHT) }
    #[inline] pub fn padding_bottom(&self, i: usize) -> f32 { self.read_node_f32(i, F_PADDING_BOTTOM) }
    #[inline] pub fn padding_left(&self, i: usize) -> f32 { self.read_node_f32(i, F_PADDING_LEFT) }
    #[inline] pub fn margin_top(&self, i: usize) -> f32 { self.read_node_f32(i, F_MARGIN_TOP) }
    #[inline] pub fn margin_right(&self, i: usize) -> f32 { self.read_node_f32(i, F_MARGIN_RIGHT) }
    #[inline] pub fn margin_bottom(&self, i: usize) -> f32 { self.read_node_f32(i, F_MARGIN_BOTTOM) }
    #[inline] pub fn margin_left(&self, i: usize) -> f32 { self.read_node_f32(i, F_MARGIN_LEFT) }
    #[inline] pub fn gap(&self, i: usize) -> f32 { self.read_node_f32(i, F_GAP) }
    #[inline] pub fn row_gap(&self, i: usize) -> f32 { self.read_node_f32(i, F_ROW_GAP) }
    #[inline] pub fn column_gap(&self, i: usize) -> f32 { self.read_node_f32(i, F_COLUMN_GAP) }
    #[inline] pub fn inset_top(&self, i: usize) -> f32 { self.read_node_f32(i, F_INSET_TOP) }
    #[inline] pub fn inset_right(&self, i: usize) -> f32 { self.read_node_f32(i, F_INSET_RIGHT) }
    #[inline] pub fn inset_bottom(&self, i: usize) -> f32 { self.read_node_f32(i, F_INSET_BOTTOM) }
    #[inline] pub fn inset_left(&self, i: usize) -> f32 { self.read_node_f32(i, F_INSET_LEFT) }

    // Layout enums - return raw u8 for direct Taffy conversion (no intermediate enum)
    // See enum definitions above for value meanings
    #[inline] pub fn flex_direction(&self, i: usize) -> u8 { self.read_node_u8(i, U_FLEX_DIRECTION) }
    #[inline] pub fn flex_wrap(&self, i: usize) -> u8 { self.read_node_u8(i, U_FLEX_WRAP) }
    #[inline] pub fn justify_content(&self, i: usize) -> u8 { self.read_node_u8(i, U_JUSTIFY_CONTENT) }
    #[inline] pub fn align_items(&self, i: usize) -> u8 { self.read_node_u8(i, U_ALIGN_ITEMS) }
    #[inline] pub fn align_content(&self, i: usize) -> u8 { self.read_node_u8(i, U_ALIGN_CONTENT) }
    #[inline] pub fn align_self(&self, i: usize) -> u8 { self.read_node_u8(i, U_ALIGN_SELF) }
    #[inline] pub fn position(&self, i: usize) -> u8 { self.read_node_u8(i, U_POSITION) }
    #[inline] pub fn overflow(&self, i: usize) -> u8 { self.read_node_u8(i, U_OVERFLOW) }
    #[inline] pub fn display(&self, i: usize) -> u8 { self.read_node_u8(i, U_DISPLAY) }

    // Border widths
    #[inline] pub fn border_width_top(&self, i: usize) -> u8 { self.read_node_u8(i, U_BORDER_WIDTH_TOP) }
    #[inline] pub fn border_width_right(&self, i: usize) -> u8 { self.read_node_u8(i, U_BORDER_WIDTH_RIGHT) }
    #[inline] pub fn border_width_bottom(&self, i: usize) -> u8 { self.read_node_u8(i, U_BORDER_WIDTH_BOTTOM) }
    #[inline] pub fn border_width_left(&self, i: usize) -> u8 { self.read_node_u8(i, U_BORDER_WIDTH_LEFT) }

    // Border width aliases (for layout compatibility)
    #[inline] pub fn border_top(&self, i: usize) -> u8 { self.border_width_top(i) }
    #[inline] pub fn border_right(&self, i: usize) -> u8 { self.border_width_right(i) }
    #[inline] pub fn border_bottom(&self, i: usize) -> u8 { self.border_width_bottom(i) }
    #[inline] pub fn border_left(&self, i: usize) -> u8 { self.border_width_left(i) }

    // Component type and visibility
    #[inline] pub fn component_type(&self, i: usize) -> u8 { self.read_node_u8(i, U_COMPONENT_TYPE) }
    #[inline] pub fn visible(&self, i: usize) -> bool { self.read_node_u8(i, U_VISIBLE) != 0 }

    // Hierarchy
    #[inline]
    pub fn parent_index(&self, i: usize) -> Option<usize> {
        let idx = self.read_node_i32(i, I_PARENT_INDEX);
        if idx < 0 { None } else { Some(idx as usize) }
    }

    #[inline] pub fn tab_index(&self, i: usize) -> i32 { self.read_node_i32(i, I_TAB_INDEX) }
    #[inline] pub fn child_count(&self, i: usize) -> i32 { self.read_node_i32(i, I_CHILD_COUNT) }

    // =========================================================================
    // OUTPUT (Rust writes, Cache Line 3)
    // =========================================================================

    #[inline] pub fn computed_x(&self, i: usize) -> f32 { self.read_node_f32(i, F_COMPUTED_X) }
    #[inline] pub fn computed_y(&self, i: usize) -> f32 { self.read_node_f32(i, F_COMPUTED_Y) }
    #[inline] pub fn computed_width(&self, i: usize) -> f32 { self.read_node_f32(i, F_COMPUTED_WIDTH) }
    #[inline] pub fn computed_height(&self, i: usize) -> f32 { self.read_node_f32(i, F_COMPUTED_HEIGHT) }
    #[inline] pub fn scroll_width(&self, i: usize) -> f32 { self.read_node_f32(i, F_SCROLL_WIDTH) }
    #[inline] pub fn scroll_height(&self, i: usize) -> f32 { self.read_node_f32(i, F_SCROLL_HEIGHT) }
    #[inline] pub fn max_scroll_x(&self, i: usize) -> f32 { self.read_node_f32(i, F_MAX_SCROLL_X) }
    #[inline] pub fn max_scroll_y(&self, i: usize) -> f32 { self.read_node_f32(i, F_MAX_SCROLL_Y) }

    #[inline] pub fn set_computed_x(&self, i: usize, v: f32) { self.write_node_f32(i, F_COMPUTED_X, v) }
    #[inline] pub fn set_computed_y(&self, i: usize, v: f32) { self.write_node_f32(i, F_COMPUTED_Y, v) }
    #[inline] pub fn set_computed_width(&self, i: usize, v: f32) { self.write_node_f32(i, F_COMPUTED_WIDTH, v) }
    #[inline] pub fn set_computed_height(&self, i: usize, v: f32) { self.write_node_f32(i, F_COMPUTED_HEIGHT, v) }
    #[inline] pub fn set_scroll_width(&self, i: usize, v: f32) { self.write_node_f32(i, F_SCROLL_WIDTH, v) }
    #[inline] pub fn set_scroll_height(&self, i: usize, v: f32) { self.write_node_f32(i, F_SCROLL_HEIGHT, v) }
    #[inline] pub fn set_max_scroll_x(&self, i: usize, v: f32) { self.write_node_f32(i, F_MAX_SCROLL_X, v) }
    #[inline] pub fn set_max_scroll_y(&self, i: usize, v: f32) { self.write_node_f32(i, F_MAX_SCROLL_Y, v) }

    /// Set all scroll-related output (called by layout engine)
    /// Note: scrollable flag is stored in U_SCROLLABLE_FLAGS
    #[inline]
    pub fn set_output_scroll(&self, i: usize, scrollable: bool, max_x: f32, max_y: f32) {
        // Set scrollable flag (bit 0 of scrollable_flags)
        let flags = if scrollable { 1u8 } else { 0u8 };
        self.write_node_u8(i, U_SCROLLABLE_FLAGS, flags);
        self.write_node_f32(i, F_MAX_SCROLL_X, max_x);
        self.write_node_f32(i, F_MAX_SCROLL_Y, max_y);
    }

    // =========================================================================
    // COLORS (Cache Line 3)
    // =========================================================================

    #[inline] pub fn fg_color(&self, i: usize) -> u32 { self.read_node_u32(i, C_FG_COLOR) }
    #[inline] pub fn bg_color(&self, i: usize) -> u32 { self.read_node_u32(i, C_BG_COLOR) }
    #[inline] pub fn border_color(&self, i: usize) -> u32 { self.read_node_u32(i, C_BORDER_COLOR) }
    #[inline] pub fn focus_ring_color(&self, i: usize) -> u32 { self.read_node_u32(i, C_FOCUS_RING_COLOR) }

    /// Get border top color (falls back to border_color if 0)
    #[inline]
    pub fn border_top_color(&self, i: usize) -> u32 {
        let c = self.read_node_u32(i, C_BORDER_TOP_COLOR);
        if c == 0 { self.border_color(i) } else { c }
    }

    /// Get border right color (falls back to border_color if 0)
    #[inline]
    pub fn border_right_color(&self, i: usize) -> u32 {
        let c = self.read_node_u32(i, C_BORDER_RIGHT_COLOR);
        if c == 0 { self.border_color(i) } else { c }
    }

    /// Get border bottom color (falls back to border_color if 0)
    #[inline]
    pub fn border_bottom_color(&self, i: usize) -> u32 {
        let c = self.read_node_u32(i, C_BORDER_BOTTOM_COLOR);
        if c == 0 { self.border_color(i) } else { c }
    }

    /// Get border left color (falls back to border_color if 0)
    #[inline]
    pub fn border_left_color(&self, i: usize) -> u32 {
        let c = self.read_node_u32(i, C_BORDER_LEFT_COLOR);
        if c == 0 { self.border_color(i) } else { c }
    }

    // Rgba helpers
    #[inline] pub fn fg_rgba(&self, i: usize) -> Rgba { Rgba::from_packed(self.fg_color(i)) }
    #[inline] pub fn bg_rgba(&self, i: usize) -> Rgba { Rgba::from_packed(self.bg_color(i)) }
    #[inline] pub fn border_rgba(&self, i: usize) -> Rgba { Rgba::from_packed(self.border_color(i)) }

    // =========================================================================
    // VISUAL PROPERTIES (Cache Line 4)
    // =========================================================================

    #[inline] pub fn cursor_fg_color(&self, i: usize) -> u32 { self.read_node_u32(i, C_CURSOR_FG_COLOR) }
    #[inline] pub fn cursor_bg_color(&self, i: usize) -> u32 { self.read_node_u32(i, C_CURSOR_BG_COLOR) }
    #[inline] pub fn selection_color(&self, i: usize) -> u32 { self.read_node_u32(i, C_SELECTION_COLOR) }
    #[inline] pub fn opacity(&self, i: usize) -> u8 { self.read_node_u8(i, U_OPACITY) }
    #[inline] pub fn opacity_f32(&self, i: usize) -> f32 { self.opacity(i) as f32 / 255.0 }
    #[inline] pub fn z_index(&self, i: usize) -> i8 { self.read_node_i8(i, I_Z_INDEX) }
    #[inline] pub fn border_style(&self, i: usize) -> BorderStyle { BorderStyle::from(self.read_node_u8(i, U_BORDER_STYLE)) }

    /// Get border style for top (falls back to border_style if 0)
    #[inline]
    pub fn border_style_top(&self, i: usize) -> BorderStyle {
        let s = self.read_node_u8(i, U_BORDER_STYLE_TOP);
        if s == 0 { self.border_style(i) } else { BorderStyle::from(s) }
    }

    /// Get border style for right (falls back to border_style if 0)
    #[inline]
    pub fn border_style_right(&self, i: usize) -> BorderStyle {
        let s = self.read_node_u8(i, U_BORDER_STYLE_RIGHT);
        if s == 0 { self.border_style(i) } else { BorderStyle::from(s) }
    }

    /// Get border style for bottom (falls back to border_style if 0)
    #[inline]
    pub fn border_style_bottom(&self, i: usize) -> BorderStyle {
        let s = self.read_node_u8(i, U_BORDER_STYLE_BOTTOM);
        if s == 0 { self.border_style(i) } else { BorderStyle::from(s) }
    }

    /// Get border style for left (falls back to border_style if 0)
    #[inline]
    pub fn border_style_left(&self, i: usize) -> BorderStyle {
        let s = self.read_node_u8(i, U_BORDER_STYLE_LEFT);
        if s == 0 { self.border_style(i) } else { BorderStyle::from(s) }
    }

    // Custom border chars
    #[inline] pub fn border_char_h(&self, i: usize) -> u16 { self.read_node_u16(i, U_BORDER_CHAR_H) }
    #[inline] pub fn border_char_v(&self, i: usize) -> u16 { self.read_node_u16(i, U_BORDER_CHAR_V) }
    #[inline] pub fn border_char_tl(&self, i: usize) -> u16 { self.read_node_u16(i, U_BORDER_CHAR_TL) }
    #[inline] pub fn border_char_tr(&self, i: usize) -> u16 { self.read_node_u16(i, U_BORDER_CHAR_TR) }
    #[inline] pub fn border_char_bl(&self, i: usize) -> u16 { self.read_node_u16(i, U_BORDER_CHAR_BL) }
    #[inline] pub fn border_char_br(&self, i: usize) -> u16 { self.read_node_u16(i, U_BORDER_CHAR_BR) }

    /// Get all border characters for a node, handling both predefined and custom styles.
    ///
    /// Returns `(horizontal, vertical, top_left, top_right, bottom_left, bottom_right)`.
    ///
    /// - For predefined styles: returns the style's built-in characters
    /// - For `BorderStyle::Custom`: reads from `U_BORDER_CHAR_*` fields
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let (h, v, tl, tr, bl, br) = buffer.border_chars(node_index);
    /// // Works for any BorderStyle, predefined or custom
    /// ```
    pub fn border_chars(&self, i: usize) -> (char, char, char, char, char, char) {
        let style = self.border_style(i);
        if style == BorderStyle::Custom {
            // Read custom chars from buffer (stored as u16 for Unicode support)
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
        let ch = self.read_node_u8(i, U_FOCUS_INDICATOR_CHAR);
        if ch == 0 { '*' } else { ch as char }
    }

    #[inline]
    pub fn focus_indicator_enabled(&self, i: usize) -> bool {
        self.read_node_u8(i, U_FOCUS_INDICATOR_ENABLED) != 0
    }

    // =========================================================================
    // TEXT PROPERTIES (Cache Line 5)
    // =========================================================================

    #[inline] pub fn text_offset(&self, i: usize) -> u32 { self.read_node_u32(i, U_TEXT_OFFSET) }
    #[inline] pub fn text_length(&self, i: usize) -> u32 { self.read_node_u32(i, U_TEXT_LENGTH) }
    #[inline] pub fn text_align(&self, i: usize) -> TextAlign { TextAlign::from(self.read_node_u8(i, U_TEXT_ALIGN)) }
    #[inline] pub fn text_wrap(&self, i: usize) -> TextWrap { TextWrap::from(self.read_node_u8(i, U_TEXT_WRAP)) }
    #[inline] pub fn text_overflow(&self, i: usize) -> TextOverflow { TextOverflow::from(self.read_node_u8(i, U_TEXT_OVERFLOW)) }
    #[inline] pub fn text_attrs(&self, i: usize) -> u8 { self.read_node_u8(i, U_TEXT_ATTRS) }
    #[inline] pub fn line_height(&self, i: usize) -> u8 { self.read_node_u8(i, U_LINE_HEIGHT) }
    #[inline] pub fn letter_spacing(&self, i: usize) -> u8 { self.read_node_u8(i, U_LETTER_SPACING) }
    #[inline] pub fn max_lines(&self, i: usize) -> u8 { self.read_node_u8(i, U_MAX_LINES) }

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

    // =========================================================================
    // INTERACTION STATE (Cache Line 6)
    // =========================================================================

    #[inline] pub fn scroll_x(&self, i: usize) -> i32 { self.read_node_i32(i, I_SCROLL_X) }
    #[inline] pub fn scroll_y(&self, i: usize) -> i32 { self.read_node_i32(i, I_SCROLL_Y) }
    #[inline] pub fn cursor_position(&self, i: usize) -> i32 { self.read_node_i32(i, I_CURSOR_POSITION) }
    #[inline] pub fn selection_start(&self, i: usize) -> i32 { self.read_node_i32(i, I_SELECTION_START) }
    #[inline] pub fn selection_end(&self, i: usize) -> i32 { self.read_node_i32(i, I_SELECTION_END) }
    #[inline] pub fn cursor_char(&self, i: usize) -> u32 { self.read_node_u32(i, U_CURSOR_CHAR) }
    #[inline] pub fn cursor_alt_char(&self, i: usize) -> u32 { self.read_node_u32(i, U_CURSOR_ALT_CHAR) }
    #[inline] pub fn cursor_style(&self, i: usize) -> CursorStyle { CursorStyle::from(self.read_node_u8(i, U_CURSOR_STYLE)) }
    #[inline] pub fn cursor_blink_rate(&self, i: usize) -> u8 { self.read_node_u8(i, U_CURSOR_BLINK_RATE) }
    #[inline] pub fn max_length(&self, i: usize) -> u8 { self.read_node_u8(i, U_MAX_LENGTH) }

    #[inline] pub fn set_scroll(&self, i: usize, x: i32, y: i32) {
        self.write_node_i32(i, I_SCROLL_X, x);
        self.write_node_i32(i, I_SCROLL_Y, y);
    }

    #[inline] pub fn set_cursor_position(&self, i: usize, pos: i32) {
        self.write_node_i32(i, I_CURSOR_POSITION, pos);
    }

    #[inline] pub fn set_selection(&self, i: usize, start: i32, end: i32) {
        self.write_node_i32(i, I_SELECTION_START, start);
        self.write_node_i32(i, I_SELECTION_END, end);
    }

    // Dirty flags
    #[inline] pub fn dirty_flags(&self, i: usize) -> u8 { self.read_node_u8(i, U_DIRTY_FLAGS) }
    #[inline] pub fn is_dirty(&self, i: usize, flag: u8) -> bool { (self.dirty_flags(i) & flag) != 0 }
    #[inline] pub fn clear_dirty(&self, i: usize) { self.write_node_u8(i, U_DIRTY_FLAGS, 0) }

    // Interaction flags
    #[inline] pub fn interaction_flags(&self, i: usize) -> u8 { self.read_node_u8(i, U_INTERACTION_FLAGS) }
    #[inline] pub fn focusable(&self, i: usize) -> bool { (self.interaction_flags(i) & FLAG_FOCUSABLE) != 0 }
    #[inline] pub fn is_focused(&self, i: usize) -> bool { (self.interaction_flags(i) & FLAG_FOCUSED) != 0 }
    #[inline] pub fn is_hovered(&self, i: usize) -> bool { (self.interaction_flags(i) & FLAG_HOVERED) != 0 }
    #[inline] pub fn is_pressed(&self, i: usize) -> bool { (self.interaction_flags(i) & FLAG_PRESSED) != 0 }
    #[inline] pub fn is_disabled(&self, i: usize) -> bool { (self.interaction_flags(i) & FLAG_DISABLED) != 0 }

    #[inline]
    pub fn set_focused(&self, i: usize, val: bool) {
        let flags = self.interaction_flags(i);
        let new_flags = if val { flags | FLAG_FOCUSED } else { flags & !FLAG_FOCUSED };
        self.write_node_u8(i, U_INTERACTION_FLAGS, new_flags);
    }

    #[inline]
    pub fn set_hovered(&self, i: usize, val: bool) {
        let flags = self.interaction_flags(i);
        let new_flags = if val { flags | FLAG_HOVERED } else { flags & !FLAG_HOVERED };
        self.write_node_u8(i, U_INTERACTION_FLAGS, new_flags);
    }

    #[inline]
    pub fn set_pressed(&self, i: usize, val: bool) {
        let flags = self.interaction_flags(i);
        let new_flags = if val { flags | FLAG_PRESSED } else { flags & !FLAG_PRESSED };
        self.write_node_u8(i, U_INTERACTION_FLAGS, new_flags);
    }

    // Cursor flags
    #[inline]
    pub fn cursor_visible(&self, i: usize) -> bool {
        (self.read_node_u8(i, U_CURSOR_FLAGS) & 0x01) != 0
    }

    #[inline]
    pub fn set_cursor_visible(&self, i: usize, val: bool) {
        let flags = self.read_node_u8(i, U_CURSOR_FLAGS);
        let new_flags = if val { flags | 0x01 } else { flags & !0x01 };
        self.write_node_u8(i, U_CURSOR_FLAGS, new_flags);
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
            ptr::write_unaligned(ptr.add(H_VERSION) as *mut u32, 2);
            ptr::write_unaligned(ptr.add(H_MAX_NODES) as *mut u32, max_nodes as u32);
            ptr::write_unaligned(ptr.add(H_TEXT_POOL_SIZE) as *mut u32, text_pool_size as u32);
        }

        let buf = unsafe { SharedBuffer::from_raw(ptr, total_size) };
        (data, buf)
    }

    #[test]
    fn test_constants_alignment() {
        // Verify cache line alignment
        assert_eq!(F_WIDTH, 0);
        assert_eq!(F_MARGIN_LEFT, 64);
        assert_eq!(F_COMPUTED_X, 128);
        assert_eq!(C_CURSOR_FG_COLOR, 192);
        assert_eq!(U_TEXT_OFFSET, 256);
        assert_eq!(I_SCROLL_X, 320);

        // Verify stride
        assert_eq!(NODE_STRIDE, 512);

        // Verify wake flags are 4-byte aligned
        assert_eq!(H_WAKE_RUST % 4, 0);
        assert_eq!(H_WAKE_TS % 4, 0);
    }

    #[test]
    fn test_buffer_creation() {
        let (_data, buf) = create_test_buffer(100, 1024);

        assert_eq!(buf.version(), 2);
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
            ptr::write_unaligned(ptr.add(node_base + F_WIDTH) as *mut f32, 100.0);
            ptr::write_unaligned(ptr.add(node_base + F_HEIGHT) as *mut f32, 50.0);
            *ptr.add(node_base + U_FLEX_DIRECTION) = 1;
            *ptr.add(node_base + U_COMPONENT_TYPE) = COMPONENT_BOX;
        }

        assert_eq!(buf.width(0), 100.0);
        assert_eq!(buf.height(0), 50.0);
        assert_eq!(buf.flex_direction(0), 1); // 1 = Column (raw u8, see FlexDirection enum)
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
            ptr::write_unaligned(data.as_mut_ptr().add(node_base + C_FG_COLOR) as *mut u32, packed);
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
            ptr::write_unaligned(data.as_mut_ptr().add(node_base + C_BORDER_COLOR) as *mut u32, base_color);
        }

        // Per-side colors are 0, should fall back to base
        assert_eq!(buf.border_top_color(0), base_color);
        assert_eq!(buf.border_right_color(0), base_color);

        // Set specific side
        let top_color = 0xFF00FF00u32;
        unsafe {
            ptr::write_unaligned(data.as_mut_ptr().add(node_base + C_BORDER_TOP_COLOR) as *mut u32, top_color);
        }

        assert_eq!(buf.border_top_color(0), top_color);
        assert_eq!(buf.border_right_color(0), base_color);
    }

    #[test]
    fn test_interaction_flags() {
        let (mut data, buf) = create_test_buffer(100, 1024);

        let node_base = HEADER_SIZE + 0 * NODE_STRIDE;
        unsafe {
            *data.as_mut_ptr().add(node_base + U_INTERACTION_FLAGS) = FLAG_FOCUSABLE | FLAG_FOCUSED;
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

        assert_eq!(BorderStyle::from(1), BorderStyle::Single);
        assert_eq!(BorderStyle::from(3), BorderStyle::Rounded);
        assert_eq!(BorderStyle::from(255), BorderStyle::Custom); // 255 = Custom
        assert_eq!(BorderStyle::from(200), BorderStyle::None); // Unknown values -> None

        assert_eq!(EventType::from(9), EventType::Focus);
        assert_eq!(EventType::from(255), EventType::None);
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
        // Single
        let (h, v, tl, tr, bl, br) = BorderStyle::Single.chars();
        assert_eq!(h, '─');
        assert_eq!(v, '│');
        assert_eq!(tl, '┌');
        assert_eq!(tr, '┐');
        assert_eq!(bl, '└');
        assert_eq!(br, '┘');

        // Double
        let (h, v, tl, tr, bl, br) = BorderStyle::Double.chars();
        assert_eq!(h, '═');
        assert_eq!(tl, '╔');

        // Rounded
        let (_, _, tl, tr, bl, br) = BorderStyle::Rounded.chars();
        assert_eq!(tl, '╭');
        assert_eq!(tr, '╮');
        assert_eq!(bl, '╰');
        assert_eq!(br, '╯');

        // Block
        let (h, v, tl, _, _, _) = BorderStyle::Block.chars();
        assert_eq!(h, '█');
        assert_eq!(v, '█');
        assert_eq!(tl, '█');

        // All variants have valid chars
        for i in 0..=13 {
            let style = BorderStyle::from(i);
            let (h, v, tl, tr, bl, br) = style.chars();
            assert!(h != '\0' && v != '\0');
            assert!(tl != '\0' && tr != '\0');
            assert!(bl != '\0' && br != '\0');
        }
    }

    #[test]
    fn test_border_style_helpers() {
        // is_predefined
        assert!(BorderStyle::Single.is_predefined());
        assert!(BorderStyle::Rounded.is_predefined());
        assert!(!BorderStyle::Custom.is_predefined());

        // is_heavy
        assert!(BorderStyle::Thick.is_heavy());
        assert!(BorderStyle::Bold.is_heavy());
        assert!(BorderStyle::HeavyDashed.is_heavy());
        assert!(!BorderStyle::Single.is_heavy());
        assert!(!BorderStyle::Dashed.is_heavy());

        // is_dashed
        assert!(BorderStyle::Dashed.is_dashed());
        assert!(BorderStyle::Dotted.is_dashed());
        assert!(BorderStyle::HeavyDashed.is_dashed());
        assert!(BorderStyle::HeavyDotted.is_dashed());
        assert!(!BorderStyle::Single.is_dashed());
        assert!(!BorderStyle::Thick.is_dashed());
    }

    #[test]
    fn test_border_style_from_u8() {
        assert_eq!(BorderStyle::from(0), BorderStyle::None);
        assert_eq!(BorderStyle::from(1), BorderStyle::Single);
        assert_eq!(BorderStyle::from(3), BorderStyle::Rounded);
        assert_eq!(BorderStyle::from(8), BorderStyle::Block);
        assert_eq!(BorderStyle::from(9), BorderStyle::DoubleHorz);
        assert_eq!(BorderStyle::from(10), BorderStyle::DoubleVert);
        assert_eq!(BorderStyle::from(13), BorderStyle::Bold);
        assert_eq!(BorderStyle::from(255), BorderStyle::Custom);
        assert_eq!(BorderStyle::from(200), BorderStyle::None); // unknown -> None
    }

    #[test]
    fn test_stats() {
        let (_data, buf) = create_test_buffer(100, 1024);

        assert_eq!(buf.exit_requested(), false);
        buf.set_exit_requested(true);
        assert_eq!(buf.exit_requested(), true);

        buf.increment_render_count();
        buf.increment_render_count();
        // Note: We can't easily read render_count without exposing it
    }
}
