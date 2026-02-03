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
//! Three threads feed a unified mpsc channel:
//!
//! - **stdin reader**: blocks on stdin.read(), sends Data messages
//! - **wake watcher**: adaptive spin on SharedBuffer wake flag, sends Wake messages
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
use std::time::Instant;

use spark_signals::{signal, derived, effect, Signal};

use crate::shared_buffer::{SharedBuffer, RenderMode, DIRTY_LAYOUT, DIRTY_TEXT, DIRTY_HIERARCHY};
use crate::layout;
use crate::framebuffer::{self, HitRegion};
use crate::renderer::{FrameBuffer, DiffRenderer, InlineRenderer};
use crate::input::parser::{InputParser, ParsedEvent};
use crate::input::focus::FocusManager;
use crate::input::keyboard;
use crate::input::mouse::MouseManager;
use crate::input::scroll::ScrollManager;
use crate::input::text_edit::TextEditor;
use crate::input::reader::{StdinReader, StdinMessage, ResizeWatcher, get_terminal_size};
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
    // 1. Setup terminal based on render mode
    let render_mode = buf.render_mode();
    let mut terminal = TerminalSetup::new();
    let is_fullscreen = render_mode == RenderMode::Diff;

    if is_fullscreen {
        terminal.enter_fullscreen()?;
    } else {
        terminal.enter_inline()?;
    }

    // 2. Create unified channel — both stdin reader and wake watcher send here
    let (tx, rx) = mpsc::channel();

    // 3. Start stdin reader (sends Data/Closed messages)
    let stdin_reader = StdinReader::spawn(tx.clone())?;

    // 4. Start wake watcher (sends Wake messages when TS writes to SharedBuffer)
    let _wake_watcher = WakeWatcher::spawn(buf, tx.clone(), running.clone());

    // 5. Start resize watcher (sends Resize messages on SIGWINCH)
    let _resize_watcher = ResizeWatcher::spawn(tx, running.clone());

    // 6. Initialize input system state
    let mut parser = InputParser::new();
    let mut focus = FocusManager::new();
    let mut editor = TextEditor::new();
    let mut scroll = ScrollManager::new();

    // Get initial terminal size (prefer ioctl over SharedBuffer for accuracy)
    let (init_tw, init_th) = get_terminal_size()
        .unwrap_or((buf.terminal_width() as u16, buf.terminal_height() as u16));

    // Cursor blink is handled by TS pulse() signal - no Rust-side timer needed
    let mouse_mgr = Rc::new(RefCell::new(MouseManager::new(init_tw, init_th)));

    // =========================================================================
    // 7. Create the reactive graph
    // =========================================================================

    // Root signal: generation counter.
    // Incremented when TS wakes Rust or when stdin input arrives.
    let generation: Signal<u64> = signal(0);

    // Terminal size signals - updated on SIGWINCH.
    // Layout derived depends on these, so resize triggers re-layout automatically.
    let terminal_width: Signal<u16> = signal(init_tw);
    let terminal_height: Signal<u16> = signal(init_th);

    // Shared frame start timestamp for timing measurements
    let frame_start: Rc<RefCell<Option<Instant>>> = Rc::new(RefCell::new(None));
    let frame_start_for_layout = frame_start.clone();

    // Layout derived: reads generation + terminal size, checks dirty flags, runs Taffy if needed.
    let gen_for_layout = generation.clone();
    let tw_for_layout = terminal_width.clone();
    let th_for_layout = terminal_height.clone();
    let layout_derived = derived(move || {
        let layout_start = Instant::now();

        // Read generation (creates reactive dependency)
        let generation_value = gen_for_layout.get();

        // Read terminal size (creates reactive dependency - resize triggers re-layout)
        let tw = tw_for_layout.get();
        let th = th_for_layout.get();

        // Update SharedBuffer with current terminal size
        // This is where layout will read available space from
        buf.set_terminal_size(tw as u32, th as u32);

        // Check dirty flags for smart skip
        let node_count = buf.node_count();

        // Always run layout on first two renders (effect creation + initial set)
        // After that, only run if dirty flags are set
        let mut needs_layout = generation_value <= 1;

        for i in 0..node_count {
            let flags = buf.dirty_flags(i);
            if flags & (DIRTY_LAYOUT | DIRTY_TEXT | DIRTY_HIERARCHY) != 0 {
                needs_layout = true;
            }
            buf.clear_dirty(i);
        }

        // Layout computation
        if needs_layout && node_count > 0 {
            layout::compute_layout(buf);
        }

        // Record layout timing
        let layout_us = layout_start.elapsed().as_micros() as u32;
        buf.set_layout_time_us(layout_us);

        // Capture frame start time if not already set
        if frame_start_for_layout.borrow().is_none() {
            *frame_start_for_layout.borrow_mut() = Some(layout_start);
        }

        // Return generation as the "result" — downstream deriveds
        // depend on this, so they re-run when generation changes
        generation_value
    });

    // Framebuffer derived: depends on layout, builds 2D cell grid.
    let layout_d = layout_derived.clone();
    let fb_derived = derived(move || {
        let fb_start = Instant::now();

        // Read layout derived (creates reactive dependency)
        let _layout_gen = layout_d.get();

        // Framebuffer dimensions come from root element's computed layout.
        // Layout already accounts for render mode (fullscreen vs inline).
        let tw = buf.computed_width(0).max(1.0) as u16;
        let th = buf.computed_height(0).max(1.0) as u16;

        // Build framebuffer from SharedBuffer
        let (buffer, hit_regions) = framebuffer::compute_framebuffer(buf, tw, th);

        // Record framebuffer timing
        let fb_us = fb_start.elapsed().as_micros() as u32;
        buf.set_framebuffer_time_us(fb_us);

        FrameBufferResult {
            buffer,
            hit_regions,
            terminal_size: (tw, th),
        }
    });

    // ONE render effect: fires when framebuffer derived changes.
    let running_for_effect = running.clone();
    let mouse_for_effect = mouse_mgr.clone();
    let frame_start_for_effect = frame_start.clone();
    let mut diff_renderer = DiffRenderer::new();
    let mut inline_renderer = InlineRenderer::new();
    let _stop_effect = effect(move || {
        let render_start = Instant::now();

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

        // Render based on mode
        match buf.render_mode() {
            RenderMode::Inline => { let _ = inline_renderer.render(&result.buffer); }
            RenderMode::Append => { /* TODO: append_renderer */ }
            RenderMode::Diff => { let _ = diff_renderer.render(&result.buffer); }
        }

        // Record render timing
        let render_us = render_start.elapsed().as_micros() as u32;
        buf.set_render_time_us(render_us);

        // Record total frame time (from frame start to render complete)
        if let Some(start) = *frame_start_for_effect.borrow() {
            let total_us = start.elapsed().as_micros() as u32;
            buf.set_total_frame_time_us(total_us);
        }

        // Clear frame start for next frame
        *frame_start_for_effect.borrow_mut() = None;

        // Increment render counter so TS can track FPS
        buf.increment_render_count();
    });

    // Clone signals for event loop
    let tw_for_loop = terminal_width.clone();
    let th_for_loop = terminal_height.clone();

    // =========================================================================
    // 8. Initial render — trigger the reactive graph once
    // =========================================================================
    //
    // The effect won't run until generation changes. Trigger initial render
    // now that all the data is in the buffer.
    generation.set(1);

    // =========================================================================
    // 9. Event-driven blocking: wait for input or wake, increment generation
    // =========================================================================
    //
    // The engine thread blocks on channel.recv(). It wakes IMMEDIATELY when
    // either stdin data arrives OR the wake watcher detects TS wrote props.
    // No polling, no timers. Cursor blink is driven by TS pulse() signal.

    while running.load(Ordering::SeqCst) {
        // Block indefinitely until input or wake
        let msg = rx.recv();

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
                            // Escape-sequence based resize (some terminals)
                            mouse_mgr.borrow_mut().resize(w, h);
                            tw_for_loop.set(w);
                            th_for_loop.set(h);
                            // Push resize event to TS
                            buf.push_resize_event(w, h);
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
            Ok(StdinMessage::Resize(w, h)) => {
                // SIGWINCH detected by ResizeWatcher
                // Update terminal size signals → triggers layout_derived → re-layout
                mouse_mgr.borrow_mut().resize(w, h);
                tw_for_loop.set(w);
                th_for_loop.set(h);
                // Push resize event to TS (optional - user callback)
                buf.push_resize_event(w, h);
                // Signal change auto-triggers reactive graph, but increment generation too
                generation.set(generation.get() + 1);
            }
            Ok(StdinMessage::Wake) => {
                // Capture frame start for timing measurement
                *frame_start.borrow_mut() = Some(Instant::now());

                // TS wrote props to SharedBuffer → increment generation → reactive propagation
                generation.set(generation.get() + 1);
            }
            Ok(StdinMessage::Closed) => break,
            Err(_) => break, // Channel disconnected
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
    if is_fullscreen {
        terminal.exit_fullscreen()?;
    } else {
        terminal.exit_inline()?;
    }

    Ok(())
}
