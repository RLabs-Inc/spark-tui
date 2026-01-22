# Rendering Pipeline Specification

## Overview

This specification documents the terminal rendering pipeline, covering the complete flow from reactive state to terminal output: FrameBuffer construction, Cell structure, ANSI escape code generation, differential rendering, and output modes.

**Source Files Analyzed:**
- `/Users/rusty/Documents/Projects/TUI/tui/src/pipeline/frameBuffer.ts` - Frame buffer construction
- `/Users/rusty/Documents/Projects/TUI/tui/src/renderer/buffer.ts` - Cell buffer
- `/Users/rusty/Documents/Projects/TUI/tui/src/renderer/ansi.ts` - ANSI escape codes
- `/Users/rusty/Documents/Projects/TUI/tui/src/renderer/output.ts` - Output handling
- `/Users/rusty/Documents/Projects/TUI/tui/src/renderer/append-region.ts` - Append mode rendering

---

## 1. Pipeline Architecture

### 1.1 Complete Flow

```
Component State (Signals)
        ↓
  Layout Derived
        ↓
 ComputedLayout
        ↓
FrameBuffer Derived
        ↓
   FrameBuffer
        ↓
 Differential Render
        ↓
   ANSI Escape Codes
        ↓
  Terminal Output
```

### 1.2 The ONE Effect Pattern

```rust
// Single effect drives ALL rendering
effect(move || {
    // 1. Read layout (creates dependency)
    let layout = layout_derived.get();

    // 2. Build frame buffer (creates dependencies on visual state)
    let buffer = build_frame_buffer(&layout);

    // 3. Diff against previous frame
    let diff = diff_buffers(&previous_buffer, &buffer);

    // 4. Generate ANSI and write to terminal
    output_diff(&diff);

    // 5. Store for next diff
    previous_buffer = buffer;
});
```

---

## 2. Core Data Structures

### 2.1 RGBA Color

```rust
/// RGBA color with special terminal values
/// r = -1: Terminal default color
/// r = -2, g = index: ANSI 256-color palette
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgba {
    pub r: i16,
    pub g: i16,
    pub b: i16,
    pub a: i16, // 0-255, 255 = fully opaque
}

impl Rgba {
    /// Terminal default color (inherits from terminal theme)
    pub const DEFAULT: Self = Self { r: -1, g: 0, b: 0, a: 255 };

    /// ANSI 256-color palette
    pub fn ansi(index: u8) -> Self {
        Self { r: -2, g: index as i16, b: 0, a: 255 }
    }

    /// True RGB color
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r: r as i16, g: g as i16, b: b as i16, a: 255 }
    }

    /// With alpha
    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r: r as i16, g: g as i16, b: b as i16, a: a as i16 }
    }

    /// Check if terminal default
    pub fn is_default(&self) -> bool {
        self.r == -1
    }

    /// Check if ANSI 256
    pub fn is_ansi(&self) -> bool {
        self.r == -2
    }

    /// Get ANSI index (only valid if is_ansi())
    pub fn ansi_index(&self) -> u8 {
        self.g as u8
    }
}
```

### 2.2 Cell Attributes

```rust
bitflags! {
    /// Text styling attributes as bitfield
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct CellAttrs: u8 {
        const BOLD          = 0b0000_0001;
        const DIM           = 0b0000_0010;
        const ITALIC        = 0b0000_0100;
        const UNDERLINE     = 0b0000_1000;
        const BLINK         = 0b0001_0000;
        const INVERSE       = 0b0010_0000;
        const HIDDEN        = 0b0100_0000;
        const STRIKETHROUGH = 0b1000_0000;
    }
}

impl Default for CellAttrs {
    fn default() -> Self {
        CellAttrs::empty()
    }
}
```

### 2.3 Cell Structure

```rust
/// A single terminal cell
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    /// Unicode codepoint (0 = wide char continuation)
    pub char: u32,
    /// Foreground color
    pub fg: Rgba,
    /// Background color
    pub bg: Rgba,
    /// Text attributes
    pub attrs: CellAttrs,
}

impl Cell {
    /// Empty cell (space with default colors)
    pub const EMPTY: Self = Self {
        char: ' ' as u32,
        fg: Rgba::DEFAULT,
        bg: Rgba::DEFAULT,
        attrs: CellAttrs::empty(),
    };

    /// Wide character continuation marker
    pub const WIDE_CONTINUATION: Self = Self {
        char: 0,
        fg: Rgba::DEFAULT,
        bg: Rgba::DEFAULT,
        attrs: CellAttrs::empty(),
    };

    /// Check if this is a wide char continuation
    pub fn is_continuation(&self) -> bool {
        self.char == 0
    }

    /// Get character (returns space for continuations)
    pub fn character(&self) -> char {
        if self.char == 0 {
            ' '
        } else {
            char::from_u32(self.char).unwrap_or(' ')
        }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::EMPTY
    }
}
```

### 2.4 FrameBuffer

```rust
/// 2D grid of cells representing the terminal screen
pub struct FrameBuffer {
    /// Cell data in row-major order
    cells: Vec<Cell>,
    /// Buffer width
    width: usize,
    /// Buffer height
    height: usize,
}

impl FrameBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            cells: vec![Cell::EMPTY; width * height],
            width,
            height,
        }
    }

    /// Get cell at position (returns None if out of bounds)
    pub fn get(&self, x: usize, y: usize) -> Option<&Cell> {
        if x < self.width && y < self.height {
            Some(&self.cells[y * self.width + x])
        } else {
            None
        }
    }

    /// Get mutable cell at position
    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut Cell> {
        if x < self.width && y < self.height {
            Some(&mut self.cells[y * self.width + x])
        } else {
            None
        }
    }

    /// Set cell at position
    pub fn set(&mut self, x: usize, y: usize, cell: Cell) {
        if x < self.width && y < self.height {
            self.cells[y * self.width + x] = cell;
        }
    }

    /// Fill rectangle with cell
    pub fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, cell: Cell) {
        for row in y..(y + h).min(self.height) {
            for col in x..(x + w).min(self.width) {
                self.cells[row * self.width + col] = cell;
            }
        }
    }

    /// Draw text at position with styling
    pub fn draw_text(
        &mut self,
        x: usize,
        y: usize,
        text: &str,
        fg: Rgba,
        bg: Rgba,
        attrs: CellAttrs,
    ) {
        use unicode_width::UnicodeWidthChar;

        let mut col = x;
        for ch in text.chars() {
            if col >= self.width {
                break;
            }

            let char_width = UnicodeWidthChar::width(ch).unwrap_or(0);

            if char_width == 0 {
                continue; // Skip zero-width chars
            }

            // Draw the character
            self.set(col, y, Cell {
                char: ch as u32,
                fg,
                bg,
                attrs,
            });

            // Mark continuation cells for wide characters
            if char_width == 2 && col + 1 < self.width {
                self.set(col + 1, y, Cell {
                    char: 0, // Continuation marker
                    fg,
                    bg,
                    attrs,
                });
            }

            col += char_width;
        }
    }

    /// Draw horizontal line
    pub fn draw_hline(&mut self, x: usize, y: usize, len: usize, ch: char, fg: Rgba, bg: Rgba) {
        for col in x..(x + len).min(self.width) {
            self.set(col, y, Cell {
                char: ch as u32,
                fg,
                bg,
                attrs: CellAttrs::empty(),
            });
        }
    }

    /// Draw vertical line
    pub fn draw_vline(&mut self, x: usize, y: usize, len: usize, ch: char, fg: Rgba, bg: Rgba) {
        for row in y..(y + len).min(self.height) {
            self.set(x, row, Cell {
                char: ch as u32,
                fg,
                bg,
                attrs: CellAttrs::empty(),
            });
        }
    }

    /// Draw box border
    pub fn draw_border(
        &mut self,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
        style: &BorderChars,
        fg: Rgba,
        bg: Rgba,
    ) {
        if w < 2 || h < 2 {
            return;
        }

        // Corners
        self.set(x, y, Cell { char: style.top_left as u32, fg, bg, attrs: CellAttrs::empty() });
        self.set(x + w - 1, y, Cell { char: style.top_right as u32, fg, bg, attrs: CellAttrs::empty() });
        self.set(x, y + h - 1, Cell { char: style.bottom_left as u32, fg, bg, attrs: CellAttrs::empty() });
        self.set(x + w - 1, y + h - 1, Cell { char: style.bottom_right as u32, fg, bg, attrs: CellAttrs::empty() });

        // Edges
        self.draw_hline(x + 1, y, w - 2, style.horizontal, fg, bg);
        self.draw_hline(x + 1, y + h - 1, w - 2, style.horizontal, fg, bg);
        self.draw_vline(x, y + 1, h - 2, style.vertical, fg, bg);
        self.draw_vline(x + w - 1, y + 1, h - 2, style.vertical, fg, bg);
    }

    /// Clear entire buffer
    pub fn clear(&mut self) {
        self.cells.fill(Cell::EMPTY);
    }

    /// Resize buffer (clears content)
    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.cells.resize(width * height, Cell::EMPTY);
        self.cells.fill(Cell::EMPTY);
    }
}
```

### 2.5 Border Characters

```rust
/// Border style character set
#[derive(Debug, Clone, Copy)]
pub struct BorderChars {
    pub horizontal: char,
    pub vertical: char,
    pub top_left: char,
    pub top_right: char,
    pub bottom_left: char,
    pub bottom_right: char,
}

impl BorderChars {
    /// Single line border (─│┌┐└┘)
    pub const SINGLE: Self = Self {
        horizontal: '─',
        vertical: '│',
        top_left: '┌',
        top_right: '┐',
        bottom_left: '└',
        bottom_right: '┘',
    };

    /// Double line border (═║╔╗╚╝)
    pub const DOUBLE: Self = Self {
        horizontal: '═',
        vertical: '║',
        top_left: '╔',
        top_right: '╗',
        bottom_left: '╚',
        bottom_right: '╝',
    };

    /// Rounded border (─│╭╮╰╯)
    pub const ROUNDED: Self = Self {
        horizontal: '─',
        vertical: '│',
        top_left: '╭',
        top_right: '╮',
        bottom_left: '╰',
        bottom_right: '╯',
    };

    /// Bold border (━┃┏┓┗┛)
    pub const BOLD: Self = Self {
        horizontal: '━',
        vertical: '┃',
        top_left: '┏',
        top_right: '┓',
        bottom_left: '┗',
        bottom_right: '┛',
    };

    /// ASCII border (-|++++)
    pub const ASCII: Self = Self {
        horizontal: '-',
        vertical: '|',
        top_left: '+',
        top_right: '+',
        bottom_left: '+',
        bottom_right: '+',
    };
}
```

---

## 3. FrameBuffer Construction

### 3.1 Recursive Rendering

```rust
/// Build frame buffer from computed layout
pub fn build_frame_buffer(
    layout: &ComputedLayout,
    arrays: &ComponentArrays,
    root_index: usize,
    width: usize,
    height: usize,
) -> FrameBuffer {
    let mut buffer = FrameBuffer::new(width, height);

    render_component(
        &mut buffer,
        root_index,
        layout,
        arrays,
        None, // No parent clip
        0,    // No parent scroll X
        0,    // No parent scroll Y
    );

    buffer
}

fn render_component(
    buffer: &mut FrameBuffer,
    index: usize,
    layout: &ComputedLayout,
    arrays: &ComponentArrays,
    parent_clip: Option<ClipRect>,
    parent_scroll_x: i32,
    parent_scroll_y: i32,
) {
    // Skip invisible components
    if !arrays.visible.get(index) {
        return;
    }

    // Get computed position and size
    let x = (layout.x[index] - parent_scroll_x) as usize;
    let y = (layout.y[index] - parent_scroll_y) as usize;
    let w = layout.width[index] as usize;
    let h = layout.height[index] as usize;

    // Calculate clip rect (intersection with parent)
    let clip = calculate_clip(x, y, w, h, parent_clip);

    if clip.is_empty() {
        return; // Fully clipped
    }

    // Get colors (with inheritance)
    let fg = get_inherited_fg(index, arrays);
    let bg = get_inherited_bg(index, arrays);
    let attrs = get_cell_attrs(index, arrays);

    // Fill background
    if !bg.is_default() {
        for row in clip.y..clip.y + clip.h {
            for col in clip.x..clip.x + clip.w {
                if let Some(cell) = buffer.get_mut(col, row) {
                    cell.bg = bg;
                }
            }
        }
    }

    // Draw borders
    if has_border(index, arrays) {
        draw_component_border(buffer, index, x, y, w, h, &clip, arrays);
    }

    // Calculate content area (inside border/padding)
    let content = get_content_rect(index, x, y, w, h, arrays);

    // Render based on component type
    match arrays.component_type.get(index) {
        ComponentType::Text => {
            render_text_content(buffer, index, &content, &clip, fg, bg, attrs, arrays);
        }
        ComponentType::Input => {
            render_input_content(buffer, index, &content, &clip, fg, bg, attrs, arrays);
        }
        ComponentType::Box => {
            // Box has no content, just children
        }
    }

    // Render children
    let is_scrollable = layout.scrollable[index];
    let scroll_x = if is_scrollable { arrays.scroll_offset_x.get(index) } else { 0 };
    let scroll_y = if is_scrollable { arrays.scroll_offset_y.get(index) } else { 0 };

    // Child clip is the content area
    let child_clip = Some(ClipRect {
        x: content.x,
        y: content.y,
        w: content.w,
        h: content.h,
    });

    for child in children(index, arrays) {
        render_component(
            buffer,
            child,
            layout,
            arrays,
            child_clip,
            parent_scroll_x + scroll_x,
            parent_scroll_y + scroll_y,
        );
    }
}
```

### 3.2 Clipping System

```rust
#[derive(Debug, Clone, Copy)]
pub struct ClipRect {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

impl ClipRect {
    pub fn is_empty(&self) -> bool {
        self.w == 0 || self.h == 0
    }

    /// Intersect with another rect
    pub fn intersect(&self, other: &ClipRect) -> ClipRect {
        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = (self.x + self.w).min(other.x + other.w);
        let y2 = (self.y + self.h).min(other.y + other.h);

        if x2 > x1 && y2 > y1 {
            ClipRect {
                x: x1,
                y: y1,
                w: x2 - x1,
                h: y2 - y1,
            }
        } else {
            ClipRect { x: 0, y: 0, w: 0, h: 0 }
        }
    }
}

fn calculate_clip(
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    parent_clip: Option<ClipRect>,
) -> ClipRect {
    let component_rect = ClipRect { x, y, w, h };

    match parent_clip {
        Some(parent) => component_rect.intersect(&parent),
        None => component_rect,
    }
}
```

### 3.3 Color Inheritance

```rust
/// Get foreground color with inheritance
fn get_inherited_fg(index: usize, arrays: &ComponentArrays) -> Rgba {
    let fg = arrays.fg.get(index);
    if !fg.is_default() {
        return fg;
    }

    // Walk parent chain
    let mut parent = arrays.parent_index.get(index);
    while parent >= 0 {
        let parent_fg = arrays.fg.get(parent as usize);
        if !parent_fg.is_default() {
            return parent_fg;
        }
        parent = arrays.parent_index.get(parent as usize);
    }

    Rgba::DEFAULT
}

/// Get background color with opacity blending
fn get_inherited_bg(index: usize, arrays: &ComponentArrays) -> Rgba {
    let bg = arrays.bg.get(index);
    let opacity = arrays.opacity.get(index);

    // Apply opacity
    let mut result = if bg.a < 255 || opacity < 1.0 {
        Rgba {
            r: bg.r,
            g: bg.g,
            b: bg.b,
            a: ((bg.a as f32 * opacity) as i16).min(255),
        }
    } else {
        bg
    };

    // Blend with parent if semi-transparent
    if result.a < 255 {
        let parent = arrays.parent_index.get(index);
        if parent >= 0 {
            let parent_bg = get_inherited_bg(parent as usize, arrays);
            result = blend_colors(parent_bg, result);
        }
    }

    result
}

/// Alpha blend foreground over background
fn blend_colors(bg: Rgba, fg: Rgba) -> Rgba {
    if fg.a == 255 {
        return fg;
    }
    if fg.a == 0 {
        return bg;
    }

    let alpha = fg.a as f32 / 255.0;
    let inv_alpha = 1.0 - alpha;

    Rgba {
        r: ((fg.r as f32 * alpha + bg.r as f32 * inv_alpha) as i16).clamp(0, 255),
        g: ((fg.g as f32 * alpha + bg.g as f32 * inv_alpha) as i16).clamp(0, 255),
        b: ((fg.b as f32 * alpha + bg.b as f32 * inv_alpha) as i16).clamp(0, 255),
        a: 255,
    }
}
```

---

## 4. ANSI Escape Codes

### 4.1 ANSI Module

```rust
pub mod ansi {
    /// Clear screen
    pub const CLEAR_SCREEN: &str = "\x1b[2J";

    /// Clear to end of line
    pub const CLEAR_LINE: &str = "\x1b[K";

    /// Move cursor to position (1-indexed)
    pub fn cursor_to(x: usize, y: usize) -> String {
        format!("\x1b[{};{}H", y + 1, x + 1)
    }

    /// Move cursor relative
    pub fn cursor_up(n: usize) -> String {
        format!("\x1b[{}A", n)
    }

    pub fn cursor_down(n: usize) -> String {
        format!("\x1b[{}B", n)
    }

    pub fn cursor_forward(n: usize) -> String {
        format!("\x1b[{}C", n)
    }

    pub fn cursor_back(n: usize) -> String {
        format!("\x1b[{}D", n)
    }

    /// Hide/show cursor
    pub const CURSOR_HIDE: &str = "\x1b[?25l";
    pub const CURSOR_SHOW: &str = "\x1b[?25h";

    /// Save/restore cursor position
    pub const CURSOR_SAVE: &str = "\x1b[s";
    pub const CURSOR_RESTORE: &str = "\x1b[u";

    /// Reset all attributes
    pub const RESET: &str = "\x1b[0m";

    /// Foreground color (RGB)
    pub fn fg_rgb(r: u8, g: u8, b: u8) -> String {
        format!("\x1b[38;2;{};{};{}m", r, g, b)
    }

    /// Foreground color (256 palette)
    pub fn fg_256(index: u8) -> String {
        format!("\x1b[38;5;{}m", index)
    }

    /// Default foreground
    pub const FG_DEFAULT: &str = "\x1b[39m";

    /// Background color (RGB)
    pub fn bg_rgb(r: u8, g: u8, b: u8) -> String {
        format!("\x1b[48;2;{};{};{}m", r, g, b)
    }

    /// Background color (256 palette)
    pub fn bg_256(index: u8) -> String {
        format!("\x1b[48;5;{}m", index)
    }

    /// Default background
    pub const BG_DEFAULT: &str = "\x1b[49m";

    /// Text attributes
    pub const BOLD_ON: &str = "\x1b[1m";
    pub const BOLD_OFF: &str = "\x1b[22m";
    pub const DIM_ON: &str = "\x1b[2m";
    pub const DIM_OFF: &str = "\x1b[22m";
    pub const ITALIC_ON: &str = "\x1b[3m";
    pub const ITALIC_OFF: &str = "\x1b[23m";
    pub const UNDERLINE_ON: &str = "\x1b[4m";
    pub const UNDERLINE_OFF: &str = "\x1b[24m";
    pub const BLINK_ON: &str = "\x1b[5m";
    pub const BLINK_OFF: &str = "\x1b[25m";
    pub const INVERSE_ON: &str = "\x1b[7m";
    pub const INVERSE_OFF: &str = "\x1b[27m";
    pub const HIDDEN_ON: &str = "\x1b[8m";
    pub const HIDDEN_OFF: &str = "\x1b[28m";
    pub const STRIKETHROUGH_ON: &str = "\x1b[9m";
    pub const STRIKETHROUGH_OFF: &str = "\x1b[29m";

    /// Alternate screen buffer
    pub const ALT_SCREEN_ENTER: &str = "\x1b[?1049h";
    pub const ALT_SCREEN_EXIT: &str = "\x1b[?1049l";

    /// Mouse tracking
    pub const MOUSE_ENABLE: &str = "\x1b[?1000h\x1b[?1002h\x1b[?1006h";
    pub const MOUSE_DISABLE: &str = "\x1b[?1000l\x1b[?1002l\x1b[?1006l";

    /// Synchronized output (prevents flicker)
    pub const SYNC_START: &str = "\x1b[?2026h";
    pub const SYNC_END: &str = "\x1b[?2026l";
}
```

### 4.2 Color to ANSI

```rust
fn fg_to_ansi(color: &Rgba) -> String {
    if color.is_default() {
        ansi::FG_DEFAULT.to_string()
    } else if color.is_ansi() {
        ansi::fg_256(color.ansi_index())
    } else {
        ansi::fg_rgb(color.r as u8, color.g as u8, color.b as u8)
    }
}

fn bg_to_ansi(color: &Rgba) -> String {
    if color.is_default() {
        ansi::BG_DEFAULT.to_string()
    } else if color.is_ansi() {
        ansi::bg_256(color.ansi_index())
    } else {
        ansi::bg_rgb(color.r as u8, color.g as u8, color.b as u8)
    }
}

fn attrs_to_ansi(attrs: CellAttrs, prev_attrs: CellAttrs) -> String {
    let mut result = String::new();

    // Bold
    if attrs.contains(CellAttrs::BOLD) != prev_attrs.contains(CellAttrs::BOLD) {
        result.push_str(if attrs.contains(CellAttrs::BOLD) {
            ansi::BOLD_ON
        } else {
            ansi::BOLD_OFF
        });
    }

    // Dim
    if attrs.contains(CellAttrs::DIM) != prev_attrs.contains(CellAttrs::DIM) {
        result.push_str(if attrs.contains(CellAttrs::DIM) {
            ansi::DIM_ON
        } else {
            ansi::DIM_OFF
        });
    }

    // Italic
    if attrs.contains(CellAttrs::ITALIC) != prev_attrs.contains(CellAttrs::ITALIC) {
        result.push_str(if attrs.contains(CellAttrs::ITALIC) {
            ansi::ITALIC_ON
        } else {
            ansi::ITALIC_OFF
        });
    }

    // Underline
    if attrs.contains(CellAttrs::UNDERLINE) != prev_attrs.contains(CellAttrs::UNDERLINE) {
        result.push_str(if attrs.contains(CellAttrs::UNDERLINE) {
            ansi::UNDERLINE_ON
        } else {
            ansi::UNDERLINE_OFF
        });
    }

    // Blink
    if attrs.contains(CellAttrs::BLINK) != prev_attrs.contains(CellAttrs::BLINK) {
        result.push_str(if attrs.contains(CellAttrs::BLINK) {
            ansi::BLINK_ON
        } else {
            ansi::BLINK_OFF
        });
    }

    // Inverse
    if attrs.contains(CellAttrs::INVERSE) != prev_attrs.contains(CellAttrs::INVERSE) {
        result.push_str(if attrs.contains(CellAttrs::INVERSE) {
            ansi::INVERSE_ON
        } else {
            ansi::INVERSE_OFF
        });
    }

    // Strikethrough
    if attrs.contains(CellAttrs::STRIKETHROUGH) != prev_attrs.contains(CellAttrs::STRIKETHROUGH) {
        result.push_str(if attrs.contains(CellAttrs::STRIKETHROUGH) {
            ansi::STRIKETHROUGH_ON
        } else {
            ansi::STRIKETHROUGH_OFF
        });
    }

    result
}
```

---

## 5. Differential Rendering

### 5.1 Stateful Cell Renderer

```rust
/// Tracks current terminal state for minimal output
pub struct StatefulCellRenderer {
    current_fg: Rgba,
    current_bg: Rgba,
    current_attrs: CellAttrs,
    current_x: usize,
    current_y: usize,
    output: String,
}

impl StatefulCellRenderer {
    pub fn new() -> Self {
        Self {
            current_fg: Rgba::DEFAULT,
            current_bg: Rgba::DEFAULT,
            current_attrs: CellAttrs::empty(),
            current_x: 0,
            current_y: 0,
            output: String::with_capacity(16384),
        }
    }

    /// Move cursor to position (only if needed)
    pub fn move_to(&mut self, x: usize, y: usize) {
        if self.current_x != x || self.current_y != y {
            // Optimization: sequential cells don't need cursor moves
            if self.current_y == y && self.current_x + 1 == x {
                // Cursor advances automatically
            } else {
                self.output.push_str(&ansi::cursor_to(x, y));
            }
            self.current_x = x;
            self.current_y = y;
        }
    }

    /// Set foreground (only if changed)
    pub fn set_fg(&mut self, fg: &Rgba) {
        if self.current_fg != *fg {
            self.output.push_str(&fg_to_ansi(fg));
            self.current_fg = *fg;
        }
    }

    /// Set background (only if changed)
    pub fn set_bg(&mut self, bg: &Rgba) {
        if self.current_bg != *bg {
            self.output.push_str(&bg_to_ansi(bg));
            self.current_bg = *bg;
        }
    }

    /// Set attributes (only changed bits)
    pub fn set_attrs(&mut self, attrs: CellAttrs) {
        if self.current_attrs != attrs {
            self.output.push_str(&attrs_to_ansi(attrs, self.current_attrs));
            self.current_attrs = attrs;
        }
    }

    /// Write cell at position
    pub fn write_cell(&mut self, x: usize, y: usize, cell: &Cell) {
        if cell.is_continuation() {
            return; // Skip wide char continuations
        }

        self.move_to(x, y);
        self.set_fg(&cell.fg);
        self.set_bg(&cell.bg);
        self.set_attrs(cell.attrs);

        // Write character
        let ch = cell.character();
        self.output.push(ch);

        // Advance cursor
        let width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(1);
        self.current_x += width;
    }

    /// Finish and get output
    pub fn finish(mut self) -> String {
        self.output.push_str(ansi::RESET);
        self.output
    }

    /// Reset state for new frame
    pub fn reset(&mut self) {
        self.output.clear();
        self.current_fg = Rgba::DEFAULT;
        self.current_bg = Rgba::DEFAULT;
        self.current_attrs = CellAttrs::empty();
        self.current_x = 0;
        self.current_y = 0;
    }
}
```

### 5.2 Frame Diffing

```rust
/// Compare two frames and render differences
pub fn render_diff(
    prev: &FrameBuffer,
    curr: &FrameBuffer,
    renderer: &mut StatefulCellRenderer,
) {
    // Ensure same dimensions
    assert_eq!(prev.width, curr.width);
    assert_eq!(prev.height, curr.height);

    for y in 0..curr.height {
        for x in 0..curr.width {
            let prev_cell = prev.get(x, y).unwrap();
            let curr_cell = curr.get(x, y).unwrap();

            // Only render if changed
            if prev_cell != curr_cell {
                renderer.write_cell(x, y, curr_cell);
            }
        }
    }
}

/// Full render (no diffing)
pub fn render_full(buffer: &FrameBuffer, renderer: &mut StatefulCellRenderer) {
    for y in 0..buffer.height {
        for x in 0..buffer.width {
            let cell = buffer.get(x, y).unwrap();
            renderer.write_cell(x, y, cell);
        }
    }
}
```

---

## 6. Output Modes

### 6.1 Fullscreen Mode

```rust
pub struct FullscreenRenderer {
    prev_buffer: FrameBuffer,
    cell_renderer: StatefulCellRenderer,
}

impl FullscreenRenderer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            prev_buffer: FrameBuffer::new(width, height),
            cell_renderer: StatefulCellRenderer::new(),
        }
    }

    pub fn enter(&self) {
        print!("{}", ansi::ALT_SCREEN_ENTER);
        print!("{}", ansi::CURSOR_HIDE);
        print!("{}", ansi::MOUSE_ENABLE);
        print!("{}", ansi::CLEAR_SCREEN);
        std::io::stdout().flush().unwrap();
    }

    pub fn exit(&self) {
        print!("{}", ansi::MOUSE_DISABLE);
        print!("{}", ansi::CURSOR_SHOW);
        print!("{}", ansi::ALT_SCREEN_EXIT);
        std::io::stdout().flush().unwrap();
    }

    pub fn render(&mut self, buffer: &FrameBuffer) {
        // Synchronized output prevents flicker
        print!("{}", ansi::SYNC_START);

        self.cell_renderer.reset();
        render_diff(&self.prev_buffer, buffer, &mut self.cell_renderer);

        print!("{}", self.cell_renderer.finish());
        print!("{}", ansi::SYNC_END);

        std::io::stdout().flush().unwrap();

        // Store for next diff
        self.prev_buffer = buffer.clone();
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.prev_buffer = FrameBuffer::new(width, height);
    }
}
```

### 6.2 Inline Mode

```rust
pub struct InlineRenderer {
    start_y: usize,
    height: usize,
}

impl InlineRenderer {
    pub fn new(height: usize) -> Self {
        Self {
            start_y: 0, // Will be set on first render
            height,
        }
    }

    pub fn render(&mut self, buffer: &FrameBuffer) {
        let mut output = String::new();

        // Save cursor
        output.push_str(ansi::CURSOR_SAVE);

        // Render each line
        let mut renderer = StatefulCellRenderer::new();
        for y in 0..buffer.height.min(self.height) {
            output.push_str(&ansi::cursor_to(0, self.start_y + y));
            output.push_str(ansi::CLEAR_LINE);

            for x in 0..buffer.width {
                let cell = buffer.get(x, y).unwrap();
                renderer.write_cell(x, self.start_y + y, cell);
            }
        }

        // Restore cursor
        output.push_str(ansi::CURSOR_RESTORE);

        print!("{}{}", output, renderer.finish());
        std::io::stdout().flush().unwrap();
    }
}
```

### 6.3 Append Mode (History)

```rust
pub struct AppendRenderer {
    written_lines: usize,
}

impl AppendRenderer {
    pub fn new() -> Self {
        Self { written_lines: 0 }
    }

    /// Append new content (one-way, no updates)
    pub fn append(&mut self, buffer: &FrameBuffer) {
        let mut renderer = StatefulCellRenderer::new();

        for y in 0..buffer.height {
            // Move to start of line
            renderer.move_to(0, self.written_lines + y);

            for x in 0..buffer.width {
                let cell = buffer.get(x, y).unwrap();
                renderer.write_cell(x, self.written_lines + y, cell);
            }

            // Newline at end
            print!("{}\n", renderer.finish());
            renderer.reset();
        }

        self.written_lines += buffer.height;
        std::io::stdout().flush().unwrap();
    }
}
```

---

## 7. Hit Testing

### 7.1 HitGrid

```rust
/// Grid mapping screen positions to component indices
pub struct HitGrid {
    grid: Vec<i32>, // -1 = no component
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

    /// Fill rectangle with component index
    pub fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, index: usize) {
        for row in y..(y + h).min(self.height) {
            for col in x..(x + w).min(self.width) {
                self.grid[row * self.width + col] = index as i32;
            }
        }
    }

    /// Get component at position
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

    /// Clear grid
    pub fn clear(&mut self) {
        self.grid.fill(-1);
    }
}
```

### 7.2 Populating During Render

```rust
fn render_component_with_hit(
    buffer: &mut FrameBuffer,
    hit_grid: &mut HitGrid,
    index: usize,
    layout: &ComputedLayout,
    arrays: &ComponentArrays,
    // ... other params
) {
    // ... render component ...

    // Register in hit grid if interactive
    if arrays.focusable.get(index)
        || arrays.on_click.get(index).is_some()
        || arrays.on_mouse_enter.get(index).is_some()
    {
        hit_grid.fill_rect(x, y, w, h, index);
    }

    // ... render children (they overwrite parent in hit grid) ...
}
```

---

## 8. Hidden Automatic Behaviors

### 8.1 Color Inheritance Cascade
Colors walk the parent chain until a non-default value is found.

### 8.2 Opacity Multiplication
Opacity multiplies through the parent chain, blending with parent background.

### 8.3 Wide Character Handling
Wide characters (CJK, emoji) automatically mark continuation cells with `char = 0`.

### 8.4 Scroll Offset Accumulation
Each nested scrollable container's offset is added to children's positions.

### 8.5 Terminal Resize Invalidation
On resize, the previous frame buffer is cleared, forcing full re-render.

### 8.6 Border Corner Intelligence
Border corners are chosen based on which sides have borders (e.g., only top+right → ┐).

### 8.7 Z-Index Sorting
Children with higher z-index render later (on top of siblings).

### 8.8 Clipping to Content Area
Children are clipped to parent's content area (inside padding/border).

### 8.9 Skip Invisible Components
Components with `visible = false` skip rendering entirely (not just transparent).

### 8.10 Synchronized Output
Fullscreen mode uses DEC synchronized output to prevent flicker.

---

## 9. Module Structure

```
crates/tui/src/renderer/
├── mod.rs           # Re-exports
├── cell.rs          # Cell, Rgba, CellAttrs
├── buffer.rs        # FrameBuffer
├── ansi.rs          # ANSI escape codes
├── diff.rs          # StatefulCellRenderer, render_diff
├── fullscreen.rs    # FullscreenRenderer
├── inline.rs        # InlineRenderer
├── append.rs        # AppendRenderer
├── hit_grid.rs      # HitGrid
└── border.rs        # BorderChars
```

---

## 10. Performance Targets

- **60fps fullscreen** (16.6ms per frame)
- <5ms layout computation
- <3ms frame buffer generation
- <2ms differential rendering
- <1ms ANSI generation

---

## 11. Summary

The rendering pipeline provides:

✅ **Efficient Cell Grid**: Parallel array storage for O(1) access
✅ **Three Color Modes**: Terminal default, ANSI 256, True RGB
✅ **8 Text Attributes**: Bold, dim, italic, underline, blink, inverse, hidden, strikethrough
✅ **Differential Rendering**: Only changed cells emit ANSI
✅ **Stateful Renderer**: Tracks current state for minimal escape codes
✅ **Three Output Modes**: Fullscreen, Inline, Append
✅ **Hit Testing**: O(1) lookup for mouse events
✅ **Unicode Support**: Proper wide character handling
✅ **Flicker-Free**: Synchronized output protocol
