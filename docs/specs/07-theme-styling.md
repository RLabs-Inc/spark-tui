# Theme & Styling Specification

## Overview

This specification documents the theming system: color handling (terminal default, ANSI 256, RGB), theme structure, style inheritance, text attributes, border styles, and reactive theming.

**Source Files Analyzed:**
- `/Users/rusty/Documents/Projects/TUI/tui/src/state/theme.ts`
- `/Users/rusty/Documents/Projects/TUI/tui/src/types/color.ts`
- `/Users/rusty/Documents/Projects/TUI/tui/src/engine/inheritance.ts`

---

## 1. Color System

### 1.1 Three Color Modes

```rust
/// RGBA color with special terminal values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgba {
    pub r: i16,  // -1 = terminal default, -2 = ANSI index in g
    pub g: i16,
    pub b: i16,
    pub a: i16,  // 0-255
}

impl Rgba {
    /// Terminal default (inherits from terminal theme)
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

    /// From hex string (#RGB, #RRGGBB, #RRGGBBAA)
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
                Some(Self::rgb(r, g, b))
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Self::rgb(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Self::rgba(r, g, b, a))
            }
            _ => None,
        }
    }

    /// Check if terminal default
    pub fn is_default(&self) -> bool {
        self.r == -1
    }

    /// Check if ANSI 256
    pub fn is_ansi(&self) -> bool {
        self.r == -2
    }

    /// Lighten color
    pub fn lighten(&self, amount: f32) -> Self {
        if self.is_default() || self.is_ansi() {
            return *self;
        }
        Self::rgb(
            (self.r as f32 + (255.0 - self.r as f32) * amount).min(255.0) as u8,
            (self.g as f32 + (255.0 - self.g as f32) * amount).min(255.0) as u8,
            (self.b as f32 + (255.0 - self.b as f32) * amount).min(255.0) as u8,
        )
    }

    /// Darken color
    pub fn darken(&self, amount: f32) -> Self {
        if self.is_default() || self.is_ansi() {
            return *self;
        }
        Self::rgb(
            (self.r as f32 * (1.0 - amount)).max(0.0) as u8,
            (self.g as f32 * (1.0 - amount)).max(0.0) as u8,
            (self.b as f32 * (1.0 - amount)).max(0.0) as u8,
        )
    }

    /// With alpha
    pub fn with_alpha(&self, alpha: u8) -> Self {
        Self {
            r: self.r,
            g: self.g,
            b: self.b,
            a: alpha as i16,
        }
    }
}
```

### 1.2 Named Colors

```rust
impl Rgba {
    // Basic colors
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    pub const RED: Self = Self::rgb(255, 0, 0);
    pub const GREEN: Self = Self::rgb(0, 255, 0);
    pub const BLUE: Self = Self::rgb(0, 0, 255);
    pub const YELLOW: Self = Self::rgb(255, 255, 0);
    pub const CYAN: Self = Self::rgb(0, 255, 255);
    pub const MAGENTA: Self = Self::rgb(255, 0, 255);

    // ANSI basic colors
    pub const ANSI_BLACK: Self = Self::ansi(0);
    pub const ANSI_RED: Self = Self::ansi(1);
    pub const ANSI_GREEN: Self = Self::ansi(2);
    pub const ANSI_YELLOW: Self = Self::ansi(3);
    pub const ANSI_BLUE: Self = Self::ansi(4);
    pub const ANSI_MAGENTA: Self = Self::ansi(5);
    pub const ANSI_CYAN: Self = Self::ansi(6);
    pub const ANSI_WHITE: Self = Self::ansi(7);

    // ANSI bright colors
    pub const ANSI_BRIGHT_BLACK: Self = Self::ansi(8);
    pub const ANSI_BRIGHT_RED: Self = Self::ansi(9);
    pub const ANSI_BRIGHT_GREEN: Self = Self::ansi(10);
    pub const ANSI_BRIGHT_YELLOW: Self = Self::ansi(11);
    pub const ANSI_BRIGHT_BLUE: Self = Self::ansi(12);
    pub const ANSI_BRIGHT_MAGENTA: Self = Self::ansi(13);
    pub const ANSI_BRIGHT_CYAN: Self = Self::ansi(14);
    pub const ANSI_BRIGHT_WHITE: Self = Self::ansi(15);
}
```

---

## 2. Theme Structure

### 2.1 Theme Definition

```rust
/// Complete theme definition
pub struct Theme {
    // Semantic colors (all as signals for reactivity)
    pub primary: Signal<Rgba>,
    pub secondary: Signal<Rgba>,
    pub success: Signal<Rgba>,
    pub warning: Signal<Rgba>,
    pub error: Signal<Rgba>,

    // Text colors
    pub text: Signal<Rgba>,
    pub text_muted: Signal<Rgba>,
    pub text_dim: Signal<Rgba>,

    // Background colors
    pub bg: Signal<Rgba>,
    pub bg_surface: Signal<Rgba>,
    pub bg_hover: Signal<Rgba>,
    pub bg_active: Signal<Rgba>,

    // Border colors
    pub border: Signal<Rgba>,
    pub border_focus: Signal<Rgba>,

    // Input colors
    pub input_bg: Signal<Rgba>,
    pub input_border: Signal<Rgba>,
    pub input_focus_border: Signal<Rgba>,
    pub placeholder: Signal<Rgba>,

    // Selection
    pub selection_bg: Signal<Rgba>,
    pub selection_fg: Signal<Rgba>,
}

impl Theme {
    pub fn new() -> Self {
        Self {
            primary: signal(Rgba::rgb(59, 130, 246)),      // Blue
            secondary: signal(Rgba::rgb(100, 116, 139)),   // Slate
            success: signal(Rgba::rgb(34, 197, 94)),       // Green
            warning: signal(Rgba::rgb(234, 179, 8)),       // Yellow
            error: signal(Rgba::rgb(239, 68, 68)),         // Red

            text: signal(Rgba::rgb(248, 250, 252)),        // Slate 50
            text_muted: signal(Rgba::rgb(148, 163, 184)),  // Slate 400
            text_dim: signal(Rgba::rgb(100, 116, 139)),    // Slate 500

            bg: signal(Rgba::rgb(15, 23, 42)),             // Slate 900
            bg_surface: signal(Rgba::rgb(30, 41, 59)),     // Slate 800
            bg_hover: signal(Rgba::rgb(51, 65, 85)),       // Slate 700
            bg_active: signal(Rgba::rgb(71, 85, 105)),     // Slate 600

            border: signal(Rgba::rgb(51, 65, 85)),         // Slate 700
            border_focus: signal(Rgba::rgb(59, 130, 246)), // Blue

            input_bg: signal(Rgba::rgb(30, 41, 59)),       // Slate 800
            input_border: signal(Rgba::rgb(51, 65, 85)),   // Slate 700
            input_focus_border: signal(Rgba::rgb(59, 130, 246)),
            placeholder: signal(Rgba::rgb(100, 116, 139)), // Slate 500

            selection_bg: signal(Rgba::rgb(59, 130, 246).with_alpha(128)),
            selection_fg: signal(Rgba::rgb(255, 255, 255)),
        }
    }
}
```

### 2.2 Global Theme

```rust
thread_local! {
    static THEME: Theme = Theme::new();
}

/// Get current theme
pub fn theme() -> &'static Theme {
    THEME.with(|t| t)
}

/// Set theme color
pub fn set_theme_primary(color: Rgba) {
    theme().primary.set(color);
}

// Convenience accessors
pub fn primary() -> Rgba {
    theme().primary.get()
}

pub fn text() -> Rgba {
    theme().text.get()
}

// etc.
```

---

## 3. Built-in Themes

### 3.1 Theme Presets

```rust
pub mod themes {
    use super::*;

    pub fn terminal() -> Theme {
        Theme {
            primary: signal(Rgba::DEFAULT),
            secondary: signal(Rgba::DEFAULT),
            success: signal(Rgba::ANSI_GREEN),
            warning: signal(Rgba::ANSI_YELLOW),
            error: signal(Rgba::ANSI_RED),
            text: signal(Rgba::DEFAULT),
            text_muted: signal(Rgba::ANSI_BRIGHT_BLACK),
            text_dim: signal(Rgba::ANSI_BRIGHT_BLACK),
            bg: signal(Rgba::DEFAULT),
            bg_surface: signal(Rgba::DEFAULT),
            bg_hover: signal(Rgba::DEFAULT),
            bg_active: signal(Rgba::DEFAULT),
            border: signal(Rgba::DEFAULT),
            border_focus: signal(Rgba::ANSI_BLUE),
            input_bg: signal(Rgba::DEFAULT),
            input_border: signal(Rgba::DEFAULT),
            input_focus_border: signal(Rgba::ANSI_BLUE),
            placeholder: signal(Rgba::ANSI_BRIGHT_BLACK),
            selection_bg: signal(Rgba::ANSI_BLUE),
            selection_fg: signal(Rgba::ANSI_WHITE),
        }
    }

    pub fn dracula() -> Theme {
        Theme {
            primary: signal(Rgba::rgb(189, 147, 249)),    // Purple
            secondary: signal(Rgba::rgb(139, 233, 253)), // Cyan
            success: signal(Rgba::rgb(80, 250, 123)),    // Green
            warning: signal(Rgba::rgb(255, 184, 108)),   // Orange
            error: signal(Rgba::rgb(255, 85, 85)),       // Red

            text: signal(Rgba::rgb(248, 248, 242)),      // Foreground
            text_muted: signal(Rgba::rgb(98, 114, 164)), // Comment
            text_dim: signal(Rgba::rgb(68, 71, 90)),

            bg: signal(Rgba::rgb(40, 42, 54)),           // Background
            bg_surface: signal(Rgba::rgb(68, 71, 90)),   // Current Line
            bg_hover: signal(Rgba::rgb(68, 71, 90)),
            bg_active: signal(Rgba::rgb(98, 114, 164)),

            border: signal(Rgba::rgb(68, 71, 90)),
            border_focus: signal(Rgba::rgb(189, 147, 249)),

            input_bg: signal(Rgba::rgb(40, 42, 54)),
            input_border: signal(Rgba::rgb(68, 71, 90)),
            input_focus_border: signal(Rgba::rgb(189, 147, 249)),
            placeholder: signal(Rgba::rgb(98, 114, 164)),

            selection_bg: signal(Rgba::rgb(68, 71, 90)),
            selection_fg: signal(Rgba::rgb(248, 248, 242)),
        }
    }

    pub fn nord() -> Theme {
        Theme {
            primary: signal(Rgba::rgb(136, 192, 208)),   // Nord8
            secondary: signal(Rgba::rgb(129, 161, 193)), // Nord9
            success: signal(Rgba::rgb(163, 190, 140)),   // Nord14
            warning: signal(Rgba::rgb(235, 203, 139)),   // Nord13
            error: signal(Rgba::rgb(191, 97, 106)),      // Nord11

            text: signal(Rgba::rgb(236, 239, 244)),      // Nord6
            text_muted: signal(Rgba::rgb(216, 222, 233)), // Nord5
            text_dim: signal(Rgba::rgb(76, 86, 106)),    // Nord3

            bg: signal(Rgba::rgb(46, 52, 64)),           // Nord0
            bg_surface: signal(Rgba::rgb(59, 66, 82)),   // Nord1
            bg_hover: signal(Rgba::rgb(67, 76, 94)),     // Nord2
            bg_active: signal(Rgba::rgb(76, 86, 106)),   // Nord3

            border: signal(Rgba::rgb(67, 76, 94)),
            border_focus: signal(Rgba::rgb(136, 192, 208)),

            input_bg: signal(Rgba::rgb(59, 66, 82)),
            input_border: signal(Rgba::rgb(67, 76, 94)),
            input_focus_border: signal(Rgba::rgb(136, 192, 208)),
            placeholder: signal(Rgba::rgb(76, 86, 106)),

            selection_bg: signal(Rgba::rgb(76, 86, 106)),
            selection_fg: signal(Rgba::rgb(236, 239, 244)),
        }
    }

    // Additional presets: catppuccin, gruvbox, tokyo_night, etc.
}
```

### 3.2 Apply Theme

```rust
/// Apply a theme preset
pub fn apply_theme(new_theme: Theme) {
    let current = theme();

    current.primary.set(new_theme.primary.get());
    current.secondary.set(new_theme.secondary.get());
    current.success.set(new_theme.success.get());
    current.warning.set(new_theme.warning.get());
    current.error.set(new_theme.error.get());
    // ... all other properties

    // Theme change triggers reactive updates automatically
}
```

---

## 4. Variants

### 4.1 Variant Definition

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    Default,
    Primary,
    Secondary,
    Success,
    Warning,
    Error,
    Ghost,
    Outline,
}

/// Get colors for variant
pub fn variant_colors(variant: Variant) -> (Rgba, Rgba) {
    match variant {
        Variant::Default => (theme().text.get(), Rgba::DEFAULT),
        Variant::Primary => (Rgba::WHITE, theme().primary.get()),
        Variant::Secondary => (Rgba::WHITE, theme().secondary.get()),
        Variant::Success => (Rgba::WHITE, theme().success.get()),
        Variant::Warning => (Rgba::BLACK, theme().warning.get()),
        Variant::Error => (Rgba::WHITE, theme().error.get()),
        Variant::Ghost => (theme().text.get(), Rgba::DEFAULT),
        Variant::Outline => (theme().primary.get(), Rgba::DEFAULT),
    }
}
```

### 4.2 Apply Variant

```rust
fn apply_variant(index: usize, variant: Reactive<Variant>) {
    // Derive colors from variant
    let fg = derived(move || variant_colors(variant.get()).0);
    let bg = derived(move || variant_colors(variant.get()).1);

    VISUAL.fg.bind_derived(index, fg);
    VISUAL.bg.bind_derived(index, bg);
}
```

---

## 5. Style Inheritance

### 5.1 Inherited Properties

Colors inherit through the component tree:

```rust
/// Get inherited foreground color
pub fn get_inherited_fg(index: usize) -> Rgba {
    let fg = VISUAL.fg.get(index);
    if !fg.is_default() {
        return fg;
    }

    // Walk parent chain
    let mut parent = LAYOUT.parent_index.get(index);
    while parent >= 0 {
        let parent_fg = VISUAL.fg.get(parent as usize);
        if !parent_fg.is_default() {
            return parent_fg;
        }
        parent = LAYOUT.parent_index.get(parent as usize);
    }

    // Return terminal default
    Rgba::DEFAULT
}

/// Get inherited background with opacity blending
pub fn get_inherited_bg(index: usize) -> Rgba {
    let bg = VISUAL.bg.get(index);
    let opacity = VISUAL.opacity.get(index);

    // Apply component opacity
    let mut result = if opacity < 1.0 {
        Rgba {
            r: bg.r,
            g: bg.g,
            b: bg.b,
            a: (bg.a as f32 * opacity) as i16,
        }
    } else {
        bg
    };

    // Blend with parent if semi-transparent
    if result.a < 255 {
        let parent = LAYOUT.parent_index.get(index);
        if parent >= 0 {
            let parent_bg = get_inherited_bg(parent as usize);
            result = blend_colors(parent_bg, result);
        }
    }

    result
}
```

### 5.2 Non-Inherited Properties

These do NOT inherit:
- Border styles
- Padding/margin
- Dimensions
- Layout properties

---

## 6. Text Attributes

### 6.1 Attribute Bitfield

```rust
bitflags! {
    pub struct TextAttrs: u8 {
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
```

### 6.2 Build Attributes

```rust
fn build_text_attrs(props: &TextProps) -> TextAttrs {
    let mut attrs = TextAttrs::empty();

    if props.bold.map(|b| b.get()).unwrap_or(false) {
        attrs |= TextAttrs::BOLD;
    }
    if props.italic.map(|i| i.get()).unwrap_or(false) {
        attrs |= TextAttrs::ITALIC;
    }
    if props.underline.map(|u| u.get()).unwrap_or(false) {
        attrs |= TextAttrs::UNDERLINE;
    }
    if props.strikethrough.map(|s| s.get()).unwrap_or(false) {
        attrs |= TextAttrs::STRIKETHROUGH;
    }
    if props.dim.map(|d| d.get()).unwrap_or(false) {
        attrs |= TextAttrs::DIM;
    }

    attrs
}
```

---

## 7. Border Styles

### 7.1 Border Style Enum

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderStyle {
    None,
    Single,     // ─│┌┐└┘
    Double,     // ═║╔╗╚╝
    Rounded,    // ─│╭╮╰╯
    Bold,       // ━┃┏┓┗┛
    DoubleSide, // ═│╒╕╘╛
    DoubleTop,  // ─║╓╖╙╜
    Ascii,      // -|++++
    Dashed,     // ┄┆┌┐└┘
    Dotted,     // ···
}
```

### 7.2 Border Characters

```rust
impl BorderStyle {
    pub fn chars(&self) -> BorderChars {
        match self {
            Self::Single => BorderChars {
                horizontal: '─',
                vertical: '│',
                top_left: '┌',
                top_right: '┐',
                bottom_left: '└',
                bottom_right: '┘',
            },
            Self::Double => BorderChars {
                horizontal: '═',
                vertical: '║',
                top_left: '╔',
                top_right: '╗',
                bottom_left: '╚',
                bottom_right: '╝',
            },
            Self::Rounded => BorderChars {
                horizontal: '─',
                vertical: '│',
                top_left: '╭',
                top_right: '╮',
                bottom_left: '╰',
                bottom_right: '╯',
            },
            // ... other styles
        }
    }
}
```

---

## 8. Reactive Theming

### 8.1 Theme Signals

All theme colors are signals, enabling:

```rust
// Component derives from theme
let border_color = derived(move || {
    if is_focused(index) {
        theme().border_focus.get()
    } else {
        theme().border.get()
    }
});

VISUAL.border_fg.bind_derived(index, border_color);

// Change theme → all components update automatically
apply_theme(themes::dracula());
```

### 8.2 Fine-Grained Updates

```rust
// Only components using primary update
theme().primary.set(Rgba::rgb(100, 200, 100));

// Components using other colors unaffected
```

---

## 9. Hidden Automatic Behaviors

### 9.1 Color Inheritance Cascade
Colors walk parent chain until non-default found.

### 9.2 Opacity Multiplication
Opacity multiplies through parent chain.

### 9.3 Variant Color Override
Variant prop overrides explicit fg/bg.

### 9.4 Border Color Fallback
Border fg defaults to text color if not specified.

### 9.5 Focus Border Override
Focused elements use `border_focus` color automatically.

### 9.6 Terminal Default Passthrough
`Rgba::DEFAULT` passes through to terminal's configured colors.

---

## 10. Module Structure

```
crates/tui/src/
├── types/
│   └── color.rs       # Rgba, color operations
├── state/
│   └── theme.rs       # Theme struct, presets, global state
└── engine/
    └── inheritance.rs # Color inheritance functions
```

---

## 11. Summary

The theme system provides:

✅ **Three Color Modes**: Terminal default, ANSI 256, True RGB
✅ **Semantic Colors**: 20+ theme properties (primary, text, bg, etc.)
✅ **Reactive Theming**: All colors as signals
✅ **Built-in Presets**: terminal, dracula, nord, catppuccin, etc.
✅ **Variants**: 8 semantic variants (primary, success, error, etc.)
✅ **Color Inheritance**: fg/bg walk parent chain
✅ **Opacity Blending**: Cascades through tree
✅ **8 Text Attributes**: Bold, italic, underline, etc.
✅ **10 Border Styles**: Single, double, rounded, etc.
✅ **Fine-Grained Updates**: Only affected components re-render
