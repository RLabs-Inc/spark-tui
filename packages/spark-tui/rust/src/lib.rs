//! SparkTUI Engine — Rust side of the hybrid TUI framework.
//!
//! This cdylib receives a SharedArrayBuffer pointer from TypeScript (via Bun FFI),
//! reads layout properties, runs Taffy layout computation, builds the framebuffer,
//! diff-renders to the terminal, and handles all input.
//!
//! # Architecture
//!
//! ```text
//! TypeScript (Developer API)                    Rust (The Engine)
//! ─────────────────────────                    ─────────────────
//! Developer writes components                   Rust OWNS everything:
//!   → box({ fg: red })                            stdin → state → layout →
//!   → text({ content: "hi" })                     framebuffer → diff → terminal
//!        │                                              │
//!        │ writes props                                 │ writes events
//!        ▼                                              ▼
//!   ┌────────────────────────────────────────────────────┐
//!   │         SharedArrayBuffer (~20MB default)          │
//!   │  1024 bytes per node (16 cache lines, contiguous)  │
//!   │  Layout, Visual, Text, Interaction, Output         │
//!   │  Event Ring Buffer (256 events)                    │
//!   └────────────────────────────────────────────────────┘
//! ```
//!
//! # Reactive Pipeline
//!
//! Pure reactive propagation. No loops. No polling. No fixed FPS.
//!
//! ```text
//! TS writes props → wake Rust → generation signal increments
//!   → layout derived (IF dirty: run Taffy, write output)
//!     → framebuffer derived (build 2D cell grid + hit regions)
//!       → render effect (diff → ANSI → terminal)
//! ```

// =============================================================================
// MODULES
// =============================================================================

pub mod shared_buffer;
pub mod utils;
pub mod layout;
pub mod renderer;
pub mod framebuffer;
pub mod input;
pub mod pipeline;

use shared_buffer::{SharedBuffer, DEFAULT_BUFFER_SIZE, calculate_buffer_size};
use std::sync::{OnceLock, Mutex, Condvar};

// =============================================================================
// GLOBAL STATE
// =============================================================================

/// The shared buffer (1024 bytes/node), initialized once via FFI.
static BUFFER: OnceLock<SharedBuffer> = OnceLock::new();

fn get_buffer() -> &'static SharedBuffer {
    BUFFER.get().expect("SharedBuffer not initialized - call spark_init() first")
}

/// Global engine handle.
static ENGINE: OnceLock<pipeline::Engine> = OnceLock::new();

/// Condvar for Rust→TS event notification.
/// TS calls spark_wait_for_events() which blocks on this.
/// Rust calls notify_ts_events() when events are written to ring buffer.
static TS_EVENT_SIGNAL: OnceLock<(Mutex<bool>, Condvar)> = OnceLock::new();

fn init_ts_event_signal() {
    let _ = TS_EVENT_SIGNAL.set((Mutex::new(false), Condvar::new()));
}

/// Signal TS that events are ready in the ring buffer.
/// Called internally by Rust when writing events.
pub fn notify_ts_events() {
    if let Some((lock, cvar)) = TS_EVENT_SIGNAL.get() {
        if let Ok(mut ready) = lock.lock() {
            *ready = true;
            cvar.notify_one();
        }
    }
}

// =============================================================================
// FFI EXPORTS
// =============================================================================

/// Initialize the engine with a pointer to the SharedArrayBuffer.
///
/// This:
/// 1. Creates the SharedBuffer view (1024 bytes per node, cache-aligned)
/// 2. Starts the engine thread (terminal setup, stdin, reactive pipeline)
///
/// Called once from TypeScript:
/// ```typescript
/// const lib = dlopen("./spark_tui_engine.dylib", {
///     spark_init: { args: ["ptr", "u32"], returns: "u32" }
/// });
/// lib.symbols.spark_init(buffer.ptr, buffer.byteLength);
/// ```
///
/// Returns: 0 = success, 1 = already initialized, 2 = engine start failed
#[unsafe(no_mangle)]
pub extern "C" fn spark_init(ptr: *mut u8, len: u32) -> u32 {
    let buf = unsafe { SharedBuffer::from_raw(ptr, len as usize) };

    // Initialize TS event signal (condvar for Rust→TS notification)
    init_ts_event_signal();

    match BUFFER.set(buf) {
        Ok(_) => {
            let buf = get_buffer();
            eprintln!(
                "[spark-engine] Initialized with {}MB buffer ({} max nodes, 1024 bytes/node)",
                len / (1024 * 1024),
                buf.max_nodes()
            );

            // Start the reactive engine
            match pipeline::Engine::start(buf) {
                Ok(engine) => {
                    let _ = ENGINE.set(engine);
                    0 // success
                }
                Err(e) => {
                    eprintln!("[spark-engine] Failed to start engine: {}", e);
                    2 // engine start failed
                }
            }
        }
        Err(_) => {
            eprintln!("[spark-engine] Already initialized!");
            1 // already init
        }
    }
}

/// Get the default shared buffer size for TypeScript to allocate.
///
/// Uses default configuration: 10,000 nodes, 10MB text pool.
/// Returns approximately 20.7MB.
#[unsafe(no_mangle)]
pub extern "C" fn spark_buffer_size() -> u32 {
    DEFAULT_BUFFER_SIZE as u32
}

/// Get custom shared buffer size for TypeScript to allocate.
///
/// Parameters:
/// - max_nodes: Maximum number of UI components
/// - text_pool_size: Bytes for text content storage
#[unsafe(no_mangle)]
pub extern "C" fn spark_buffer_size_custom(max_nodes: u32, text_pool_size: u32) -> u32 {
    calculate_buffer_size(max_nodes as usize, text_pool_size as usize) as u32
}

/// Wake the engine (TS calls this after writing props to SharedBuffer).
///
/// This sets the wake flag AND unparks the wake watcher thread.
/// The combination gives us:
/// - 0% CPU when idle (thread is parked)
/// - Instant wake (~1-2μs latency)
/// - FFI overhead: ~5ns
///
/// Safe to call before spark_init() — silently no-ops if engine isn't ready.
/// This allows TS to create the component tree before starting the engine,
/// with wake calls during construction being harmless no-ops.
#[unsafe(no_mangle)]
pub extern "C" fn spark_wake() {
    if let Some(buf) = BUFFER.get() {
        buf.set_wake_flag();
    }
    pipeline::wake::unpark_wake_thread();
}

/// Stop the engine and clean up.
///
/// Call this before program exit to restore terminal state.
#[unsafe(no_mangle)]
pub extern "C" fn spark_cleanup() {
    // Wake TS event loop so it can exit
    notify_ts_events();

    if let Some(engine) = ENGINE.get() {
        engine.stop();
    }
}

/// Wait for events from Rust (TS calls this).
///
/// Blocks until Rust writes events to the ring buffer.
/// This is the Rust→TS notification mechanism, symmetric with spark_wake().
///
/// - 0% CPU while waiting (condvar = kernel-level sleep)
/// - Instant wake when events arrive
/// - No polling, no fixed FPS
#[unsafe(no_mangle)]
pub extern "C" fn spark_wait_for_events() {
    if let Some((lock, cvar)) = TS_EVENT_SIGNAL.get() {
        if let Ok(mut ready) = lock.lock() {
            while !*ready {
                ready = cvar.wait(ready).unwrap();
            }
            *ready = false;
        }
    }
}

// =============================================================================
// RE-EXPORTS: Wake mechanism test functions
// =============================================================================
//
// These are used by TypeScript integration tests to verify cross-language
// atomic wake works correctly. They live in pipeline/wake.rs.

pub use pipeline::wake::{spark_test_adaptive_wake, spark_test_atomic_wait};

// =============================================================================
// BENCHMARKING FFI OVERHEAD
// =============================================================================

/// No-op function for benchmarking pure FFI call overhead.
/// Does absolutely nothing - measures only the JS→Rust→JS roundtrip cost.
#[unsafe(no_mangle)]
pub extern "C" fn spark_noop() {
    // Intentionally empty
}

/// No-op with args for benchmarking marshaling overhead.
#[unsafe(no_mangle)]
pub extern "C" fn spark_noop_args(_a: u32, _b: u32) -> u32 {
    0
}

/// No-op that touches an atomic to prevent over-optimization.
#[unsafe(no_mangle)]
pub extern "C" fn spark_noop_atomic() {
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    COUNTER.fetch_add(1, Ordering::Relaxed);
}
