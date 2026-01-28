//! SharedArrayBuffer bridge — SoA (Structure of Arrays) layout.
//!
//! This module is the CONTRACT between TypeScript and Rust.
//! Both sides define the same offsets. If you change this, change ts/bridge/shared-buffer.ts too.
//!
//! Buffer layout (v3 SoA, ~2.0MB for 4096 nodes):
//!   Header          64 bytes       (16 × u32)
//!   Dirty Flags     4,096 bytes    (1 byte per node)
//!   Float32 Fields  507,904 bytes  (31 fields × 4096 × 4)
//!   Uint32 Fields   196,608 bytes  (12 fields × 4096 × 4)
//!   Int32 Fields    212,992 bytes  (13 fields × 4096 × 4)
//!   Uint8 Fields    114,688 bytes  (28 fields × 4096)
//!   Text Pool       1,048,576 bytes
//!
//! Each field is a contiguous array of MAX_NODES elements.
//! Access pattern: base_ptr + field_index * MAX_NODES + node_index

use std::sync::atomic::{AtomicU32, Ordering};

// =============================================================================
// MEMORY LAYOUT CONSTANTS
// =============================================================================

/// Maximum nodes supported. Must match TypeScript side.
pub const MAX_NODES: usize = 4096;

/// Header size in bytes (16 x u32 = 64 bytes).
pub const HEADER_BYTES: usize = 64;
pub const HEADER_U32_COUNT: usize = 16;

// Header field offsets (in u32 units from start)
pub const HEADER_VERSION: usize = 0;
pub const HEADER_NODE_COUNT: usize = 1;
pub const HEADER_MAX_NODES: usize = 2;
pub const HEADER_TERMINAL_WIDTH: usize = 3;
pub const HEADER_TERMINAL_HEIGHT: usize = 4;
pub const HEADER_WAKE_FLAG: usize = 5;
pub const HEADER_GENERATION: usize = 6;
pub const HEADER_TEXT_POOL_WRITE_PTR: usize = 7;
pub const HEADER_TEXT_POOL_CAPACITY: usize = 8;
pub const HEADER_RENDER_COUNT: usize = 9;
// 10-15 reserved

// =============================================================================
// SOA SECTION LAYOUT
// =============================================================================

/// Field counts per type
pub const F32_FIELD_COUNT: usize = 31;
pub const U32_FIELD_COUNT: usize = 12;
pub const I32_FIELD_COUNT: usize = 13;
pub const U8_FIELD_COUNT: usize = 28;

/// Per-field array sizes in bytes
pub const F32_FIELD_BYTES: usize = MAX_NODES * 4;
pub const U32_FIELD_BYTES: usize = MAX_NODES * 4;
pub const I32_FIELD_BYTES: usize = MAX_NODES * 4;
pub const U8_FIELD_BYTES: usize = MAX_NODES;

/// Text pool size in bytes (1MB).
pub const TEXT_POOL_SIZE: usize = 1_048_576;

// Section offsets (computed incrementally)
pub const SECTION_DIRTY_OFFSET: usize = HEADER_BYTES;
pub const SECTION_DIRTY_SIZE: usize = MAX_NODES;

pub const SECTION_F32_OFFSET: usize = SECTION_DIRTY_OFFSET + SECTION_DIRTY_SIZE;
pub const SECTION_F32_SIZE: usize = F32_FIELD_COUNT * F32_FIELD_BYTES;

pub const SECTION_U32_OFFSET: usize = SECTION_F32_OFFSET + SECTION_F32_SIZE;
pub const SECTION_U32_SIZE: usize = U32_FIELD_COUNT * U32_FIELD_BYTES;

pub const SECTION_I32_OFFSET: usize = SECTION_U32_OFFSET + SECTION_U32_SIZE;
pub const SECTION_I32_SIZE: usize = I32_FIELD_COUNT * I32_FIELD_BYTES;

pub const SECTION_U8_OFFSET: usize = SECTION_I32_OFFSET + SECTION_I32_SIZE;
pub const SECTION_U8_SIZE: usize = U8_FIELD_COUNT * U8_FIELD_BYTES;

pub const SECTION_TEXT_POOL_OFFSET: usize = SECTION_U8_OFFSET + SECTION_U8_SIZE;
pub const SECTION_TEXT_POOL_SIZE: usize = TEXT_POOL_SIZE;

/// Total buffer size in bytes.
pub const TOTAL_BUFFER_SIZE: usize = SECTION_TEXT_POOL_OFFSET + SECTION_TEXT_POOL_SIZE;

// =============================================================================
// FLOAT32 FIELD INDICES (0-30)
// =============================================================================

// Layout input (0-23)
pub const F32_WIDTH: usize = 0;
pub const F32_HEIGHT: usize = 1;
pub const F32_MIN_WIDTH: usize = 2;
pub const F32_MAX_WIDTH: usize = 3;
pub const F32_MIN_HEIGHT: usize = 4;
pub const F32_MAX_HEIGHT: usize = 5;
pub const F32_GROW: usize = 6;
pub const F32_SHRINK: usize = 7;
pub const F32_BASIS: usize = 8;
pub const F32_GAP: usize = 9;
pub const F32_PADDING_TOP: usize = 10;
pub const F32_PADDING_RIGHT: usize = 11;
pub const F32_PADDING_BOTTOM: usize = 12;
pub const F32_PADDING_LEFT: usize = 13;
pub const F32_MARGIN_TOP: usize = 14;
pub const F32_MARGIN_RIGHT: usize = 15;
pub const F32_MARGIN_BOTTOM: usize = 16;
pub const F32_MARGIN_LEFT: usize = 17;
pub const F32_INSET_TOP: usize = 18;
pub const F32_INSET_RIGHT: usize = 19;
pub const F32_INSET_BOTTOM: usize = 20;
pub const F32_INSET_LEFT: usize = 21;
pub const F32_ROW_GAP: usize = 22;
pub const F32_COLUMN_GAP: usize = 23;

// Layout output (24-30) — Rust writes
pub const F32_COMPUTED_X: usize = 24;
pub const F32_COMPUTED_Y: usize = 25;
pub const F32_COMPUTED_WIDTH: usize = 26;
pub const F32_COMPUTED_HEIGHT: usize = 27;
pub const F32_SCROLLABLE: usize = 28;
pub const F32_MAX_SCROLL_X: usize = 29;
pub const F32_MAX_SCROLL_Y: usize = 30;

// =============================================================================
// UINT32 FIELD INDICES (0-11)
// =============================================================================

// Colors (0-9) — packed ARGB
pub const U32_FG_COLOR: usize = 0;
pub const U32_BG_COLOR: usize = 1;
pub const U32_BORDER_COLOR: usize = 2;
pub const U32_BORDER_COLOR_TOP: usize = 3;
pub const U32_BORDER_COLOR_RIGHT: usize = 4;
pub const U32_BORDER_COLOR_BOTTOM: usize = 5;
pub const U32_BORDER_COLOR_LEFT: usize = 6;
pub const U32_FOCUS_RING_COLOR: usize = 7;
pub const U32_CURSOR_FG_COLOR: usize = 8;
pub const U32_CURSOR_BG_COLOR: usize = 9;

// Text index (10-11)
pub const U32_TEXT_OFFSET: usize = 10;
pub const U32_TEXT_LENGTH: usize = 11;

// =============================================================================
// INT32 FIELD INDICES (0-12)
// =============================================================================

// Hierarchy
pub const I32_PARENT_INDEX: usize = 0;

// Interaction (1-12)
pub const I32_SCROLL_X: usize = 1;
pub const I32_SCROLL_Y: usize = 2;
pub const I32_TAB_INDEX: usize = 3;
pub const I32_CURSOR_POS: usize = 4;
pub const I32_SELECTION_START: usize = 5;
pub const I32_SELECTION_END: usize = 6;
pub const I32_CURSOR_CHAR: usize = 7;
pub const I32_CURSOR_ALT_CHAR: usize = 8;
pub const I32_CURSOR_BLINK_FPS: usize = 9;
pub const I32_HOVERED: usize = 10;
pub const I32_PRESSED: usize = 11;
pub const I32_CURSOR_VISIBLE: usize = 12;

// =============================================================================
// UINT8 FIELD INDICES (0-27)
// =============================================================================

// Layout enums (0-13)
pub const U8_COMPONENT_TYPE: usize = 0;
pub const U8_VISIBLE: usize = 1;
pub const U8_FLEX_DIRECTION: usize = 2;
pub const U8_FLEX_WRAP: usize = 3;
pub const U8_JUSTIFY_CONTENT: usize = 4;
pub const U8_ALIGN_ITEMS: usize = 5;
pub const U8_ALIGN_SELF: usize = 6;
pub const U8_ALIGN_CONTENT: usize = 7;
pub const U8_OVERFLOW: usize = 8;
pub const U8_POSITION: usize = 9;
pub const U8_BORDER_TOP_WIDTH: usize = 10;
pub const U8_BORDER_RIGHT_WIDTH: usize = 11;
pub const U8_BORDER_BOTTOM_WIDTH: usize = 12;
pub const U8_BORDER_LEFT_WIDTH: usize = 13;

// Visual (14-21)
pub const U8_BORDER_STYLE: usize = 14;
pub const U8_BORDER_STYLE_TOP: usize = 15;
pub const U8_BORDER_STYLE_RIGHT: usize = 16;
pub const U8_BORDER_STYLE_BOTTOM: usize = 17;
pub const U8_BORDER_STYLE_LEFT: usize = 18;
pub const U8_SHOW_FOCUS_RING: usize = 19;
pub const U8_OPACITY: usize = 20;
pub const U8_Z_INDEX: usize = 21;

// Text (22-25)
pub const U8_TEXT_ATTRS: usize = 22;
pub const U8_TEXT_ALIGN: usize = 23;
pub const U8_TEXT_WRAP: usize = 24;
pub const U8_ELLIPSIS_MODE: usize = 25;

// Interaction (26-27)
pub const U8_FOCUSABLE: usize = 26;
pub const U8_MOUSE_ENABLED: usize = 27;

// =============================================================================
// DIRTY FLAG BITS
// =============================================================================

pub const DIRTY_LAYOUT: u8 = 1 << 0;
pub const DIRTY_VISUAL: u8 = 1 << 1;
pub const DIRTY_TEXT: u8 = 1 << 2;
pub const DIRTY_INTERACTION: u8 = 1 << 3;
pub const DIRTY_HIERARCHY: u8 = 1 << 4;

// =============================================================================
// BACKWARD-COMPAT ALIASES (old constant names → new SoA names)
// =============================================================================

// Old float field names
pub const FLOAT_WIDTH: usize = F32_WIDTH;
pub const FLOAT_HEIGHT: usize = F32_HEIGHT;
pub const FLOAT_MIN_WIDTH: usize = F32_MIN_WIDTH;
pub const FLOAT_MAX_WIDTH: usize = F32_MAX_WIDTH;
pub const FLOAT_MIN_HEIGHT: usize = F32_MIN_HEIGHT;
pub const FLOAT_MAX_HEIGHT: usize = F32_MAX_HEIGHT;
pub const FLOAT_GROW: usize = F32_GROW;
pub const FLOAT_SHRINK: usize = F32_SHRINK;
pub const FLOAT_BASIS: usize = F32_BASIS;
pub const FLOAT_GAP: usize = F32_GAP;
pub const FLOAT_PADDING_TOP: usize = F32_PADDING_TOP;
pub const FLOAT_PADDING_RIGHT: usize = F32_PADDING_RIGHT;
pub const FLOAT_PADDING_BOTTOM: usize = F32_PADDING_BOTTOM;
pub const FLOAT_PADDING_LEFT: usize = F32_PADDING_LEFT;
pub const FLOAT_MARGIN_TOP: usize = F32_MARGIN_TOP;
pub const FLOAT_MARGIN_RIGHT: usize = F32_MARGIN_RIGHT;
pub const FLOAT_MARGIN_BOTTOM: usize = F32_MARGIN_BOTTOM;
pub const FLOAT_MARGIN_LEFT: usize = F32_MARGIN_LEFT;
pub const FLOAT_TOP: usize = F32_INSET_TOP;
pub const FLOAT_RIGHT: usize = F32_INSET_RIGHT;
pub const FLOAT_BOTTOM: usize = F32_INSET_BOTTOM;
pub const FLOAT_LEFT: usize = F32_INSET_LEFT;
pub const FLOAT_ROW_GAP: usize = F32_ROW_GAP;
pub const FLOAT_COLUMN_GAP: usize = F32_COLUMN_GAP;

// Old output field names
pub const OUTPUT_X: usize = F32_COMPUTED_X;
pub const OUTPUT_Y: usize = F32_COMPUTED_Y;
pub const OUTPUT_WIDTH: usize = F32_COMPUTED_WIDTH;
pub const OUTPUT_HEIGHT: usize = F32_COMPUTED_HEIGHT;
pub const OUTPUT_SCROLLABLE: usize = F32_SCROLLABLE;
pub const OUTPUT_MAX_SCROLL_X: usize = F32_MAX_SCROLL_X;
pub const OUTPUT_MAX_SCROLL_Y: usize = F32_MAX_SCROLL_Y;

// Old color field names
pub const COLOR_FG: usize = U32_FG_COLOR;
pub const COLOR_BG: usize = U32_BG_COLOR;
pub const COLOR_BORDER: usize = U32_BORDER_COLOR;
pub const COLOR_BORDER_TOP: usize = U32_BORDER_COLOR_TOP;
pub const COLOR_BORDER_RIGHT: usize = U32_BORDER_COLOR_RIGHT;
pub const COLOR_BORDER_BOTTOM: usize = U32_BORDER_COLOR_BOTTOM;
pub const COLOR_BORDER_LEFT: usize = U32_BORDER_COLOR_LEFT;
pub const COLOR_FOCUS_RING: usize = U32_FOCUS_RING_COLOR;
pub const COLOR_CURSOR_FG: usize = U32_CURSOR_FG_COLOR;
pub const COLOR_CURSOR_BG: usize = U32_CURSOR_BG_COLOR;

// Old text index names
pub const TEXT_OFFSET: usize = U32_TEXT_OFFSET;
pub const TEXT_LENGTH: usize = U32_TEXT_LENGTH;

// Old interaction field names
pub const INTERACT_SCROLL_X: usize = I32_SCROLL_X;
pub const INTERACT_SCROLL_Y: usize = I32_SCROLL_Y;
pub const INTERACT_TAB_INDEX: usize = I32_TAB_INDEX;
pub const INTERACT_CURSOR_POS: usize = I32_CURSOR_POS;
pub const INTERACT_SELECTION_START: usize = I32_SELECTION_START;
pub const INTERACT_SELECTION_END: usize = I32_SELECTION_END;
pub const INTERACT_CURSOR_CHAR: usize = I32_CURSOR_CHAR;
pub const INTERACT_CURSOR_ALT_CHAR: usize = I32_CURSOR_ALT_CHAR;
pub const INTERACT_CURSOR_BLINK_FPS: usize = I32_CURSOR_BLINK_FPS;
pub const INTERACT_HOVERED: usize = I32_HOVERED;
pub const INTERACT_PRESSED: usize = I32_PRESSED;
pub const INTERACT_CURSOR_VISIBLE: usize = I32_CURSOR_VISIBLE;

// Old metadata field names
pub const META_COMPONENT_TYPE: usize = U8_COMPONENT_TYPE;
pub const META_VISIBLE: usize = U8_VISIBLE;
pub const META_FLEX_DIRECTION: usize = U8_FLEX_DIRECTION;
pub const META_FLEX_WRAP: usize = U8_FLEX_WRAP;
pub const META_JUSTIFY_CONTENT: usize = U8_JUSTIFY_CONTENT;
pub const META_ALIGN_ITEMS: usize = U8_ALIGN_ITEMS;
pub const META_ALIGN_SELF: usize = U8_ALIGN_SELF;
pub const META_ALIGN_CONTENT: usize = U8_ALIGN_CONTENT;
pub const META_OVERFLOW: usize = U8_OVERFLOW;
pub const META_POSITION: usize = U8_POSITION;
pub const META_BORDER_TOP_WIDTH: usize = U8_BORDER_TOP_WIDTH;
pub const META_BORDER_RIGHT_WIDTH: usize = U8_BORDER_RIGHT_WIDTH;
pub const META_BORDER_BOTTOM_WIDTH: usize = U8_BORDER_BOTTOM_WIDTH;
pub const META_BORDER_LEFT_WIDTH: usize = U8_BORDER_LEFT_WIDTH;
pub const META_BORDER_STYLE: usize = U8_BORDER_STYLE;
pub const META_BORDER_STYLE_TOP: usize = U8_BORDER_STYLE_TOP;
pub const META_BORDER_STYLE_RIGHT: usize = U8_BORDER_STYLE_RIGHT;
pub const META_BORDER_STYLE_BOTTOM: usize = U8_BORDER_STYLE_BOTTOM;
pub const META_BORDER_STYLE_LEFT: usize = U8_BORDER_STYLE_LEFT;
pub const META_SHOW_FOCUS_RING: usize = U8_SHOW_FOCUS_RING;
pub const META_OPACITY: usize = U8_OPACITY;
pub const META_Z_INDEX: usize = U8_Z_INDEX;
pub const META_TEXT_ATTRS: usize = U8_TEXT_ATTRS;
pub const META_TEXT_ALIGN: usize = U8_TEXT_ALIGN;
pub const META_TEXT_WRAP: usize = U8_TEXT_WRAP;
pub const META_ELLIPSIS_MODE: usize = U8_ELLIPSIS_MODE;
pub const META_FOCUSABLE: usize = U8_FOCUSABLE;
pub const META_MOUSE_ENABLED: usize = U8_MOUSE_ENABLED;

// Old border width aliases
pub const META_BORDER_TOP: usize = U8_BORDER_TOP_WIDTH;
pub const META_BORDER_RIGHT: usize = U8_BORDER_RIGHT_WIDTH;
pub const META_BORDER_BOTTOM: usize = U8_BORDER_BOTTOM_WIDTH;
pub const META_BORDER_LEFT: usize = U8_BORDER_LEFT_WIDTH;

// =============================================================================
// SHARED BUFFER — SoA facade
// =============================================================================

/// A view into the SharedArrayBuffer from TypeScript.
///
/// SoA layout: each field is a contiguous array of MAX_NODES elements.
/// Access: field_base_ptr + node_index (no stride multiplication).
///
/// TypeScript writes layout input, Rust reads it and writes output.
pub struct SharedBuffer {
    ptr: *mut u8,
}

// SAFETY: SharedBuffer is accessed from a single Rust thread.
// The TypeScript side writes, then notifies, then Rust reads.
// No concurrent writes from both sides to the same section.
unsafe impl Send for SharedBuffer {}
unsafe impl Sync for SharedBuffer {}

impl SharedBuffer {
    /// Create a new SharedBuffer from a raw pointer.
    ///
    /// # Safety
    /// - `ptr` must point to a valid SharedArrayBuffer of at least `TOTAL_BUFFER_SIZE` bytes
    /// - The buffer must remain alive for the lifetime of this SharedBuffer
    pub unsafe fn from_ptr(ptr: *mut u8, len: usize) -> Self {
        assert!(len >= TOTAL_BUFFER_SIZE, "Buffer too small: {} < {}", len, TOTAL_BUFFER_SIZE);
        Self { ptr }
    }

    // =========================================================================
    // SoA field access helpers — raw pointer arithmetic
    // =========================================================================

    /// Get pointer to a Float32 field array (MAX_NODES f32 elements).
    #[inline]
    fn f32_field_ptr(&self, field: usize) -> *const f32 {
        unsafe { self.ptr.add(SECTION_F32_OFFSET + field * F32_FIELD_BYTES) as *const f32 }
    }

    /// Get mutable pointer to a Float32 field array.
    #[inline]
    fn f32_field_ptr_mut(&self, field: usize) -> *mut f32 {
        unsafe { self.ptr.add(SECTION_F32_OFFSET + field * F32_FIELD_BYTES) as *mut f32 }
    }

    /// Read a Float32 field value for a node.
    #[inline]
    fn f32_val(&self, field: usize, node: usize) -> f32 {
        unsafe { *self.f32_field_ptr(field).add(node) }
    }

    /// Write a Float32 field value for a node.
    #[inline]
    fn set_f32_val(&self, field: usize, node: usize, value: f32) {
        unsafe { *self.f32_field_ptr_mut(field).add(node) = value; }
    }

    /// Get pointer to a Uint32 field array (MAX_NODES u32 elements).
    #[inline]
    fn u32_field_ptr(&self, field: usize) -> *const u32 {
        unsafe { self.ptr.add(SECTION_U32_OFFSET + field * U32_FIELD_BYTES) as *const u32 }
    }

    /// Read a Uint32 field value for a node.
    #[inline]
    fn u32_val(&self, field: usize, node: usize) -> u32 {
        unsafe { *self.u32_field_ptr(field).add(node) }
    }

    /// Get pointer to an Int32 field array (MAX_NODES i32 elements).
    #[inline]
    fn i32_field_ptr(&self, field: usize) -> *const i32 {
        unsafe { self.ptr.add(SECTION_I32_OFFSET + field * I32_FIELD_BYTES) as *const i32 }
    }

    /// Get mutable pointer to an Int32 field array.
    #[inline]
    fn i32_field_ptr_mut(&self, field: usize) -> *mut i32 {
        unsafe { self.ptr.add(SECTION_I32_OFFSET + field * I32_FIELD_BYTES) as *mut i32 }
    }

    /// Read an Int32 field value for a node.
    #[inline]
    fn i32_val(&self, field: usize, node: usize) -> i32 {
        unsafe { *self.i32_field_ptr(field).add(node) }
    }

    /// Write an Int32 field value for a node.
    #[inline]
    fn set_i32_val(&self, field: usize, node: usize, value: i32) {
        unsafe { *self.i32_field_ptr_mut(field).add(node) = value; }
    }

    /// Get pointer to a Uint8 field array (MAX_NODES u8 elements).
    #[inline]
    fn u8_field_ptr(&self, field: usize) -> *const u8 {
        unsafe { self.ptr.add(SECTION_U8_OFFSET + field * U8_FIELD_BYTES) }
    }

    /// Read a Uint8 field value for a node.
    #[inline]
    fn u8_val(&self, field: usize, node: usize) -> u8 {
        unsafe { *self.u8_field_ptr(field).add(node) }
    }

    /// Write a Uint8 field value for a node.
    #[inline]
    fn set_u8_val(&self, field: usize, node: usize, value: u8) {
        unsafe { *(self.ptr.add(SECTION_U8_OFFSET + field * U8_FIELD_BYTES).add(node)) = value; }
    }

    // =========================================================================
    // Header access
    // =========================================================================

    fn header(&self) -> &[AtomicU32] {
        unsafe {
            std::slice::from_raw_parts(
                self.ptr as *const AtomicU32,
                HEADER_U32_COUNT,
            )
        }
    }

    pub fn version(&self) -> u32 {
        self.header()[HEADER_VERSION].load(Ordering::Relaxed)
    }

    pub fn node_count(&self) -> usize {
        self.header()[HEADER_NODE_COUNT].load(Ordering::Relaxed) as usize
    }

    pub fn terminal_width(&self) -> u16 {
        self.header()[HEADER_TERMINAL_WIDTH].load(Ordering::Relaxed) as u16
    }

    pub fn terminal_height(&self) -> u16 {
        self.header()[HEADER_TERMINAL_HEIGHT].load(Ordering::Relaxed) as u16
    }

    pub fn generation(&self) -> u32 {
        self.header()[HEADER_GENERATION].load(Ordering::Relaxed)
    }

    pub fn text_pool_write_ptr(&self) -> usize {
        self.header()[HEADER_TEXT_POOL_WRITE_PTR].load(Ordering::Relaxed) as usize
    }

    pub fn text_pool_capacity(&self) -> usize {
        self.header()[HEADER_TEXT_POOL_CAPACITY].load(Ordering::Relaxed) as usize
    }

    /// Read and clear the wake flag atomically.
    pub fn consume_wake(&self) -> bool {
        self.header()[HEADER_WAKE_FLAG].swap(0, Ordering::AcqRel) != 0
    }

    // =========================================================================
    // Dirty flags (per-node byte in dedicated section)
    // =========================================================================

    fn dirty_ptr(&self) -> *mut u8 {
        unsafe { self.ptr.add(SECTION_DIRTY_OFFSET) }
    }

    pub fn dirty_flags(&self, node: usize) -> u8 {
        unsafe { *self.dirty_ptr().add(node) }
    }

    pub fn is_dirty(&self, node: usize, flag: u8) -> bool {
        self.dirty_flags(node) & flag != 0
    }

    pub fn clear_dirty(&self, node: usize, flag: u8) {
        unsafe {
            let p = self.dirty_ptr().add(node);
            *p &= !flag;
        }
    }

    pub fn clear_all_dirty(&self, node: usize) {
        unsafe { *self.dirty_ptr().add(node) = 0; }
    }

    // =========================================================================
    // Layout metadata readers (u8 fields)
    // =========================================================================

    pub fn component_type(&self, node: usize) -> u8 {
        self.u8_val(U8_COMPONENT_TYPE, node)
    }

    pub fn visible(&self, node: usize) -> bool {
        self.u8_val(U8_VISIBLE, node) != 0
    }

    pub fn flex_direction(&self, node: usize) -> u8 {
        self.u8_val(U8_FLEX_DIRECTION, node)
    }

    pub fn flex_wrap(&self, node: usize) -> u8 {
        self.u8_val(U8_FLEX_WRAP, node)
    }

    pub fn justify_content(&self, node: usize) -> u8 {
        self.u8_val(U8_JUSTIFY_CONTENT, node)
    }

    pub fn align_items(&self, node: usize) -> u8 {
        self.u8_val(U8_ALIGN_ITEMS, node)
    }

    pub fn align_self(&self, node: usize) -> u8 {
        self.u8_val(U8_ALIGN_SELF, node)
    }

    pub fn align_content(&self, node: usize) -> u8 {
        self.u8_val(U8_ALIGN_CONTENT, node)
    }

    pub fn overflow(&self, node: usize) -> u8 {
        self.u8_val(U8_OVERFLOW, node)
    }

    pub fn position(&self, node: usize) -> u8 {
        self.u8_val(U8_POSITION, node)
    }

    pub fn border_top_width(&self, node: usize) -> u8 {
        self.u8_val(U8_BORDER_TOP_WIDTH, node)
    }

    pub fn border_right_width(&self, node: usize) -> u8 {
        self.u8_val(U8_BORDER_RIGHT_WIDTH, node)
    }

    pub fn border_bottom_width(&self, node: usize) -> u8 {
        self.u8_val(U8_BORDER_BOTTOM_WIDTH, node)
    }

    pub fn border_left_width(&self, node: usize) -> u8 {
        self.u8_val(U8_BORDER_LEFT_WIDTH, node)
    }

    // Backward-compat aliases
    pub fn border_top(&self, node: usize) -> u8 { self.border_top_width(node) }
    pub fn border_right(&self, node: usize) -> u8 { self.border_right_width(node) }
    pub fn border_bottom(&self, node: usize) -> u8 { self.border_bottom_width(node) }
    pub fn border_left(&self, node: usize) -> u8 { self.border_left_width(node) }

    // Visual metadata
    pub fn border_style(&self, node: usize) -> u8 {
        self.u8_val(U8_BORDER_STYLE, node)
    }

    pub fn border_style_top(&self, node: usize) -> u8 {
        self.u8_val(U8_BORDER_STYLE_TOP, node)
    }

    pub fn border_style_right(&self, node: usize) -> u8 {
        self.u8_val(U8_BORDER_STYLE_RIGHT, node)
    }

    pub fn border_style_bottom(&self, node: usize) -> u8 {
        self.u8_val(U8_BORDER_STYLE_BOTTOM, node)
    }

    pub fn border_style_left(&self, node: usize) -> u8 {
        self.u8_val(U8_BORDER_STYLE_LEFT, node)
    }

    pub fn show_focus_ring(&self, node: usize) -> bool {
        self.u8_val(U8_SHOW_FOCUS_RING, node) != 0
    }

    /// Opacity as u8 (0-255). Maps to 0.0-1.0.
    pub fn opacity(&self, node: usize) -> u8 {
        self.u8_val(U8_OPACITY, node)
    }

    /// Opacity as float (0.0-1.0).
    pub fn opacity_f32(&self, node: usize) -> f32 {
        self.u8_val(U8_OPACITY, node) as f32 / 255.0
    }

    /// Z-index as u8 (0-255, 128 = default neutral).
    pub fn z_index(&self, node: usize) -> u8 {
        self.u8_val(U8_Z_INDEX, node)
    }

    // Text metadata
    pub fn text_attrs(&self, node: usize) -> u8 {
        self.u8_val(U8_TEXT_ATTRS, node)
    }

    pub fn text_align(&self, node: usize) -> u8 {
        self.u8_val(U8_TEXT_ALIGN, node)
    }

    pub fn text_wrap(&self, node: usize) -> u8 {
        self.u8_val(U8_TEXT_WRAP, node)
    }

    pub fn ellipsis_mode(&self, node: usize) -> u8 {
        self.u8_val(U8_ELLIPSIS_MODE, node)
    }

    // Interaction metadata
    pub fn focusable(&self, node: usize) -> bool {
        self.u8_val(U8_FOCUSABLE, node) != 0
    }

    pub fn mouse_enabled(&self, node: usize) -> bool {
        self.u8_val(U8_MOUSE_ENABLED, node) != 0
    }

    // =========================================================================
    // Float section — SoA layout
    // =========================================================================

    pub fn width(&self, node: usize) -> f32 { self.f32_val(F32_WIDTH, node) }
    pub fn height(&self, node: usize) -> f32 { self.f32_val(F32_HEIGHT, node) }
    pub fn min_width(&self, node: usize) -> f32 { self.f32_val(F32_MIN_WIDTH, node) }
    pub fn max_width(&self, node: usize) -> f32 { self.f32_val(F32_MAX_WIDTH, node) }
    pub fn min_height(&self, node: usize) -> f32 { self.f32_val(F32_MIN_HEIGHT, node) }
    pub fn max_height(&self, node: usize) -> f32 { self.f32_val(F32_MAX_HEIGHT, node) }
    pub fn grow(&self, node: usize) -> f32 { self.f32_val(F32_GROW, node) }
    pub fn shrink(&self, node: usize) -> f32 { self.f32_val(F32_SHRINK, node) }
    pub fn basis(&self, node: usize) -> f32 { self.f32_val(F32_BASIS, node) }
    pub fn gap(&self, node: usize) -> f32 { self.f32_val(F32_GAP, node) }
    pub fn padding_top(&self, node: usize) -> f32 { self.f32_val(F32_PADDING_TOP, node) }
    pub fn padding_right(&self, node: usize) -> f32 { self.f32_val(F32_PADDING_RIGHT, node) }
    pub fn padding_bottom(&self, node: usize) -> f32 { self.f32_val(F32_PADDING_BOTTOM, node) }
    pub fn padding_left(&self, node: usize) -> f32 { self.f32_val(F32_PADDING_LEFT, node) }
    pub fn margin_top(&self, node: usize) -> f32 { self.f32_val(F32_MARGIN_TOP, node) }
    pub fn margin_right(&self, node: usize) -> f32 { self.f32_val(F32_MARGIN_RIGHT, node) }
    pub fn margin_bottom(&self, node: usize) -> f32 { self.f32_val(F32_MARGIN_BOTTOM, node) }
    pub fn margin_left(&self, node: usize) -> f32 { self.f32_val(F32_MARGIN_LEFT, node) }
    pub fn inset_top(&self, node: usize) -> f32 { self.f32_val(F32_INSET_TOP, node) }
    pub fn inset_right(&self, node: usize) -> f32 { self.f32_val(F32_INSET_RIGHT, node) }
    pub fn inset_bottom(&self, node: usize) -> f32 { self.f32_val(F32_INSET_BOTTOM, node) }
    pub fn inset_left(&self, node: usize) -> f32 { self.f32_val(F32_INSET_LEFT, node) }
    pub fn row_gap(&self, node: usize) -> f32 { self.f32_val(F32_ROW_GAP, node) }
    pub fn column_gap(&self, node: usize) -> f32 { self.f32_val(F32_COLUMN_GAP, node) }

    // =========================================================================
    // Colors section — SoA layout (packed ARGB u32)
    // =========================================================================

    pub fn fg_color(&self, node: usize) -> u32 { self.u32_val(U32_FG_COLOR, node) }
    pub fn bg_color(&self, node: usize) -> u32 { self.u32_val(U32_BG_COLOR, node) }
    pub fn border_color(&self, node: usize) -> u32 { self.u32_val(U32_BORDER_COLOR, node) }
    pub fn border_color_top(&self, node: usize) -> u32 { self.u32_val(U32_BORDER_COLOR_TOP, node) }
    pub fn border_color_right(&self, node: usize) -> u32 { self.u32_val(U32_BORDER_COLOR_RIGHT, node) }
    pub fn border_color_bottom(&self, node: usize) -> u32 { self.u32_val(U32_BORDER_COLOR_BOTTOM, node) }
    pub fn border_color_left(&self, node: usize) -> u32 { self.u32_val(U32_BORDER_COLOR_LEFT, node) }
    pub fn focus_ring_color(&self, node: usize) -> u32 { self.u32_val(U32_FOCUS_RING_COLOR, node) }
    pub fn cursor_fg_color(&self, node: usize) -> u32 { self.u32_val(U32_CURSOR_FG_COLOR, node) }
    pub fn cursor_bg_color(&self, node: usize) -> u32 { self.u32_val(U32_CURSOR_BG_COLOR, node) }

    /// Unpack ARGB u32 → (r, g, b, a). Returns None if 0 (inherit).
    pub fn unpack_color(packed: u32) -> Option<(u8, u8, u8, u8)> {
        if packed == 0 {
            return None;
        }
        Some((
            ((packed >> 16) & 0xFF) as u8,
            ((packed >> 8) & 0xFF) as u8,
            (packed & 0xFF) as u8,
            ((packed >> 24) & 0xFF) as u8,
        ))
    }

    /// Unpack ARGB u32 → Rgba.
    pub fn unpack_to_rgba(packed: u32) -> crate::types::Rgba {
        if packed == 0 {
            return crate::types::Rgba::TERMINAL_DEFAULT;
        }
        let a = ((packed >> 24) & 0xFF) as u8;
        if a == 1 {
            let index = ((packed >> 16) & 0xFF) as u8;
            return crate::types::Rgba::ansi(index);
        }
        crate::types::Rgba::new(
            ((packed >> 16) & 0xFF) as u8,
            ((packed >> 8) & 0xFF) as u8,
            (packed & 0xFF) as u8,
            a,
        )
    }

    pub fn fg_rgba(&self, node: usize) -> crate::types::Rgba {
        Self::unpack_to_rgba(self.fg_color(node))
    }

    pub fn bg_rgba(&self, node: usize) -> crate::types::Rgba {
        Self::unpack_to_rgba(self.bg_color(node))
    }

    pub fn border_rgba(&self, node: usize) -> crate::types::Rgba {
        Self::unpack_to_rgba(self.border_color(node))
    }

    pub fn border_color_top_rgba(&self, node: usize) -> crate::types::Rgba {
        let c = self.border_color_top(node);
        if c != 0 { Self::unpack_to_rgba(c) } else { self.border_rgba(node) }
    }

    pub fn border_color_right_rgba(&self, node: usize) -> crate::types::Rgba {
        let c = self.border_color_right(node);
        if c != 0 { Self::unpack_to_rgba(c) } else { self.border_rgba(node) }
    }

    pub fn border_color_bottom_rgba(&self, node: usize) -> crate::types::Rgba {
        let c = self.border_color_bottom(node);
        if c != 0 { Self::unpack_to_rgba(c) } else { self.border_rgba(node) }
    }

    pub fn border_color_left_rgba(&self, node: usize) -> crate::types::Rgba {
        let c = self.border_color_left(node);
        if c != 0 { Self::unpack_to_rgba(c) } else { self.border_rgba(node) }
    }

    pub fn cursor_fg_rgba(&self, node: usize) -> crate::types::Rgba {
        Self::unpack_to_rgba(self.cursor_fg_color(node))
    }

    pub fn cursor_bg_rgba(&self, node: usize) -> crate::types::Rgba {
        Self::unpack_to_rgba(self.cursor_bg_color(node))
    }

    // =========================================================================
    // Interaction section — SoA layout (i32)
    // =========================================================================

    pub fn scroll_offset_x(&self, node: usize) -> i32 { self.i32_val(I32_SCROLL_X, node) }
    pub fn scroll_offset_y(&self, node: usize) -> i32 { self.i32_val(I32_SCROLL_Y, node) }
    pub fn tab_index(&self, node: usize) -> i32 { self.i32_val(I32_TAB_INDEX, node) }
    pub fn cursor_position(&self, node: usize) -> i32 { self.i32_val(I32_CURSOR_POS, node) }
    pub fn selection_start(&self, node: usize) -> i32 { self.i32_val(I32_SELECTION_START, node) }
    pub fn selection_end(&self, node: usize) -> i32 { self.i32_val(I32_SELECTION_END, node) }
    pub fn cursor_char(&self, node: usize) -> i32 { self.i32_val(I32_CURSOR_CHAR, node) }
    pub fn cursor_alt_char(&self, node: usize) -> i32 { self.i32_val(I32_CURSOR_ALT_CHAR, node) }
    pub fn cursor_blink_fps(&self, node: usize) -> i32 { self.i32_val(I32_CURSOR_BLINK_FPS, node) }
    pub fn hovered(&self, node: usize) -> bool { self.i32_val(I32_HOVERED, node) != 0 }
    pub fn pressed(&self, node: usize) -> bool { self.i32_val(I32_PRESSED, node) != 0 }
    pub fn cursor_visible(&self, node: usize) -> bool { self.i32_val(I32_CURSOR_VISIBLE, node) != 0 }

    // Rust writes
    pub fn set_hovered(&self, node: usize, hovered: bool) {
        self.set_i32_val(I32_HOVERED, node, hovered as i32);
    }

    pub fn set_pressed(&self, node: usize, pressed: bool) {
        self.set_i32_val(I32_PRESSED, node, pressed as i32);
    }

    pub fn set_cursor_visible(&self, node: usize, visible: bool) {
        self.set_i32_val(I32_CURSOR_VISIBLE, node, visible as i32);
    }

    // =========================================================================
    // Hierarchy — SoA layout (i32, field 0)
    // =========================================================================

    pub fn parent_index(&self, node: usize) -> Option<usize> {
        let val = self.i32_val(I32_PARENT_INDEX, node);
        if val < 0 { None } else { Some(val as usize) }
    }

    // =========================================================================
    // Output section — Rust writes (f32, fields 24-30)
    // =========================================================================

    pub fn set_output(&self, node: usize, x: f32, y: f32, w: f32, h: f32) {
        self.set_f32_val(F32_COMPUTED_X, node, x);
        self.set_f32_val(F32_COMPUTED_Y, node, y);
        self.set_f32_val(F32_COMPUTED_WIDTH, node, w);
        self.set_f32_val(F32_COMPUTED_HEIGHT, node, h);
    }

    pub fn set_output_scroll(&self, node: usize, scrollable: bool, max_scroll_x: f32, max_scroll_y: f32) {
        self.set_f32_val(F32_SCROLLABLE, node, if scrollable { 1.0 } else { 0.0 });
        self.set_f32_val(F32_MAX_SCROLL_X, node, max_scroll_x);
        self.set_f32_val(F32_MAX_SCROLL_Y, node, max_scroll_y);
    }

    pub fn output_x(&self, node: usize) -> f32 { self.f32_val(F32_COMPUTED_X, node) }
    pub fn output_y(&self, node: usize) -> f32 { self.f32_val(F32_COMPUTED_Y, node) }
    pub fn output_width(&self, node: usize) -> f32 { self.f32_val(F32_COMPUTED_WIDTH, node) }
    pub fn output_height(&self, node: usize) -> f32 { self.f32_val(F32_COMPUTED_HEIGHT, node) }
    pub fn output_scrollable(&self, node: usize) -> bool { self.f32_val(F32_SCROLLABLE, node) != 0.0 }
    pub fn output_max_scroll_x(&self, node: usize) -> f32 { self.f32_val(F32_MAX_SCROLL_X, node) }
    pub fn output_max_scroll_y(&self, node: usize) -> f32 { self.f32_val(F32_MAX_SCROLL_Y, node) }

    pub fn set_scroll_offset(&self, node: usize, x: i32, y: i32) {
        self.set_i32_val(I32_SCROLL_X, node, x);
        self.set_i32_val(I32_SCROLL_Y, node, y);
    }

    pub fn set_cursor_position(&self, node: usize, pos: i32) {
        self.set_i32_val(I32_CURSOR_POS, node, pos);
    }

    pub fn set_selection(&self, node: usize, start: i32, end: i32) {
        self.set_i32_val(I32_SELECTION_START, node, start);
        self.set_i32_val(I32_SELECTION_END, node, end);
    }

    // =========================================================================
    // Text section (u32 text index + u8 text pool)
    // =========================================================================

    pub fn text_offset(&self, node: usize) -> usize {
        self.u32_val(U32_TEXT_OFFSET, node) as usize
    }

    pub fn text_length(&self, node: usize) -> usize {
        self.u32_val(U32_TEXT_LENGTH, node) as usize
    }

    fn text_pool_slice(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.ptr.add(SECTION_TEXT_POOL_OFFSET),
                TEXT_POOL_SIZE,
            )
        }
    }

    pub fn text_content(&self, node: usize) -> &str {
        let offset = self.text_offset(node);
        let length = self.text_length(node);
        if length == 0 {
            return "";
        }
        let pool = self.text_pool_slice();
        let end = (offset + length).min(pool.len());
        std::str::from_utf8(&pool[offset..end]).unwrap_or("")
    }

    pub fn write_text(&self, node: usize, text: &str) {
        let pool = unsafe {
            std::slice::from_raw_parts_mut(
                self.ptr.add(SECTION_TEXT_POOL_OFFSET),
                TEXT_POOL_SIZE,
            )
        };
        let write_ptr = self.text_pool_write_ptr();
        let bytes = text.as_bytes();
        let end = (write_ptr + bytes.len()).min(TEXT_POOL_SIZE);
        let actual_len = end - write_ptr;
        pool[write_ptr..end].copy_from_slice(&bytes[..actual_len]);
        // Update text index via SoA u32 fields
        unsafe {
            let offset_ptr = self.ptr.add(SECTION_U32_OFFSET + U32_TEXT_OFFSET * U32_FIELD_BYTES) as *mut u32;
            let length_ptr = self.ptr.add(SECTION_U32_OFFSET + U32_TEXT_LENGTH * U32_FIELD_BYTES) as *mut u32;
            *offset_ptr.add(node) = write_ptr as u32;
            *length_ptr.add(node) = actual_len as u32;
        }
        self.header()[HEADER_TEXT_POOL_WRITE_PTR].store(end as u32, Ordering::Relaxed);
    }

    // =========================================================================
    // Wake / generation
    // =========================================================================

    pub fn set_wake_flag(&self) {
        self.header()[HEADER_WAKE_FLAG].store(1, Ordering::Release);
    }

    pub fn increment_generation(&self) {
        self.header()[HEADER_GENERATION].fetch_add(1, Ordering::AcqRel);
    }

    pub fn increment_render_count(&self) {
        self.header()[HEADER_RENDER_COUNT].fetch_add(1, Ordering::Relaxed);
    }

    // =========================================================================
    // Utility
    // =========================================================================

    pub fn allocated_indices(&self) -> Vec<usize> {
        let count = self.node_count();
        let mut indices = Vec::with_capacity(count);
        for i in 0..count {
            if self.component_type(i) != 0 {
                indices.push(i);
            }
        }
        indices
    }

    pub fn visible_indices(&self) -> Vec<usize> {
        let count = self.node_count();
        let mut indices = Vec::with_capacity(count);
        for i in 0..count {
            if self.component_type(i) != 0 && self.visible(i) {
                indices.push(i);
            }
        }
        indices
    }
}
