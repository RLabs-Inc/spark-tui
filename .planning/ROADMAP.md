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

Plans:
- [ ] 01-01-PLAN.md - Mouse module foundation (types, HitGrid, registry, dispatch)
- [ ] 01-02-PLAN.md - Input integration (crossterm event conversion)
- [ ] 01-03-PLAN.md - Component wiring (BoxProps/TextProps callbacks)
- [ ] 01-04-PLAN.md - Event loop integration (mount, global keys)

---

## Phase 2: Theme System

**Goal:** Enable visual customization with reactive theme colors.

**Requirements:** R2.1-R2.5

**Deliverables:**
- Rgba color type with sentinels
- Theme struct with signals
- 3 presets (terminal, dracula, nord)
- t.* accessor pattern
- Color inheritance

**Dependencies:** None (can parallel with Phase 1)

**Plans:** 0 plans

---

## Phase 3: Input Component

**Goal:** Text entry primitive with full editing capabilities.

**Requirements:** R3.1-R3.7

**Deliverables:**
- Input component with value binding
- Placeholder support
- Password mode
- Cursor position tracking
- Keyboard navigation
- onChange/onSubmit/onCancel events

**Dependencies:**
- Phase 1 (click-to-focus)
- Phase 5 (cursor rendering) - partial, can start without blink

**Plans:** 0 plans

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

**Plans:** 0 plans

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
