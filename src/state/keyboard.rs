//! Keyboard Module - Keyboard event state and handler registry
//!
//! State and handler registry for keyboard events.
//! Does NOT own stdin (that will be the input module).
//! Does NOT handle global shortcuts (that will be global-keys module).
//!
//! # API
//!
//! - `last_event` - Get last keyboard event
//! - `last_key` - Get last key pressed
//! - `on(handler)` - Subscribe to all keyboard events
//! - `on_key(key, fn)` - Subscribe to specific key(s)
//! - `on_focused(i, fn)` - Subscribe when component i has focus
//!
//! # Example
//!
//! ```ignore
//! use spark_tui::state::keyboard;
//!
//! // Subscribe to all keyboard events
//! let cleanup = keyboard::on(|event| {
//!     println!("Key: {}", event.key);
//!     false // Don't consume
//! });
//!
//! // Subscribe to specific key
//! let cleanup = keyboard::on_key("Enter", || {
//!     println!("Enter pressed!");
//!     true // Consume event
//! });
//!
//! // Subscribe to events when component has focus
//! let cleanup = keyboard::on_focused(component_index, |event| {
//!     println!("Focused component got: {}", event.key);
//!     false
//! });
//! ```

use std::cell::RefCell;
use std::collections::HashMap;
use spark_signals::{signal, Signal};

// =============================================================================
// TYPES
// =============================================================================

/// Keyboard modifier state
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

impl Modifiers {
    /// Create empty modifiers
    pub fn none() -> Self {
        Self::default()
    }

    /// Create modifiers with ctrl
    pub fn ctrl() -> Self {
        Self { ctrl: true, ..Self::default() }
    }

    /// Create modifiers with alt
    pub fn alt() -> Self {
        Self { alt: true, ..Self::default() }
    }

    /// Create modifiers with shift
    pub fn shift() -> Self {
        Self { shift: true, ..Self::default() }
    }
}

/// Key event state (press, repeat, release)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyState {
    Press,
    Repeat,
    Release,
}

impl Default for KeyState {
    fn default() -> Self {
        Self::Press
    }
}

/// Keyboard event
#[derive(Clone, Debug, PartialEq)]
pub struct KeyboardEvent {
    /// The key that was pressed (e.g., "a", "Enter", "ArrowUp")
    pub key: String,
    /// Modifier keys state
    pub modifiers: Modifiers,
    /// Press/repeat/release state
    pub state: KeyState,
    /// Raw escape sequence (if available)
    pub raw: Option<String>,
}

impl KeyboardEvent {
    /// Create a simple key press event
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            modifiers: Modifiers::default(),
            state: KeyState::Press,
            raw: None,
        }
    }

    /// Create a key press with modifiers
    pub fn with_modifiers(key: impl Into<String>, modifiers: Modifiers) -> Self {
        Self {
            key: key.into(),
            modifiers,
            state: KeyState::Press,
            raw: None,
        }
    }

    /// Check if this is a press event
    pub fn is_press(&self) -> bool {
        self.state == KeyState::Press
    }
}

/// Handler for keyboard events. Return true to consume the event.
pub type KeyHandler = Box<dyn Fn(&KeyboardEvent) -> bool>;

/// Handler for specific key. Return true to consume the event.
pub type KeySpecificHandler = Box<dyn Fn() -> bool>;

// =============================================================================
// STATE
// =============================================================================

thread_local! {
    static LAST_EVENT: Signal<Option<KeyboardEvent>> = signal(None);
}

/// Get the last keyboard event
pub fn last_event() -> Option<KeyboardEvent> {
    LAST_EVENT.with(|s| s.get())
}

/// Get the last key pressed
pub fn last_key() -> String {
    last_event().map(|e| e.key).unwrap_or_default()
}

// =============================================================================
// HANDLER REGISTRY
// =============================================================================

// We need interior mutability for the handler sets
// Using raw pointers to identify handlers for removal

struct HandlerRegistry {
    global_handlers: Vec<(usize, KeyHandler)>,
    key_handlers: HashMap<String, Vec<(usize, KeySpecificHandler)>>,
    focused_handlers: HashMap<usize, Vec<(usize, KeyHandler)>>,
    next_id: usize,
}

impl HandlerRegistry {
    fn new() -> Self {
        Self {
            global_handlers: Vec::new(),
            key_handlers: HashMap::new(),
            focused_handlers: HashMap::new(),
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
// EVENT DISPATCH
// =============================================================================

/// Update the last event state without dispatching to handlers.
///
/// This is used by the central router (global_keys::route_keyboard_event)
/// to update reactive state before routing through the priority chain.
pub fn update_last_event(event: KeyboardEvent) {
    LAST_EVENT.with(|s| s.set(Some(event)));
}

/// Dispatch a keyboard event to all registered handlers.
/// Returns true if any handler consumed the event.
///
/// **Note:** In the new architecture, this function is primarily used for testing.
/// Production code should use `global_keys::route_keyboard_event()` which
/// enforces the correct priority order.
pub fn dispatch(event: KeyboardEvent) -> bool {
    // Always update reactive state
    LAST_EVENT.with(|s| s.set(Some(event.clone())));

    // Only dispatch press events to handlers
    if event.state != KeyState::Press {
        return false;
    }

    dispatch_to_handlers(&event)
}

/// Dispatch to key-specific and global handlers only (not focused).
///
/// This is used by the central router after focused component handlers
/// have had their chance. Returns true if any handler consumed the event.
pub fn dispatch_to_handlers(event: &KeyboardEvent) -> bool {
    REGISTRY.with(|reg| {
        let reg = reg.borrow();

        // Dispatch to key-specific handlers
        if let Some(handlers) = reg.key_handlers.get(&event.key) {
            for (_, handler) in handlers {
                if handler() {
                    return true;
                }
            }
        }

        // Dispatch to global handlers
        for (_, handler) in &reg.global_handlers {
            if handler(event) {
                return true;
            }
        }

        false
    })
}

/// Dispatch to focused component handlers.
/// Returns true if consumed.
pub fn dispatch_focused(focused_index: i32, event: &KeyboardEvent) -> bool {
    if focused_index < 0 {
        return false;
    }
    if event.state != KeyState::Press {
        return false;
    }

    REGISTRY.with(|reg| {
        let reg = reg.borrow();
        if let Some(handlers) = reg.focused_handlers.get(&(focused_index as usize)) {
            for (_, handler) in handlers {
                if handler(event) {
                    return true;
                }
            }
        }
        false
    })
}

// =============================================================================
// PUBLIC API
// =============================================================================

/// Subscribe to all keyboard events.
/// Return true from handler to consume the event.
/// Returns cleanup function.
pub fn on<F>(handler: F) -> impl FnOnce()
where
    F: Fn(&KeyboardEvent) -> bool + 'static,
{
    let id = REGISTRY.with(|reg| {
        let mut reg = reg.borrow_mut();
        let id = reg.next_id();
        reg.global_handlers.push((id, Box::new(handler)));
        id
    });

    move || {
        REGISTRY.with(|reg| {
            let mut reg = reg.borrow_mut();
            reg.global_handlers.retain(|(handler_id, _)| *handler_id != id);
        });
    }
}

/// Subscribe to specific key(s).
/// Handler receives no arguments - check last_event if needed.
/// Return true to consume the event.
/// Returns cleanup function.
pub fn on_key<F>(key: &str, handler: F) -> impl FnOnce()
where
    F: Fn() -> bool + 'static,
{
    let key = key.to_string();
    let id = REGISTRY.with(|reg| {
        let mut reg = reg.borrow_mut();
        let id = reg.next_id();
        reg.key_handlers
            .entry(key.clone())
            .or_insert_with(Vec::new)
            .push((id, Box::new(handler)));
        id
    });

    let key_clone = key;
    move || {
        REGISTRY.with(|reg| {
            let mut reg = reg.borrow_mut();
            if let Some(handlers) = reg.key_handlers.get_mut(&key_clone) {
                handlers.retain(|(handler_id, _)| *handler_id != id);
                if handlers.is_empty() {
                    reg.key_handlers.remove(&key_clone);
                }
            }
        });
    }
}

/// Subscribe to multiple keys with the same handler.
/// Returns cleanup function.
pub fn on_keys<F>(keys: &[&str], handler: F) -> impl FnOnce()
where
    F: Fn() -> bool + Clone + 'static,
{
    let ids: Vec<(String, usize)> = REGISTRY.with(|reg| {
        let mut reg = reg.borrow_mut();
        keys.iter()
            .map(|key| {
                let id = reg.next_id();
                reg.key_handlers
                    .entry(key.to_string())
                    .or_insert_with(Vec::new)
                    .push((id, Box::new(handler.clone())));
                (key.to_string(), id)
            })
            .collect()
    });

    move || {
        REGISTRY.with(|reg| {
            let mut reg = reg.borrow_mut();
            for (key, id) in &ids {
                if let Some(handlers) = reg.key_handlers.get_mut(key) {
                    handlers.retain(|(handler_id, _)| *handler_id != *id);
                    if handlers.is_empty() {
                        reg.key_handlers.remove(key);
                    }
                }
            }
        });
    }
}

/// Subscribe to events when a specific component has focus.
/// Return true from handler to consume the event.
/// Returns cleanup function.
pub fn on_focused<F>(index: usize, handler: F) -> impl FnOnce()
where
    F: Fn(&KeyboardEvent) -> bool + 'static,
{
    let id = REGISTRY.with(|reg| {
        let mut reg = reg.borrow_mut();
        let id = reg.next_id();
        reg.focused_handlers
            .entry(index)
            .or_insert_with(Vec::new)
            .push((id, Box::new(handler)));
        id
    });

    move || {
        REGISTRY.with(|reg| {
            let mut reg = reg.borrow_mut();
            if let Some(handlers) = reg.focused_handlers.get_mut(&index) {
                handlers.retain(|(handler_id, _)| *handler_id != id);
                if handlers.is_empty() {
                    reg.focused_handlers.remove(&index);
                }
            }
        });
    }
}

/// Clean up all handlers for a component index.
/// Called when component is released to prevent memory leaks.
pub fn cleanup_index(index: usize) {
    REGISTRY.with(|reg| {
        let mut reg = reg.borrow_mut();
        reg.focused_handlers.remove(&index);
    });
}

/// Clear all state and handlers.
pub fn cleanup() {
    REGISTRY.with(|reg| {
        let mut reg = reg.borrow_mut();
        reg.global_handlers.clear();
        reg.key_handlers.clear();
        reg.focused_handlers.clear();
    });
    LAST_EVENT.with(|s| s.set(None));
}

/// Reset keyboard state (for testing)
pub fn reset_keyboard_state() {
    cleanup();
    REGISTRY.with(|reg| {
        let mut reg = reg.borrow_mut();
        reg.next_id = 0;
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
        reset_keyboard_state();
    }

    #[test]
    fn test_initial_state() {
        setup();
        assert!(last_event().is_none());
        assert_eq!(last_key(), "");
    }

    #[test]
    fn test_dispatch_updates_state() {
        setup();

        dispatch(KeyboardEvent::new("a"));
        assert_eq!(last_key(), "a");

        dispatch(KeyboardEvent::new("Enter"));
        assert_eq!(last_key(), "Enter");
    }

    #[test]
    fn test_global_handler() {
        setup();

        let count = Rc::new(Cell::new(0));
        let count_clone = count.clone();

        let cleanup = on(move |_event| {
            count_clone.set(count_clone.get() + 1);
            false
        });

        dispatch(KeyboardEvent::new("a"));
        assert_eq!(count.get(), 1);

        dispatch(KeyboardEvent::new("b"));
        assert_eq!(count.get(), 2);

        cleanup();

        dispatch(KeyboardEvent::new("c"));
        assert_eq!(count.get(), 2); // No more increments
    }

    #[test]
    fn test_key_specific_handler() {
        setup();

        let enter_count = Rc::new(Cell::new(0));
        let enter_clone = enter_count.clone();

        let cleanup = on_key("Enter", move || {
            enter_clone.set(enter_clone.get() + 1);
            true
        });

        dispatch(KeyboardEvent::new("a"));
        assert_eq!(enter_count.get(), 0);

        dispatch(KeyboardEvent::new("Enter"));
        assert_eq!(enter_count.get(), 1);

        dispatch(KeyboardEvent::new("Enter"));
        assert_eq!(enter_count.get(), 2);

        cleanup();

        dispatch(KeyboardEvent::new("Enter"));
        assert_eq!(enter_count.get(), 2);
    }

    #[test]
    fn test_handler_consumption() {
        setup();

        let consumed = Rc::new(Cell::new(false));
        let consumed_clone = consumed.clone();

        // First handler consumes
        let _c1 = on_key("Enter", move || {
            consumed_clone.set(true);
            true // Consume
        });

        let reached = Rc::new(Cell::new(false));
        let reached_clone = reached.clone();

        // Second handler should not be called if first consumes
        let _c2 = on(move |_| {
            reached_clone.set(true);
            false
        });

        let result = dispatch(KeyboardEvent::new("Enter"));
        assert!(result); // Event was consumed
        assert!(consumed.get());
        assert!(!reached.get()); // Global handler not reached
    }

    #[test]
    fn test_focused_handler() {
        setup();

        let count = Rc::new(Cell::new(0));
        let count_clone = count.clone();

        let cleanup = on_focused(5, move |_event| {
            count_clone.set(count_clone.get() + 1);
            false
        });

        let event = KeyboardEvent::new("a");

        // Wrong index - not called
        dispatch_focused(3, &event);
        assert_eq!(count.get(), 0);

        // Correct index - called
        dispatch_focused(5, &event);
        assert_eq!(count.get(), 1);

        cleanup();

        dispatch_focused(5, &event);
        assert_eq!(count.get(), 1);
    }

    #[test]
    fn test_only_press_dispatched() {
        setup();

        let count = Rc::new(Cell::new(0));
        let count_clone = count.clone();

        let _cleanup = on(move |_| {
            count_clone.set(count_clone.get() + 1);
            false
        });

        // Press - dispatched
        dispatch(KeyboardEvent {
            key: "a".to_string(),
            modifiers: Modifiers::default(),
            state: KeyState::Press,
            raw: None,
        });
        assert_eq!(count.get(), 1);

        // Repeat - not dispatched to handlers
        dispatch(KeyboardEvent {
            key: "a".to_string(),
            modifiers: Modifiers::default(),
            state: KeyState::Repeat,
            raw: None,
        });
        assert_eq!(count.get(), 1);

        // Release - not dispatched to handlers
        dispatch(KeyboardEvent {
            key: "a".to_string(),
            modifiers: Modifiers::default(),
            state: KeyState::Release,
            raw: None,
        });
        assert_eq!(count.get(), 1);
    }

    #[test]
    fn test_modifiers() {
        setup();

        let ctrl_pressed = Rc::new(Cell::new(false));
        let ctrl_clone = ctrl_pressed.clone();

        let _cleanup = on(move |event| {
            if event.modifiers.ctrl && event.key == "c" {
                ctrl_clone.set(true);
            }
            false
        });

        dispatch(KeyboardEvent::with_modifiers("c", Modifiers::ctrl()));
        assert!(ctrl_pressed.get());
    }
}
