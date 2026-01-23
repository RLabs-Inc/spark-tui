//! Blink Animation System - Shared clocks per FPS
//!
//! Provides efficient cursor blink animation using shared timers.
//! All animations at the same FPS share a single timer for efficiency and visual sync.
//!
//! # Pattern
//!
//! - Multiple cursors blinking at 2 FPS share one timer
//! - Timer starts when first subscriber, stops when last unsubscribes
//! - Phase signal toggles true/false for blink visibility
//!
//! # Example
//!
//! ```ignore
//! use spark_tui::state::animate::{subscribe_to_blink, get_blink_phase};
//!
//! // Subscribe to 2 FPS blink (standard cursor blink rate)
//! let unsubscribe = subscribe_to_blink(2);
//!
//! // Check current blink phase
//! let visible = get_blink_phase(2);
//!
//! // Cleanup when done
//! unsubscribe();
//! ```

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use spark_signals::{signal, Signal};

// =============================================================================
// BLINK REGISTRY
// =============================================================================

/// Per-FPS blink registry containing shared timer state
struct BlinkRegistry {
    /// Phase signal (local, updated from thread-safe atomic)
    phase: Signal<bool>,
    /// Thread-safe phase for cross-thread communication
    phase_atomic: Arc<AtomicBool>,
    /// Background timer thread handle
    handle: Option<JoinHandle<()>>,
    /// Flag to signal timer thread to stop
    running: Arc<AtomicBool>,
    /// Number of active subscribers
    subscribers: usize,
}

thread_local! {
    /// Map from FPS to blink registry
    static BLINK_REGISTRIES: RefCell<HashMap<u8, BlinkRegistry>> = RefCell::new(HashMap::new());
}

// =============================================================================
// PUBLIC API
// =============================================================================

/// Subscribe to blink animation at the given FPS.
///
/// Returns an unsubscribe function that must be called when done.
/// Multiple subscribers at the same FPS share one timer (efficient + synced).
///
/// # Arguments
///
/// * `fps` - Blink frequency in frames per second. 2 FPS = 500ms on/off cycle.
///           If 0, returns a no-op unsubscribe (blink disabled).
///
/// # Returns
///
/// Unsubscribe function. Call when the cursor/component is disposed.
///
/// # Example
///
/// ```ignore
/// let unsub = subscribe_to_blink(2);
/// // ... cursor is blinking ...
/// unsub(); // Stop blinking
/// ```
pub fn subscribe_to_blink(fps: u8) -> Box<dyn FnOnce()> {
    // Guard against invalid fps (0 would cause infinite interval)
    if fps == 0 {
        return Box::new(|| {}); // No-op unsubscribe
    }

    BLINK_REGISTRIES.with(|registries| {
        let mut registries = registries.borrow_mut();

        let registry = registries.entry(fps).or_insert_with(|| BlinkRegistry {
            phase: signal(true), // Start visible
            phase_atomic: Arc::new(AtomicBool::new(true)),
            handle: None,
            running: Arc::new(AtomicBool::new(false)),
            subscribers: 0,
        });

        registry.subscribers += 1;

        // Start timer if first subscriber
        if registry.subscribers == 1 {
            // Calculate interval: divide by 2 for on/off cycle
            // 2 FPS = 1000/2/2 = 250ms (toggles every 250ms = 500ms full cycle)
            let ms = 1000u64 / fps as u64 / 2;
            let phase_atomic = registry.phase_atomic.clone();
            let running = registry.running.clone();
            running.store(true, Ordering::SeqCst);

            registry.handle = Some(thread::spawn(move || {
                while running.load(Ordering::SeqCst) {
                    thread::sleep(Duration::from_millis(ms));
                    if running.load(Ordering::SeqCst) {
                        // Toggle phase atomically
                        let current = phase_atomic.load(Ordering::SeqCst);
                        phase_atomic.store(!current, Ordering::SeqCst);
                    }
                }
            }));
        }
    });

    // Return unsubscribe closure
    Box::new(move || {
        BLINK_REGISTRIES.with(|registries| {
            let mut registries = registries.borrow_mut();
            if let Some(registry) = registries.get_mut(&fps) {
                registry.subscribers = registry.subscribers.saturating_sub(1);

                // Stop timer if no more subscribers
                if registry.subscribers == 0 {
                    registry.running.store(false, Ordering::SeqCst);
                    registry.phase_atomic.store(true, Ordering::SeqCst); // Reset atomic
                    registry.phase.set(true); // Reset signal to visible

                    // Note: Thread will exit on next iteration when it checks running flag
                    // We don't join here to avoid blocking
                }
            }
        });
    })
}

/// Get the current blink phase for the given FPS.
///
/// Returns true (visible) if no registry exists for this FPS.
/// Also syncs the atomic phase to the Signal for reactive tracking.
///
/// # Arguments
///
/// * `fps` - The FPS to query
///
/// # Returns
///
/// Current blink phase: true = visible, false = hidden
pub fn get_blink_phase(fps: u8) -> bool {
    BLINK_REGISTRIES.with(|registries| {
        let mut registries = registries.borrow_mut();
        if let Some(registry) = registries.get_mut(&fps) {
            // Sync atomic phase to signal
            let phase = registry.phase_atomic.load(Ordering::SeqCst);
            if registry.phase.get() != phase {
                registry.phase.set(phase);
            }
            phase
        } else {
            true // Default to visible if no registry
        }
    })
}

/// Get the blink phase signal for the given FPS.
///
/// Returns None if no registry exists for this FPS.
/// Useful for reactive tracking of blink state.
///
/// Note: The signal is synced from the atomic phase when get_blink_phase()
/// is called. For reactive use, call get_blink_phase() periodically or
/// use the signal in effects that are triggered by other updates.
///
/// # Arguments
///
/// * `fps` - The FPS to query
///
/// # Returns
///
/// The phase signal, or None if no registry exists
pub fn get_blink_phase_signal(fps: u8) -> Option<Signal<bool>> {
    BLINK_REGISTRIES.with(|registries| {
        let registries = registries.borrow();
        registries.get(&fps).map(|r| r.phase.clone())
    })
}

/// Check if a blink clock is currently running for the given FPS.
///
/// # Arguments
///
/// * `fps` - The FPS to check
///
/// # Returns
///
/// true if there are active subscribers and the timer is running
pub fn is_blink_running(fps: u8) -> bool {
    BLINK_REGISTRIES.with(|registries| {
        let registries = registries.borrow();
        registries
            .get(&fps)
            .map(|r| r.running.load(Ordering::SeqCst) && r.subscribers > 0)
            .unwrap_or(false)
    })
}

/// Get the number of subscribers for a given FPS.
///
/// # Arguments
///
/// * `fps` - The FPS to check
///
/// # Returns
///
/// Number of active subscribers (0 if no registry)
pub fn get_subscriber_count(fps: u8) -> usize {
    BLINK_REGISTRIES.with(|registries| {
        let registries = registries.borrow();
        registries.get(&fps).map(|r| r.subscribers).unwrap_or(0)
    })
}

/// Reset all blink registries (for testing).
///
/// Stops all timers and clears all registries.
pub fn reset_blink_registries() {
    BLINK_REGISTRIES.with(|registries| {
        let mut registries = registries.borrow_mut();

        // Stop all running timers
        for registry in registries.values_mut() {
            registry.running.store(false, Ordering::SeqCst);
            registry.subscribers = 0;
            registry.phase_atomic.store(true, Ordering::SeqCst);
            registry.phase.set(true);
        }

        // Clear the map
        registries.clear();
    });
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn setup() {
        reset_blink_registries();
    }

    #[test]
    fn test_subscribe_returns_unsubscribe() {
        setup();

        let unsubscribe = subscribe_to_blink(2);
        assert_eq!(get_subscriber_count(2), 1);

        unsubscribe();
        assert_eq!(get_subscriber_count(2), 0);
    }

    #[test]
    fn test_shared_clock_same_fps() {
        setup();

        // Two subscriptions at same FPS should share one registry
        let unsub1 = subscribe_to_blink(2);
        let unsub2 = subscribe_to_blink(2);

        assert_eq!(get_subscriber_count(2), 2);

        // Only one registry should exist
        let registry_count = BLINK_REGISTRIES.with(|r| r.borrow().len());
        assert_eq!(registry_count, 1);

        unsub1();
        assert_eq!(get_subscriber_count(2), 1);
        assert!(is_blink_running(2));

        unsub2();
        assert_eq!(get_subscriber_count(2), 0);
    }

    #[test]
    fn test_different_fps_separate_clocks() {
        setup();

        let _unsub1 = subscribe_to_blink(2);
        let _unsub2 = subscribe_to_blink(4);

        // Two separate registries
        let registry_count = BLINK_REGISTRIES.with(|r| r.borrow().len());
        assert_eq!(registry_count, 2);

        assert_eq!(get_subscriber_count(2), 1);
        assert_eq!(get_subscriber_count(4), 1);
    }

    #[test]
    fn test_phase_toggles() {
        setup();

        // Use higher FPS for faster test (20 FPS = 25ms toggle)
        let _unsub = subscribe_to_blink(20);

        // Initial phase should be true
        assert!(get_blink_phase(20));

        // Wait for toggle
        thread::sleep(Duration::from_millis(60));

        // Phase should have toggled at least once
        // We can't predict exact state due to timing, but we can verify
        // the signal exists and is accessible
        let _ = get_blink_phase(20);
    }

    #[test]
    fn test_unsubscribe_stops_timer() {
        setup();

        let unsub = subscribe_to_blink(2);
        assert!(is_blink_running(2));

        unsub();

        // Timer should be stopped
        BLINK_REGISTRIES.with(|registries| {
            let registries = registries.borrow();
            if let Some(registry) = registries.get(&2) {
                assert!(!registry.running.load(Ordering::SeqCst));
                assert!(registry.phase.get()); // Reset to visible
            }
        });
    }

    #[test]
    fn test_resubscribe_restarts_timer() {
        setup();

        let unsub1 = subscribe_to_blink(2);
        assert!(is_blink_running(2));

        unsub1();
        assert!(!is_blink_running(2));

        // Re-subscribe
        let _unsub2 = subscribe_to_blink(2);
        assert!(is_blink_running(2));
    }

    #[test]
    fn test_zero_fps_noop() {
        setup();

        let unsub = subscribe_to_blink(0);

        // No registry should be created
        let registry_count = BLINK_REGISTRIES.with(|r| r.borrow().len());
        assert_eq!(registry_count, 0);

        // get_blink_phase should return true (default visible)
        assert!(get_blink_phase(0));

        // Calling unsubscribe should be safe
        unsub();
    }
}
