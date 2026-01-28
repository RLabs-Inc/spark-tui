//! Event System - Truly reactive input handling
//!
//! Replaces polling with event-driven architecture matching TypeScript.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │  INPUT THREAD                                        │
//! │  crossterm::read() ─── BLOCKS ─── OS wakes on input │
//! │           │                                          │
//! │           ▼                                          │
//! │  tx.send(AppEvent::Input(ev))                       │
//! └─────────────────────────────────────────────────────┘
//!                       │
//!                       ▼ channel
//! ┌─────────────────────────────────────────────────────┐
//! │  BLINK THREAD (only if cursors active)              │
//! │  thread::sleep(interval) → tx.send(BlinkTick)       │
//! └─────────────────────────────────────────────────────┘
//!                       │
//!                       ▼ channel
//! ┌─────────────────────────────────────────────────────┐
//! │  MAIN THREAD                                         │
//! │  rx.recv() ─── BLOCKS ─── zero CPU when idle        │
//! │           │                                          │
//! │           ▼                                          │
//! │  route_event(ev) → signal.set() → effect runs       │
//! └─────────────────────────────────────────────────────┘
//! ```
//!
//! # Benefits
//!
//! - **Zero CPU when idle**: Main thread blocks on `rx.recv()`
//! - **Instant response**: OS wakes us immediately on stdin data
//! - **Faithful to TypeScript**: Both use OS-level blocking on stdin
//! - **No dependencies**: Uses only std library (no tokio)

use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossterm::event::{self, Event as CrosstermEvent};

// =============================================================================
// APP EVENT
// =============================================================================

/// Application events that can wake the main thread.
///
/// These are the only ways the main thread should be woken:
/// - Input from terminal (keyboard, mouse, resize)
/// - Blink timer tick (cursor animation)
/// - Stop signal (graceful shutdown)
#[derive(Debug)]
pub enum AppEvent {
    /// Terminal input event (keyboard, mouse, resize, etc.)
    Input(CrosstermEvent),

    /// Cursor blink tick - blink phase has changed.
    ///
    /// The blink signal is already updated by the timer thread.
    /// This event just wakes the main thread so effects can re-run.
    BlinkTick,

    /// Stop the event loop.
    ///
    /// Sent when:
    /// - `MountHandle::stop()` is called
    /// - Input thread encounters an error
    /// - Cleanup is requested
    Stop,
}

// =============================================================================
// EVENT CHANNEL
// =============================================================================

/// Bidirectional event channel for thread communication.
///
/// The sender can be cloned for multiple producers:
/// - Input thread sends `Input` events
/// - Blink timer sends `BlinkTick` events
/// - Stop can be sent from anywhere
pub struct EventChannel {
    /// Sender side - clone this for multiple producers
    pub tx: Sender<AppEvent>,
    /// Receiver side - only main thread should own this
    pub rx: Receiver<AppEvent>,
}

impl EventChannel {
    /// Create a new event channel.
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self { tx, rx }
    }

    /// Create a sender clone for another producer.
    pub fn sender(&self) -> Sender<AppEvent> {
        self.tx.clone()
    }

    /// Receive an event, blocking until one arrives.
    ///
    /// This is the core of the reactive event loop - the thread
    /// sleeps here with zero CPU until an event arrives.
    pub fn recv(&self) -> Option<AppEvent> {
        self.rx.recv().ok()
    }

    /// Try to receive an event without blocking.
    ///
    /// Returns `None` if no event is available.
    /// Use this for non-blocking event loops (games, animations).
    pub fn try_recv(&self) -> Option<AppEvent> {
        match self.rx.try_recv() {
            Ok(event) => Some(event),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => None,
        }
    }

    /// Receive with timeout.
    ///
    /// Returns `None` if no event within the timeout.
    pub fn recv_timeout(&self, timeout: Duration) -> Option<AppEvent> {
        self.rx.recv_timeout(timeout).ok()
    }
}

impl Default for EventChannel {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// INPUT THREAD
// =============================================================================

/// Dedicated thread for reading terminal input.
///
/// This thread uses `crossterm::event::poll()` which maps to OS-level
/// event notification (`epoll` on Linux, `kqueue` on macOS, `WaitForMultipleObjects`
/// on Windows). The thread truly sleeps in kernel space with **zero CPU usage**
/// until either:
/// - stdin has data → OS wakes thread immediately
/// - shutdown timeout expires → thread checks running flag
///
/// # Why a separate thread?
///
/// `crossterm::event::read()` is blocking. If we called it on the main
/// thread, we couldn't do anything else while waiting for input. By
/// putting it in a dedicated thread, the main thread can block on the
/// channel and still receive events from other sources (blink timer).
///
/// # Important: This is NOT busy-polling!
///
/// The poll syscall blocks in kernel space. The CPU is not used while waiting.
/// The shutdown timeout (default 100ms) exists only because Rust cannot
/// interrupt a blocking syscall from another thread. This is a cooperative
/// shutdown mechanism, not a polling loop.
///
/// When input arrives, the OS kernel immediately wakes the thread - there is
/// no latency from the timeout. The timeout only affects shutdown speed.
pub struct InputThread {
    handle: Option<JoinHandle<()>>,
    running: Arc<AtomicBool>,
}

/// Configuration for the input thread.
#[derive(Clone)]
pub struct InputThreadConfig {
    /// Timeout for shutdown detection.
    ///
    /// The thread polls with this timeout to allow checking the running flag.
    /// This does NOT add latency to input handling - input is processed
    /// immediately when available. The timeout only affects how quickly
    /// the thread can respond to shutdown requests.
    ///
    /// Default: 100ms (fast shutdown, negligible overhead: 10 wakeups/sec)
    /// Use longer values if you want fewer context switches at cost of slower shutdown.
    pub shutdown_timeout: Duration,
}

impl Default for InputThreadConfig {
    fn default() -> Self {
        Self {
            shutdown_timeout: Duration::from_millis(100),
        }
    }
}

impl InputThread {
    /// Spawn the input thread with default configuration.
    pub fn spawn(event_tx: Sender<AppEvent>) -> io::Result<Self> {
        Self::spawn_with_config(event_tx, InputThreadConfig::default())
    }

    /// Spawn the input thread with custom configuration.
    pub fn spawn_with_config(event_tx: Sender<AppEvent>, config: InputThreadConfig) -> io::Result<Self> {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        let handle = thread::Builder::new()
            .name("tui-input".to_string())
            .spawn(move || {
                Self::run_loop(running_clone, event_tx, config.shutdown_timeout);
            })?;

        Ok(Self {
            handle: Some(handle),
            running,
        })
    }

    /// The input reading loop.
    ///
    /// Uses OS-level event waiting (epoll/kqueue) with configurable timeout
    /// for shutdown detection. The thread sleeps in kernel space with zero
    /// CPU usage until input arrives or timeout expires.
    fn run_loop(running: Arc<AtomicBool>, tx: Sender<AppEvent>, shutdown_timeout: Duration) {
        while running.load(Ordering::SeqCst) {
            // poll() is an OS-level syscall that blocks in kernel space.
            // Zero CPU usage while waiting. Wakes immediately on input.
            // Timeout is only for shutdown detection.
            match event::poll(shutdown_timeout) {
                Ok(true) => {
                    // Input is ready - read immediately (no latency)
                    match event::read() {
                        Ok(ev) => {
                            if tx.send(AppEvent::Input(ev)).is_err() {
                                // Channel closed, main thread gone
                                break;
                            }
                        }
                        Err(e) => {
                            // Read error - might be transient (e.g., signal interrupt)
                            // Log but continue
                            #[cfg(debug_assertions)]
                            eprintln!("[tui-input] Read error: {}", e);
                            let _ = e; // Suppress unused warning in release
                        }
                    }
                }
                Ok(false) => {
                    // Timeout expired - no input. Loop to check running flag.
                    // This is NOT polling - we just woke from kernel sleep.
                    continue;
                }
                Err(e) => {
                    // Poll error - terminal might be disconnected
                    #[cfg(debug_assertions)]
                    eprintln!("[tui-input] Poll error: {}", e);
                    let _ = e;
                    let _ = tx.send(AppEvent::Stop);
                    break;
                }
            }
        }
    }

    /// Stop the input thread and wait for it to finish.
    ///
    /// Sets the running flag to false and waits for the thread.
    /// The thread will exit within the shutdown timeout.
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);

        if let Some(handle) = self.handle.take() {
            // Thread will exit within shutdown_timeout
            let _ = handle.join();
        }
    }

    /// Check if the input thread is still running.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst) && self.handle.is_some()
    }
}

impl Drop for InputThread {
    fn drop(&mut self) {
        self.stop();
    }
}

// =============================================================================
// CONVERT CROSSTERM TO INPUT EVENT
// =============================================================================

use crate::state::input::{convert_key_event, convert_mouse_event, InputEvent};
use crate::pipeline::terminal::set_terminal_size;

/// Convert a crossterm event to our InputEvent type.
///
/// Also handles side effects for resize events (updates terminal size signal).
pub fn convert_crossterm_event(event: CrosstermEvent) -> InputEvent {
    match event {
        CrosstermEvent::Key(key) => InputEvent::Key(convert_key_event(key)),
        CrosstermEvent::Mouse(mouse) => InputEvent::Mouse(convert_mouse_event(mouse)),
        CrosstermEvent::Resize(w, h) => {
            // Side effect: update terminal size signal
            // This triggers the reactive pipeline to recompute
            set_terminal_size(w, h);
            InputEvent::Resize(w, h)
        }
        CrosstermEvent::FocusGained => InputEvent::None,
        CrosstermEvent::FocusLost => InputEvent::None,
        CrosstermEvent::Paste(_) => InputEvent::None, // TODO: Handle paste
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_channel_creation() {
        let channel = EventChannel::new();

        // Should be able to send and receive
        channel.tx.send(AppEvent::Stop).unwrap();

        match channel.recv() {
            Some(AppEvent::Stop) => {} // Expected
            other => panic!("Expected Stop, got {:?}", other),
        }
    }

    #[test]
    fn test_event_channel_try_recv_empty() {
        let channel = EventChannel::new();

        // Should return None when empty
        assert!(channel.try_recv().is_none());
    }

    #[test]
    fn test_event_channel_multiple_senders() {
        let channel = EventChannel::new();
        let tx1 = channel.sender();
        let tx2 = channel.sender();

        tx1.send(AppEvent::BlinkTick).unwrap();
        tx2.send(AppEvent::Stop).unwrap();

        // Should receive both events
        assert!(matches!(channel.recv(), Some(AppEvent::BlinkTick)));
        assert!(matches!(channel.recv(), Some(AppEvent::Stop)));
    }

    #[test]
    fn test_event_channel_timeout() {
        let channel = EventChannel::new();

        // Should timeout quickly
        let result = channel.recv_timeout(Duration::from_millis(10));
        assert!(result.is_none());
    }

    // Note: InputThread tests would require mocking crossterm
    // which is complex. Integration tests should verify the full flow.
}
