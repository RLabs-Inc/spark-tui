//! AoS (Array of Structs) SharedBuffer
//!
//! Each node's data is contiguous in memory for cache-friendly reads.
//! This module provides zero-copy access to the SharedArrayBuffer.

use std::ptr;

use bitflags::bitflags;

use crate::types::Rgba;

// =============================================================================
// CONSTANTS
// =============================================================================

pub const STRIDE: usize = 256;
pub const HEADER_SIZE: usize = 256;
pub const TEXT_POOL_SIZE: usize = 1024 * 1024; // 1MB
pub const MAX_NODES: usize = 100_000; // 100K nodes
pub const EVENT_RING_TOTAL: usize = 12 + 256 * 32; // header + 256 events * 32 bytes
pub const TOTAL_BUFFER_SIZE: usize = HEADER_SIZE + MAX_NODES * STRIDE + TEXT_POOL_SIZE + EVENT_RING_TOTAL;

// Header offsets - Core (0-35)
pub const H_VERSION: usize = 0;
pub const H_NODE_COUNT: usize = 4;
pub const H_MAX_NODES: usize = 8;
pub const H_TERMINAL_WIDTH: usize = 12;
pub const H_TERMINAL_HEIGHT: usize = 16;
pub const H_WAKE_FLAG: usize = 20; // Legacy, kept for compatibility
pub const H_GENERATION: usize = 24;
pub const H_TEXT_POOL_SIZE: usize = 28;
pub const H_TEXT_POOL_WRITE_PTR: usize = 32;

// Event ring indices
pub const H_EVENT_WRITE_IDX: usize = 36; // Rust writes
pub const H_EVENT_READ_IDX: usize = 40;  // TS writes

// Current state (Rust writes)
pub const H_FOCUSED_INDEX: usize = 44;   // i32, -1 = none
pub const H_HOVERED_INDEX: usize = 48;
pub const H_PRESSED_INDEX: usize = 52;
pub const H_MOUSE_X: usize = 56;         // u16
pub const H_MOUSE_Y: usize = 58;         // u16

// Wake flags (4-byte aligned for atomics)
pub const H_WAKE_RUST: usize = 64;       // TS notifies Rust
pub const H_WAKE_TS: usize = 68;         // Rust notifies TS

// Config (TS writes, Rust reads)
pub const H_CONFIG_FLAGS: usize = 72;
pub const H_RENDER_MODE: usize = 76;
pub const H_CURSOR_CONFIG: usize = 80;
pub const H_SCROLL_SPEED: usize = 84;
pub const H_RENDER_COUNT: usize = 88; // u32 for FPS tracking
pub const H_EXIT_REQUESTED: usize = 92; // u8, set by push_event on Exit

// =============================================================================
// CONFIG FLAGS
// =============================================================================

bitflags! {
    /// Configuration flags written by TS, read by Rust.
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
        Self::EXIT_ON_CTRL_C
            | Self::TAB_NAVIGATION
            | Self::ARROW_SCROLL
            | Self::PAGE_SCROLL
            | Self::HOME_END_SCROLL
            | Self::WHEEL_SCROLL
            | Self::FOCUS_ON_CLICK
    }
}

// Node field offsets (within each 256-byte node)
// Layout floats (0-95)
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
pub const F_MARGIN_LEFT: usize = 64;
pub const F_GAP: usize = 68;
pub const F_ROW_GAP: usize = 72;
pub const F_COLUMN_GAP: usize = 76;
pub const F_INSET_TOP: usize = 80;
pub const F_INSET_RIGHT: usize = 84;
pub const F_INSET_BOTTOM: usize = 88;
pub const F_INSET_LEFT: usize = 92;

// Layout enums (96-111)
pub const U_FLEX_DIRECTION: usize = 96;
pub const U_FLEX_WRAP: usize = 97;
pub const U_JUSTIFY_CONTENT: usize = 98;
pub const U_ALIGN_ITEMS: usize = 99;
pub const U_ALIGN_CONTENT: usize = 100;
pub const U_ALIGN_SELF: usize = 101;
pub const U_POSITION: usize = 102;
pub const U_OVERFLOW: usize = 103;
pub const U_DISPLAY: usize = 104;
pub const U_BORDER_TOP: usize = 105;
pub const U_BORDER_RIGHT: usize = 106;
pub const U_BORDER_BOTTOM: usize = 107;
pub const U_BORDER_LEFT: usize = 108;
pub const U_COMPONENT_TYPE: usize = 109;
pub const U_VISIBLE: usize = 110;

// Visual (112-147) - Colors + style
pub const C_FG_COLOR: usize = 112;
pub const C_BG_COLOR: usize = 116;
pub const C_BORDER_COLOR: usize = 120;
pub const C_FOCUS_RING_COLOR: usize = 124;
pub const C_CURSOR_FG: usize = 128;
pub const C_CURSOR_BG: usize = 132;
pub const C_SELECTION_COLOR: usize = 136;
pub const U_OPACITY: usize = 140;
pub const I_Z_INDEX: usize = 141;
pub const U_BORDER_STYLE: usize = 142;
// Per-side border styles (in reserved visual space 143-146)
pub const U_BORDER_STYLE_TOP: usize = 143;
pub const U_BORDER_STYLE_RIGHT: usize = 144;
pub const U_BORDER_STYLE_BOTTOM: usize = 145;
pub const U_BORDER_STYLE_LEFT: usize = 146;
// Focus indicator char (in reserved space 147)
pub const U_FOCUS_INDICATOR_CHAR: usize = 147; // u8, default '*' (0x2A)

// Legacy alias
pub const C_CURSOR_COLOR: usize = C_CURSOR_FG;

// Interaction (148-171) - Focus, cursor, scroll
pub const I_SCROLL_X: usize = 148;
pub const I_SCROLL_Y: usize = 152;
pub const I_TAB_INDEX: usize = 156;
pub const I_CURSOR_POSITION: usize = 160;
pub const I_SELECTION_START: usize = 164;
pub const I_SELECTION_END: usize = 168;

// Flags (172-179)
pub const U_DIRTY_FLAGS: usize = 172;
pub const U_INTERACTION_FLAGS: usize = 173;
pub const U_CURSOR_FLAGS: usize = 174;
pub const U_CURSOR_STYLE: usize = 175; // 0=block, 1=bar, 2=underline
pub const U_CURSOR_FPS: usize = 176;
pub const U_MAX_LENGTH: usize = 177;
pub const U_FOCUS_INDICATOR_ENABLED: usize = 178; // u8, 1=enabled, 0xFF=disabled
// 179 reserved

// Interaction flags (bitfield at U_INTERACTION_FLAGS)
pub const FLAG_FOCUSABLE: u8 = 0x01;
pub const FLAG_FOCUSED: u8 = 0x02;
pub const FLAG_HOVERED: u8 = 0x04;
pub const FLAG_PRESSED: u8 = 0x08;
pub const FLAG_DISABLED: u8 = 0x10;

// Hierarchy (180-183)
pub const I_PARENT_INDEX: usize = 180;

// Text (184-199)
pub const U_TEXT_OFFSET: usize = 184;
pub const U_TEXT_LENGTH: usize = 188;
pub const U_TEXT_ALIGN: usize = 192;
pub const U_TEXT_WRAP: usize = 193;
pub const U_TEXT_OVERFLOW: usize = 194;
pub const U_TEXT_ATTRS: usize = 195;

// Cursor character (196-199)
pub const U_CURSOR_CHAR: usize = 196; // u32 - custom cursor character (UTF-32)

// Output - written by Rust (200-232)
pub const F_COMPUTED_X: usize = 200;
pub const F_COMPUTED_Y: usize = 204;
pub const F_COMPUTED_WIDTH: usize = 208;
pub const F_COMPUTED_HEIGHT: usize = 212;
pub const F_SCROLL_WIDTH: usize = 216;
pub const F_SCROLL_HEIGHT: usize = 220;
pub const F_MAX_SCROLL_X: usize = 224;
pub const F_MAX_SCROLL_Y: usize = 228;
pub const U_SCROLLABLE: usize = 232;
// Cursor alt char in reserved output space
pub const U_CURSOR_ALT_CHAR: usize = 233; // u32 (but only uses 233-236, which is fine in reserved space)
// 237-255 reserved (Note: 236-251 used by per-side border colors below)

// Per-side border colors (if 0, falls back to C_BORDER_COLOR)
// Placed in reserved space (236-255)
pub const C_BORDER_TOP_COLOR: usize = 236;
pub const C_BORDER_RIGHT_COLOR: usize = 240;
pub const C_BORDER_BOTTOM_COLOR: usize = 244;
pub const C_BORDER_LEFT_COLOR: usize = 248;

// Legacy alias
pub const F_CONTENT_WIDTH: usize = F_SCROLL_WIDTH;

// Dirty flags
pub const DIRTY_LAYOUT: u8 = 0x01;
pub const DIRTY_VISUAL: u8 = 0x02;
pub const DIRTY_TEXT: u8 = 0x04;
pub const DIRTY_HIERARCHY: u8 = 0x08;

// Component types
pub const COMPONENT_NONE: u8 = 0;
pub const COMPONENT_BOX: u8 = 1;
pub const COMPONENT_TEXT: u8 = 2;
pub const COMPONENT_INPUT: u8 = 3;

// =============================================================================
// EVENT RING BUFFER
// =============================================================================

/// Event ring buffer location in shared memory.
pub const EVENT_RING_OFFSET: usize = HEADER_SIZE + MAX_NODES * STRIDE + TEXT_POOL_SIZE;
/// Header size for the ring buffer (write_idx, read_idx, reserved).
pub const EVENT_RING_HEADER_SIZE: usize = 12;
/// Size of each event slot in bytes.
pub const EVENT_SLOT_SIZE: usize = 20;
/// Maximum number of events in the ring buffer.
pub const MAX_EVENTS: usize = 256;
/// Total size of the event ring buffer.
pub const EVENT_RING_SIZE: usize = EVENT_RING_HEADER_SIZE + MAX_EVENTS * EVENT_SLOT_SIZE;

// =============================================================================
// AOS BUFFER
// =============================================================================

/// AoS SharedBuffer - wraps raw pointer to SharedArrayBuffer.
///
/// Memory layout:
/// - Header (256 bytes)
/// - Nodes (256 bytes each x MAX_NODES)
/// - Text pool (1MB)
/// - Event ring buffer (5,132 bytes)
pub struct AoSBuffer {
    ptr: *mut u8,
    len: usize,
}

// SAFETY: The buffer is shared with JS via SharedArrayBuffer.
// Access is coordinated through atomic operations on wake_flag.
unsafe impl Send for AoSBuffer {}
unsafe impl Sync for AoSBuffer {}

impl AoSBuffer {
    /// Create from raw pointer (from FFI).
    ///
    /// # Safety
    /// - `ptr` must point to a valid SharedArrayBuffer of at least `len` bytes
    /// - The buffer must remain valid for the lifetime of this struct
    pub unsafe fn from_raw(ptr: *mut u8, len: usize) -> Self {
        Self { ptr, len }
    }

    /// Get pointer to start of buffer.
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.ptr
    }

    /// Get mutable pointer to start of buffer.
    #[inline]
    pub fn as_mut_ptr(&self) -> *mut u8 {
        self.ptr
    }

    /// Get buffer length.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    // =========================================================================
    // HEADER ACCESS
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
    fn write_header_u16(&self, offset: usize, value: u16) {
        unsafe { ptr::write_unaligned(self.ptr.add(offset) as *mut u16, value) }
    }

    #[inline]
    pub fn version(&self) -> u32 {
        self.read_header_u32(H_VERSION)
    }

    #[inline]
    pub fn node_count(&self) -> usize {
        self.read_header_u32(H_NODE_COUNT) as usize
    }

    #[inline]
    pub fn max_nodes(&self) -> usize {
        self.read_header_u32(H_MAX_NODES) as usize
    }

    #[inline]
    pub fn terminal_width(&self) -> u32 {
        self.read_header_u32(H_TERMINAL_WIDTH)
    }

    #[inline]
    pub fn terminal_height(&self) -> u32 {
        self.read_header_u32(H_TERMINAL_HEIGHT)
    }

    #[inline]
    pub fn wake_flag(&self) -> u32 {
        self.read_header_u32(H_WAKE_FLAG)
    }

    #[inline]
    pub fn generation(&self) -> u32 {
        self.read_header_u32(H_GENERATION)
    }

    // =========================================================================
    // CONFIG FLAGS
    // =========================================================================

    /// Read config flags from header.
    #[inline]
    pub fn config_flags(&self) -> ConfigFlags {
        ConfigFlags::from_bits_truncate(self.read_header_u32(H_CONFIG_FLAGS))
    }

    /// Increment render count (for FPS tracking)
    #[inline]
    pub fn increment_render_count(&self) {
        let count = self.read_header_u32(H_RENDER_COUNT);
        self.write_header_u32(H_RENDER_COUNT, count.wrapping_add(1));
    }

    // =========================================================================
    // EVENT RING BUFFER
    // =========================================================================

    /// Read event write index (Rust writes, monotonically increasing).
    #[inline]
    pub fn event_write_idx(&self) -> u32 {
        self.read_header_u32(H_EVENT_WRITE_IDX)
    }

    /// Set event write index.
    #[inline]
    pub fn set_event_write_idx(&self, idx: u32) {
        self.write_header_u32(H_EVENT_WRITE_IDX, idx)
    }

    /// Read event read index (TS writes, monotonically increasing).
    #[inline]
    pub fn event_read_idx(&self) -> u32 {
        self.read_header_u32(H_EVENT_READ_IDX)
    }

    // =========================================================================
    // STATE TRACKING (Rust writes)
    // =========================================================================

    /// Set the focused component index (-1 = none).
    #[inline]
    pub fn set_focused_index(&self, idx: i32) {
        self.write_header_i32(H_FOCUSED_INDEX, idx)
    }

    /// Get the focused component index (-1 = none).
    #[inline]
    pub fn focused_index(&self) -> i32 {
        self.read_header_i32(H_FOCUSED_INDEX)
    }

    /// Set the hovered component index (-1 = none).
    #[inline]
    pub fn set_hovered_index(&self, idx: i32) {
        self.write_header_i32(H_HOVERED_INDEX, idx)
    }

    /// Get the hovered component index (-1 = none).
    #[inline]
    pub fn hovered_index(&self) -> i32 {
        self.read_header_i32(H_HOVERED_INDEX)
    }

    /// Set the pressed component index (-1 = none).
    #[inline]
    pub fn set_pressed_index(&self, idx: i32) {
        self.write_header_i32(H_PRESSED_INDEX, idx)
    }

    /// Get the pressed component index (-1 = none).
    #[inline]
    pub fn pressed_index(&self) -> i32 {
        self.read_header_i32(H_PRESSED_INDEX)
    }

    /// Set the current mouse position.
    #[inline]
    pub fn set_mouse_pos(&self, x: u16, y: u16) {
        self.write_header_u16(H_MOUSE_X, x);
        self.write_header_u16(H_MOUSE_Y, y);
    }

    // =========================================================================
    // EVENT PUSH (Rust writes events to ring buffer)
    // =========================================================================

    /// Write an event to the ring buffer and notify TS.
    ///
    /// Uses the Event type from crate::input::events.
    pub fn push_event(&self, event: &crate::input::events::Event) {
        use crate::input::events::EventType;

        let write_idx = self.event_write_idx() as usize;
        let slot = write_idx % MAX_EVENTS;
        let offset = EVENT_RING_OFFSET + EVENT_RING_HEADER_SIZE + slot * EVENT_SLOT_SIZE;

        unsafe {
            let ptr = self.ptr.add(offset);
            *ptr = event.event_type as u8;
            // ptr[1] is padding
            ptr::write_unaligned(ptr.add(2) as *mut u16, event.component_index);
            ptr::copy_nonoverlapping(event.data.as_ptr(), ptr.add(4), 16);
        }

        // Set exit flag if this is an exit event
        if event.event_type == EventType::Exit {
            unsafe {
                *self.ptr.add(H_EXIT_REQUESTED) = 1;
            }
        }

        self.set_event_write_idx((write_idx + 1) as u32);
        self.notify_ts();
    }

    /// Check if an exit event has been requested.
    #[inline]
    pub fn exit_requested(&self) -> bool {
        unsafe { *self.ptr.add(H_EXIT_REQUESTED) != 0 }
    }

    /// Wake the TS side via atomic store.
    /// Uses futex-style notification - just sets the flag and notifies.
    pub fn notify_ts(&self) {
        use std::sync::atomic::{AtomicU32, Ordering};
        unsafe {
            let wake_ptr = self.ptr.add(H_WAKE_TS) as *const AtomicU32;
            (*wake_ptr).store(1, Ordering::SeqCst);
            // Note: actual futex wake would need libc::syscall on Linux
            // For now, TS polls or uses Atomics.waitAsync
        }
    }

    /// Read and clear the wake flag atomically (consume)
    #[inline]
    pub fn consume_wake(&self) -> bool {
        use std::sync::atomic::{AtomicU32, Ordering};
        unsafe {
            let wake_ptr = self.ptr.add(H_WAKE_RUST) as *const AtomicU32;
            (*wake_ptr).swap(0, Ordering::AcqRel) != 0
        }
    }

    /// Set the wake flag (TS calls this)
    #[inline]
    pub fn set_wake_flag(&self) {
        use std::sync::atomic::{AtomicU32, Ordering};
        unsafe {
            let wake_ptr = self.ptr.add(H_WAKE_RUST) as *const AtomicU32;
            (*wake_ptr).store(1, Ordering::Release);
        }
    }

    // =========================================================================
    // NODE ACCESS - All reads from contiguous memory!
    // =========================================================================

    /// Get pointer to start of node data.
    #[inline]
    fn node_ptr(&self, node_index: usize) -> *const u8 {
        unsafe { self.ptr.add(HEADER_SIZE + node_index * STRIDE) }
    }

    /// Get mutable pointer to start of node data.
    #[inline]
    fn node_ptr_mut(&self, node_index: usize) -> *mut u8 {
        unsafe { self.ptr.add(HEADER_SIZE + node_index * STRIDE) }
    }

    /// Read f32 from node at field offset.
    #[inline]
    fn read_node_f32(&self, node_index: usize, field: usize) -> f32 {
        unsafe {
            let p = self.node_ptr(node_index).add(field) as *const f32;
            ptr::read_unaligned(p)
        }
    }

    /// Write f32 to node at field offset.
    #[inline]
    fn write_node_f32(&self, node_index: usize, field: usize, value: f32) {
        unsafe {
            let p = self.node_ptr_mut(node_index).add(field) as *mut f32;
            ptr::write_unaligned(p, value);
        }
    }

    /// Read u8 from node at field offset.
    #[inline]
    fn read_node_u8(&self, node_index: usize, field: usize) -> u8 {
        unsafe { *self.node_ptr(node_index).add(field) }
    }

    /// Write u8 to node at field offset.
    #[inline]
    fn write_node_u8(&self, node_index: usize, field: usize, value: u8) {
        unsafe { *self.node_ptr_mut(node_index).add(field) = value }
    }

    /// Read u32 from node at field offset.
    #[inline]
    fn read_node_u32(&self, node_index: usize, field: usize) -> u32 {
        unsafe {
            let p = self.node_ptr(node_index).add(field) as *const u32;
            ptr::read_unaligned(p)
        }
    }

    /// Read i32 from node at field offset.
    #[inline]
    fn read_node_i32(&self, node_index: usize, field: usize) -> i32 {
        unsafe {
            let p = self.node_ptr(node_index).add(field) as *const i32;
            ptr::read_unaligned(p)
        }
    }

    /// Write i32 to node at field offset.
    #[inline]
    fn write_node_i32(&self, node_index: usize, field: usize, value: i32) {
        unsafe {
            let p = self.node_ptr_mut(node_index).add(field) as *mut i32;
            ptr::write_unaligned(p, value);
        }
    }

    // =========================================================================
    // LAYOUT PROPERTIES (all contiguous reads!)
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

    // Layout enums
    #[inline] pub fn flex_direction(&self, i: usize) -> u8 { self.read_node_u8(i, U_FLEX_DIRECTION) }
    #[inline] pub fn flex_wrap(&self, i: usize) -> u8 { self.read_node_u8(i, U_FLEX_WRAP) }
    #[inline] pub fn justify_content(&self, i: usize) -> u8 { self.read_node_u8(i, U_JUSTIFY_CONTENT) }
    #[inline] pub fn align_items(&self, i: usize) -> u8 { self.read_node_u8(i, U_ALIGN_ITEMS) }
    #[inline] pub fn align_content(&self, i: usize) -> u8 { self.read_node_u8(i, U_ALIGN_CONTENT) }
    #[inline] pub fn align_self(&self, i: usize) -> u8 { self.read_node_u8(i, U_ALIGN_SELF) }
    #[inline] pub fn position(&self, i: usize) -> u8 { self.read_node_u8(i, U_POSITION) }
    #[inline] pub fn overflow(&self, i: usize) -> u8 { self.read_node_u8(i, U_OVERFLOW) }
    #[inline] pub fn display(&self, i: usize) -> u8 { self.read_node_u8(i, U_DISPLAY) }
    #[inline] pub fn border_top(&self, i: usize) -> u8 { self.read_node_u8(i, U_BORDER_TOP) }
    #[inline] pub fn border_right(&self, i: usize) -> u8 { self.read_node_u8(i, U_BORDER_RIGHT) }
    #[inline] pub fn border_bottom(&self, i: usize) -> u8 { self.read_node_u8(i, U_BORDER_BOTTOM) }
    #[inline] pub fn border_left(&self, i: usize) -> u8 { self.read_node_u8(i, U_BORDER_LEFT) }
    #[inline] pub fn component_type(&self, i: usize) -> u8 { self.read_node_u8(i, U_COMPONENT_TYPE) }
    #[inline] pub fn visible(&self, i: usize) -> bool { self.read_node_u8(i, U_VISIBLE) != 0 }

    // Hierarchy
    #[inline]
    pub fn parent_index(&self, i: usize) -> Option<usize> {
        let idx = self.read_node_i32(i, I_PARENT_INDEX);
        if idx < 0 { None } else { Some(idx as usize) }
    }

    // Dirty flags
    #[inline]
    pub fn dirty_flags(&self, i: usize) -> u8 {
        self.read_node_u8(i, U_DIRTY_FLAGS)
    }

    #[inline]
    pub fn is_dirty(&self, i: usize, flag: u8) -> bool {
        (self.dirty_flags(i) & flag) != 0
    }

    /// Clear all dirty flags for a node
    #[inline]
    pub fn clear_all_dirty(&self, i: usize) {
        self.write_node_u8(i, U_DIRTY_FLAGS, 0);
    }

    // =========================================================================
    // OUTPUT WRITES (Rust writes computed layout)
    // =========================================================================

    #[inline]
    pub fn set_computed_x(&self, i: usize, v: f32) {
        self.write_node_f32(i, F_COMPUTED_X, v);
    }

    #[inline]
    pub fn set_computed_y(&self, i: usize, v: f32) {
        self.write_node_f32(i, F_COMPUTED_Y, v);
    }

    #[inline]
    pub fn set_computed_width(&self, i: usize, v: f32) {
        self.write_node_f32(i, F_COMPUTED_WIDTH, v);
    }

    #[inline]
    pub fn set_computed_height(&self, i: usize, v: f32) {
        self.write_node_f32(i, F_COMPUTED_HEIGHT, v);
    }

    // =========================================================================
    // OUTPUT SECTION READS (for framebuffer)
    // =========================================================================

    /// Read computed X position (set by layout engine)
    #[inline]
    pub fn output_x(&self, i: usize) -> f32 {
        self.read_node_f32(i, F_COMPUTED_X)
    }

    /// Read computed Y position (set by layout engine)
    #[inline]
    pub fn output_y(&self, i: usize) -> f32 {
        self.read_node_f32(i, F_COMPUTED_Y)
    }

    /// Read computed width (set by layout engine)
    #[inline]
    pub fn output_width(&self, i: usize) -> f32 {
        self.read_node_f32(i, F_COMPUTED_WIDTH)
    }

    /// Read computed height (set by layout engine)
    #[inline]
    pub fn output_height(&self, i: usize) -> f32 {
        self.read_node_f32(i, F_COMPUTED_HEIGHT)
    }

    /// Read content scroll width (total scrollable content width)
    #[inline]
    pub fn scroll_width(&self, i: usize) -> f32 {
        self.read_node_f32(i, F_SCROLL_WIDTH)
    }

    /// Read content scroll height (total scrollable content height)
    #[inline]
    pub fn scroll_height(&self, i: usize) -> f32 {
        self.read_node_f32(i, F_SCROLL_HEIGHT)
    }

    /// Read max scroll X (how far can scroll horizontally)
    #[inline]
    pub fn output_max_scroll_x(&self, i: usize) -> f32 {
        self.read_node_f32(i, F_MAX_SCROLL_X)
    }

    /// Read max scroll Y (how far can scroll vertically)
    #[inline]
    pub fn output_max_scroll_y(&self, i: usize) -> f32 {
        self.read_node_f32(i, F_MAX_SCROLL_Y)
    }

    /// Check if node is scrollable
    #[inline]
    pub fn output_scrollable(&self, i: usize) -> bool {
        self.read_node_u8(i, U_SCROLLABLE) != 0
    }

    /// Set all scroll-related output (called by layout engine)
    #[inline]
    pub fn set_output_scroll(&self, i: usize, scrollable: bool, max_x: f32, max_y: f32) {
        self.write_node_u8(i, U_SCROLLABLE, if scrollable { 1 } else { 0 });
        self.write_node_f32(i, F_MAX_SCROLL_X, max_x);
        self.write_node_f32(i, F_MAX_SCROLL_Y, max_y);
    }

    // =========================================================================
    // COLOR ACCESSORS
    // =========================================================================

    /// Read foreground color (packed ARGB)
    #[inline]
    pub fn fg_color(&self, node: usize) -> u32 {
        self.read_node_u32(node, C_FG_COLOR)
    }

    /// Read background color (packed ARGB)
    #[inline]
    pub fn bg_color(&self, node: usize) -> u32 {
        self.read_node_u32(node, C_BG_COLOR)
    }

    /// Read border color (all sides default)
    #[inline]
    pub fn border_color(&self, node: usize) -> u32 {
        self.read_node_u32(node, C_BORDER_COLOR)
    }

    /// Read per-side border colors (falls back to border_color if 0)
    #[inline]
    pub fn border_color_top(&self, node: usize) -> u32 {
        let c = self.read_node_u32(node, C_BORDER_TOP_COLOR);
        if c == 0 { self.border_color(node) } else { c }
    }

    #[inline]
    pub fn border_color_right(&self, node: usize) -> u32 {
        let c = self.read_node_u32(node, C_BORDER_RIGHT_COLOR);
        if c == 0 { self.border_color(node) } else { c }
    }

    #[inline]
    pub fn border_color_bottom(&self, node: usize) -> u32 {
        let c = self.read_node_u32(node, C_BORDER_BOTTOM_COLOR);
        if c == 0 { self.border_color(node) } else { c }
    }

    #[inline]
    pub fn border_color_left(&self, node: usize) -> u32 {
        let c = self.read_node_u32(node, C_BORDER_LEFT_COLOR);
        if c == 0 { self.border_color(node) } else { c }
    }

    /// Read focus ring color
    #[inline]
    pub fn focus_ring_color(&self, node: usize) -> u32 {
        self.read_node_u32(node, C_FOCUS_RING_COLOR)
    }

    /// Read cursor foreground color
    #[inline]
    pub fn cursor_fg_color(&self, node: usize) -> u32 {
        self.read_node_u32(node, C_CURSOR_FG)
    }

    /// Read cursor background color
    #[inline]
    pub fn cursor_bg_color(&self, node: usize) -> u32 {
        self.read_node_u32(node, C_CURSOR_BG)
    }

    // =========================================================================
    // VISUAL PROPERTIES
    // =========================================================================

    /// Read opacity (0-255)
    #[inline]
    pub fn opacity(&self, node: usize) -> u8 {
        self.read_node_u8(node, U_OPACITY)
    }

    /// Read opacity as float (0.0-1.0)
    #[inline]
    pub fn opacity_f32(&self, node: usize) -> f32 {
        self.opacity(node) as f32 / 255.0
    }

    /// Read z-index for stacking order (i8, can be negative)
    #[inline]
    pub fn z_index(&self, node: usize) -> i8 {
        self.read_node_u8(node, I_Z_INDEX) as i8
    }

    /// Read border style (0=none, 1=single, 2=double, 3=rounded, etc)
    #[inline]
    pub fn border_style(&self, node: usize) -> u8 {
        self.read_node_u8(node, U_BORDER_STYLE)
    }

    /// Read per-side border styles (falls back to border_style if 0)
    #[inline]
    pub fn border_style_top(&self, node: usize) -> u8 {
        let s = self.read_node_u8(node, U_BORDER_STYLE_TOP);
        if s == 0 { self.border_style(node) } else { s }
    }

    #[inline]
    pub fn border_style_right(&self, node: usize) -> u8 {
        let s = self.read_node_u8(node, U_BORDER_STYLE_RIGHT);
        if s == 0 { self.border_style(node) } else { s }
    }

    #[inline]
    pub fn border_style_bottom(&self, node: usize) -> u8 {
        let s = self.read_node_u8(node, U_BORDER_STYLE_BOTTOM);
        if s == 0 { self.border_style(node) } else { s }
    }

    #[inline]
    pub fn border_style_left(&self, node: usize) -> u8 {
        let s = self.read_node_u8(node, U_BORDER_STYLE_LEFT);
        if s == 0 { self.border_style(node) } else { s }
    }

    /// Get focus indicator character (default '*')
    #[inline]
    pub fn focus_indicator_char(&self, node: usize) -> char {
        let ch = self.read_node_u8(node, U_FOCUS_INDICATOR_CHAR);
        if ch == 0 { '*' } else { ch as char }
    }

    /// Check if focus indicator is enabled for this node
    #[inline]
    pub fn focus_indicator_enabled(&self, node: usize) -> bool {
        // Default to enabled (0xFF = disabled, anything else = enabled)
        self.read_node_u8(node, U_FOCUS_INDICATOR_ENABLED) != 0xFF
    }

    // =========================================================================
    // INTERACTION FLAGS
    // =========================================================================

    /// Read raw interaction flags bitfield
    #[inline]
    pub fn interaction_flags(&self, node: usize) -> u8 {
        self.read_node_u8(node, U_INTERACTION_FLAGS)
    }

    /// Check if node is focusable
    #[inline]
    pub fn focusable(&self, node: usize) -> bool {
        self.interaction_flags(node) & FLAG_FOCUSABLE != 0
    }

    /// Check if node is currently focused
    #[inline]
    pub fn is_focused(&self, node: usize) -> bool {
        self.interaction_flags(node) & FLAG_FOCUSED != 0
    }

    /// Check if node is currently hovered
    #[inline]
    pub fn is_hovered(&self, node: usize) -> bool {
        self.interaction_flags(node) & FLAG_HOVERED != 0
    }

    /// Check if node is currently pressed
    #[inline]
    pub fn is_pressed(&self, node: usize) -> bool {
        self.interaction_flags(node) & FLAG_PRESSED != 0
    }

    /// Check if node is disabled
    #[inline]
    pub fn is_disabled(&self, node: usize) -> bool {
        self.interaction_flags(node) & FLAG_DISABLED != 0
    }

    /// Set hovered state
    #[inline]
    pub fn set_hovered(&self, node: usize, val: bool) {
        let flags = self.interaction_flags(node);
        let new_flags = if val { flags | FLAG_HOVERED } else { flags & !FLAG_HOVERED };
        self.write_node_u8(node, U_INTERACTION_FLAGS, new_flags);
    }

    /// Set pressed state
    #[inline]
    pub fn set_pressed(&self, node: usize, val: bool) {
        let flags = self.interaction_flags(node);
        let new_flags = if val { flags | FLAG_PRESSED } else { flags & !FLAG_PRESSED };
        self.write_node_u8(node, U_INTERACTION_FLAGS, new_flags);
    }

    /// Set focused state
    #[inline]
    pub fn set_focused(&self, node: usize, val: bool) {
        let flags = self.interaction_flags(node);
        let new_flags = if val { flags | FLAG_FOCUSED } else { flags & !FLAG_FOCUSED };
        self.write_node_u8(node, U_INTERACTION_FLAGS, new_flags);
    }

    // =========================================================================
    // TEXT PROPERTIES
    // =========================================================================

    /// Read text alignment (0=left, 1=center, 2=right)
    #[inline]
    pub fn text_align(&self, node: usize) -> u8 {
        self.read_node_u8(node, U_TEXT_ALIGN)
    }

    /// Read text wrap mode (0=nowrap, 1=wrap, 2=truncate)
    #[inline]
    pub fn text_wrap(&self, node: usize) -> u8 {
        self.read_node_u8(node, U_TEXT_WRAP)
    }

    /// Read text overflow mode (0=clip, 1=ellipsis, 2=fade)
    #[inline]
    pub fn text_overflow(&self, node: usize) -> u8 {
        self.read_node_u8(node, U_TEXT_OVERFLOW)
    }

    /// Read text attributes (bold, italic, etc bitfield)
    #[inline]
    pub fn text_attrs(&self, node: usize) -> u8 {
        self.read_node_u8(node, U_TEXT_ATTRS)
    }

    // =========================================================================
    // SCROLL POSITION
    // =========================================================================

    /// Read current scroll X position
    #[inline]
    pub fn scroll_x(&self, node: usize) -> i32 {
        self.read_node_i32(node, I_SCROLL_X)
    }

    /// Read current scroll Y position
    #[inline]
    pub fn scroll_y(&self, node: usize) -> i32 {
        self.read_node_i32(node, I_SCROLL_Y)
    }

    /// Read tab index for focus order
    #[inline]
    pub fn tab_index(&self, node: usize) -> i32 {
        self.read_node_i32(node, I_TAB_INDEX)
    }

    /// Set scroll position
    #[inline]
    pub fn set_scroll(&self, node: usize, x: i32, y: i32) {
        self.write_node_i32(node, I_SCROLL_X, x);
        self.write_node_i32(node, I_SCROLL_Y, y);
    }

    // =========================================================================
    // CURSOR/SELECTION STATE (for input components)
    // =========================================================================

    /// Read cursor position within text
    #[inline]
    pub fn cursor_position(&self, node: usize) -> i32 {
        self.read_node_i32(node, I_CURSOR_POSITION)
    }

    /// Set cursor position
    #[inline]
    pub fn set_cursor_position(&self, node: usize, pos: i32) {
        self.write_node_i32(node, I_CURSOR_POSITION, pos);
    }

    /// Read selection start
    #[inline]
    pub fn selection_start(&self, node: usize) -> i32 {
        self.read_node_i32(node, I_SELECTION_START)
    }

    /// Read selection end
    #[inline]
    pub fn selection_end(&self, node: usize) -> i32 {
        self.read_node_i32(node, I_SELECTION_END)
    }

    /// Set selection range
    #[inline]
    pub fn set_selection(&self, node: usize, start: i32, end: i32) {
        self.write_node_i32(node, I_SELECTION_START, start);
        self.write_node_i32(node, I_SELECTION_END, end);
    }

    /// Check if cursor is visible (for blink state)
    #[inline]
    pub fn cursor_visible(&self, node: usize) -> bool {
        self.read_node_u8(node, U_CURSOR_FLAGS) & 0x01 != 0
    }

    /// Set cursor visibility (called by blink manager)
    #[inline]
    pub fn set_cursor_visible(&self, node: usize, visible: bool) {
        let flags = self.read_node_u8(node, U_CURSOR_FLAGS);
        let new_flags = if visible { flags | 0x01 } else { flags & !0x01 };
        self.write_node_u8(node, U_CURSOR_FLAGS, new_flags);
    }

    /// Read cursor style (0=block, 1=bar, 2=underline)
    #[inline]
    pub fn cursor_style(&self, node: usize) -> u8 {
        self.read_node_u8(node, U_CURSOR_STYLE)
    }

    /// Read cursor character (0 = use default block)
    #[inline]
    pub fn cursor_char(&self, node: usize) -> u32 {
        self.read_node_u32(node, U_CURSOR_CHAR)
    }

    /// Get cursor alt char (shown during blink-off phase)
    #[inline]
    pub fn cursor_alt_char(&self, node: usize) -> u32 {
        self.read_node_u32(node, U_CURSOR_ALT_CHAR)
    }

    // =========================================================================
    // RGBA DECODERS
    // =========================================================================

    /// Unpack ARGB u32 to Rgba struct
    #[inline]
    pub fn unpack_to_rgba(packed: u32) -> Rgba {
        Rgba {
            r: ((packed >> 16) & 0xFF) as i16,
            g: ((packed >> 8) & 0xFF) as i16,
            b: (packed & 0xFF) as i16,
            a: ((packed >> 24) & 0xFF) as i16,
        }
    }

    /// Read foreground as Rgba
    #[inline]
    pub fn fg_rgba(&self, node: usize) -> Rgba {
        Self::unpack_to_rgba(self.fg_color(node))
    }

    /// Read background as Rgba
    #[inline]
    pub fn bg_rgba(&self, node: usize) -> Rgba {
        Self::unpack_to_rgba(self.bg_color(node))
    }

    /// Read border color as Rgba (all sides)
    #[inline]
    pub fn border_rgba(&self, node: usize) -> Rgba {
        Self::unpack_to_rgba(self.border_color(node))
    }

    /// Read per-side border colors as Rgba
    #[inline]
    pub fn border_color_top_rgba(&self, node: usize) -> Rgba {
        Self::unpack_to_rgba(self.border_color_top(node))
    }

    #[inline]
    pub fn border_color_right_rgba(&self, node: usize) -> Rgba {
        Self::unpack_to_rgba(self.border_color_right(node))
    }

    #[inline]
    pub fn border_color_bottom_rgba(&self, node: usize) -> Rgba {
        Self::unpack_to_rgba(self.border_color_bottom(node))
    }

    #[inline]
    pub fn border_color_left_rgba(&self, node: usize) -> Rgba {
        Self::unpack_to_rgba(self.border_color_left(node))
    }

    /// Read cursor colors as Rgba
    #[inline]
    pub fn cursor_fg_rgba(&self, node: usize) -> Rgba {
        Self::unpack_to_rgba(self.cursor_fg_color(node))
    }

    #[inline]
    pub fn cursor_bg_rgba(&self, node: usize) -> Rgba {
        Self::unpack_to_rgba(self.cursor_bg_color(node))
    }

    // =========================================================================
    // TEXT ACCESS
    // =========================================================================

    /// Write u32 to node at field offset.
    #[inline]
    fn write_node_u32(&self, node_index: usize, field: usize, value: u32) {
        unsafe {
            let p = self.node_ptr_mut(node_index).add(field) as *mut u32;
            ptr::write_unaligned(p, value);
        }
    }

    /// Read text offset into pool
    #[inline]
    pub fn text_offset(&self, node: usize) -> u32 {
        self.read_node_u32(node, U_TEXT_OFFSET)
    }

    /// Read text length in bytes
    #[inline]
    pub fn text_length(&self, node: usize) -> u32 {
        self.read_node_u32(node, U_TEXT_LENGTH)
    }

    /// Read text content as string slice.
    ///
    /// Returns the text stored in the text pool for this node.
    /// Returns empty string if offset/length are invalid.
    pub fn text(&self, i: usize) -> &str {
        let offset = self.read_node_u32(i, U_TEXT_OFFSET) as usize;
        let length = self.read_node_u32(i, U_TEXT_LENGTH) as usize;

        if length == 0 {
            return "";
        }

        let text_pool_start = HEADER_SIZE + MAX_NODES * STRIDE;
        let text_end = text_pool_start + offset + length;

        // Bounds check
        if text_end > self.len {
            return "";
        }

        unsafe {
            let ptr = self.ptr.add(text_pool_start + offset);
            let slice = std::slice::from_raw_parts(ptr, length);
            std::str::from_utf8_unchecked(slice)
        }
    }

    /// Alias for text() - reads text content as string slice.
    #[inline]
    pub fn text_content(&self, node: usize) -> &str {
        self.text(node)
    }

    /// Write text to the text pool.
    ///
    /// Allocates space in the text pool and updates the node's offset/length.
    /// Returns false if text pool is full.
    pub fn write_text(&self, node: usize, text: &str) -> bool {
        let bytes = text.as_bytes();
        let len = bytes.len();

        if len == 0 {
            self.write_node_u32(node, U_TEXT_OFFSET, 0);
            self.write_node_u32(node, U_TEXT_LENGTH, 0);
            return true;
        }

        // Read current write pointer from header
        let write_ptr = self.text_pool_write_ptr();
        let new_ptr = write_ptr + len;

        // Check if we have space
        if new_ptr > TEXT_POOL_SIZE {
            return false; // Pool full
        }

        // Write text to pool
        let pool_start = HEADER_SIZE + MAX_NODES * STRIDE;
        let pool_offset = pool_start + write_ptr;
        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr(), self.ptr.add(pool_offset), len);
        }

        // Update header write pointer
        self.set_text_pool_write_ptr(new_ptr);

        // Update node's offset/length
        self.write_node_u32(node, U_TEXT_OFFSET, write_ptr as u32);
        self.write_node_u32(node, U_TEXT_LENGTH, len as u32);

        true
    }

    /// Get current text pool write pointer
    #[inline]
    pub fn text_pool_write_ptr(&self) -> usize {
        self.read_header_u32(H_TEXT_POOL_WRITE_PTR) as usize
    }

    /// Set text pool write pointer
    #[inline]
    fn set_text_pool_write_ptr(&self, ptr: usize) {
        self.write_header_u32(H_TEXT_POOL_WRITE_PTR, ptr as u32);
    }

    /// Get text pool capacity
    #[inline]
    pub fn text_pool_capacity(&self) -> usize {
        TEXT_POOL_SIZE
    }

    /// Get remaining text pool space
    #[inline]
    pub fn text_pool_remaining(&self) -> usize {
        TEXT_POOL_SIZE.saturating_sub(self.text_pool_write_ptr())
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        // Verify output fields fit in stride (U_SCROLLABLE is at 216, needs 1 byte)
        assert!(U_SCROLLABLE + 1 <= STRIDE);
        // Verify per-node cursor fields fit in stride
        assert!(C_CURSOR_BG + 4 <= STRIDE);
        // Verify header fields don't overlap
        assert!(H_TEXT_POOL_WRITE_PTR + 4 <= H_EVENT_WRITE_IDX);
        assert!(H_SCROLL_SPEED + 4 <= HEADER_SIZE);
        // Verify wake flags are 4-byte aligned for atomics
        assert!(H_WAKE_RUST % 4 == 0);
        assert!(H_WAKE_TS % 4 == 0);
        // Verify event ring buffer fits after text pool
        let expected_ring_start = HEADER_SIZE + MAX_NODES * STRIDE + TEXT_POOL_SIZE;
        assert_eq!(EVENT_RING_OFFSET, expected_ring_start);
    }

    #[test]
    fn test_config_flags_default() {
        let default = ConfigFlags::default();
        assert!(default.contains(ConfigFlags::EXIT_ON_CTRL_C));
        assert!(default.contains(ConfigFlags::TAB_NAVIGATION));
        assert!(default.contains(ConfigFlags::ARROW_SCROLL));
        assert!(default.contains(ConfigFlags::PAGE_SCROLL));
        assert!(default.contains(ConfigFlags::HOME_END_SCROLL));
        assert!(default.contains(ConfigFlags::WHEEL_SCROLL));
        assert!(default.contains(ConfigFlags::FOCUS_ON_CLICK));
        // Mouse and Kitty keyboard are opt-in
        assert!(!default.contains(ConfigFlags::MOUSE_ENABLED));
        assert!(!default.contains(ConfigFlags::KITTY_KEYBOARD));
    }

    #[test]
    fn test_config_flags_bits() {
        assert_eq!(ConfigFlags::EXIT_ON_CTRL_C.bits(), 1 << 0);
        assert_eq!(ConfigFlags::TAB_NAVIGATION.bits(), 1 << 1);
        assert_eq!(ConfigFlags::MOUSE_ENABLED.bits(), 1 << 7);
        assert_eq!(ConfigFlags::KITTY_KEYBOARD.bits(), 1 << 8);
    }

    #[test]
    fn test_buffer_creation() {
        let mut data = vec![0u8; HEADER_SIZE + 10 * STRIDE + TEXT_POOL_SIZE];

        // Set some header values
        let ptr = data.as_mut_ptr();
        unsafe {
            *(ptr.add(H_NODE_COUNT) as *mut u32) = 5;
            *(ptr.add(H_TERMINAL_WIDTH) as *mut u32) = 120;
            *(ptr.add(H_TERMINAL_HEIGHT) as *mut u32) = 40;
        }

        let buf = unsafe { AoSBuffer::from_raw(ptr, data.len()) };

        assert_eq!(buf.node_count(), 5);
        assert_eq!(buf.terminal_width(), 120);
        assert_eq!(buf.terminal_height(), 40);
    }

    #[test]
    fn test_node_fields() {
        let mut data = vec![0u8; HEADER_SIZE + 10 * STRIDE + TEXT_POOL_SIZE];
        let ptr = data.as_mut_ptr();

        // Set node count
        unsafe { *(ptr.add(H_NODE_COUNT) as *mut u32) = 3; }

        let buf = unsafe { AoSBuffer::from_raw(ptr, data.len()) };

        // Write some values to node 1
        unsafe {
            let node_base = ptr.add(HEADER_SIZE + 1 * STRIDE);
            *(node_base.add(F_WIDTH) as *mut f32) = 100.0;
            *(node_base.add(F_HEIGHT) as *mut f32) = 50.0;
            *node_base.add(U_FLEX_DIRECTION) = 1; // column
            *node_base.add(U_COMPONENT_TYPE) = COMPONENT_BOX;
            *node_base.add(U_VISIBLE) = 1;
            *(node_base.add(I_PARENT_INDEX) as *mut i32) = 0;
        }

        assert_eq!(buf.width(1), 100.0);
        assert_eq!(buf.height(1), 50.0);
        assert_eq!(buf.flex_direction(1), 1);
        assert_eq!(buf.component_type(1), COMPONENT_BOX);
        assert!(buf.visible(1));
        assert_eq!(buf.parent_index(1), Some(0));

        // Node 0 should have no parent
        unsafe {
            let node_base = ptr.add(HEADER_SIZE);
            *(node_base.add(I_PARENT_INDEX) as *mut i32) = -1;
        }
        assert_eq!(buf.parent_index(0), None);
    }

    #[test]
    fn test_output_writes() {
        let mut data = vec![0u8; HEADER_SIZE + 10 * STRIDE + TEXT_POOL_SIZE];
        let ptr = data.as_mut_ptr();
        let buf = unsafe { AoSBuffer::from_raw(ptr, data.len()) };

        buf.set_computed_x(0, 10.0);
        buf.set_computed_y(0, 20.0);
        buf.set_computed_width(0, 100.0);
        buf.set_computed_height(0, 50.0);

        // Verify by reading back
        unsafe {
            let node_base = ptr.add(HEADER_SIZE);
            assert_eq!(*(node_base.add(F_COMPUTED_X) as *const f32), 10.0);
            assert_eq!(*(node_base.add(F_COMPUTED_Y) as *const f32), 20.0);
            assert_eq!(*(node_base.add(F_COMPUTED_WIDTH) as *const f32), 100.0);
            assert_eq!(*(node_base.add(F_COMPUTED_HEIGHT) as *const f32), 50.0);
        }
    }

    #[test]
    fn test_state_tracking() {
        let mut data = vec![0u8; HEADER_SIZE + 10 * STRIDE + TEXT_POOL_SIZE + EVENT_RING_SIZE];
        let ptr = data.as_mut_ptr();
        let buf = unsafe { AoSBuffer::from_raw(ptr, data.len()) };

        // Test focused index
        buf.set_focused_index(5);
        assert_eq!(buf.focused_index(), 5);
        buf.set_focused_index(-1);
        assert_eq!(buf.focused_index(), -1);

        // Test hovered index
        buf.set_hovered_index(10);
        assert_eq!(buf.hovered_index(), 10);

        // Test pressed index
        buf.set_pressed_index(3);
        assert_eq!(buf.pressed_index(), 3);

        // Test mouse position
        buf.set_mouse_pos(100, 50);
        unsafe {
            let x = std::ptr::read_unaligned(ptr.add(H_MOUSE_X) as *const u16);
            let y = std::ptr::read_unaligned(ptr.add(H_MOUSE_Y) as *const u16);
            assert_eq!(x, 100);
            assert_eq!(y, 50);
        }
    }

    #[test]
    fn test_event_ring_indices() {
        let mut data = vec![0u8; HEADER_SIZE + 10 * STRIDE + TEXT_POOL_SIZE + EVENT_RING_SIZE];
        let ptr = data.as_mut_ptr();
        let buf = unsafe { AoSBuffer::from_raw(ptr, data.len()) };

        // Initial state
        assert_eq!(buf.event_write_idx(), 0);
        assert_eq!(buf.event_read_idx(), 0);

        // Write some indices
        buf.set_event_write_idx(5);
        assert_eq!(buf.event_write_idx(), 5);

        // Simulate TS updating read idx
        unsafe {
            std::ptr::write_unaligned(ptr.add(H_EVENT_READ_IDX) as *mut u32, 3);
        }
        assert_eq!(buf.event_read_idx(), 3);
    }

    #[test]
    fn test_config_flags_read() {
        let mut data = vec![0u8; HEADER_SIZE + 10 * STRIDE + TEXT_POOL_SIZE + EVENT_RING_SIZE];
        let ptr = data.as_mut_ptr();

        // Write config flags (simulate TS write)
        let flags = ConfigFlags::EXIT_ON_CTRL_C | ConfigFlags::MOUSE_ENABLED;
        unsafe {
            std::ptr::write_unaligned(ptr.add(H_CONFIG_FLAGS) as *mut u32, flags.bits());
        }

        let buf = unsafe { AoSBuffer::from_raw(ptr, data.len()) };
        let read_flags = buf.config_flags();

        assert!(read_flags.contains(ConfigFlags::EXIT_ON_CTRL_C));
        assert!(read_flags.contains(ConfigFlags::MOUSE_ENABLED));
        assert!(!read_flags.contains(ConfigFlags::TAB_NAVIGATION));
    }

    #[test]
    fn test_event_ring_write() {
        use crate::input::events::{Event, EventType};

        let mut data = vec![0u8; HEADER_SIZE + MAX_NODES * STRIDE + TEXT_POOL_SIZE + EVENT_RING_SIZE];
        let ptr = data.as_mut_ptr();
        let buf = unsafe { AoSBuffer::from_raw(ptr, data.len()) };

        // Push a focus event
        let event = Event::focus(42);
        buf.push_event(&event);

        // Verify write index incremented
        assert_eq!(buf.event_write_idx(), 1);

        // Verify event data was written
        let event_offset = EVENT_RING_OFFSET + EVENT_RING_HEADER_SIZE;
        unsafe {
            let event_ptr = ptr.add(event_offset);
            assert_eq!(*event_ptr, EventType::Focus as u8);
            let comp_idx = std::ptr::read_unaligned(event_ptr.add(2) as *const u16);
            assert_eq!(comp_idx, 42);
        }

        // Push another event and verify wrapping math
        let event2 = Event::blur(7);
        buf.push_event(&event2);
        assert_eq!(buf.event_write_idx(), 2);

        let event2_offset = EVENT_RING_OFFSET + EVENT_RING_HEADER_SIZE + EVENT_SLOT_SIZE;
        unsafe {
            let event_ptr = ptr.add(event2_offset);
            assert_eq!(*event_ptr, EventType::Blur as u8);
            let comp_idx = std::ptr::read_unaligned(event_ptr.add(2) as *const u16);
            assert_eq!(comp_idx, 7);
        }
    }

    #[test]
    fn test_notify_ts_sets_wake_flag() {
        let mut data = vec![0u8; HEADER_SIZE + 10 * STRIDE + TEXT_POOL_SIZE + EVENT_RING_SIZE];
        let ptr = data.as_mut_ptr();
        let buf = unsafe { AoSBuffer::from_raw(ptr, data.len()) };

        // Initial wake flag should be 0
        unsafe {
            let wake = std::ptr::read_unaligned(ptr.add(H_WAKE_TS) as *const u32);
            assert_eq!(wake, 0);
        }

        // Notify TS
        buf.notify_ts();

        // Wake flag should be 1
        unsafe {
            let wake = std::ptr::read_unaligned(ptr.add(H_WAKE_TS) as *const u32);
            assert_eq!(wake, 1);
        }
    }

    #[test]
    fn test_output_reads() {
        let mut data = vec![0u8; HEADER_SIZE + STRIDE * 10 + TEXT_POOL_SIZE];
        let ptr = data.as_mut_ptr();
        let buf = unsafe { AoSBuffer::from_raw(ptr, data.len()) };

        // Write computed layout values
        buf.set_computed_x(0, 10.0);
        buf.set_computed_y(0, 20.0);
        buf.set_computed_width(0, 100.0);
        buf.set_computed_height(0, 50.0);
        buf.set_output_scroll(0, true, 200.0, 300.0);

        // Read them back via output_* methods
        assert_eq!(buf.output_x(0), 10.0);
        assert_eq!(buf.output_y(0), 20.0);
        assert_eq!(buf.output_width(0), 100.0);
        assert_eq!(buf.output_height(0), 50.0);
        assert!(buf.output_scrollable(0));
        assert_eq!(buf.output_max_scroll_x(0), 200.0);
        assert_eq!(buf.output_max_scroll_y(0), 300.0);

        // Test non-scrollable node
        buf.set_output_scroll(1, false, 0.0, 0.0);
        assert!(!buf.output_scrollable(1));
    }

    #[test]
    fn test_color_accessors() {
        let mut data = vec![0u8; HEADER_SIZE + STRIDE * 10 + TEXT_POOL_SIZE];
        let ptr = data.as_mut_ptr();
        let buf = unsafe { AoSBuffer::from_raw(ptr, data.len()) };

        // Write a color (ARGB: alpha=255, red=128, green=64, blue=32)
        let packed = 0xFF804020u32;
        let offset = HEADER_SIZE + 0 * STRIDE + C_FG_COLOR;
        unsafe { ptr::write_unaligned(ptr.add(offset) as *mut u32, packed) };

        assert_eq!(buf.fg_color(0), packed);

        let rgba = buf.fg_rgba(0);
        assert_eq!(rgba.r, 128);
        assert_eq!(rgba.g, 64);
        assert_eq!(rgba.b, 32);
        assert_eq!(rgba.a, 255);
    }

    #[test]
    fn test_border_color_fallback() {
        let mut data = vec![0u8; HEADER_SIZE + STRIDE * 10 + TEXT_POOL_SIZE];
        let ptr = data.as_mut_ptr();
        let buf = unsafe { AoSBuffer::from_raw(ptr, data.len()) };

        // Set base border color
        let base_color = 0xFFFF0000u32; // red
        let offset = HEADER_SIZE + 0 * STRIDE + C_BORDER_COLOR;
        unsafe { ptr::write_unaligned(ptr.add(offset) as *mut u32, base_color) };

        // Per-side colors are 0, should fall back to base
        assert_eq!(buf.border_color_top(0), base_color);
        assert_eq!(buf.border_color_right(0), base_color);
        assert_eq!(buf.border_color_bottom(0), base_color);
        assert_eq!(buf.border_color_left(0), base_color);

        // Set a specific side color
        let top_color = 0xFF00FF00u32; // green
        let top_offset = HEADER_SIZE + 0 * STRIDE + C_BORDER_TOP_COLOR;
        unsafe { ptr::write_unaligned(ptr.add(top_offset) as *mut u32, top_color) };

        // Top should now be green, others still fall back to red
        assert_eq!(buf.border_color_top(0), top_color);
        assert_eq!(buf.border_color_right(0), base_color);
        assert_eq!(buf.border_color_bottom(0), base_color);
        assert_eq!(buf.border_color_left(0), base_color);
    }

    #[test]
    fn test_unpack_to_rgba() {
        // Test ARGB unpacking
        let packed = 0x80FF8040u32; // alpha=128, red=255, green=128, blue=64
        let rgba = AoSBuffer::unpack_to_rgba(packed);

        assert_eq!(rgba.a, 128);
        assert_eq!(rgba.r, 255);
        assert_eq!(rgba.g, 128);
        assert_eq!(rgba.b, 64);
    }

    #[test]
    fn test_visual_accessors() {
        let mut data = vec![0u8; HEADER_SIZE + STRIDE * 10 + TEXT_POOL_SIZE];
        let ptr = data.as_mut_ptr();
        let buf = unsafe { AoSBuffer::from_raw(ptr, data.len()) };

        // Set opacity
        let opacity_offset = HEADER_SIZE + 0 * STRIDE + U_OPACITY;
        unsafe { *ptr.add(opacity_offset) = 128; }
        assert_eq!(buf.opacity(0), 128);
        assert!((buf.opacity_f32(0) - 0.502).abs() < 0.01);

        // Set z-index (can be negative as i8)
        let z_offset = HEADER_SIZE + 0 * STRIDE + I_Z_INDEX;
        unsafe { *ptr.add(z_offset) = 250; } // 250 as u8 = -6 as i8
        assert_eq!(buf.z_index(0), -6);

        // Set border style
        let style_offset = HEADER_SIZE + 0 * STRIDE + U_BORDER_STYLE;
        unsafe { *ptr.add(style_offset) = 2; } // double border
        assert_eq!(buf.border_style(0), 2);
    }

    #[test]
    fn test_interaction_flags() {
        let mut data = vec![0u8; HEADER_SIZE + STRIDE * 10 + TEXT_POOL_SIZE];
        let ptr = data.as_mut_ptr();
        let buf = unsafe { AoSBuffer::from_raw(ptr, data.len()) };

        // Set flags
        let flags_offset = HEADER_SIZE + 0 * STRIDE + U_INTERACTION_FLAGS;
        unsafe { *ptr.add(flags_offset) = FLAG_FOCUSABLE | FLAG_FOCUSED; }

        assert!(buf.focusable(0));
        assert!(buf.is_focused(0));
        assert!(!buf.is_hovered(0));
        assert!(!buf.is_pressed(0));
        assert!(!buf.is_disabled(0));

        // Change to hovered and pressed
        unsafe { *ptr.add(flags_offset) = FLAG_HOVERED | FLAG_PRESSED; }
        assert!(!buf.focusable(0));
        assert!(!buf.is_focused(0));
        assert!(buf.is_hovered(0));
        assert!(buf.is_pressed(0));

        // Test disabled
        unsafe { *ptr.add(flags_offset) = FLAG_DISABLED; }
        assert!(buf.is_disabled(0));
    }

    #[test]
    fn test_text_properties() {
        let mut data = vec![0u8; HEADER_SIZE + STRIDE * 10 + TEXT_POOL_SIZE];
        let ptr = data.as_mut_ptr();
        let buf = unsafe { AoSBuffer::from_raw(ptr, data.len()) };

        // Set text properties
        let node_base = HEADER_SIZE + 0 * STRIDE;
        unsafe {
            *ptr.add(node_base + U_TEXT_ALIGN) = 1; // center
            *ptr.add(node_base + U_TEXT_WRAP) = 2;  // truncate
            *ptr.add(node_base + U_TEXT_OVERFLOW) = 1; // ellipsis
            *ptr.add(node_base + U_TEXT_ATTRS) = 0x03; // bold + italic
        }

        assert_eq!(buf.text_align(0), 1);
        assert_eq!(buf.text_wrap(0), 2);
        assert_eq!(buf.text_overflow(0), 1);
        assert_eq!(buf.text_attrs(0), 0x03);
    }

    #[test]
    fn test_scroll_position() {
        let mut data = vec![0u8; HEADER_SIZE + STRIDE * 10 + TEXT_POOL_SIZE];
        let ptr = data.as_mut_ptr();
        let buf = unsafe { AoSBuffer::from_raw(ptr, data.len()) };

        // Set scroll positions
        let node_base = HEADER_SIZE + 0 * STRIDE;
        unsafe {
            std::ptr::write_unaligned(ptr.add(node_base + I_SCROLL_X) as *mut i32, 100);
            std::ptr::write_unaligned(ptr.add(node_base + I_SCROLL_Y) as *mut i32, 200);
            std::ptr::write_unaligned(ptr.add(node_base + I_TAB_INDEX) as *mut i32, 5);
        }

        assert_eq!(buf.scroll_x(0), 100);
        assert_eq!(buf.scroll_y(0), 200);
        assert_eq!(buf.tab_index(0), 5);
    }

    #[test]
    fn test_text_content() {
        // Must use full buffer size since text pool starts at HEADER_SIZE + MAX_NODES * STRIDE
        let total_size = HEADER_SIZE + MAX_NODES * STRIDE + TEXT_POOL_SIZE;
        let mut data = vec![0u8; total_size];
        let buf = unsafe { AoSBuffer::from_raw(data.as_mut_ptr(), data.len()) };

        // Write some text
        assert!(buf.write_text(0, "Hello, SparkTUI!"));
        assert_eq!(buf.text_content(0), "Hello, SparkTUI!");
        assert_eq!(buf.text(0), "Hello, SparkTUI!");
        assert_eq!(buf.text_offset(0), 0);
        assert_eq!(buf.text_length(0), 16);

        // Write more text to different node
        assert!(buf.write_text(1, "Another string"));
        assert_eq!(buf.text_content(1), "Another string");
        assert_eq!(buf.text_offset(1), 16); // After first string

        // Original still intact
        assert_eq!(buf.text_content(0), "Hello, SparkTUI!");

        // Empty text
        assert!(buf.write_text(2, ""));
        assert_eq!(buf.text_content(2), "");
        assert_eq!(buf.text_length(2), 0);

        // Unicode
        assert!(buf.write_text(3, "こんにちは 🎉"));
        assert_eq!(buf.text_content(3), "こんにちは 🎉");

        // Test pool helpers
        assert_eq!(buf.text_pool_capacity(), TEXT_POOL_SIZE);
        assert!(buf.text_pool_remaining() < TEXT_POOL_SIZE);
        assert!(buf.text_pool_write_ptr() > 0);
    }

    #[test]
    fn test_interaction_state() {
        let mut data = vec![0u8; HEADER_SIZE + STRIDE * 10 + TEXT_POOL_SIZE];
        let ptr = data.as_mut_ptr();
        let buf = unsafe { AoSBuffer::from_raw(ptr, data.len()) };

        // Test scroll setter
        buf.set_scroll(0, 10, 20);
        assert_eq!(buf.scroll_x(0), 10);
        assert_eq!(buf.scroll_y(0), 20);

        // Test cursor position
        buf.set_cursor_position(0, 5);
        assert_eq!(buf.cursor_position(0), 5);

        // Test selection
        buf.set_selection(0, 2, 8);
        assert_eq!(buf.selection_start(0), 2);
        assert_eq!(buf.selection_end(0), 8);

        // Test cursor visibility
        buf.set_cursor_visible(0, true);
        assert!(buf.cursor_visible(0));
        buf.set_cursor_visible(0, false);
        assert!(!buf.cursor_visible(0));

        // Test cursor style
        let node_base = HEADER_SIZE + 0 * STRIDE;
        unsafe { *ptr.add(node_base + U_CURSOR_STYLE) = 1; } // bar cursor
        assert_eq!(buf.cursor_style(0), 1);

        // Test cursor char
        unsafe {
            std::ptr::write_unaligned(ptr.add(node_base + U_CURSOR_CHAR) as *mut u32, 0x2588); // full block
        }
        assert_eq!(buf.cursor_char(0), 0x2588);
    }

    #[test]
    fn test_clear_all_dirty() {
        let mut data = vec![0u8; HEADER_SIZE + STRIDE * 10 + TEXT_POOL_SIZE];
        let ptr = data.as_mut_ptr();
        let buf = unsafe { AoSBuffer::from_raw(ptr, data.len()) };

        // Set dirty flags
        unsafe { *ptr.add(HEADER_SIZE + U_DIRTY_FLAGS) = DIRTY_LAYOUT | DIRTY_VISUAL; }
        assert_ne!(buf.dirty_flags(0), 0);

        buf.clear_all_dirty(0);
        assert_eq!(buf.dirty_flags(0), 0);
    }

    #[test]
    fn test_interaction_setters() {
        let mut data = vec![0u8; HEADER_SIZE + STRIDE * 10 + TEXT_POOL_SIZE];
        let ptr = data.as_mut_ptr();
        let buf = unsafe { AoSBuffer::from_raw(ptr, data.len()) };

        assert!(!buf.is_hovered(0));
        buf.set_hovered(0, true);
        assert!(buf.is_hovered(0));
        buf.set_hovered(0, false);
        assert!(!buf.is_hovered(0));

        assert!(!buf.is_pressed(0));
        buf.set_pressed(0, true);
        assert!(buf.is_pressed(0));
        buf.set_pressed(0, false);
        assert!(!buf.is_pressed(0));

        assert!(!buf.is_focused(0));
        buf.set_focused(0, true);
        assert!(buf.is_focused(0));
        buf.set_focused(0, false);
        assert!(!buf.is_focused(0));
    }

    #[test]
    fn test_wake_flag() {
        let mut data = vec![0u8; HEADER_SIZE + STRIDE * 10 + TEXT_POOL_SIZE + EVENT_RING_SIZE];
        let ptr = data.as_mut_ptr();
        let buf = unsafe { AoSBuffer::from_raw(ptr, data.len()) };

        assert!(!buf.consume_wake());
        buf.set_wake_flag();
        assert!(buf.consume_wake());
        assert!(!buf.consume_wake()); // Consumed
    }

    #[test]
    fn test_render_count() {
        let mut data = vec![0u8; HEADER_SIZE + STRIDE * 10 + TEXT_POOL_SIZE + EVENT_RING_SIZE];
        let ptr = data.as_mut_ptr();
        let buf = unsafe { AoSBuffer::from_raw(ptr, data.len()) };

        // Initial count should be 0
        assert_eq!(buf.read_header_u32(H_RENDER_COUNT), 0);

        buf.increment_render_count();
        assert_eq!(buf.read_header_u32(H_RENDER_COUNT), 1);

        buf.increment_render_count();
        buf.increment_render_count();
        assert_eq!(buf.read_header_u32(H_RENDER_COUNT), 3);
    }

    #[test]
    fn test_border_style_per_side() {
        let mut data = vec![0u8; HEADER_SIZE + STRIDE * 10 + TEXT_POOL_SIZE];
        let ptr = data.as_mut_ptr();
        let buf = unsafe { AoSBuffer::from_raw(ptr, data.len()) };

        // Set base border style
        let node_base = HEADER_SIZE + 0 * STRIDE;
        unsafe { *ptr.add(node_base + U_BORDER_STYLE) = 1; } // single border

        // Per-side styles should fall back to base
        assert_eq!(buf.border_style_top(0), 1);
        assert_eq!(buf.border_style_right(0), 1);
        assert_eq!(buf.border_style_bottom(0), 1);
        assert_eq!(buf.border_style_left(0), 1);

        // Set a specific side style
        unsafe { *ptr.add(node_base + U_BORDER_STYLE_TOP) = 2; } // double border
        assert_eq!(buf.border_style_top(0), 2);
        assert_eq!(buf.border_style_right(0), 1); // Still fallback
    }

    #[test]
    fn test_focus_indicator() {
        let mut data = vec![0u8; HEADER_SIZE + STRIDE * 10 + TEXT_POOL_SIZE];
        let ptr = data.as_mut_ptr();
        let buf = unsafe { AoSBuffer::from_raw(ptr, data.len()) };

        // Default focus indicator char is '*'
        assert_eq!(buf.focus_indicator_char(0), '*');

        // Set custom char
        let node_base = HEADER_SIZE + 0 * STRIDE;
        unsafe { *ptr.add(node_base + U_FOCUS_INDICATOR_CHAR) = b'>'; }
        assert_eq!(buf.focus_indicator_char(0), '>');

        // Focus indicator enabled by default (0 != 0xFF)
        assert!(buf.focus_indicator_enabled(0));

        // Disable it
        unsafe { *ptr.add(node_base + U_FOCUS_INDICATOR_ENABLED) = 0xFF; }
        assert!(!buf.focus_indicator_enabled(0));
    }

    #[test]
    fn test_cursor_alt_char() {
        let mut data = vec![0u8; HEADER_SIZE + STRIDE * 10 + TEXT_POOL_SIZE];
        let ptr = data.as_mut_ptr();
        let buf = unsafe { AoSBuffer::from_raw(ptr, data.len()) };

        // Default is 0
        assert_eq!(buf.cursor_alt_char(0), 0);

        // Set cursor alt char
        let node_base = HEADER_SIZE + 0 * STRIDE;
        unsafe {
            std::ptr::write_unaligned(ptr.add(node_base + U_CURSOR_ALT_CHAR) as *mut u32, 0x2591); // light shade
        }
        assert_eq!(buf.cursor_alt_char(0), 0x2591);
    }
}
