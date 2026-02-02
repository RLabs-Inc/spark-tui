# Animation Primitives

> Reactive animation utilities for spinners, cursors, progress indicators, and color cycling.

SparkTUI's animation primitives are **signal sources**, not render loops. They use `setInterval` to periodically update a signal value, which then propagates reactively through the system. There is no fixed FPS rendering - the Rust engine renders when data changes.

## Import

```ts
import { cycle, pulse, Frames } from 'spark-tui';
```

## Functions

### cycle

Create a signal that cycles through an array of values at a given FPS.

```ts
function cycle<T>(
  frames: readonly T[],
  options?: CycleOptions<T>
): WritableSignal<T>
```

#### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `frames` | `readonly T[]` | Array of values to cycle through. Must have at least one element. |
| `options` | `CycleOptions<T>` | Optional configuration (see below). |

#### CycleOptions

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `fps` | `number` | `10` | Frames per second - how often the signal updates. |
| `active` | `boolean \| ReadableSignal<boolean> \| (() => boolean)` | `true` | Whether animation is active. Can be reactive for dynamic control. |
| `startIndex` | `number` | `0` | Starting index in the frames array. |
| `autoStart` | `boolean` | `true` | Start immediately or wait for manual start. |

#### Returns

A `WritableSignal<T>` containing the current frame value. The signal updates at the specified FPS rate.

#### Examples

```ts
// Spinner animation at 12 FPS
text({ content: cycle(Frames.spinner, { fps: 12 }) })

// Color cycling
const rainbow = [red, orange, yellow, green, blue, purple]
box({ bg: cycle(rainbow, { fps: 2 }) })

// Conditional animation - pauses when isLoading is false
const isLoading = signal(true)
text({ content: cycle(Frames.spinner, { fps: 12, active: isLoading }) })

// Start at specific frame
text({ content: cycle(Frames.spinner, { fps: 10, startIndex: 5 }) })

// Custom loading message
text({ content: cycle(['Loading.', 'Loading..', 'Loading...'], { fps: 2 }) })
```

---

### pulse

Create a boolean signal that toggles between `true` and `false` (blink effect).

```ts
function pulse(options?: PulseOptions): WritableSignal<boolean>
```

#### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `options` | `PulseOptions` | Optional configuration (see below). |

#### PulseOptions

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `fps` | `number` | `2` | Blink rate. At 2 FPS, the signal toggles every 500ms. |
| `onDuration` | `number` | - | Duration in ms for the 'on' (true) state. If set, uses custom timing instead of even split. |
| `active` | `boolean \| ReadableSignal<boolean> \| (() => boolean)` | `true` | Whether pulse is active. Can be reactive. |
| `autoStart` | `boolean` | `true` | Start immediately or wait for manual start. |

#### Returns

A `WritableSignal<boolean>` that toggles between `true` and `false`.

#### Examples

```ts
// Standard cursor blink at 2 FPS (500ms on, 500ms off)
input({ cursor: { visible: pulse({ fps: 2 }) } })

// Faster blink (4 FPS = 250ms cycle)
input({ cursor: { visible: pulse({ fps: 4 }) } })

// Custom timing: 300ms on, 700ms off (at 1 FPS = 1000ms period)
input({ cursor: { visible: pulse({ fps: 1, onDuration: 300 }) } })

// Conditional pulse - stops when not loading
const isLoading = signal(true)
text({
  content: () => pulse({ fps: 2, active: isLoading }).value ? '*' : ' '
})
```

---

## Built-in Frame Sets

The `Frames` object provides pre-defined animation frames for common UI patterns.

### Frames.spinner

Classic braille dots spinner. Best at 10-12 FPS.

```ts
['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏']
```

```ts
text({ content: cycle(Frames.spinner, { fps: 12 }) })
```

### Frames.dots

Braille dots with vertical pattern. Best at 8-10 FPS.

```ts
['⣾', '⣽', '⣻', '⢿', '⡿', '⣟', '⣯', '⣷']
```

```ts
text({ content: cycle(Frames.dots, { fps: 10 }) })
```

### Frames.line

Simple ASCII line spinner. Works in any terminal. Best at 4-8 FPS.

```ts
['-', '\\', '|', '/']
```

```ts
text({ content: cycle(Frames.line, { fps: 6 }) })
```

### Frames.bar

Growing bar from thin to thick. Good for loading progress. Best at 4-8 FPS.

```ts
['▏', '▎', '▍', '▌', '▋', '▊', '▉', '█']
```

```ts
text({ content: cycle(Frames.bar, { fps: 6 }) })
```

### Frames.clock

Clock face emoji animation. Best at 1 FPS for real-time feel.

```ts
['🕐', '🕑', '🕒', '🕓', '🕔', '🕕', '🕖', '🕗', '🕘', '🕙', '🕚', '🕛']
```

```ts
text({ content: cycle(Frames.clock, { fps: 1 }) })
```

### Frames.bounce

Bouncing dot effect. Best at 4-6 FPS.

```ts
['⠁', '⠂', '⠄', '⠂']
```

```ts
text({ content: cycle(Frames.bounce, { fps: 5 }) })
```

### Frames.arrow

Rotating arrow. Best at 4-8 FPS.

```ts
['←', '↖', '↑', '↗', '→', '↘', '↓', '↙']
```

```ts
text({ content: cycle(Frames.arrow, { fps: 6 }) })
```

### Frames.pulse

Pulsing circle. Best at 4-6 FPS.

```ts
['◯', '◔', '◑', '◕', '●', '◕', '◑', '◔']
```

```ts
text({ content: cycle(Frames.pulse, { fps: 5 }) })
```

---

## Custom Frame Arrays

Create your own frame arrays for custom animations.

### Text-based Frames

```ts
// Custom loading text
const loadingFrames = ['Loading.', 'Loading..', 'Loading...']
text({ content: cycle(loadingFrames, { fps: 2 }) })

// Progress indicator
const progressFrames = ['[    ]', '[=   ]', '[==  ]', '[=== ]', '[====]']
text({ content: cycle(progressFrames, { fps: 4 }) })

// Wave effect
const waveFrames = ['~', '~~', '~~~', '~~~~', '~~~', '~~', '~']
text({ content: cycle(waveFrames, { fps: 3 }) })
```

### Color Cycling

```ts
import { packColor } from 'spark-tui';

// Rainbow colors
const rainbow = [
  packColor(255, 100, 100, 255), // red
  packColor(255, 180, 100, 255), // orange
  packColor(255, 255, 100, 255), // yellow
  packColor(100, 255, 140, 255), // green
  packColor(100, 200, 255, 255), // blue
  packColor(180, 140, 255, 255), // purple
]

box({ bg: cycle(rainbow, { fps: 2 }) })

// Status colors
const statusColors = [colors.success, colors.warning, colors.error]
text({ fg: cycle(statusColors, { fps: 1 }) })
```

### Indicator Patterns

```ts
// Online/offline indicator
text({
  content: cycle(['●', '○'], { fps: 1 }),
  fg: colors.online,
})

// Recording indicator (fast blink)
text({
  content: cycle(['●', ' '], { fps: 2 }),
  fg: colors.recording,
})
```

---

## Examples

### Basic Spinner with Label

```ts
import { mount } from 'spark-tui';
import { box, text, cycle, Frames } from 'spark-tui/primitives';

await mount(() => {
  box({
    flexDirection: 'row',
    alignItems: 'center',
    gap: 1,
    children: () => {
      text({ content: cycle(Frames.spinner, { fps: 12 }) })
      text({ content: 'Loading...' })
    },
  })
})
```

### Multiple Spinners Showcase

```ts
import { mount } from 'spark-tui';
import { box, text, cycle, Frames } from 'spark-tui/primitives';

await mount(() => {
  box({
    flexDirection: 'row',
    gap: 3,
    children: () => {
      // All built-in spinners side by side
      const spinnerTypes = [
        ['spinner', Frames.spinner],
        ['dots', Frames.dots],
        ['line', Frames.line],
        ['bar', Frames.bar],
        ['bounce', Frames.bounce],
        ['arrow', Frames.arrow],
        ['pulse', Frames.pulse],
      ] as const

      for (const [name, frames] of spinnerTypes) {
        box({
          flexDirection: 'column',
          alignItems: 'center',
          children: () => {
            text({ content: cycle(frames, { fps: 10 }) })
            text({ content: name })
          },
        })
      }
    },
  })
})
```

### FPS Comparison

```ts
import { mount } from 'spark-tui';
import { box, text, cycle, Frames } from 'spark-tui/primitives';

await mount(() => {
  box({
    flexDirection: 'row',
    gap: 4,
    children: () => {
      for (const fps of [2, 4, 8, 12, 24]) {
        box({
          flexDirection: 'column',
          alignItems: 'center',
          children: () => {
            text({ content: cycle(Frames.spinner, { fps }) })
            text({ content: `${fps} fps` })
          },
        })
      }
    },
  })
})
```

### Conditional Animation

```ts
import { signal } from '@rlabs-inc/signals';
import { mount } from 'spark-tui';
import { box, text, cycle, Frames } from 'spark-tui/primitives';

const isLoading = signal(true)

await mount(() => {
  box({
    flexDirection: 'column',
    gap: 1,
    children: () => {
      box({
        flexDirection: 'row',
        gap: 1,
        children: () => {
          // Animation pauses when isLoading is false
          text({ content: cycle(Frames.spinner, { fps: 12, active: isLoading }) })
          text({ content: () => isLoading.value ? 'Loading...' : 'Done!' })
        },
      })
    },
    onKey: (e) => {
      if (String.fromCharCode(e.keycode) === ' ') {
        isLoading.value = !isLoading.value
        return true
      }
    },
    focusable: true,
  })
})

// Toggle with: isLoading.value = false
```

### Combined Animations

```ts
import { mount } from 'spark-tui';
import { box, text, cycle, Frames } from 'spark-tui/primitives';
import { packColor } from 'spark-tui';

const rainbow = [
  packColor(255, 100, 100, 255),
  packColor(255, 200, 100, 255),
  packColor(100, 255, 140, 255),
  packColor(100, 200, 255, 255),
  packColor(180, 140, 255, 255),
]

await mount(() => {
  // Spinner with cycling background color
  const spinnerSig = cycle(Frames.spinner, { fps: 12 })
  const bgSig = cycle(rainbow, { fps: 2 })

  box({
    width: 20,
    height: 3,
    bg: bgSig,
    alignItems: 'center',
    justifyContent: 'center',
    border: 1,
    children: () => {
      text({
        content: () => ` ${spinnerSig.value} Processing `,
      })
    },
  })
})
```

---

## How It Works

Animation primitives are **reactive signal sources**:

1. `setInterval` updates a signal value at the specified FPS
2. The signal change propagates through the reactive system
3. Components using the signal re-render with the new value
4. The Rust engine renders when data changes

This is different from traditional animation loops:
- **No render loop** - Rendering happens reactively
- **No fixed FPS rendering** - Only renders when data changes
- **Shared clocks** - Animations at the same FPS share a single `setInterval`
- **Automatic cleanup** - Animations clean up when their scope is disposed

### Clock Sharing

Multiple animations at the same FPS share a single timer:

```ts
// These three animations share ONE setInterval (all at 10 FPS)
text({ content: cycle(Frames.spinner, { fps: 10 }) })
text({ content: cycle(Frames.dots, { fps: 10 }) })
text({ content: cycle(Frames.bounce, { fps: 10 }) })

// This uses a separate timer (different FPS)
text({ content: cycle(Frames.clock, { fps: 1 }) })
```

### Automatic Cleanup

When created inside a `scoped()` block, animations automatically clean up:

```ts
import { scoped, onCleanup } from 'spark-tui';

function LoadingIndicator() {
  return scoped(() => {
    // Animation automatically cleans up when scope is disposed
    text({ content: cycle(Frames.spinner, { fps: 12 }) })
  })
}

const cleanup = LoadingIndicator()
// Later...
cleanup() // Animation stops, timer is released
```

---

## See Also

- [Lifecycle and Scoping](../lifecycle/scoping.md) - How animations clean up automatically
- [Reactivity Concepts](../concepts/reactivity.md) - How signals and deriveds work
- [Text Component](../components/text.md) - Using animations with text
