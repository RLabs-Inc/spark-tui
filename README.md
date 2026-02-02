# SparkTUI

**Rust speed. TypeScript vibes.**

Build blazing-fast terminal UIs in TypeScript — powered by a Rust engine, without writing a single line of Rust.

```typescript
import { signal } from '@rlabs-inc/signals'
import { box, text, mount } from 'spark-tui'

const count = signal(0)

await mount(() => {
  box({ padding: 2, border: 1 }, () => {
    text({ content: () => `Count: ${count.value}` })
  })

  box({
    focusable: true,
    onClick: () => count.value++
  }, () => {
    text({ content: '[ + ]' })
  })
})
```

Change `count.value`. The UI updates in **50 microseconds**. No render loops. No polling. No `requestAnimationFrame`. Just reactive state that propagates instantly.

---

## Why SparkTUI?

Every TUI framework makes you choose:

- **Want TypeScript?** Accept JavaScript performance.
- **Want speed?** Write Rust, Go, or C.

SparkTUI says no. You get both.

|  | Ink / Blessed | Ratatui | SparkTUI |
|--|---------------|---------|----------|
| **Language** | TypeScript | Rust | TypeScript |
| **Idle CPU** | 5-10% | 0% | **0%** |
| **Update latency** | ~16ms | ~100μs | **~50μs** |
| **Layout engine** | JS | Rust | **Rust (Taffy)** |
| **Learning curve** | Easy | Steep | **Easy** |

---

## The Architecture

SparkTUI doesn't serialize data between TypeScript and Rust. They share the same memory.

```
┌─────────────────────────────────────────────────────────────┐
│  TypeScript                                                 │
│  ─────────────────────────────────────────────────────────  │
│  const count = signal(0)     // Reactive state              │
│  count.value = 42            // Write to SharedArrayBuffer  │
│                              // ↓ 0.4 nanoseconds           │
├─────────────────────────────────────────────────────────────┤
│  SharedArrayBuffer           // Same memory, both sides     │
│                              // ↓ FFI wake: 10 nanoseconds  │
├─────────────────────────────────────────────────────────────┤
│  Rust Engine                                                │
│  ─────────────────────────────────────────────────────────  │
│  • Taffy (W3C Flexbox)       // Layout: ~4μs                │
│  • Framebuffer               // Cell grid: ~144μs           │
│  • ANSI diff renderer        // Only changed cells          │
└─────────────────────────────────────────────────────────────┘
```

**No loops. No polling. No fixed FPS.**

Think of it like a spreadsheet: change a cell, dependents recalculate. SparkTUI works the same way — signals form a dependency graph, and changes propagate through it instantly.

---

## Features

### Primitives
- **`box`** — Flexbox/Grid container with borders, colors, scroll
- **`text`** — Styled text with alignment, wrapping, attributes
- **`input`** — Text input with cursor, validation, password masking

### Reactivity
- **`signal()`** — Reactive state
- **`derived()`** — Computed values
- **`each()`** — Reactive lists with fine-grained updates
- **`show()`** — Conditional rendering

### Layout
- Full **W3C Flexbox** via Taffy
- Full **CSS Grid** via Taffy
- `gridTemplateColumns`, `gridTemplateRows`, `gridAutoFlow`
- Percentage and fixed dimensions
- Padding, margin, gap
- `justify-content`, `align-items`, `flex-grow`

### Styling
- **13 built-in themes** (Dracula, Nord, Catppuccin, Tokyo Night...)
- Semantic variants (primary, success, warning, error)
- 10 border styles (single, double, rounded, bold...)
- Text attributes (bold, italic, underline, strikethrough)
- Colors: hex, RGB, HSL, OKLCH, ANSI 0-255

### Events
- Keyboard with modifiers (Ctrl, Alt, Shift)
- Mouse (click, hover, scroll)
- Focus management with Tab navigation
- Event bubbling

### Scroll
- Auto-scroll when content overflows
- Keyboard: arrows, Page Up/Down, Home/End
- Mouse wheel (fullscreen mode)
- Scroll chaining to parent containers

### Animation
- **`cycle()`** — Frame-based animation (spinners, progress)
- Built-in frame sets (spinner, dots, bounce, arrow...)
- Reactive: pauses when signal is false

---

## Performance

Benchmarked on Apple M1:

| Operation | Time | Throughput |
|-----------|------|------------|
| Signal write | 0.4ns | 2.26B/sec |
| FFI wake | 9.8ns | 101M/sec |
| Prop + wake | 12.3ns | 81M/sec |
| Full frame (layout + render) | ~558μs | 1,792 FPS |

**60 FPS budget**: 16,667μs
**SparkTUI usage**: ~558μs (3.3%)
**Headroom**: 96.7% free for your app logic

---

## Installation

```bash
# Coming soon to npm
bun add spark-tui
```

Requirements:
- Bun 1.0+ (Node support planned)
- macOS, Linux, or Windows

---

## Quick Start

```typescript
import { signal, derived } from '@rlabs-inc/signals'
import { box, text, input, mount } from 'spark-tui'
import { t, setTheme } from 'spark-tui/theme'

// Reactive state
const name = signal('')
const greeting = derived(() =>
  name.value ? `Hello, ${name.value}!` : 'Enter your name...'
)

// Set theme
setTheme('dracula')

// Mount app
await mount(() => {
  box({
    flexDirection: 'column',
    padding: 2,
    gap: 1,
    border: 1,
    borderColor: t.primary,
  }, () => {
    text({ content: 'Welcome to SparkTUI', fg: t.primary })

    input({
      value: name,
      placeholder: 'Your name',
      width: 30,
      border: 1,
    })

    text({ content: greeting, fg: t.success })
  })
})
```

---

## Themes

Switch themes with one line:

```typescript
import { setTheme } from 'spark-tui/theme'

setTheme('nord')        // Arctic blues
setTheme('dracula')     // Dark purple
setTheme('catppuccin')  // Soothing pastels
setTheme('tokyoNight')  // City lights
setTheme('gruvbox')     // Retro warm
// ... 8 more built-in themes
```

All components update instantly — themes are reactive.

---

## How It Works

### The Problem with Traditional TUIs

Most TUI frameworks use a **render loop**:

```
while (running) {
  processInput()
  updateState()
  render()        // ← Full repaint every frame
  sleep(16ms)     // ← Fixed 60 FPS, wastes CPU when idle
}
```

This burns CPU even when nothing changes, and caps your latency at the frame interval.

### The SparkTUI Way

SparkTUI uses **pure reactivity**:

```
User changes signal
  → Value written to SharedArrayBuffer (0.4ns)
    → FFI wakes Rust engine (10ns)
      → Layout computed if needed (4μs)
        → Framebuffer built (144μs)
          → Only changed cells rendered (400μs)
```

- **0% CPU when idle** — engine thread is parked, not spinning
- **Instant wake** — FFI call, not polling
- **Smart skip** — visual-only changes skip layout
- **Diff rendering** — only changed cells written to terminal

---

## Comparison

### vs Ink (React for CLI)

Ink is great for simple CLIs. SparkTUI is for when you need:
- Sub-millisecond updates
- Complex layouts (nested flex, scroll)
- Zero idle CPU
- Native performance

### vs Ratatui (Rust)

Ratatui is blazing fast. SparkTUI gives you:
- TypeScript/JavaScript API
- No borrow checker
- Familiar reactive patterns
- Same Taffy layout engine

### vs Blessed/Neo-blessed

Blessed is abandoned and slow. SparkTUI offers:
- Active development
- Modern reactive architecture
- 1000x better performance
- Type safety

---

## Roadmap

- [x] Core primitives (box, text, input)
- [x] Flexbox layout (Taffy)
- [x] Grid layout (Taffy)
- [x] Reactive signals
- [x] Theme system
- [x] Keyboard/mouse events
- [x] Scroll with auto-focus
- [x] Animation (cycle)
- [ ] Tables
- [ ] Select/dropdown
- [ ] Tabs
- [ ] Modal/dialog
- [ ] npm package
- [ ] Node.js support
- [ ] Documentation site

---

## Contributing

SparkTUI is in active development. We'd love your help!

```bash
git clone https://github.com/RLabs-Inc/spark-tui
cd spark-tui
bun install
cd rust && cargo build --release && cd ..
bun run examples/counter.ts
```

---

## Philosophy

> "All the power of Rust without borrowing a single thing."

SparkTUI exists because we believe:

1. **TypeScript developers deserve native performance** — without learning Rust
2. **Render loops are wasteful** — reactive propagation is the answer
3. **TUIs should feel instant** — 50μs, not 16ms
4. **Complexity belongs in the framework** — not your code

---

## License

MIT

---

<p align="center">
  <b>SparkTUI</b> — Rust speed. TypeScript vibes.
</p>
