//! Keyboard dispatch chain.
//!
//! Routes parsed key events through the dispatch chain:
//! 1. Ctrl+C → EXIT event
//! 2. Non-press events → ring buffer for TS
//! 3. Tab / Shift+Tab → focus navigation (consumed)
//! 4. Focused input → text editing (insert, delete, cursor move)
//! 5. Key event → ring buffer for TS onKey handlers
//! 6. Framework defaults (arrow scroll, page scroll, home/end)

use crate::shared_buffer::{SharedBuffer, EventType};
use super::parser::{KeyEvent, KeyCode, Modifier, KeyState};
use super::focus::FocusManager;
use super::text_edit::TextEditor;
use super::scroll::ScrollManager;

// Component type constants
const COMP_INPUT: u8 = 3;

/// Route a key event through the dispatch chain.
/// Returns true if the event was consumed.
pub fn dispatch_key(
    buf: &SharedBuffer,
    focus: &mut FocusManager,
    editor: &mut TextEditor,
    scroll: &mut ScrollManager,
    key: &KeyEvent,
) -> bool {
    // 1. Ctrl+C → EXIT
    if key.code == KeyCode::Char('c') && key.modifiers.contains(Modifier::CTRL) {
        buf.push_exit_event(0);
        return true;
    }

    // 2. Non-press events → send to TS for handling
    if key.state != KeyState::Press {
        let target = focus.focused().unwrap_or(0);
        push_key_event(buf, target as u16, &key.code, key.modifiers.bits());
        return false;
    }

    // 3. Tab / Shift+Tab → focus navigation
    if key.code == KeyCode::Tab {
        if key.modifiers.contains(Modifier::SHIFT) {
            focus.focus_previous(buf);
        } else {
            focus.focus_next(buf);
        }
        return true;
    }

    // 4. Focused input → text editing
    if let Some(focused) = focus.focused() {
        let comp_type = buf.component_type(focused);
        if comp_type == COMP_INPUT {
            if editor.handle_key(buf, focused, key) {
                return true;
            }
        }
    }

    // 5. Write key event to ring buffer (TS dispatches onKey)
    // Default to root (0) if nothing is focused
    let target = focus.focused().unwrap_or(0);
    push_key_event(buf, target as u16, &key.code, key.modifiers.bits());

    // 6. Framework defaults (arrow scroll, page scroll, home/end)
    // Keyboard scroll does NOT chain to parent (only mouse wheel chains)
    if let Some(focused) = focus.focused() {
        match &key.code {
            KeyCode::Up => {
                scroll.scroll_by(buf, focused, 0, -1, false);
                return true;
            }
            KeyCode::Down => {
                scroll.scroll_by(buf, focused, 0, 1, false);
                return true;
            }
            KeyCode::Left => {
                scroll.scroll_by(buf, focused, -1, 0, false);
                return true;
            }
            KeyCode::Right => {
                scroll.scroll_by(buf, focused, 1, 0, false);
                return true;
            }
            KeyCode::PageUp => {
                let viewport_h = buf.computed_height(focused) as i32;
                scroll.scroll_by(buf, focused, 0, -viewport_h, false);
                return true;
            }
            KeyCode::PageDown => {
                let viewport_h = buf.computed_height(focused) as i32;
                scroll.scroll_by(buf, focused, 0, viewport_h, false);
                return true;
            }
            KeyCode::Home => {
                scroll.scroll_to(buf, focused, 0, 0);
                return true;
            }
            KeyCode::End => {
                let max_y = buf.max_scroll_y(focused) as i32;
                scroll.scroll_to(buf, focused, 0, max_y);
                return true;
            }
            _ => {}
        }
    }

    false
}

/// Push a key event to the SharedBuffer event ring.
fn push_key_event(buf: &SharedBuffer, target: u16, code: &KeyCode, modifiers: u8) {
    let keycode = key_code_to_u32(code);
    let mut data = [0u8; 16];
    data[0..4].copy_from_slice(&keycode.to_le_bytes());
    data[4] = modifiers;
    buf.push_event(EventType::Key, target, &data);
}

/// Convert KeyCode to u32 for event data.
fn key_code_to_u32(code: &KeyCode) -> u32 {
    match code {
        KeyCode::Char(ch) => *ch as u32,
        KeyCode::Enter => 13,
        KeyCode::Tab => 9,
        KeyCode::Backspace => 8,
        KeyCode::Escape => 27,
        KeyCode::Delete => 127,
        KeyCode::Up => 0x1001,
        KeyCode::Down => 0x1002,
        KeyCode::Left => 0x1003,
        KeyCode::Right => 0x1004,
        KeyCode::Home => 0x1005,
        KeyCode::End => 0x1006,
        KeyCode::PageUp => 0x1007,
        KeyCode::PageDown => 0x1008,
        KeyCode::Insert => 0x1009,
        KeyCode::F(n) => 0x2000 + *n as u32,
        KeyCode::Null => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_code_to_u32() {
        assert_eq!(key_code_to_u32(&KeyCode::Char('a')), 97);
        assert_eq!(key_code_to_u32(&KeyCode::Enter), 13);
        assert_eq!(key_code_to_u32(&KeyCode::F(5)), 0x2005);
    }
}
