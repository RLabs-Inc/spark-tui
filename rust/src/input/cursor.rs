//! Cursor blink state reader.
//!
//! In SparkTUI's architecture, cursor blink timing is handled by TypeScript's
//! `pulse()` signal which writes `cursor_visible` to SharedBuffer. Rust just
//! reads this value — NO timing logic here.
//!
//! This module exists only to satisfy the import in setup.rs. It will be
//! removed entirely once setup.rs is updated.

use crate::shared_buffer_aos::AoSBuffer;

/// Stub for cursor blink management.
///
/// In the reactive architecture, cursor blink is driven by TS pulse() signal.
/// Rust just reads cursor_visible from the buffer — no timing here.
pub struct BlinkManager;

impl BlinkManager {
    pub fn new() -> Self {
        Self
    }

    /// No-op: blink timing is handled by TS pulse() signal
    pub fn subscribe(&mut self, _component_index: usize, _fps: u32) {}

    /// No-op
    pub fn unsubscribe(&mut self, _component_index: usize) {}

    /// No-op: returns false (no state change from Rust side)
    pub fn tick(&mut self, _buf: &AoSBuffer) -> bool {
        false
    }

    /// No deadline: blink timing is TS-side
    pub fn next_deadline(&self) -> Option<std::time::Duration> {
        None
    }

    /// Always false: no active clocks in Rust
    pub fn has_active(&self) -> bool {
        false
    }
}

impl Default for BlinkManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared_buffer_aos::{HEADER_SIZE, STRIDE, TEXT_POOL_SIZE, MAX_NODES, EVENT_RING_SIZE};

    #[test]
    fn test_blink_manager_stub() {
        let mut bm = BlinkManager::new();

        // All methods should be no-ops
        assert!(!bm.has_active());
        assert!(bm.next_deadline().is_none());

        bm.subscribe(0, 2);
        assert!(!bm.has_active()); // Still false - no timing in Rust

        bm.unsubscribe(0);
        assert!(!bm.has_active());
    }

    #[test]
    fn test_blink_manager_tick_noop() {
        let mut bm = BlinkManager::new();
        let mut data = vec![0u8; HEADER_SIZE + MAX_NODES * STRIDE + TEXT_POOL_SIZE + EVENT_RING_SIZE];
        let buf = unsafe { crate::shared_buffer_aos::AoSBuffer::from_raw(data.as_mut_ptr(), data.len()) };

        // tick() should always return false (no state changes from Rust)
        assert!(!bm.tick(&buf));

        bm.subscribe(0, 2);
        assert!(!bm.tick(&buf)); // Still false
    }

    #[test]
    fn test_blink_manager_default() {
        let bm = BlinkManager::default();
        assert!(!bm.has_active());
    }
}
