# Phase 5: Cursor System - Research

**Researched:** 2026-01-23
**Domain:** Terminal cursor rendering, blink animation, focus integration
**Confidence:** HIGH

## Summary

This phase implements visual cursor feedback for text entry components. The cursor system has two distinct parts: (1) **terminal-native cursor** for positioning the hardware cursor, and (2) **drawn cursor** for rendering cursor characters directly into the FrameBuffer with blink animation.

The TypeScript reference (`drawnCursor.ts`) provides a complete implementation pattern: cursor styles (block/bar/underline), blink animation via shared clocks per FPS, and focus integration where blink subscriptions start/stop based on focus state.

**Primary recommendation:** Implement a unified `animate()` primitive that handles all periodic animations (cursor blink and future spinners/progress), using shared timers per FPS for efficiency. The drawn cursor renders into FrameBuffer during the `render_input()` phase with INVERSE attribute for block style.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| crossterm | 0.28 | Terminal cursor ANSI sequences | Already in use, provides cursor_to, cursor_show/hide, cursor_shape |
| spark-signals | local | Reactive blink phase signal | Existing reactive primitive |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| std::time | stdlib | Duration for blink intervals | Timer calculations |
| std::thread | stdlib | Background timer thread | Blink clock management |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Thread-based timers | tokio timers | Async would add complexity; blocking timers are fine for TUI |
| Per-cursor timers | Shared FPS clocks | Shared clocks sync all cursors, more efficient |

**Installation:**
```bash
# No new dependencies needed - uses existing crossterm + spark-signals
```

## Architecture Patterns

### Recommended Module Structure
```
src/
├── state/
│   ├── mod.rs           # Re-export animate, cursor
│   ├── animate.rs       # NEW: animate() primitive with shared clocks
│   └── cursor.rs        # NEW: Terminal native cursor API
├── primitives/
│   └── input.rs         # Cursor rendering in render_input()
└── pipeline/
    └── frame_buffer_derived.rs  # Cursor integration in render_input()
```

### Pattern 1: Shared Blink Clocks per FPS
**What:** All animations at the same FPS share a single timer/signal
**When to use:** Cursor blink, spinners, any periodic animation
**Example:**
```rust
// Source: TypeScript drawnCursor.ts lines 88-140
struct BlinkRegistry {
    phase: Signal<bool>,    // true = visible
    subscribers: usize,     // Reference count
}

// Map<FPS, BlinkRegistry> - one registry per FPS
// Timer starts when first subscriber, stops when last unsubscribes
fn subscribe_to_blink(fps: u8) -> impl FnOnce() {
    // Returns unsubscribe function
}
```

### Pattern 2: Focus-Driven Blink Subscription
**What:** Blink starts on focus, stops on blur
**When to use:** Cursor blink should only animate when focused
**Example:**
```rust
// Source: TypeScript drawnCursor.ts lines 208-221
// In createCursor:
let unsub_focus = focus::register_callbacks(index, FocusCallbacks {
    on_focus: Some(Box::new(move || {
        unsub_blink = Some(subscribe_to_blink(fps));
    })),
    on_blur: Some(Box::new(move || {
        if let Some(unsub) = unsub_blink.take() {
            unsub();
        }
    })),
});
```

### Pattern 3: Cursor Rendering in FrameBuffer
**What:** Cursor is drawn as a character in the buffer, not terminal-native
**When to use:** Default for inputs (consistent across terminals)
**Example:**
```rust
// During render_input in frame_buffer_derived.rs:
let cursor_visible = interaction::get_cursor_visible(index) && is_focused;
if cursor_visible {
    let cursor_pos = interaction::get_cursor_position(index);
    let cursor_x = content_x + cursor_pos - scroll_offset;

    // Block cursor: render with INVERSE attribute
    let char_at_cursor = text.chars().nth(cursor_pos).unwrap_or(' ');
    buffer.set_cell(cursor_x, content_y, char_at_cursor as u32, bg, fg, Attr::INVERSE, clip);
}
```

### Anti-Patterns to Avoid
- **Per-cursor timers:** Creating a timer for each cursor wastes resources and causes visual desync
- **Terminal-native cursor for inputs:** Inconsistent rendering across terminals, can't customize appearance
- **Blink in render loop:** Causes unnecessary re-renders; use reactive signal instead

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Cursor positioning | Manual ANSI escape codes | `ansi::cursor_to()` in `renderer/ansi.rs` | Already implemented correctly |
| Cursor visibility | Manual escape codes | `ansi::cursor_show/hide()` | Already implemented |
| Cursor shape | Manual DECSCUSR codes | `ansi::cursor_shape()` | Already implemented |
| Timer management | Raw thread spawning | Shared clock registry pattern | Handles subscriber counting, cleanup |
| Focus tracking | Polling focused state | `focus::register_callbacks()` | Already exists, fires at source |

**Key insight:** The terminal-native cursor API is already complete in `ansi.rs`. The drawn cursor is the primary work, and the blink animation is the complexity center.

## Common Pitfalls

### Pitfall 1: Timer Leaks
**What goes wrong:** Timers continue running after all cursors are disposed
**Why it happens:** Forgetting to unsubscribe from blink clock on cleanup
**How to avoid:** Use reference counting in blink registry; stop timer when subscribers hit 0
**Warning signs:** CPU usage increases over time, animations run after components unmount

### Pitfall 2: Blink Phase Desync
**What goes wrong:** Multiple cursors blink at different times
**Why it happens:** Each cursor creates its own timer instead of sharing
**How to avoid:** Shared clocks per FPS - all 2 FPS animations share one timer
**Warning signs:** Visual "chasing" effect when multiple inputs are visible

### Pitfall 3: Focus/Blur Race Conditions
**What goes wrong:** Cursor stays visible after blur, or invisible after focus
**Why it happens:** Blink subscription state out of sync with focus state
**How to avoid:** Use focus callbacks (fires at source), reset blink phase on focus
**Warning signs:** Cursor frozen in wrong state after tab navigation

### Pitfall 4: Cursor Position Off by Scroll Offset
**What goes wrong:** Cursor renders at wrong position in scrolled input
**Why it happens:** Not accounting for horizontal scroll offset
**How to avoid:** `cursor_render_x = cursor_pos - scroll_offset_x`
**Warning signs:** Cursor appears outside visible text region

### Pitfall 5: Block Cursor Hides Character
**What goes wrong:** Block cursor shows blank space instead of character underneath
**Why it happens:** Drawing cursor char instead of text char with inverse
**How to avoid:** For block: render the text character with INVERSE attribute
**Warning signs:** Characters "disappear" when cursor moves over them

## Code Examples

Verified patterns from TypeScript reference:

### Cursor Style Characters
```rust
// Source: TypeScript drawnCursor.ts lines 79-83
const CURSOR_CHARS: [(CursorStyle, char); 3] = [
    (CursorStyle::Block, '\0'),      // 0 = special case: inverse
    (CursorStyle::Bar, '\u{2502}'),  // │ vertical line
    (CursorStyle::Underline, '_'),   // underscore
];
```

### Blink Registry Structure
```rust
// Source: TypeScript drawnCursor.ts lines 90-96
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

struct BlinkRegistry {
    phase: Signal<bool>,
    handle: Option<thread::JoinHandle<()>>,
    running: Arc<AtomicBool>,
    subscribers: usize,
}

// Singleton map: FPS -> Registry
thread_local! {
    static BLINK_REGISTRIES: RefCell<HashMap<u8, BlinkRegistry>> = RefCell::new(HashMap::new());
}
```

### Subscribe/Unsubscribe Pattern
```rust
// Source: TypeScript drawnCursor.ts lines 113-140
fn subscribe_to_blink(fps: u8) -> Box<dyn FnOnce()> {
    if fps == 0 {
        return Box::new(|| {}); // No-op for disabled blink
    }

    BLINK_REGISTRIES.with(|reg| {
        let mut reg = reg.borrow_mut();
        let entry = reg.entry(fps).or_insert_with(|| {
            let phase = signal(true);
            BlinkRegistry {
                phase,
                handle: None,
                running: Arc::new(AtomicBool::new(false)),
                subscribers: 0,
            }
        });

        entry.subscribers += 1;

        // Start timer if first subscriber
        if entry.subscribers == 1 {
            let ms = 1000 / fps as u64 / 2; // Divide by 2 for on/off cycle
            let phase = entry.phase.clone();
            let running = entry.running.clone();
            running.store(true, Ordering::SeqCst);

            entry.handle = Some(thread::spawn(move || {
                while running.load(Ordering::SeqCst) {
                    thread::sleep(Duration::from_millis(ms));
                    phase.set(!phase.get());
                }
            }));
        }
    });

    // Return unsubscribe
    Box::new(move || {
        BLINK_REGISTRIES.with(|reg| {
            let mut reg = reg.borrow_mut();
            if let Some(entry) = reg.get_mut(&fps) {
                entry.subscribers = entry.subscribers.saturating_sub(1);
                if entry.subscribers == 0 {
                    entry.running.store(false, Ordering::SeqCst);
                    entry.phase.set(true); // Reset to visible
                    // Thread will exit on next iteration
                }
            }
        });
    })
}
```

### Cursor Visibility Getter
```rust
// Source: TypeScript drawnCursor.ts lines 224-239
// In createCursor, set up reactive cursor visibility:
interaction::set_cursor_visible_getter(index, move || {
    // Manual override takes precedence
    if let Some(manual) = manual_visible.get() {
        return manual;
    }
    // Not focused = always show cursor (no blink)
    if !focus::is_focused(index) {
        return true;
    }
    // Focused + blink disabled = always visible
    if !should_blink {
        return true;
    }
    // Focused + blink enabled = follow blink clock
    get_blink_phase(fps)
});
```

### Render Cursor in FrameBuffer
```rust
// In frame_buffer_derived.rs render_input():
fn render_input_cursor(
    buffer: &mut FrameBuffer,
    index: usize,
    content_x: u16,
    content_y: u16,
    content_w: u16,
    text: &str,
    fg: Rgba,
    bg: Rgba,
    clip: &ClipRect,
) {
    // Only render if focused and visible
    let is_focused = focus::is_focused(index);
    let cursor_visible = interaction::get_cursor_visible(index);

    if !is_focused || !cursor_visible {
        return;
    }

    let cursor_pos = interaction::get_cursor_position(index) as usize;
    let scroll_x = interaction::get_scroll_offset_x(index) as usize;

    // Calculate cursor screen position
    let cursor_screen_x = cursor_pos.saturating_sub(scroll_x);
    if cursor_screen_x >= content_w as usize {
        return; // Cursor off-screen
    }

    let render_x = content_x + cursor_screen_x as u16;

    // Get character at cursor (or space if at end)
    let char_at_cursor = text.chars().nth(cursor_pos).unwrap_or(' ');

    // TODO: Support cursor styles (Block = INVERSE, Bar = special char, Underline = special char)
    // For now, block cursor with inverse
    buffer.set_cell(
        render_x,
        content_y,
        char_at_cursor as u32,
        bg,  // Swap fg/bg for inverse
        fg,
        Attr::INVERSE,
        Some(clip),
    );
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Terminal-native cursor only | Drawn cursor default | Modern TUI frameworks | Better customization, consistency |
| Per-component timers | Shared FPS clocks | Performance optimization | Reduced CPU, synced animations |
| Polling for blink | Reactive signal | With reactive frameworks | Cleaner, no polling loops |

**Deprecated/outdated:**
- Manual cursor positioning in raw mode: Use the existing `ansi::cursor_to()` helper
- Direct terminal cursor for input fields: Prefer drawn cursor for consistency

## Open Questions

Things that couldn't be fully resolved:

1. **Thread vs Async Timers**
   - What we know: TypeScript uses `setInterval`, Rust needs explicit threading or async
   - What's unclear: Whether async (tokio) would be better than std::thread
   - Recommendation: Use std::thread for simplicity; TUI doesn't need async complexity

2. **Selection Highlighting**
   - What we know: CONTEXT.md mentions selection highlighting with inverse colors
   - What's unclear: Exact integration with cursor rendering
   - Recommendation: Plan for it but defer to Phase 5 scope (cursor only, selection is display)

3. **Cursor Color Customization**
   - What we know: CursorConfig has fg/bg color options in props
   - What's unclear: How to blend custom colors with inverse rendering
   - Recommendation: Start with theme colors, custom colors can override

## Sources

### Primary (HIGH confidence)
- TypeScript reference: `/Users/rusty/Documents/Projects/TUI/tui/src/state/drawnCursor.ts` - Complete implementation
- TypeScript reference: `/Users/rusty/Documents/Projects/TUI/tui/src/state/cursor.ts` - Terminal native API
- TypeScript reference: `/Users/rusty/Documents/Projects/TUI/tui/src/primitives/input.ts` - Input cursor integration

### Secondary (HIGH confidence)
- Existing Rust code: `src/renderer/ansi.rs` - ANSI cursor sequences
- Existing Rust code: `src/engine/arrays/interaction.rs` - Cursor arrays
- Existing Rust code: `src/state/focus.rs` - Focus callbacks
- Existing Rust code: `src/primitives/types.rs` - CursorStyle, CursorConfig

### Context (HIGH confidence)
- Phase context: `.planning/phases/05-cursor-system/05-CONTEXT.md` - User decisions

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Using existing libraries (crossterm, spark-signals)
- Architecture: HIGH - TypeScript reference provides complete pattern
- Pitfalls: HIGH - Common issues documented in TS implementation comments

**Research date:** 2026-01-23
**Valid until:** 60 days (stable domain, no fast-moving dependencies)

## Implementation Notes from CONTEXT.md

The following decisions were made in the discussion phase and must be followed:

1. **Hybrid cursor model**: Drawn cursor by default, terminal native available via override
2. **Animation primitive**: Named `animate()` (not `useAnimation`), unified for cursor blink + general animations
3. **Shared clocks per FPS**: All animations at same FPS share one timer
4. **Blink default**: 2 FPS (500ms on/off cycle)
5. **Block cursor**: Render with INVERSE (swap fg/bg), shows character underneath
6. **Bar/Underline**: Render cursor character with component's fg/bg
7. **Focus integration**: Cursor only visible when focused, blink subscription starts on focus
