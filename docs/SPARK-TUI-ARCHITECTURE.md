# spark-tui Architecture Document

## Overview

This document captures the complete architecture of the TypeScript TUI framework (`@rlabs-inc/tui`) to guide the Rust implementation (`spark-tui`). The goal is to create a fully reactive terminal UI framework with TypeScript-like ergonomics, built on `spark-signals`.

**Repository**: `https://github.com/RLabs-Inc/spark-tui`
**Foundation**: `spark-signals` (published on crates.io)

---

## Core Philosophy

1. **No Fixed FPS** - Pure reactivity, renders only when data changes
2. **Single Render Effect** - All terminal output flows from ONE effect
3. **Parallel Arrays (ECS-like)** - Component data in contiguous arrays, not object trees
4. **Slot Binding** - Props bind to array slots, preserving reactivity
5. **Derived Purity** - Layout and framebuffer are pure computations
6. **Fine-grained Tracking** - Per-index reactivity via TrackedSlotArray

---

## Reactive Pipeline

```
User Code (component functions)
    ↓
Props (Signal<T> | T | Fn() -> T)
    ↓ setSource() binding
Parallel Arrays (SlotArray<T>)
    ↓ dirty tracking (ReactiveSet<usize>)
layoutDerived (TITAN 5-pass flexbox)
    ↓ ComputedLayout { x[], y[], width[], height[], scrollable[] }
frameBufferDerived (recursive component rendering)
    ↓ FrameBuffer { cells[][], hit_regions[] }
SINGLE render effect
    ↓ HitGrid update + diff rendering
Terminal Output (ANSI escape sequences)
```

---

## Parallel Arrays Architecture

### Overview

Instead of component objects with properties, all component state lives in **parallel arrays** indexed by component ID. This provides:
- Cache-friendly memory layout
- O(1) component property access
- No object allocation per component
- Easy serialization/debugging

### Array Categories (93+ arrays in TypeScript)

#### 1. Core Arrays (4)
```rust
component_type: Vec<ComponentType>,     // Box, Text, Input, etc.
parent_index: SlotArray<i32>,           // Parent component (-1 = root)
visible: SlotArray<bool>,               // Visibility flag
component_id: SlotArray<String>,        // Debug identifier
```

#### 2. Dimension Arrays (6)
```rust
width: SlotArray<Dimension>,            // 0 = auto, number, or "50%"
height: SlotArray<Dimension>,
min_width: SlotArray<Dimension>,
min_height: SlotArray<Dimension>,
max_width: SlotArray<Dimension>,        // 0 = no limit
max_height: SlotArray<Dimension>,
```

#### 3. Spacing Arrays (11)
```rust
// Margin (space outside)
margin_top: SlotArray<i32>,
margin_right: SlotArray<i32>,
margin_bottom: SlotArray<i32>,
margin_left: SlotArray<i32>,

// Padding (space inside)
padding_top: SlotArray<i32>,
padding_right: SlotArray<i32>,
padding_bottom: SlotArray<i32>,
padding_left: SlotArray<i32>,

// Gap (between flex children)
gap: SlotArray<i32>,
row_gap: SlotArray<i32>,
column_gap: SlotArray<i32>,
```

#### 4. Layout Arrays (21)
```rust
// Flex container
flex_direction: SlotArray<FlexDirection>,   // Column, Row, etc.
flex_wrap: SlotArray<FlexWrap>,
justify_content: SlotArray<JustifyContent>,
align_items: SlotArray<AlignItems>,
align_content: SlotArray<AlignContent>,

// Flex item
flex_grow: SlotArray<f32>,
flex_shrink: SlotArray<f32>,
flex_basis: SlotArray<i32>,
align_self: SlotArray<AlignSelf>,
order: SlotArray<i32>,

// Positioning
position: SlotArray<Position>,              // Relative, Absolute
top: SlotArray<i32>,
right: SlotArray<i32>,
bottom: SlotArray<i32>,
left: SlotArray<i32>,

// Border (layout impact)
border_top: SlotArray<i32>,                 // 0 or 1 cell
border_right: SlotArray<i32>,
border_bottom: SlotArray<i32>,
border_left: SlotArray<i32>,

// Stacking
z_index: SlotArray<i32>,
overflow: SlotArray<Overflow>,              // Visible, Hidden, Scroll, Auto
```

#### 5. Visual Arrays (15)
```rust
fg_color: SlotArray<Option<RGBA>>,          // None = inherit
bg_color: SlotArray<Option<RGBA>>,          // None = transparent
opacity: SlotArray<f32>,                    // 0.0 - 1.0

// Border styling
border_style: SlotArray<BorderStyle>,       // Single, Double, Rounded, etc.
border_color: SlotArray<Option<RGBA>>,
border_style_top: SlotArray<BorderStyle>,   // Per-side overrides
border_style_right: SlotArray<BorderStyle>,
border_style_bottom: SlotArray<BorderStyle>,
border_style_left: SlotArray<BorderStyle>,
border_color_top: SlotArray<Option<RGBA>>,
border_color_right: SlotArray<Option<RGBA>>,
border_color_bottom: SlotArray<Option<RGBA>>,
border_color_left: SlotArray<Option<RGBA>>,

// Focus
show_focus_ring: SlotArray<bool>,
focus_ring_color: SlotArray<RGBA>,
```

#### 6. Text Arrays (5) - Use TrackedSlotArray!
```rust
text_content: TrackedSlotArray<String>,     // Per-index dirty tracking
text_attrs: SlotArray<CellAttrs>,           // Bold, italic, underline
text_align: SlotArray<TextAlign>,           // Left, Center, Right
text_wrap: SlotArray<TextWrap>,             // NoWrap, Wrap, Truncate
ellipsis: SlotArray<String>,                // "..." or custom
```

#### 7. Interaction Arrays (15)
```rust
// Scroll
scroll_offset_y: SlotArray<i32>,
scroll_offset_x: SlotArray<i32>,

// Focus
focusable: SlotArray<bool>,
tab_index: SlotArray<i32>,
focused_index: Signal<i32>,                 // GLOBAL - single signal

// Mouse
hovered: SlotArray<bool>,
pressed: SlotArray<bool>,
mouse_enabled: SlotArray<bool>,

// Text cursor
cursor_position: SlotArray<usize>,
selection_start: SlotArray<i32>,            // -1 = no selection
selection_end: SlotArray<i32>,

// Cursor styling
cursor_char: SlotArray<char>,
cursor_blink_fps: SlotArray<u8>,
cursor_visible: SlotArray<bool>,
cursor_fg: SlotArray<Option<RGBA>>,
cursor_bg: SlotArray<Option<RGBA>>,
```

### Dirty Tracking

```rust
dirty_text: ReactiveSet<usize>,        // Text content changes
dirty_layout: ReactiveSet<usize>,      // Dimension/spacing/layout changes
dirty_visual: ReactiveSet<usize>,      // Color/border changes
dirty_hierarchy: ReactiveSet<usize>,   // Parent/child changes
dirty_scroll: ReactiveSet<usize>,      // Scroll position changes
```

**Usage**: Skip expensive computations when `dirty_set.is_empty()`

---

## Layout Engine (TITAN)

### 5-Pass Algorithm

1. **Build Tree** - Create linked-list hierarchy (firstChild, nextSibling)
2. **BFS Order** - Breadth-first traversal for processing order
3. **Measure** - Bottom-up intrinsic size calculation (text measurement cached)
4. **Position** - Top-down flexbox positioning
5. **Absolute** - Position absolute elements against containing block

### Output Structure
```rust
struct ComputedLayout {
    x: Vec<i32>,
    y: Vec<i32>,
    width: Vec<i32>,
    height: Vec<i32>,
    scrollable: Vec<bool>,
    max_scroll_x: Vec<i32>,
    max_scroll_y: Vec<i32>,
    content_width: i32,
    content_height: i32,
}
```

### Key Optimizations
- Module-level array reuse (zero allocation per frame)
- Text measurement caching (hash + length + available_width)
- Linked-list tree (no object allocation)

---

## FrameBuffer & Rendering

### Cell Structure
```rust
struct Cell {
    char: char,
    fg: RGBA,
    bg: RGBA,
    attrs: CellAttrs,  // Bold, italic, underline, etc.
}

struct FrameBuffer {
    width: usize,
    height: usize,
    cells: Vec<Vec<Cell>>,  // cells[y][x]
}
```

### Hit Regions (for mouse)
```rust
struct HitRegion {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    component_index: usize,
}
```

### Rendering Order
1. Sort components by z_index
2. For each component:
   - Apply scroll offset from ancestors
   - Intersect with parent clip rect
   - Fill background (with alpha blending)
   - Draw borders
   - Draw content (text, input cursor, progress bar)
   - Recurse to children

### Diff Renderer (Fullscreen Mode)
- Compare new buffer with previous
- Only output changed cells
- Stateful ANSI code emission (track fg/bg/attrs)
- Synchronized output (CSI ?2026h/l)

---

## Component System

### Component Pattern
```rust
pub fn box_component(props: BoxProps) -> Cleanup {
    // 1. Allocate index
    let index = allocate_index(props.id);

    // 2. Set component type
    core::component_type[index] = ComponentType::Box;

    // 3. Bind props to arrays (PRESERVES REACTIVITY!)
    core::visible.set_source(index, props.visible);
    visual::fg_color.set_source(index, props.fg);
    dimensions::width.set_source(index, props.width);
    layout::flex_direction.set_source(index, props.flex_direction);

    // 4. Register event handlers
    if let Some(on_key) = props.on_key {
        keyboard::on_focused(index, on_key);
    }
    if let Some(on_click) = props.on_click {
        mouse::on_component(index, on_click);
    }

    // 5. Render children with parent context
    if let Some(children) = props.children {
        push_parent_context(index);
        children();
        pop_parent_context();
    }

    // 6. Return cleanup
    Box::new(move || {
        release_index(index);
    })
}
```

### Props System
```rust
// Props can be static, signal, or getter
pub enum PropInput<T> {
    Static(T),
    Signal(Signal<T>),
    Getter(Box<dyn Fn() -> T>),
}

// Bind any input type to slot array
slot_array.set_source(index, prop_input);
```

### Component Types
```rust
enum ComponentType {
    None,
    Box,
    Text,
    Input,
    Progress,
    Select,
}
```

---

## State Modules

### Theme System
```rust
pub struct Theme {
    // Palette
    pub primary: ThemeColor,
    pub secondary: ThemeColor,
    pub tertiary: ThemeColor,
    pub accent: ThemeColor,

    // Semantic
    pub success: ThemeColor,
    pub warning: ThemeColor,
    pub error: ThemeColor,
    pub info: ThemeColor,

    // Text
    pub text: ThemeColor,
    pub text_muted: ThemeColor,
    pub text_disabled: ThemeColor,

    // Background
    pub background: ThemeColor,
    pub surface: ThemeColor,

    // Border
    pub border: ThemeColor,
    pub border_focus: ThemeColor,
}

// ThemeColor supports: null (terminal default), ANSI 0-255, RGB 0xRRGGBB
pub enum ThemeColor {
    Default,           // Terminal default
    Ansi(u8),          // 0-255 palette
    Rgb(u8, u8, u8),   // True color
}
```

### Variant Styles
```rust
pub enum Variant {
    Default, Primary, Secondary, Tertiary, Accent,
    Success, Warning, Error, Info,
    Muted, Surface, Elevated, Ghost, Outline,
}

pub struct VariantStyle {
    pub fg: RGBA,
    pub bg: RGBA,
    pub border: RGBA,
    pub border_focus: RGBA,
}
```

### Focus System
```rust
pub struct FocusManager {
    focused_index: Signal<i32>,              // Current focus (-1 = none)
    focus_history: Vec<(usize, String)>,     // For restoration
    focus_traps: Vec<usize>,                 // Modal stack
}

impl FocusManager {
    fn focus_next(&mut self);
    fn focus_previous(&mut self);
    fn focus(&mut self, index: usize);
    fn blur(&mut self);
    fn push_focus_trap(&mut self, container: usize);
    fn pop_focus_trap(&mut self);
}
```

### Keyboard System
```rust
type KeyHandler = Box<dyn Fn(KeyboardEvent) -> bool>;

pub struct KeyboardState {
    global_handlers: Vec<KeyHandler>,
    key_handlers: HashMap<String, Vec<KeyHandler>>,
    focused_handlers: HashMap<usize, Vec<KeyHandler>>,
    last_event: Signal<Option<KeyboardEvent>>,
}

// Dispatch order: key-specific → global → focused (separate)
```

### Mouse System
```rust
pub struct HitGrid {
    grid: Vec<i32>,  // Flat array, grid[y * width + x]
    width: usize,
    height: usize,
}

impl HitGrid {
    fn get(&self, x: i32, y: i32) -> i32;  // O(1) lookup
    fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, index: i32);
}

pub struct MouseState {
    hovered_component: i32,
    pressed_component: i32,
    pressed_button: MouseButton,
    mouse_x: Signal<i32>,
    mouse_y: Signal<i32>,
    is_mouse_down: Signal<bool>,
}
```

### Context System (Dependency Injection)
```rust
pub struct Context<T> {
    id: TypeId,
    default_value: T,
}

pub fn create_context<T: Default>(default: T) -> Context<T>;
pub fn provide<T>(context: &Context<T>, value: T);
pub fn use_context<T>(context: &Context<T>) -> T;
```

---

## Mount & Render Loop

### Initialization
```rust
pub fn mount<F: FnOnce()>(root: F, options: MountOptions) -> Cleanup {
    // 1. Set render mode
    render_mode.set(options.mode);

    // 2. Setup terminal (alt screen, hide cursor, enable mouse)
    setup_terminal(&options);

    // 3. Initialize input handlers
    global_keys::initialize();

    // 4. Create component tree
    root();

    // 5. Create THE SINGLE RENDER EFFECT
    let stop_effect = effect(|| {
        // Read deriveds (triggers computation if needed)
        let layout = layout_derived.get();
        let FrameBufferResult { buffer, hit_regions, .. } = frame_buffer_derived.get();

        // Apply hit regions (side effect in effect, not derived!)
        hit_grid.clear();
        for region in hit_regions {
            hit_grid.fill_rect(region.x, region.y, region.width, region.height, region.component_index);
        }

        // Render to terminal
        match render_mode.get() {
            RenderMode::Fullscreen => diff_renderer.render(&buffer),
            RenderMode::Inline => inline_renderer.render(&buffer),
            RenderMode::Append => append_renderer.render(&buffer),
        }
    });

    // 6. Return cleanup
    Box::new(move || {
        stop_effect();
        restore_terminal();
    })
}
```

### Render Modes
- **Fullscreen**: Alt screen, fixed dimensions, diff rendering
- **Inline**: Normal buffer, content height, full redraws
- **Append**: CLI-style with history preservation

---

## Rust Implementation Strategy

### Phase 1: Core Infrastructure
1. Create `spark-tui` crate with its own git repo
2. Set up parallel arrays module
3. Implement component registry (allocate/release indices)
4. Create basic `box`, `text` primitives

### Phase 2: Layout Engine
1. Port TITAN 5-pass algorithm
2. Implement flexbox calculations
3. Add text measurement (unicode-width crate)
4. Cache intrinsic sizes

### Phase 3: Rendering
1. Implement FrameBuffer and Cell types
2. Create recursive component renderer
3. Build DiffRenderer with ANSI output
4. Add HitGrid for mouse detection

### Phase 4: Input & State
1. Port keyboard handling (crossterm events)
2. Implement mouse tracking
3. Add focus management
4. Create theme system

### Phase 5: Polish
1. Add remaining components (Input, Progress, Select)
2. Implement context system
3. Add effect scopes for lifecycle
4. Write comprehensive tests

### Key Rust Considerations
- Use `crossterm` for terminal I/O
- `unicode-width` for text measurement
- `Rc<RefCell<T>>` or `RefCell<T>` for interior mutability
- Consider `parking_lot` for better RefCell performance
- Use `TypeId` for context system

---

## API Design Goals

### TypeScript-like Ergonomics
```rust
// The dream API
fn counter() -> Cleanup {
    let count = signal(0);

    box_component(BoxProps {
        padding: 1,
        border: BorderStyle::Single,
        children: Some(Box::new(|| {
            text(TextProps {
                content: derived(move || format!("Count: {}", count.get())),
                ..Default::default()
            });

            box_component(BoxProps {
                on_click: Some(Box::new(move |_| {
                    count.update(|n| n + 1);
                })),
                children: Some(Box::new(|| {
                    text(TextProps {
                        content: "Click me".into(),
                        ..Default::default()
                    });
                })),
                ..Default::default()
            });
        })),
        ..Default::default()
    })
}

fn main() {
    mount(counter, MountOptions::default());
}
```

### With Macros (Future Enhancement)
```rust
// Potential macro-based DSL
tui! {
    box(padding=1, border=single) {
        text(content=format!("Count: {}", count.get()))
        box(on_click=|_| count.update(|n| n + 1)) {
            text(content="Click me")
        }
    }
}
```

---

## Performance Targets

Based on TypeScript benchmarks:
- Layout: < 1ms for 100 components
- Buffer generation: < 1ms
- Diff render: < 2ms
- Total frame time: < 5ms (200+ FPS capability)

Rust should achieve 2-5x improvement over TypeScript.

---

## File Structure

```
spark-tui/
├── Cargo.toml
├── LICENSE
├── README.md
├── CLAUDE.md
├── src/
│   ├── lib.rs
│   ├── engine/
│   │   ├── mod.rs
│   │   ├── registry.rs        # Index allocation
│   │   ├── arrays/
│   │   │   ├── mod.rs
│   │   │   ├── core.rs
│   │   │   ├── dimensions.rs
│   │   │   ├── spacing.rs
│   │   │   ├── layout.rs
│   │   │   ├── visual.rs
│   │   │   ├── text.rs
│   │   │   └── interaction.rs
│   │   └── dirty.rs           # Dirty tracking sets
│   ├── pipeline/
│   │   ├── mod.rs
│   │   ├── layout.rs          # TITAN engine
│   │   ├── frame_buffer.rs
│   │   └── types.rs
│   ├── primitives/
│   │   ├── mod.rs
│   │   ├── box.rs
│   │   ├── text.rs
│   │   ├── input.rs
│   │   └── types.rs           # Props definitions
│   ├── renderer/
│   │   ├── mod.rs
│   │   ├── buffer.rs
│   │   ├── ansi.rs
│   │   ├── diff.rs
│   │   └── output.rs
│   ├── state/
│   │   ├── mod.rs
│   │   ├── theme.rs
│   │   ├── focus.rs
│   │   ├── keyboard.rs
│   │   ├── mouse.rs
│   │   └── context.rs
│   └── api/
│       ├── mod.rs
│       └── mount.rs
├── examples/
│   ├── hello.rs
│   ├── counter.rs
│   └── dashboard.rs
└── tests/
    └── basic.rs
```

---

## References

- TypeScript TUI: `/Users/rusty/Documents/Projects/TUI/tui/`
- TypeScript Signals: `/Users/rusty/Documents/Projects/AI/Tools/ClaudeTools/memory-ts/packages/signals/`
- Rust spark-signals: `/Users/rusty/Documents/Projects/TUI/tui-rust/crates/signals/`
- spark-signals on crates.io: `https://crates.io/crates/spark-signals`
