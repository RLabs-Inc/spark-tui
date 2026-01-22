//! Mount API - Application lifecycle and render effect.
//!
//! This module provides the entry point for mounting a TUI application.
//! It sets up the render effect that monitors the reactive pipeline and
//! outputs to the terminal.

use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use spark_signals::effect;

use crate::renderer::{DiffRenderer, InlineRenderer, AppendRenderer};
use crate::state::mouse;
use super::layout_derived::create_layout_derived;
use super::frame_buffer_derived::create_frame_buffer_derived;
use super::terminal::{RenderMode, render_mode, detect_terminal_size};

// =============================================================================
// Mount Handle
// =============================================================================

/// Handle returned by mount() that allows unmounting.
pub struct MountHandle {
    stop_effect: Option<Box<dyn FnOnce()>>,
    running: Arc<AtomicBool>,
}

impl MountHandle {
    /// Stop the render effect and clean up.
    pub fn unmount(mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(stop) = self.stop_effect.take() {
            stop();
        }
    }

    /// Check if still running.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

impl Drop for MountHandle {
    fn drop(&mut self) {
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
/// This sets up the reactive render pipeline:
/// 1. Detects terminal size
/// 2. Creates layoutDerived and frameBufferDerived
/// 3. Starts the render effect
/// 4. Enters the appropriate screen mode
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

    Ok(MountHandle {
        stop_effect: Some(stop),
        running,
    })
}

/// Unmount and clean up.
pub fn unmount(handle: MountHandle) {
    handle.unmount();
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
}
