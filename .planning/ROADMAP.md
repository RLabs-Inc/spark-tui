# spark-tui Roadmap

## Milestone 1: Complete TUI Port

**Goal:** Full feature parity with TypeScript TUI v0.8.2

**Success Criteria:**
- All 6 phases complete
- All requirements validated
- TypeScript-like ergonomics achieved
- TDD throughout

---

## Phase 1: Mouse System + Event Wiring

**Goal:** Enable components to respond to mouse and keyboard interactions.

**Requirements:** R1.1-R1.5

**Deliverables:**
- HitGrid implementation
- MouseEvent parsing and dispatch
- Hover tracking (enter/leave)
- Click detection
- Callback wiring for Box, Text
- Global keyboard handlers

**Dependencies:** None (builds on existing infrastructure)

**Plans:** 4 plans

**Status:** COMPLETE (2026-01-22)

Plans:
- [x] 01-01-PLAN.md - Mouse module foundation (types, HitGrid, registry, dispatch)
- [x] 01-02-PLAN.md - Input integration (crossterm event conversion)
- [x] 01-03-PLAN.md - Component wiring (BoxProps/TextProps callbacks)
- [x] 01-04-PLAN.md - Event loop integration (mount, global keys)

---

## Phase 2: Theme System

**Goal:** Enable visual customization with reactive theme colors.

**Requirements:** R2.1-R2.5

**Deliverables:**
- ThemeColor type with color parsing (hex, OKLCH)
- Theme struct with all semantic colors
- All 13 TypeScript presets
- Reactive theme state with signals
- t.* accessor pattern with deriveds
- Variant system with contrast calculation
- Color modifiers (lighten, darken, alpha, etc.)

**Dependencies:** None (can parallel with Phase 1)

**Plans:** 4 plans

**Status:** COMPLETE (2026-01-22)

Plans:
- [x] 02-01-PLAN.md - Color parsing and theme types (ThemeColor, Theme, 13 presets)
- [x] 02-02-PLAN.md - Reactive theme state and t.* accessor
- [x] 02-03-PLAN.md - Variant system with contrast calculation
- [x] 02-04-PLAN.md - Color modifiers and t.contrast()

---

## Phase 3: Input Component

**Goal:** Text entry primitive with full editing capabilities.

**Requirements:** R3.1-R3.7

**Deliverables:**
- Input component with value binding
- Placeholder support
- Password mode
- Cursor position tracking
- Keyboard navigation (basic + word-level with Ctrl)
- Selection support (Shift+arrows)
- Clipboard operations (Ctrl+C/V/X)
- History navigation (Up/Down arrows)
- onChange/onSubmit/onCancel events

**Dependencies:**
- Phase 1 (click-to-focus)
- Phase 5 (cursor rendering) - partial, can start without blink

**Plans:** 4 plans

**Status:** COMPLETE (2026-01-22)

Plans:
- [x] 03-01-PLAN.md - Types and foundation (InputProps, CursorStyle, basic rendering)
- [x] 03-02-PLAN.md - Word navigation and Ctrl+A (Ctrl+arrows, Ctrl+Backspace/Delete)
- [x] 03-03-PLAN.md - Selection and clipboard (Shift+arrows, Ctrl+C/V/X)
- [x] 03-04-PLAN.md - History and overflow (Up/Down arrows, scroll offset)

---

## Phase 4: Scroll System

**Goal:** Handle content overflow with scrolling.

**Requirements:** R4.1-R4.6

**Deliverables:**
- Overflow modes
- ScrollManager per component
- Keyboard scrolling
- Mouse wheel scrolling
- Scroll chaining
- scrollIntoView

**Dependencies:**
- Phase 1 (mouse wheel events)

**Plans:** 5 plans

**Status:** GAPS FOUND - Gap closure plan created

Plans:
- [x] 04-01-PLAN.md - Scroll core module (offset, bounds, operations)
- [x] 04-02-PLAN.md - Keyboard scroll handlers (arrow, page, home/end)
- [x] 04-03-PLAN.md - Mouse wheel scroll handlers (chaining, scrollIntoView)
- [x] 04-04-PLAN.md - Scrollbar rendering + stick_to_bottom
- [ ] 04-05-PLAN.md - Gap closure: pipeline integration (layout accessor, stick_to_bottom effect)

---

## Phase 5: Cursor System

**Goal:** Visual cursor feedback for text entry.

**Requirements:** R5.1-R5.4

**Deliverables:**
- Terminal cursor positioning
- Drawn cursor rendering
- Blink animation
- Focus integration

**Dependencies:**
- Phase 3 (Input needs cursor)

**Plans:** 0 plans

---

## Phase 6: Control Flow

**Goal:** Ergonomic helpers for dynamic UI.

**Requirements:** R6.1-R6.3

**Deliverables:**
- show() conditional rendering
- each() list rendering
- when() async handling

**Dependencies:** None (orthogonal to other phases)

**Plans:** 0 plans

---

## Phase Execution Order

**Recommended sequence:**

```
Phase 1: Mouse + Events -----+
                             +---> Phase 3: Input --> Phase 5: Cursor
Phase 2: Theme --------------+         |
                                       v
                              Phase 4: Scroll

Phase 6: Control Flow (can run in parallel)
```

**Parallelization opportunities:**
- Phase 1 and Phase 2 can run in parallel
- Phase 6 is independent and can run anytime
- Phase 3 can start after Phase 1, cursor rendering (Phase 5) added later

---

## Validation Approach

Each phase ends with:
1. All requirements tested (TDD)
2. Integration test with example app
3. CLAUDE.md updated with completion status
4. Demo showing feature working

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Macro ergonomics difficult | Start simple, iterate; reference TypeScript API as target |
| Performance regression | Benchmark against TypeScript version; profile early |
| Reactivity bugs | spark-signals has 161 tests; add integration tests |
| Edge cases in scroll/cursor | Reference TypeScript implementation for behavior |

---

*Last updated: 2026-01-22*
