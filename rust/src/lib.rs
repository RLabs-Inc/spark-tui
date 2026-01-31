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
use std::sync::OnceLock;

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
/// This sets the wake flag which the wake watcher thread monitors.
/// The engine will process the changes on the next reactive cycle.
#[unsafe(no_mangle)]
pub extern "C" fn spark_wake() {
    let buf = get_buffer();
    buf.set_wake_flag();
}

/// Stop the engine and clean up.
///
/// Call this before program exit to restore terminal state.
#[unsafe(no_mangle)]
pub extern "C" fn spark_cleanup() {
    if let Some(engine) = ENGINE.get() {
        engine.stop();
    }
}

// =============================================================================
// RE-EXPORTS: Wake mechanism test functions
// =============================================================================
//
// These are used by TypeScript integration tests to verify cross-language
// atomic wake works correctly. They live in pipeline/wake.rs.

pub use pipeline::wake::{spark_test_adaptive_wake, spark_test_atomic_wait};
