//! Keyboard dispatch chain.
//!
//! Routes parsed key events through the dispatch chain:
//! 1. Ctrl+C → EXIT event
//! 2. Non-press events → ring buffer for TS
//! 3. Tab / Shift+Tab → focus navigation (consumed)
//! 4. Focused input → text editing (insert, delete, cursor move)
//! 5. Key event → ring buffer for TS onKey handlers
//! 6. Framework defaults (arrow scroll, page scroll, home/end)

use crate::shared_buffer_aos::AoSBuffer;
use super::parser::{KeyEvent, KeyCode, Modifier, KeyState};
use super::events::Event;
use super::focus::FocusManager;
use super::text_edit::TextEditor;
use super::scroll::ScrollManager;

// Component type constants
const COMP_INPUT: u8 = 3;

/// Route a key event through the dispatch chain.
/// Returns true if the event was consumed.
pub fn dispatch_key(
    buf: &AoSBuffer,
    focus: &mut FocusManager,
    editor: &mut TextEditor,
    scroll: &mut ScrollManager,
    key: &KeyEvent,
) -> bool {
    // 1. Ctrl+C → EXIT
    if key.code == KeyCode::Char('c') && key.modifiers.contains(Modifier::CTRL) {
        buf.push_event(&Event::exit());
        return true;
    }

    // 2. Non-press events → send to TS for handling
    if key.state != KeyState::Press {
        if let Some(focused) = focus.focused() {
            let keycode = key_code_to_u32(&key.code);
            buf.push_event(&Event::key(focused as u16, keycode, key.modifiers.bits()));
        }
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
    if let Some(focused) = focus.focused() {
        let keycode = key_code_to_u32(&key.code);
        buf.push_event(&Event::key(focused as u16, keycode, key.modifiers.bits()));
    }

    // 6. Framework defaults (arrow scroll, page scroll, home/end)
    if let Some(focused) = focus.focused() {
        match &key.code {
            KeyCode::Up => {
                scroll.scroll_by(buf, focused, 0, -1);
                return true;
            }
            KeyCode::Down => {
                scroll.scroll_by(buf, focused, 0, 1);
                return true;
            }
            KeyCode::Left => {
                scroll.scroll_by(buf, focused, -1, 0);
                return true;
            }
            KeyCode::Right => {
                scroll.scroll_by(buf, focused, 1, 0);
                return true;
            }
            KeyCode::PageUp => {
                let viewport_h = buf.output_height(focused) as i32;
                scroll.scroll_by(buf, focused, 0, -viewport_h);
                return true;
            }
            KeyCode::PageDown => {
                let viewport_h = buf.output_height(focused) as i32;
                scroll.scroll_by(buf, focused, 0, viewport_h);
                return true;
            }
            KeyCode::Home => {
                scroll.scroll_to(buf, focused, 0, 0);
                return true;
            }
            KeyCode::End => {
                let max_y = buf.output_max_scroll_y(focused) as i32;
                scroll.scroll_to(buf, focused, 0, max_y);
                return true;
            }
            _ => {}
        }
    }

    false
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
