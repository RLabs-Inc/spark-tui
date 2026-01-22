//! Mount API - Application lifecycle and render effect.
//!
//! This module provides the entry point for mounting a TUI application.
//! It sets up the render effect that monitors the reactive pipeline and
//! outputs to the terminal.
//!
//! # Example
//!
//! ```ignore
//! use spark_tui::pipeline::mount;
//!
//! // Mount the application
//! let handle = mount::mount()?;
//!
//! // Option 1: Run blocking event loop
//! mount::run(&handle)?;
//!
//! // Option 2: Tick manually in your own loop
//! while mount::tick(&handle)? {
//!     // Your logic here
//! }
//!
//! // Clean up
//! handle.unmount();
//! ```

use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use spark_signals::effect;

use crate::renderer::{DiffRenderer, InlineRenderer, AppendRenderer};
use crate::state::{mouse, input, global_keys};
use super::layout_derived::create_layout_derived;
use super::frame_buffer_derived::create_frame_buffer_derived;
use super::terminal::{RenderMode, render_mode, detect_terminal_size};

// =============================================================================
// Mount Handle
// =============================================================================

/// Handle returned by mount() that allows unmounting.
///
/// Holds references to:
/// - The render effect stop function
/// - The running flag (set to false on Ctrl+C or unmount)
/// - The global keys handle (for cleanup)
pub struct MountHandle {
    stop_effect: Option<Box<dyn FnOnce()>>,
    running: Arc<AtomicBool>,
    global_keys: Option<global_keys::GlobalKeysHandle>,
}

impl MountHandle {
    /// Stop the render effect and clean up.
    ///
    /// This will:
    /// 1. Set running to false
    /// 2. Clean up global key handlers
    /// 3. Disable mouse capture
    /// 4. Stop the render effect
    pub fn unmount(mut self) {
        self.running.store(false, Ordering::SeqCst);

        // Clean up global keys
        if let Some(handle) = self.global_keys.take() {
            handle.cleanup();
        }

        // Disable mouse capture
        let _ = input::disable_mouse();

        // Stop render effect
        if let Some(stop) = self.stop_effect.take() {
            stop();
        }
    }

    /// Check if still running.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Stop the application (sets running to false).
    /// Use this to trigger graceful shutdown from custom code.
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

impl Drop for MountHandle {
    fn drop(&mut self) {
        // Disable mouse on drop (best effort)
        let _ = input::disable_mouse();

        if let Some(stop) = self.stop_effect.take() {
            stop();
        }
    }
}

// =============================================================================
// Mount Function
// =============================================================================

/// Mount the TUI application.
///
/// This sets up:
/// 1. Terminal size detection
/// 2. Reactive render pipeline (layout -> frame buffer -> renderer)
/// 3. Mouse capture
/// 4. Global key handlers (Ctrl+C for shutdown, Tab/Shift+Tab for focus)
///
/// # Usage
///
/// After mounting, you have two options for running the event loop:
///
/// **Option 1: Blocking event loop**
/// ```ignore
/// let handle = mount()?;
/// run(&handle)?;  // Blocks until Ctrl+C or handle.stop()
/// handle.unmount();
/// ```
///
/// **Option 2: Manual ticking**
/// ```ignore
/// let handle = mount()?;
/// while tick(&handle)? {
///     // Your custom logic here
/// }
/// handle.unmount();
/// ```
///
/// Returns a MountHandle for cleanup.
pub fn mount() -> io::Result<MountHandle> {
    // Detect terminal size
    detect_terminal_size();

    // Create reactive pipeline
    let layout_derived = create_layout_derived();
    let fb_derived = create_frame_buffer_derived(layout_derived.clone());

    // Create renderer based on mode
    let mode = render_mode();

    let running = Arc::new(AtomicBool::new(true));
    let running_clone = running.clone();

    // Initialize the global hit grid with terminal size
    {
        let (tw, th) = {
            let result = fb_derived.get();
            result.terminal_size
        };
        mouse::resize_hit_grid(tw, th);
    }

    // Track current hit grid size for resize detection
    let mut last_hit_grid_size: (u16, u16) = mouse::hit_grid_size();

    // Create the ONE render effect
    // Each branch needs to be boxed because effect() returns different impl FnOnce() types
    let stop: Box<dyn FnOnce()> = match mode {
        RenderMode::Fullscreen => {
            let mut renderer = DiffRenderer::new();
            renderer.enter_fullscreen()?;

            let stop_fn = effect(move || {
                if !running_clone.load(Ordering::SeqCst) {
                    return;
                }

                // Read from derived (creates dependency)
                let result = fb_derived.get();

                // Resize/clear hit grid if needed
                let (tw, th) = result.terminal_size;
                if last_hit_grid_size != (tw, th) {
                    mouse::resize_hit_grid(tw, th);
                    last_hit_grid_size = (tw, th);
                } else {
                    mouse::clear_hit_grid();
                }

                // Apply hit regions (side effect!)
                for region in &result.hit_regions {
                    mouse::fill_hit_rect(
                        region.x,
                        region.y,
                        region.width,
                        region.height,
                        region.component_index,
                    );
                }

                // Render to terminal (side effect!)
                let _ = renderer.render(&result.buffer);
            });
            Box::new(stop_fn)
        }
        RenderMode::Inline => {
            let mut renderer = InlineRenderer::new();

            let stop_fn = effect(move || {
                if !running_clone.load(Ordering::SeqCst) {
                    return;
                }

                let result = fb_derived.get();

                // Resize/clear hit grid
                let (tw, th) = result.terminal_size;
                if last_hit_grid_size != (tw, th) {
                    mouse::resize_hit_grid(tw, th);
                    last_hit_grid_size = (tw, th);
                } else {
                    mouse::clear_hit_grid();
                }

                for region in &result.hit_regions {
                    mouse::fill_hit_rect(
                        region.x,
                        region.y,
                        region.width,
                        region.height,
                        region.component_index,
                    );
                }

                let _ = renderer.render(&result.buffer);
            });
            Box::new(stop_fn)
        }
        RenderMode::Append => {
            let mut renderer = AppendRenderer::new();

            let stop_fn = effect(move || {
                if !running_clone.load(Ordering::SeqCst) {
                    return;
                }

                let result = fb_derived.get();

                // Resize/clear hit grid
                let (tw, th) = result.terminal_size;
                if last_hit_grid_size != (tw, th) {
                    mouse::resize_hit_grid(tw, th);
                    last_hit_grid_size = (tw, th);
                } else {
                    mouse::clear_hit_grid();
                }

                for region in &result.hit_regions {
                    mouse::fill_hit_rect(
                        region.x,
                        region.y,
                        region.width,
                        region.height,
                        region.component_index,
                    );
                }

                let _ = renderer.render_active(&result.buffer);
            });
            Box::new(stop_fn)
        }
    };

    // Enable mouse capture
    input::enable_mouse()?;

    // Set up global key handlers (Ctrl+C, Tab, Shift+Tab)
    let global_keys_handle = global_keys::setup_global_keys(running.clone());

    Ok(MountHandle {
        stop_effect: Some(stop),
        running,
        global_keys: Some(global_keys_handle),
    })
}

/// Unmount and clean up.
pub fn unmount(handle: MountHandle) {
    handle.unmount();
}

// =============================================================================
// Event Loop
// =============================================================================

/// Run the event loop once (non-blocking).
///
/// Call this in your main loop to process input events.
/// Returns `Ok(false)` if the application should stop running.
///
/// # Arguments
///
/// * `handle` - The mount handle returned from `mount()`
///
/// # Returns
///
/// * `Ok(true)` - Continue running
/// * `Ok(false)` - Stop requested (Ctrl+C pressed or `handle.stop()` called)
/// * `Err(e)` - I/O error while polling
///
/// # Example
///
/// ```ignore
/// let handle = mount()?;
/// while tick(&handle)? {
///     // Process your application logic
/// }
/// handle.unmount();
/// ```
pub fn tick(handle: &MountHandle) -> io::Result<bool> {
    if !handle.is_running() {
        return Ok(false);
    }

    // Poll with short timeout (~60fps)
    if let Some(event) = input::poll_event(Duration::from_millis(16))? {
        input::route_event(event);
    }

    Ok(handle.is_running())
}

/// Run the event loop (blocking until stopped).
///
/// This function blocks until:
/// - Ctrl+C is pressed (sets running to false)
/// - `handle.stop()` is called from another thread/handler
///
/// # Arguments
///
/// * `handle` - The mount handle returned from `mount()`
///
/// # Example
///
/// ```ignore
/// let handle = mount()?;
/// run(&handle)?;  // Blocks here until Ctrl+C
/// handle.unmount();
/// ```
pub fn run(handle: &MountHandle) -> io::Result<()> {
    while tick(handle)? {
        // Continue processing events
    }
    Ok(())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use crate::state::mouse::HitGrid;

    #[test]
    fn test_hit_grid() {
        let mut grid = HitGrid::new(10, 10);

        // Initially empty
        assert_eq!(grid.get(5, 5), None);

        // Fill a rectangle
        grid.fill_rect(2, 2, 4, 4, 42);

        // Inside
        assert_eq!(grid.get(3, 3), Some(42));
        assert_eq!(grid.get(5, 5), Some(42));

        // Outside
        assert_eq!(grid.get(0, 0), None);
        assert_eq!(grid.get(8, 8), None);

        // Clear
        grid.clear();
        assert_eq!(grid.get(3, 3), None);
    }

    #[test]
    fn test_hit_grid_resize() {
        let mut grid = HitGrid::new(10, 10);
        grid.fill_rect(0, 0, 5, 5, 1);

        grid.resize(20, 20);
        // Should be cleared after resize
        assert_eq!(grid.get(2, 2), None);
    }

    #[test]
    fn test_running_flag() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};

        let running = Arc::new(AtomicBool::new(true));
        assert!(running.load(Ordering::SeqCst));

        running.store(false, Ordering::SeqCst);
        assert!(!running.load(Ordering::SeqCst));
    }
}
