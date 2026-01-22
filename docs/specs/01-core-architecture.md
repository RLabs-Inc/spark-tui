# Core Architecture Specification

## Overview

This specification documents the core architecture of the TUI framework, covering FlexNode structure, parallel arrays pattern, registry system, node lifecycle, and integration with the Rust signals crate.

**Source Files Analyzed:**
- `/Users/rusty/Documents/Projects/TUI/tui/src/engine/FlexNode.ts`
- `/Users/rusty/Documents/Projects/TUI/tui/src/engine/flexNodeRegistry.ts`
- `/Users/rusty/Documents/Projects/TUI/tui/src/engine/inheritance.ts`

---

## 1. Architecture Philosophy

The TUI framework uses an **ECS-style (Entity-Component-System) design** with:

- **Entities**: Component indices (integers)
- **Components**: Parallel arrays storing properties by index
- **Systems**: Derived computations (layout, rendering) that read arrays

**Key Insight**: This is the "father state pattern" - parallel arrays where each index represents an entity and each array represents a component type.

---

## 2. FlexNode Structure

### 2.1 All 33 Slot Properties

Each FlexNode contains **33 Slot properties** that bind to parallel arrays:

```rust
pub struct FlexNode {
    pub index: usize,

    // === CORE (4) ===
    pub id: Slot<Option<String>>,
    pub visible: Slot<bool>,
    pub component_type: Slot<ComponentType>,
    pub z_index: Slot<i32>,

    // === DIMENSIONS (6) ===
    pub width: Slot<Dimension>,
    pub height: Slot<Dimension>,
    pub min_width: Slot<Dimension>,
    pub min_height: Slot<Dimension>,
    pub max_width: Slot<Dimension>,
    pub max_height: Slot<Dimension>,

    // === SPACING - PADDING (4) ===
    pub padding_top: Slot<i32>,
    pub padding_right: Slot<i32>,
    pub padding_bottom: Slot<i32>,
    pub padding_left: Slot<i32>,

    // === SPACING - MARGIN (4) ===
    pub margin_top: Slot<MarginValue>,
    pub margin_right: Slot<MarginValue>,
    pub margin_bottom: Slot<MarginValue>,
    pub margin_left: Slot<MarginValue>,

    // === SPACING - GAP (3) ===
    pub gap: Slot<i32>,
    pub row_gap: Slot<i32>,
    pub column_gap: Slot<i32>,

    // === LAYOUT - FLEX CONTAINER (6) ===
    pub flex_direction: Slot<FlexDirection>,
    pub flex_wrap: Slot<FlexWrap>,
    pub justify_content: Slot<JustifyContent>,
    pub align_items: Slot<AlignItems>,
    pub align_content: Slot<AlignContent>,
    pub overflow: Slot<Overflow>,

    // === LAYOUT - FLEX ITEM (4) ===
    pub flex_grow: Slot<f32>,
    pub flex_shrink: Slot<f32>,
    pub flex_basis: Slot<Dimension>,
    pub align_self: Slot<AlignSelf>,

    // === VISUAL - BORDER (4) ===
    pub border_top: Slot<bool>,
    pub border_right: Slot<bool>,
    pub border_bottom: Slot<bool>,
    pub border_left: Slot<bool>,
}
```

### 2.2 Slot Type Definition

```rust
/// A Slot binds a FlexNode property to a parallel array index
pub struct Slot<T> {
    index: usize,
    array: &'static SlotArray<T>,
}

impl<T: Clone> Slot<T> {
    pub fn get(&self) -> T {
        self.array.get(self.index)
    }

    pub fn set(&self, value: T) {
        self.array.set(self.index, value);
    }

    /// Bind to a reactive source (Signal, Derived, or static value)
    pub fn bind<S: IntoSignalSource<T>>(&self, source: S) {
        self.array.set_source(self.index, source.into_source());
    }
}
```

---

## 3. Parallel Arrays Pattern (Father State)

### 3.1 Array Categories

The framework uses **81 total arrays** across **7 categories**:

#### Category 1: Core Arrays (4)
```rust
pub struct CoreArrays {
    pub id: SlotArray<Option<String>>,
    pub visible: SlotArray<bool>,
    pub component_type: SlotArray<ComponentType>,
    pub z_index: SlotArray<i32>,
}
```

#### Category 2: Dimension Arrays (6)
```rust
pub struct DimensionArrays {
    pub width: SlotArray<Dimension>,
    pub height: SlotArray<Dimension>,
    pub min_width: SlotArray<Dimension>,
    pub min_height: SlotArray<Dimension>,
    pub max_width: SlotArray<Dimension>,
    pub max_height: SlotArray<Dimension>,
}
```

#### Category 3: Spacing Arrays (11)
```rust
pub struct SpacingArrays {
    // Padding
    pub padding_top: SlotArray<i32>,
    pub padding_right: SlotArray<i32>,
    pub padding_bottom: SlotArray<i32>,
    pub padding_left: SlotArray<i32>,

    // Margin
    pub margin_top: SlotArray<MarginValue>,
    pub margin_right: SlotArray<MarginValue>,
    pub margin_bottom: SlotArray<MarginValue>,
    pub margin_left: SlotArray<MarginValue>,

    // Gap
    pub gap: SlotArray<i32>,
    pub row_gap: SlotArray<i32>,
    pub column_gap: SlotArray<i32>,
}
```

#### Category 4: Layout Arrays (24)
```rust
pub struct LayoutArrays {
    // Flex container
    pub flex_direction: SlotArray<FlexDirection>,
    pub flex_wrap: SlotArray<FlexWrap>,
    pub justify_content: SlotArray<JustifyContent>,
    pub align_items: SlotArray<AlignItems>,
    pub align_content: SlotArray<AlignContent>,
    pub overflow: SlotArray<Overflow>,

    // Flex item
    pub flex_grow: SlotArray<f32>,
    pub flex_shrink: SlotArray<f32>,
    pub flex_basis: SlotArray<Dimension>,
    pub align_self: SlotArray<AlignSelf>,
    pub order: SlotArray<i32>,

    // Position
    pub position: SlotArray<Position>,
    pub top: SlotArray<Dimension>,
    pub right: SlotArray<Dimension>,
    pub bottom: SlotArray<Dimension>,
    pub left: SlotArray<Dimension>,

    // Tree structure (computed)
    pub parent_index: SlotArray<i32>,
    pub first_child: SlotArray<i32>,
    pub next_sibling: SlotArray<i32>,
    pub prev_sibling: SlotArray<i32>,
    pub last_child: SlotArray<i32>,
    pub child_count: SlotArray<i32>,

    // Dirty tracking
    pub layout_dirty: SlotArray<bool>,
}
```

#### Category 5: Visual Arrays (15)
```rust
pub struct VisualArrays {
    // Border flags
    pub border_top: SlotArray<bool>,
    pub border_right: SlotArray<bool>,
    pub border_bottom: SlotArray<bool>,
    pub border_left: SlotArray<bool>,

    // Border style
    pub border_style: SlotArray<BorderStyle>,

    // Colors
    pub fg: SlotArray<Rgba>,
    pub bg: SlotArray<Rgba>,
    pub border_fg: SlotArray<Rgba>,
    pub border_bg: SlotArray<Rgba>,

    // Opacity
    pub opacity: SlotArray<f32>,

    // Theme variant
    pub variant: SlotArray<Variant>,

    // Text attributes
    pub bold: SlotArray<bool>,
    pub italic: SlotArray<bool>,
    pub underline: SlotArray<bool>,
    pub strikethrough: SlotArray<bool>,
}
```

#### Category 6: Text Arrays (5)
```rust
pub struct TextArrays {
    pub text_content: SlotArray<String>,
    pub text_wrap: SlotArray<TextWrap>,
    pub text_align: SlotArray<TextAlign>,
    pub text_overflow: SlotArray<TextOverflow>,
    pub text_attrs: SlotArray<u32>,  // Bitfield
}
```

#### Category 7: Interaction Arrays (16)
```rust
pub struct InteractionArrays {
    // Focus
    pub focusable: SlotArray<bool>,
    pub tab_index: SlotArray<i32>,

    // Scroll
    pub scroll_offset_x: SlotArray<i32>,
    pub scroll_offset_y: SlotArray<i32>,

    // Cursor
    pub cursor_position: SlotArray<usize>,
    pub cursor_visible: SlotArray<bool>,
    pub cursor_char: SlotArray<char>,
    pub cursor_blink_fps: SlotArray<u32>,

    // Selection
    pub selection_start: SlotArray<i32>,
    pub selection_end: SlotArray<i32>,

    // Event handlers (stored separately)
    pub on_click: SlotArray<Option<ClickHandler>>,
    pub on_key_down: SlotArray<Option<KeyHandler>>,
    pub on_mouse_enter: SlotArray<Option<MouseHandler>>,
    pub on_mouse_leave: SlotArray<Option<MouseHandler>>,
    pub on_focus: SlotArray<Option<FocusHandler>>,
    pub on_blur: SlotArray<Option<FocusHandler>>,
}
```

### 3.2 SlotArray Implementation

```rust
use std::cell::RefCell;
use spark_signals::{Signal, Derived, SignalGet, SignalSet};

pub enum SlotSource<T> {
    Static(T),
    Signal(Signal<T>),
    Derived(Derived<T>),
    Getter(Box<dyn Fn() -> T>),
}

pub struct SlotArray<T: Clone + Default> {
    values: RefCell<Vec<T>>,
    sources: RefCell<Vec<Option<SlotSource<T>>>>,
}

impl<T: Clone + Default + 'static> SlotArray<T> {
    pub fn new() -> Self {
        Self {
            values: RefCell::new(Vec::new()),
            sources: RefCell::new(Vec::new()),
        }
    }

    pub fn get(&self, index: usize) -> T {
        let sources = self.sources.borrow();
        if let Some(Some(source)) = sources.get(index) {
            match source {
                SlotSource::Static(v) => v.clone(),
                SlotSource::Signal(s) => s.get(),
                SlotSource::Derived(d) => d.get(),
                SlotSource::Getter(f) => f(),
            }
        } else {
            self.values.borrow().get(index).cloned().unwrap_or_default()
        }
    }

    pub fn set(&self, index: usize, value: T) {
        // Ensure capacity
        {
            let mut values = self.values.borrow_mut();
            if index >= values.len() {
                values.resize(index + 1, T::default());
            }
            values[index] = value.clone();
        }

        // Clear source if setting directly
        let mut sources = self.sources.borrow_mut();
        if index >= sources.len() {
            sources.resize_with(index + 1, || None);
        }
        sources[index] = Some(SlotSource::Static(value));
    }

    pub fn set_source(&self, index: usize, source: SlotSource<T>) {
        let mut sources = self.sources.borrow_mut();
        if index >= sources.len() {
            sources.resize_with(index + 1, || None);
        }
        sources[index] = Some(source);
    }

    pub fn clear(&self, index: usize) {
        if let Some(source) = self.sources.borrow_mut().get_mut(index) {
            *source = None;
        }
        if let Some(value) = self.values.borrow_mut().get_mut(index) {
            *value = T::default();
        }
    }
}
```

---

## 4. Registry System

### 4.1 Index Allocation

```rust
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static REGISTRY: RefCell<FlexNodeRegistry> = RefCell::new(FlexNodeRegistry::new());
}

pub struct FlexNodeRegistry {
    // Free pool for index reuse
    free_indices: Vec<usize>,

    // Next index if pool empty
    next_index: usize,

    // ID → Index mapping
    id_to_index: HashMap<String, usize>,

    // Index → ID mapping (for reverse lookup)
    index_to_id: HashMap<usize, String>,

    // Parent stack for tree building
    parent_stack: Vec<usize>,

    // Current capacity
    capacity: usize,
}

impl FlexNodeRegistry {
    pub fn new() -> Self {
        Self {
            free_indices: Vec::new(),
            next_index: 0,
            id_to_index: HashMap::new(),
            index_to_id: HashMap::new(),
            parent_stack: Vec::new(),
            capacity: 0,
        }
    }

    /// Allocate a new index (reuse from pool or increment)
    pub fn allocate_index(&mut self) -> usize {
        if let Some(index) = self.free_indices.pop() {
            index
        } else {
            let index = self.next_index;
            self.next_index += 1;

            // Grow arrays if needed
            if index >= self.capacity {
                self.grow_capacity(index + 64);
            }

            index
        }
    }

    /// Release an index back to the pool
    pub fn release_index(&mut self, index: usize) {
        // Clear all arrays at this index
        clear_all_arrays(index);

        // Remove ID mapping if exists
        if let Some(id) = self.index_to_id.remove(&index) {
            self.id_to_index.remove(&id);
        }

        // Return to pool
        self.free_indices.push(index);
    }

    /// Register an ID for an index
    pub fn register_id(&mut self, index: usize, id: String) {
        self.id_to_index.insert(id.clone(), index);
        self.index_to_id.insert(index, id);
    }

    /// Get index by ID
    pub fn get_by_id(&self, id: &str) -> Option<usize> {
        self.id_to_index.get(id).copied()
    }

    /// Push parent for tree building
    pub fn push_parent(&mut self, index: usize) {
        self.parent_stack.push(index);
    }

    /// Pop parent after children complete
    pub fn pop_parent(&mut self) {
        self.parent_stack.pop();
    }

    /// Get current parent (for child attachment)
    pub fn current_parent(&self) -> Option<usize> {
        self.parent_stack.last().copied()
    }

    fn grow_capacity(&mut self, new_capacity: usize) {
        // Each SlotArray will grow lazily, but we track capacity here
        self.capacity = new_capacity;
    }
}
```

### 4.2 Public API

```rust
/// Allocate a new component index
pub fn allocate_index() -> usize {
    REGISTRY.with(|r| r.borrow_mut().allocate_index())
}

/// Release a component index
pub fn release_index(index: usize) {
    REGISTRY.with(|r| r.borrow_mut().release_index(index))
}

/// Get component by ID
pub fn get_by_id(id: &str) -> Option<usize> {
    REGISTRY.with(|r| r.borrow().get_by_id(id))
}

/// Push parent context
pub fn push_parent(index: usize) {
    REGISTRY.with(|r| r.borrow_mut().push_parent(index))
}

/// Pop parent context
pub fn pop_parent() {
    REGISTRY.with(|r| r.borrow_mut().pop_parent())
}

/// Get current parent
pub fn current_parent() -> Option<usize> {
    REGISTRY.with(|r| r.borrow().current_parent())
}
```

---

## 5. Node Lifecycle

### 5.1 Lifecycle Phases

```
Creation → Setup → Mount → Active → Unmount → Cleanup
    ↓         ↓       ↓        ↓         ↓         ↓
allocate   bind    attach   reactive  detach    release
 index    props   to tree   updates  from tree   index
```

### 5.2 Creation Phase

```rust
pub fn create_flex_node() -> FlexNode {
    let index = allocate_index();

    FlexNode {
        index,
        // All slots bind to arrays at this index
        id: Slot::new(index, &CORE.id),
        visible: Slot::new(index, &CORE.visible),
        // ... all 33 slots
    }
}
```

### 5.3 Setup Phase

```rust
impl FlexNode {
    pub fn setup<P: Props>(&self, props: P) {
        // Bind each prop to its slot
        // Props can be: static value, Signal, Derived, or getter

        if let Some(width) = props.width {
            self.width.bind(width);
        }

        if let Some(on_click) = props.on_click {
            INTERACTION.on_click.set(self.index, Some(on_click));
        }

        // Set defaults for unspecified props
        self.visible.set(true);
        self.flex_shrink.set(1.0);
        // ...
    }
}
```

### 5.4 Mount Phase

```rust
impl FlexNode {
    pub fn mount(&self) {
        // Attach to parent
        if let Some(parent_index) = current_parent() {
            attach_child(parent_index, self.index);
        }

        // Push self as parent for children
        push_parent(self.index);
    }

    pub fn mount_complete(&self) {
        // Pop self from parent stack
        pop_parent();
    }
}

fn attach_child(parent: usize, child: usize) {
    LAYOUT.parent_index.set(child, parent as i32);

    let last_child = LAYOUT.last_child.get(parent);

    if last_child < 0 {
        // First child
        LAYOUT.first_child.set(parent, child as i32);
    } else {
        // Append to sibling chain
        LAYOUT.next_sibling.set(last_child as usize, child as i32);
        LAYOUT.prev_sibling.set(child, last_child);
    }

    LAYOUT.last_child.set(parent, child as i32);

    // Increment child count
    let count = LAYOUT.child_count.get(parent);
    LAYOUT.child_count.set(parent, count + 1);
}
```

### 5.5 Unmount Phase

```rust
impl FlexNode {
    pub fn unmount(&self) {
        // Detach from parent
        detach_from_parent(self.index);

        // Recursively unmount children
        let mut child = LAYOUT.first_child.get(self.index);
        while child >= 0 {
            let next = LAYOUT.next_sibling.get(child as usize);
            // Child unmount will release its index
            unmount_node(child as usize);
            child = next;
        }
    }
}

fn detach_from_parent(index: usize) {
    let parent = LAYOUT.parent_index.get(index);
    if parent < 0 {
        return;
    }

    let prev = LAYOUT.prev_sibling.get(index);
    let next = LAYOUT.next_sibling.get(index);

    // Update sibling chain
    if prev >= 0 {
        LAYOUT.next_sibling.set(prev as usize, next);
    } else {
        // Was first child
        LAYOUT.first_child.set(parent as usize, next);
    }

    if next >= 0 {
        LAYOUT.prev_sibling.set(next as usize, prev);
    } else {
        // Was last child
        LAYOUT.last_child.set(parent as usize, prev);
    }

    // Decrement count
    let count = LAYOUT.child_count.get(parent as usize);
    LAYOUT.child_count.set(parent as usize, count - 1);

    // Clear own pointers
    LAYOUT.parent_index.set(index, -1);
    LAYOUT.prev_sibling.set(index, -1);
    LAYOUT.next_sibling.set(index, -1);
}
```

### 5.6 Cleanup Phase

```rust
impl FlexNode {
    pub fn cleanup(&self) {
        // Release index back to pool
        release_index(self.index);
    }
}
```

---

## 6. Parent-Child Relationships

### 6.1 Tree Storage

The tree structure uses **linked list representation** in parallel arrays:

```
parent_index[i]  = parent of node i (-1 if root)
first_child[i]   = first child of node i (-1 if leaf)
last_child[i]    = last child of node i (-1 if leaf)
next_sibling[i]  = next sibling of node i (-1 if last)
prev_sibling[i]  = previous sibling of node i (-1 if first)
child_count[i]   = number of children
```

### 6.2 Tree Traversal

```rust
/// Iterate children of a node
pub fn children(index: usize) -> impl Iterator<Item = usize> {
    ChildIterator {
        current: LAYOUT.first_child.get(index),
    }
}

struct ChildIterator {
    current: i32,
}

impl Iterator for ChildIterator {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < 0 {
            None
        } else {
            let index = self.current as usize;
            self.current = LAYOUT.next_sibling.get(index);
            Some(index)
        }
    }
}

/// Iterate all descendants (depth-first)
pub fn descendants(index: usize) -> impl Iterator<Item = usize> {
    let mut stack = vec![index];
    std::iter::from_fn(move || {
        stack.pop().map(|i| {
            // Push children in reverse order for correct traversal
            let mut children: Vec<_> = children(i).collect();
            children.reverse();
            stack.extend(children);
            i
        })
    })
}
```

### 6.3 Recursive Operations

```rust
/// Recursively destroy a subtree
pub fn destroy_subtree(index: usize) {
    // Destroy children first (post-order)
    for child in children(index).collect::<Vec<_>>() {
        destroy_subtree(child);
    }

    // Detach and release this node
    detach_from_parent(index);
    release_index(index);
}
```

---

## 7. ID Generation and Tracking

### 7.1 ID Types

```rust
pub enum NodeId {
    /// No ID (anonymous component)
    None,

    /// User-provided ID
    Custom(String),

    /// Auto-generated ID (for debugging)
    Auto(usize),
}
```

### 7.2 ID Registration

```rust
impl FlexNode {
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        let id = id.into();
        REGISTRY.with(|r| {
            r.borrow_mut().register_id(self.index, id.clone());
        });
        self.id.set(Some(id));
        self
    }
}
```

### 7.3 ID Lookup

```rust
/// Get node by ID (returns None if not found)
pub fn get_node_by_id(id: &str) -> Option<usize> {
    REGISTRY.with(|r| r.borrow().get_by_id(id))
}

/// Get ID of node (returns None if anonymous)
pub fn get_node_id(index: usize) -> Option<String> {
    CORE.id.get(index)
}
```

---

## 8. Hidden Automatic Behaviors

### 8.1 Parent Context Propagation

When creating nested components, the parent is automatically tracked:

```rust
// This happens automatically:
Box::new()  // allocates index, pushes parent
    .child(Text::new("hello"))  // attaches to current parent
    .child(Text::new("world"))  // attaches to same parent
    .build()  // pops parent
```

### 8.2 Recursive Cleanup

When a component is destroyed, all descendants are automatically destroyed:

```rust
// Destroying parent automatically destroys children
destroy_subtree(parent_index);
// All children, grandchildren, etc. are released
```

### 8.3 Auto Capacity Growth

Arrays grow automatically when indices exceed capacity:

```rust
// No need to pre-allocate
let index = allocate_index(); // May be 1000
CORE.visible.set(index, true); // Array grows automatically
```

### 8.4 Default Value Initialization

Unset properties return sensible defaults:

```rust
// These return defaults even if never set:
LAYOUT.flex_shrink.get(index)  // 1.0
CORE.visible.get(index)        // true
LAYOUT.flex_direction.get(index) // Column
```

### 8.5 ID Collision Prevention

Duplicate IDs log warnings but don't crash:

```rust
// Second registration overwrites first
node1.with_id("my-id");
node2.with_id("my-id");  // Warning logged, node2 now owns "my-id"
```

### 8.6 Sibling Order Preservation

Children maintain insertion order for layout:

```rust
Box::new()
    .child(a)  // Rendered first
    .child(b)  // Rendered second
    .child(c)  // Rendered third
```

### 8.7 Focus Auto-Blur on Destroy

If a focused component is destroyed, focus is automatically cleared:

```rust
// In release_index:
if FOCUS.focused_index.get() == index as i32 {
    FOCUS.focused_index.set(-1);
}
```

### 8.8 Scroll Position Reset on Destroy

Scroll positions are cleared when components are destroyed.

### 8.9 Handler Cleanup

Event handlers are automatically cleared when components are released.

### 8.10 Index Reuse Safety

Released indices are reused, but ID mappings prevent stale references:

```rust
// Index 5 released
release_index(5);

// Index 5 reused for new component
let new_index = allocate_index(); // Returns 5

// Old ID no longer maps to 5
get_node_by_id("old-id"); // Returns None
```

---

## 9. Rust Module Structure

```
crates/tui/src/
├── engine/
│   ├── mod.rs
│   ├── flex_node.rs      # FlexNode struct and Slot type
│   ├── registry.rs       # FlexNodeRegistry and allocation
│   ├── arrays/
│   │   ├── mod.rs
│   │   ├── core.rs       # CoreArrays
│   │   ├── dimension.rs  # DimensionArrays
│   │   ├── spacing.rs    # SpacingArrays
│   │   ├── layout.rs     # LayoutArrays
│   │   ├── visual.rs     # VisualArrays
│   │   ├── text.rs       # TextArrays
│   │   └── interaction.rs # InteractionArrays
│   └── slot_array.rs     # SlotArray<T> implementation
```

---

## 10. Integration with Signals

### 10.1 Slot Binding to Signals

```rust
use spark_signals::{signal, derived, Signal, Derived};

// Create a signal
let width = signal(100);

// Bind to FlexNode slot
node.width.bind(width.clone());

// Later, update signal → layout recalculates
width.set(200);
```

### 10.2 Derived Slots

```rust
let base_width = signal(100);
let padding = signal(10);

// Derived dimension
let total_width = derived(move || base_width.get() + padding.get() * 2);

node.width.bind(total_width);
```

### 10.3 Layout Derived

The layout engine reads all slots, creating dependencies:

```rust
let layout = derived(move || {
    // Reading flex_node.width.get() creates dependency
    // on the underlying signal/source
    compute_flexbox_layout(root_index)
});

// Layout auto-recalculates when any bound signal changes
```

### 10.4 Dirty Tracking with ReactiveSet

```rust
use spark_signals::ReactiveSet;

thread_local! {
    static LAYOUT_DIRTY: ReactiveSet<usize> = ReactiveSet::new();
}

// Mark node dirty
LAYOUT_DIRTY.with(|s| s.insert(index));

// In layout derived, check dirty set
let dirty_nodes = LAYOUT_DIRTY.with(|s| s.drain());
```

### 10.5 Batched Updates

```rust
use spark_signals::batch;

// Multiple updates, single recalculation
batch(|| {
    node1.width.set(100);
    node2.height.set(50);
    node3.visible.set(false);
});
// Layout recalculates once here
```

### 10.6 Untracked Reads

```rust
use spark_signals::untracked;

// Read without creating dependency
let width = untracked(|| node.width.get());
```

---

## 11. Implementation Checklist

### Core Types
- [ ] `Slot<T>` with array binding
- [ ] `SlotSource<T>` enum (Static, Signal, Derived, Getter)
- [ ] `SlotArray<T>` with reactive sources
- [ ] `FlexNode` with all 33 slots
- [ ] `FlexNodeRegistry` with allocation/release

### Array Categories
- [ ] CoreArrays (4 arrays)
- [ ] DimensionArrays (6 arrays)
- [ ] SpacingArrays (11 arrays)
- [ ] LayoutArrays (24 arrays)
- [ ] VisualArrays (15 arrays)
- [ ] TextArrays (5 arrays)
- [ ] InteractionArrays (16 arrays)

### Tree Operations
- [ ] `attach_child(parent, child)`
- [ ] `detach_from_parent(index)`
- [ ] `children(index)` iterator
- [ ] `descendants(index)` iterator
- [ ] `destroy_subtree(index)`

### Registry Operations
- [ ] `allocate_index()`
- [ ] `release_index(index)`
- [ ] `register_id(index, id)`
- [ ] `get_by_id(id)`
- [ ] Parent stack (push/pop/current)

### Signal Integration
- [ ] Slot binding to Signal<T>
- [ ] Slot binding to Derived<T>
- [ ] Slot binding to getter fn
- [ ] Dirty tracking with ReactiveSet
- [ ] Batch support
- [ ] Untracked reads

---

## 12. Summary

The core architecture uses:

1. **FlexNode** - 33 Slot properties binding to parallel arrays
2. **SlotArray** - Reactive arrays supporting Signal/Derived/static sources
3. **Registry** - Index allocation, reuse, ID mapping, parent tracking
4. **Tree Structure** - Linked list in arrays (parent, child, sibling pointers)
5. **Lifecycle** - Create → Setup → Mount → Active → Unmount → Cleanup
6. **Signals Integration** - Automatic dependency tracking and recalculation

This architecture enables:
- O(1) property access
- Cache-friendly data layout
- Automatic reactivity
- Efficient memory reuse
- Clean component lifecycle

**Total: 81 arrays, 33 FlexNode slots, 10 hidden automatic behaviors**
