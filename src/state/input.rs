//! Input Module - Event conversion and polling
//!
//! Bridges crossterm's event system with our mouse and keyboard modules.
//! Provides event polling, conversion, and routing.
//!
//! # API
//!
//! - `convert_mouse_event` - Convert crossterm MouseEvent to our MouseEvent
//! - `convert_key_event` - Convert crossterm KeyEvent to our KeyboardEvent
//! - `poll_event` - Non-blocking event check with timeout
//! - `read_event` - Blocking event read
//! - `route_event` - Dispatch event to appropriate handler
//! - `enable_mouse` / `disable_mouse` - Control mouse capture
//!
//! # Example
//!
//! ```ignore
//! use spark_tui::state::input::{poll_event, route_event, InputEvent};
//! use std::time::Duration;
//!
//! // Event loop
//! loop {
//!     if let Ok(Some(event)) = poll_event(Duration::from_millis(16)) {
//!         route_event(event);
//!     }
//! }
//! ```

use crossterm::event::{
    Event as CrosstermEvent,
    KeyCode, KeyModifiers,
    KeyEvent as CrosstermKeyEvent,
    MouseButton as CrosstermMouseButton,
    MouseEvent as CrosstermMouseEvent,
    MouseEventKind,
    poll, read,
    EnableMouseCapture, DisableMouseCapture,
};
use crossterm::execute;
use std::io::stdout;
use std::time::Duration;

use super::keyboard::{KeyboardEvent, KeyState, Modifiers};
use super::mouse::{MouseEvent, MouseAction, MouseButton, ScrollDirection, ScrollInfo};

// =============================================================================
// INPUT EVENT ENUM
// =============================================================================

/// Unified event type for our framework
#[derive(Debug, Clone)]
pub enum InputEvent {
    /// Mouse event (click, scroll, move, etc.)
    Mouse(MouseEvent),
    /// Keyboard event (key press, release, etc.)
    Key(KeyboardEvent),
    /// Terminal resize event (new width, height)
    Resize(u16, u16),
    /// No event or unhandled event type
    None,
}

// =============================================================================
// MOUSE EVENT CONVERSION
// =============================================================================

/// Convert crossterm MouseEvent to our MouseEvent
pub fn convert_mouse_event(event: CrosstermMouseEvent) -> MouseEvent {
    let (action, button) = match event.kind {
        MouseEventKind::Down(btn) => (MouseAction::Down, convert_mouse_button(btn)),
        MouseEventKind::Up(btn) => (MouseAction::Up, convert_mouse_button(btn)),
        MouseEventKind::Drag(btn) => (MouseAction::Drag, convert_mouse_button(btn)),
        MouseEventKind::Moved => (MouseAction::Move, MouseButton::None),
        MouseEventKind::ScrollUp => (MouseAction::Scroll, MouseButton::None),
        MouseEventKind::ScrollDown => (MouseAction::Scroll, MouseButton::None),
        MouseEventKind::ScrollLeft => (MouseAction::Scroll, MouseButton::None),
        MouseEventKind::ScrollRight => (MouseAction::Scroll, MouseButton::None),
    };

    let scroll = match event.kind {
        MouseEventKind::ScrollUp => Some(ScrollInfo {
            direction: ScrollDirection::Up,
            delta: 1,
        }),
        MouseEventKind::ScrollDown => Some(ScrollInfo {
            direction: ScrollDirection::Down,
            delta: 1,
        }),
        MouseEventKind::ScrollLeft => Some(ScrollInfo {
            direction: ScrollDirection::Left,
            delta: 1,
        }),
        MouseEventKind::ScrollRight => Some(ScrollInfo {
            direction: ScrollDirection::Right,
            delta: 1,
        }),
        _ => None,
    };

    MouseEvent {
        action,
        button,
        x: event.column,
        y: event.row,
        modifiers: convert_modifiers(event.modifiers),
        scroll,
        component_index: None, // Filled by dispatch
    }
}

/// Convert crossterm MouseButton to our MouseButton
fn convert_mouse_button(btn: CrosstermMouseButton) -> MouseButton {
    match btn {
        CrosstermMouseButton::Left => MouseButton::Left,
        CrosstermMouseButton::Right => MouseButton::Right,
        CrosstermMouseButton::Middle => MouseButton::Middle,
    }
}

// =============================================================================
// KEY EVENT CONVERSION
// =============================================================================

/// Convert crossterm KeyEvent to our KeyboardEvent
pub fn convert_key_event(event: CrosstermKeyEvent) -> KeyboardEvent {
    let key = match event.code {
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Delete => "Delete".to_string(),
        KeyCode::Esc => "Escape".to_string(),
        KeyCode::Up => "ArrowUp".to_string(),
        KeyCode::Down => "ArrowDown".to_string(),
        KeyCode::Left => "ArrowLeft".to_string(),
        KeyCode::Right => "ArrowRight".to_string(),
        KeyCode::Home => "Home".to_string(),
        KeyCode::End => "End".to_string(),
        KeyCode::PageUp => "PageUp".to_string(),
        KeyCode::PageDown => "PageDown".to_string(),
        KeyCode::F(n) => format!("F{}", n),
        KeyCode::Insert => "Insert".to_string(),
        KeyCode::Null => String::new(),
        _ => String::new(),
    };

    let state = match event.kind {
        crossterm::event::KeyEventKind::Press => KeyState::Press,
        crossterm::event::KeyEventKind::Repeat => KeyState::Repeat,
        crossterm::event::KeyEventKind::Release => KeyState::Release,
    };

    KeyboardEvent {
        key,
        modifiers: convert_modifiers(event.modifiers),
        state,
        raw: None,
    }
}

// =============================================================================
// MODIFIER CONVERSION
// =============================================================================

/// Convert crossterm KeyModifiers to our Modifiers
fn convert_modifiers(mods: KeyModifiers) -> Modifiers {
    Modifiers {
        ctrl: mods.contains(KeyModifiers::CONTROL),
        alt: mods.contains(KeyModifiers::ALT),
        shift: mods.contains(KeyModifiers::SHIFT),
        meta: false, // Not exposed by crossterm
    }
}

// =============================================================================
// EVENT POLLING
// =============================================================================

/// Poll for an event with timeout.
/// Returns None if no event within timeout.
pub fn poll_event(timeout: Duration) -> std::io::Result<Option<InputEvent>> {
    if poll(timeout)? {
        Ok(Some(read_event()?))
    } else {
        Ok(None)
    }
}

/// Read the next event (blocking).
pub fn read_event() -> std::io::Result<InputEvent> {
    match read()? {
        CrosstermEvent::Mouse(mouse) => Ok(InputEvent::Mouse(convert_mouse_event(mouse))),
        CrosstermEvent::Key(key) => Ok(InputEvent::Key(convert_key_event(key))),
        CrosstermEvent::Resize(w, h) => Ok(InputEvent::Resize(w, h)),
        _ => Ok(InputEvent::None),
    }
}

// =============================================================================
// EVENT ROUTING
// =============================================================================

/// Route an event to the appropriate handler.
/// Returns true if any handler consumed the event.
pub fn route_event(event: InputEvent) -> bool {
    match event {
        InputEvent::Mouse(mouse) => super::mouse::dispatch(mouse),
        InputEvent::Key(key) => super::keyboard::dispatch(key),
        InputEvent::Resize(w, h) => {
            // Update terminal size signal
            crate::pipeline::terminal::set_terminal_size(w, h);
            false
        }
        InputEvent::None => false,
    }
}

// =============================================================================
// MOUSE CAPTURE
// =============================================================================

/// Enable mouse capture.
pub fn enable_mouse() -> std::io::Result<()> {
    execute!(stdout(), EnableMouseCapture)
}

/// Disable mouse capture.
pub fn disable_mouse() -> std::io::Result<()> {
    execute!(stdout(), DisableMouseCapture)
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_mouse_down() {
        let crossterm_event = CrosstermMouseEvent {
            kind: MouseEventKind::Down(CrosstermMouseButton::Left),
            column: 10,
            row: 5,
            modifiers: KeyModifiers::empty(),
        };

        let event = convert_mouse_event(crossterm_event);

        assert_eq!(event.action, MouseAction::Down);
        assert_eq!(event.button, MouseButton::Left);
        assert_eq!(event.x, 10);
        assert_eq!(event.y, 5);
        assert!(!event.modifiers.ctrl);
        assert!(event.scroll.is_none());
    }

    #[test]
    fn test_convert_mouse_up() {
        let crossterm_event = CrosstermMouseEvent {
            kind: MouseEventKind::Up(CrosstermMouseButton::Right),
            column: 20,
            row: 15,
            modifiers: KeyModifiers::empty(),
        };

        let event = convert_mouse_event(crossterm_event);

        assert_eq!(event.action, MouseAction::Up);
        assert_eq!(event.button, MouseButton::Right);
        assert_eq!(event.x, 20);
        assert_eq!(event.y, 15);
    }

    #[test]
    fn test_convert_mouse_scroll() {
        let crossterm_event = CrosstermMouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: 0,
            row: 0,
            modifiers: KeyModifiers::empty(),
        };

        let event = convert_mouse_event(crossterm_event);

        assert_eq!(event.action, MouseAction::Scroll);
        assert!(event.scroll.is_some());
        let scroll = event.scroll.unwrap();
        assert_eq!(scroll.direction, ScrollDirection::Down);
        assert_eq!(scroll.delta, 1);
    }

    #[test]
    fn test_convert_mouse_scroll_directions() {
        // Test all scroll directions
        let directions = [
            (MouseEventKind::ScrollUp, ScrollDirection::Up),
            (MouseEventKind::ScrollDown, ScrollDirection::Down),
            (MouseEventKind::ScrollLeft, ScrollDirection::Left),
            (MouseEventKind::ScrollRight, ScrollDirection::Right),
        ];

        for (kind, expected_dir) in directions {
            let crossterm_event = CrosstermMouseEvent {
                kind,
                column: 0,
                row: 0,
                modifiers: KeyModifiers::empty(),
            };

            let event = convert_mouse_event(crossterm_event);
            assert_eq!(event.action, MouseAction::Scroll);
            let scroll = event.scroll.unwrap();
            assert_eq!(scroll.direction, expected_dir);
        }
    }

    #[test]
    fn test_convert_mouse_with_modifiers() {
        let crossterm_event = CrosstermMouseEvent {
            kind: MouseEventKind::Down(CrosstermMouseButton::Left),
            column: 0,
            row: 0,
            modifiers: KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        };

        let event = convert_mouse_event(crossterm_event);

        assert!(event.modifiers.ctrl);
        assert!(event.modifiers.shift);
        assert!(!event.modifiers.alt);
        assert!(!event.modifiers.meta);
    }

    #[test]
    fn test_convert_mouse_drag() {
        let crossterm_event = CrosstermMouseEvent {
            kind: MouseEventKind::Drag(CrosstermMouseButton::Left),
            column: 5,
            row: 5,
            modifiers: KeyModifiers::empty(),
        };

        let event = convert_mouse_event(crossterm_event);

        assert_eq!(event.action, MouseAction::Drag);
        assert_eq!(event.button, MouseButton::Left);
    }

    #[test]
    fn test_convert_mouse_move() {
        let crossterm_event = CrosstermMouseEvent {
            kind: MouseEventKind::Moved,
            column: 30,
            row: 20,
            modifiers: KeyModifiers::empty(),
        };

        let event = convert_mouse_event(crossterm_event);

        assert_eq!(event.action, MouseAction::Move);
        assert_eq!(event.button, MouseButton::None);
        assert_eq!(event.x, 30);
        assert_eq!(event.y, 20);
    }

    #[test]
    fn test_convert_key_char() {
        let crossterm_event = CrosstermKeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::empty(),
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };

        let event = convert_key_event(crossterm_event);

        assert_eq!(event.key, "a");
        assert_eq!(event.state, KeyState::Press);
        assert!(!event.modifiers.ctrl);
    }

    #[test]
    fn test_convert_key_special() {
        let crossterm_event = CrosstermKeyEvent {
            code: KeyCode::Enter,
            modifiers: KeyModifiers::empty(),
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };

        let event = convert_key_event(crossterm_event);
        assert_eq!(event.key, "Enter");

        // Test arrow keys
        let crossterm_event = CrosstermKeyEvent {
            code: KeyCode::Up,
            modifiers: KeyModifiers::empty(),
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };

        let event = convert_key_event(crossterm_event);
        assert_eq!(event.key, "ArrowUp");
    }

    #[test]
    fn test_convert_key_all_arrows() {
        let arrows = [
            (KeyCode::Up, "ArrowUp"),
            (KeyCode::Down, "ArrowDown"),
            (KeyCode::Left, "ArrowLeft"),
            (KeyCode::Right, "ArrowRight"),
        ];

        for (code, expected) in arrows {
            let crossterm_event = CrosstermKeyEvent {
                code,
                modifiers: KeyModifiers::empty(),
                kind: crossterm::event::KeyEventKind::Press,
                state: crossterm::event::KeyEventState::NONE,
            };

            let event = convert_key_event(crossterm_event);
            assert_eq!(event.key, expected);
        }
    }

    #[test]
    fn test_convert_key_navigation() {
        let nav_keys = [
            (KeyCode::Home, "Home"),
            (KeyCode::End, "End"),
            (KeyCode::PageUp, "PageUp"),
            (KeyCode::PageDown, "PageDown"),
            (KeyCode::Insert, "Insert"),
            (KeyCode::Delete, "Delete"),
            (KeyCode::Backspace, "Backspace"),
            (KeyCode::Tab, "Tab"),
            (KeyCode::Esc, "Escape"),
        ];

        for (code, expected) in nav_keys {
            let crossterm_event = CrosstermKeyEvent {
                code,
                modifiers: KeyModifiers::empty(),
                kind: crossterm::event::KeyEventKind::Press,
                state: crossterm::event::KeyEventState::NONE,
            };

            let event = convert_key_event(crossterm_event);
            assert_eq!(event.key, expected);
        }
    }

    #[test]
    fn test_convert_key_function_keys() {
        for n in 1..=12 {
            let crossterm_event = CrosstermKeyEvent {
                code: KeyCode::F(n),
                modifiers: KeyModifiers::empty(),
                kind: crossterm::event::KeyEventKind::Press,
                state: crossterm::event::KeyEventState::NONE,
            };

            let event = convert_key_event(crossterm_event);
            assert_eq!(event.key, format!("F{}", n));
        }
    }

    #[test]
    fn test_convert_key_with_ctrl() {
        let crossterm_event = CrosstermKeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };

        let event = convert_key_event(crossterm_event);

        assert_eq!(event.key, "c");
        assert!(event.modifiers.ctrl);
        assert!(!event.modifiers.alt);
        assert!(!event.modifiers.shift);
    }

    #[test]
    fn test_convert_key_with_all_modifiers() {
        let crossterm_event = CrosstermKeyEvent {
            code: KeyCode::Char('x'),
            modifiers: KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SHIFT,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };

        let event = convert_key_event(crossterm_event);

        assert!(event.modifiers.ctrl);
        assert!(event.modifiers.alt);
        assert!(event.modifiers.shift);
        assert!(!event.modifiers.meta); // Not exposed by crossterm
    }

    #[test]
    fn test_convert_key_states() {
        let states = [
            (crossterm::event::KeyEventKind::Press, KeyState::Press),
            (crossterm::event::KeyEventKind::Repeat, KeyState::Repeat),
            (crossterm::event::KeyEventKind::Release, KeyState::Release),
        ];

        for (kind, expected) in states {
            let crossterm_event = CrosstermKeyEvent {
                code: KeyCode::Char('a'),
                modifiers: KeyModifiers::empty(),
                kind,
                state: crossterm::event::KeyEventState::NONE,
            };

            let event = convert_key_event(crossterm_event);
            assert_eq!(event.state, expected);
        }
    }

    #[test]
    fn test_all_mouse_buttons() {
        assert_eq!(
            convert_mouse_button(CrosstermMouseButton::Left),
            MouseButton::Left
        );
        assert_eq!(
            convert_mouse_button(CrosstermMouseButton::Right),
            MouseButton::Right
        );
        assert_eq!(
            convert_mouse_button(CrosstermMouseButton::Middle),
            MouseButton::Middle
        );
    }

    #[test]
    fn test_input_event_enum() {
        // Test that InputEvent can hold all event types
        let mouse_event = MouseEvent::down(MouseButton::Left, 0, 0);
        let key_event = KeyboardEvent::new("Enter");

        let _input_mouse = InputEvent::Mouse(mouse_event);
        let _input_key = InputEvent::Key(key_event);
        let _input_resize = InputEvent::Resize(120, 40);
        let _input_none = InputEvent::None;

        // Just verify they compile and can be matched
        match _input_none {
            InputEvent::Mouse(_) => {}
            InputEvent::Key(_) => {}
            InputEvent::Resize(_, _) => {}
            InputEvent::None => {}
        }
    }
}
