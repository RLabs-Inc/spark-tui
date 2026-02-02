//! Wake watcher — FFI-triggered thread parking for TS → Rust notification.
//!
//! Uses `std::thread::park/unpark` for true 0% CPU idle with instant wake:
//!
//! 1. Watcher thread calls `thread::park()` — blocks, 0% CPU
//! 2. TS calls FFI `spark_wake()` → Rust calls `thread.unpark()` — instant wake
//! 3. Watcher processes the wake, sends to engine channel, parks again
//!
//! This replaces the adaptive spin-wait approach which burned 10%+ CPU.
//! FFI call overhead is ~5ns, unpark latency is ~1-2μs.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Arc, OnceLock};
use std::thread::{self, JoinHandle, Thread};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::input::reader::StdinMessage;
use crate::shared_buffer::SharedBuffer;

// =============================================================================
// GLOBAL THREAD HANDLE
// =============================================================================

/// The wake thread's handle, stored for FFI to unpark.
static WAKE_THREAD: OnceLock<Thread> = OnceLock::new();

/// Unpark the wake thread. Called by FFI `spark_wake()`.
pub fn unpark_wake_thread() {
    if let Some(thread) = WAKE_THREAD.get() {
        thread.unpark();
    }
}

// =============================================================================
// WAKE WATCHER
// =============================================================================

/// Park-based wake watcher thread.
///
/// Blocks on `thread::park()` with 0% CPU, wakes instantly when
/// FFI `spark_wake()` calls `unpark()`.
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
                // Store this thread's handle so FFI can unpark us
                let _ = WAKE_THREAD.set(thread::current());

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
        while running.load(Ordering::Relaxed) {
            // Check for wake flag (may have been set before we parked)
            if buf.consume_wake() {
                // Drain coalesced wakes (multiple TS writes may have fired)
                while buf.consume_wake() {}

                // === Instrumentation ===
                let ts_notify_us = buf.ts_notify_timestamp();
                let now_us = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_micros() as u64)
                    .unwrap_or(0);

                if ts_notify_us > 0 && now_us >= ts_notify_us {
                    let latency_us = (now_us - ts_notify_us) as u32;
                    buf.set_wake_latency_us(latency_us);
                }
                buf.increment_wake_count();
                // === End instrumentation ===

                if tx.send(StdinMessage::Wake).is_err() {
                    break; // Channel closed, engine shutting down
                }

                // Don't park immediately — check for more wakes first
                continue;
            }

            // No wake pending — park until FFI unparks us (0% CPU, instant wake)
            thread::park();
        }
    }
}

impl Drop for WakeWatcher {
    fn drop(&mut self) {
        // Unpark to ensure the thread can exit if it's parked
        if let Some(thread) = WAKE_THREAD.get() {
            thread.unpark();
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

// =============================================================================
// FFI TEST FUNCTIONS (kept for reference/testing)
// =============================================================================

use std::sync::atomic::AtomicU32;

/// Test the park/unpark mechanism.
///
/// Buffer layout (24 bytes):
///   [0]:  u32 — wake flag (FFI stores 1 + unparks)
///   [1]:  u32 — result (0=waiting, 1=detected)
///   [2]:  u32 — latency in microseconds
///   [3]:  u32 — reserved
///   [4]:  u32 — reserved
///   [5]:  u32 — reserved
#[unsafe(no_mangle)]
pub extern "C" fn spark_test_adaptive_wake(ptr: *mut u8) -> u32 {
    if ptr.is_null() {
        return 1;
    }

    let addr = ptr as usize;

    thread::Builder::new()
        .name("park-wake-test".into())
        .spawn(move || {
            let flag = unsafe { &*(addr as *const AtomicU32) };
            let result = unsafe { &*((addr + 4) as *const AtomicU32) };
            let latency = unsafe { &*((addr + 8) as *const AtomicU32) };

            let start = std::time::Instant::now();

            // Park until woken
            loop {
                if flag.load(Ordering::Acquire) != 0 {
                    let elapsed = start.elapsed().as_micros() as u32;
                    result.store(1, Ordering::Release);
                    latency.store(elapsed, Ordering::Release);
                    return;
                }
                thread::park();
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
                // Mode 0: atomic_wait::wait
                0 => {
                    atomic_wait::wait(flag, 0);
                    let elapsed = start.elapsed().as_micros() as u32;
                    result.store(1, Ordering::Release);
                    latency.store(elapsed, Ordering::Release);
                    eprintln!("[rust] atomic_wait::wait woke after {}μs", elapsed);
                }
                // Mode 1: Spin on atomic load (control)
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
                // Mode 2: wait_on_address
                2 => {
                    use wait_on_address::AtomicWait;
                    flag.wait(0);
                    let elapsed = start.elapsed().as_micros() as u32;
                    result.store(1, Ordering::Release);
                    latency.store(elapsed, Ordering::Release);
                    eprintln!("[rust] wait_on_address woke after {}μs", elapsed);
                }
                // Mode 3: ecmascript_futex
                3 => {
                    use ecmascript_atomics::RacyMemory;
                    use ecmascript_futex::ECMAScriptAtomicWait;
                    use std::ptr::NonNull;

                    let u32_ptr = NonNull::new(addr as *mut u32).unwrap();
                    let slice_ptr = NonNull::slice_from_raw_parts(u32_ptr, 4);
                    let racy_mem = unsafe { RacyMemory::<u32>::enter_slice(slice_ptr) };

                    {
                        let slice = racy_mem.as_slice();
                        let racy_flag = slice.get(0).unwrap();
                        let racy_result = slice.get(1).unwrap();
                        let racy_latency = slice.get(2).unwrap();

                        let wait_result = racy_flag.wait(0);
                        let elapsed = start.elapsed().as_micros() as u32;

                        racy_result.store(1, ecmascript_atomics::Ordering::SeqCst);
                        racy_latency.store(elapsed, ecmascript_atomics::Ordering::SeqCst);
                        eprintln!(
                            "[rust] ecmascript_futex woke after {}μs (wait result: {:?})",
                            elapsed, wait_result
                        );
                    }

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
