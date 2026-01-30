//! Wake watcher — adaptive spin-wait for TS → Rust notification.
//!
//! Monitors the wake flag in AoSBuffer using adaptive spinning:
//!
//! - Active use (typing, mouse):  spin_loop()  → nanosecond detection
//! - Brief idle (between keys):   yield_now()  → microsecond detection
//! - Long idle (no interaction):  sleep(50μs)  → 50μs max latency, ~0% CPU
//!
//! When a wake is detected, sends a message through the same mpsc channel
//! as stdin — so the engine thread wakes immediately from either source.
//! After detection, resets to tight spinning for the burst of activity
//! that typically follows.
//!
//! No FFI from TS. No polling in the engine thread. Pure shared memory.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::input::reader::StdinMessage;
use crate::shared_buffer_aos::AoSBuffer;

/// Adaptive spin-wait wake watcher thread.
///
/// Monitors the AoSBuffer wake flag and forwards wake events
/// through the unified engine channel.
pub struct WakeWatcher {
    handle: Option<JoinHandle<()>>,
}

impl WakeWatcher {
    /// Spawn the wake watcher thread.
    ///
    /// - `buf`: AoSBuffer with the wake flag to monitor
    /// - `tx`: Sender for the unified engine channel (shared with stdin reader)
    /// - `running`: Shared shutdown flag
    pub fn spawn(
        buf: &'static AoSBuffer,
        tx: Sender<StdinMessage>,
        running: Arc<AtomicBool>,
    ) -> Self {
        let handle = thread::Builder::new()
            .name("spark-wake".to_string())
            .spawn(move || {
                Self::watch_loop(buf, tx, running);
            })
            .expect("Failed to spawn wake watcher thread");

        Self {
            handle: Some(handle),
        }
    }

    fn watch_loop(
        buf: &'static AoSBuffer,
        tx: Sender<StdinMessage>,
        running: Arc<AtomicBool>,
    ) {
        let mut idle_count: u32 = 0;

        while running.load(Ordering::Relaxed) {
            if buf.consume_wake() {
                // Drain coalesced wakes (multiple TS microtasks may have fired)
                while buf.consume_wake() {}

                if tx.send(StdinMessage::Wake).is_err() {
                    break; // Channel closed, engine shutting down
                }

                // Reset to tight spin — activity burst expected
                idle_count = 0;
                continue;
            }

            // Adaptive backoff
            idle_count = idle_count.saturating_add(1);
            if idle_count < 64 {
                // Phase 1: tight spin — nanosecond detection
                // PAUSE on x86, YIELD on ARM
                std::hint::spin_loop();
            } else if idle_count < 256 {
                // Phase 2: OS yield — microsecond detection
                thread::yield_now();
            } else {
                // Phase 3: short sleep — 50μs max latency, ~0% CPU
                thread::sleep(Duration::from_micros(50));
            }
        }
    }
}

impl Drop for WakeWatcher {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = handle;
        }
    }
}
