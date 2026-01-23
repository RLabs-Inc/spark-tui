# Phase 6: Control Flow - Context

**Gathered:** 2026-01-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Ergonomic helpers for dynamic UI: `show()` for conditional rendering, `each()` for list rendering, `when()` for async handling. These are control flow primitives, not new component types.

</domain>

<decisions>
## Implementation Decisions

### show() Conditional Rendering
- Accepts `bool` or `Signal<bool>` as condition (auto-tracks signals)
- Optional else parameter: `show(cond, then_render, else_render?)`
- **Destroy and recreate** when condition toggles (no hidden state preservation)
- Simpler mental model, matches TypeScript behavior

### each() List Rendering
- Key function optional: `each(items, |item, idx| render)` with `.keyed(|item| key)` for move detection
- **Always provides index**: callback is `|item, idx|`
- Empty state via chained method: `each(items, render).empty(|| fallback)`
- Accepts `Signal<Vec<T>>` and `ReactiveVec<T>` collection types
- Fine-grained updates when using ReactiveVec with keys

### when() Async Handling
- **Built-in spinner** with full customization via `.pending(|| custom_render)`
- Optional `.catch(|err| render)` handler
- Unhandled errors log to stderr and render nothing (don't crash app)
- **Cancel on unmount** - drop the future, never call callbacks

### API Ergonomics
- **Standalone functions**: `use spark_tui::{show, each, when};`
- Return component indices AND add to parent context (both behaviors)
- **Fully nestable** - any combination of show/each/when inside each other
- TypeScript-like ergonomics is the north star

### Claude's Discretion
- Retry support for when() (whether to include `.retry(count)`)
- Render callback context (just data vs additional context parameter)
- Built-in spinner design/animation
- Internal implementation details for efficient diffing

</decisions>

<specifics>
## Specific Ideas

- TypeScript-like ergonomics as much as possible - should feel natural to someone familiar with the TS version
- Control flow helpers should integrate seamlessly with Box, Text, Input primitives
- Index-based default for each() (no key required) enables simple lists without boilerplate

</specifics>

<deferred>
## Deferred Ideas

None - discussion stayed within phase scope

</deferred>

---

*Phase: 06-control-flow*
*Context gathered: 2026-01-23*
