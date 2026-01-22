# Text & Cursor Specification

## Overview

This specification documents text handling (measurement, wrapping, truncation) and cursor management (terminal cursor, drawn cursor, blink, selection).

**Source Files Analyzed:**
- `/Users/rusty/Documents/Projects/TUI/tui/src/state/cursor.ts`
- `/Users/rusty/Documents/Projects/TUI/tui/src/state/drawnCursor.ts`
- `/Users/rusty/Documents/Projects/TUI/tui/src/utils/text.ts`
- `/Users/rusty/Documents/Projects/TUI/tui/src/primitives/text.ts`
- `/Users/rusty/Documents/Projects/TUI/tui/src/pipeline/layout/utils/text-measure.ts`

---

## 1. Text Measurement

### 1.1 Unicode-Aware Width

```rust
use unicode_width::{UnicodeWidthStr, UnicodeWidthChar};

/// Measure display width of text (terminal columns)
pub fn measure_text_width(text: &str) -> usize {
    UnicodeWidthStr::width(text)
}

/// Measure single character width
pub fn measure_char_width(ch: char) -> usize {
    UnicodeWidthChar::width(ch).unwrap_or(0)
}
```

### 1.2 Width Rules

| Character Type | Width |
|----------------|-------|
| ASCII (0x20-0x7E) | 1 |
| CJK | 2 |
| Emoji | 2 (usually) |
| Control chars | 0 |
| Combining marks | 0 |
| Zero-width chars | 0 |

### 1.3 Height Measurement

```rust
/// Measure text height given max width (for wrapping)
pub fn measure_text_height(text: &str, max_width: usize) -> usize {
    if text.is_empty() || max_width == 0 {
        return 0;
    }

    let mut lines = 1;
    let mut current_width = 0;

    for paragraph in text.split('\n') {
        if paragraph.is_empty() {
            lines += 1;
            continue;
        }

        for ch in paragraph.chars() {
            let char_width = measure_char_width(ch);
            if char_width == 0 {
                continue;
            }

            if current_width + char_width > max_width && current_width > 0 {
                lines += 1;
                current_width = char_width;
            } else {
                current_width += char_width;
            }
        }

        current_width = 0;
    }

    lines
}
```

---

## 2. Text Wrapping

### 2.1 Wrap Modes

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextWrap {
    /// No wrapping - single line, may overflow
    None,
    /// Word wrap - break at word boundaries
    Wrap,
    /// Character wrap - break at any character
    Char,
}
```

### 2.2 Word Wrap Algorithm

```rust
pub fn wrap_text_word(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![];
    }

    let mut lines = Vec::new();

    for paragraph in text.split('\n') {
        if paragraph.is_empty() {
            lines.push(String::new());
            continue;
        }

        let mut current_line = String::new();
        let mut current_width = 0;

        for word in paragraph.split_whitespace() {
            let word_width = measure_text_width(word);

            if current_width == 0 {
                // First word on line
                if word_width <= max_width {
                    current_line = word.to_string();
                    current_width = word_width;
                } else {
                    // Word too long - character wrap
                    let (first, rest) = split_word_at_width(word, max_width);
                    lines.push(first);

                    // Handle remaining characters
                    let mut remaining = rest;
                    while measure_text_width(&remaining) > max_width {
                        let (chunk, new_rest) = split_word_at_width(&remaining, max_width);
                        lines.push(chunk);
                        remaining = new_rest;
                    }

                    if !remaining.is_empty() {
                        current_line = remaining;
                        current_width = measure_text_width(&current_line);
                    }
                }
            } else if current_width + 1 + word_width <= max_width {
                // Word fits with space
                current_line.push(' ');
                current_line.push_str(word);
                current_width += 1 + word_width;
            } else {
                // Word doesn't fit - new line
                lines.push(current_line);
                current_line = word.to_string();
                current_width = word_width;
            }
        }

        if !current_line.is_empty() || paragraph.is_empty() {
            lines.push(current_line);
        }
    }

    lines
}

fn split_word_at_width(word: &str, max_width: usize) -> (String, String) {
    let mut first = String::new();
    let mut width = 0;
    let mut char_indices = word.char_indices().peekable();

    while let Some((_, ch)) = char_indices.next() {
        let ch_width = measure_char_width(ch);
        if width + ch_width > max_width && width > 0 {
            break;
        }
        first.push(ch);
        width += ch_width;
    }

    let rest_start = first.len();
    let rest = word[rest_start..].to_string();

    (first, rest)
}
```

### 2.3 Character Wrap Algorithm

```rust
pub fn wrap_text_char(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![];
    }

    let mut lines = Vec::new();

    for paragraph in text.split('\n') {
        if paragraph.is_empty() {
            lines.push(String::new());
            continue;
        }

        let mut current_line = String::new();
        let mut current_width = 0;

        for ch in paragraph.chars() {
            let ch_width = measure_char_width(ch);

            if current_width + ch_width > max_width && current_width > 0 {
                lines.push(current_line);
                current_line = String::new();
                current_width = 0;
            }

            current_line.push(ch);
            current_width += ch_width;
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }
    }

    lines
}
```

---

## 3. Text Truncation

### 3.1 Truncation Modes

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextOverflow {
    /// Show overflow (no truncation)
    Visible,
    /// Clip at boundary
    Clip,
    /// Add ellipsis at end
    Ellipsis,
    /// Add ellipsis at start
    EllipsisStart,
    /// Add ellipsis in middle
    EllipsisMiddle,
}
```

### 3.2 Truncation Implementation

```rust
const ELLIPSIS: &str = "…";
const ELLIPSIS_WIDTH: usize = 1;

pub fn truncate_text(text: &str, max_width: usize, mode: TextOverflow) -> String {
    let text_width = measure_text_width(text);

    if text_width <= max_width {
        return text.to_string();
    }

    match mode {
        TextOverflow::Visible => text.to_string(),
        TextOverflow::Clip => truncate_at_width(text, max_width),
        TextOverflow::Ellipsis => truncate_end(text, max_width),
        TextOverflow::EllipsisStart => truncate_start(text, max_width),
        TextOverflow::EllipsisMiddle => truncate_middle(text, max_width),
    }
}

fn truncate_at_width(text: &str, max_width: usize) -> String {
    let mut result = String::new();
    let mut width = 0;

    for ch in text.chars() {
        let ch_width = measure_char_width(ch);
        if width + ch_width > max_width {
            break;
        }
        result.push(ch);
        width += ch_width;
    }

    result
}

fn truncate_end(text: &str, max_width: usize) -> String {
    if max_width < ELLIPSIS_WIDTH {
        return String::new();
    }

    let available = max_width - ELLIPSIS_WIDTH;
    let mut result = truncate_at_width(text, available);
    result.push_str(ELLIPSIS);
    result
}

fn truncate_start(text: &str, max_width: usize) -> String {
    if max_width < ELLIPSIS_WIDTH {
        return String::new();
    }

    let available = max_width - ELLIPSIS_WIDTH;

    // Find where to start
    let text_width = measure_text_width(text);
    let skip_width = text_width - available;

    let mut result = String::from(ELLIPSIS);
    let mut skipped = 0;

    for ch in text.chars() {
        let ch_width = measure_char_width(ch);
        if skipped < skip_width {
            skipped += ch_width;
        } else {
            result.push(ch);
        }
    }

    result
}

fn truncate_middle(text: &str, max_width: usize) -> String {
    if max_width < ELLIPSIS_WIDTH {
        return String::new();
    }

    let available = max_width - ELLIPSIS_WIDTH;
    let left_width = available / 2;
    let right_width = available - left_width;

    let left = truncate_at_width(text, left_width);

    // Get right portion
    let text_width = measure_text_width(text);
    let skip_width = text_width - right_width;

    let mut right = String::new();
    let mut skipped = 0;

    for ch in text.chars() {
        let ch_width = measure_char_width(ch);
        if skipped < skip_width {
            skipped += ch_width;
        } else {
            right.push(ch);
        }
    }

    format!("{}{}{}", left, ELLIPSIS, right)
}
```

---

## 4. Text Alignment

### 4.1 Alignment Modes

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}
```

### 4.2 Alignment Implementation

```rust
pub fn align_text(text: &str, width: usize, align: TextAlign) -> String {
    let text_width = measure_text_width(text);

    if text_width >= width {
        return text.to_string();
    }

    let padding = width - text_width;

    match align {
        TextAlign::Left => {
            let mut result = text.to_string();
            result.push_str(&" ".repeat(padding));
            result
        }
        TextAlign::Right => {
            let mut result = " ".repeat(padding);
            result.push_str(text);
            result
        }
        TextAlign::Center => {
            let left_pad = padding / 2;
            let right_pad = padding - left_pad;
            format!("{}{}{}", " ".repeat(left_pad), text, " ".repeat(right_pad))
        }
    }
}
```

---

## 5. Terminal Cursor

### 5.1 Cursor State

```rust
thread_local! {
    static TERMINAL_CURSOR: RefCell<TerminalCursor> = RefCell::new(TerminalCursor::new());
}

pub struct TerminalCursor {
    /// Current X position (0-indexed)
    pub x: Signal<usize>,
    /// Current Y position (0-indexed)
    pub y: Signal<usize>,
    /// Visibility
    pub visible: Signal<bool>,
    /// Cursor shape
    pub shape: Signal<CursorShape>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorShape {
    Block,
    Underline,
    Bar,
}

impl TerminalCursor {
    pub fn new() -> Self {
        Self {
            x: signal(0),
            y: signal(0),
            visible: signal(false),
            shape: signal(CursorShape::Block),
        }
    }
}
```

### 5.2 Cursor Control

```rust
/// Move terminal cursor to position
pub fn move_cursor(x: usize, y: usize) {
    TERMINAL_CURSOR.with(|c| {
        let cursor = c.borrow();
        cursor.x.set(x);
        cursor.y.set(y);
    });
}

/// Show/hide terminal cursor
pub fn set_cursor_visible(visible: bool) {
    TERMINAL_CURSOR.with(|c| {
        c.borrow().visible.set(visible);
    });
}

/// Set cursor shape
pub fn set_cursor_shape(shape: CursorShape) {
    TERMINAL_CURSOR.with(|c| {
        c.borrow().shape.set(shape);
    });
}
```

### 5.3 ANSI Cursor Codes

```rust
pub mod cursor_ansi {
    pub fn move_to(x: usize, y: usize) -> String {
        format!("\x1b[{};{}H", y + 1, x + 1)
    }

    pub const HIDE: &str = "\x1b[?25l";
    pub const SHOW: &str = "\x1b[?25h";

    pub fn shape(shape: CursorShape) -> &'static str {
        match shape {
            CursorShape::Block => "\x1b[2 q",
            CursorShape::Underline => "\x1b[4 q",
            CursorShape::Bar => "\x1b[6 q",
        }
    }
}
```

---

## 6. Drawn Cursor (Per-Component)

### 6.1 Drawn Cursor State

```rust
/// Cursor rendered as part of component (Input)
pub struct DrawnCursor {
    /// Character position in text
    pub position: Signal<usize>,
    /// Character to display (0 = inverse video block)
    pub char: Signal<u32>,
    /// Alternate character for blink "off" phase
    pub alt_char: Signal<u32>,
    /// Blink frequency (0 = no blink)
    pub blink_fps: Signal<u32>,
    /// Current visibility (derived from focus + blink)
    pub visible: Derived<bool>,
}
```

### 6.2 Cursor Styles

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawnCursorStyle {
    /// Block cursor with inverse video (default)
    Block,
    /// Vertical bar │
    Bar,
    /// Underline _
    Underline,
    /// Custom character
    Custom(char),
}

impl DrawnCursorStyle {
    pub fn char(&self) -> u32 {
        match self {
            Self::Block => 0,      // 0 = special inverse video
            Self::Bar => '│' as u32,
            Self::Underline => '_' as u32,
            Self::Custom(c) => *c as u32,
        }
    }
}
```

### 6.3 Cursor Blink System

```rust
thread_local! {
    /// Shared blink clocks by FPS
    static BLINK_CLOCKS: RefCell<HashMap<u32, BlinkClock>> = RefCell::new(HashMap::new());
}

struct BlinkClock {
    fps: u32,
    phase: Signal<bool>,  // true = visible, false = hidden
    subscribers: usize,
    interval: Option<IntervalHandle>,
}

impl BlinkClock {
    fn subscribe(&mut self) -> Signal<bool> {
        self.subscribers += 1;

        if self.subscribers == 1 {
            // Start interval
            let phase = self.phase.clone();
            let interval_ms = 500 / self.fps;  // 2 phases per cycle

            self.interval = Some(set_interval(move || {
                phase.set(!phase.get());
            }, interval_ms));
        }

        self.phase.clone()
    }

    fn unsubscribe(&mut self) {
        self.subscribers -= 1;

        if self.subscribers == 0 {
            if let Some(handle) = self.interval.take() {
                clear_interval(handle);
            }
        }
    }
}

/// Get blink phase for given FPS
pub fn get_blink_phase(fps: u32) -> Signal<bool> {
    BLINK_CLOCKS.with(|clocks| {
        let mut clocks = clocks.borrow_mut();
        let clock = clocks.entry(fps).or_insert_with(|| BlinkClock {
            fps,
            phase: signal(true),
            subscribers: 0,
            interval: None,
        });
        clock.subscribe()
    })
}
```

### 6.4 Setup Cursor Blink

```rust
pub fn setup_cursor_blink(index: usize, fps: u32) {
    let blink_phase = get_blink_phase(fps);
    let focused = focused_index_signal();

    // Visible = focused AND blink phase
    let visible = derived(move || {
        is_focused(index) && blink_phase.get()
    });

    INTERACTION.cursor_visible.bind_derived(index, visible);
}

pub fn cleanup_cursor_blink(index: usize, fps: u32) {
    BLINK_CLOCKS.with(|clocks| {
        if let Some(clock) = clocks.borrow_mut().get_mut(&fps) {
            clock.unsubscribe();
        }
    });
}
```

---

## 7. Cursor Movement

### 7.1 Movement Operations

```rust
/// Move cursor left
pub fn cursor_left(index: usize) {
    let pos = INTERACTION.cursor_position.get(index);
    if pos > 0 {
        INTERACTION.cursor_position.set(index, pos - 1);
    }
}

/// Move cursor right
pub fn cursor_right(index: usize) {
    let content = TEXT.text_content.get(index);
    let pos = INTERACTION.cursor_position.get(index);
    let max_pos = content.chars().count();

    if pos < max_pos {
        INTERACTION.cursor_position.set(index, pos + 1);
    }
}

/// Move cursor to start
pub fn cursor_home(index: usize) {
    INTERACTION.cursor_position.set(index, 0);
}

/// Move cursor to end
pub fn cursor_end(index: usize) {
    let content = TEXT.text_content.get(index);
    let len = content.chars().count();
    INTERACTION.cursor_position.set(index, len);
}

/// Move cursor by word
pub fn cursor_word_left(index: usize) {
    let content = TEXT.text_content.get(index);
    let pos = INTERACTION.cursor_position.get(index);

    if pos == 0 {
        return;
    }

    let chars: Vec<char> = content.chars().collect();
    let mut new_pos = pos - 1;

    // Skip whitespace
    while new_pos > 0 && chars[new_pos].is_whitespace() {
        new_pos -= 1;
    }

    // Skip word
    while new_pos > 0 && !chars[new_pos - 1].is_whitespace() {
        new_pos -= 1;
    }

    INTERACTION.cursor_position.set(index, new_pos);
}

pub fn cursor_word_right(index: usize) {
    let content = TEXT.text_content.get(index);
    let chars: Vec<char> = content.chars().collect();
    let pos = INTERACTION.cursor_position.get(index);
    let len = chars.len();

    if pos >= len {
        return;
    }

    let mut new_pos = pos;

    // Skip current word
    while new_pos < len && !chars[new_pos].is_whitespace() {
        new_pos += 1;
    }

    // Skip whitespace
    while new_pos < len && chars[new_pos].is_whitespace() {
        new_pos += 1;
    }

    INTERACTION.cursor_position.set(index, new_pos);
}
```

---

## 8. Text Selection

### 8.1 Selection State

```rust
/// Selection is stored as start/end indices
/// -1 = no selection
pub struct Selection {
    pub start: Signal<i32>,
    pub end: Signal<i32>,
}

impl Selection {
    pub fn new() -> Self {
        Self {
            start: signal(-1),
            end: signal(-1),
        }
    }

    pub fn is_active(&self) -> bool {
        self.start.get() >= 0 && self.end.get() >= 0
    }

    pub fn clear(&self) {
        self.start.set(-1);
        self.end.set(-1);
    }

    pub fn get_range(&self) -> Option<(usize, usize)> {
        let start = self.start.get();
        let end = self.end.get();

        if start < 0 || end < 0 {
            return None;
        }

        let (s, e) = if start <= end {
            (start as usize, end as usize)
        } else {
            (end as usize, start as usize)
        };

        Some((s, e))
    }
}
```

### 8.2 Selection Operations

```rust
/// Start selection at cursor
pub fn start_selection(index: usize) {
    let pos = INTERACTION.cursor_position.get(index);
    INTERACTION.selection_start.set(index, pos as i32);
    INTERACTION.selection_end.set(index, pos as i32);
}

/// Extend selection to cursor
pub fn extend_selection(index: usize) {
    let pos = INTERACTION.cursor_position.get(index);
    INTERACTION.selection_end.set(index, pos as i32);
}

/// Select all text
pub fn select_all(index: usize) {
    let content = TEXT.text_content.get(index);
    let len = content.chars().count();

    INTERACTION.selection_start.set(index, 0);
    INTERACTION.selection_end.set(index, len as i32);
}

/// Get selected text
pub fn get_selected_text(index: usize) -> Option<String> {
    let start = INTERACTION.selection_start.get(index);
    let end = INTERACTION.selection_end.get(index);

    if start < 0 || end < 0 {
        return None;
    }

    let (s, e) = if start <= end {
        (start as usize, end as usize)
    } else {
        (end as usize, start as usize)
    };

    let content = TEXT.text_content.get(index);
    let selected: String = content.chars().skip(s).take(e - s).collect();
    Some(selected)
}

/// Delete selected text
pub fn delete_selection(index: usize, value: &Signal<String>) {
    let start = INTERACTION.selection_start.get(index);
    let end = INTERACTION.selection_end.get(index);

    if start < 0 || end < 0 {
        return;
    }

    let (s, e) = if start <= end {
        (start as usize, end as usize)
    } else {
        (end as usize, start as usize)
    };

    let content = value.get();
    let chars: Vec<char> = content.chars().collect();

    let new_content: String = chars[..s]
        .iter()
        .chain(chars[e..].iter())
        .collect();

    value.set(new_content);
    INTERACTION.cursor_position.set(index, s);
    clear_selection(index);
}

pub fn clear_selection(index: usize) {
    INTERACTION.selection_start.set(index, -1);
    INTERACTION.selection_end.set(index, -1);
}
```

---

## 9. Index ↔ Column Conversion

### 9.1 Character Index to Display Column

```rust
/// Convert character index to display column
pub fn index_to_column(text: &str, char_index: usize) -> usize {
    text.chars()
        .take(char_index)
        .map(|ch| measure_char_width(ch))
        .sum()
}
```

### 9.2 Display Column to Character Index

```rust
/// Convert display column to character index
pub fn column_to_index(text: &str, column: usize) -> usize {
    let mut current_column = 0;
    let mut char_index = 0;

    for ch in text.chars() {
        if current_column >= column {
            break;
        }
        current_column += measure_char_width(ch);
        char_index += 1;
    }

    char_index
}
```

---

## 10. Horizontal Scroll (Input)

### 10.1 Keep Cursor Visible

```rust
/// Calculate display offset to keep cursor visible
pub fn calculate_input_scroll(
    content: &str,
    cursor_pos: usize,
    display_width: usize,
) -> (String, usize) {
    let cursor_column = index_to_column(content, cursor_pos);
    let content_width = measure_text_width(content);

    if content_width <= display_width {
        // No scroll needed
        return (content.to_string(), cursor_column);
    }

    let mut scroll_offset = 0;

    // Ensure cursor is visible
    if cursor_column >= display_width {
        scroll_offset = cursor_column - display_width + 1;
    }

    // Extract visible portion
    let visible = extract_visible_text(content, scroll_offset, display_width);
    let visible_cursor = cursor_column - scroll_offset;

    (visible, visible_cursor)
}

fn extract_visible_text(content: &str, start_column: usize, width: usize) -> String {
    let mut result = String::new();
    let mut current_column = 0;
    let end_column = start_column + width;

    for ch in content.chars() {
        let ch_width = measure_char_width(ch);

        if current_column >= end_column {
            break;
        }

        if current_column + ch_width > start_column {
            result.push(ch);
        }

        current_column += ch_width;
    }

    result
}
```

---

## 11. Rendering Cursor

### 11.1 Block Cursor (Inverse Video)

```rust
fn render_cursor_block(
    buffer: &mut FrameBuffer,
    x: usize,
    y: usize,
    ch: char,
    fg: Rgba,
    bg: Rgba,
) {
    // Inverse video: swap fg and bg
    buffer.set(x, y, Cell {
        char: ch as u32,
        fg: bg,    // Swap
        bg: fg,    // Swap
        attrs: CellAttrs::empty(),
    });
}
```

### 11.2 Bar/Underline Cursor

```rust
fn render_cursor_char(
    buffer: &mut FrameBuffer,
    x: usize,
    y: usize,
    content_char: char,
    cursor_char: char,
    fg: Rgba,
    bg: Rgba,
) {
    // For bar cursor, overlay on content
    // For underline, could use combining char or separate position

    // Simple approach: show cursor char after content
    buffer.set(x, y, Cell {
        char: content_char as u32,
        fg,
        bg,
        attrs: CellAttrs::empty(),
    });

    // Draw cursor at same position with different style
    // This depends on how the cursor overlays
}
```

---

## 12. Password Masking

```rust
/// Mask password with character
pub fn mask_password(content: &str, mask_char: char) -> String {
    content.chars().map(|_| mask_char).collect()
}

// Usage in Input
if is_password {
    let masked = mask_password(&content, mask_char);
    render_text_content(buffer, &masked, ...);
}
```

---

## 13. Hidden Automatic Behaviors

### 13.1 Cursor Position Clamping
Cursor position auto-clamped when text shortens.

### 13.2 Horizontal Scroll
Input auto-scrolls to keep cursor visible.

### 13.3 Empty Placeholder
Show placeholder when value is empty.

### 13.4 Password Masking
Content masked with repeated mask char.

### 13.5 Blink Only When Focused
Cursor static (always visible) when unfocused.

### 13.6 Wide Char at Boundary
Don't draw partial wide characters at boundary.

### 13.7 ANSI Stripping
Strip ANSI codes before measuring.

### 13.8 Max Length Enforcement
Silent rejection when max length exceeded.

### 13.9 Newline Preservation
Empty lines maintained in wrapped text.

### 13.10 Block Cursor Special
Inverse video with character under cursor.

---

## 14. Module Structure

```
crates/tui/src/
├── utils/
│   └── text.rs         # Measurement, wrapping, truncation
├── state/
│   ├── cursor.rs       # Terminal cursor
│   ├── drawn_cursor.rs # Per-component cursor
│   └── blink.rs        # Shared blink clocks
└── engine/
    └── arrays/
        └── text.rs     # Text content arrays
```

---

## 15. Dependencies

```toml
unicode-width = "0.1"
unicode-segmentation = "1.10"  # For grapheme clusters
```

---

## 16. Summary

The text & cursor system provides:

✅ **Unicode Measurement**: Proper CJK/emoji width
✅ **Word Wrap**: Break at word boundaries
✅ **Character Wrap**: Break at any character
✅ **3 Truncation Modes**: End, start, middle ellipsis
✅ **3 Alignment Modes**: Left, center, right
✅ **Terminal Cursor**: System cursor control
✅ **Drawn Cursor**: Per-component cursors
✅ **Shared Blink Clock**: Efficient multi-cursor blink
✅ **3 Cursor Styles**: Block, bar, underline
✅ **Selection Support**: Start/end range tracking
✅ **Horizontal Scroll**: Auto-scroll in Input
✅ **Password Masking**: Custom mask character
✅ **Index ↔ Column Conversion**: Unicode-aware
