# Phase 5: Cursor System - Context

**Gathered:** 2026-01-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Visual cursor feedback for text entry. The Input component (Phase 3) already handles text editing, cursor position tracking, and keyboard navigation — this phase makes the cursor *visible* and provides the animation primitive that powers blink.

Includes:
- Terminal native cursor API (positioning, shape, visibility)
- Drawn cursor for Inputs (rendered into frameBuffer)
- Animation primitive (`animate()`) for cursor blink and general use
- Selection highlighting

</domain>

<decisions>
## Implementation Decisions

### Rendering Approach
- **Hybrid model**: Default to drawn cursor (consistent across terminals), allow override to terminal native
- **Smart default + override**: Drawn cursor by default, `cursor_mode: Native` prop to switch
- **Both APIs exposed**: Framework users get both terminal native and drawn cursor APIs with full TypeScript-equivalent ergonomics

### Drawn Cursor Rendering
- **Block cursor**: Inverse (swap fg/bg) — shows character underneath
- **Bar/Underline/Custom**: Render cursor char with component's fg/bg
- **Terminal theme handling**: When bg is unset, `TERMINAL_DEFAULT` used — terminal handles inversion
- **Blink off phase**: Either show altChar or show original text (configurable)

### Animation Primitive
- **Name**: `animate()` (not `useAnimation` — less React-y)
- **Unified system**: One animation primitive for cursor blink AND general animations (spinners, progress)
- **Shared clocks per FPS**: All animations at same FPS share one timer — efficient, sync'd
- **Included in Phase 5**: Powers cursor blink, reusable for other animations later

### Blink Behavior
- **Cursor only visible when focused**: No cursor shown on unfocused Inputs
- **Blink only when focused + blink enabled**: If `blink: false`, cursor shows solid
- **Default FPS**: 2 (500ms on/off cycle — standard cursor blink rate)
- **Focus integration**: Blink subscription starts on focus, stops on blur

### Cursor Styles
- **Built-in styles**: Block, Bar (│), Underline (_), Half-block (▄), Box outline (☐)
- **Custom char support**: User can pass any character as cursor
- **Default style**: Block
- **Per-Input configuration**: Each Input can have its own cursor style

### Cursor Colors
- **Theme colors supported**: e.g., `t.primary()`
- **Custom colors supported**: Explicit Rgba values
- **Default**: Inherit from component fg/bg

### Visual Feedback
- **Selection highlighting**: Inverse (swap fg/bg) — safe contrast with any theme
- **Unfocused state**: No cursor visible
- **Focus indication**: Cursor presence only (no border change)
- **End position**: Cursor shows on empty cell (space with cursor style)

### Public APIs
1. **Terminal native cursor** — `cursor.show()`, `cursor.hide()`, `cursor.set_shape()`, `cursor.move_to()`, `cursor.save()`, `cursor.restore()`
2. **Drawn cursor** — `create_cursor()`, `dispose_cursor()` for components
3. **Animation** — `animate(frames, options)` for general animations

### Claude's Discretion
- Animation primitive implementation details
- Exact interaction array structure for cursor state
- Integration with existing Input component from Phase 3
- Terminal native cursor ANSI escape code implementation

</decisions>

<specifics>
## Specific Ideas

- Follow TypeScript TUI implementation patterns exactly for cursor rendering logic
- `animate()` naming to avoid React-y conventions
- Both terminal native AND drawn cursor APIs should be public, with same ergonomics as TypeScript version
- Unified animation system (unlike TS which has separate animation.ts and drawnCursor.ts blink registries)

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 05-cursor-system*
*Context gathered: 2026-01-23*
