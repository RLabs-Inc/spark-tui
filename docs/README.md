# SparkTUI Documentation

> A hybrid TUI framework with pure reactive propagation — all Rust benefits without borrowing a single thing.

## Quick Start

```ts
import { mount, box, text, signal } from 'spark-tui';

await mount(() => {
  const count = signal(0);

  box({ border: 1, padding: 1, children: () => {
    text({ content: () => `Count: ${count.value}` });
  }});
});
```

## Documentation

### Getting Started

- **[Getting Started](./getting-started.md)** — Installation, first app, core concepts

### Concepts

- **[Reactivity](./concepts/reactivity.md)** — Signals, derived values, effects, and the reactive pipeline

### Components

- **[box](./components/box.md)** — Container with flexbox/grid layout, borders, and interaction
- **[text](./components/text.md)** — Text display with styling, alignment, and wrapping
- **[input](./components/input.md)** — Single-line text input with cursor and validation

### Control Flow

- **[each](./control-flow/each.md)** — Reactive list rendering with fine-grained updates
- **[show](./control-flow/show.md)** — Conditional rendering based on boolean state
- **[when](./control-flow/when.md)** — Async rendering with promise state handling

### Animation

- **[cycle & pulse](./animation/cycle-pulse.md)** — Frame-based animations and boolean toggles

### Lifecycle

- **[Scoping](./lifecycle/scoping.md)** — Cleanup collection, lifecycle hooks, resource management

### Theming

- **[Themes](./theming/themes.md)** — Built-in themes, custom themes, reactive styling

### Events

- **[Keyboard](./events/keyboard.md)** — Key events, helpers, constants, focus management
- **[Mouse](./events/mouse.md)** — Click, scroll, hover, drag patterns

### API Reference

- **[mount](./api-reference/mount.md)** — Application mounting, modes, and configuration
- **[Types](./api-reference/types.md)** — RGBA, Dimension, BorderStyle, Attr, Grid types

## Architecture

SparkTUI uses **pure reactive propagation** — no loops, no polling, no fixed FPS:

```
Signal changes → Rust notified → Layout (if needed) → Framebuffer → Terminal
```

The entire pipeline is driven by a single effect that fires when data changes. Think of it like a spreadsheet: change a cell, dependent cells update automatically.

## Package Imports

```ts
// Main package (re-exports everything)
import { box, text, input, mount, signal } from 'spark-tui';

// Specific imports
import { box, text, input, each, show, when } from 'spark-tui/primitives';
import { mount, mountSync, isEnter, hasCtrl } from 'spark-tui/engine';
import { setTheme, theme, t } from 'spark-tui/theme';
import { RGBA, BorderStyle, Attr } from 'spark-tui/types';

// Signals (external package)
import { signal, derived, effect } from '@rlabs-inc/signals';
```

## Examples

See the `/examples` directory for working demos:

| Example | Description |
|---------|-------------|
| `counter.ts` | Simple reactive counter |
| `demo-form.ts` | Form with input validation |
| `demo-dashboard.ts` | Complex multi-panel layout |
| `demo-spinners.ts` | Animation examples |
| `demo-themes.ts` | Theme switching |
| `demo-scroll.ts` | Scrollable containers |
| `demo-mouse.ts` | Mouse interaction patterns |
| `api-showcase.ts` | Comprehensive API usage |

Run examples with:

```bash
bun run examples/counter.ts
```

## License

MIT
