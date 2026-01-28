//! SparkTUI Engine - Rust side of the hybrid TUI framework.
//!
//! This cdylib receives a SharedArrayBuffer pointer from TypeScript (via Bun FFI),
//! reads layout properties, runs Taffy flexbox computation, builds the framebuffer,
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
//!   │            SharedArrayBuffer (~2MB+)                │
//!   │  Layout arrays ← TS writes, Rust reads              │
//!   │  Visual arrays ← TS writes, Rust reads              │
//!   │  Text pool     ← TS writes (+ Rust for input edits) │
//!   │  Interaction   ← Rust writes (focus, scroll, hover)  │
//!   │  Output        ← Rust writes (computed layout)        │
//!   │  Event Ring    ← Rust writes, TS reads (callbacks)    │
//!   └────────────────────────────────────────────────────┘
//! ```

pub mod shared_buffer;
pub mod types;
pub mod layout;
pub mod renderer;
pub mod framebuffer;
pub mod input;
pub mod pipeline;

use shared_buffer::SharedBuffer;
use std::sync::OnceLock;

// =============================================================================
// GLOBAL STATE
// =============================================================================

/// The shared buffer, initialized once via FFI.
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
/// 1. Creates the SharedBuffer view
/// 2. Starts the engine thread (terminal setup, stdin, reactive pipeline)
///
/// Called once from TypeScript:
/// ```typescript
/// const lib = dlopen("./spark_tui_engine.dylib", { spark_init: { args: ["ptr", "u32"], returns: "u32" } })
/// lib.symbols.spark_init(buffer.ptr, buffer.byteLength)
/// ```
#[unsafe(no_mangle)]
pub extern "C" fn spark_init(ptr: *mut u8, len: u32) -> u32 {
    let buf = unsafe { SharedBuffer::from_ptr(ptr, len as usize) };
    match BUFFER.set(buf) {
        Ok(_) => {
            eprintln!("[spark-engine] Initialized with {}KB shared buffer ({} max nodes)",
                len / 1024, shared_buffer::MAX_NODES);

            // Start the engine
            let buf = get_buffer();
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

/// Compute layout using Taffy's low-level trait API and write results to the output section.
///
/// Uses `LayoutTree` which implements Taffy's traits directly on SharedBuffer.
/// NodeId IS the component index — zero translation, zero double-bookkeeping.
///
/// Returns the number of nodes laid out.
#[unsafe(no_mangle)]
pub extern "C" fn spark_compute_layout() -> u32 {
    let buf = get_buffer();
    layout::compute_layout_direct(buf)
}

/// Compute framebuffer from SharedBuffer data and render to terminal.
///
/// Called by TS after writing props to trigger a render.
/// In the full reactive pipeline, this happens automatically via the engine thread.
#[unsafe(no_mangle)]
pub extern "C" fn spark_render() -> u32 {
    let buf = get_buffer();
    let tw = buf.terminal_width();
    let th = buf.terminal_height();

    // Layout
    layout::compute_layout_direct(buf);

    // Framebuffer
    let (fb, _hit_regions) = framebuffer::compute_framebuffer(buf, tw, th);

    // We can't keep a renderer in a static easily, so just report buffer size
    (fb.width() as u32) * (fb.height() as u32)
}

/// Get the total shared buffer size needed (for TypeScript to allocate).
#[unsafe(no_mangle)]
pub extern "C" fn spark_buffer_size() -> u32 {
    shared_buffer::TOTAL_BUFFER_SIZE as u32
}

/// Wake the engine (TS calls this after writing props to SharedBuffer).
#[unsafe(no_mangle)]
pub extern "C" fn spark_wake() {
    let buf = get_buffer();
    buf.set_wake_flag();
}

/// Stop the engine and clean up.
#[unsafe(no_mangle)]
pub extern "C" fn spark_cleanup() {
    if let Some(engine) = ENGINE.get() {
        engine.stop();
    }
}
