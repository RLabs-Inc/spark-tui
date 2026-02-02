# Getting Started with SparkTUI

> Build blazing-fast terminal UIs in TypeScript, powered by a Rust engine.

SparkTUI is a hybrid TUI framework that combines the developer experience of TypeScript with the performance of Rust. You write familiar TypeScript code with reactive signals, and a Rust engine handles layout and rendering at native speed.

## Installation

```bash
# Install from npm (requires Bun 1.0+)
bun add spark-tui @rlabs-inc/signals
```

**Requirements:**
- Bun 1.0 or later (Node.js support planned)
- macOS, Linux, or Windows

## Your First App

Create a file called `hello.ts`:

```ts
import { signal } from '@rlabs-inc/signals'
import { mount } from 'spark-tui'
import { box, text } from 'spark-tui/primitives'

await mount(() => {
  box({
    padding: 2,
    border: 1,
    children: () => {
      text({ content: 'Hello, SparkTUI!' })
    },
  })
})
```

Run it:

```bash
bun run hello.ts
```

You should see a bordered box with "Hello, SparkTUI!" displayed in your terminal. Press `Ctrl+C` or `q` to exit.

## Adding Reactivity

SparkTUI shines when you add reactive state. Change a signal, and the UI updates instantly.

```ts
import { signal, derived } from '@rlabs-inc/signals'
import { mount } from 'spark-tui'
import { box, text } from 'spark-tui/primitives'

// Reactive state
const count = signal(0)

await mount(() => {
  box({
    flexDirection: 'column',
    padding: 2,
    border: 1,
    gap: 1,
    children: () => {
      // Static text
      text({ content: 'Counter Demo' })

      // Reactive text - updates when count changes
      text({ content: () => `Count: ${count.value}` })

      // Buttons row
      box({
        flexDirection: 'row',
        gap: 2,
        children: () => {
          // Decrement button
          box({
            padding: 1,
            border: 1,
            focusable: true,
            onClick: () => count.value--,
            children: () => text({ content: ' - ' }),
          })

          // Increment button
          box({
            padding: 1,
            border: 1,
            focusable: true,
            onClick: () => count.value++,
            children: () => text({ content: ' + ' }),
          })
        },
      })
    },

    // Global keyboard handler
    onKey: (key) => {
      const char = String.fromCharCode(key.keycode)
      if (char === '+' || char === '=') {
        count.value++
        return true
      }
      if (char === '-') {
        count.value--
        return true
      }
    },
  })
})
```

Now you can:
- Click the `+` and `-` buttons with your mouse
- Press `+` or `-` on your keyboard
- Watch the count update instantly

## Understanding the Mount API

The `mount()` function is the entry point for SparkTUI applications. It handles:

- Bridge initialization (SharedArrayBuffer between TypeScript and Rust)
- Rust engine loading
- Event listener setup
- Terminal size detection
- Clean shutdown on exit

### Basic Usage

```ts
import { mount } from 'spark-tui'
import { box, text } from 'spark-tui/primitives'

// mount() returns a Promise that resolves when the app exits
await mount(() => {
  box({
    children: () => {
      text({ content: 'Hello!' })
    },
  })
})

console.log('App exited')
```

### Mount Options

```ts
await mount(() => {
  // Your app
}, {
  // Render mode: 'fullscreen' | 'inline' | 'append'
  mode: 'fullscreen',

  // Terminal dimensions (auto-detected if not specified)
  width: 80,
  height: 24,

  // Disable default behaviors
  disableCtrlC: false,        // Allow Ctrl+C to exit
  disableTabNavigation: false, // Enable Tab focus navigation
  disableMouse: false,         // Enable mouse support

  // Buffer configuration for large apps
  maxNodes: 10000,       // Maximum UI components
  textPoolSize: 10485760, // Text pool size in bytes (10MB)
})
```

### Render Modes

- **`fullscreen`** (default): Clears the screen and uses the alternate terminal buffer. Best for full-screen applications.
- **`inline`**: Renders within the normal terminal flow. Good for CLI tools that output above/below your UI.
- **`append`**: Appends output without clearing. Useful for streaming or progressive rendering.

### Synchronous Mount

For tests or when you need manual control, use `mountSync()`:

```ts
import { mountSync } from 'spark-tui'

const app = mountSync(() => {
  box({
    children: () => text({ content: 'Testing!' }),
  })
})

// Access internals
console.log(app.buffer)  // SharedBuffer instance
console.log(app.engine)  // Rust engine handle

// Manual unmount
app.unmount()
```

## Core Primitives

SparkTUI provides three core primitives:

### box - Container Component

Flexbox/Grid container with borders, backgrounds, and children:

```ts
box({
  // Dimensions
  width: 40,
  height: 10,
  minWidth: 20,
  maxWidth: '100%',

  // Flexbox layout
  flexDirection: 'column',  // 'row' | 'column' | 'row-reverse' | 'column-reverse'
  justifyContent: 'center', // 'flex-start' | 'center' | 'flex-end' | 'space-between' | 'space-around' | 'space-evenly'
  alignItems: 'center',     // 'stretch' | 'flex-start' | 'center' | 'flex-end' | 'baseline'
  gap: 1,

  // Spacing
  padding: 2,
  margin: 1,

  // Visual - colors accept flexible formats!
  border: 1,              // 0=none, 1=single, 2=double, 3=rounded, etc.
  borderColor: '#6496c8', // Hex string
  bg: 'rgb(30, 30, 40)',  // RGB string
  fg: 'white',            // CSS name

  // Interaction
  focusable: true,
  tabIndex: 0,
  onClick: (event) => console.log('Clicked!'),
  onKey: (event) => console.log('Key pressed:', event.keycode),

  children: () => {
    // Child components here
  },
})
```

#### Flexible Color Formats

All color props (`fg`, `bg`, `borderColor`) accept multiple formats:

```ts
// All of these work for any color prop:
box({ bg: '#ff0000' })               // Hex (6-digit)
box({ bg: '#f00' })                  // Hex (3-digit shorthand)
box({ bg: 'red' })                   // CSS named color
box({ bg: 'dodgerblue' })            // CSS named color
box({ bg: 'rgb(255, 0, 0)' })        // RGB string
box({ bg: 'rgba(255, 0, 0, 0.5)' })  // RGBA with alpha
box({ bg: 'hsl(0, 100%, 50%)' })     // HSL string
box({ bg: 'oklch(0.7 0.15 30)' })    // OKLCH (perceptually uniform!)
box({ bg: 0xff0000 })                // Integer (0xRRGGBB)
box({ bg: { r: 255, g: 0, b: 0, a: 255 } }) // RGBA object
box({ bg: null })                    // Terminal default
```

### text - Text Display

Display text with styling and alignment:

```ts
text({
  content: 'Hello, World!',  // Static string
  // or
  content: () => `Count: ${count.value}`,  // Reactive

  // Styling - use any color format!
  fg: 'coral',         // CSS named color
  bg: '#000',          // Hex shorthand

  // Layout
  align: 'left',    // 'left' | 'center' | 'right'
  wrap: 'wrap',     // 'wrap' | 'nowrap' | 'truncate'
})
```

### input - Text Input

Single-line text input with cursor and editing:

```ts
import { signal } from '@rlabs-inc/signals'

const name = signal('')

input({
  value: name,                    // Two-way bound signal
  placeholder: 'Enter your name',
  maxLength: 50,
  password: false,                // true for password masking

  // Cursor configuration
  cursor: {
    style: 'block',  // 'block' | 'bar' | 'underline'
    blink: true,     // Enable cursor blinking
  },

  // Events
  onChange: (value) => console.log('Value:', value),
  onSubmit: (value) => console.log('Submitted:', value),
  onFocus: () => console.log('Focused'),
  onBlur: () => console.log('Blurred'),
})
```

## Reactive Props

All component props can be reactive. Pass a signal, derived, or getter function:

```ts
import { signal, derived } from '@rlabs-inc/signals'

const width = signal(40)
const isVisible = signal(true)
const color = derived(() => isVisible.value
  ? { r: 0, g: 255, b: 0, a: 255 }
  : { r: 255, g: 0, b: 0, a: 255 }
)

box({
  width: width,                     // Signal directly
  visible: isVisible,               // Signal directly
  fg: color,                        // Derived signal
  height: () => width.value / 2,    // Getter function
  children: () => {
    text({ content: () => `Width: ${width.value}` })
  },
})
```

When the signal changes, only the affected properties update. No full re-renders.

## Control Flow

SparkTUI provides reactive control flow primitives:

### show - Conditional Rendering

```ts
import { signal } from '@rlabs-inc/signals'
import { show } from 'spark-tui/primitives'

const isLoggedIn = signal(false)

show(
  () => isLoggedIn.value,
  () => text({ content: 'Welcome back!' }),
  () => text({ content: 'Please log in.' })  // Optional else
)
```

### each - List Rendering

```ts
import { signal } from '@rlabs-inc/signals'
import { each } from 'spark-tui/primitives'

interface Item {
  id: string
  name: string
}

const items = signal<Item[]>([
  { id: '1', name: 'Apple' },
  { id: '2', name: 'Banana' },
])

each(
  () => items.value,
  (getItem, key) => {
    return text({ content: () => getItem().name })
  },
  { key: (item) => item.id }
)
```

### when - Async State

```ts
import { when } from 'spark-tui/primitives'

when(
  () => fetchData(),
  {
    pending: () => text({ content: 'Loading...' }),
    then: (data) => text({ content: `Got: ${data}` }),
    catch: (error) => text({ content: `Error: ${error.message}` }),
  }
)
```

## Theming

SparkTUI includes 13 built-in themes:

```ts
import { setTheme, t } from 'spark-tui/theme'

// Set the active theme
setTheme('dracula')  // or 'nord', 'catppuccin', 'tokyoNight', 'gruvbox', etc.

// Use theme colors (reactive!)
box({
  border: 1,
  borderColor: t.primary,
  bg: t.background,
  children: () => {
    text({ content: 'Themed!', fg: t.text })
  },
})
```

## Animation

Create animated spinners and indicators:

```ts
import { cycle, pulse, Frames } from 'spark-tui/primitives'

// Spinner animation
text({ content: cycle(Frames.spinner, { fps: 12 }) })

// Blinking indicator
text({
  content: () => pulse({ fps: 2 }).value ? 'ON' : 'OFF'
})

// Custom frames
text({
  content: cycle(['Loading.', 'Loading..', 'Loading...'], { fps: 2 })
})
```

Built-in frame sets: `spinner`, `dots`, `line`, `bar`, `clock`, `bounce`, `arrow`, `pulse`.

## Next Steps

- [Reactivity Concepts](./concepts/reactivity.md) - Deep dive into signals, derived, and effects
- [Components Reference](./components/) - Full API documentation for all primitives
- [Examples](./examples/) - Complete example applications

## See Also

- [Architecture Overview](./concepts/architecture.md)
- [Theming Guide](./theming/)
- [Event Handling](./events/)
