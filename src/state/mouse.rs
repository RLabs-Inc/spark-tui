//! Mouse Module - Mouse event state and handler registry
//!
//! HitGrid for coordinate-to-component lookup.
//! State and handler registry for mouse events.
//! Does NOT own stdin (that will be the input module).
//!
//! # API
//!
//! - `last_event` - Get last mouse event
//! - `mouse_x`, `mouse_y` - Get cursor position
//! - `is_mouse_down` - Get button state
//! - `hovered_component` - Get currently hovered component
//! - `dispatch(event)` - Dispatch mouse event
//! - `on_component(index, handlers)` - Per-component handlers
//! - `on_mouse_down(fn)` - Global mouse down handler
//! - `on_mouse_up(fn)` - Global mouse up handler
//! - `on_click(fn)` - Global click handler
//!
//! # Example
//!
//! ```ignore
//! use spark_tui::state::mouse;
//!
//! // Subscribe to clicks on a component
//! let cleanup = mouse::on_component(component_index, MouseHandlers {
//!     on_click: Some(Box::new(|event| {
//!         println!("Clicked at ({}, {})", event.x, event.y);
//!         true // Consume event
//!     })),
//!     ..Default::default()
//! });
//!
//! // Subscribe to all mouse clicks
//! let cleanup = mouse::on_click(|event| {
//!     println!("Click at ({}, {})", event.x, event.y);
//!     false // Don't consume
//! });
//! ```

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use spark_signals::{signal, Signal};

use super::keyboard::Modifiers;
use crate::engine::arrays::interaction;

// =============================================================================
// TYPES
// =============================================================================

/// Mouse action type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseAction {
    Down,
    Up,
    Move,
    Drag,
    Scroll,
}

/// Mouse button
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    None,
}

impl Default for MouseButton {
    fn default() -> Self {
        Self::None
    }
}

/// Scroll direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Scroll information
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollInfo {
    pub direction: ScrollDirection,
    pub delta: u16,
}

/// Mouse event
#[derive(Debug, Clone, PartialEq)]
pub struct MouseEvent {
    /// Action type (down, up, move, drag, scroll)
    pub action: MouseAction,
    /// Button pressed
    pub button: MouseButton,
    /// X coordinate (0-indexed)
    pub x: u16,
    /// Y coordinate (0-indexed)
    pub y: u16,
    /// Modifier keys state
    pub modifiers: Modifiers,
    /// Scroll info (only for scroll events)
    pub scroll: Option<ScrollInfo>,
    /// Component index at this position (filled by dispatch)
    pub component_index: Option<usize>,
}

impl MouseEvent {
    /// Create a new mouse event
    pub fn new(action: MouseAction, button: MouseButton, x: u16, y: u16) -> Self {
        Self {
            action,
            button,
            x,
            y,
            modifiers: Modifiers::default(),
            scroll: None,
            component_index: None,
        }
    }

    /// Create a scroll event
    pub fn scroll(x: u16, y: u16, direction: ScrollDirection, delta: u16) -> Self {
        Self {
            action: MouseAction::Scroll,
            button: MouseButton::None,
            x,
            y,
            modifiers: Modifiers::default(),
            scroll: Some(ScrollInfo { direction, delta }),
            component_index: None,
        }
    }

    /// Create a mouse down event
    pub fn down(button: MouseButton, x: u16, y: u16) -> Self {
        Self::new(MouseAction::Down, button, x, y)
    }

    /// Create a mouse up event
    pub fn up(button: MouseButton, x: u16, y: u16) -> Self {
        Self::new(MouseAction::Up, button, x, y)
    }

    /// Create a mouse move event
    pub fn move_to(x: u16, y: u16) -> Self {
        Self::new(MouseAction::Move, MouseButton::None, x, y)
    }
}

// =============================================================================
// HIT GRID - O(1) Coordinate to Component Lookup
// =============================================================================

/// A grid for O(1) mouse hit detection.
///
/// Each cell contains the component index that occupies that position,
/// or `None` if empty.
pub struct HitGrid {
    width: u16,
    height: u16,
    cells: Vec<usize>,
}

impl HitGrid {
    /// Create a new hit grid with the given dimensions.
    pub fn new(width: u16, height: u16) -> Self {
        let size = width as usize * height as usize;
        Self {
            width,
            height,
            cells: vec![usize::MAX; size],
        }
    }

    /// Get the grid width.
    pub fn width(&self) -> u16 {
        self.width
    }

    /// Get the grid height.
    pub fn height(&self) -> u16 {
        self.height
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

    /// Set a single cell to a component index.
    pub fn set(&mut self, x: u16, y: u16, index: usize) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = y as usize * self.width as usize + x as usize;
        if idx < self.cells.len() {
            self.cells[idx] = index;
        }
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
// GLOBAL HIT GRID
// =============================================================================

thread_local! {
    static HIT_GRID: RefCell<HitGrid> = RefCell::new(HitGrid::new(80, 24));
}

/// Resize the global hit grid.
pub fn resize_hit_grid(width: u16, height: u16) {
    HIT_GRID.with(|g| g.borrow_mut().resize(width, height));
}

/// Clear the global hit grid.
pub fn clear_hit_grid() {
    HIT_GRID.with(|g| g.borrow_mut().clear());
}

/// Fill a rectangle in the global hit grid.
pub fn fill_hit_rect(x: u16, y: u16, width: u16, height: u16, index: usize) {
    HIT_GRID.with(|g| g.borrow_mut().fill_rect(x, y, width, height, index));
}

/// Get the component at a position from the global hit grid.
pub fn hit_test(x: u16, y: u16) -> Option<usize> {
    HIT_GRID.with(|g| g.borrow().get(x, y))
}

/// Get the global hit grid dimensions.
pub fn hit_grid_size() -> (u16, u16) {
    HIT_GRID.with(|g| {
        let grid = g.borrow();
        (grid.width(), grid.height())
    })
}

// =============================================================================
// REACTIVE STATE
// =============================================================================

thread_local! {
    static LAST_EVENT: Signal<Option<MouseEvent>> = signal(None);
    static MOUSE_X: Signal<u16> = signal(0);
    static MOUSE_Y: Signal<u16> = signal(0);
    static IS_MOUSE_DOWN: Signal<bool> = signal(false);
    static HOVERED_COMPONENT: Signal<Option<usize>> = signal(None);
    static PRESSED_COMPONENT: Signal<Option<usize>> = signal(None);
    static PRESSED_BUTTON: Signal<MouseButton> = signal(MouseButton::None);
}

/// Get the last mouse event
pub fn last_event() -> Option<MouseEvent> {
    LAST_EVENT.with(|s| s.get())
}

/// Get current mouse X position
pub fn mouse_x() -> u16 {
    MOUSE_X.with(|s| s.get())
}

/// Get current mouse Y position
pub fn mouse_y() -> u16 {
    MOUSE_Y.with(|s| s.get())
}

/// Check if mouse button is currently down
pub fn is_mouse_down() -> bool {
    IS_MOUSE_DOWN.with(|s| s.get())
}

/// Get the currently hovered component index
pub fn hovered_component() -> Option<usize> {
    HOVERED_COMPONENT.with(|s| s.get())
}

/// Get the currently pressed component index
pub fn pressed_component() -> Option<usize> {
    PRESSED_COMPONENT.with(|s| s.get())
}

// =============================================================================
// HANDLER TYPES
// =============================================================================

/// Handler for mouse events. Return true to consume the event.
pub type MouseHandler = Box<dyn Fn(&MouseEvent) -> bool>;

/// Handler for enter/leave events (no return value).
pub type MouseEnterLeaveHandler = Box<dyn Fn(&MouseEvent)>;

/// Handlers for a component.
///
/// Uses Rc<dyn Fn> for handlers to allow cloning callbacks into closures
/// (e.g., wrapping user's on_click with click-to-focus behavior).
#[derive(Default)]
pub struct MouseHandlers {
    pub on_mouse_down: Option<Rc<dyn Fn(&MouseEvent)>>,
    pub on_mouse_up: Option<Rc<dyn Fn(&MouseEvent)>>,
    pub on_click: Option<Rc<dyn Fn(&MouseEvent)>>,
    pub on_mouse_enter: Option<Rc<dyn Fn(&MouseEvent)>>,
    pub on_mouse_leave: Option<Rc<dyn Fn(&MouseEvent)>>,
    pub on_scroll: Option<Rc<dyn Fn(&MouseEvent) -> bool>>,
}

// =============================================================================
// HANDLER REGISTRY
// =============================================================================

struct HandlerRegistry {
    component_handlers: HashMap<usize, MouseHandlers>,
    global_down_handlers: Vec<(usize, MouseHandler)>,
    global_up_handlers: Vec<(usize, MouseHandler)>,
    global_click_handlers: Vec<(usize, MouseHandler)>,
    global_scroll_handlers: Vec<(usize, MouseHandler)>,
    next_id: usize,
}

impl HandlerRegistry {
    fn new() -> Self {
        Self {
            component_handlers: HashMap::new(),
            global_down_handlers: Vec::new(),
            global_up_handlers: Vec::new(),
            global_click_handlers: Vec::new(),
            global_scroll_handlers: Vec::new(),
            next_id: 0,
        }
    }

    fn next_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

thread_local! {
    static REGISTRY: RefCell<HandlerRegistry> = RefCell::new(HandlerRegistry::new());
}

// =============================================================================
// PUBLIC API - REGISTRATION
// =============================================================================

/// Register handlers for a component. Returns cleanup function.
pub fn on_component(index: usize, handlers: MouseHandlers) -> impl FnOnce() {
    REGISTRY.with(|reg| {
        let mut reg = reg.borrow_mut();
        reg.component_handlers.insert(index, handlers);
    });

    move || {
        REGISTRY.with(|reg| {
            let mut reg = reg.borrow_mut();
            reg.component_handlers.remove(&index);
        });
    }
}

/// Register a global mouse down handler. Returns cleanup function.
pub fn on_mouse_down<F>(handler: F) -> impl FnOnce()
where
    F: Fn(&MouseEvent) -> bool + 'static,
{
    let id = REGISTRY.with(|reg| {
        let mut reg = reg.borrow_mut();
        let id = reg.next_id();
        reg.global_down_handlers.push((id, Box::new(handler)));
        id
    });

    move || {
        REGISTRY.with(|reg| {
            let mut reg = reg.borrow_mut();
            reg.global_down_handlers.retain(|(handler_id, _)| *handler_id != id);
        });
    }
}

/// Register a global mouse up handler. Returns cleanup function.
pub fn on_mouse_up<F>(handler: F) -> impl FnOnce()
where
    F: Fn(&MouseEvent) -> bool + 'static,
{
    let id = REGISTRY.with(|reg| {
        let mut reg = reg.borrow_mut();
        let id = reg.next_id();
        reg.global_up_handlers.push((id, Box::new(handler)));
        id
    });

    move || {
        REGISTRY.with(|reg| {
            let mut reg = reg.borrow_mut();
            reg.global_up_handlers.retain(|(handler_id, _)| *handler_id != id);
        });
    }
}

/// Register a global click handler. Returns cleanup function.
pub fn on_click<F>(handler: F) -> impl FnOnce()
where
    F: Fn(&MouseEvent) -> bool + 'static,
{
    let id = REGISTRY.with(|reg| {
        let mut reg = reg.borrow_mut();
        let id = reg.next_id();
        reg.global_click_handlers.push((id, Box::new(handler)));
        id
    });

    move || {
        REGISTRY.with(|reg| {
            let mut reg = reg.borrow_mut();
            reg.global_click_handlers.retain(|(handler_id, _)| *handler_id != id);
        });
    }
}

/// Register a global scroll handler. Returns cleanup function.
pub fn on_scroll<F>(handler: F) -> impl FnOnce()
where
    F: Fn(&MouseEvent) -> bool + 'static,
{
    let id = REGISTRY.with(|reg| {
        let mut reg = reg.borrow_mut();
        let id = reg.next_id();
        reg.global_scroll_handlers.push((id, Box::new(handler)));
        id
    });

    move || {
        REGISTRY.with(|reg| {
            let mut reg = reg.borrow_mut();
            reg.global_scroll_handlers.retain(|(handler_id, _)| *handler_id != id);
        });
    }
}

// =============================================================================
// DISPATCH
// =============================================================================

/// Dispatch a mouse event to all registered handlers.
/// Returns true if any handler consumed the event.
pub fn dispatch(mut event: MouseEvent) -> bool {
    // 1. Lookup component from HitGrid
    event.component_index = hit_test(event.x, event.y);

    // 2. Update reactive state
    LAST_EVENT.with(|s| s.set(Some(event.clone())));
    MOUSE_X.with(|s| s.set(event.x));
    MOUSE_Y.with(|s| s.set(event.y));

    // Update is_mouse_down based on action
    match event.action {
        MouseAction::Down => {
            IS_MOUSE_DOWN.with(|s| s.set(true));
        }
        MouseAction::Up => {
            IS_MOUSE_DOWN.with(|s| s.set(false));
        }
        _ => {}
    }

    let component_index = event.component_index;

    // 3. Handle hover (enter/leave)
    let prev_hovered = HOVERED_COMPONENT.with(|s| s.get());
    if component_index != prev_hovered {
        // Fire leave on previous
        if let Some(prev_idx) = prev_hovered {
            REGISTRY.with(|reg| {
                let reg = reg.borrow();
                if let Some(handlers) = reg.component_handlers.get(&prev_idx) {
                    if let Some(ref on_leave) = handlers.on_mouse_leave {
                        let mut leave_event = event.clone();
                        leave_event.component_index = Some(prev_idx);
                        on_leave(&leave_event);
                    }
                }
            });
            // Update interaction array
            interaction::set_hovered(prev_idx, false);
        }

        // Fire enter on new
        if let Some(idx) = component_index {
            REGISTRY.with(|reg| {
                let reg = reg.borrow();
                if let Some(handlers) = reg.component_handlers.get(&idx) {
                    if let Some(ref on_enter) = handlers.on_mouse_enter {
                        on_enter(&event);
                    }
                }
            });
            // Update interaction array
            interaction::set_hovered(idx, true);
        }

        HOVERED_COMPONENT.with(|s| s.set(component_index));
    }

    // 4. Handle specific actions
    match event.action {
        MouseAction::Scroll => dispatch_scroll(&event),
        MouseAction::Down => dispatch_down(&event),
        MouseAction::Up => dispatch_up(&event),
        _ => false,
    }
}

fn dispatch_scroll(event: &MouseEvent) -> bool {
    // Component handler first
    if let Some(idx) = event.component_index {
        let consumed = REGISTRY.with(|reg| {
            let reg = reg.borrow();
            if let Some(handlers) = reg.component_handlers.get(&idx) {
                if let Some(ref on_scroll) = handlers.on_scroll {
                    return on_scroll(event);
                }
            }
            false
        });
        if consumed {
            return true;
        }
    }

    // Global handlers
    REGISTRY.with(|reg| {
        let reg = reg.borrow();
        for (_, handler) in &reg.global_scroll_handlers {
            if handler(event) {
                return true;
            }
        }
        false
    })
}

fn dispatch_down(event: &MouseEvent) -> bool {
    // Track pressed component
    PRESSED_COMPONENT.with(|s| s.set(event.component_index));
    PRESSED_BUTTON.with(|s| s.set(event.button));

    // Update interaction array
    if let Some(idx) = event.component_index {
        interaction::set_pressed(idx, true);
    }

    // Component handler (non-consuming, just fires)
    if let Some(idx) = event.component_index {
        REGISTRY.with(|reg| {
            let reg = reg.borrow();
            if let Some(handlers) = reg.component_handlers.get(&idx) {
                if let Some(ref on_down) = handlers.on_mouse_down {
                    on_down(event);
                }
            }
        });
    }

    // Global handlers
    REGISTRY.with(|reg| {
        let reg = reg.borrow();
        for (_, handler) in &reg.global_down_handlers {
            if handler(event) {
                return true;
            }
        }
        false
    })
}

fn dispatch_up(event: &MouseEvent) -> bool {
    let pressed_idx = PRESSED_COMPONENT.with(|s| s.get());
    let pressed_btn = PRESSED_BUTTON.with(|s| s.get());

    // Clear pressed state in interaction array
    if let Some(idx) = pressed_idx {
        interaction::set_pressed(idx, false);
    }

    // Component handler (non-consuming, just fires)
    if let Some(idx) = event.component_index {
        REGISTRY.with(|reg| {
            let reg = reg.borrow();
            if let Some(handlers) = reg.component_handlers.get(&idx) {
                if let Some(ref on_up) = handlers.on_mouse_up {
                    on_up(event);
                }
            }
        });
    }

    // Global up handlers
    let mut consumed = REGISTRY.with(|reg| {
        let reg = reg.borrow();
        for (_, handler) in &reg.global_up_handlers {
            if handler(event) {
                return true;
            }
        }
        false
    });

    // Detect click (press and release on same component with same button)
    if pressed_idx == event.component_index && pressed_btn == event.button {
        // Component click handler with bubbling - walk up parent chain until handler found
        if let Some(idx) = event.component_index {
            let mut current = Some(idx);
            while let Some(component_idx) = current {
                let handler_found = REGISTRY.with(|reg| {
                    let reg = reg.borrow();
                    if let Some(handlers) = reg.component_handlers.get(&component_idx) {
                        if let Some(ref on_click) = handlers.on_click {
                            on_click(event);
                            return true;
                        }
                    }
                    false
                });

                if handler_found {
                    break; // Handler found and fired, stop bubbling
                }

                // Bubble up to parent
                current = crate::engine::arrays::core::get_parent_index(component_idx);
            }
        }

        // Global click handlers (can consume)
        let global_consumed = REGISTRY.with(|reg| {
            let reg = reg.borrow();
            for (_, handler) in &reg.global_click_handlers {
                if handler(event) {
                    return true;
                }
            }
            false
        });
        if global_consumed {
            consumed = true;
        }
    }

    // Clear pressed tracking
    PRESSED_COMPONENT.with(|s| s.set(None));
    PRESSED_BUTTON.with(|s| s.set(MouseButton::None));

    consumed
}

// =============================================================================
// CLEANUP
// =============================================================================

/// Clean up all handlers for a component index.
/// Called when component is released to prevent memory leaks.
pub fn cleanup_index(index: usize) {
    REGISTRY.with(|reg| {
        let mut reg = reg.borrow_mut();
        reg.component_handlers.remove(&index);
    });

    // Clear hover/pressed if this component was active
    let hovered = HOVERED_COMPONENT.with(|s| s.get());
    if hovered == Some(index) {
        HOVERED_COMPONENT.with(|s| s.set(None));
    }

    let pressed = PRESSED_COMPONENT.with(|s| s.get());
    if pressed == Some(index) {
        PRESSED_COMPONENT.with(|s| s.set(None));
        PRESSED_BUTTON.with(|s| s.set(MouseButton::None));
    }
}

/// Clear all state and handlers.
pub fn cleanup() {
    REGISTRY.with(|reg| {
        let mut reg = reg.borrow_mut();
        reg.component_handlers.clear();
        reg.global_down_handlers.clear();
        reg.global_up_handlers.clear();
        reg.global_click_handlers.clear();
        reg.global_scroll_handlers.clear();
    });

    LAST_EVENT.with(|s| s.set(None));
    MOUSE_X.with(|s| s.set(0));
    MOUSE_Y.with(|s| s.set(0));
    IS_MOUSE_DOWN.with(|s| s.set(false));
    HOVERED_COMPONENT.with(|s| s.set(None));
    PRESSED_COMPONENT.with(|s| s.set(None));
    PRESSED_BUTTON.with(|s| s.set(MouseButton::None));

    HIT_GRID.with(|g| g.borrow_mut().clear());
}

/// Reset mouse state (for testing)
pub fn reset_mouse_state() {
    cleanup();
    REGISTRY.with(|reg| {
        let mut reg = reg.borrow_mut();
        reg.next_id = 0;
    });
    HIT_GRID.with(|g| {
        let mut grid = g.borrow_mut();
        grid.resize(80, 24);
    });
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    fn setup() {
        reset_mouse_state();
    }

    // -------------------------------------------------------------------------
    // HitGrid Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_hit_grid_get_set() {
        let mut grid = HitGrid::new(10, 10);

        // Initially empty
        assert_eq!(grid.get(5, 5), None);

        // Set a cell
        grid.set(5, 5, 42);
        assert_eq!(grid.get(5, 5), Some(42));

        // Other cells still empty
        assert_eq!(grid.get(4, 5), None);
        assert_eq!(grid.get(5, 4), None);
    }

    #[test]
    fn test_hit_grid_fill_rect() {
        let mut grid = HitGrid::new(10, 10);

        // Fill a rectangle
        grid.fill_rect(2, 2, 4, 4, 42);

        // Inside
        assert_eq!(grid.get(2, 2), Some(42));
        assert_eq!(grid.get(3, 3), Some(42));
        assert_eq!(grid.get(5, 5), Some(42));

        // Outside
        assert_eq!(grid.get(0, 0), None);
        assert_eq!(grid.get(1, 2), None);
        assert_eq!(grid.get(6, 2), None);
        assert_eq!(grid.get(8, 8), None);
    }

    #[test]
    fn test_hit_grid_resize() {
        let mut grid = HitGrid::new(10, 10);
        grid.fill_rect(0, 0, 5, 5, 1);

        assert_eq!(grid.get(2, 2), Some(1));

        grid.resize(20, 20);
        // Should be cleared after resize
        assert_eq!(grid.get(2, 2), None);
        assert_eq!(grid.width(), 20);
        assert_eq!(grid.height(), 20);
    }

    #[test]
    fn test_hit_grid_out_of_bounds() {
        let mut grid = HitGrid::new(10, 10);

        // Out of bounds get returns None
        assert_eq!(grid.get(10, 5), None);
        assert_eq!(grid.get(5, 10), None);
        assert_eq!(grid.get(100, 100), None);

        // Out of bounds set is a no-op
        grid.set(100, 100, 42);
        assert_eq!(grid.get(100, 100), None);

        // Fill rect clips to bounds
        grid.fill_rect(8, 8, 10, 10, 42);
        assert_eq!(grid.get(8, 8), Some(42));
        assert_eq!(grid.get(9, 9), Some(42));
    }

    #[test]
    fn test_hit_grid_clear() {
        let mut grid = HitGrid::new(10, 10);
        grid.fill_rect(0, 0, 10, 10, 42);

        assert_eq!(grid.get(5, 5), Some(42));

        grid.clear();
        assert_eq!(grid.get(5, 5), None);
    }

    // -------------------------------------------------------------------------
    // Global HitGrid Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_global_hit_grid() {
        setup();

        fill_hit_rect(0, 0, 5, 5, 10);
        assert_eq!(hit_test(2, 2), Some(10));
        assert_eq!(hit_test(6, 6), None);

        clear_hit_grid();
        assert_eq!(hit_test(2, 2), None);
    }

    // -------------------------------------------------------------------------
    // Handler Registration Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_on_component_registration() {
        setup();

        let called = Rc::new(Cell::new(false));
        let called_clone = called.clone();

        let cleanup = on_component(5, MouseHandlers {
            on_click: Some(Rc::new(move |_| {
                called_clone.set(true);
            })),
            ..Default::default()
        });

        // Set up hit grid so component 5 is at (2,2)
        fill_hit_rect(0, 0, 5, 5, 5);

        // Mouse down then up on same component = click
        dispatch(MouseEvent::down(MouseButton::Left, 2, 2));
        dispatch(MouseEvent::up(MouseButton::Left, 2, 2));

        assert!(called.get());

        // Cleanup removes handler
        cleanup();
        called.set(false);

        dispatch(MouseEvent::down(MouseButton::Left, 2, 2));
        dispatch(MouseEvent::up(MouseButton::Left, 2, 2));

        assert!(!called.get());
    }

    #[test]
    fn test_handler_cleanup() {
        setup();

        let count = Rc::new(Cell::new(0));
        let count_clone = count.clone();

        let cleanup = on_click(move |_| {
            count_clone.set(count_clone.get() + 1);
            false
        });

        // Click anywhere (no component)
        dispatch(MouseEvent::down(MouseButton::Left, 0, 0));
        dispatch(MouseEvent::up(MouseButton::Left, 0, 0));
        assert_eq!(count.get(), 1);

        cleanup();

        dispatch(MouseEvent::down(MouseButton::Left, 0, 0));
        dispatch(MouseEvent::up(MouseButton::Left, 0, 0));
        assert_eq!(count.get(), 1); // No increment after cleanup
    }

    // -------------------------------------------------------------------------
    // Dispatch Tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_dispatch_updates_state() {
        setup();

        dispatch(MouseEvent::new(MouseAction::Move, MouseButton::None, 10, 20));

        assert_eq!(mouse_x(), 10);
        assert_eq!(mouse_y(), 20);
        assert!(!is_mouse_down());

        dispatch(MouseEvent::down(MouseButton::Left, 15, 25));

        assert_eq!(mouse_x(), 15);
        assert_eq!(mouse_y(), 25);
        assert!(is_mouse_down());

        dispatch(MouseEvent::up(MouseButton::Left, 15, 25));
        assert!(!is_mouse_down());
    }

    #[test]
    fn test_hover_detection() {
        setup();

        let enter_count = Rc::new(Cell::new(0));
        let leave_count = Rc::new(Cell::new(0));
        let enter_clone = enter_count.clone();
        let leave_clone = leave_count.clone();

        // Register handlers for component 5
        let _cleanup = on_component(5, MouseHandlers {
            on_mouse_enter: Some(Rc::new(move |_| {
                enter_clone.set(enter_clone.get() + 1);
            })),
            on_mouse_leave: Some(Rc::new(move |_| {
                leave_clone.set(leave_clone.get() + 1);
            })),
            ..Default::default()
        });

        // Set up hit grid
        fill_hit_rect(5, 5, 5, 5, 5);

        // Move outside component
        dispatch(MouseEvent::move_to(0, 0));
        assert_eq!(enter_count.get(), 0);
        assert_eq!(leave_count.get(), 0);

        // Move into component
        dispatch(MouseEvent::move_to(7, 7));
        assert_eq!(enter_count.get(), 1);
        assert_eq!(leave_count.get(), 0);

        // Move within component
        dispatch(MouseEvent::move_to(8, 8));
        assert_eq!(enter_count.get(), 1); // No additional enter
        assert_eq!(leave_count.get(), 0);

        // Move out of component
        dispatch(MouseEvent::move_to(0, 0));
        assert_eq!(enter_count.get(), 1);
        assert_eq!(leave_count.get(), 1);
    }

    #[test]
    fn test_click_detection() {
        setup();

        let click_count = Rc::new(Cell::new(0));
        let click_clone = click_count.clone();

        let _cleanup = on_component(5, MouseHandlers {
            on_click: Some(Rc::new(move |_| {
                click_clone.set(click_clone.get() + 1);
            })),
            ..Default::default()
        });

        fill_hit_rect(0, 0, 10, 10, 5);

        // Press and release on same component = click
        dispatch(MouseEvent::down(MouseButton::Left, 5, 5));
        dispatch(MouseEvent::up(MouseButton::Left, 5, 5));
        assert_eq!(click_count.get(), 1);

        // Press on component, release elsewhere = no click
        dispatch(MouseEvent::down(MouseButton::Left, 5, 5));
        clear_hit_grid(); // Simulate moving to empty area
        dispatch(MouseEvent::up(MouseButton::Left, 50, 50));
        assert_eq!(click_count.get(), 1); // No additional click

        // Different buttons = no click
        fill_hit_rect(0, 0, 10, 10, 5);
        dispatch(MouseEvent::down(MouseButton::Left, 5, 5));
        dispatch(MouseEvent::up(MouseButton::Right, 5, 5));
        assert_eq!(click_count.get(), 1); // No additional click
    }

    #[test]
    fn test_scroll_dispatch() {
        setup();

        let scroll_count = Rc::new(Cell::new(0));
        let scroll_clone = scroll_count.clone();

        let _cleanup = on_scroll(move |event| {
            if let Some(scroll) = &event.scroll {
                if scroll.direction == ScrollDirection::Up {
                    scroll_clone.set(scroll_clone.get() + 1);
                }
            }
            false
        });

        dispatch(MouseEvent::scroll(5, 5, ScrollDirection::Up, 3));
        assert_eq!(scroll_count.get(), 1);

        dispatch(MouseEvent::scroll(5, 5, ScrollDirection::Down, 3));
        assert_eq!(scroll_count.get(), 1); // Down doesn't increment

        dispatch(MouseEvent::scroll(5, 5, ScrollDirection::Up, 1));
        assert_eq!(scroll_count.get(), 2);
    }

    #[test]
    fn test_global_click_handler() {
        setup();

        let click_count = Rc::new(Cell::new(0));
        let click_clone = click_count.clone();

        let cleanup = on_click(move |_| {
            click_clone.set(click_clone.get() + 1);
            false
        });

        // Click on empty space
        dispatch(MouseEvent::down(MouseButton::Left, 0, 0));
        dispatch(MouseEvent::up(MouseButton::Left, 0, 0));
        assert_eq!(click_count.get(), 1);

        cleanup();

        dispatch(MouseEvent::down(MouseButton::Left, 0, 0));
        dispatch(MouseEvent::up(MouseButton::Left, 0, 0));
        assert_eq!(click_count.get(), 1);
    }

    #[test]
    fn test_cleanup_index() {
        setup();

        let called = Rc::new(Cell::new(false));
        let called_clone = called.clone();

        let _cleanup = on_component(5, MouseHandlers {
            on_click: Some(Rc::new(move |_| {
                called_clone.set(true);
            })),
            ..Default::default()
        });

        fill_hit_rect(0, 0, 10, 10, 5);

        // Click works before cleanup
        dispatch(MouseEvent::down(MouseButton::Left, 5, 5));
        dispatch(MouseEvent::up(MouseButton::Left, 5, 5));
        assert!(called.get());

        // Cleanup index
        cleanup_index(5);
        called.set(false);

        // Click no longer works
        dispatch(MouseEvent::down(MouseButton::Left, 5, 5));
        dispatch(MouseEvent::up(MouseButton::Left, 5, 5));
        assert!(!called.get());
    }
}
