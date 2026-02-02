# Reactivity in SparkTUI

> SparkTUI uses pure reactive propagation. No render loops, no polling, no fixed FPS.

## Overview

SparkTUI's reactivity is powered by `@rlabs-inc/signals`, a fine-grained reactive system. When you change a signal, the change propagates through a dependency graph and updates only what needs updating.

```
Developer changes a signal
  -> Value written to SharedArrayBuffer
    -> Rust engine notified (via FFI, not polling)
      -> Layout computed (if needed)
        -> Framebuffer updated
          -> Terminal renders changed cells
```

This is fundamentally different from render-loop frameworks. There's no "tick" or "frame". Changes propagate instantly.

## Import

```ts
import { signal, derived, effect, effectScope } from '@rlabs-inc/signals'
```

## Signals

A signal is a reactive container for a value. When you read or write it, the system tracks dependencies.

### Creating Signals

```ts
import { signal } from '@rlabs-inc/signals'

// Primitive values
const count = signal(0)
const name = signal('')
const isLoading = signal(false)

// Objects and arrays
const user = signal({ name: 'Alice', age: 30 })
const items = signal<string[]>([])
```

### Reading Signals

Read a signal's value with `.value`:

```ts
const count = signal(42)

console.log(count.value)  // 42
```

When you read `.value` inside a reactive context (derived, effect, or component prop), a dependency is established.

### Writing Signals

Write to a signal with `.value =`:

```ts
const count = signal(0)

count.value = 1       // Simple assignment
count.value++         // Increment
count.value = count.value + 10  // Computed update
```

Each write triggers an update to all dependents.

### Signals in Components

Pass signals directly to component props:

```ts
import { signal } from '@rlabs-inc/signals'
import { box, text } from 'spark-tui/primitives'

const width = signal(40)
const message = signal('Hello')

box({
  width: width,  // Reactive! Changes when width.value changes
  children: () => {
    text({ content: message })  // Also reactive
  },
})

// Later...
width.value = 60   // Box width updates instantly
message.value = 'Goodbye'  // Text updates instantly
```

## Derived (Computed Values)

A derived signal computes its value from other signals. It re-computes only when its dependencies change.

### Creating Derived Signals

```ts
import { signal, derived } from '@rlabs-inc/signals'

const firstName = signal('John')
const lastName = signal('Doe')

// Derived from two signals
const fullName = derived(() => `${firstName.value} ${lastName.value}`)

console.log(fullName.value)  // "John Doe"

firstName.value = 'Jane'
console.log(fullName.value)  // "Jane Doe"
```

### Derived in Components

Derived signals work exactly like regular signals in components:

```ts
const count = signal(0)
const doubled = derived(() => count.value * 2)
const isEven = derived(() => count.value % 2 === 0)

box({
  bg: derived(() => isEven.value
    ? { r: 0, g: 100, b: 0, a: 255 }
    : { r: 100, g: 0, b: 0, a: 255 }
  ),
  children: () => {
    text({ content: () => `Count: ${count.value}` })
    text({ content: () => `Doubled: ${doubled.value}` })
  },
})
```

### Inline Derived (Getters)

For simple computations, use an inline getter function instead of `derived()`:

```ts
const count = signal(0)

text({
  // Inline getter - recomputes when count changes
  content: () => `Count: ${count.value}`,
})

box({
  // Inline getter for computed dimensions
  height: () => Math.max(5, count.value),
})
```

The difference: `derived()` caches the result and only recomputes when dependencies change. Inline getters recompute on every read. For simple expressions, the difference is negligible.

## Effects

An effect runs a function when its dependencies change. Use effects for side effects like logging, API calls, or DOM manipulation.

### Basic Effects

```ts
import { signal, effect } from '@rlabs-inc/signals'

const count = signal(0)

// Effect runs immediately, then again when count changes
effect(() => {
  console.log('Count is now:', count.value)
})

count.value = 1  // Logs: "Count is now: 1"
count.value = 2  // Logs: "Count is now: 2"
```

### Effects in SparkTUI

In SparkTUI, you rarely need manual effects because component props are reactive. However, effects are useful for:

1. **Logging/debugging:**
   ```ts
   effect(() => {
     console.log('User changed:', user.value)
   })
   ```

2. **Syncing with external systems:**
   ```ts
   effect(() => {
     localStorage.setItem('count', String(count.value))
   })
   ```

3. **Conditional side effects:**
   ```ts
   effect(() => {
     if (isError.value) {
       playErrorSound()
     }
   })
   ```

### Effect Cleanup

Return a cleanup function from an effect to run when it re-runs or disposes:

```ts
effect(() => {
  const timer = setInterval(() => {
    console.log('Tick')
  }, 1000)

  // Cleanup when effect re-runs or is disposed
  return () => clearInterval(timer)
})
```

## Effect Scopes

Effect scopes group effects and provide automatic cleanup. SparkTUI's `scoped()` function uses this internally.

### Using scoped()

The `scoped()` function creates a cleanup boundary for your components:

```ts
import { scoped, onCleanup } from 'spark-tui/primitives'

function MyComponent() {
  return scoped(() => {
    const count = signal(0)

    // Start a timer
    const timer = setInterval(() => count.value++, 1000)

    // Register cleanup
    onCleanup(() => clearInterval(timer))

    // UI
    box({
      children: () => text({ content: () => `${count.value}s` }),
    })
  })
}
```

When the component is unmounted, all cleanups run automatically.

### onCleanup

Register cleanup functions within a scope:

```ts
import { onCleanup } from 'spark-tui/primitives'

scoped(() => {
  // WebSocket connection
  const ws = new WebSocket('ws://example.com')
  onCleanup(() => ws.close())

  // Timer
  const timer = setInterval(() => tick(), 100)
  onCleanup(() => clearInterval(timer))

  // Event listener
  process.on('SIGINT', handleExit)
  onCleanup(() => process.off('SIGINT', handleExit))
})
```

## Reactive Props (Reactive<T>)

SparkTUI components accept `Reactive<T>` props, which can be:

1. **Static value:** `42`, `'hello'`, `{ r: 255, g: 0, b: 0, a: 255 }`
2. **Signal:** `mySignal`
3. **Derived:** `myDerived`
4. **Getter:** `() => count.value * 2`

```ts
const width = signal(40)
const color = derived(() => ({ r: width.value, g: 100, b: 200, a: 255 }))

box({
  width: width,                    // Signal
  height: 10,                      // Static
  fg: color,                       // Derived
  opacity: () => width.value / 100, // Getter
})
```

The component system uses `repeat()` internally to wire reactive props to the SharedArrayBuffer.

## How It Works: The Reactive Pipeline

SparkTUI's architecture is purely reactive. Here's how a signal change propagates:

### 1. Signal Write

```ts
count.value = 42
```

The value is written to TypeScript memory. The signals library marks all dependents as dirty.

### 2. Reactive Propagation

Through `repeat()`, the new value is written to the SharedArrayBuffer at the component's index. This is zero-copy - TypeScript and Rust share the same memory.

### 3. Rust Notification

The FFI wake function notifies the Rust engine. This is a direct function call, not polling.

### 4. Layout (Conditional)

If a layout-affecting property changed (width, height, padding, etc.), Taffy computes new positions. If only a visual property changed (color, text content), layout is skipped.

### 5. Framebuffer

The Rust engine reads the SharedArrayBuffer and builds a 2D cell grid.

### 6. Render

Only changed cells are written to the terminal via ANSI escape codes.

**Total latency: ~50 microseconds** for a typical update.

## Best Practices

### 1. Prefer Signals Over Effects

Instead of:
```ts
// Avoid: imperative updates
effect(() => {
  if (isLoading.value) {
    setStatusText('Loading...')
  }
})
```

Use:
```ts
// Better: reactive derivation
const statusText = derived(() =>
  isLoading.value ? 'Loading...' : 'Ready'
)
text({ content: statusText })
```

### 2. Keep Derivations Pure

Derived signals should be pure computations without side effects:

```ts
// Good: pure computation
const fullName = derived(() => `${first.value} ${last.value}`)

// Bad: side effect in derived
const fullName = derived(() => {
  console.log('Computing name')  // Side effect!
  return `${first.value} ${last.value}`
})
```

### 3. Use Inline Getters for Simple Expressions

```ts
// Overkill for simple expressions
const doubled = derived(() => count.value * 2)
text({ content: doubled })

// Better: inline getter
text({ content: () => count.value * 2 })
```

### 4. Batch Updates When Possible

Each signal write triggers propagation. For multiple related updates, batch them:

```ts
// Multiple updates, multiple propagations
firstName.value = 'John'
lastName.value = 'Doe'
age.value = 30

// Better: single object signal
const user = signal({ firstName: 'John', lastName: 'Doe', age: 30 })
user.value = { ...user.value, firstName: 'Jane', age: 31 }
```

### 5. Clean Up Resources

Always clean up timers, WebSockets, and other resources:

```ts
scoped(() => {
  const timer = setInterval(() => tick(), 1000)
  onCleanup(() => clearInterval(timer))

  // Component UI...
})
```

## Common Patterns

### Toggle State

```ts
const isOpen = signal(false)

box({
  onClick: () => isOpen.value = !isOpen.value,
  children: () => {
    text({ content: () => isOpen.value ? 'Close' : 'Open' })
  },
})
```

### Form State

```ts
const email = signal('')
const password = signal('')
const isValid = derived(() =>
  email.value.includes('@') && password.value.length >= 8
)

box({
  children: () => {
    input({ value: email, placeholder: 'Email' })
    input({ value: password, placeholder: 'Password', password: true })
    box({
      visible: isValid,
      children: () => text({ content: 'Submit' }),
    })
  },
})
```

### Loading State

```ts
const isLoading = signal(false)
const data = signal<Data | null>(null)
const error = signal<string | null>(null)

async function fetchData() {
  isLoading.value = true
  error.value = null
  try {
    data.value = await api.getData()
  } catch (e) {
    error.value = e.message
  } finally {
    isLoading.value = false
  }
}

show(
  () => isLoading.value,
  () => text({ content: cycle(Frames.spinner, { fps: 10 }) })
)

show(
  () => error.value !== null,
  () => text({ content: () => `Error: ${error.value}`, fg: { r: 255, g: 0, b: 0, a: 255 } })
)

show(
  () => data.value !== null && !isLoading.value,
  () => text({ content: () => JSON.stringify(data.value) })
)
```

### Animated Values

```ts
import { cycle, pulse } from 'spark-tui/primitives'

// Spinner
text({ content: cycle(['/', '-', '\\', '|'], { fps: 8 }) })

// Blinking cursor
const cursorVisible = pulse({ fps: 2 })
text({ content: () => cursorVisible.value ? '_' : ' ' })

// Color animation
const colors = [
  { r: 255, g: 0, b: 0, a: 255 },
  { r: 0, g: 255, b: 0, a: 255 },
  { r: 0, g: 0, b: 255, a: 255 },
]
text({ content: 'Rainbow!', fg: cycle(colors, { fps: 2 }) })
```

## Debugging

### Trace Signal Reads

```ts
effect(() => {
  console.log('Rendering with:', {
    count: count.value,
    name: name.value,
  })
})
```

### Track Update Frequency

```ts
let updateCount = 0
effect(() => {
  const _ = mySignal.value  // Track this signal
  console.log(`Update #${++updateCount}`)
})
```

### Performance Timing

```ts
effect(() => {
  const start = performance.now()
  const _ = expensiveDerived.value
  const elapsed = performance.now() - start
  console.log(`Computation took ${elapsed.toFixed(2)}ms`)
})
```

## See Also

- [Getting Started](../getting-started.md) - Installation and first app
- [Architecture Overview](./architecture.md) - How SparkTUI works under the hood
- [Components Reference](../components/) - Full API for box, text, input
- [Control Flow](../control-flow/) - show, each, when primitives
