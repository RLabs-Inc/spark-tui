# Event System Specification

## Overview

This specification documents the complete event system: raw terminal input parsing, keyboard and mouse event structures, event propagation, hit testing, and handler registration.

**Source Files Analyzed:**
- `/Users/rusty/Documents/Projects/TUI/tui/src/state/keyboard.ts`
- `/Users/rusty/Documents/Projects/TUI/tui/src/state/mouse.ts`
- `/Users/rusty/Documents/Projects/TUI/tui/src/state/input.ts`
- `/Users/rusty/Documents/Projects/TUI/tui/src/state/global-keys.ts`
- `/Users/rusty/Documents/Projects/TUI/tui/src/state/context.ts`

---

## 1. Input Pipeline

### 1.1 Architecture

```
Terminal (stdin)
      ↓
  Raw Bytes
      ↓
  InputBuffer (with timeout)
      ↓
  Parse Escape Sequences
      ↓
  KeyboardEvent / MouseEvent
      ↓
  Global Handlers (Ctrl+C, Tab, etc.)
      ↓
  Focused Component Dispatch
      ↓
  Component Event Handlers
```

### 1.2 Raw Input Reader

```rust
use std::io::{self, Read};
use std::time::Duration;

pub struct InputReader {
    buffer: Vec<u8>,
    timeout: Duration,
}

impl InputReader {
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(32),
            timeout: Duration::from_millis(10),
        }
    }

    /// Read input with timeout (for escape sequence detection)
    pub fn read(&mut self) -> io::Result<Option<Vec<u8>>> {
        self.buffer.clear();

        // Set stdin to non-blocking with timeout
        let stdin = io::stdin();
        let mut handle = stdin.lock();

        // Read first byte (blocking)
        let mut first = [0u8; 1];
        match handle.read(&mut first) {
            Ok(0) => return Ok(None),
            Ok(_) => self.buffer.push(first[0]),
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => return Ok(None),
            Err(e) => return Err(e),
        }

        // If escape, wait briefly for more bytes (escape sequence)
        if first[0] == 0x1b {
            std::thread::sleep(self.timeout);

            // Read any additional bytes (non-blocking)
            let mut more = [0u8; 31];
            match handle.read(&mut more) {
                Ok(n) if n > 0 => self.buffer.extend_from_slice(&more[..n]),
                _ => {}
            }
        }

        Ok(Some(self.buffer.clone()))
    }
}
```

---

## 2. Keyboard Events

### 2.1 KeyboardEvent Structure

```rust
#[derive(Debug, Clone)]
pub struct KeyboardEvent {
    /// Key name (e.g., "a", "Enter", "ArrowUp", "F1")
    pub key: String,

    /// Modifier keys
    pub modifiers: Modifiers,

    /// Key state
    pub state: KeyState,

    /// Raw bytes (for debugging)
    pub raw: Vec<u8>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyState {
    Press,
    Release,
    Repeat,
}
```

### 2.2 Key Names

```rust
/// All recognized key names
pub mod keys {
    // Letters
    pub const A: &str = "a";
    pub const B: &str = "b";
    // ... through z

    // Digits
    pub const DIGIT_0: &str = "0";
    // ... through 9

    // Special keys
    pub const ENTER: &str = "Enter";
    pub const TAB: &str = "Tab";
    pub const ESCAPE: &str = "Escape";
    pub const SPACE: &str = " ";
    pub const BACKSPACE: &str = "Backspace";
    pub const DELETE: &str = "Delete";

    // Navigation
    pub const ARROW_UP: &str = "ArrowUp";
    pub const ARROW_DOWN: &str = "ArrowDown";
    pub const ARROW_LEFT: &str = "ArrowLeft";
    pub const ARROW_RIGHT: &str = "ArrowRight";
    pub const HOME: &str = "Home";
    pub const END: &str = "End";
    pub const PAGE_UP: &str = "PageUp";
    pub const PAGE_DOWN: &str = "PageDown";
    pub const INSERT: &str = "Insert";

    // Function keys
    pub const F1: &str = "F1";
    pub const F2: &str = "F2";
    // ... through F12
}
```

### 2.3 Escape Sequence Parsing

```rust
pub fn parse_keyboard_event(bytes: &[u8]) -> Option<KeyboardEvent> {
    if bytes.is_empty() {
        return None;
    }

    // Single byte (printable or control)
    if bytes.len() == 1 {
        return parse_single_byte(bytes[0]);
    }

    // Escape sequence
    if bytes[0] == 0x1b {
        if bytes.len() == 1 {
            // Bare escape
            return Some(KeyboardEvent {
                key: "Escape".to_string(),
                modifiers: Modifiers::default(),
                state: KeyState::Press,
                raw: bytes.to_vec(),
            });
        }

        // CSI sequence: ESC [
        if bytes.len() >= 2 && bytes[1] == b'[' {
            return parse_csi_sequence(&bytes[2..], bytes);
        }

        // SS3 sequence: ESC O
        if bytes.len() >= 2 && bytes[1] == b'O' {
            return parse_ss3_sequence(&bytes[2..], bytes);
        }

        // Alt+key: ESC <key>
        if bytes.len() == 2 {
            let mut event = parse_single_byte(bytes[1])?;
            event.modifiers.alt = true;
            event.raw = bytes.to_vec();
            return Some(event);
        }
    }

    None
}

fn parse_single_byte(byte: u8) -> Option<KeyboardEvent> {
    let (key, modifiers) = match byte {
        // Control characters
        0x00 => ("@".to_string(), Modifiers { ctrl: true, ..Default::default() }), // Ctrl+@
        0x01..=0x1a => {
            let ch = (byte + 0x60) as char;
            (ch.to_string(), Modifiers { ctrl: true, ..Default::default() })
        }
        0x1b => ("Escape".to_string(), Modifiers::default()),
        0x7f => ("Backspace".to_string(), Modifiers::default()),

        // Printable ASCII
        0x20..=0x7e => ((byte as char).to_string(), Modifiers::default()),

        _ => return None,
    };

    Some(KeyboardEvent {
        key,
        modifiers,
        state: KeyState::Press,
        raw: vec![byte],
    })
}

fn parse_csi_sequence(params: &[u8], raw: &[u8]) -> Option<KeyboardEvent> {
    // Find terminator
    let terminator_pos = params.iter().position(|&b| b >= 0x40 && b <= 0x7e)?;
    let terminator = params[terminator_pos];
    let param_bytes = &params[..terminator_pos];

    // Parse parameters (semicolon-separated)
    let param_str = std::str::from_utf8(param_bytes).ok()?;
    let params: Vec<u32> = param_str
        .split(';')
        .filter_map(|s| s.parse().ok())
        .collect();

    // Extract modifiers from param 2 (if present)
    let modifiers = if params.len() >= 2 {
        decode_modifiers(params[1])
    } else {
        Modifiers::default()
    };

    // Determine key based on terminator
    let key = match terminator {
        b'A' => "ArrowUp",
        b'B' => "ArrowDown",
        b'C' => "ArrowRight",
        b'D' => "ArrowLeft",
        b'H' => "Home",
        b'F' => "End",
        b'P' => "F1",
        b'Q' => "F2",
        b'R' => "F3",
        b'S' => "F4",
        b'~' => {
            // Tilde sequences: ESC [ <n> ~
            match params.get(0) {
                Some(1) => "Home",
                Some(2) => "Insert",
                Some(3) => "Delete",
                Some(4) => "End",
                Some(5) => "PageUp",
                Some(6) => "PageDown",
                Some(15) => "F5",
                Some(17) => "F6",
                Some(18) => "F7",
                Some(19) => "F8",
                Some(20) => "F9",
                Some(21) => "F10",
                Some(23) => "F11",
                Some(24) => "F12",
                _ => return None,
            }
        }
        _ => return None,
    };

    Some(KeyboardEvent {
        key: key.to_string(),
        modifiers,
        state: KeyState::Press,
        raw: raw.to_vec(),
    })
}

fn parse_ss3_sequence(params: &[u8], raw: &[u8]) -> Option<KeyboardEvent> {
    if params.is_empty() {
        return None;
    }

    let key = match params[0] {
        b'A' => "ArrowUp",
        b'B' => "ArrowDown",
        b'C' => "ArrowRight",
        b'D' => "ArrowLeft",
        b'H' => "Home",
        b'F' => "End",
        b'P' => "F1",
        b'Q' => "F2",
        b'R' => "F3",
        b'S' => "F4",
        _ => return None,
    };

    Some(KeyboardEvent {
        key: key.to_string(),
        modifiers: Modifiers::default(),
        state: KeyState::Press,
        raw: raw.to_vec(),
    })
}

fn decode_modifiers(code: u32) -> Modifiers {
    // Modifier encoding: 1 + (shift ? 1 : 0) + (alt ? 2 : 0) + (ctrl ? 4 : 0) + (meta ? 8 : 0)
    let bits = code.saturating_sub(1);
    Modifiers {
        shift: bits & 1 != 0,
        alt: bits & 2 != 0,
        ctrl: bits & 4 != 0,
        meta: bits & 8 != 0,
    }
}
```

---

## 3. Mouse Events

### 3.1 MouseEvent Structure

```rust
#[derive(Debug, Clone)]
pub struct MouseEvent {
    /// X position (0-indexed)
    pub x: u16,
    /// Y position (0-indexed)
    pub y: u16,
    /// Mouse button
    pub button: MouseButton,
    /// Event type
    pub action: MouseAction,
    /// Modifier keys
    pub modifiers: Modifiers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    None,
    WheelUp,
    WheelDown,
    WheelLeft,
    WheelRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseAction {
    Down,
    Up,
    Move,
    Drag,
}
```

### 3.2 Mouse Protocol

```rust
/// Enable mouse tracking
pub const MOUSE_ENABLE: &str = concat!(
    "\x1b[?1000h",  // Basic mouse tracking
    "\x1b[?1002h",  // Button-event tracking
    "\x1b[?1003h",  // All motion tracking
    "\x1b[?1006h",  // SGR extended mode
);

/// Disable mouse tracking
pub const MOUSE_DISABLE: &str = concat!(
    "\x1b[?1000l",
    "\x1b[?1002l",
    "\x1b[?1003l",
    "\x1b[?1006l",
);
```

### 3.3 SGR Mouse Parsing

```rust
pub fn parse_mouse_event(bytes: &[u8]) -> Option<MouseEvent> {
    // SGR format: ESC [ < Cb ; Cx ; Cy M/m
    // Where Cb = button code, Cx = x+1, Cy = y+1
    // M = press, m = release

    if bytes.len() < 6 || bytes[0] != 0x1b || bytes[1] != b'[' || bytes[2] != b'<' {
        return None;
    }

    // Find terminator (M or m)
    let terminator_pos = bytes.iter().position(|&b| b == b'M' || b == b'm')?;
    let is_release = bytes[terminator_pos] == b'm';

    // Parse parameters
    let param_str = std::str::from_utf8(&bytes[3..terminator_pos]).ok()?;
    let params: Vec<u16> = param_str
        .split(';')
        .filter_map(|s| s.parse().ok())
        .collect();

    if params.len() < 3 {
        return None;
    }

    let button_code = params[0];
    let x = params[1].saturating_sub(1);
    let y = params[2].saturating_sub(1);

    // Decode button and modifiers
    let (button, action, modifiers) = decode_mouse_button(button_code, is_release);

    Some(MouseEvent {
        x,
        y,
        button,
        action,
        modifiers,
    })
}

fn decode_mouse_button(code: u16, is_release: bool) -> (MouseButton, MouseAction, Modifiers) {
    // Button encoding:
    // bits 0-1: button (0=left, 1=middle, 2=right, 3=release)
    // bit 2: shift
    // bit 3: meta
    // bit 4: ctrl
    // bit 5: motion
    // bits 6-7: 01 = wheel up, 10 = wheel down

    let button_bits = code & 0b11;
    let shift = code & 0b100 != 0;
    let meta = code & 0b1000 != 0;
    let ctrl = code & 0b10000 != 0;
    let motion = code & 0b100000 != 0;
    let wheel = (code >> 6) & 0b11;

    let modifiers = Modifiers {
        shift,
        alt: false,
        ctrl,
        meta,
    };

    // Wheel events
    if wheel != 0 {
        let button = match wheel {
            1 => MouseButton::WheelUp,
            2 => MouseButton::WheelDown,
            _ => MouseButton::None,
        };
        return (button, MouseAction::Down, modifiers);
    }

    // Regular buttons
    let button = match button_bits {
        0 => MouseButton::Left,
        1 => MouseButton::Middle,
        2 => MouseButton::Right,
        3 => MouseButton::None,
        _ => MouseButton::None,
    };

    let action = if is_release {
        MouseAction::Up
    } else if motion {
        MouseAction::Drag
    } else {
        MouseAction::Down
    };

    (button, action, modifiers)
}
```

---

## 4. Event Dispatch

### 4.1 Dispatch Order

```
1. Global handlers (Ctrl+C, Tab navigation)
2. Focused component keyboard handlers
3. Component-specific handlers
4. Default behaviors
```

### 4.2 Keyboard Dispatch

```rust
pub fn dispatch_keyboard_event(event: &KeyboardEvent) -> bool {
    // Only dispatch Press events to handlers
    if event.state != KeyState::Press {
        return false;
    }

    // 1. Global handlers first
    if handle_global_key(event) {
        return true;
    }

    // 2. Focused component
    let focused = get_focused_index();
    if focused >= 0 {
        let index = focused as usize;

        // Check for handler
        if let Some(handler) = INTERACTION.on_key_down.get(index) {
            handler(event.clone());
            return true;
        }
    }

    false
}
```

### 4.3 Mouse Dispatch

```rust
pub fn dispatch_mouse_event(event: &MouseEvent, hit_grid: &HitGrid) -> bool {
    // Get component under cursor
    let hit = hit_grid.hit_test(event.x as usize, event.y as usize);

    match event.action {
        MouseAction::Down => {
            if let Some(index) = hit {
                // Track for click detection
                set_mouse_down_target(index);

                // Fire mouse down handler
                if let Some(handler) = INTERACTION.on_mouse_down.get(index) {
                    handler(event.clone());
                }

                // Focus on click
                if INTERACTION.focusable.get(index) {
                    focus(index);
                }

                return true;
            }
        }

        MouseAction::Up => {
            let down_target = get_mouse_down_target();
            clear_mouse_down_target();

            if let Some(index) = hit {
                // Fire mouse up handler
                if let Some(handler) = INTERACTION.on_mouse_up.get(index) {
                    handler(event.clone());
                }

                // Click = down + up on same component
                if Some(index) == down_target {
                    if let Some(handler) = INTERACTION.on_click.get(index) {
                        handler(event.clone());
                    }
                }

                return true;
            }
        }

        MouseAction::Move | MouseAction::Drag => {
            // Track hover state
            let prev_hover = get_hover_target();

            if hit != prev_hover {
                // Mouse leave previous
                if let Some(prev) = prev_hover {
                    if let Some(handler) = INTERACTION.on_mouse_leave.get(prev) {
                        handler(event.clone());
                    }
                }

                // Mouse enter new
                if let Some(index) = hit {
                    if let Some(handler) = INTERACTION.on_mouse_enter.get(index) {
                        handler(event.clone());
                    }
                }

                set_hover_target(hit);
            }
        }
    }

    // Wheel events
    if matches!(event.button, MouseButton::WheelUp | MouseButton::WheelDown) {
        return handle_wheel_scroll(event, hit_grid);
    }

    false
}
```

---

## 5. Global Handlers

### 5.1 Built-in Global Keys

```rust
pub fn handle_global_key(event: &KeyboardEvent) -> bool {
    // Ctrl+C - Exit
    if event.modifiers.ctrl && event.key == "c" {
        request_exit();
        return true;
    }

    // Tab - Focus navigation
    if event.key == "Tab" {
        if event.modifiers.shift {
            focus_previous();
        } else {
            focus_next();
        }
        return true;
    }

    // Arrow keys - Scroll focused scrollable
    if matches!(event.key.as_str(), "ArrowUp" | "ArrowDown" | "ArrowLeft" | "ArrowRight") {
        if handle_arrow_scroll(event) {
            return true;
        }
    }

    // Page Up/Down
    if matches!(event.key.as_str(), "PageUp" | "PageDown") {
        if handle_page_scroll(event) {
            return true;
        }
    }

    // Home/End
    if matches!(event.key.as_str(), "Home" | "End") {
        if event.modifiers.ctrl {
            if handle_scroll_home_end(event) {
                return true;
            }
        }
    }

    false
}
```

### 5.2 Scroll Handling

```rust
fn handle_arrow_scroll(event: &KeyboardEvent) -> bool {
    let focused = get_focused_index();
    if focused < 0 {
        return false;
    }

    let index = focused as usize;

    // Find nearest scrollable ancestor
    let scrollable = find_scrollable_ancestor(index);
    if scrollable.is_none() {
        return false;
    }

    let scrollable = scrollable.unwrap();

    match event.key.as_str() {
        "ArrowUp" => scroll_by(scrollable, 0, -1),
        "ArrowDown" => scroll_by(scrollable, 0, 1),
        "ArrowLeft" => scroll_by(scrollable, -1, 0),
        "ArrowRight" => scroll_by(scrollable, 1, 0),
        _ => false,
    }
}

fn handle_wheel_scroll(event: &MouseEvent, hit_grid: &HitGrid) -> bool {
    // Try element under cursor
    if let Some(index) = hit_grid.hit_test(event.x as usize, event.y as usize) {
        let scrollable = find_scrollable_ancestor(index);
        if let Some(s) = scrollable {
            let delta = match event.button {
                MouseButton::WheelUp => -3,
                MouseButton::WheelDown => 3,
                _ => return false,
            };
            return scroll_by(s, 0, delta);
        }
    }

    // Fallback to focused scrollable
    let focused = get_focused_index();
    if focused >= 0 {
        if let Some(scrollable) = find_scrollable_ancestor(focused as usize) {
            let delta = match event.button {
                MouseButton::WheelUp => -3,
                MouseButton::WheelDown => 3,
                _ => return false,
            };
            return scroll_by(scrollable, 0, delta);
        }
    }

    false
}
```

---

## 6. Hit Testing

### 6.1 HitGrid Structure

```rust
/// O(1) hit testing via pre-computed grid
pub struct HitGrid {
    grid: Vec<i32>,
    width: usize,
    height: usize,
}

impl HitGrid {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            grid: vec![-1; width * height],
            width,
            height,
        }
    }

    pub fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, index: usize) {
        for row in y..(y + h).min(self.height) {
            for col in x..(x + w).min(self.width) {
                self.grid[row * self.width + col] = index as i32;
            }
        }
    }

    pub fn hit_test(&self, x: usize, y: usize) -> Option<usize> {
        if x < self.width && y < self.height {
            let index = self.grid[y * self.width + x];
            if index >= 0 {
                Some(index as usize)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.grid.fill(-1);
    }
}
```

### 6.2 Population During Render

```rust
fn populate_hit_grid(
    hit_grid: &mut HitGrid,
    index: usize,
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    arrays: &ComponentArrays,
) {
    // Only register interactive elements
    let is_interactive = INTERACTION.focusable.get(index)
        || INTERACTION.on_click.get(index).is_some()
        || INTERACTION.on_mouse_enter.get(index).is_some()
        || INTERACTION.on_mouse_down.get(index).is_some();

    if is_interactive {
        hit_grid.fill_rect(x, y, w, h, index);
    }

    // Children are rendered after (higher z-index overwrites)
    for child in children(index, arrays) {
        // ... recursive populate
    }
}
```

---

## 7. Handler Registration

### 7.1 API

```rust
/// Register keyboard handler for component
pub fn on_key_down<F: Fn(KeyboardEvent) + 'static>(index: usize, handler: F) {
    INTERACTION.on_key_down.set(index, Some(Box::new(handler)));
}

/// Register specific key handler
pub fn on_key<F: Fn() + 'static>(index: usize, key: &str, handler: F) {
    let key = key.to_string();
    on_key_down(index, move |event| {
        if event.key == key {
            handler();
        }
    });
}

/// Register handler only when focused
pub fn on_focused_key<F: Fn() + 'static>(index: usize, key: &str, handler: F) {
    let key = key.to_string();
    on_key_down(index, move |event| {
        if event.key == key && is_focused(index) {
            handler();
        }
    });
}

/// Mouse handlers
pub fn on_click<F: Fn(MouseEvent) + 'static>(index: usize, handler: F) {
    INTERACTION.on_click.set(index, Some(Box::new(handler)));
}

pub fn on_mouse_enter<F: Fn(MouseEvent) + 'static>(index: usize, handler: F) {
    INTERACTION.on_mouse_enter.set(index, Some(Box::new(handler)));
}

pub fn on_mouse_leave<F: Fn(MouseEvent) + 'static>(index: usize, handler: F) {
    INTERACTION.on_mouse_leave.set(index, Some(Box::new(handler)));
}

pub fn on_scroll<F: Fn(ScrollEvent) + 'static>(index: usize, handler: F) {
    INTERACTION.on_scroll.set(index, Some(Box::new(handler)));
}
```

---

## 8. Event Loop Integration

### 8.1 Main Loop

```rust
pub fn run_event_loop() {
    let mut input_reader = InputReader::new();
    let mut hit_grid = HitGrid::new(terminal_width(), terminal_height());

    loop {
        // Check for exit request
        if should_exit() {
            break;
        }

        // Read input
        if let Ok(Some(bytes)) = input_reader.read() {
            // Try keyboard event
            if let Some(key_event) = parse_keyboard_event(&bytes) {
                dispatch_keyboard_event(&key_event);
            }
            // Try mouse event
            else if let Some(mouse_event) = parse_mouse_event(&bytes) {
                dispatch_mouse_event(&mouse_event, &hit_grid);
            }
        }

        // Run reactive effects (may update hit_grid)
        flush_effects();

        // Update hit grid from latest render
        update_hit_grid(&mut hit_grid);

        // Small sleep to prevent busy loop
        std::thread::sleep(Duration::from_millis(1));
    }
}
```

---

## 9. Hidden Automatic Behaviors

### 9.1 Ctrl+C Exits
Ctrl+C always triggers application exit (can be overridden).

### 9.2 Tab Cycles Focus
Tab and Shift+Tab navigate focus ring automatically.

### 9.3 Arrow Keys Scroll
Arrow keys scroll the focused scrollable container.

### 9.4 Click Detection
Click = MouseDown + MouseUp on same component.

### 9.5 Hover Tracking
Mouse enter/leave tracked automatically as cursor moves.

### 9.6 Focus on Click
Focusable elements automatically focus when clicked.

### 9.7 Press-Only Dispatch
Only Press events (not Release/Repeat) fire handlers.

### 9.8 Escape Sequence Timeout
10ms timeout distinguishes bare Escape from escape sequences.

### 9.9 Modifier Decoding
Modifiers extracted from CSI parameter 2 automatically.

### 9.10 SGR Mouse Mode
Uses SGR extended mode for large terminal support.

---

## 10. Module Structure

```
crates/tui/src/state/
├── mod.rs
├── input.rs        # InputReader, raw byte handling
├── keyboard.rs     # KeyboardEvent, parsing
├── mouse.rs        # MouseEvent, SGR parsing
├── global_keys.rs  # Global handlers (Ctrl+C, Tab, etc.)
├── hit_grid.rs     # HitGrid for mouse hit testing
└── dispatch.rs     # Event dispatch logic
```

---

## 11. Summary

The event system provides:

✅ **Raw Input Parsing**: Handles escape sequences with timeout
✅ **Keyboard Events**: Full key names, modifiers, CSI/SS3 sequences
✅ **Mouse Events**: SGR extended mode, buttons, wheel, drag
✅ **Global Handlers**: Ctrl+C, Tab navigation, arrow scroll
✅ **Hit Testing**: O(1) lookup via pre-computed grid
✅ **Focus Integration**: Click-to-focus, keyboard dispatch to focused
✅ **Handler Registration**: Simple API for component handlers
✅ **Hover Tracking**: Automatic enter/leave detection
