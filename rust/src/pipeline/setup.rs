//! Engine setup — the main entry point for the Rust pipeline.
//!
//! Sets up the reactive graph using spark-signals:
//!
//! ```text
//! generation signal (incremented on TS wake or stdin input)
//!   │
//!   ├─→ layout_derived (spark-signals derived)
//!   │     reads generation → reads SharedBuffer dirty flags
//!   │     IF layout-dirty: run Taffy → write positions to output section
//!   │     IF visual-only-dirty: SKIP layout (smart skip)
//!   │
//!   ├─→ framebuffer_derived (spark-signals derived)
//!   │     depends on: layout_derived
//!   │     Read output + visual + text + interaction from SharedBuffer
//!   │     Build 2D Cell grid + collect hit regions
//!   │
//!   └─→ render_effect (spark-signals effect)
//!         depends on: framebuffer_derived
//!         Diff against previous framebuffer
//!         Write ANSI to stdout
//!         Update hit grid
//!         ONE effect. Fires because data changed. Period.
//! ```
//!
//! The engine thread blocks on input events (stdin channel). When an event
//! arrives OR TS writes wake flag, generation is incremented → reactive graph
//! propagates → layout → framebuffer → render. No polling. No loop. Pure
//! reactive propagation.

use std::cell::RefCell;
use std::io;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::sync::mpsc;

use spark_signals::{signal, derived, effect, Signal};

use crate::shared_buffer::{SharedBuffer, DIRTY_LAYOUT, DIRTY_TEXT, DIRTY_HIERARCHY};
use crate::layout;
use crate::framebuffer::{self, HitRegion};
use crate::renderer::{FrameBuffer, DiffRenderer};
use crate::input::parser::{InputParser, ParsedEvent};
use crate::input::focus::FocusManager;
use crate::input::keyboard;
use crate::input::mouse::MouseManager;
use crate::input::scroll::ScrollManager;
use crate::input::text_edit::TextEditor;
use crate::input::cursor::BlinkManager;
use crate::input::events::{EventRingBuffer, EventType};
use crate::input::reader::{StdinReader, StdinMessage};
use super::terminal::TerminalSetup;

// =============================================================================
// Types
// =============================================================================

/// Result of the framebuffer derived computation.
/// Must be Clone + PartialEq for spark-signals derived.
#[derive(Debug, Clone, PartialEq)]
struct FrameBufferResult {
    buffer: FrameBuffer,
    hit_regions: Vec<HitRegion>,
    terminal_size: (u16, u16),
}

// =============================================================================
// Engine
// =============================================================================

/// The SparkTUI engine.
///
/// Owns all Rust-side state and runs the reactive pipeline.
pub struct Engine {
    running: Arc<AtomicBool>,
}

impl Engine {
    /// Start the engine.
    ///
    /// Spawns the engine thread which:
    /// 1. Sets up terminal
    /// 2. Creates the reactive graph (generation → layout → framebuffer → render)
    /// 3. Blocks on stdin events — increments generation on input or wake
    ///
    /// Returns an Engine handle for shutdown.
    pub fn start(buf: &'static SharedBuffer) -> io::Result<Self> {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        thread::Builder::new()
            .name("spark-engine".to_string())
            .spawn(move || {
                if let Err(e) = run_engine(buf, running_clone) {
                    eprintln!("[spark-engine] Error: {}", e);
                }
            })?;

        Ok(Self { running })
    }

    /// Stop the engine gracefully.
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// Check if the engine is running.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.stop();
    }
}

// =============================================================================
// Reactive Pipeline
// =============================================================================


/// Main engine function. Runs on the engine thread.
fn run_engine(buf: &'static SharedBuffer, running: Arc<AtomicBool>) -> io::Result<()> {
    // 1. Setup terminal
    let mut terminal = TerminalSetup::new();
    terminal.enter_fullscreen()?;

    // 2. Start stdin reader
    let (stdin_reader, stdin_rx) = StdinReader::spawn()?;

    // 3. Initialize input system state
    let mut parser = InputParser::new();
    let mut focus = FocusManager::new();
    let mut events = EventRingBuffer::new();
    let mut editor = TextEditor::new();
    let mut scroll = ScrollManager::new();
    let mut blink = BlinkManager::new();
    let mouse_mgr = Rc::new(RefCell::new(MouseManager::new(buf.terminal_width(), buf.terminal_height())));

    // =========================================================================
    // 4. Create the reactive graph
    // =========================================================================

    // Root signal: generation counter.
    // Incremented when TS wakes Rust or when stdin input arrives.
    let generation: Signal<u64> = signal(0);

    // Layout derived: reads generation, checks dirty flags, runs Taffy if needed.
    let gen_for_layout = generation.clone();
    let layout_derived = derived(move || {
        // Read generation (creates reactive dependency)
        let _gen = gen_for_layout.get();

        // Smart skip: check dirty flags
        let node_count = buf.node_count();
        let mut needs_layout = false;
        for i in 0..node_count {
            let flags = buf.dirty_flags(i);
            if flags & (DIRTY_LAYOUT | DIRTY_TEXT | DIRTY_HIERARCHY) != 0 {
                needs_layout = true;
            }
            buf.clear_all_dirty(i);
        }

        // Layout computation (only if layout-affecting props changed)
        if needs_layout {
            layout::compute_layout_direct(buf);
        }

        // Return generation as the "result" — downstream deriveds
        // depend on this, so they re-run when generation changes
        _gen
    });

    // Framebuffer derived: depends on layout, builds 2D cell grid.
    let layout_d = layout_derived.clone();
    let fb_derived = derived(move || {
        // Read layout derived (creates reactive dependency)
        let _layout_gen = layout_d.get();

        // Read terminal size from SharedBuffer header
        let tw = buf.terminal_width();
        let th = buf.terminal_height();

        // Build framebuffer from SharedBuffer
        let (buffer, hit_regions) = framebuffer::compute_framebuffer(buf, tw, th);

        FrameBufferResult {
            buffer,
            hit_regions,
            terminal_size: (tw, th),
        }
    });

    // ONE render effect: fires when framebuffer derived changes.
    let running_for_effect = running.clone();
    let mouse_for_effect = mouse_mgr.clone();
    let mut renderer = DiffRenderer::new();
    let _stop_effect = effect(move || {
        if !running_for_effect.load(Ordering::SeqCst) {
            return;
        }

        // Read framebuffer (creates reactive dependency)
        let result = fb_derived.get();

        // Update hit grid (side effect)
        let (tw, th) = result.terminal_size;
        let mut mouse = mouse_for_effect.borrow_mut();
        mouse.hit_grid.resize(tw, th);
        for hr in &result.hit_regions {
            mouse.hit_grid.fill_rect(hr.x, hr.y, hr.width, hr.height, hr.component_index);
        }

        // Diff render to terminal (side effect)
        let _ = renderer.render(&result.buffer);
    });

    // =========================================================================
    // 5. Event-driven blocking: wait for input, increment generation
    // =========================================================================
    //
    // This is NOT a polling loop. It blocks on channel.recv() until
    // an event arrives. When an event arrives or TS wake flag is set,
    // we increment generation → reactive graph propagates automatically.

    while running.load(Ordering::SeqCst) {
        // Calculate timeout based on blink state
        let timeout = blink.next_deadline().unwrap_or(Duration::from_millis(100));

        // Block on stdin channel (NOT polling — OS-level event notification)
        match stdin_rx.recv_timeout(timeout) {
            Ok(StdinMessage::Data(data)) => {
                // Parse and dispatch input
                let parsed = parser.parse(&data);
                for event in parsed {
                    match event {
                        ParsedEvent::Key(key) => {
                            keyboard::dispatch_key(
                                buf, &mut events, &mut focus,
                                &mut editor, &mut scroll, &key,
                            );
                        }
                        ParsedEvent::Mouse(mouse) => {
                            mouse_mgr.borrow_mut().dispatch(
                                buf, &mut events, &mut focus,
                                &mut scroll, &mouse,
                            );
                        }
                        ParsedEvent::Resize(w, h) => {
                            mouse_mgr.borrow_mut().resize(w, h);
                        }
                        _ => {}
                    }
                }

                // Input changed state → increment generation → reactive propagation
                generation.set(generation.get() + 1);
            }
            Ok(StdinMessage::Closed) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // Check blink timers (cursor blink is a signal SOURCE)
                if blink.tick(buf) {
                    generation.set(generation.get() + 1);
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }

        // Check if TS wrote new props (wake flag)
        if buf.consume_wake() {
            generation.set(generation.get() + 1);
        }

        // Flush incomplete escape sequences after timeout
        if parser.has_pending() {
            let pending = parser.flush_pending();
            for event in pending {
                if let ParsedEvent::Key(key) = event {
                    keyboard::dispatch_key(
                        buf, &mut events, &mut focus,
                        &mut editor, &mut scroll, &key,
                    );
                }
            }
            generation.set(generation.get() + 1);
        }

        // Check for exit event
        if events.has_pending() {
            for ev in events.drain() {
                if ev.event_type == EventType::Exit {
                    running.store(false, Ordering::SeqCst);
                    break;
                }
                // TODO: Write events to SharedBuffer ring buffer section
                // for TS to read and dispatch callbacks
            }
        }
    }

    // Cleanup
    drop(stdin_reader);
    terminal.exit_fullscreen()?;

    Ok(())
}
