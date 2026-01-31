//! Wake watcher — adaptive spin-wait for TS → Rust notification.
//!
//! Monitors the wake flag in SharedBuffer using adaptive spinning:
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

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::input::reader::StdinMessage;
use crate::shared_buffer::SharedBuffer;

// =============================================================================
// WAKE WATCHER
// =============================================================================

/// Adaptive spin-wait wake watcher thread.
///
/// Monitors the SharedBuffer wake flag and forwards wake events
/// through the unified engine channel.
pub struct WakeWatcher {
    handle: Option<JoinHandle<()>>,
}

impl WakeWatcher {
    /// Spawn the wake watcher thread.
    ///
    /// - `buf`: SharedBuffer with the wake flag to monitor
    /// - `tx`: Sender for the unified engine channel (shared with stdin reader)
    /// - `running`: Shared shutdown flag
    pub fn spawn(
        buf: &'static SharedBuffer,
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
        buf: &'static SharedBuffer,
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

// =============================================================================
// FFI TEST FUNCTIONS
// =============================================================================
//
// These are FFI exports for TypeScript to test the cross-language wake
// mechanism. They validate that JS Atomics.notify() properly wakes Rust
// threads waiting on shared memory.

/// Test the adaptive spin-wait mechanism.
///
/// Buffer layout (24 bytes):
///   [0]:  u32 — wake flag (JS stores 1)
///   [1]:  u32 — result (0=waiting, 1=detected)
///   [2]:  u32 — latency in microseconds
///   [3]:  u32 — phase when detected (1=spin, 2=yield, 3=sleep)
///   [4]:  u32 — iteration count when detected
///   [5]:  u32 — reserved
#[unsafe(no_mangle)]
pub extern "C" fn spark_test_adaptive_wake(ptr: *mut u8) -> u32 {
    if ptr.is_null() {
        return 1;
    }

    let addr = ptr as usize;

    thread::Builder::new()
        .name("adaptive-wake-test".into())
        .spawn(move || {
            let flag = unsafe { &*(addr as *const AtomicU32) };
            let result = unsafe { &*((addr + 4) as *const AtomicU32) };
            let latency = unsafe { &*((addr + 8) as *const AtomicU32) };
            let phase_out = unsafe { &*((addr + 12) as *const AtomicU32) };
            let iter_out = unsafe { &*((addr + 16) as *const AtomicU32) };

            let start = std::time::Instant::now();
            let mut idle_count: u32 = 0;

            loop {
                if flag.load(Ordering::Acquire) != 0 {
                    let elapsed = start.elapsed().as_micros() as u32;
                    let phase = if idle_count < 64 {
                        1
                    } else if idle_count < 256 {
                        2
                    } else {
                        3
                    };

                    result.store(1, Ordering::Release);
                    latency.store(elapsed, Ordering::Release);
                    phase_out.store(phase, Ordering::Release);
                    iter_out.store(idle_count, Ordering::Release);
                    return;
                }

                idle_count = idle_count.saturating_add(1);
                if idle_count < 64 {
                    std::hint::spin_loop();
                } else if idle_count < 256 {
                    thread::yield_now();
                } else {
                    thread::sleep(Duration::from_micros(50));
                }
            }
        })
        .ok();

    0
}

/// Test cross-language atomic wake mechanisms.
///
/// Buffer layout (16 bytes, must be SharedArrayBuffer):
///   [0..4]:   u32 — flag (JS stores 1 + Atomics.notify)
///   [4..8]:   u32 — result (0=waiting, 1=woken)
///   [8..12]:  u32 — latency in microseconds
///   [12..16]: u32 — reserved
///
/// mode: 0 = atomic_wait::wait, 1 = spin loop (control), 2 = wait_on_address, 3 = ecmascript_futex
#[unsafe(no_mangle)]
pub extern "C" fn spark_test_atomic_wait(ptr: *mut u8, mode: u32) -> u32 {
    if ptr.is_null() {
        return 1;
    }

    let addr = ptr as usize;

    thread::Builder::new()
        .name("atomic-wait-test".into())
        .spawn(move || {
            let flag = unsafe { &*(addr as *const AtomicU32) };
            let result = unsafe { &*((addr + 4) as *const AtomicU32) };
            let latency = unsafe { &*((addr + 8) as *const AtomicU32) };

            let start = std::time::Instant::now();

            match mode {
                // Mode 0: atomic_wait::wait — THE critical test
                // macOS: libc++ atomic_wait (same ABI as C++20 std::atomic_wait)
                // Linux: futex(FUTEX_WAIT)
                0 => {
                    atomic_wait::wait(flag, 0);
                    let elapsed = start.elapsed().as_micros() as u32;
                    result.store(1, Ordering::Release);
                    latency.store(elapsed, Ordering::Release);
                    eprintln!("[rust] atomic_wait::wait woke after {}μs", elapsed);
                }
                // Mode 1: Spin on atomic load (control — always works)
                1 => {
                    loop {
                        if flag.load(Ordering::Acquire) != 0 {
                            break;
                        }
                        std::hint::spin_loop();
                    }
                    let elapsed = start.elapsed().as_micros() as u32;
                    result.store(1, Ordering::Release);
                    latency.store(elapsed, Ordering::Release);
                    eprintln!("[rust] spin detected change in {}μs", elapsed);
                }
                // Mode 2: wait_on_address — uses os_sync_wait_on_address on macOS 14.4+
                2 => {
                    use wait_on_address::AtomicWait;
                    flag.wait(0);
                    let elapsed = start.elapsed().as_micros() as u32;
                    result.store(1, Ordering::Release);
                    latency.store(elapsed, Ordering::Release);
                    eprintln!("[rust] wait_on_address woke after {}μs", elapsed);
                }
                // Mode 3: ecmascript_futex — ECMAScript memory model futex
                // macOS: os_sync_wait_on_address via Racy<u32> (ECMAScript-compatible)
                3 => {
                    use std::ptr::NonNull;
                    use ecmascript_atomics::RacyMemory;
                    use ecmascript_futex::ECMAScriptAtomicWait;

                    let u32_ptr = NonNull::new(addr as *mut u32).unwrap();
                    let slice_ptr = NonNull::slice_from_raw_parts(u32_ptr, 4);
                    let racy_mem = unsafe { RacyMemory::<u32>::enter_slice(slice_ptr) };

                    {
                        let slice = racy_mem.as_slice();
                        let racy_flag = slice.get(0).unwrap();
                        let racy_result = slice.get(1).unwrap();
                        let racy_latency = slice.get(2).unwrap();

                        // Wait while value == 0 (block until JS stores 1)
                        let wait_result = racy_flag.wait(0);
                        let elapsed = start.elapsed().as_micros() as u32;

                        racy_result.store(1, ecmascript_atomics::Ordering::SeqCst);
                        racy_latency.store(elapsed, ecmascript_atomics::Ordering::SeqCst);
                        eprintln!("[rust] ecmascript_futex woke after {}μs (wait result: {:?})", elapsed, wait_result);
                    }

                    // Don't exit — JS owns this memory. Just forget the handle.
                    std::mem::forget(racy_mem);
                }
                _ => {
                    result.store(3, Ordering::Release);
                }
            }
        })
        .ok();

    0
}
