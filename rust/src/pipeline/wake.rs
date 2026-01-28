//! Wake mechanism for TS → Rust notification.
//!
//! The "sleepy thread" pattern:
//! - Thread blocks on futex_wait (or equivalent) until TS writes wake flag
//! - When woken: increments generation → reactive graph propagates
//! - After render: clears wake flag, thread sleeps again
//!
//! NOT a polling loop — it's a notification mechanism.

use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;
use std::time::Duration;

/// Poll the wake flag in the SharedBuffer header.
///
/// This is a simple implementation that checks the wake flag periodically.
/// On platforms that support it, this could use futex_wait for true zero-CPU
/// blocking. For now, we use a short sleep as a portable fallback.
///
/// Returns true if woken (wake flag was set).
pub fn wait_for_wake(wake_flag: &AtomicU32, timeout: Duration) -> bool {
    // First check without sleeping
    if wake_flag.swap(0, Ordering::AcqRel) != 0 {
        return true;
    }

    // On Linux, we could use futex_wait here for zero-CPU blocking.
    // For macOS portability, we use a short sleep.
    thread::sleep(timeout.min(Duration::from_millis(1)));

    // Check again after sleep
    wake_flag.swap(0, Ordering::AcqRel) != 0
}
