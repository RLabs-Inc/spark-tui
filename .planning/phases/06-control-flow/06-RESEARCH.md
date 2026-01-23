# Phase 6: Control Flow - Research

**Researched:** 2026-01-23
**Domain:** Reactive control flow primitives (conditional, list, async rendering)
**Confidence:** HIGH

## Summary

Control flow primitives (show, each, when) enable dynamic UI construction through reactive patterns. The TypeScript reference implementation uses EffectScope from spark-signals for cleanup management, parent context stacking for component hierarchy, and per-item signals for fine-grained list reactivity.

The Rust port has all the necessary building blocks already in place: EffectScope with nested scopes and on_scope_dispose, parent context stack in the registry, ReactiveVec with per-index tracking, and cleanup patterns established in box_primitive/text/input. The main work is composing these primitives into the three control flow functions.

**Primary recommendation:** Implement show(), each(), when() as standalone functions in a new `src/primitives/control_flow.rs` module, following the exact patterns from TypeScript but adapting for Rust's ownership model (closures capture by move, cleanup returns Box<dyn FnOnce()>).

## Standard Stack

The control flow implementation builds on existing infrastructure:

### Core (Already Available)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| spark-signals | 0.1.0 | EffectScope, effect, Signal, ReactiveVec | Project's reactive foundation |
| std::collections::HashMap | - | Key-to-cleanup mapping for each() | Standard library, no external deps |

### No Additional Dependencies Required

Control flow is pure composition of existing primitives. No new crates needed.

**Installation:** N/A - uses existing dependencies

## Architecture Patterns

### Recommended Project Structure
```
src/primitives/
    mod.rs              # Add: pub use control_flow::*;
    control_flow.rs     # NEW: show(), each(), when()
    box_primitive.rs    # Existing
    text.rs             # Existing
    input.rs            # Existing
    types.rs            # Existing
```

### Pattern 1: EffectScope-based Cleanup

**What:** Wrap all component creation in an EffectScope that tracks cleanups. When the scope stops, all components are automatically cleaned up.

**When to use:** Every control flow function

**Example:**
```rust
// Source: TypeScript show.ts, adapted for Rust
pub fn show<C, E>(
    condition: impl Fn() -> bool + 'static,
    then_fn: impl Fn() -> C + 'static,
    else_fn: Option<impl Fn() -> E + 'static>,
) -> Cleanup
where
    C: Into<Cleanup>,
    E: Into<Cleanup>,
{
    let scope = effect_scope();
    let parent_index = get_current_parent_index();

    // Mutable state for tracking
    let cleanup: Rc<RefCell<Option<Cleanup>>> = Rc::new(RefCell::new(None));
    let was_true: Rc<Cell<Option<bool>>> = Rc::new(Cell::new(None));

    scope.run(|| {
        let cleanup = cleanup.clone();
        let was_true = was_true.clone();

        // Effect that re-runs when condition changes
        effect(move || {
            let cond = condition();

            // Skip if unchanged
            if was_true.get() == Some(cond) {
                return;
            }
            was_true.set(Some(cond));

            // Cleanup previous render
            if let Some(c) = cleanup.borrow_mut().take() {
                c();
            }

            // Render appropriate branch with parent context
            push_parent_context(parent_index);
            let new_cleanup = if cond {
                Some(then_fn().into())
            } else {
                else_fn.as_ref().map(|f| f().into())
            };
            pop_parent_context();

            *cleanup.borrow_mut() = new_cleanup;
        });

        on_scope_dispose(move || {
            if let Some(c) = cleanup.borrow_mut().take() {
                c();
            }
        });
    });

    Box::new(move || scope.stop())
}
```

### Pattern 2: Key-based Reconciliation for each()

**What:** Track items by key, create per-item signals for fine-grained updates. New items get new components, removed items get cleaned up, updated items just update their signal.

**When to use:** each() with keyed collections

**Example:**
```rust
// Source: TypeScript each.ts, adapted for Rust
pub fn each<T, K, R>(
    items_getter: impl Fn() -> Vec<T> + 'static,
    render_fn: impl Fn(&dyn Fn() -> T, &K) -> R + Clone + 'static,
    key_fn: impl Fn(&T) -> K + 'static,
) -> Cleanup
where
    T: Clone + PartialEq + 'static,
    K: Clone + Eq + std::hash::Hash + 'static,
    R: Into<Cleanup>,
{
    let scope = effect_scope();
    let parent_index = get_current_parent_index();

    let cleanups: Rc<RefCell<HashMap<K, Cleanup>>> = Rc::new(RefCell::new(HashMap::new()));
    let item_signals: Rc<RefCell<HashMap<K, Signal<T>>>> = Rc::new(RefCell::new(HashMap::new()));

    scope.run(|| {
        let cleanups = cleanups.clone();
        let item_signals = item_signals.clone();
        let render_fn = render_fn.clone();

        effect(move || {
            let items = items_getter();
            let mut current_keys = std::collections::HashSet::new();

            push_parent_context(parent_index);

            for item in items.iter() {
                let key = key_fn(item);
                current_keys.insert(key.clone());

                if !item_signals.borrow().contains_key(&key) {
                    // NEW item - create signal and component
                    let item_signal = signal(item.clone());
                    let item_sig_clone = item_signal.clone();
                    let getter = move || item_sig_clone.get();
                    let cleanup = render_fn(&getter, &key).into();

                    item_signals.borrow_mut().insert(key.clone(), item_signal);
                    cleanups.borrow_mut().insert(key, cleanup);
                } else {
                    // EXISTING item - just update signal
                    if let Some(sig) = item_signals.borrow().get(&key) {
                        sig.set(item.clone());
                    }
                }
            }

            pop_parent_context();

            // Cleanup removed items
            let keys_to_remove: Vec<K> = cleanups.borrow()
                .keys()
                .filter(|k| !current_keys.contains(*k))
                .cloned()
                .collect();

            for key in keys_to_remove {
                if let Some(cleanup) = cleanups.borrow_mut().remove(&key) {
                    cleanup();
                }
                item_signals.borrow_mut().remove(&key);
            }
        });

        on_scope_dispose(|| {
            for cleanup in cleanups.borrow_mut().drain() {
                cleanup.1();
            }
        });
    });

    Box::new(move || scope.stop())
}
```

### Pattern 3: State Machine for when()

**What:** Track pending/resolved/rejected states explicitly. Current promise identity prevents stale callbacks. Cleanup runs cleanup functions from previous state before rendering new state.

**When to use:** Async handling with when()

**Example:**
```rust
// Conceptual pattern - actual async needs consideration
enum AsyncState<T, E> {
    Pending,
    Resolved(T),
    Rejected(E),
}

pub fn when<T, E, P, S, F>(
    promise_getter: impl Fn() -> oneshot::Receiver<Result<T, E>> + 'static,
    options: WhenOptions<T, E, P, S, F>,
) -> Cleanup
where
    T: Clone + 'static,
    E: Clone + 'static,
    P: Into<Cleanup>,
    S: Into<Cleanup>,
    F: Into<Cleanup>,
{
    // Track current promise identity to prevent stale callbacks
    let current_promise_id = Rc::new(Cell::new(0u64));
    let scope = effect_scope();
    let parent_index = get_current_parent_index();

    // ... state machine implementation

    Box::new(move || scope.stop())
}
```

### Anti-Patterns to Avoid

- **Calling cleanup inside tracked context:** Never call cleanup functions while an effect is executing - can corrupt tracking state. Use Rc<RefCell> to defer cleanup.

- **Forgetting parent context:** Always push/pop parent context when creating components dynamically. Components created without parent context become orphans.

- **Re-creating components on every update:** Use keys to identify stable items. Only create new components for new items.

- **Holding references across cleanup:** Cleanup closures should not capture references to things that will be dropped.

## Don't Hand-Roll

Problems with existing solutions in the codebase:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Cleanup tracking | Manual Vec<Cleanup> management | EffectScope + on_scope_dispose | Handles nested scopes, automatic cleanup on stop |
| Parent context | Manual stack tracking | push/pop_parent_context | Already thread-local, works with all primitives |
| Per-item reactivity | Re-render entire list | Signal per item + key | TypeScript pattern, fine-grained updates |
| Effect scheduling | Manual dirty tracking | effect() from spark-signals | Handles batching, scheduling automatically |

**Key insight:** The TypeScript reference shows exactly how to compose these primitives. Don't invent new patterns - translate the proven TypeScript patterns to Rust.

## Common Pitfalls

### Pitfall 1: Borrow Conflicts in Closures

**What goes wrong:** Rust's borrow checker prevents capturing mutable references in multiple closures (effect + cleanup).

**Why it happens:** Effect needs to read/write state, cleanup needs to read/clear state.

**How to avoid:** Use Rc<RefCell<T>> for shared mutable state:
```rust
let cleanup: Rc<RefCell<Option<Cleanup>>> = Rc::new(RefCell::new(None));
let cleanup_for_effect = cleanup.clone();
let cleanup_for_dispose = cleanup.clone();

effect(move || {
    // Use cleanup_for_effect
});

on_scope_dispose(move || {
    // Use cleanup_for_dispose
});
```

**Warning signs:** Compiler errors about moving out of captured variables.

### Pitfall 2: Parent Context Lost in Nested Control Flow

**What goes wrong:** Components created inside show/each don't have correct parent when control flow is nested.

**Why it happens:** Parent context captured at control flow creation, but effect runs later when context may have changed.

**How to avoid:** Capture parent index at creation time, push/pop around every render:
```rust
let parent_index = get_current_parent_index(); // Capture NOW

effect(move || {
    push_parent_context(parent_index.unwrap_or(usize::MAX)); // Restore when running
    // Create components
    pop_parent_context();
});
```

**Warning signs:** Components rendered at wrong level in hierarchy, orphaned components.

### Pitfall 3: Stale Closures in each()

**What goes wrong:** Item getter captures old signal reference, doesn't track updates.

**Why it happens:** Rust closures capture by value. If you clone the wrong thing, updates don't propagate.

**How to avoid:** Clone the Signal itself (which is cheap, just Rc), not the value:
```rust
let item_signal = signal(item.clone());
let item_sig_clone = item_signal.clone(); // Clone the Signal
let getter = move || item_sig_clone.get(); // Getter calls get() on the signal
```

**Warning signs:** List items don't update when data changes.

### Pitfall 4: Cleanup Order in Nested Structures

**What goes wrong:** Parent cleanup runs before child cleanup, causing panic when child tries to access released parent.

**Why it happens:** EffectScope stops in wrong order, or cleanup called manually in wrong order.

**How to avoid:** Let release_index handle recursive child cleanup:
```rust
// release_index already handles this!
// It finds all children and releases them FIRST
pub fn release_index(index: usize) {
    // FIRST: Find and release all children (recursive!)
    let children = find_children(index);
    for child in children {
        release_index(child); // Children cleaned up first
    }
    // THEN: Clean up this index
    // ...
}
```

**Warning signs:** Panics during cleanup, "already released" errors.

### Pitfall 5: Async Cancellation Without Cleanup

**What goes wrong:** Promise resolves after component unmounted, tries to render into destroyed parent.

**Why it happens:** Promise callback doesn't check if still mounted.

**How to avoid:** Track "current promise" identity, ignore callbacks from outdated promises:
```rust
let promise_id = Rc::new(Cell::new(0u64));
let current_id = promise_id.clone();

// When starting new promise:
let id = generate_unique_id();
promise_id.set(id);

// In callback:
if current_id.get() != id {
    return; // Stale callback, ignore
}
```

**Warning signs:** Rendering errors after navigation, "component already destroyed" errors.

## Code Examples

### show() - Conditional Rendering

```rust
// Source: TypeScript show.ts pattern
use spark_signals::{effect, effect_scope, on_scope_dispose, Signal};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::engine::{get_current_parent_index, push_parent_context, pop_parent_context};
use crate::primitives::Cleanup;

/// Conditional rendering based on a reactive condition.
///
/// # Arguments
/// * `condition` - Getter returning bool (auto-tracks dependencies)
/// * `then_fn` - Render function when condition is true
/// * `else_fn` - Optional render function when condition is false
///
/// # Example
/// ```ignore
/// let visible = signal(true);
///
/// let cleanup = show(
///     move || visible.get(),
///     || text(TextProps { content: "Visible!".into(), ..default() }),
///     Some(|| text(TextProps { content: "Hidden".into(), ..default() })),
/// );
/// ```
pub fn show<ThenF, ElseF, ThenR, ElseR>(
    condition: impl Fn() -> bool + 'static,
    then_fn: ThenF,
    else_fn: Option<ElseF>,
) -> Cleanup
where
    ThenF: Fn() -> ThenR + 'static,
    ElseF: Fn() -> ElseR + 'static,
    ThenR: Into<Cleanup>,
    ElseR: Into<Cleanup>,
{
    let scope = effect_scope();
    let parent_index = get_current_parent_index();

    let cleanup: Rc<RefCell<Option<Cleanup>>> = Rc::new(RefCell::new(None));
    let was_true: Rc<Cell<Option<bool>>> = Rc::new(Cell::new(None));

    let cleanup_for_effect = cleanup.clone();
    let cleanup_for_dispose = cleanup.clone();

    scope.run(|| {
        effect(move || {
            let cond = condition();

            if was_true.get() == Some(cond) {
                return;
            }
            was_true.set(Some(cond));

            // Cleanup previous
            if let Some(c) = cleanup_for_effect.borrow_mut().take() {
                c();
            }

            // Render with parent context
            if let Some(p) = parent_index {
                push_parent_context(p);
            }

            let new_cleanup = if cond {
                Some(then_fn().into())
            } else {
                else_fn.as_ref().map(|f| f().into())
            };

            if parent_index.is_some() {
                pop_parent_context();
            }

            *cleanup_for_effect.borrow_mut() = new_cleanup;
        });

        on_scope_dispose(move || {
            if let Some(c) = cleanup_for_dispose.borrow_mut().take() {
                c();
            }
        });
    });

    Box::new(move || scope.stop())
}
```

### each() - List Rendering

```rust
// Source: TypeScript each.ts pattern
use spark_signals::{effect, effect_scope, on_scope_dispose, signal, Signal};
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

use crate::engine::{get_current_parent_index, push_parent_context, pop_parent_context};
use crate::primitives::Cleanup;

/// List rendering with fine-grained updates.
///
/// # Arguments
/// * `items_getter` - Getter returning items Vec (auto-tracks)
/// * `render_fn` - Receives item getter and key, returns cleanup
/// * `key_fn` - Extract unique key from item
///
/// # Example
/// ```ignore
/// let todos = signal(vec![
///     Todo { id: 1, text: "Learn Rust".into() },
///     Todo { id: 2, text: "Build TUI".into() },
/// ]);
///
/// let cleanup = each(
///     move || todos.get(),
///     |get_item, key| {
///         let get_item = get_item.clone();
///         text(TextProps {
///             content: PropValue::Getter(Rc::new(move || get_item().text.clone())),
///             ..default()
///         })
///     },
///     |todo| todo.id,
/// );
/// ```
pub fn each<T, K, RenderF, R>(
    items_getter: impl Fn() -> Vec<T> + 'static,
    render_fn: RenderF,
    key_fn: impl Fn(&T) -> K + 'static,
) -> Cleanup
where
    T: Clone + PartialEq + 'static,
    K: Clone + Eq + Hash + std::fmt::Debug + 'static,
    RenderF: Fn(Rc<dyn Fn() -> T>, K) -> R + Clone + 'static,
    R: Into<Cleanup>,
{
    let scope = effect_scope();
    let parent_index = get_current_parent_index();

    let cleanups: Rc<RefCell<HashMap<K, Cleanup>>> = Rc::new(RefCell::new(HashMap::new()));
    let item_signals: Rc<RefCell<HashMap<K, Signal<T>>>> = Rc::new(RefCell::new(HashMap::new()));

    let cleanups_effect = cleanups.clone();
    let item_signals_effect = item_signals.clone();
    let cleanups_dispose = cleanups.clone();

    scope.run(|| {
        let render_fn = render_fn.clone();

        effect(move || {
            let items = items_getter();
            let mut current_keys = std::collections::HashSet::new();

            if let Some(p) = parent_index {
                push_parent_context(p);
            }

            for item in items.iter() {
                let key = key_fn(item);
                current_keys.insert(key.clone());

                let mut signals = item_signals_effect.borrow_mut();
                if !signals.contains_key(&key) {
                    // NEW item
                    let item_signal = signal(item.clone());
                    let item_sig_clone = item_signal.clone();
                    let getter: Rc<dyn Fn() -> T> = Rc::new(move || item_sig_clone.get());

                    let cleanup = render_fn(getter, key.clone()).into();

                    signals.insert(key.clone(), item_signal);
                    drop(signals); // Release borrow before cleanup insert
                    cleanups_effect.borrow_mut().insert(key, cleanup);
                } else {
                    // EXISTING item - update signal
                    if let Some(sig) = signals.get(&key) {
                        sig.set(item.clone());
                    }
                }
            }

            if parent_index.is_some() {
                pop_parent_context();
            }

            // Cleanup removed items
            let keys_to_remove: Vec<K> = cleanups_effect.borrow()
                .keys()
                .filter(|k| !current_keys.contains(*k))
                .cloned()
                .collect();

            for key in keys_to_remove {
                if let Some(cleanup) = cleanups_effect.borrow_mut().remove(&key) {
                    cleanup();
                }
                item_signals_effect.borrow_mut().remove(&key);
            }
        });

        on_scope_dispose(move || {
            for (_, cleanup) in cleanups_dispose.borrow_mut().drain() {
                cleanup();
            }
        });
    });

    Box::new(move || scope.stop())
}
```

### when() - Async Handling (Conceptual)

```rust
// Source: TypeScript when.ts pattern, adapted for synchronous Rust
// Note: True async requires runtime integration (tokio/async-std)

use spark_signals::{effect, effect_scope, on_scope_dispose, signal, Signal};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::engine::{get_current_parent_index, push_parent_context, pop_parent_context};
use crate::primitives::Cleanup;

/// Options for when() async handling.
pub struct WhenOptions<T, E, PendingF, ThenF, CatchF> {
    pub pending: Option<PendingF>,
    pub then_fn: ThenF,
    pub catch_fn: Option<CatchF>,
    _marker: std::marker::PhantomData<(T, E)>,
}

/// Async state for when().
///
/// Since spark-tui is synchronous, when() works with polling:
/// The user provides a getter that returns the current state.
#[derive(Clone)]
pub enum AsyncState<T, E> {
    Pending,
    Resolved(T),
    Rejected(E),
}

/// Render based on async state.
///
/// # Example
/// ```ignore
/// // User manages async state themselves
/// let fetch_state: Signal<AsyncState<Data, Error>> = signal(AsyncState::Pending);
///
/// // Start fetch in background (user's responsibility)
/// spawn_fetch(fetch_state.clone());
///
/// let cleanup = when(
///     move || fetch_state.get(),
///     WhenOptions {
///         pending: Some(|| text(TextProps { content: "Loading...".into(), ..default() })),
///         then_fn: |data| text(TextProps { content: data.to_string().into(), ..default() }),
///         catch_fn: Some(|err| text(TextProps { content: format!("Error: {}", err).into(), ..default() })),
///     },
/// );
/// ```
pub fn when<T, E, PendingF, ThenF, CatchF, PendingR, ThenR, CatchR>(
    state_getter: impl Fn() -> AsyncState<T, E> + 'static,
    options: WhenOptions<T, E, PendingF, ThenF, CatchF>,
) -> Cleanup
where
    T: Clone + 'static,
    E: Clone + std::fmt::Display + 'static,
    PendingF: Fn() -> PendingR + 'static,
    ThenF: Fn(T) -> ThenR + 'static,
    CatchF: Fn(E) -> CatchR + 'static,
    PendingR: Into<Cleanup>,
    ThenR: Into<Cleanup>,
    CatchR: Into<Cleanup>,
{
    let scope = effect_scope();
    let parent_index = get_current_parent_index();

    let cleanup: Rc<RefCell<Option<Cleanup>>> = Rc::new(RefCell::new(None));
    let cleanup_effect = cleanup.clone();
    let cleanup_dispose = cleanup.clone();

    scope.run(|| {
        effect(move || {
            let state = state_getter();

            // Cleanup previous
            if let Some(c) = cleanup_effect.borrow_mut().take() {
                c();
            }

            if let Some(p) = parent_index {
                push_parent_context(p);
            }

            let new_cleanup: Option<Cleanup> = match state {
                AsyncState::Pending => {
                    options.pending.as_ref().map(|f| f().into())
                }
                AsyncState::Resolved(data) => {
                    Some((options.then_fn)(data).into())
                }
                AsyncState::Rejected(err) => {
                    if let Some(ref catch_fn) = options.catch_fn {
                        Some(catch_fn(err).into())
                    } else {
                        eprintln!("[when] Unhandled rejection: {}", err);
                        None
                    }
                }
            };

            if parent_index.is_some() {
                pop_parent_context();
            }

            *cleanup_effect.borrow_mut() = new_cleanup;
        });

        on_scope_dispose(move || {
            if let Some(c) = cleanup_dispose.borrow_mut().take() {
                c();
            }
        });
    });

    Box::new(move || scope.stop())
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Virtual DOM diffing | Fine-grained signals | SolidJS 2020+ | No diff needed, direct updates |
| Key-based reconciliation | Per-item signals | SolidJS/Vue 3 | Items update without recreating |
| Manual cleanup tracking | EffectScope | Vue 3 2020 | Automatic nested cleanup |

**Deprecated/outdated:**
- Virtual DOM for reactive UIs: Signals eliminate need for diffing
- Array index as key: Causes unnecessary recreations on reorder

## Open Questions

1. **Async Runtime Integration**
   - What we know: TypeScript uses native Promises. Rust needs async runtime (tokio/async-std/smol).
   - What's unclear: Should when() require a runtime, or work with polling?
   - Recommendation: Start with polling-based AsyncState<T, E>. Users manage their own async. Add runtime integration later if needed.

2. **Index-based Default for each()**
   - What we know: CONTEXT.md mentions optional keyed mode with index-based default
   - What's unclear: How to efficiently track by index when items shift
   - Recommendation: Always require key function for first implementation. Add `.keyed()` extension later.

3. **Empty State for each()**
   - What we know: CONTEXT.md mentions `.empty()` chained method
   - What's unclear: Builder pattern vs wrapper type vs separate parameter
   - Recommendation: Start with optional empty_fn parameter. Add builder pattern if ergonomics suffer.

4. **Retry Support for when()**
   - What we know: CONTEXT.md mentions potential `.retry(count)`
   - What's unclear: How retry interacts with signal-based async state
   - Recommendation: Defer to later. Users can implement retry in their fetch logic.

## Sources

### Primary (HIGH confidence)
- TypeScript show.ts, each.ts, when.ts - Direct reference implementation
- TypeScript registry.ts - Parent context pattern
- TypeScript scope.ts - Component scope pattern
- Rust spark-signals EffectScope - Cleanup mechanism
- Rust registry.rs - Parent context stack

### Secondary (MEDIUM confidence)
- SolidJS documentation (conceptual patterns)
- Vue 3 Composition API (EffectScope origins)

### Tertiary (LOW confidence)
- None - all patterns verified against TypeScript reference

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Uses existing dependencies only
- Architecture: HIGH - Direct port from working TypeScript
- Pitfalls: HIGH - Identified from TypeScript implementation analysis

**Research date:** 2026-01-23
**Valid until:** Indefinite - patterns are stable, TypeScript reference is authoritative
