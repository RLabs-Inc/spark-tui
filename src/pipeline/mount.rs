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
use super::layout_derived::create_layout_derived;
use super::frame_buffer_derived::create_frame_buffer_derived;
use super::terminal::{RenderMode, render_mode, detect_terminal_size};

// =============================================================================
// HitGrid - Mouse interaction detection
// =============================================================================

/// A grid for O(1) mouse hit detection.
///
/// Each cell contains the component index that occupies that position,
/// or usize::MAX if empty.
pub struct HitGrid {
    width: u16,
    height: u16,
    cells: Vec<usize>,
}

impl HitGrid {
    /// Create a new hit grid.
    pub fn new(width: u16, height: u16) -> Self {
        let size = width as usize * height as usize;
        Self {
            width,
            height,
            cells: vec![usize::MAX; size],
        }
    }

    /// Resize the grid, clearing all contents.
    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        let size = width as usize * height as usize;
        self.cells.resize(size, usize::MAX);
        self.clear();
    }

    /// Clear all cells.
    pub fn clear(&mut self) {
        self.cells.fill(usize::MAX);
    }

    /// Fill a rectangle with a component index.
    pub fn fill_rect(&mut self, x: u16, y: u16, width: u16, height: u16, index: usize) {
        for dy in 0..height {
            let cy = y + dy;
            if cy >= self.height {
                break;
            }
            for dx in 0..width {
                let cx = x + dx;
                if cx >= self.width {
                    break;
                }
                let idx = cy as usize * self.width as usize + cx as usize;
                if idx < self.cells.len() {
                    self.cells[idx] = index;
                }
            }
        }
    }

    /// Get the component index at a position.
    pub fn get(&self, x: u16, y: u16) -> Option<usize> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = y as usize * self.width as usize + x as usize;
        let value = self.cells.get(idx).copied().unwrap_or(usize::MAX);
        if value == usize::MAX {
            None
        } else {
            Some(value)
        }
    }
}

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

    // Create hit grid
    let mut hit_grid = {
        let (tw, th) = {
            let result = fb_derived.get();
            result.terminal_size
        };
        HitGrid::new(tw, th)
    };

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

                // Resize hit grid if needed
                let (tw, th) = result.terminal_size;
                if hit_grid.width != tw || hit_grid.height != th {
                    hit_grid.resize(tw, th);
                } else {
                    hit_grid.clear();
                }

                // Apply hit regions (side effect!)
                for region in &result.hit_regions {
                    hit_grid.fill_rect(
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

                // Update hit grid
                let (tw, th) = result.terminal_size;
                if hit_grid.width != tw || hit_grid.height != th {
                    hit_grid.resize(tw, th);
                } else {
                    hit_grid.clear();
                }

                for region in &result.hit_regions {
                    hit_grid.fill_rect(
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

                // Update hit grid
                let (tw, th) = result.terminal_size;
                if hit_grid.width != tw || hit_grid.height != th {
                    hit_grid.resize(tw, th);
                } else {
                    hit_grid.clear();
                }

                for region in &result.hit_regions {
                    hit_grid.fill_rect(
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
    use super::*;

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
