//! Memory test for SparkTUI's threading architecture.
//!
//! Simulates the exact pattern SparkTUI uses:
//! - Wake watcher thread (adaptive spin)
//! - Channel feeding engine thread
//! - Reactive pipeline (signal → derived → effect)
//! - FrameBuffer allocation each frame
//!
//! NO FFI, NO Bun - pure Rust.
//!
//! Run with: cargo test --test memory_threads -- --nocapture

use std::cell::Cell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use spark_signals::{signal, derived, effect, flush_sync};

// =============================================================================
// FAKE SHARED BUFFER (simulates the wake flag)
// =============================================================================

struct FakeSharedBuffer {
    wake_flag: AtomicU32,
}

impl FakeSharedBuffer {
    fn new() -> Self {
        Self {
            wake_flag: AtomicU32::new(0),
        }
    }

    fn set_wake(&self) {
        self.wake_flag.store(1, Ordering::Release);
    }

    fn consume_wake(&self) -> bool {
        self.wake_flag.swap(0, Ordering::AcqRel) != 0
    }
}

// =============================================================================
// MESSAGE TYPE (same as SparkTUI)
// =============================================================================

enum Message {
    Wake,
    Stop,
}

// =============================================================================
// WAKE WATCHER (same adaptive spin as SparkTUI)
// =============================================================================

fn spawn_wake_watcher(
    buf: &'static FakeSharedBuffer,
    tx: Sender<Message>,
    running: Arc<AtomicBool>,
) -> JoinHandle<()> {
    thread::Builder::new()
        .name("wake-watcher".to_string())
        .spawn(move || {
            let mut idle_count: u32 = 0;

            while running.load(Ordering::Relaxed) {
                if buf.consume_wake() {
                    // Drain coalesced wakes
                    while buf.consume_wake() {}

                    if tx.send(Message::Wake).is_err() {
                        break;
                    }

                    idle_count = 0;
                    continue;
                }

                // Adaptive backoff (exactly like SparkTUI)
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
        .expect("spawn wake watcher")
}

// =============================================================================
// SIMULATED TYPES (like SparkTUI's FrameBuffer)
// =============================================================================

#[derive(Clone, PartialEq)]
struct FakeFrameBuffer {
    cells: Vec<u32>,
    width: u16,
    height: u16,
}

impl FakeFrameBuffer {
    fn new(width: u16, height: u16) -> Self {
        Self {
            cells: vec![0u32; width as usize * height as usize],
            width,
            height,
        }
    }
}

#[derive(Clone, PartialEq)]
struct FakeHitRegion {
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    component_index: u32,
}

#[derive(Clone, PartialEq)]
struct FrameResult {
    buffer: FakeFrameBuffer,
    hit_regions: Vec<FakeHitRegion>,
}

// =============================================================================
// MAIN TEST
// =============================================================================

#[test]
fn test_threading_memory() {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║     SPARKTUI THREADING MEMORY TEST (Pure Rust)             ║");
    println!("║                                                            ║");
    println!("║  Tests the exact threading pattern SparkTUI uses:          ║");
    println!("║  - Wake watcher with adaptive spin                         ║");
    println!("║  - mpsc channel to engine                                  ║");
    println!("║  - Reactive pipeline (signal → derived → effect)           ║");
    println!("║  - FrameBuffer allocation each frame                       ║");
    println!("║                                                            ║");
    println!("║  NO FFI, NO Bun - pure Rust                                ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Leak the buffer so it's 'static (like SparkTUI does with SharedBuffer)
    let buf: &'static FakeSharedBuffer = Box::leak(Box::new(FakeSharedBuffer::new()));

    let running = Arc::new(AtomicBool::new(true));
    let (tx, rx) = mpsc::channel();

    // Spawn wake watcher (like SparkTUI)
    let running_for_watcher = running.clone();
    let _wake_handle = spawn_wake_watcher(buf, tx.clone(), running_for_watcher);

    // Spawn "TS simulator" - thread that sets wake flags rapidly
    let running_for_ts = running.clone();
    let ts_handle = thread::Builder::new()
        .name("ts-simulator".to_string())
        .spawn(move || {
            let mut count = 0u64;
            while running_for_ts.load(Ordering::Relaxed) && count < 500_000 {
                buf.set_wake();
                count += 1;
                // No sleep - fire as fast as possible like real TS would
            }
            println!("  [TS Simulator] Sent {} wake signals", count);
        })
        .expect("spawn ts simulator");

    // Engine thread runs here (main test thread)
    println!("▶ Setting up reactive pipeline...\n");

    let effect_count = Rc::new(Cell::new(0u64));
    let effect_count_clone = effect_count.clone();

    let generation = signal(0u64);
    let gen_for_layout = generation.clone();
    let gen_for_fb = generation.clone();

    // Layout derived (like SparkTUI)
    let layout_derived = derived(move || {
        let g = gen_for_layout.get();
        // Simulate Taffy work
        let _ = (0..100).sum::<i32>();
        g
    });

    // Framebuffer derived (like SparkTUI)
    let layout_d = layout_derived.clone();
    let fb_derived = derived(move || {
        let _layout = layout_d.get();
        let _gen = gen_for_fb.get();

        // Allocate framebuffer (120x40 terminal)
        let buffer = FakeFrameBuffer::new(120, 40);

        // Hit regions (100 elements)
        let hit_regions: Vec<FakeHitRegion> = (0..100)
            .map(|i| FakeHitRegion {
                x: i as u16,
                y: 0,
                width: 10,
                height: 4,
                component_index: i,
            })
            .collect();

        FrameResult { buffer, hit_regions }
    });

    // Render effect (like SparkTUI)
    let running_for_effect = running.clone();
    let _stop = effect(move || {
        if !running_for_effect.load(Ordering::Relaxed) {
            return;
        }
        let result = fb_derived.get();
        let _ = result.buffer.cells.len();
        let _ = result.hit_regions.len();
        effect_count_clone.set(effect_count_clone.get() + 1);
    });

    flush_sync();
    println!("  Initial effect ran. Count: {}\n", effect_count.get());

    // ═══════════════════════════════════════════════════════════════
    // PHASE 1: Process 500,000 wake messages through channel
    // ═══════════════════════════════════════════════════════════════
    println!("▶ PHASE 1: Processing wake messages from channel...");
    println!("  (TS simulator sending 500K wakes as fast as possible)\n");

    let start = Instant::now();
    let mut wake_count = 0u64;
    let mut last_print = Instant::now();

    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(Message::Wake) => {
                wake_count += 1;
                generation.set(wake_count);
                flush_sync();

                if last_print.elapsed() > Duration::from_secs(1) {
                    println!("  Processed {} wakes, effect count: {}",
                             wake_count, effect_count.get());
                    last_print = Instant::now();
                }
            }
            Ok(Message::Stop) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // Check if TS simulator is done
                if !running.load(Ordering::Relaxed) || wake_count >= 500_000 {
                    break;
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }

        if wake_count >= 500_000 {
            break;
        }
    }

    println!("\n  Phase 1 complete in {:?}", start.elapsed());
    println!("  Processed {} wakes", wake_count);
    println!("  Effect count: {}\n", effect_count.get());

    // Wait for TS simulator to finish
    let _ = ts_handle.join();

    // ═══════════════════════════════════════════════════════════════
    // PHASE 2: Sleep - memory should stay flat
    // ═══════════════════════════════════════════════════════════════
    println!("▶ PHASE 2: SLEEPING 15 SECONDS");
    println!("  ┌─────────────────────────────────────────────────┐");
    println!("  │  WATCH MEMORY NOW!                              │");
    println!("  │  Wake watcher is still running (adaptive spin)  │");
    println!("  │  If memory grows = LEAK IN THREADING            │");
    println!("  └─────────────────────────────────────────────────┘\n");

    for i in 0..15 {
        thread::sleep(Duration::from_secs(1));
        println!("  Sleep: {} of 15 seconds...", i + 1);
    }
    println!();

    // ═══════════════════════════════════════════════════════════════
    // PHASE 3: Another burst
    // ═══════════════════════════════════════════════════════════════
    println!("▶ PHASE 3: Direct signal updates (no channel)");
    println!("  500,000 more updates...\n");

    let start = Instant::now();
    for i in 0..500_000u64 {
        generation.set(500_000 + i);
        flush_sync();

        if i % 100_000 == 0 {
            println!("  {} updates, effect count: {}", i, effect_count.get());
        }
    }

    println!("  Phase 3 complete in {:?}", start.elapsed());
    println!("  Effect count: {}\n", effect_count.get());

    // ═══════════════════════════════════════════════════════════════
    // PHASE 4: Final sleep
    // ═══════════════════════════════════════════════════════════════
    println!("▶ PHASE 4: FINAL SLEEP (15 seconds)\n");

    for i in 0..15 {
        thread::sleep(Duration::from_secs(1));
        println!("  Sleep: {} of 15 seconds...", i + 1);
    }

    // Stop the wake watcher
    running.store(false, Ordering::SeqCst);
    thread::sleep(Duration::from_millis(100));

    println!();
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║  TEST COMPLETE - 1,000,000 UPDATES                         ║");
    println!("║                                                            ║");
    println!("║  Final effect count: {:>10}                          ║", effect_count.get());
    println!("║                                                            ║");
    println!("║  If memory stayed flat → Threading is NOT leaking          ║");
    println!("║  If memory grew → Problem is in threading/channels         ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");
}
