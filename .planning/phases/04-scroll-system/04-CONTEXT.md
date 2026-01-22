# Phase 4: Scroll System - Context

**Gathered:** 2026-01-22
**Status:** Ready for planning

<domain>
## Phase Boundary

Handle content overflow with scrolling. Components with content larger than their constrained dimensions become scrollable. Includes keyboard scrolling (focused element), mouse wheel scrolling (hovered element with fallback), scroll chaining for mouse only, and scrollIntoView for focus management.

</domain>

<decisions>
## Implementation Decisions

### Overflow behavior
- **Default implicit scrolling**: If overflow prop not set and content exceeds container with fixed/bounded height, auto-enable scrolling
- When auto-scroll triggers, auto-enable `focusable` and `scrollable` in interaction arrays
- Explicit `overflow: hidden` disables scrolling, shows only visible content
- `overflow: scroll` = full scrollbar (1 column track + thumb)
- `overflow: auto` = minimal scroll indicator (position marker only)
- **Vertical scroll only**: Skip horizontal scrolling for now - text wraps, flex items shrink, horizontal overflow is rare edge case

### Scroll interaction
- **Keyboard (focused element)**: Arrows scroll 1 line, PageUp/Down scroll viewport height, Home/End scroll to top/bottom
- **Mouse wheel (hovered element)**: Research opentui for smooth/analog scrolling approach (user will clone to docs/references). Fallback to focused scrollable if hovered element isn't scrollable.
- **No scroll margin**: Scroll minimum amount to reveal item, no extra context lines
- **Scrollbar interaction**: Claude's discretion (click-to-jump vs drag vs visual-only)

### Focus vs scroll relationship
- Scrollable containers enter focusable list with their tabIndex
- Tab cycles through all focusables (containers and children) based on tabIndex
- Focused element receives keyboard scroll events
- Mouse wheel targets hovered element, falls back to focused scrollable

### Scroll chaining
- **Mouse wheel only**: Chain to parent scrollable when nested scrollable hits boundary
- **No keyboard chaining**: Would conflict with focus management - user Tabs to focus parent if they want to scroll it
- Chain enabled by default for mouse wheel
- Boundary feedback: Claude's discretion

### scrollIntoView
- **Auto-scroll on focus**: When focus moves to element inside scrollable, auto-scroll if element is outside visible area
- **Alignment**: Claude's discretion (nearest edge vs center)
- **Nested scrollables**: Claude's discretion (all ancestors vs immediate parent)
- **Public API**: Expose `scroll_into_view(component_id)` for programmatic scrolling

### Stick-to-bottom
- Prop `stick_to_bottom: bool` on scrollable containers
- When enabled AND scroll position at bottom AND new content added → auto-scroll to new bottom
- User scrolling up (even 1 line) disables auto-follow
- User scrolling back to bottom re-enables auto-follow
- Use case: logs, chat views, terminal output

### Claude's Discretion
- Mouse wheel scroll amount (pending opentui research)
- Scrollbar click/drag interaction
- scrollIntoView alignment (nearest edge vs center)
- Nested scrollIntoView scope (all ancestors vs immediate parent)
- Visual boundary feedback on scroll chain handoff

</decisions>

<specifics>
## Specific Ideas

- "Auto-scroll-to-the-bottom for logs and chat-views" - stick_to_bottom prop pattern
- Research opentui for smooth/analog mouse scrolling (fluid movement, not line-based)
- Full scrollbar for `overflow: scroll`, minimal indicator for `overflow: auto`

</specifics>

<deferred>
## Deferred Ideas

- **Focus indicator**: Subtle `*` at top-right of focused element in theme accent color (enabled by default, optional). Better fit for Phase 5 Cursor System or Focus enhancement.
- **Horizontal scrolling**: Skip for now, rare in TUI with text wrapping and flex shrink

</deferred>

---

*Phase: 04-scroll-system*
*Context gathered: 2026-01-22*
