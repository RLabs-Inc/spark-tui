# Phase 5: Cursor System - Backlog

## Planned Features (per ROADMAP.md)

- Terminal cursor positioning
- Drawn cursor rendering
- Blink animation
- Focus integration

---

## Additional Features

### Focus Indicator for Box/Text

**Priority:** Medium
**Added:** 2026-01-22
**Context:** Discussed during keyboard system refactor

**Description:**
Visual indicator showing which component is focused. Avoids the "focus ring" pattern that web developers hate.

**Implementation:**
- **Box (with border):** Show `*` at top-right corner in theme accent color
- **Text (focusable):** Show `*` after text content in theme accent color
- **Input:** No indicator needed (cursor already shows focus)

**Props:**
```rust
// BoxProps
pub focus_indicator: Option<bool>,  // default: true

// TextProps
pub focus_indicator: Option<bool>,  // default: true
```

**Technical Notes:**
- Render in `frame_buffer_derived.rs` during component rendering
- Check `focus::get_focused_index() == index` to determine if focused
- Use `t.accent()` for color (requires theme integration in renderer)
- Edge cases:
  - Box without border: indicator at content top-right
  - Text with truncation: indicator replaces last char? or appends?
  - Nested focusables: only innermost shows indicator

**Estimated Effort:** ~50-100 lines across frame_buffer_derived, theme integration

---

*Last updated: 2026-01-22*
