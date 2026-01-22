# Component Primitives Specification

## Overview

This specification documents all UI primitives: Box, Text, Input, Show, When, Each, Scope, and Animation. Each primitive's complete API, props, behaviors, and implementation details are covered.

**Source Files Analyzed:**
- `/Users/rusty/Documents/Projects/TUI/tui/src/primitives/box.ts`
- `/Users/rusty/Documents/Projects/TUI/tui/src/primitives/text.ts`
- `/Users/rusty/Documents/Projects/TUI/tui/src/primitives/input.ts`
- `/Users/rusty/Documents/Projects/TUI/tui/src/primitives/when.ts`
- `/Users/rusty/Documents/Projects/TUI/tui/src/primitives/show.ts`
- `/Users/rusty/Documents/Projects/TUI/tui/src/primitives/each.ts`
- `/Users/rusty/Documents/Projects/TUI/tui/src/primitives/scope.ts`
- `/Users/rusty/Documents/Projects/TUI/tui/src/primitives/animation.ts`
- `/Users/rusty/Documents/Projects/TUI/tui/src/primitives/types.ts`
- `/Users/rusty/Documents/Projects/TUI/tui/src/primitives/utils.ts`

---

## 1. Reactive Props System

### 1.1 The Reactive<T> Pattern

Props can be:
- Static value: `T`
- Reactive signal: `Signal<T>`
- Derived value: `Derived<T>`
- Getter function: `Fn() -> T`

```rust
/// A value that can be reactive or static
pub enum Reactive<T> {
    Static(T),
    Signal(Signal<T>),
    Derived(Derived<T>),
    Getter(Box<dyn Fn() -> T>),
}

impl<T: Clone + 'static> Reactive<T> {
    pub fn get(&self) -> T {
        match self {
            Reactive::Static(v) => v.clone(),
            Reactive::Signal(s) => s.get(),
            Reactive::Derived(d) => d.get(),
            Reactive::Getter(f) => f(),
        }
    }
}

/// Convert various types to Reactive<T>
impl<T> From<T> for Reactive<T> {
    fn from(value: T) -> Self {
        Reactive::Static(value)
    }
}

impl<T: Clone + 'static> From<Signal<T>> for Reactive<T> {
    fn from(signal: Signal<T>) -> Self {
        Reactive::Signal(signal)
    }
}

impl<T: Clone + 'static> From<Derived<T>> for Reactive<T> {
    fn from(derived: Derived<T>) -> Self {
        Reactive::Derived(derived)
    }
}
```

### 1.2 Binding Props to Slots

```rust
/// Bind a reactive prop to a FlexNode slot
fn bind_prop<T: Clone + 'static>(
    slot: &Slot<T>,
    prop: Option<Reactive<T>>,
    default: T,
) {
    match prop {
        Some(Reactive::Static(v)) => slot.set(v),
        Some(Reactive::Signal(s)) => slot.bind_signal(s),
        Some(Reactive::Derived(d)) => slot.bind_derived(d),
        Some(Reactive::Getter(f)) => slot.bind_getter(f),
        None => slot.set(default),
    }
}
```

---

## 2. Common Prop Groups

### 2.1 Style Props

```rust
pub struct StyleProps {
    pub fg: Option<Reactive<Rgba>>,
    pub bg: Option<Reactive<Rgba>>,
    pub opacity: Option<Reactive<f32>>,
    pub bold: Option<Reactive<bool>>,
    pub italic: Option<Reactive<bool>>,
    pub underline: Option<Reactive<bool>>,
    pub strikethrough: Option<Reactive<bool>>,
    pub dim: Option<Reactive<bool>>,
    pub inverse: Option<Reactive<bool>>,
    pub variant: Option<Reactive<Variant>>,
}

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
    // ... more variants
}
```

### 2.2 Border Props

```rust
pub struct BorderProps {
    /// Enable all borders
    pub border: Option<Reactive<bool>>,
    /// Individual sides
    pub border_top: Option<Reactive<bool>>,
    pub border_right: Option<Reactive<bool>>,
    pub border_bottom: Option<Reactive<bool>>,
    pub border_left: Option<Reactive<bool>>,
    /// Border style
    pub border_style: Option<Reactive<BorderStyle>>,
    /// Border color
    pub border_fg: Option<Reactive<Rgba>>,
    pub border_bg: Option<Reactive<Rgba>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderStyle {
    None,
    Single,    // ─│┌┐└┘
    Double,    // ═║╔╗╚╝
    Rounded,   // ─│╭╮╰╯
    Bold,      // ━┃┏┓┗┛
    DoubleSide, // ═│╒╕╘╛
    DoubleTop,  // ─║╓╖╙╜
    Ascii,     // -|++++
    Dashed,    // ┄┆┌┐└┘
    Dotted,    // ···
}
```

### 2.3 Dimension Props

```rust
pub struct DimensionProps {
    pub width: Option<Reactive<Dimension>>,
    pub height: Option<Reactive<Dimension>>,
    pub min_width: Option<Reactive<Dimension>>,
    pub min_height: Option<Reactive<Dimension>>,
    pub max_width: Option<Reactive<Dimension>>,
    pub max_height: Option<Reactive<Dimension>>,
}
```

### 2.4 Spacing Props

```rust
pub struct SpacingProps {
    /// Shorthand for all padding
    pub padding: Option<Reactive<i32>>,
    /// Individual padding
    pub padding_top: Option<Reactive<i32>>,
    pub padding_right: Option<Reactive<i32>>,
    pub padding_bottom: Option<Reactive<i32>>,
    pub padding_left: Option<Reactive<i32>>,
    /// Shorthand for horizontal/vertical padding
    pub padding_x: Option<Reactive<i32>>,
    pub padding_y: Option<Reactive<i32>>,

    /// Shorthand for all margin
    pub margin: Option<Reactive<MarginValue>>,
    /// Individual margin
    pub margin_top: Option<Reactive<MarginValue>>,
    pub margin_right: Option<Reactive<MarginValue>>,
    pub margin_bottom: Option<Reactive<MarginValue>>,
    pub margin_left: Option<Reactive<MarginValue>>,
    /// Shorthand for horizontal/vertical margin
    pub margin_x: Option<Reactive<MarginValue>>,
    pub margin_y: Option<Reactive<MarginValue>>,

    /// Gap between children
    pub gap: Option<Reactive<i32>>,
    pub row_gap: Option<Reactive<i32>>,
    pub column_gap: Option<Reactive<i32>>,
}
```

### 2.5 Layout Props

```rust
pub struct LayoutProps {
    /// Flex container
    pub flex_direction: Option<Reactive<FlexDirection>>,
    pub flex_wrap: Option<Reactive<FlexWrap>>,
    pub justify_content: Option<Reactive<JustifyContent>>,
    pub align_items: Option<Reactive<AlignItems>>,
    pub align_content: Option<Reactive<AlignContent>>,

    /// Flex item
    pub flex: Option<Reactive<f32>>,  // Shorthand for grow
    pub flex_grow: Option<Reactive<f32>>,
    pub flex_shrink: Option<Reactive<f32>>,
    pub flex_basis: Option<Reactive<Dimension>>,
    pub align_self: Option<Reactive<AlignSelf>>,
    pub order: Option<Reactive<i32>>,

    /// Overflow
    pub overflow: Option<Reactive<Overflow>>,
    pub overflow_x: Option<Reactive<Overflow>>,
    pub overflow_y: Option<Reactive<Overflow>>,

    /// Z-index
    pub z_index: Option<Reactive<i32>>,
}
```

### 2.6 Interaction Props

```rust
pub struct InteractionProps {
    pub focusable: Option<Reactive<bool>>,
    pub tab_index: Option<Reactive<i32>>,
    pub auto_focus: Option<bool>,
}
```

### 2.7 Mouse Props

```rust
pub struct MouseProps {
    pub on_mouse_down: Option<Box<dyn Fn(MouseEvent)>>,
    pub on_mouse_up: Option<Box<dyn Fn(MouseEvent)>>,
    pub on_click: Option<Box<dyn Fn(MouseEvent)>>,
    pub on_mouse_enter: Option<Box<dyn Fn(MouseEvent)>>,
    pub on_mouse_leave: Option<Box<dyn Fn(MouseEvent)>>,
    pub on_scroll: Option<Box<dyn Fn(ScrollEvent)>>,
}
```

### 2.8 Keyboard Props

```rust
pub struct KeyboardProps {
    pub on_key_down: Option<Box<dyn Fn(KeyboardEvent)>>,
    pub on_key_up: Option<Box<dyn Fn(KeyboardEvent)>>,
    pub on_focus: Option<Box<dyn Fn()>>,
    pub on_blur: Option<Box<dyn Fn()>>,
}
```

---

## 3. Box Primitive

### 3.1 Complete Props

```rust
pub struct BoxProps {
    // Identity
    pub id: Option<String>,

    // Visibility
    pub visible: Option<Reactive<bool>>,

    // Style
    pub fg: Option<Reactive<Rgba>>,
    pub bg: Option<Reactive<Rgba>>,
    pub opacity: Option<Reactive<f32>>,
    pub variant: Option<Reactive<Variant>>,

    // Border
    pub border: Option<Reactive<bool>>,
    pub border_top: Option<Reactive<bool>>,
    pub border_right: Option<Reactive<bool>>,
    pub border_bottom: Option<Reactive<bool>>,
    pub border_left: Option<Reactive<bool>>,
    pub border_style: Option<Reactive<BorderStyle>>,
    pub border_fg: Option<Reactive<Rgba>>,
    pub border_bg: Option<Reactive<Rgba>>,

    // Dimensions
    pub width: Option<Reactive<Dimension>>,
    pub height: Option<Reactive<Dimension>>,
    pub min_width: Option<Reactive<Dimension>>,
    pub min_height: Option<Reactive<Dimension>>,
    pub max_width: Option<Reactive<Dimension>>,
    pub max_height: Option<Reactive<Dimension>>,

    // Padding
    pub padding: Option<Reactive<i32>>,
    pub padding_top: Option<Reactive<i32>>,
    pub padding_right: Option<Reactive<i32>>,
    pub padding_bottom: Option<Reactive<i32>>,
    pub padding_left: Option<Reactive<i32>>,
    pub padding_x: Option<Reactive<i32>>,
    pub padding_y: Option<Reactive<i32>>,

    // Margin
    pub margin: Option<Reactive<MarginValue>>,
    pub margin_top: Option<Reactive<MarginValue>>,
    pub margin_right: Option<Reactive<MarginValue>>,
    pub margin_bottom: Option<Reactive<MarginValue>>,
    pub margin_left: Option<Reactive<MarginValue>>,
    pub margin_x: Option<Reactive<MarginValue>>,
    pub margin_y: Option<Reactive<MarginValue>>,

    // Gap
    pub gap: Option<Reactive<i32>>,
    pub row_gap: Option<Reactive<i32>>,
    pub column_gap: Option<Reactive<i32>>,

    // Flex container
    pub flex_direction: Option<Reactive<FlexDirection>>,
    pub flex_wrap: Option<Reactive<FlexWrap>>,
    pub justify_content: Option<Reactive<JustifyContent>>,
    pub align_items: Option<Reactive<AlignItems>>,
    pub align_content: Option<Reactive<AlignContent>>,

    // Flex item
    pub flex: Option<Reactive<f32>>,
    pub flex_grow: Option<Reactive<f32>>,
    pub flex_shrink: Option<Reactive<f32>>,
    pub flex_basis: Option<Reactive<Dimension>>,
    pub align_self: Option<Reactive<AlignSelf>>,
    pub order: Option<Reactive<i32>>,

    // Overflow
    pub overflow: Option<Reactive<Overflow>>,

    // Z-index
    pub z_index: Option<Reactive<i32>>,

    // Focus
    pub focusable: Option<Reactive<bool>>,
    pub tab_index: Option<Reactive<i32>>,
    pub auto_focus: Option<bool>,

    // Events - Mouse
    pub on_mouse_down: Option<Box<dyn Fn(MouseEvent)>>,
    pub on_mouse_up: Option<Box<dyn Fn(MouseEvent)>>,
    pub on_click: Option<Box<dyn Fn(MouseEvent)>>,
    pub on_mouse_enter: Option<Box<dyn Fn(MouseEvent)>>,
    pub on_mouse_leave: Option<Box<dyn Fn(MouseEvent)>>,
    pub on_scroll: Option<Box<dyn Fn(ScrollEvent)>>,

    // Events - Keyboard
    pub on_key_down: Option<Box<dyn Fn(KeyboardEvent)>>,
    pub on_key_up: Option<Box<dyn Fn(KeyboardEvent)>>,
    pub on_focus: Option<Box<dyn Fn()>>,
    pub on_blur: Option<Box<dyn Fn()>>,

    // Children
    pub children: Option<Box<dyn FnOnce()>>,
}
```

### 3.2 Box Implementation

```rust
pub fn create_box(props: BoxProps) -> usize {
    // 1. Allocate index
    let index = allocate_index();

    // 2. Set component type
    CORE.component_type.set(index, ComponentType::Box);

    // 3. Register ID if provided
    if let Some(id) = props.id {
        register_id(index, id);
    }

    // 4. Bind visibility
    bind_prop(&CORE.visible, props.visible, true);

    // 5. Expand shorthand props
    let (pt, pr, pb, pl) = expand_padding(props);
    let (mt, mr, mb, ml) = expand_margin(props);
    let (rg, cg) = expand_gap(props);

    // 6. Bind all layout props to FlexNode slots
    bind_prop(&DIMENSIONS.width, props.width, Dimension::Auto);
    bind_prop(&DIMENSIONS.height, props.height, Dimension::Auto);
    // ... bind all dimension props

    bind_prop(&SPACING.padding_top, Some(pt), 0);
    bind_prop(&SPACING.padding_right, Some(pr), 0);
    bind_prop(&SPACING.padding_bottom, Some(pb), 0);
    bind_prop(&SPACING.padding_left, Some(pl), 0);
    // ... bind all spacing props

    bind_prop(&LAYOUT.flex_direction, props.flex_direction, FlexDirection::Column);
    bind_prop(&LAYOUT.flex_wrap, props.flex_wrap, FlexWrap::NoWrap);
    // ... bind all layout props

    // 7. Handle border (dual binding - layout AND visual)
    let has_border = props.border.map(|b| b.get()).unwrap_or(false);
    if has_border {
        // Bind to FlexNode (for layout sizing)
        bind_prop(&LAYOUT.border_top, props.border_top.or(props.border.clone()), false);
        bind_prop(&LAYOUT.border_right, props.border_right.or(props.border.clone()), false);
        bind_prop(&LAYOUT.border_bottom, props.border_bottom.or(props.border.clone()), false);
        bind_prop(&LAYOUT.border_left, props.border_left.or(props.border.clone()), false);

        // Bind to visual arrays (for rendering)
        bind_prop(&VISUAL.border_style, props.border_style, BorderStyle::Single);
        bind_prop(&VISUAL.border_fg, props.border_fg, Rgba::DEFAULT);
        bind_prop(&VISUAL.border_bg, props.border_bg, Rgba::DEFAULT);
    }

    // 8. Bind visual props
    bind_prop(&VISUAL.fg, props.fg, Rgba::DEFAULT);
    bind_prop(&VISUAL.bg, props.bg, Rgba::DEFAULT);
    bind_prop(&VISUAL.opacity, props.opacity, 1.0);

    // 9. Apply variant (theme colors)
    if let Some(variant) = props.variant {
        apply_variant(index, variant);
    }

    // 10. Handle focusable
    let should_focus = props.focusable.map(|f| f.get()).unwrap_or(false)
        || (props.overflow.map(|o| o.get()) == Some(Overflow::Scroll)
            && props.focusable.is_none());

    if should_focus {
        INTERACTION.focusable.set(index, true);
        if let Some(tab_idx) = props.tab_index {
            INTERACTION.tab_index.set(index, tab_idx.get());
        }
    }

    // 11. Register event handlers
    if let Some(handler) = props.on_click {
        INTERACTION.on_click.set(index, Some(handler));
    }
    // ... register all handlers

    // 12. Attach to parent
    if let Some(parent) = current_parent() {
        attach_child(parent, index);
    }

    // 13. Push as parent and render children
    push_parent(index);
    if let Some(children) = props.children {
        children();
    }
    pop_parent();

    // 14. Handle auto-focus
    if props.auto_focus == Some(true) {
        defer_focus(index);
    }

    // 15. Return cleanup function (for scope)
    register_cleanup(move || {
        release_index(index);
    });

    index
}
```

### 3.3 Hidden Behaviors

1. **Auto-focusable scrollable**: `overflow: scroll` implies `focusable: true` unless explicitly set to `false`
2. **Click-to-focus**: Focusable boxes automatically focus on click
3. **Variant color application**: Variant prop overrides fg/bg with theme colors
4. **Border dual-binding**: Border props bind to BOTH FlexNode (layout) AND visual arrays
5. **Padding shorthand expansion**: `padding` expands to all four sides, `padding_x`/`padding_y` override specific axes

---

## 4. Text Primitive

### 4.1 Complete Props

```rust
pub struct TextProps {
    // Content (required)
    pub content: Reactive<TextSource>,

    // Style
    pub fg: Option<Reactive<Rgba>>,
    pub bg: Option<Reactive<Rgba>>,
    pub opacity: Option<Reactive<f32>>,
    pub bold: Option<Reactive<bool>>,
    pub italic: Option<Reactive<bool>>,
    pub underline: Option<Reactive<bool>>,
    pub strikethrough: Option<Reactive<bool>>,
    pub dim: Option<Reactive<bool>>,

    // Text layout
    pub align: Option<Reactive<TextAlign>>,
    pub wrap: Option<Reactive<TextWrap>>,
    pub overflow: Option<Reactive<TextOverflow>>,

    // Flex item props (Text is never a container)
    pub flex: Option<Reactive<f32>>,
    pub flex_grow: Option<Reactive<f32>>,
    pub flex_shrink: Option<Reactive<f32>>,
    pub align_self: Option<Reactive<AlignSelf>>,

    // Margin
    pub margin: Option<Reactive<MarginValue>>,
    pub margin_top: Option<Reactive<MarginValue>>,
    pub margin_right: Option<Reactive<MarginValue>>,
    pub margin_bottom: Option<Reactive<MarginValue>>,
    pub margin_left: Option<Reactive<MarginValue>>,

    // Events
    pub on_click: Option<Box<dyn Fn(MouseEvent)>>,
}

/// Text content source
pub enum TextSource {
    String(String),
    Number(f64),
    Signal(Signal<String>),
}

impl From<&str> for TextSource {
    fn from(s: &str) -> Self {
        TextSource::String(s.to_string())
    }
}

impl From<String> for TextSource {
    fn from(s: String) -> Self {
        TextSource::String(s)
    }
}

impl From<i32> for TextSource {
    fn from(n: i32) -> Self {
        TextSource::Number(n as f64)
    }
}

impl From<f64> for TextSource {
    fn from(n: f64) -> Self {
        TextSource::Number(n)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextWrap {
    None,    // No wrapping, may overflow
    Wrap,    // Word wrap
    Char,    // Character wrap
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextOverflow {
    Visible,  // Show overflow
    Clip,     // Clip at boundary
    Ellipsis, // Add ... at end
}
```

### 4.2 Text Implementation

```rust
pub fn create_text(props: TextProps) -> usize {
    let index = allocate_index();

    CORE.component_type.set(index, ComponentType::Text);

    // Convert content to string source and bind
    let content_signal = content_to_signal(props.content);
    TEXT.text_content.bind_signal(index, content_signal);

    // Text layout
    bind_prop(&TEXT.text_align, props.align, TextAlign::Left);
    bind_prop(&TEXT.text_wrap, props.wrap, TextWrap::None);
    bind_prop(&TEXT.text_overflow, props.overflow, TextOverflow::Visible);

    // Build text attributes bitfield
    let attrs = build_text_attrs(&props);
    TEXT.text_attrs.set(index, attrs);

    // Visual props
    bind_prop(&VISUAL.fg, props.fg, Rgba::DEFAULT);
    bind_prop(&VISUAL.bg, props.bg, Rgba::DEFAULT);

    // Flex item props
    bind_prop(&LAYOUT.flex_grow, props.flex.or(props.flex_grow), 0.0);
    bind_prop(&LAYOUT.flex_shrink, props.flex_shrink, 1.0);
    bind_prop(&LAYOUT.align_self, props.align_self, AlignSelf::Auto);

    // Attach to parent
    if let Some(parent) = current_parent() {
        attach_child(parent, index);
    }

    register_cleanup(move || release_index(index));

    index
}

fn content_to_signal(source: Reactive<TextSource>) -> Signal<String> {
    match source {
        Reactive::Static(TextSource::String(s)) => signal(s),
        Reactive::Static(TextSource::Number(n)) => signal(format_number(n)),
        Reactive::Signal(s) => s,
        Reactive::Derived(d) => {
            // Convert derived to signal
            let sig = signal(d.get());
            effect(move || sig.set(d.get()));
            sig
        }
        _ => signal(String::new()),
    }
}
```

---

## 5. Input Primitive

### 5.1 Complete Props

```rust
pub struct InputProps {
    // Two-way value binding (required)
    pub value: Signal<String>,

    // Style
    pub fg: Option<Reactive<Rgba>>,
    pub bg: Option<Reactive<Rgba>>,
    pub placeholder: Option<String>,
    pub placeholder_fg: Option<Reactive<Rgba>>,

    // Cursor
    pub cursor_style: Option<CursorStyle>,
    pub cursor_fg: Option<Reactive<Rgba>>,
    pub cursor_bg: Option<Reactive<Rgba>>,
    pub cursor_blink: Option<bool>,
    pub cursor_blink_fps: Option<u32>,

    // Behavior
    pub password: Option<bool>,
    pub password_char: Option<char>,
    pub max_length: Option<usize>,
    pub auto_focus: Option<bool>,

    // Dimensions
    pub width: Option<Reactive<Dimension>>,
    pub min_width: Option<Reactive<Dimension>>,
    pub max_width: Option<Reactive<Dimension>>,

    // Padding
    pub padding: Option<Reactive<i32>>,
    pub padding_left: Option<Reactive<i32>>,
    pub padding_right: Option<Reactive<i32>>,

    // Border
    pub border: Option<Reactive<bool>>,
    pub border_style: Option<Reactive<BorderStyle>>,
    pub border_fg: Option<Reactive<Rgba>>,

    // Focus
    pub tab_index: Option<Reactive<i32>>,

    // Events
    pub on_change: Option<Box<dyn Fn(String)>>,
    pub on_submit: Option<Box<dyn Fn(String)>>,
    pub on_cancel: Option<Box<dyn Fn()>>,
    pub on_focus: Option<Box<dyn Fn()>>,
    pub on_blur: Option<Box<dyn Fn()>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorStyle {
    Block,     // █ (inverse video)
    Bar,       // │
    Underline, // _
}
```

### 5.2 Input Implementation

```rust
pub fn create_input(props: InputProps) -> usize {
    let index = allocate_index();

    CORE.component_type.set(index, ComponentType::Input);

    // Input is always focusable
    INTERACTION.focusable.set(index, true);
    if let Some(tab_idx) = props.tab_index {
        INTERACTION.tab_index.set(index, tab_idx.get());
    }

    // Bind value signal to text content (for rendering)
    TEXT.text_content.bind_signal(index, props.value.clone());

    // Cursor state
    let cursor_pos = signal(0usize);
    INTERACTION.cursor_position.bind_signal(index, cursor_pos.clone());
    INTERACTION.cursor_visible.set(index, true);

    // Cursor style
    let cursor_char = match props.cursor_style.unwrap_or(CursorStyle::Block) {
        CursorStyle::Block => 0,     // 0 = inverse video
        CursorStyle::Bar => '│' as u32,
        CursorStyle::Underline => '_' as u32,
    };
    INTERACTION.cursor_char.set(index, cursor_char);

    // Cursor blink
    if props.cursor_blink.unwrap_or(true) {
        let fps = props.cursor_blink_fps.unwrap_or(2);
        setup_cursor_blink(index, fps);
    }

    // Password masking
    let is_password = props.password.unwrap_or(false);
    let mask_char = props.password_char.unwrap_or('•');

    // Keyboard handler
    let value = props.value.clone();
    let max_len = props.max_length;
    let on_change = props.on_change;
    let on_submit = props.on_submit;
    let on_cancel = props.on_cancel;

    INTERACTION.on_key_down.set(index, Some(Box::new(move |event: KeyboardEvent| {
        let mut current = value.get();
        let mut pos = cursor_pos.get();

        match event.key.as_str() {
            "ArrowLeft" => {
                if pos > 0 {
                    cursor_pos.set(pos - 1);
                }
            }
            "ArrowRight" => {
                if pos < current.len() {
                    cursor_pos.set(pos + 1);
                }
            }
            "Home" => {
                cursor_pos.set(0);
            }
            "End" => {
                cursor_pos.set(current.len());
            }
            "Backspace" => {
                if pos > 0 {
                    current.remove(pos - 1);
                    value.set(current.clone());
                    cursor_pos.set(pos - 1);
                    if let Some(ref cb) = on_change {
                        cb(current);
                    }
                }
            }
            "Delete" => {
                if pos < current.len() {
                    current.remove(pos);
                    value.set(current.clone());
                    if let Some(ref cb) = on_change {
                        cb(current);
                    }
                }
            }
            "Enter" => {
                if let Some(ref cb) = on_submit {
                    cb(value.get());
                }
            }
            "Escape" => {
                if let Some(ref cb) = on_cancel {
                    cb();
                }
            }
            key if key.len() == 1 => {
                // Printable character
                let ch = key.chars().next().unwrap();
                if max_len.map(|m| current.len() < m).unwrap_or(true) {
                    current.insert(pos, ch);
                    value.set(current.clone());
                    cursor_pos.set(pos + 1);
                    if let Some(ref cb) = on_change {
                        cb(current);
                    }
                }
            }
            _ => {}
        }
    })));

    // Focus handlers
    if let Some(on_focus) = props.on_focus {
        register_focus_callback(index, FocusEvent::Focus, on_focus);
    }
    if let Some(on_blur) = props.on_blur {
        register_focus_callback(index, FocusEvent::Blur, on_blur);
    }

    // Layout props
    bind_prop(&DIMENSIONS.width, props.width, Dimension::Auto);

    // Border
    if props.border.map(|b| b.get()).unwrap_or(false) {
        LAYOUT.border_top.set(index, true);
        LAYOUT.border_right.set(index, true);
        LAYOUT.border_bottom.set(index, true);
        LAYOUT.border_left.set(index, true);
    }

    // Attach to parent
    if let Some(parent) = current_parent() {
        attach_child(parent, index);
    }

    // Auto-focus
    if props.auto_focus == Some(true) {
        defer_focus(index);
    }

    register_cleanup(move || {
        cleanup_cursor_blink(index);
        release_index(index);
    });

    index
}
```

---

## 6. Show Primitive

### 6.1 Props

```rust
pub struct ShowProps<T: FnOnce()> {
    pub when: Reactive<bool>,
    pub children: T,
    pub fallback: Option<Box<dyn FnOnce()>>,
}
```

### 6.2 Implementation

```rust
pub fn create_show<T: FnOnce() + 'static>(props: ShowProps<T>) {
    let when = props.when;
    let children = props.children;
    let fallback = props.fallback;

    // Track current state
    let mounted = signal(false);
    let cleanup_fn: RefCell<Option<Box<dyn FnOnce()>>> = RefCell::new(None);

    effect(move || {
        let should_show = when.get();
        let currently_mounted = mounted.get();

        if should_show && !currently_mounted {
            // Mount children
            mounted.set(true);

            // Create scope for cleanup
            let scope = create_scope();
            push_scope(scope);
            children();
            pop_scope();

            // Store cleanup
            *cleanup_fn.borrow_mut() = Some(Box::new(move || {
                dispose_scope(scope);
            }));
        } else if !should_show && currently_mounted {
            // Unmount
            mounted.set(false);

            if let Some(cleanup) = cleanup_fn.borrow_mut().take() {
                cleanup();
            }

            // Mount fallback if provided
            if let Some(fb) = fallback.take() {
                fb();
            }
        }
    });
}
```

---

## 7. When Primitive (Async)

### 7.1 Props

```rust
pub struct WhenProps<T, E, P, Th, C>
where
    T: 'static,
    E: 'static,
    P: FnOnce(),
    Th: FnOnce(T),
    C: FnOnce(E),
{
    pub future: Pin<Box<dyn Future<Output = Result<T, E>>>>,
    pub pending: P,
    pub then: Th,
    pub catch: C,
}
```

### 7.2 Implementation

```rust
pub fn create_when<T, E, P, Th, C>(props: WhenProps<T, E, P, Th, C>)
where
    T: 'static,
    E: 'static,
    P: FnOnce() + 'static,
    Th: FnOnce(T) + 'static,
    C: FnOnce(E) + 'static,
{
    // Show pending state
    let pending_scope = create_scope();
    push_scope(pending_scope);
    (props.pending)();
    pop_scope();

    // Track future ID for cancellation
    let future_id = signal(generate_future_id());

    // Spawn future
    let current_id = future_id.get();
    spawn_local(async move {
        let result = props.future.await;

        // Check if still relevant
        if future_id.get() != current_id {
            return; // Cancelled
        }

        // Cleanup pending
        dispose_scope(pending_scope);

        // Show result
        match result {
            Ok(value) => {
                let scope = create_scope();
                push_scope(scope);
                (props.then)(value);
                pop_scope();
            }
            Err(error) => {
                let scope = create_scope();
                push_scope(scope);
                (props.catch)(error);
                pop_scope();
            }
        }
    });
}
```

---

## 8. Each Primitive

### 8.1 Props

```rust
pub struct EachProps<T, K, F>
where
    T: Clone + 'static,
    K: Hash + Eq + Clone + 'static,
    F: Fn(T, usize) + 'static,
{
    pub items: Reactive<Vec<T>>,
    pub key: Box<dyn Fn(&T) -> K>,
    pub children: F,
}
```

### 8.2 Implementation

```rust
pub fn create_each<T, K, F>(props: EachProps<T, K, F>)
where
    T: Clone + 'static,
    K: Hash + Eq + Clone + 'static,
    F: Fn(T, usize) + Clone + 'static,
{
    // Track rendered items by key
    let rendered: RefCell<HashMap<K, (usize, Box<dyn FnOnce()>)>> = RefCell::new(HashMap::new());

    effect(move || {
        let items = props.items.get();
        let mut new_rendered = HashMap::new();
        let mut old_rendered = rendered.borrow_mut();

        for (index, item) in items.iter().enumerate() {
            let key = (props.key)(item);

            if let Some((_, cleanup)) = old_rendered.remove(&key) {
                // Reuse existing
                new_rendered.insert(key, (index, cleanup));
            } else {
                // Create new
                let scope = create_scope();
                push_scope(scope);

                // Create item signal for fine-grained reactivity
                let item_signal = signal(item.clone());
                (props.children.clone())(item_signal.get(), index);

                pop_scope();

                let cleanup = Box::new(move || dispose_scope(scope));
                new_rendered.insert(key.clone(), (index, cleanup));
            }
        }

        // Cleanup removed items
        for (_, (_, cleanup)) in old_rendered.drain() {
            cleanup();
        }

        *old_rendered = new_rendered;
    });
}
```

---

## 9. Scope Primitive

### 9.1 Implementation

```rust
thread_local! {
    static SCOPE_STACK: RefCell<Vec<Scope>> = RefCell::new(Vec::new());
}

pub struct Scope {
    id: usize,
    cleanups: RefCell<Vec<Box<dyn FnOnce()>>>,
    children: RefCell<Vec<Scope>>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            id: generate_scope_id(),
            cleanups: RefCell::new(Vec::new()),
            children: RefCell::new(Vec::new()),
        }
    }

    pub fn on_cleanup<F: FnOnce() + 'static>(&self, f: F) {
        self.cleanups.borrow_mut().push(Box::new(f));
    }

    pub fn dispose(self) {
        // Dispose children first
        for child in self.children.borrow_mut().drain(..) {
            child.dispose();
        }

        // Run cleanups in reverse order
        for cleanup in self.cleanups.borrow_mut().drain(..).rev() {
            cleanup();
        }
    }
}

pub fn create_scope() -> Scope {
    let scope = Scope::new();

    // Add as child of current scope
    SCOPE_STACK.with(|stack| {
        if let Some(parent) = stack.borrow().last() {
            parent.children.borrow_mut().push(scope.clone());
        }
    });

    scope
}

pub fn push_scope(scope: Scope) {
    SCOPE_STACK.with(|stack| stack.borrow_mut().push(scope));
}

pub fn pop_scope() -> Option<Scope> {
    SCOPE_STACK.with(|stack| stack.borrow_mut().pop())
}

pub fn on_cleanup<F: FnOnce() + 'static>(f: F) {
    SCOPE_STACK.with(|stack| {
        if let Some(scope) = stack.borrow().last() {
            scope.on_cleanup(f);
        }
    });
}
```

---

## 10. Animation Primitive

### 10.1 Frame Sets

```rust
pub struct FrameSet {
    pub frames: Vec<&'static str>,
    pub fps: u32,
}

impl FrameSet {
    pub const SPINNER: Self = Self {
        frames: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
        fps: 10,
    };

    pub const DOTS: Self = Self {
        frames: vec![".", "..", "...", ""],
        fps: 3,
    };

    pub const LINE: Self = Self {
        frames: vec!["-", "\\", "|", "/"],
        fps: 8,
    };

    pub const BAR: Self = Self {
        frames: vec!["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█", "▇", "▆", "▅", "▄", "▃", "▂"],
        fps: 10,
    };

    pub const BOUNCE: Self = Self {
        frames: vec!["⠁", "⠂", "⠄", "⠂"],
        fps: 6,
    };

    pub const CLOCK: Self = Self {
        frames: vec!["🕐", "🕑", "🕒", "🕓", "🕔", "🕕", "🕖", "🕗", "🕘", "🕙", "🕚", "🕛"],
        fps: 2,
    };

    pub const PULSE: Self = Self {
        frames: vec!["█", "▓", "▒", "░", "▒", "▓"],
        fps: 6,
    };
}
```

### 10.2 Blink Clock Registry

```rust
thread_local! {
    static BLINK_CLOCKS: RefCell<HashMap<u32, BlinkClock>> = RefCell::new(HashMap::new());
}

struct BlinkClock {
    fps: u32,
    phase: Signal<usize>,
    frame_count: usize,
    subscribers: usize,
    interval: Option<IntervalHandle>,
}

impl BlinkClock {
    fn new(fps: u32, frame_count: usize) -> Self {
        Self {
            fps,
            phase: signal(0),
            frame_count,
            subscribers: 0,
            interval: None,
        }
    }

    fn subscribe(&mut self) -> Signal<usize> {
        self.subscribers += 1;

        if self.subscribers == 1 {
            // Start interval
            let phase = self.phase.clone();
            let frame_count = self.frame_count;
            let interval_ms = 1000 / self.fps;

            self.interval = Some(set_interval(move || {
                let current = phase.get();
                phase.set((current + 1) % frame_count);
            }, interval_ms));
        }

        self.phase.clone()
    }

    fn unsubscribe(&mut self) {
        self.subscribers -= 1;

        if self.subscribers == 0 {
            // Stop interval
            if let Some(handle) = self.interval.take() {
                clear_interval(handle);
            }
        }
    }
}

pub fn get_blink_phase(fps: u32, frame_count: usize) -> Signal<usize> {
    BLINK_CLOCKS.with(|clocks| {
        let mut clocks = clocks.borrow_mut();
        let clock = clocks.entry(fps).or_insert_with(|| BlinkClock::new(fps, frame_count));
        clock.subscribe()
    })
}
```

### 10.3 Animation Implementation

```rust
pub struct AnimationProps {
    pub frames: FrameSet,
    pub active: Option<Reactive<bool>>,
    // ... style props
}

pub fn create_animation(props: AnimationProps) -> usize {
    let index = allocate_index();

    CORE.component_type.set(index, ComponentType::Text);

    let frames = props.frames.frames;
    let fps = props.frames.fps;

    // Get shared blink phase
    let phase = get_blink_phase(fps, frames.len());

    // Derive current frame text
    let text = derived(move || {
        let active = props.active.as_ref().map(|a| a.get()).unwrap_or(true);
        if active {
            frames[phase.get() % frames.len()].to_string()
        } else {
            frames[0].to_string()
        }
    });

    TEXT.text_content.bind_derived(index, text);

    // ... rest of setup

    register_cleanup(move || {
        BLINK_CLOCKS.with(|clocks| {
            if let Some(clock) = clocks.borrow_mut().get_mut(&fps) {
                clock.unsubscribe();
            }
        });
        release_index(index);
    });

    index
}
```

---

## 11. Rust Ergonomics

### 11.1 Builder Pattern

```rust
pub struct BoxBuilder {
    props: BoxProps,
}

impl BoxBuilder {
    pub fn new() -> Self {
        Self {
            props: BoxProps::default(),
        }
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.props.id = Some(id.into());
        self
    }

    pub fn width(mut self, width: impl Into<Reactive<Dimension>>) -> Self {
        self.props.width = Some(width.into());
        self
    }

    pub fn height(mut self, height: impl Into<Reactive<Dimension>>) -> Self {
        self.props.height = Some(height.into());
        self
    }

    pub fn padding(mut self, padding: i32) -> Self {
        self.props.padding = Some(padding.into());
        self
    }

    pub fn bg(mut self, color: impl Into<Reactive<Rgba>>) -> Self {
        self.props.bg = Some(color.into());
        self
    }

    pub fn border(mut self) -> Self {
        self.props.border = Some(true.into());
        self
    }

    pub fn flex_row(mut self) -> Self {
        self.props.flex_direction = Some(FlexDirection::Row.into());
        self
    }

    pub fn children<F: FnOnce() + 'static>(mut self, f: F) -> Self {
        self.props.children = Some(Box::new(f));
        self
    }

    pub fn build(self) -> usize {
        create_box(self.props)
    }
}

// Usage:
// Box::new()
//     .width(100)
//     .height(50)
//     .padding(2)
//     .border()
//     .bg(Rgba::rgb(30, 30, 30))
//     .children(|| {
//         Text::new("Hello");
//     })
//     .build();
```

### 11.2 Declarative Macros

```rust
#[macro_export]
macro_rules! box_ {
    ($($key:ident : $value:expr),* $(,)? ; $children:expr) => {{
        let mut builder = BoxBuilder::new();
        $(
            builder = builder.$key($value);
        )*
        builder.children($children).build()
    }};

    ($($key:ident : $value:expr),* $(,)?) => {{
        let mut builder = BoxBuilder::new();
        $(
            builder = builder.$key($value);
        )*
        builder.build()
    }};
}

#[macro_export]
macro_rules! text {
    ($content:expr $(, $key:ident : $value:expr)* $(,)?) => {{
        let mut builder = TextBuilder::new($content);
        $(
            builder = builder.$key($value);
        )*
        builder.build()
    }};
}

// Usage:
// box_! {
//     width: 100,
//     padding: 2,
//     border: true;
//     || {
//         text!("Hello", bold: true);
//         text!("World");
//     }
// };
```

---

## 12. Summary

### Primitives Implemented

| Primitive | Purpose | Key Features |
|-----------|---------|--------------|
| **Box** | Container | Flexbox, border, scroll, focus, all events |
| **Text** | Display text | Wrap, align, truncate, styling |
| **Input** | Text input | Two-way binding, cursor, password, validation |
| **Show** | Conditional | Mount/unmount based on boolean |
| **When** | Async | Pending/then/catch states |
| **Each** | Lists | Keyed reconciliation |
| **Scope** | Cleanup | Automatic resource management |
| **Animation** | Frames | Shared clock, multiple frame sets |

### Props Count

- **Box**: 40+ props
- **Text**: 15+ props
- **Input**: 25+ props
- **Show**: 3 props
- **When**: 4 props
- **Each**: 3 props

### Hidden Behaviors

1. Auto-focusable scrollable containers
2. Click-to-focus for focusable elements
3. Variant color application from theme
4. Border dual-binding (layout + visual)
5. Padding/margin shorthand expansion
6. Cursor blink shared clock
7. Password masking
8. Max length enforcement
9. Scope automatic cleanup
10. Each keyed reconciliation
