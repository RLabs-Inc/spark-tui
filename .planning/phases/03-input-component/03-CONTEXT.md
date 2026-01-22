# Phase 3: Input Component - Context

**Gathered:** 2026-01-22
**Status:** Ready for planning

<domain>
## Phase Boundary

Text entry primitive with full editing capabilities. Single-line text input for forms and commands. Users can type, edit, select, and submit text. Multiline input and vim-style editing are separate phases.

</domain>

<decisions>
## Implementation Decisions

### Visual presentation
- Full border support like Box (all border styles, per-side control)
- Focus indicator: `*` character in configurable position (default top-right), theme accent color
  - **Note:** Focus indicator is system-level, not per-Input (affects all focusable components)
- Cursor fully customizable: style (block/bar/underline), custom char, color, blink
- Placeholder shown as dimmed text when empty
- Password mask character is configurable (user chooses)
- Text alignment configurable (left/center/right), default left
- Input text is plain — no formatting attributes (bold, italic, etc.)
- Typing resets cursor blink (cursor goes solid briefly)
- Long text: cursor always visible + edge indicators (< >) when text extends beyond view
- Emphasis on full customization over any specific visual reference

### Keyboard behavior
- Ctrl+A selects all (when focused)
- Ctrl+Left/Right jumps by word
- Ctrl+Backspace/Delete deletes word
- Home/End for start/end navigation
- Enter submits (fixed, not configurable)
- Up/Down cycles through input history (when history provided)
- History: either user-provided prop OR auto-tracked (configurable)
- Minimal shortcuts only — vim-style deferred to multiline input

### Value binding
- Both controlled and uncontrolled modes supported
- Leverage spark-signals two-way bind and Slot for flexible binding
- onChange fires on every keystroke
- maxLength prop supported (prevents typing beyond limit)
- onSubmit receives current value: `onSubmit(value: String)`

### Selection & clipboard
- Text selection supported (Shift+arrows to select)
- Shift+Ctrl+arrows for word-level selection
- System clipboard for copy/paste (Ctrl+C/V)
- Ctrl+X for cut supported

### Claude's Discretion
- Cancel key (likely Escape)
- Width behavior (fixed vs grow to max)
- Clipboard fallback if system clipboard unavailable

</decisions>

<specifics>
## Specific Ideas

- Focus indicator pattern (`*` in corner) will extend to Box and all focusable components — design it as a shared system
- Reference spark-signals crate (`../signals`) for two-way bind and Slot when implementing value binding
- Edge indicators (< >) for text overflow — common pattern in terminal inputs

</specifics>

<deferred>
## Deferred Ideas

- Vim-style editing — deferred to multiline input phase
- Multiline input component — separate phase

</deferred>

---

*Phase: 03-input-component*
*Context gathered: 2026-01-22*
