//! Escape sequence parser for terminal input.
//!
//! Parses raw stdin bytes into structured events:
//! - CSI sequences (Arrow keys, Home, End, Insert, Delete, PageUp/Down, F1-F12)
//! - SS3 sequences (F1-F4, alternate encodings)
//! - SGR mouse (button, position, modifiers, press/release)
//! - Kitty keyboard protocol (codepoint, modifiers, state)
//! - Alt+key (ESC + char)
//! - Control keys (bytes 0-31)
//!
//! Uses a 10ms timeout for incomplete sequences to distinguish
//! genuine ESC key from the start of an escape sequence.


// =============================================================================
// Types
// =============================================================================

/// A parsed input event.
#[derive(Debug, Clone, PartialEq)]
pub enum ParsedEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    FocusGained,
    FocusLost,
    Paste(String),
    None,
}

/// A key event.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: Modifier,
    pub state: KeyState,
}

/// Key state (for Kitty keyboard protocol).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyState {
    Press,
    Repeat,
    Release,
}

/// Key code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyCode {
    Char(char),
    Enter,
    Tab,
    Backspace,
    Escape,
    Delete,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    F(u8),
    Null,
}

bitflags::bitflags! {
    /// Keyboard modifiers.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Modifier: u8 {
        const NONE  = 0;
        const SHIFT = 1 << 0;
        const ALT   = 1 << 1;
        const CTRL  = 1 << 2;
        const SUPER = 1 << 3;
    }
}

/// A mouse event.
#[derive(Debug, Clone, PartialEq)]
pub struct MouseEvent {
    pub kind: MouseKind,
    pub x: u16,
    pub y: u16,
    pub modifiers: Modifier,
}

/// Mouse event kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseKind {
    Press(MouseButton),
    Release(MouseButton),
    Move,
    ScrollUp,
    ScrollDown,
}

/// Mouse button.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

// =============================================================================
// Parser
// =============================================================================

/// Input parser state machine.
pub struct InputParser {
    buf: Vec<u8>,
}

impl InputParser {
    pub fn new() -> Self {
        Self { buf: Vec::with_capacity(64) }
    }

    /// Parse a byte sequence into events.
    /// Returns all parsed events and any remaining bytes.
    pub fn parse(&mut self, data: &[u8]) -> Vec<ParsedEvent> {
        self.buf.extend_from_slice(data);
        let mut events = Vec::new();

        while !self.buf.is_empty() {
            match self.try_parse_one() {
                ParseResult::Event(ev) => events.push(ev),
                ParseResult::Incomplete => break,
                ParseResult::None => {
                    // Consume one byte and continue
                    self.buf.remove(0);
                }
            }
        }

        events
    }

    /// Check if there's an incomplete sequence that might complete with a timeout.
    pub fn has_pending(&self) -> bool {
        !self.buf.is_empty()
    }

    /// Flush pending bytes as raw key events (timeout expired).
    pub fn flush_pending(&mut self) -> Vec<ParsedEvent> {
        let mut events = Vec::new();
        while !self.buf.is_empty() {
            let byte = self.buf.remove(0);
            events.push(ParsedEvent::Key(KeyEvent {
                code: KeyCode::Char(byte as char),
                modifiers: Modifier::NONE,
                state: KeyState::Press,
            }));
        }
        events
    }

    fn try_parse_one(&mut self) -> ParseResult {
        if self.buf.is_empty() {
            return ParseResult::None;
        }

        let first = self.buf[0];

        match first {
            // ESC
            0x1B => self.parse_escape(),
            // Control characters
            0x00 => { self.consume(1); ParseResult::Event(key(KeyCode::Null, Modifier::CTRL)) }
            0x01..=0x07 => {
                let ch = (first + b'a' - 1) as char;
                self.consume(1);
                ParseResult::Event(key(KeyCode::Char(ch), Modifier::CTRL))
            }
            0x08 => { self.consume(1); ParseResult::Event(key(KeyCode::Backspace, Modifier::NONE)) }
            0x09 => { self.consume(1); ParseResult::Event(key(KeyCode::Tab, Modifier::NONE)) }
            0x0A | 0x0D => { self.consume(1); ParseResult::Event(key(KeyCode::Enter, Modifier::NONE)) }
            0x0B..=0x0C => {
                let ch = (first + b'a' - 1) as char;
                self.consume(1);
                ParseResult::Event(key(KeyCode::Char(ch), Modifier::CTRL))
            }
            0x0E..=0x1A => {
                let ch = (first + b'a' - 1) as char;
                self.consume(1);
                ParseResult::Event(key(KeyCode::Char(ch), Modifier::CTRL))
            }
            0x7F => { self.consume(1); ParseResult::Event(key(KeyCode::Backspace, Modifier::NONE)) }
            // UTF-8 or ASCII printable
            0x20..=0x7E => {
                let ch = first as char;
                self.consume(1);
                ParseResult::Event(key(KeyCode::Char(ch), Modifier::NONE))
            }
            // UTF-8 multi-byte
            0x80..=0xFF => self.parse_utf8(),
            _ => {
                self.consume(1);
                ParseResult::None
            }
        }
    }

    fn parse_escape(&mut self) -> ParseResult {
        if self.buf.len() < 2 {
            return ParseResult::Incomplete;
        }

        match self.buf[1] {
            // CSI: ESC [
            b'[' => self.parse_csi(),
            // SS3: ESC O
            b'O' => self.parse_ss3(),
            // Alt+char: ESC + printable
            0x20..=0x7E => {
                let ch = self.buf[1] as char;
                self.consume(2);
                ParseResult::Event(key(KeyCode::Char(ch), Modifier::ALT))
            }
            // ESC ESC → Alt+Escape
            0x1B => {
                self.consume(2);
                ParseResult::Event(key(KeyCode::Escape, Modifier::ALT))
            }
            _ => {
                // Standalone ESC
                self.consume(1);
                ParseResult::Event(key(KeyCode::Escape, Modifier::NONE))
            }
        }
    }

    fn parse_csi(&mut self) -> ParseResult {
        // Minimum: ESC [ X (3 bytes)
        if self.buf.len() < 3 {
            return ParseResult::Incomplete;
        }

        // SGR mouse: ESC [ <
        if self.buf[2] == b'<' {
            return self.parse_sgr_mouse();
        }

        // X10 mouse: ESC [ M
        if self.buf[2] == b'M' {
            return self.parse_x10_mouse();
        }

        // Focus reporting: ESC [ I (gained) / ESC [ O (lost)
        if self.buf[2] == b'I' {
            self.consume(3);
            return ParseResult::Event(ParsedEvent::FocusGained);
        }
        if self.buf[2] == b'O' {
            self.consume(3);
            return ParseResult::Event(ParsedEvent::FocusLost);
        }

        // Find the final byte (0x40-0x7E)
        let mut end = 2;
        while end < self.buf.len() {
            if (0x40..=0x7E).contains(&self.buf[end]) {
                break;
            }
            end += 1;
        }

        if end >= self.buf.len() {
            return ParseResult::Incomplete;
        }

        let final_byte = self.buf[end];
        let params_str = String::from_utf8_lossy(&self.buf[2..end]).to_string();
        let params: Vec<u32> = params_str
            .split(';')
            .map(|s| s.parse::<u32>().unwrap_or(0))
            .collect();
        let consumed = end + 1;

        // Kitty keyboard: final byte is 'u'
        if final_byte == b'u' {
            self.consume(consumed);
            return self.parse_kitty_key(&params);
        }

        let modifiers = if params.len() >= 2 && params[1] > 0 {
            decode_modifier(params[1])
        } else {
            Modifier::NONE
        };

        let event = match final_byte {
            b'A' => key(KeyCode::Up, modifiers),
            b'B' => key(KeyCode::Down, modifiers),
            b'C' => key(KeyCode::Right, modifiers),
            b'D' => key(KeyCode::Left, modifiers),
            b'H' => key(KeyCode::Home, modifiers),
            b'F' => key(KeyCode::End, modifiers),
            b'P' => key(KeyCode::F(1), modifiers),
            b'Q' => key(KeyCode::F(2), modifiers),
            b'R' => key(KeyCode::F(3), modifiers),
            b'S' => key(KeyCode::F(4), modifiers),
            b'Z' => key(KeyCode::Tab, Modifier::SHIFT), // Shift+Tab
            b'~' => {
                match params.first().copied().unwrap_or(0) {
                    1 => key(KeyCode::Home, modifiers),
                    2 => key(KeyCode::Insert, modifiers),
                    3 => key(KeyCode::Delete, modifiers),
                    4 => key(KeyCode::End, modifiers),
                    5 => key(KeyCode::PageUp, modifiers),
                    6 => key(KeyCode::PageDown, modifiers),
                    15 => key(KeyCode::F(5), modifiers),
                    17 => key(KeyCode::F(6), modifiers),
                    18 => key(KeyCode::F(7), modifiers),
                    19 => key(KeyCode::F(8), modifiers),
                    20 => key(KeyCode::F(9), modifiers),
                    21 => key(KeyCode::F(10), modifiers),
                    23 => key(KeyCode::F(11), modifiers),
                    24 => key(KeyCode::F(12), modifiers),
                    _ => ParsedEvent::None,
                }
            }
            _ => ParsedEvent::None,
        };

        self.consume(consumed);
        ParseResult::Event(event)
    }

    fn parse_ss3(&mut self) -> ParseResult {
        if self.buf.len() < 3 {
            return ParseResult::Incomplete;
        }

        let event = match self.buf[2] {
            b'A' => key(KeyCode::Up, Modifier::NONE),
            b'B' => key(KeyCode::Down, Modifier::NONE),
            b'C' => key(KeyCode::Right, Modifier::NONE),
            b'D' => key(KeyCode::Left, Modifier::NONE),
            b'H' => key(KeyCode::Home, Modifier::NONE),
            b'F' => key(KeyCode::End, Modifier::NONE),
            b'P' => key(KeyCode::F(1), Modifier::NONE),
            b'Q' => key(KeyCode::F(2), Modifier::NONE),
            b'R' => key(KeyCode::F(3), Modifier::NONE),
            b'S' => key(KeyCode::F(4), Modifier::NONE),
            _ => ParsedEvent::None,
        };

        self.consume(3);
        ParseResult::Event(event)
    }

    fn parse_sgr_mouse(&mut self) -> ParseResult {
        // ESC [ < Pb ; Px ; Py M/m
        // Need at least: ESC [ < d ; d ; d M = 10+ bytes
        let start = 3; // skip ESC [ <
        let mut end = start;
        while end < self.buf.len() {
            if self.buf[end] == b'M' || self.buf[end] == b'm' {
                break;
            }
            end += 1;
        }

        if end >= self.buf.len() {
            return ParseResult::Incomplete;
        }

        let is_release = self.buf[end] == b'm';
        let params_str = String::from_utf8_lossy(&self.buf[start..end]).to_string();
        let parts: Vec<u16> = params_str.split(';').map(|s| s.parse().unwrap_or(0)).collect();

        let consumed = end + 1;
        self.consume(consumed);

        if parts.len() < 3 {
            return ParseResult::Event(ParsedEvent::None);
        }

        let cb = parts[0];
        let x = parts[1].saturating_sub(1); // 1-indexed → 0-indexed
        let y = parts[2].saturating_sub(1);

        let mut modifiers = Modifier::NONE;
        if cb & 4 != 0 { modifiers |= Modifier::SHIFT; }
        if cb & 8 != 0 { modifiers |= Modifier::ALT; }
        if cb & 16 != 0 { modifiers |= Modifier::CTRL; }

        let base = cb & 3;
        let kind = if cb & 64 != 0 {
            // Scroll wheel
            match base {
                0 => MouseKind::ScrollUp,
                1 => MouseKind::ScrollDown,
                _ => MouseKind::ScrollUp,
            }
        } else if cb & 32 != 0 {
            // Motion
            MouseKind::Move
        } else if is_release {
            let button = match base { 0 => MouseButton::Left, 1 => MouseButton::Middle, _ => MouseButton::Right };
            MouseKind::Release(button)
        } else {
            let button = match base { 0 => MouseButton::Left, 1 => MouseButton::Middle, _ => MouseButton::Right };
            MouseKind::Press(button)
        };

        ParseResult::Event(ParsedEvent::Mouse(MouseEvent {
            kind,
            x,
            y,
            modifiers,
        }))
    }

    fn parse_x10_mouse(&mut self) -> ParseResult {
        // ESC [ M Cb Cx Cy (6 bytes)
        if self.buf.len() < 6 {
            return ParseResult::Incomplete;
        }

        let cb = self.buf[3].wrapping_sub(32);
        let x = self.buf[4].wrapping_sub(33) as u16;
        let y = self.buf[5].wrapping_sub(33) as u16;

        self.consume(6);

        let mut modifiers = Modifier::NONE;
        if cb & 4 != 0 { modifiers |= Modifier::SHIFT; }
        if cb & 8 != 0 { modifiers |= Modifier::ALT; }
        if cb & 16 != 0 { modifiers |= Modifier::CTRL; }

        let base = cb & 3;
        let kind = if cb & 64 != 0 {
            match base { 0 => MouseKind::ScrollUp, _ => MouseKind::ScrollDown }
        } else if base == 3 {
            MouseKind::Release(MouseButton::Left)
        } else {
            let button = match base { 0 => MouseButton::Left, 1 => MouseButton::Middle, _ => MouseButton::Right };
            MouseKind::Press(button)
        };

        ParseResult::Event(ParsedEvent::Mouse(MouseEvent {
            kind,
            x,
            y,
            modifiers,
        }))
    }

    fn parse_kitty_key(&self, params: &[u32]) -> ParseResult {
        let codepoint = params.first().copied().unwrap_or(0);
        let modifiers = if params.len() >= 2 { decode_modifier(params[1]) } else { Modifier::NONE };
        let state = if params.len() >= 3 {
            match params[2] {
                2 => KeyState::Repeat,
                3 => KeyState::Release,
                _ => KeyState::Press,
            }
        } else {
            KeyState::Press
        };

        let code = match codepoint {
            9 => KeyCode::Tab,
            13 => KeyCode::Enter,
            27 => KeyCode::Escape,
            127 => KeyCode::Backspace,
            cp => {
                if let Some(ch) = char::from_u32(cp) {
                    KeyCode::Char(ch)
                } else {
                    KeyCode::Null
                }
            }
        };

        ParseResult::Event(ParsedEvent::Key(KeyEvent {
            code,
            modifiers,
            state,
        }))
    }

    fn parse_utf8(&mut self) -> ParseResult {
        let first = self.buf[0];
        let expected_len = if first & 0xE0 == 0xC0 { 2 }
            else if first & 0xF0 == 0xE0 { 3 }
            else if first & 0xF8 == 0xF0 { 4 }
            else {
                self.consume(1);
                return ParseResult::None;
            };

        if self.buf.len() < expected_len {
            return ParseResult::Incomplete;
        }

        let s = String::from_utf8_lossy(&self.buf[..expected_len]).to_string();
        self.consume(expected_len);

        if let Some(ch) = s.chars().next() {
            ParseResult::Event(key(KeyCode::Char(ch), Modifier::NONE))
        } else {
            ParseResult::None
        }
    }

    fn consume(&mut self, n: usize) {
        self.buf.drain(..n);
    }
}

impl Default for InputParser {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Helpers
// =============================================================================

enum ParseResult {
    Event(ParsedEvent),
    Incomplete,
    None,
}

fn key(code: KeyCode, modifiers: Modifier) -> ParsedEvent {
    ParsedEvent::Key(KeyEvent {
        code,
        modifiers,
        state: KeyState::Press,
    })
}

/// Decode CSI modifier parameter (1-based).
fn decode_modifier(param: u32) -> Modifier {
    let val = if param > 0 { param - 1 } else { 0 };
    let mut m = Modifier::NONE;
    if val & 1 != 0 { m |= Modifier::SHIFT; }
    if val & 2 != 0 { m |= Modifier::ALT; }
    if val & 4 != 0 { m |= Modifier::CTRL; }
    if val & 8 != 0 { m |= Modifier::SUPER; }
    m
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_bytes(data: &[u8]) -> Vec<ParsedEvent> {
        let mut parser = InputParser::new();
        parser.parse(data)
    }

    #[test]
    fn test_ascii_chars() {
        let events = parse_bytes(b"abc");
        assert_eq!(events.len(), 3);
        assert_eq!(events[0], key(KeyCode::Char('a'), Modifier::NONE));
        assert_eq!(events[1], key(KeyCode::Char('b'), Modifier::NONE));
        assert_eq!(events[2], key(KeyCode::Char('c'), Modifier::NONE));
    }

    #[test]
    fn test_enter() {
        let events = parse_bytes(b"\r");
        assert_eq!(events[0], key(KeyCode::Enter, Modifier::NONE));
    }

    #[test]
    fn test_ctrl_c() {
        let events = parse_bytes(b"\x03");
        assert_eq!(events[0], key(KeyCode::Char('c'), Modifier::CTRL));
    }

    #[test]
    fn test_arrow_keys() {
        assert_eq!(parse_bytes(b"\x1b[A")[0], key(KeyCode::Up, Modifier::NONE));
        assert_eq!(parse_bytes(b"\x1b[B")[0], key(KeyCode::Down, Modifier::NONE));
        assert_eq!(parse_bytes(b"\x1b[C")[0], key(KeyCode::Right, Modifier::NONE));
        assert_eq!(parse_bytes(b"\x1b[D")[0], key(KeyCode::Left, Modifier::NONE));
    }

    #[test]
    fn test_function_keys() {
        assert_eq!(parse_bytes(b"\x1bOP")[0], key(KeyCode::F(1), Modifier::NONE));
        assert_eq!(parse_bytes(b"\x1b[15~")[0], key(KeyCode::F(5), Modifier::NONE));
    }

    #[test]
    fn test_shift_tab() {
        assert_eq!(parse_bytes(b"\x1b[Z")[0], key(KeyCode::Tab, Modifier::SHIFT));
    }

    #[test]
    fn test_alt_key() {
        assert_eq!(parse_bytes(b"\x1bx")[0], key(KeyCode::Char('x'), Modifier::ALT));
    }

    #[test]
    fn test_delete() {
        assert_eq!(parse_bytes(b"\x1b[3~")[0], key(KeyCode::Delete, Modifier::NONE));
    }

    #[test]
    fn test_page_up_down() {
        assert_eq!(parse_bytes(b"\x1b[5~")[0], key(KeyCode::PageUp, Modifier::NONE));
        assert_eq!(parse_bytes(b"\x1b[6~")[0], key(KeyCode::PageDown, Modifier::NONE));
    }

    #[test]
    fn test_sgr_mouse_press() {
        // ESC [ < 0 ; 10 ; 20 M → Left press at (9, 19)
        let events = parse_bytes(b"\x1b[<0;10;20M");
        if let ParsedEvent::Mouse(m) = &events[0] {
            assert_eq!(m.kind, MouseKind::Press(MouseButton::Left));
            assert_eq!(m.x, 9);
            assert_eq!(m.y, 19);
        } else {
            panic!("Expected mouse event");
        }
    }

    #[test]
    fn test_sgr_mouse_release() {
        let events = parse_bytes(b"\x1b[<0;10;20m");
        if let ParsedEvent::Mouse(m) = &events[0] {
            assert_eq!(m.kind, MouseKind::Release(MouseButton::Left));
        } else {
            panic!("Expected mouse event");
        }
    }

    #[test]
    fn test_sgr_scroll() {
        let events = parse_bytes(b"\x1b[<64;10;20M");
        if let ParsedEvent::Mouse(m) = &events[0] {
            assert_eq!(m.kind, MouseKind::ScrollUp);
        } else {
            panic!("Expected mouse event");
        }
    }

    #[test]
    fn test_modifier_decode() {
        assert_eq!(decode_modifier(2), Modifier::SHIFT);
        assert_eq!(decode_modifier(3), Modifier::ALT);
        assert_eq!(decode_modifier(5), Modifier::CTRL);
    }
}
