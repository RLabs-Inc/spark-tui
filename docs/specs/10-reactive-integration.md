# 10. Reactive Integration Specification

## Overview

This specification defines how `crates/signals` integrates with every layer of the TUI framework to create a fully reactive terminal UI system. The TypeScript implementation achieves **complete reactivity**: from component props through layout computation to individual characters on screen—all driven by a single unified effect system.

## Core Philosophy: The ONE Effect Pattern

### Single Render Effect

The entire rendering pipeline is driven by **ONE effect** that observes all reactive dependencies:

```rust
// The ONE effect - drives everything
pub struct RenderLoop {
    /// Single effect that tracks all reactive dependencies
    render_effect: Effect<()>,

    /// Terminal handle
    terminal: Terminal,

    /// Root component index
    root: usize,
}

impl RenderLoop {
    pub fn new(terminal: Terminal, root: usize) -> Self {
        let render_effect = Effect::new(move || {
            // This single closure:
            // 1. Observes all Signal<T> props on all components
            // 2. Runs complete layout computation
            // 3. Renders to frame buffer
            // 4. Diffs and outputs to terminal

            // The magic: any Signal<T>.get() inside creates a dependency
            // When ANY signal changes, this entire effect re-runs
            render_frame();
        });

        Self { render_effect, terminal, root }
    }
}
```

### Why ONE Effect?

1. **Automatic Batching**: Multiple signal changes in one tick = one render
2. **Guaranteed Consistency**: No partial renders or torn states
3. **Simple Mental Model**: Change signal → frame renders
4. **Optimal Performance**: Framework controls when/how to update

---

## Signal Types Integration

### From `crates/signals`

```rust
// Core reactive primitives we use
pub use signals::{
    Signal,      // Read-write reactive value
    Derived,     // Computed value from other signals
    Effect,      // Side effect that tracks dependencies
    Memo,        // Cached derived value
    batch,       // Group multiple updates
    untrack,     // Read without creating dependency
};
```

### Reactive<T> Wrapper

Central abstraction for props that can be static OR reactive:

```rust
/// A value that may be static or reactive
pub enum Reactive<T> {
    /// Static value - never changes
    Static(T),

    /// Signal - can change, creates dependency when read
    Signal(Signal<T>),

    /// Derived - computed from other signals
    Derived(Derived<T>),
}

impl<T: Clone> Reactive<T> {
    /// Get current value, tracking dependency if reactive
    pub fn get(&self) -> T {
        match self {
            Reactive::Static(v) => v.clone(),
            Reactive::Signal(s) => s.get(),
            Reactive::Derived(d) => d.get(),
        }
    }

    /// Get current value WITHOUT tracking (for conditional reads)
    pub fn get_untracked(&self) -> T {
        untrack(|| self.get())
    }

    /// Check if this is reactive (for optimization)
    pub fn is_reactive(&self) -> bool {
        !matches!(self, Reactive::Static(_))
    }
}

// Convenient conversions
impl<T> From<T> for Reactive<T> {
    fn from(value: T) -> Self {
        Reactive::Static(value)
    }
}

impl<T> From<Signal<T>> for Reactive<T> {
    fn from(signal: Signal<T>) -> Self {
        Reactive::Signal(signal)
    }
}

impl<T> From<Derived<T>> for Reactive<T> {
    fn from(derived: Derived<T>) -> Self {
        Reactive::Derived(derived)
    }
}
```

---

## SlotArray<T>: Reactive Parallel Arrays

### Core Structure

```rust
/// Reactive array where each slot can be static or reactive
pub struct SlotArray<T> {
    /// Source for each slot's value
    sources: Vec<SlotSource<T>>,

    /// Cached values (updated on access or notification)
    values: Vec<T>,

    /// Generation counter for change detection
    generation: u64,
}

pub enum SlotSource<T> {
    /// Static value - set once, never changes
    Static,

    /// Reactive signal - may change
    Signal(Signal<T>),

    /// Derived computation
    Derived(Derived<T>),

    /// Inherited from parent (index stored)
    Inherited(usize),
}
```

### Reactive Reading

```rust
impl<T: Clone + Default> SlotArray<T> {
    /// Get value, creating reactive dependency if source is reactive
    pub fn get(&self, index: usize) -> T {
        match &self.sources[index] {
            SlotSource::Static => self.values[index].clone(),
            SlotSource::Signal(s) => s.get(),  // Creates dependency!
            SlotSource::Derived(d) => d.get(), // Creates dependency!
            SlotSource::Inherited(parent) => self.get(*parent),
        }
    }

    /// Get without creating dependency
    pub fn get_untracked(&self, index: usize) -> T {
        untrack(|| self.get(index))
    }

    /// Set static value
    pub fn set_static(&mut self, index: usize, value: T) {
        self.sources[index] = SlotSource::Static;
        self.values[index] = value;
        self.generation += 1;
    }

    /// Bind to reactive source
    pub fn bind(&mut self, index: usize, source: impl Into<Reactive<T>>) {
        match source.into() {
            Reactive::Static(v) => self.set_static(index, v),
            Reactive::Signal(s) => {
                self.sources[index] = SlotSource::Signal(s);
                self.generation += 1;
            }
            Reactive::Derived(d) => {
                self.sources[index] = SlotSource::Derived(d);
                self.generation += 1;
            }
        }
    }
}
```

---

## Component Prop Reactivity

### FlexNode Reactive Props

All 33 FlexNode slots support reactive binding:

```rust
impl FlexNode {
    /// Bind width to reactive source
    pub fn width(mut self, value: impl Into<Reactive<Dimension>>) -> Self {
        self.registry.width.bind(self.index, value);
        self
    }

    /// Bind height to reactive source
    pub fn height(mut self, value: impl Into<Reactive<Dimension>>) -> Self {
        self.registry.height.bind(self.index, value);
        self
    }

    /// Bind flex_grow to reactive source
    pub fn flex_grow(mut self, value: impl Into<Reactive<f32>>) -> Self {
        self.registry.flex_grow.bind(self.index, value);
        self
    }

    /// Bind background color to reactive source
    pub fn bg(mut self, value: impl Into<Reactive<Rgba>>) -> Self {
        self.registry.bg.bind(self.index, value);
        self
    }

    // ... all 33 slots follow this pattern
}
```

### Usage Examples

```rust
// Static props (no reactivity)
Box::new()
    .width(Dimension::Percent(100.0))
    .height(Dimension::Fixed(3))
    .bg(Rgba::rgb(30, 30, 30));

// Reactive props
let width_signal = Signal::new(Dimension::Percent(50.0));
let is_active = Signal::new(false);

let bg_derived = Derived::new(move || {
    if is_active.get() {
        Rgba::rgb(0, 100, 0)  // Green when active
    } else {
        Rgba::rgb(50, 50, 50) // Gray when inactive
    }
});

Box::new()
    .width(width_signal.clone())  // Reactive!
    .bg(bg_derived);              // Reactive!

// Later: changing signal triggers re-render
width_signal.set(Dimension::Percent(75.0));  // Frame re-renders
is_active.set(true);                          // Frame re-renders
```

---

## Derived Pipeline

### Layout as Derived Computation

```rust
/// Complete layout state - derived from all node properties
pub struct LayoutState {
    /// Computed X positions for all nodes
    pub x: Vec<i32>,

    /// Computed Y positions for all nodes
    pub y: Vec<i32>,

    /// Computed widths for all nodes
    pub computed_width: Vec<u16>,

    /// Computed heights for all nodes
    pub computed_height: Vec<u16>,

    /// Layout generation (for dirty checking)
    pub generation: u64,
}

impl LayoutState {
    /// Compute layout - called inside render effect
    /// Automatically tracks all reactive slot dependencies
    pub fn compute(registry: &FlexNodeRegistry, root: usize, viewport: Size) -> Self {
        // TITAN engine runs here
        // Every registry.width.get(i), registry.height.get(i), etc.
        // creates a reactive dependency

        let mut state = Self::new(registry.len());
        titan_layout(registry, root, viewport, &mut state);
        state
    }
}
```

### Frame Buffer as Derived

```rust
/// Frame buffer - derived from layout and visual properties
pub struct FrameBuffer {
    pub cells: Vec<Cell>,
    pub width: u16,
    pub height: u16,
}

impl FrameBuffer {
    /// Render all nodes to buffer - called inside render effect
    /// Automatically tracks visual property dependencies
    pub fn render(
        registry: &FlexNodeRegistry,
        layout: &LayoutState,
        root: usize,
    ) -> Self {
        let mut buffer = Self::new(layout.viewport_width, layout.viewport_height);

        // For each visible node:
        // - registry.bg.get(i) creates dependency on background color
        // - registry.fg.get(i) creates dependency on foreground color
        // - registry.text.get(i) creates dependency on text content
        // - etc.

        render_tree(registry, layout, root, &mut buffer);
        buffer
    }
}
```

---

## The Complete Render Effect

### Main Loop Structure

```rust
pub fn create_render_loop(
    terminal: &mut Terminal,
    registry: &FlexNodeRegistry,
    root: usize,
) -> Effect<()> {
    // Previous frame for diffing
    let prev_buffer: RefCell<Option<FrameBuffer>> = RefCell::new(None);

    // Stateful renderer for ANSI optimization
    let renderer: RefCell<StatefulCellRenderer> = RefCell::new(StatefulCellRenderer::new());

    Effect::new(move || {
        // 1. Get viewport size (may be reactive)
        let viewport = terminal.size();

        // 2. Compute layout (tracks all layout-affecting signals)
        let layout = LayoutState::compute(registry, root, viewport);

        // 3. Render to buffer (tracks all visual signals)
        let buffer = FrameBuffer::render(registry, &layout, root);

        // 4. Diff with previous frame
        let mut prev = prev_buffer.borrow_mut();
        let mut rend = renderer.borrow_mut();

        let output = match prev.as_ref() {
            Some(old) => rend.render_diff(old, &buffer),
            None => rend.render_full(&buffer),
        };

        // 5. Write to terminal
        terminal.write(&output);
        terminal.flush();

        // 6. Store for next diff
        *prev = Some(buffer);
    })
}
```

### Dependency Graph Visualization

```
User Changes Signal
        │
        ▼
┌──────────────────┐
│  Signal<T>.set() │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Marks dependents │
│    as dirty      │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│   Render Effect  │◄─── THE ONE EFFECT
│    re-executes   │
└────────┬─────────┘
         │
    ┌────┴────┐
    ▼         ▼
┌───────┐ ┌───────────┐
│Layout │ │Frame Buff │
│Compute│ │  Render   │
└───┬───┘ └─────┬─────┘
    │           │
    └─────┬─────┘
          ▼
    ┌───────────┐
    │   Diff    │
    └─────┬─────┘
          ▼
    ┌───────────┐
    │  Output   │
    │  to TTY   │
    └───────────┘
```

---

## Global State Signals

### Application State

```rust
/// Global application state - all reactive
pub struct AppState {
    /// Currently focused node index
    pub focused: Signal<Option<usize>>,

    /// Focus history stack
    pub focus_history: Signal<Vec<usize>>,

    /// Current theme
    pub theme: Signal<Theme>,

    /// Terminal dimensions
    pub viewport: Signal<Size>,

    /// Whether terminal has focus
    pub terminal_focused: Signal<bool>,

    /// Current cursor position (if visible)
    pub cursor_position: Signal<Option<(u16, u16)>>,

    /// Global cursor blink state
    pub cursor_visible: Signal<bool>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            focused: Signal::new(None),
            focus_history: Signal::new(Vec::new()),
            theme: Signal::new(Theme::default()),
            viewport: Signal::new(Size { width: 80, height: 24 }),
            terminal_focused: Signal::new(true),
            cursor_position: Signal::new(None),
            cursor_visible: Signal::new(true),
        }
    }
}
```

### Derived State

```rust
/// Derived values computed from app state
pub struct DerivedState {
    /// Is any input focused?
    pub has_input_focus: Derived<bool>,

    /// Current theme colors (resolved)
    pub resolved_colors: Derived<ResolvedColors>,

    /// Should show cursor?
    pub show_cursor: Derived<bool>,
}

impl DerivedState {
    pub fn new(app: &AppState, registry: &FlexNodeRegistry) -> Self {
        let focused = app.focused.clone();
        let registry_clone = registry.clone();

        let has_input_focus = Derived::new(move || {
            focused.get().map_or(false, |idx| {
                registry_clone.is_input.get_untracked(idx)
            })
        });

        let theme = app.theme.clone();
        let resolved_colors = Derived::new(move || {
            theme.get().resolve()
        });

        let cursor_visible = app.cursor_visible.clone();
        let terminal_focused = app.terminal_focused.clone();
        let has_input = has_input_focus.clone();

        let show_cursor = Derived::new(move || {
            cursor_visible.get()
                && terminal_focused.get()
                && has_input.get()
        });

        Self {
            has_input_focus,
            resolved_colors,
            show_cursor,
        }
    }
}
```

---

## Batching Updates

### Automatic Batching

```rust
// Multiple updates in same synchronous block = one render
fn handle_key_event(key: KeyEvent, state: &AppState) {
    // These all happen in one tick
    state.focused.set(Some(5));
    state.theme.set(Theme::dark());
    update_input_value(key);

    // Render effect runs ONCE after this function returns
}
```

### Explicit Batching

```rust
use signals::batch;

// Force multiple async updates into one render
fn handle_complex_update(state: &AppState) {
    batch(|| {
        // All these updates are batched
        for i in 0..100 {
            update_item(i);
        }
        state.focused.set(Some(50));
    });
    // Single render happens here
}
```

### Batch Scope

```rust
/// Scope that automatically batches all updates within
pub struct BatchScope {
    // Internal tracking
}

impl BatchScope {
    pub fn new() -> Self {
        signals::begin_batch();
        Self {}
    }
}

impl Drop for BatchScope {
    fn drop(&mut self) {
        signals::end_batch();
        // Render happens when last batch scope drops
    }
}

// Usage
fn complex_operation() {
    let _batch = BatchScope::new();

    // All updates batched
    do_many_updates();

    // Render happens when _batch drops
}
```

---

## Effect Cleanup

### Cleanup Pattern

```rust
/// Effect with cleanup function
pub fn create_effect_with_cleanup<F, C>(effect_fn: F) -> Effect<()>
where
    F: Fn() -> C + 'static,
    C: FnOnce() + 'static,
{
    let cleanup: RefCell<Option<Box<dyn FnOnce()>>> = RefCell::new(None);

    Effect::new(move || {
        // Run previous cleanup
        if let Some(c) = cleanup.borrow_mut().take() {
            c();
        }

        // Run effect and get new cleanup
        let new_cleanup = effect_fn();
        *cleanup.borrow_mut() = Some(Box::new(new_cleanup));
    })
}

// Usage: Timer effect with cleanup
let timer_effect = create_effect_with_cleanup(|| {
    let interval = set_interval(Duration::from_millis(500), || {
        cursor_visible.update(|v| !v);
    });

    // Cleanup: cancel timer when effect re-runs or disposes
    move || interval.cancel()
});
```

### Component Lifecycle

```rust
/// Component mount/unmount tracking
pub struct ComponentLifecycle {
    /// Called when component mounts
    on_mount: Option<Box<dyn Fn()>>,

    /// Called when component unmounts
    on_unmount: Option<Box<dyn FnOnce()>>,

    /// Active effects for this component
    effects: Vec<Effect<()>>,
}

impl ComponentLifecycle {
    pub fn mount(&mut self) {
        if let Some(ref on_mount) = self.on_mount {
            on_mount();
        }
    }

    pub fn unmount(mut self) {
        // Dispose all effects
        for effect in self.effects.drain(..) {
            effect.dispose();
        }

        // Run unmount callback
        if let Some(on_unmount) = self.on_unmount.take() {
            on_unmount();
        }
    }
}
```

---

## Conditional Rendering (Show/When)

### Show Component Reactivity

```rust
/// Reactive conditional rendering
pub struct Show<T> {
    /// Condition signal
    condition: Reactive<bool>,

    /// Child component (lazily created)
    child: RefCell<Option<T>>,

    /// Placeholder when hidden
    placeholder: Option<Box<dyn Fn() -> T>>,
}

impl<T: Component> Show<T> {
    pub fn render(&self, registry: &mut FlexNodeRegistry) -> Option<usize> {
        // This creates dependency on condition!
        let visible = self.condition.get();

        if visible {
            let mut child = self.child.borrow_mut();
            if child.is_none() {
                // Lazily create child
                *child = Some(self.create_child());
            }
            child.as_ref().map(|c| c.render(registry))
        } else {
            // Unmount child if exists
            if let Some(child) = self.child.borrow_mut().take() {
                child.unmount();
            }

            // Render placeholder if provided
            self.placeholder.as_ref().map(|p| p().render(registry))
        }
    }
}
```

### When (Switch) Component

```rust
/// Reactive switch/case rendering
pub struct When<T, V> {
    /// Value to match against
    value: Reactive<V>,

    /// Match arms: (pattern, component_factory)
    arms: Vec<(V, Box<dyn Fn() -> T>)>,

    /// Default arm
    default: Option<Box<dyn Fn() -> T>>,

    /// Currently rendered arm
    current: RefCell<Option<(V, T)>>,
}

impl<T: Component, V: PartialEq + Clone> When<T, V> {
    pub fn render(&self, registry: &mut FlexNodeRegistry) -> Option<usize> {
        // Creates dependency on value!
        let current_value = self.value.get();

        // Check if we need to switch
        let needs_switch = self.current.borrow()
            .as_ref()
            .map_or(true, |(v, _)| *v != current_value);

        if needs_switch {
            // Unmount previous
            if let Some((_, old)) = self.current.borrow_mut().take() {
                old.unmount();
            }

            // Find matching arm
            let factory = self.arms.iter()
                .find(|(pattern, _)| *pattern == current_value)
                .map(|(_, f)| f)
                .or(self.default.as_ref());

            if let Some(f) = factory {
                let component = f();
                let index = component.render(registry);
                *self.current.borrow_mut() = Some((current_value, component));
                return index;
            }
        }

        // Render current
        self.current.borrow().as_ref().map(|(_, c)| c.render(registry))
    }
}
```

---

## List Rendering (Each)

### Each Component Reactivity

```rust
/// Reactive list rendering with keyed reconciliation
pub struct Each<T, K, C> {
    /// Source data (reactive)
    items: Reactive<Vec<T>>,

    /// Key extractor
    key_fn: Box<dyn Fn(&T) -> K>,

    /// Component factory
    render_fn: Box<dyn Fn(&T, usize) -> C>,

    /// Currently rendered items by key
    rendered: RefCell<IndexMap<K, (T, C)>>,
}

impl<T: Clone, K: Hash + Eq + Clone, C: Component> Each<T, K, C> {
    pub fn render(&self, registry: &mut FlexNodeRegistry) -> Vec<usize> {
        // Creates dependency on items list!
        let items = self.items.get();

        let mut new_rendered = IndexMap::new();
        let mut indices = Vec::new();
        let mut old_rendered = self.rendered.borrow_mut();

        for (i, item) in items.iter().enumerate() {
            let key = (self.key_fn)(item);

            let component = if let Some((_, existing)) = old_rendered.remove(&key) {
                // Reuse existing component
                existing
            } else {
                // Create new component
                (self.render_fn)(item, i)
            };

            let index = component.render(registry);
            indices.push(index);
            new_rendered.insert(key, (item.clone(), component));
        }

        // Unmount removed items
        for (_, (_, component)) in old_rendered.drain(..) {
            component.unmount();
        }

        *old_rendered = new_rendered;
        indices
    }
}
```

### List Diffing Strategy

```rust
/// Efficient list reconciliation
pub enum ListDiff<K> {
    /// No changes
    None,

    /// Items appended to end
    Append(usize),

    /// Items removed from end
    Truncate(usize),

    /// Items inserted at position
    Insert(usize, usize),

    /// Items removed at position
    Remove(usize, usize),

    /// Items moved
    Move(Vec<(usize, usize)>),

    /// Full rebuild needed
    Full,
}

impl<K: Hash + Eq> ListDiff<K> {
    pub fn compute(old_keys: &[K], new_keys: &[K]) -> Self {
        // LCS-based diffing for optimal moves
        // Returns minimal set of operations
        todo!()
    }
}
```

---

## Animation Integration

### Animated Values

```rust
/// Reactive animated value
pub struct Animated<T> {
    /// Current value (reactive)
    current: Signal<T>,

    /// Target value
    target: Signal<T>,

    /// Animation progress (0.0 - 1.0)
    progress: Signal<f32>,

    /// Easing function
    easing: EasingFn,

    /// Duration
    duration: Duration,

    /// Animation effect
    effect: Option<Effect<()>>,
}

impl<T: Interpolate + Clone + 'static> Animated<T> {
    pub fn new(initial: T) -> Self {
        Self {
            current: Signal::new(initial.clone()),
            target: Signal::new(initial),
            progress: Signal::new(1.0),
            easing: EasingFn::EaseInOut,
            duration: Duration::from_millis(200),
            effect: None,
        }
    }

    /// Animate to new value
    pub fn animate_to(&self, value: T) {
        self.target.set(value);
        self.progress.set(0.0);

        // Start animation effect
        self.start_animation();
    }

    fn start_animation(&self) {
        let current = self.current.clone();
        let target = self.target.clone();
        let progress = self.progress.clone();
        let easing = self.easing;
        let duration = self.duration;
        let start = Instant::now();
        let start_value = current.get();

        // Animation tick effect
        create_animation_frame(move || {
            let elapsed = start.elapsed();
            let t = (elapsed.as_secs_f32() / duration.as_secs_f32()).min(1.0);
            let eased_t = easing.apply(t);

            let interpolated = start_value.interpolate(&target.get(), eased_t);
            current.set(interpolated);
            progress.set(t);

            // Continue animation?
            t < 1.0
        });
    }

    /// Get current value (reactive)
    pub fn get(&self) -> T {
        self.current.get()
    }
}

/// Trait for interpolatable values
pub trait Interpolate {
    fn interpolate(&self, target: &Self, t: f32) -> Self;
}

impl Interpolate for f32 {
    fn interpolate(&self, target: &Self, t: f32) -> Self {
        self + (target - self) * t
    }
}

impl Interpolate for Rgba {
    fn interpolate(&self, target: &Self, t: f32) -> Self {
        Rgba {
            r: (self.r as f32 + (target.r as f32 - self.r as f32) * t) as i16,
            g: (self.g as f32 + (target.g as f32 - self.g as f32) * t) as i16,
            b: (self.b as f32 + (target.b as f32 - self.b as f32) * t) as i16,
            a: (self.a as f32 + (target.a as f32 - self.a as f32) * t) as i16,
        }
    }
}
```

---

## Terminal Lifecycle

### Mount Sequence

```rust
pub struct App {
    registry: FlexNodeRegistry,
    state: AppState,
    terminal: Terminal,
    render_effect: Option<Effect<()>>,
    event_loop: Option<EventLoop>,
}

impl App {
    /// Mount application to terminal
    pub fn mount(root: impl Component) -> Result<Self, Error> {
        // 1. Initialize terminal
        let mut terminal = Terminal::new()?;
        terminal.enable_raw_mode()?;
        terminal.enable_mouse()?;
        terminal.enter_alternate_screen()?;
        terminal.hide_cursor()?;

        // 2. Create registry and state
        let mut registry = FlexNodeRegistry::new();
        let state = AppState::new();

        // 3. Initial viewport
        let size = terminal.size()?;
        state.viewport.set(size);

        // 4. Render root component
        let root_index = root.render(&mut registry);

        // 5. Create render effect (THE ONE EFFECT)
        let render_effect = create_render_loop(
            &mut terminal,
            &registry,
            root_index,
        );

        // 6. Start event loop
        let event_loop = EventLoop::new(&terminal, &state, &registry)?;

        Ok(Self {
            registry,
            state,
            terminal,
            render_effect: Some(render_effect),
            event_loop: Some(event_loop),
        })
    }

    /// Run application until exit
    pub fn run(&mut self) -> Result<(), Error> {
        self.event_loop.as_mut().unwrap().run()
    }
}

impl Drop for App {
    fn drop(&mut self) {
        // Cleanup in reverse order

        // 1. Stop event loop
        self.event_loop.take();

        // 2. Dispose render effect
        if let Some(effect) = self.render_effect.take() {
            effect.dispose();
        }

        // 3. Restore terminal
        let _ = self.terminal.show_cursor();
        let _ = self.terminal.leave_alternate_screen();
        let _ = self.terminal.disable_mouse();
        let _ = self.terminal.disable_raw_mode();
    }
}
```

### Event Loop Integration

```rust
pub struct EventLoop {
    terminal: Terminal,
    state: AppState,
    registry: FlexNodeRegistry,
    running: bool,
}

impl EventLoop {
    pub fn run(&mut self) -> Result<(), Error> {
        self.running = true;

        while self.running {
            // 1. Poll for events (with timeout for animations)
            if let Some(event) = self.terminal.poll_event(Duration::from_millis(16))? {
                // 2. Batch all event processing
                batch(|| {
                    match event {
                        Event::Key(key) => self.handle_key(key),
                        Event::Mouse(mouse) => self.handle_mouse(mouse),
                        Event::Resize(w, h) => {
                            self.state.viewport.set(Size { width: w, height: h });
                        }
                        Event::FocusGained => {
                            self.state.terminal_focused.set(true);
                        }
                        Event::FocusLost => {
                            self.state.terminal_focused.set(false);
                        }
                    }
                });
                // Single render after batch
            }

            // 3. Process any pending animations
            process_animation_frames();
        }

        Ok(())
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
```

---

## History System

### State History for Undo/Redo

```rust
/// History tracking for state changes
pub struct History<T> {
    /// Past states
    past: Signal<Vec<T>>,

    /// Current state
    present: Signal<T>,

    /// Future states (for redo)
    future: Signal<Vec<T>>,

    /// Maximum history size
    max_size: usize,
}

impl<T: Clone> History<T> {
    pub fn new(initial: T, max_size: usize) -> Self {
        Self {
            past: Signal::new(Vec::new()),
            present: Signal::new(initial),
            future: Signal::new(Vec::new()),
            max_size,
        }
    }

    /// Get current value (reactive)
    pub fn get(&self) -> T {
        self.present.get()
    }

    /// Update value (clears future, adds to past)
    pub fn set(&self, value: T) {
        batch(|| {
            // Add current to past
            self.past.update(|past| {
                past.push(self.present.get_untracked());
                if past.len() > self.max_size {
                    past.remove(0);
                }
            });

            // Clear future
            self.future.set(Vec::new());

            // Set new present
            self.present.set(value);
        });
    }

    /// Undo to previous state
    pub fn undo(&self) -> bool {
        let past = self.past.get_untracked();
        if past.is_empty() {
            return false;
        }

        batch(|| {
            // Move present to future
            self.future.update(|future| {
                future.push(self.present.get_untracked());
            });

            // Pop from past to present
            self.past.update(|past| {
                if let Some(prev) = past.pop() {
                    self.present.set(prev);
                }
            });
        });

        true
    }

    /// Redo to next state
    pub fn redo(&self) -> bool {
        let future = self.future.get_untracked();
        if future.is_empty() {
            return false;
        }

        batch(|| {
            // Move present to past
            self.past.update(|past| {
                past.push(self.present.get_untracked());
            });

            // Pop from future to present
            self.future.update(|future| {
                if let Some(next) = future.pop() {
                    self.present.set(next);
                }
            });
        });

        true
    }

    /// Can undo?
    pub fn can_undo(&self) -> Derived<bool> {
        let past = self.past.clone();
        Derived::new(move || !past.get().is_empty())
    }

    /// Can redo?
    pub fn can_redo(&self) -> Derived<bool> {
        let future = self.future.clone();
        Derived::new(move || !future.get().is_empty())
    }
}
```

---

## Complete Data Flow

### End-to-End Reactive Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                         USER INPUT                               │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                      RAW TERMINAL INPUT                          │
│  • Keyboard bytes → KeyEvent                                     │
│  • Mouse bytes → MouseEvent                                      │
│  • Resize SIGWINCH → ResizeEvent                                │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                       EVENT DISPATCH                             │
│  • Hit testing for mouse events                                  │
│  • Focus-based routing for keyboard                              │
│  • Global handlers (shortcuts)                                   │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                      SIGNAL UPDATES                              │
│  • state.focused.set(new_focus)                                 │
│  • input_value.set(new_text)                                    │
│  • scroll_offset.set(new_offset)                                │
│  • Batched within single event handler                          │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    THE ONE RENDER EFFECT                         │
│  • Triggered by ANY signal dependency change                     │
│  • Runs after batch completes                                    │
└─────────────────────────────────────────────────────────────────┘
                                │
                    ┌───────────┴───────────┐
                    ▼                       ▼
┌─────────────────────────────┐ ┌─────────────────────────────────┐
│       LAYOUT COMPUTE        │ │      DERIVED STATE COMPUTE       │
│  • Read all Dimension slots │ │  • Theme resolution              │
│  • Read all Spacing slots   │ │  • Cursor visibility             │
│  • Run TITAN algorithm      │ │  • Focus indicators              │
│  • Compute x, y, w, h       │ │  • Active styles                 │
└─────────────────────────────┘ └─────────────────────────────────┘
                    │                       │
                    └───────────┬───────────┘
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                      FRAME BUFFER RENDER                         │
│  • Read all Visual slots (bg, fg, text)                         │
│  • Read all Text slots (content, wrap)                          │
│  • Apply computed layout positions                               │
│  • Fill cell grid with characters + attributes                   │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                        DIFFERENTIAL DIFF                         │
│  • Compare with previous frame buffer                            │
│  • Identify changed cells only                                   │
│  • Group into efficient ANSI sequences                           │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                       ANSI OUTPUT                                │
│  • StatefulCellRenderer tracks cursor/style state               │
│  • Minimal escape sequences generated                            │
│  • Single write() to terminal                                    │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                      TERMINAL DISPLAY                            │
│  • Characters appear on screen                                   │
│  • Hardware cursor positioned (if visible)                       │
│  • User sees updated UI                                          │
└─────────────────────────────────────────────────────────────────┘
```

---

## Performance Characteristics

### What Makes This Fast

1. **Single Effect Execution**: No cascading effects or effect storms
2. **Automatic Dependency Pruning**: Only accessed signals create dependencies
3. **Batched Updates**: Multiple changes = one render
4. **Differential Rendering**: Only changed cells go to terminal
5. **Integer-Only Layout**: No floating point in hot path
6. **Pre-allocated Buffers**: No allocations during render

### Memory Model

```rust
/// Memory layout optimized for cache efficiency
pub struct OptimizedRegistry {
    // Hot data: read every frame
    hot: HotArrays,

    // Warm data: read when visible
    warm: WarmArrays,

    // Cold data: read on interaction
    cold: ColdArrays,
}

struct HotArrays {
    // Layout: always needed
    x: Vec<i32>,
    y: Vec<i32>,
    width: Vec<u16>,
    height: Vec<u16>,

    // Visual: always needed
    bg: SlotArray<Rgba>,
    fg: SlotArray<Rgba>,
    visible: Vec<bool>,
}

struct WarmArrays {
    // Text: needed for text nodes
    text: SlotArray<String>,
    text_wrap: SlotArray<TextWrap>,

    // Borders: needed for bordered nodes
    border_style: SlotArray<BorderStyle>,
}

struct ColdArrays {
    // Interaction: needed on events
    focusable: Vec<bool>,
    on_key: Vec<Option<KeyHandler>>,
    on_click: Vec<Option<ClickHandler>>,
}
```

---

## Migration Checklist

### From TypeScript Implementation

- [ ] Port `Signal<T>` usage to `crates/signals::Signal<T>`
- [ ] Port `Derived<T>` usage to `crates/signals::Derived<T>`
- [ ] Port `Effect` usage to `crates/signals::Effect<()>`
- [ ] Implement `Reactive<T>` enum wrapper
- [ ] Implement `SlotArray<T>` with reactive sources
- [ ] Create single render effect (THE ONE EFFECT)
- [ ] Implement batch scoping
- [ ] Port lifecycle management
- [ ] Implement animation frame scheduling
- [ ] Port history system

### Integration Points

- [ ] All FlexNode slots accept `impl Into<Reactive<T>>`
- [ ] Layout reads from reactive slots
- [ ] Frame buffer reads from reactive slots
- [ ] Event handlers update signals
- [ ] Derived state computes from signals
- [ ] Animations interpolate signals
- [ ] Conditional rendering tracks conditions
- [ ] List rendering tracks items

---

## API Summary

```rust
// Core types from crates/signals
use signals::{Signal, Derived, Effect, Memo, batch, untrack};

// TUI reactive wrapper
pub enum Reactive<T> { Static(T), Signal(Signal<T>), Derived(Derived<T>) }

// Reactive arrays
pub struct SlotArray<T> { /* ... */ }

// Global state
pub struct AppState { /* all Signal<T> fields */ }

// Mount API
pub fn mount(root: impl Component) -> App;

// Component trait
pub trait Component {
    fn render(&self, registry: &mut FlexNodeRegistry) -> Option<usize>;
    fn unmount(self);
}

// Animation
pub struct Animated<T> { /* ... */ }

// History
pub struct History<T> { /* ... */ }
```

This completes the reactive integration specification, defining how `crates/signals` powers every layer of the TUI framework from props to pixels.
