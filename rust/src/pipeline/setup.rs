//! Engine setup — the main entry point for the Rust pipeline.
//!
//! Sets up the reactive graph using spark-signals:
//!
//! ```text
//! generation signal (incremented on TS wake or stdin input)
//!   │
//!   ├─→ layout_derived (spark-signals derived)
//!   │     reads generation → reads AoSBuffer dirty flags
//!   │     IF layout-dirty: run Taffy → write positions to output section
//!   │     IF visual-only-dirty: SKIP layout (smart skip)
//!   │
//!   ├─→ framebuffer_derived (spark-signals derived)
//!   │     depends on: layout_derived
//!   │     Read output + visual + text + interaction from AoSBuffer
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
//! Three threads feed a unified mpsc channel:
//!
//! - **stdin reader**: blocks on stdin.read(), sends Data messages
//! - **wake watcher**: adaptive spin on AoSBuffer wake flag, sends Wake messages
//! - **engine thread**: blocks on channel.recv(), processes both immediately
//!
//! No polling. No fixed timeout. Pure event-driven reactive propagation.

use std::cell::RefCell;
use std::io;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::sync::mpsc;

use spark_signals::{signal, derived, effect, Signal};

use crate::shared_buffer_aos::{AoSBuffer, DIRTY_LAYOUT, DIRTY_TEXT, DIRTY_HIERARCHY};
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
use crate::input::reader::{StdinReader, StdinMessage};
use super::terminal::TerminalSetup;
use super::wake::WakeWatcher;

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
    /// 2. Creates unified channel (stdin + wake → engine)
    /// 3. Creates the reactive graph (generation → layout → framebuffer → render)
    /// 4. Blocks on channel events — increments generation on input or wake
    ///
    /// Returns an Engine handle for shutdown.
    pub fn start(buf: &'static AoSBuffer) -> io::Result<Self> {
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
fn run_engine(buf: &'static AoSBuffer, running: Arc<AtomicBool>) -> io::Result<()> {
    // 1. Setup terminal
    let mut terminal = TerminalSetup::new();
    terminal.enter_fullscreen()?;

    // 2. Create unified channel — both stdin reader and wake watcher send here
    let (tx, rx) = mpsc::channel();

    // 3. Start stdin reader (sends Data/Closed messages)
    let stdin_reader = StdinReader::spawn(tx.clone())?;

    // 4. Start wake watcher (sends Wake messages when TS writes to AoSBuffer)
    let _wake_watcher = WakeWatcher::spawn(buf, tx, running.clone());

    // 5. Initialize input system state
    let mut parser = InputParser::new();
    let mut focus = FocusManager::new();
    let mut editor = TextEditor::new();
    let mut scroll = ScrollManager::new();
    let mut blink = BlinkManager::new();
    let mouse_mgr = Rc::new(RefCell::new(MouseManager::new(buf.terminal_width() as u16, buf.terminal_height() as u16)));

    // =========================================================================
    // 6. Create the reactive graph
    // =========================================================================

    // Root signal: generation counter.
    // Incremented when TS wakes Rust or when stdin input arrives.
    let generation: Signal<u64> = signal(0);

    // Layout derived: reads generation, checks dirty flags, runs Taffy if needed.
    let gen_for_layout = generation.clone();
    let layout_derived = derived(move || {
        // Read generation (creates reactive dependency)
        let _gen = gen_for_layout.get();

        // Check dirty flags for smart skip
        let node_count = buf.node_count();

        // Always run layout on first two renders (effect creation + initial set)
        // After that, only run if dirty flags are set
        let mut needs_layout = _gen <= 1;

        for i in 0..node_count {
            let flags = buf.dirty_flags(i);
            if flags & (DIRTY_LAYOUT | DIRTY_TEXT | DIRTY_HIERARCHY) != 0 {
                needs_layout = true;
            }
            buf.clear_all_dirty(i);
        }

        // Layout computation
        if needs_layout && node_count > 0 {
            layout::compute_layout_aos(buf);
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

        // Read terminal size from AoSBuffer header
        let tw = buf.terminal_width() as u16;
        let th = buf.terminal_height() as u16;

        // Build framebuffer from AoSBuffer
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

        // Increment render counter so TS can track FPS
        buf.increment_render_count();
    });

    // =========================================================================
    // 7. Initial render — trigger the reactive graph once
    // =========================================================================
    //
    // The effect won't run until generation changes. Trigger initial render
    // now that all the data is in the buffer.
    generation.set(1);

    // =========================================================================
    // 8. Event-driven blocking: wait for input or wake, increment generation
    // =========================================================================
    //
    // The engine thread blocks on channel.recv(). It wakes IMMEDIATELY when
    // either stdin data arrives OR the wake watcher detects TS wrote props.
    // The only timeout is for cursor blink timers — a legitimate time-based
    // signal source, not polling.

    while running.load(Ordering::SeqCst) {
        // Calculate timeout: only for blink timer, otherwise block indefinitely
        let msg = match blink.next_deadline() {
            Some(timeout) => rx.recv_timeout(timeout),
            None => rx.recv().map_err(|_| mpsc::RecvTimeoutError::Disconnected),
        };

        match msg {
            Ok(StdinMessage::Data(data)) => {
                // Parse and dispatch input
                let parsed = parser.parse(&data);
                for event in parsed {
                    match event {
                        ParsedEvent::Key(key) => {
                            keyboard::dispatch_key(
                                buf, &mut focus,
                                &mut editor, &mut scroll, &key,
                            );
                        }
                        ParsedEvent::Mouse(mouse) => {
                            mouse_mgr.borrow_mut().dispatch(
                                buf, &mut focus,
                                &mut scroll, &mouse,
                            );
                        }
                        ParsedEvent::Resize(w, h) => {
                            mouse_mgr.borrow_mut().resize(w, h);
                        }
                        _ => {}
                    }
                }

                // Check for exit event (Ctrl+C)
                if buf.exit_requested() {
                    running.store(false, Ordering::SeqCst);
                }

                // Input changed state → increment generation → reactive propagation
                generation.set(generation.get() + 1);
            }
            Ok(StdinMessage::Wake) => {
                // TS wrote props to AoSBuffer → increment generation → reactive propagation
                generation.set(generation.get() + 1);
            }
            Ok(StdinMessage::Closed) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // Only blink timer expired — cursor blink is a signal SOURCE
                if blink.tick(buf) {
                    generation.set(generation.get() + 1);
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }

        // Flush incomplete escape sequences after timeout
        if parser.has_pending() {
            let pending = parser.flush_pending();
            for event in pending {
                if let ParsedEvent::Key(key) = event {
                    keyboard::dispatch_key(
                        buf, &mut focus,
                        &mut editor, &mut scroll, &key,
                    );
                }
            }

            // Check for exit event after flush
            if buf.exit_requested() {
                running.store(false, Ordering::SeqCst);
            }

            generation.set(generation.get() + 1);
        }
    }

    // Cleanup
    drop(stdin_reader);
    terminal.exit_fullscreen()?;

    Ok(())
}
