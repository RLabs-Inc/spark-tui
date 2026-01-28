//! Input text editing (Rust-owned).
//!
//! Handles character insertion, deletion, cursor movement,
//! maxLength enforcement, and fires value change events.
//!
//! All text editing happens directly in SharedBuffer's text pool.

use crate::shared_buffer::SharedBuffer;
use super::parser::{KeyEvent, KeyCode, Modifier};
use super::events::{Event, EventRingBuffer};

/// Text editor for input components.
pub struct TextEditor;

impl TextEditor {
    pub fn new() -> Self {
        Self
    }

    /// Handle a key event for an input component.
    /// Returns true if the event was consumed.
    pub fn handle_key(
        &mut self,
        buf: &SharedBuffer,
        events: &mut EventRingBuffer,
        index: usize,
        key: &KeyEvent,
    ) -> bool {
        match &key.code {
            KeyCode::Char(ch) => {
                if key.modifiers.contains(Modifier::CTRL) || key.modifiers.contains(Modifier::ALT) {
                    return false; // Don't consume modified chars
                }
                self.insert_char(buf, events, index, *ch);
                true
            }
            KeyCode::Backspace => {
                self.delete_backward(buf, events, index);
                true
            }
            KeyCode::Delete => {
                self.delete_forward(buf, events, index);
                true
            }
            KeyCode::Left => {
                self.move_cursor(buf, index, -1);
                true
            }
            KeyCode::Right => {
                self.move_cursor(buf, index, 1);
                true
            }
            KeyCode::Home => {
                buf.set_cursor_position(index, 0);
                true
            }
            KeyCode::End => {
                let len = self.char_count(buf, index);
                buf.set_cursor_position(index, len as i32);
                true
            }
            KeyCode::Enter => {
                events.push(Event::submit(index as u16));
                true
            }
            KeyCode::Escape => {
                events.push(Event::cancel(index as u16));
                true
            }
            _ => false,
        }
    }

    /// Insert a character at the cursor position.
    fn insert_char(
        &self,
        buf: &SharedBuffer,
        events: &mut EventRingBuffer,
        index: usize,
        ch: char,
    ) {
        let content = buf.text_content(index).to_string();
        let chars: Vec<char> = content.chars().collect();
        let cursor = (buf.cursor_position(index) as usize).min(chars.len());

        // TODO: maxLength enforcement would check chars.len() + 1 > maxLength

        // Build new string
        let mut new_chars = chars;
        new_chars.insert(cursor, ch);
        let new_text: String = new_chars.into_iter().collect();

        // Write back to SharedBuffer
        buf.write_text(index, &new_text);
        buf.set_cursor_position(index, (cursor + 1) as i32);

        events.push(Event::value_change(index as u16));
    }

    /// Delete character before cursor (Backspace).
    fn delete_backward(
        &self,
        buf: &SharedBuffer,
        events: &mut EventRingBuffer,
        index: usize,
    ) {
        let content = buf.text_content(index).to_string();
        let chars: Vec<char> = content.chars().collect();
        let cursor = (buf.cursor_position(index) as usize).min(chars.len());

        if cursor == 0 {
            return;
        }

        let mut new_chars = chars;
        new_chars.remove(cursor - 1);
        let new_text: String = new_chars.into_iter().collect();

        buf.write_text(index, &new_text);
        buf.set_cursor_position(index, (cursor - 1) as i32);

        events.push(Event::value_change(index as u16));
    }

    /// Delete character after cursor (Delete key).
    fn delete_forward(
        &self,
        buf: &SharedBuffer,
        events: &mut EventRingBuffer,
        index: usize,
    ) {
        let content = buf.text_content(index).to_string();
        let chars: Vec<char> = content.chars().collect();
        let cursor = (buf.cursor_position(index) as usize).min(chars.len());

        if cursor >= chars.len() {
            return;
        }

        let mut new_chars = chars;
        new_chars.remove(cursor);
        let new_text: String = new_chars.into_iter().collect();

        buf.write_text(index, &new_text);
        // Cursor stays at same position

        events.push(Event::value_change(index as u16));
    }

    /// Move cursor by delta (-1 for left, +1 for right).
    fn move_cursor(&self, buf: &SharedBuffer, index: usize, delta: i32) {
        let len = self.char_count(buf, index) as i32;
        let current = buf.cursor_position(index);
        let new_pos = (current + delta).clamp(0, len);
        buf.set_cursor_position(index, new_pos);
    }

    /// Get the character count of the text content.
    fn char_count(&self, buf: &SharedBuffer, index: usize) -> usize {
        buf.text_content(index).chars().count()
    }
}

impl Default for TextEditor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_editor_new() {
        let _te = TextEditor::new();
    }
}
