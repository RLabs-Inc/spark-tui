# Focus & Navigation Specification

## Overview

This specification documents focus management, tab navigation, focus groups, auto-focus, programmatic focus control, and focus-related callbacks.

**Source Files Analyzed:**
- `/Users/rusty/Documents/Projects/TUI/tui/src/state/focus.ts`
- `/Users/rusty/Documents/Projects/TUI/tui/src/state/context.ts`

---

## 1. Core Concepts

### 1.1 Focus State

```rust
thread_local! {
    /// Currently focused component index (-1 = nothing focused)
    static FOCUSED_INDEX: Signal<i32> = signal(-1);

    /// Focus callback registry
    static FOCUS_CALLBACKS: RefCell<HashMap<usize, FocusCallbacks>> = RefCell::new(HashMap::new());

    /// Focus trap stack (for modals)
    static FOCUS_TRAP_STACK: RefCell<Vec<usize>> = RefCell::new(Vec::new());

    /// Focus history (for restoration)
    static FOCUS_HISTORY: RefCell<VecDeque<FocusHistoryEntry>> = RefCell::new(VecDeque::new());
}

const MAX_FOCUS_HISTORY: usize = 10;
```

### 1.2 Types

```rust
pub struct FocusCallbacks {
    pub on_focus: Vec<Box<dyn Fn()>>,
    pub on_blur: Vec<Box<dyn Fn()>>,
}

struct FocusHistoryEntry {
    index: usize,
    id: Option<String>,
}
```

---

## 2. Focus State Management

### 2.1 Get/Set Focus

```rust
/// Get currently focused index (-1 if none)
pub fn get_focused_index() -> i32 {
    FOCUSED_INDEX.with(|f| f.get())
}

/// Check if specific component is focused
pub fn is_focused(index: usize) -> bool {
    get_focused_index() == index as i32
}

/// Get focused index as signal (for reactive subscriptions)
pub fn focused_index_signal() -> Signal<i32> {
    FOCUSED_INDEX.with(|f| f.clone())
}

/// Set focus to specific component
pub fn focus(index: usize) {
    let current = get_focused_index();

    // No-op if already focused
    if current == index as i32 {
        return;
    }

    // Check if component is focusable
    if !INTERACTION.focusable.get(index) {
        return;
    }

    // Check if component is visible
    if !CORE.visible.get(index) {
        return;
    }

    // Check focus trap
    if !is_within_focus_trap(index) {
        return;
    }

    // Fire blur on previous
    if current >= 0 {
        fire_blur_callbacks(current as usize);
        save_to_focus_history(current as usize);
    }

    // Update focus
    FOCUSED_INDEX.with(|f| f.set(index as i32));

    // Fire focus on new
    fire_focus_callbacks(index);

    // Scroll into view
    scroll_into_view(index);
}

/// Clear focus
pub fn blur() {
    let current = get_focused_index();
    if current >= 0 {
        fire_blur_callbacks(current as usize);
        save_to_focus_history(current as usize);
    }
    FOCUSED_INDEX.with(|f| f.set(-1));
}
```

### 2.2 Callbacks

```rust
fn fire_focus_callbacks(index: usize) {
    FOCUS_CALLBACKS.with(|callbacks| {
        if let Some(cbs) = callbacks.borrow().get(&index) {
            for cb in &cbs.on_focus {
                cb();
            }
        }
    });
}

fn fire_blur_callbacks(index: usize) {
    FOCUS_CALLBACKS.with(|callbacks| {
        if let Some(cbs) = callbacks.borrow().get(&index) {
            for cb in &cbs.on_blur {
                cb();
            }
        }
    });
}

/// Register focus callback
pub fn on_focus<F: Fn() + 'static>(index: usize, handler: F) {
    FOCUS_CALLBACKS.with(|callbacks| {
        callbacks
            .borrow_mut()
            .entry(index)
            .or_insert_with(|| FocusCallbacks {
                on_focus: Vec::new(),
                on_blur: Vec::new(),
            })
            .on_focus
            .push(Box::new(handler));
    });
}

/// Register blur callback
pub fn on_blur<F: Fn() + 'static>(index: usize, handler: F) {
    FOCUS_CALLBACKS.with(|callbacks| {
        callbacks
            .borrow_mut()
            .entry(index)
            .or_insert_with(|| FocusCallbacks {
                on_focus: Vec::new(),
                on_blur: Vec::new(),
            })
            .on_blur
            .push(Box::new(handler));
    });
}

/// Cleanup callbacks for released component
pub fn cleanup_focus_callbacks(index: usize) {
    FOCUS_CALLBACKS.with(|callbacks| {
        callbacks.borrow_mut().remove(&index);
    });
}
```

---

## 3. Focusable Elements

### 3.1 Focusable State

```rust
/// Set whether component is focusable
pub fn set_focusable(index: usize, focusable: bool) {
    INTERACTION.focusable.set(index, focusable);

    // Auto-blur if making non-focusable while focused
    if !focusable && is_focused(index) {
        blur();
    }
}

/// Get whether component is focusable
pub fn is_focusable(index: usize) -> bool {
    INTERACTION.focusable.get(index)
}
```

### 3.2 Default Focusability

| Component | Default | Notes |
|-----------|---------|-------|
| Box | `false` | `true` if `overflow: scroll` |
| Input | `true` | Always focusable |
| Text | `false` | Never focusable |

```rust
// In Box implementation
let should_be_focusable = props.focusable.map(|f| f.get()).unwrap_or(false)
    || (props.overflow == Some(Overflow::Scroll) && props.focusable != Some(false));

if should_be_focusable {
    INTERACTION.focusable.set(index, true);
}
```

---

## 4. Tab Navigation

### 4.1 Tab Order

```rust
/// Get all focusable components in tab order
fn get_focusable_in_order() -> Vec<usize> {
    let mut focusables = Vec::new();

    // Collect all visible focusable components
    for index in 0..get_component_count() {
        if INTERACTION.focusable.get(index) && CORE.visible.get(index) {
            // Check focus trap
            if is_within_focus_trap(index) {
                focusables.push(index);
            }
        }
    }

    // Sort by tab_index, then by allocation order
    focusables.sort_by(|&a, &b| {
        let tab_a = INTERACTION.tab_index.get(a);
        let tab_b = INTERACTION.tab_index.get(b);

        // Negative tab index = not in tab order
        if tab_a < 0 && tab_b < 0 {
            return a.cmp(&b);
        }
        if tab_a < 0 {
            return std::cmp::Ordering::Greater;
        }
        if tab_b < 0 {
            return std::cmp::Ordering::Less;
        }

        // Sort by tab_index, then by index
        match tab_a.cmp(&tab_b) {
            std::cmp::Ordering::Equal => a.cmp(&b),
            other => other,
        }
    });

    // Filter out negative tab indices
    focusables.retain(|&idx| INTERACTION.tab_index.get(idx) >= 0);

    focusables
}

/// Focus next component in tab order
pub fn focus_next() {
    let focusables = get_focusable_in_order();
    if focusables.is_empty() {
        return;
    }

    let current = get_focused_index();

    if current < 0 {
        // Nothing focused - focus first
        focus(focusables[0]);
        return;
    }

    // Find current position
    let current_pos = focusables
        .iter()
        .position(|&idx| idx == current as usize);

    match current_pos {
        Some(pos) => {
            // Focus next (wrap around)
            let next_pos = (pos + 1) % focusables.len();
            focus(focusables[next_pos]);
        }
        None => {
            // Current not in list - focus first
            focus(focusables[0]);
        }
    }
}

/// Focus previous component in tab order
pub fn focus_previous() {
    let focusables = get_focusable_in_order();
    if focusables.is_empty() {
        return;
    }

    let current = get_focused_index();

    if current < 0 {
        // Nothing focused - focus last
        focus(*focusables.last().unwrap());
        return;
    }

    // Find current position
    let current_pos = focusables
        .iter()
        .position(|&idx| idx == current as usize);

    match current_pos {
        Some(pos) => {
            // Focus previous (wrap around)
            let prev_pos = if pos == 0 {
                focusables.len() - 1
            } else {
                pos - 1
            };
            focus(focusables[prev_pos]);
        }
        None => {
            // Current not in list - focus last
            focus(*focusables.last().unwrap());
        }
    }
}
```

### 4.2 Tab Index

```rust
/// Set tab index for component
pub fn set_tab_index(index: usize, tab_index: i32) {
    INTERACTION.tab_index.set(index, tab_index);
}

/// Tab index conventions:
/// -1 = Not in tab order (focusable via click only)
///  0 = Default tab order (by allocation order)
///  1+ = Explicit tab order (lower = earlier)
```

---

## 5. Focus Groups & Trapping

### 5.1 Focus Trap

```rust
/// Push a focus trap (for modals)
pub fn push_focus_trap(container_index: usize) {
    FOCUS_TRAP_STACK.with(|stack| {
        stack.borrow_mut().push(container_index);
    });

    // Focus first focusable within trap
    focus_first_in_container(container_index);
}

/// Pop focus trap
pub fn pop_focus_trap() {
    FOCUS_TRAP_STACK.with(|stack| {
        stack.borrow_mut().pop();
    });

    // Restore focus from history
    restore_focus_from_history();
}

/// Check if index is within current focus trap
fn is_within_focus_trap(index: usize) -> bool {
    FOCUS_TRAP_STACK.with(|stack| {
        let stack = stack.borrow();
        if stack.is_empty() {
            return true; // No trap = all allowed
        }

        let trap_container = *stack.last().unwrap();
        is_descendant_of(index, trap_container)
    })
}

/// Check if child is descendant of ancestor
fn is_descendant_of(child: usize, ancestor: usize) -> bool {
    if child == ancestor {
        return true;
    }

    let mut current = LAYOUT.parent_index.get(child);
    while current >= 0 {
        if current as usize == ancestor {
            return true;
        }
        current = LAYOUT.parent_index.get(current as usize);
    }

    false
}
```

### 5.2 Focus Within Container

```rust
/// Focus first focusable descendant
pub fn focus_first_in_container(container: usize) {
    for descendant in descendants(container) {
        if INTERACTION.focusable.get(descendant) && CORE.visible.get(descendant) {
            focus(descendant);
            return;
        }
    }
}

/// Focus last focusable descendant
pub fn focus_last_in_container(container: usize) {
    let mut last_focusable = None;

    for descendant in descendants(container) {
        if INTERACTION.focusable.get(descendant) && CORE.visible.get(descendant) {
            last_focusable = Some(descendant);
        }
    }

    if let Some(idx) = last_focusable {
        focus(idx);
    }
}
```

---

## 6. Auto-Focus

### 6.1 Deferred Focus

```rust
thread_local! {
    static DEFERRED_FOCUS: RefCell<Vec<usize>> = RefCell::new(Vec::new());
}

/// Queue component for auto-focus after mount
pub fn defer_focus(index: usize) {
    DEFERRED_FOCUS.with(|queue| {
        queue.borrow_mut().push(index);
    });
}

/// Process deferred focus (called after mount completes)
pub fn process_deferred_focus() {
    DEFERRED_FOCUS.with(|queue| {
        let mut queue = queue.borrow_mut();

        // Focus last queued (if multiple auto-focus)
        if let Some(index) = queue.pop() {
            if INTERACTION.focusable.get(index) && CORE.visible.get(index) {
                focus(index);
            }
        }

        queue.clear();
    });
}
```

### 6.2 Usage in Primitives

```rust
// In Box/Input implementation
if props.auto_focus == Some(true) {
    defer_focus(index);
}
```

---

## 7. Click-to-Focus

### 7.1 Automatic Behavior

```rust
// In mouse dispatch
MouseAction::Down => {
    if let Some(index) = hit {
        // Focus on click (if focusable)
        if INTERACTION.focusable.get(index) {
            focus(index);
        }

        // ... rest of handler
    }
}
```

---

## 8. Focus Restoration

### 8.1 History Management

```rust
fn save_to_focus_history(index: usize) {
    let id = CORE.id.get(index);

    FOCUS_HISTORY.with(|history| {
        let mut history = history.borrow_mut();

        // Add to front
        history.push_front(FocusHistoryEntry { index, id });

        // Limit size
        while history.len() > MAX_FOCUS_HISTORY {
            history.pop_back();
        }
    });
}

/// Restore focus from history
pub fn restore_focus_from_history() {
    FOCUS_HISTORY.with(|history| {
        let mut history = history.borrow_mut();

        while let Some(entry) = history.pop_front() {
            // Validate entry is still valid
            if is_valid_focus_target(&entry) {
                focus(entry.index);
                return;
            }
        }
    });
}

fn is_valid_focus_target(entry: &FocusHistoryEntry) -> bool {
    // Check index is allocated
    if entry.index >= get_component_count() {
        return false;
    }

    // Check ID matches (detect recycled indices)
    let current_id = CORE.id.get(entry.index);
    if entry.id != current_id {
        return false;
    }

    // Check still focusable and visible
    INTERACTION.focusable.get(entry.index) && CORE.visible.get(entry.index)
}
```

---

## 9. Programmatic Focus Control

### 9.1 Public API

```rust
/// Focus specific component
pub fn focus(index: usize);

/// Clear focus
pub fn blur();

/// Focus first focusable
pub fn focus_first() {
    let focusables = get_focusable_in_order();
    if let Some(&first) = focusables.first() {
        focus(first);
    }
}

/// Focus last focusable
pub fn focus_last() {
    let focusables = get_focusable_in_order();
    if let Some(&last) = focusables.last() {
        focus(last);
    }
}

/// Focus next in tab order
pub fn focus_next();

/// Focus previous in tab order
pub fn focus_previous();

/// Restore from history
pub fn restore_focus_from_history();
```

---

## 10. Focus Visibility

### 10.1 Visual Feedback

```rust
/// Derive border color based on focus
let border_color = derived(move || {
    if is_focused(index) {
        theme::primary()
    } else {
        theme::border()
    }
});

// Bind to visual arrays
VISUAL.border_fg.bind_derived(index, border_color);
```

### 10.2 Scroll Into View

```rust
/// Ensure focused component is visible
fn scroll_into_view(index: usize) {
    // Find nearest scrollable ancestor
    let mut current = LAYOUT.parent_index.get(index);
    while current >= 0 {
        let is_scrollable = is_scrollable(current as usize);
        if is_scrollable {
            scroll_component_into_view(current as usize, index);
            return;
        }
        current = LAYOUT.parent_index.get(current as usize);
    }
}
```

---

## 11. Disabled Elements

### 11.1 Handling

```rust
// Disabled = not focusable
// No built-in "disabled" prop - implement via focusable

pub fn set_disabled(index: usize, disabled: bool) {
    set_focusable(index, !disabled);

    // Also update visual state
    VISUAL.opacity.set(index, if disabled { 0.5 } else { 1.0 });
}
```

---

## 12. Auto-Blur on Destroy/Hide

### 12.1 On Release

```rust
pub fn release_index(index: usize) {
    // Auto-blur if focused
    if is_focused(index) {
        blur();
        restore_focus_from_history();
    }

    // Cleanup callbacks
    cleanup_focus_callbacks(index);

    // ... rest of release
}
```

### 12.2 On Hide

```rust
// Effect that watches visibility
effect(move || {
    let focused = get_focused_index();
    if focused >= 0 {
        let visible = CORE.visible.get(focused as usize);
        if !visible {
            blur();
        }
    }
});
```

---

## 13. Hidden Automatic Behaviors

### 13.1 Auto-Blur on Hide
If focused component becomes `visible = false`, focus is automatically cleared.

### 13.2 Auto-Blur on Unmount
When releasing a focused component, focus is cleared and history restored.

### 13.3 Scrollable Auto-Focusable
`overflow: scroll` implies keyboard interaction needed → auto-focusable.

### 13.4 Tab Wrapping
Tab on last → first, Shift+Tab on first → last.

### 13.5 Focus Trap Filtering
When trap active, only descendants of trap container are focusable.

### 13.6 Deferred Auto-Focus
Auto-focus processed after mount completes (not during render).

### 13.7 History Validation
Focus history validates ID to detect recycled indices.

### 13.8 Scroll Into View
Focused component auto-scrolls into view.

### 13.9 Click-to-Focus
Focusable elements auto-focus on click.

### 13.10 Last Auto-Focus Wins
If multiple components have `auto_focus`, last one wins.

---

## 14. Module Structure

```
crates/tui/src/state/
├── focus.rs        # Main focus management
├── focus_trap.rs   # Focus trap stack
├── focus_history.rs # Focus restoration
└── tab_order.rs    # Tab navigation
```

---

## 15. Integration with Keyboard

```rust
// In global key handler
if event.key == "Tab" {
    if event.modifiers.shift {
        focus_previous();
    } else {
        focus_next();
    }
    return true;
}
```

---

## 16. Integration with Scroll

```rust
// In focus() after updating focus
fn focus(index: usize) {
    // ... set focus ...

    // Scroll into view
    scroll_into_view(index);
}
```

---

## 17. Testing Checklist

- [ ] Focus/blur fires callbacks
- [ ] Tab cycles through focusables
- [ ] Shift+Tab cycles backwards
- [ ] Tab wraps at boundaries
- [ ] Focus trap constrains navigation
- [ ] Auto-focus on mount
- [ ] Click-to-focus
- [ ] Auto-blur on hide
- [ ] Auto-blur on unmount
- [ ] Focus restoration from history
- [ ] Scroll into view on focus
- [ ] Disabled (non-focusable) elements skipped
- [ ] Negative tab index excluded from tab order

---

## 18. Summary

The focus system provides:

✅ **Single Focus**: Only one component focused at a time
✅ **Tab Navigation**: Tab/Shift+Tab with wrapping
✅ **Focus Traps**: Modal/dialog focus containment
✅ **Auto-Focus**: Deferred until mount complete
✅ **Click-to-Focus**: Automatic for focusable elements
✅ **Focus Restoration**: History-based restore
✅ **Scroll Into View**: Auto-scroll focused element visible
✅ **Reactive State**: Focus index as signal
✅ **Callbacks**: on_focus/on_blur handlers
✅ **Auto-Cleanup**: Blur on hide/unmount
