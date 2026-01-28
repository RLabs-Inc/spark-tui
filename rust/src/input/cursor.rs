//! Cursor blink management.
//!
//! This is the ONE place where timing enters the system.
//! Blink timers fire → write to SharedBuffer → reactive graph propagates → render.
//! It's a signal SOURCE (external event entering the reactive graph), NOT a pipeline loop.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::shared_buffer::SharedBuffer;

/// Manages cursor blink clocks.
///
/// Multiple inputs can share the same blink clock if they have the same FPS.
/// When no inputs are focused, all clocks are stopped (zero CPU).
pub struct BlinkManager {
    /// Shared clocks keyed by FPS. Value: (last_toggle_time, current_phase).
    clocks: HashMap<u32, BlinkClock>,
    /// Active subscriptions: component_index → fps.
    subscriptions: HashMap<usize, u32>,
}

struct BlinkClock {
    interval: Duration,
    last_toggle: Instant,
    phase: bool,
}

impl BlinkManager {
    pub fn new() -> Self {
        Self {
            clocks: HashMap::new(),
            subscriptions: HashMap::new(),
        }
    }

    /// Subscribe a component to blink at a given FPS.
    pub fn subscribe(&mut self, component_index: usize, fps: u32) {
        if fps == 0 {
            return;
        }
        self.subscriptions.insert(component_index, fps);
        self.clocks.entry(fps).or_insert_with(|| BlinkClock {
            interval: Duration::from_millis(1000 / (fps * 2) as u64), // Toggle every half-period
            last_toggle: Instant::now(),
            phase: true,
        });
    }

    /// Unsubscribe a component from blink.
    pub fn unsubscribe(&mut self, component_index: usize) {
        if let Some(fps) = self.subscriptions.remove(&component_index) {
            // Remove clock if no more subscribers at this FPS
            let has_subscribers = self.subscriptions.values().any(|&f| f == fps);
            if !has_subscribers {
                self.clocks.remove(&fps);
            }
        }
    }

    /// Tick all clocks. Returns true if any phase changed.
    /// Call this from the main thread when a blink timer fires.
    pub fn tick(&mut self, buf: &SharedBuffer) -> bool {
        let now = Instant::now();
        let mut any_changed = false;

        for (fps, clock) in &mut self.clocks {
            if now.duration_since(clock.last_toggle) >= clock.interval {
                clock.phase = !clock.phase;
                clock.last_toggle = now;

                // Update all components subscribed to this FPS
                for (&comp_idx, &comp_fps) in &self.subscriptions {
                    if comp_fps == *fps {
                        buf.set_cursor_visible(comp_idx, clock.phase);
                        any_changed = true;
                    }
                }
            }
        }

        any_changed
    }

    /// Get the next blink deadline (for sleep/timeout calculation).
    pub fn next_deadline(&self) -> Option<Duration> {
        if self.clocks.is_empty() {
            return None;
        }

        let now = Instant::now();
        let mut min_wait = Duration::MAX;

        for clock in self.clocks.values() {
            let elapsed = now.duration_since(clock.last_toggle);
            if elapsed >= clock.interval {
                return Some(Duration::ZERO); // Already due
            }
            let remaining = clock.interval - elapsed;
            if remaining < min_wait {
                min_wait = remaining;
            }
        }

        Some(min_wait)
    }

    /// Check if any clocks are active.
    pub fn has_active(&self) -> bool {
        !self.clocks.is_empty()
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

    #[test]
    fn test_blink_manager_subscribe() {
        let mut bm = BlinkManager::new();
        assert!(!bm.has_active());

        bm.subscribe(0, 2);
        assert!(bm.has_active());
        assert_eq!(bm.subscriptions.len(), 1);

        bm.unsubscribe(0);
        assert!(!bm.has_active());
    }

    #[test]
    fn test_blink_manager_shared_clock() {
        let mut bm = BlinkManager::new();
        bm.subscribe(0, 2);
        bm.subscribe(1, 2);
        assert_eq!(bm.clocks.len(), 1); // Shared clock

        bm.unsubscribe(0);
        assert_eq!(bm.clocks.len(), 1); // Still has subscriber

        bm.unsubscribe(1);
        assert_eq!(bm.clocks.len(), 0); // No subscribers
    }

    #[test]
    fn test_next_deadline_empty() {
        let bm = BlinkManager::new();
        assert!(bm.next_deadline().is_none());
    }
}
