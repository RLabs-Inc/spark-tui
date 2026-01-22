# spark-tui

## What This Is

A fully reactive terminal UI framework for Rust, porting the battle-tested TypeScript TUI framework (v0.8.2) to Rust. Built on spark-signals, it provides TypeScript-like ergonomics with Rust's performance and safety guarantees. Target users are Rust developers who want to build terminal applications with a modern reactive component model.

## Core Value

**Reactive correctness AND TypeScript-like ergonomics** — Signals must propagate correctly (if reactivity breaks, nothing works), and the API must feel like TypeScript (use macros if needed to achieve near-identical developer experience). Both are essential for validation.

## Requirements

### Validated

<!-- Shipped and confirmed valuable. -->

- ✓ Reactive signals library (spark-signals) — 161 tests, production ready
- ✓ Parallel arrays (ECS pattern) for component storage — core architecture
- ✓ FlexNode with 33 reactive Slot properties — layout integration
- ✓ Component registry with index allocation — O(1) reuse
- ✓ Taffy bridge for flexbox layout — layout_derived
- ✓ FrameBuffer with Cell structure — rendering foundation
- ✓ DiffRenderer for fullscreen mode — differential updates
- ✓ InlineRenderer for inline mode — clear + redraw
- ✓ AppendRenderer for append mode — history + active region
- ✓ Box primitive with layout props, borders, colors — container component
- ✓ Text primitive with content, attrs, align, wrap — display component
- ✓ Focus system basics (tab cycling, trap, history) — navigation foundation
- ✓ Keyboard event types and dispatch — input handling foundation

### Active

<!-- Current scope. Building toward these. -->

**Phase 1: Mouse System + Event Wiring**
- [ ] HitGrid for O(1) coordinate-to-component lookup
- [ ] Mouse event dispatch (down, up, move, scroll)
- [ ] Hover tracking with enter/leave detection
- [ ] Click detection (down + up on same component)
- [ ] Connect Box/Text onClick, onMouse* callbacks
- [ ] Connect onKey callbacks to components

**Phase 2: Theme System**
- [ ] Rgba color type with DEFAULT (-1) and ANSI (-2) sentinels
- [ ] Theme struct with semantic color signals
- [ ] Terminal default preset (respects terminal colors)
- [ ] t.* accessor pattern for theme colors
- [ ] Color inheritance through component tree
- [ ] At least 3 built-in presets (terminal, dracula, nord)

**Phase 3: Input Component**
- [ ] Two-way value binding with Signal
- [ ] Placeholder with separate color
- [ ] Password mode with mask character
- [ ] Cursor system (position tracking, movement)
- [ ] Keyboard navigation (arrows, home, end, backspace, delete)
- [ ] Events: onChange, onSubmit, onCancel
- [ ] Auto-focus support

**Phase 4: Scroll System**
- [ ] Overflow modes (visible, hidden, scroll, auto)
- [ ] ScrollManager per component
- [ ] Keyboard scrolling (arrows, PageUp/Down, Home/End)
- [ ] Mouse wheel scrolling
- [ ] Scroll chaining (parent fallback at boundaries)
- [ ] scrollIntoView for focus changes

**Phase 5: Cursor System**
- [ ] Terminal cursor positioning
- [ ] Drawn cursor for inputs (character rendered)
- [ ] Cursor styles (block, bar, underline)
- [ ] Blink animation with shared clock
- [ ] Focus integration (show/blink when focused)

**Phase 6: Control Flow**
- [ ] show(condition, render, else) — conditional rendering
- [ ] each(items, render, {key}) — list rendering with fine-grained updates
- [ ] when(promise, {pending, then, catch}) — async handling

### Out of Scope

<!-- Explicit boundaries. Includes reasoning to prevent re-adding. -->

- GPU rendering — Terminal is the target, not graphics cards
- Custom layout engines — Taffy (flexbox) is sufficient and battle-tested
- Networking/IO — Framework is UI only; users handle their own IO
- State management beyond signals — spark-signals is the foundation
- Web target — Terminal-only; web has plenty of options

## Context

**TypeScript Reference:** `/Users/rusty/Documents/Projects/TUI/tui/` (v0.8.2)
- Battle-tested, production-ready implementation
- All patterns and ergonomics are proven
- Key files documented in CLAUDE.md

**Existing Rust Implementation:**
- `crates/signals/` — Complete, 161 tests
- `crates/tui/` — Partial implementation
- 10 detailed spec files in `crates/tui/docs/specs/`

**Architecture Patterns:**
- Parallel arrays (ECS) for component storage
- FlexNode with reactive Slots bound to props
- Pure derived-based render pipeline
- Three rendering modes (fullscreen, inline, append)

## Constraints

- **Tech stack**: Rust stable, no nightly features — broad compatibility
- **Dependencies**: spark-signals, taffy, crossterm — minimal, proven
- **Ergonomics**: Must use macros to achieve TypeScript-like API — non-negotiable
- **Testing**: TDD strict — tests first, implement to pass
- **Reference**: TypeScript version is the specification — match behavior exactly

## Key Decisions

<!-- Decisions that constrain future work. Add throughout project lifecycle. -->

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| ECS pattern with parallel arrays | Match TypeScript architecture; cache-friendly memory layout | ✓ Good |
| FlexNode with reactive Slots | Direct prop→layout binding preserves reactivity | ✓ Good |
| Taffy for flexbox | Mature, well-tested layout engine; matches TypeScript Yoga usage | ✓ Good |
| spark-signals as foundation | Full control over reactive primitives; TypeScript-like API | ✓ Good |
| TDD approach | Catch regressions early; spec files define expected behavior | — Pending |
| Macros for ergonomics | Achieve TypeScript-like syntax in Rust | — Pending |

---
*Last updated: 2026-01-22 after GSD initialization*
