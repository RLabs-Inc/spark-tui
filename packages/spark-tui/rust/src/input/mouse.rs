//! Mouse dispatch and HitGrid for O(1) component lookup.
//!
//! Routes mouse events through:
//! - HitGrid: O(1) lookup from (x, y) -> component_index
//! - Hover tracking: enter/leave events
//! - Click detection: press + release on same component
//! - Scroll wheel: route to component under cursor

use crate::shared_buffer::{SharedBuffer, EventType};
use super::parser::{MouseEvent, MouseKind, MouseButton};
use super::focus::FocusManager;
use super::scroll::ScrollManager;

/// Push a mouse event to the SharedBuffer event ring.
fn push_mouse_event(buf: &SharedBuffer, event_type: EventType, component: u16, x: u16, y: u16, button: u8) {
    let mut data = [0u8; 16];
    data[0..2].copy_from_slice(&x.to_le_bytes());
    data[2..4].copy_from_slice(&y.to_le_bytes());
    data[4] = button;
    buf.push_event(event_type, component, &data);
}

/// Push a scroll event to the SharedBuffer event ring.
fn push_scroll_event(buf: &SharedBuffer, component: u16, dx: i32, dy: i32) {
    let mut data = [0u8; 16];
    data[0..4].copy_from_slice(&dx.to_le_bytes());
    data[4..8].copy_from_slice(&dy.to_le_bytes());
    buf.push_event(EventType::Scroll, component, &data);
}

// =============================================================================
// HitGrid
// =============================================================================

/// Flat grid mapping screen coordinates -> component index.
///
/// O(1) lookup: just index `grid[y * width + x]`.
/// -1 = no component at this position.
pub struct HitGrid {
    grid: Vec<i16>,
    width: u16,
    height: u16,
}

impl HitGrid {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            grid: vec![-1; width as usize * height as usize],
            width,
            height,
        }
    }

    /// Fill a rectangle in the grid with a component index.
    pub fn fill_rect(&mut self, x: u16, y: u16, w: u16, h: u16, component_index: usize) {
        let idx = component_index as i16;
        let x2 = (x + w).min(self.width);
        let y2 = (y + h).min(self.height);

        for row in y..y2 {
            let row_start = row as usize * self.width as usize;
            for col in x..x2 {
                self.grid[row_start + col as usize] = idx;
            }
        }
    }

    /// Look up the component at screen coordinates.
    pub fn hit_test(&self, x: u16, y: u16) -> Option<usize> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = self.grid[y as usize * self.width as usize + x as usize];
        if idx >= 0 { Some(idx as usize) } else { None }
    }

    /// Clear the grid.
    pub fn clear(&mut self) {
        for cell in &mut self.grid {
            *cell = -1;
        }
    }

    /// Resize the grid (clears content).
    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        self.grid.resize(width as usize * height as usize, -1);
        self.clear();
    }
}

// =============================================================================
// Mouse Manager
// =============================================================================

/// Manages mouse state: hover tracking, click detection.
pub struct MouseManager {
    /// Currently hovered component.
    hovered: Option<usize>,
    /// Component that was pressed (for click detection).
    pressed_component: Option<usize>,
    /// Button that was pressed.
    pressed_button: Option<MouseButton>,
    /// The hit grid.
    pub hit_grid: HitGrid,
}

impl MouseManager {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            hovered: None,
            pressed_component: None,
            pressed_button: None,
            hit_grid: HitGrid::new(width, height),
        }
    }

    /// Dispatch a mouse event.
    pub fn dispatch(
        &mut self,
        buf: &SharedBuffer,
        focus: &mut FocusManager,
        scroll: &mut ScrollManager,
        mouse: &MouseEvent,
    ) {
        let target = self.hit_grid.hit_test(mouse.x, mouse.y);

        match mouse.kind {
            MouseKind::Move => {
                self.handle_hover(buf, target);
            }
            MouseKind::Press(button) => {
                // Update hover first
                self.handle_hover(buf, target);

                if let Some(idx) = target {
                    self.pressed_component = Some(idx);
                    self.pressed_button = Some(button);

                    // Set pressed state in SharedBuffer
                    buf.set_pressed(idx, true);

                    // Write mouse down event
                    push_mouse_event(buf, EventType::MouseDown, idx as u16, mouse.x, mouse.y, button as u8);

                    // Focus on click
                    focus.focus_by_click(buf, idx);
                }
            }
            MouseKind::Release(button) => {
                if let Some(idx) = target {
                    // Write mouse up event
                    push_mouse_event(buf, EventType::MouseUp, idx as u16, mouse.x, mouse.y, button as u8);

                    // Click detection: same component pressed and released
                    if self.pressed_component == Some(idx)
                        && self.pressed_button == Some(button)
                    {
                        push_mouse_event(buf, EventType::Click, idx as u16, mouse.x, mouse.y, button as u8);
                    }
                }

                // Clear pressed state
                if let Some(prev) = self.pressed_component.take() {
                    buf.set_pressed(prev, false);
                }
                self.pressed_button = None;
            }
            MouseKind::ScrollUp => {
                // Route to component under cursor, or focused scrollable
                // Mouse scroll DOES chain to parent (natural UX)
                if let Some(idx) = target {
                    scroll.scroll_by(buf, idx, 0, -3, true);
                    push_scroll_event(buf, idx as u16, 0, -3);
                } else if let Some(focused) = focus.focused() {
                    scroll.scroll_by(buf, focused, 0, -3, true);
                    push_scroll_event(buf, focused as u16, 0, -3);
                }
            }
            MouseKind::ScrollDown => {
                // Mouse scroll DOES chain to parent (natural UX)
                if let Some(idx) = target {
                    scroll.scroll_by(buf, idx, 0, 3, true);
                    push_scroll_event(buf, idx as u16, 0, 3);
                } else if let Some(focused) = focus.focused() {
                    scroll.scroll_by(buf, focused, 0, 3, true);
                    push_scroll_event(buf, focused as u16, 0, 3);
                }
            }
        }
    }

    /// Handle hover state changes (enter/leave events).
    fn handle_hover(
        &mut self,
        buf: &SharedBuffer,
        target: Option<usize>,
    ) {
        if target == self.hovered {
            return;
        }

        // Leave previous
        if let Some(prev) = self.hovered.take() {
            buf.set_hovered(prev, false);
            push_mouse_event(buf, EventType::MouseLeave, prev as u16, 0, 0, 0);
        }

        // Enter new
        if let Some(idx) = target {
            buf.set_hovered(idx, true);
            push_mouse_event(buf, EventType::MouseEnter, idx as u16, 0, 0, 0);
            self.hovered = Some(idx);
        }
    }

    /// Resize the hit grid (e.g., on terminal resize).
    pub fn resize(&mut self, width: u16, height: u16) {
        self.hit_grid.resize(width, height);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hit_grid() {
        let mut grid = HitGrid::new(10, 10);
        assert_eq!(grid.hit_test(5, 5), None);

        grid.fill_rect(2, 2, 4, 4, 42);
        assert_eq!(grid.hit_test(3, 3), Some(42));
        assert_eq!(grid.hit_test(5, 5), Some(42));
        assert_eq!(grid.hit_test(0, 0), None);
        assert_eq!(grid.hit_test(8, 8), None);

        grid.clear();
        assert_eq!(grid.hit_test(3, 3), None);
    }

    #[test]
    fn test_hit_grid_resize() {
        let mut grid = HitGrid::new(10, 10);
        grid.fill_rect(0, 0, 5, 5, 1);
        assert_eq!(grid.hit_test(2, 2), Some(1));

        grid.resize(20, 20);
        assert_eq!(grid.hit_test(2, 2), None); // Cleared after resize
    }

    #[test]
    fn test_hit_grid_bounds() {
        let grid = HitGrid::new(10, 10);
        assert_eq!(grid.hit_test(10, 0), None);
        assert_eq!(grid.hit_test(0, 10), None);
        assert_eq!(grid.hit_test(100, 100), None);
    }
}
