# Lifecycle and Scoping

> Automatic cleanup collection for components, effects, and resources.

SparkTUI provides a scoping system that automatically collects and runs cleanup functions when components are unmounted. This ensures timers, subscriptions, and other resources are properly disposed.

## Import

```ts
import { scoped, onCleanup, trackCleanup } from 'spark-tui';
import { onMount, onDestroy } from 'spark-tui';
```

## Concepts

### The Cleanup Pattern

Every component in SparkTUI returns a `Cleanup` function:

```ts
type Cleanup = () => void
```

When you call this function, the component unmounts and releases its resources. The scoping system automates collecting these cleanups.

### Why Scoping Matters

Without scoping, you must manually track every cleanup:

```ts
// Without scoping - manual cleanup tracking
function Timer() {
  const elapsed = signal(0)
  const interval = setInterval(() => elapsed.value++, 1000)

  const textCleanup = text({ content: () => `${elapsed.value}s` })

  // Must return combined cleanup
  return () => {
    clearInterval(interval)
    textCleanup()
  }
}
```

With scoping, cleanups are collected automatically:

```ts
// With scoping - automatic cleanup collection
function Timer() {
  return scoped(() => {
    const elapsed = signal(0)

    const interval = setInterval(() => elapsed.value++, 1000)
    onCleanup(() => clearInterval(interval))

    text({ content: () => `${elapsed.value}s` })
    // text() cleanup is auto-collected by the scope
  })
}
```

---

## Functions

### scoped

Execute a function with automatic cleanup collection. All primitives (`box`, `text`, `input`) and effects created inside automatically register their cleanups.

```ts
function scoped(fn: () => void): Cleanup
```

#### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `fn` | `() => void` | Function to execute within the scope. |

#### Returns

A `Cleanup` function that, when called:
1. Stops all effects created within the scope
2. Runs all registered cleanup callbacks
3. Disposes all components created within the scope

#### Examples

```ts
// Basic component with scoped cleanup
function Counter() {
  return scoped(() => {
    const count = signal(0)

    box({
      children: () => {
        text({ content: () => `Count: ${count.value}` })
      }
    })

    effect(() => console.log('Count:', count.value))
  })
}

// Use the component
const cleanup = Counter()

// Later, unmount everything
cleanup()
```

---

### onCleanup

Register a cleanup callback with the active scope. Use for timers, subscriptions, event listeners, or any resource that needs disposal.

```ts
function onCleanup(cleanup: Cleanup): void
```

#### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `cleanup` | `Cleanup` | Function to call when scope is disposed. |

#### Notes

- Must be called inside a `scoped()` block
- If called outside a scope, the cleanup is silently ignored
- Cleanups run in the order they were registered
- Cleanup errors are logged but don't stop other cleanups

#### Examples

```ts
function Timer() {
  return scoped(() => {
    const elapsed = signal(0)

    // Timer cleanup
    const interval = setInterval(() => elapsed.value++, 1000)
    onCleanup(() => clearInterval(interval))

    // Event listener cleanup
    const handler = () => console.log('resize')
    process.stdout.on('resize', handler)
    onCleanup(() => process.stdout.off('resize', handler))

    // WebSocket cleanup
    const ws = new WebSocket('ws://example.com')
    onCleanup(() => ws.close())

    text({ content: () => `${elapsed.value}s` })
  })
}
```

---

### trackCleanup

Register a cleanup and return it for chaining. Useful when you need the cleanup function reference.

```ts
function trackCleanup<T extends Cleanup>(cleanup: T): T
```

#### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `cleanup` | `T extends Cleanup` | Cleanup function to track and return. |

#### Returns

The same cleanup function, for chaining.

#### Examples

```ts
function MyComponent() {
  return scoped(() => {
    // Track and store cleanup for manual use
    const stopTimer = trackCleanup(() => clearInterval(interval))
    const interval = setInterval(() => tick(), 1000)

    // Can call stopTimer() manually before scope ends
    box({
      children: () => text({ content: 'Timer' }),
      onKey: (e) => {
        if (e.keycode === 27) { // Escape
          stopTimer() // Stop timer early
          return true
        }
      },
      focusable: true,
    })
  })
}
```

---

### onMount

Register a callback to run after the current component is fully set up. The callback runs synchronously after component setup is complete.

```ts
function onMount(fn: () => void): void
```

#### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `fn` | `() => void` | Callback to run after component is mounted. |

#### Notes

- Must be called during component creation (inside primitive callbacks)
- If called outside component creation, logs a warning
- Multiple `onMount` calls register multiple callbacks
- Callbacks run in registration order

#### Examples

```ts
function DataLoader() {
  return scoped(() => {
    const data = signal<string | null>(null)

    box({
      children: () => {
        // onMount runs after this box is set up
        onMount(() => {
          console.log('Component mounted!')
          fetchData().then(result => data.value = result)
        })

        text({ content: () => data.value ?? 'Loading...' })
      }
    })
  })
}
```

---

### onDestroy

Register a callback to run when the current component is destroyed. Use for cleanup that's tied to a specific component rather than the entire scope.

```ts
function onDestroy(fn: () => void): void
```

#### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `fn` | `() => void` | Cleanup callback to run on component destruction. |

#### Notes

- Must be called during component creation
- If called outside component creation, logs a warning
- Different from `onCleanup`: tied to component, not scope
- Useful when nesting components with different lifetimes

#### Examples

```ts
function Timer() {
  return scoped(() => {
    const elapsed = signal(0)

    box({
      children: () => {
        // Cleanup tied to this specific component
        const interval = setInterval(() => elapsed.value++, 1000)
        onDestroy(() => {
          console.log('Timer component destroyed')
          clearInterval(interval)
        })

        text({ content: () => `${elapsed.value}s` })
      }
    })
  })
}
```

---

## Patterns

### Basic Component Pattern

The recommended pattern for creating components:

```ts
import { scoped, onCleanup } from 'spark-tui';
import { signal } from '@rlabs-inc/signals';
import { box, text } from 'spark-tui/primitives';

function MyComponent(props: { title: string }): Cleanup {
  return scoped(() => {
    const count = signal(0)

    box({
      padding: 1,
      border: 1,
      children: () => {
        text({ content: props.title })
        text({ content: () => `Count: ${count.value}` })
      },
    })
  })
}
```

### Timer Component

```ts
function Timer(): Cleanup {
  return scoped(() => {
    const elapsed = signal(0)

    // Register timer cleanup
    const interval = setInterval(() => elapsed.value++, 1000)
    onCleanup(() => clearInterval(interval))

    box({
      flexDirection: 'row',
      gap: 1,
      children: () => {
        text({ content: 'Elapsed:' })
        text({ content: () => `${elapsed.value}s` })
      },
    })
  })
}
```

### Nested Components

Child component cleanups are tracked by the parent scope:

```ts
function Child(): Cleanup {
  return scoped(() => {
    text({ content: 'I am a child' })
  })
}

function Parent(): Cleanup {
  return scoped(() => {
    box({
      children: () => {
        text({ content: 'Parent' })
        Child() // Child's cleanup is tracked by parent scope
      },
    })
  })
}

const cleanup = Parent()
cleanup() // Cleans up both parent AND child
```

### Conditional Cleanup

For resources that may need early cleanup:

```ts
function ConditionalResource(): Cleanup {
  return scoped(() => {
    const isActive = signal(true)
    let cleanup: Cleanup | null = null

    // Create resource that can be stopped early
    const startResource = () => {
      const interval = setInterval(() => tick(), 100)
      cleanup = () => clearInterval(interval)
      onCleanup(cleanup)
    }

    const stopResource = () => {
      cleanup?.()
      cleanup = null
    }

    box({
      children: () => {
        text({ content: () => isActive.value ? 'Active' : 'Stopped' })
      },
      onKey: (e) => {
        if (e.keycode === 32) { // Space
          if (isActive.value) {
            stopResource()
          } else {
            startResource()
          }
          isActive.value = !isActive.value
          return true
        }
      },
      focusable: true,
    })

    startResource()
  })
}
```

### With Animations

Animation primitives automatically clean up within scopes:

```ts
import { cycle, Frames } from 'spark-tui/primitives';

function LoadingSpinner(): Cleanup {
  return scoped(() => {
    // cycle() registers its cleanup with the scope automatically
    text({ content: cycle(Frames.spinner, { fps: 12 }) })
  })
}

const cleanup = LoadingSpinner()
// Later...
cleanup() // Animation timer is automatically cleared
```

### Multiple Resources

```ts
function Dashboard(): Cleanup {
  return scoped(() => {
    const stats = signal({ cpu: 0, memory: 0 })

    // Multiple timers, all auto-cleaned
    const cpuInterval = setInterval(() => {
      stats.value = { ...stats.value, cpu: Math.random() * 100 }
    }, 1000)
    onCleanup(() => clearInterval(cpuInterval))

    const memInterval = setInterval(() => {
      stats.value = { ...stats.value, memory: Math.random() * 100 }
    }, 2000)
    onCleanup(() => clearInterval(memInterval))

    // WebSocket
    const ws = new WebSocket('ws://example.com/stats')
    ws.onmessage = (e) => {
      stats.value = JSON.parse(e.data)
    }
    onCleanup(() => ws.close())

    box({
      flexDirection: 'column',
      children: () => {
        text({ content: () => `CPU: ${stats.value.cpu.toFixed(1)}%` })
        text({ content: () => `Memory: ${stats.value.memory.toFixed(1)}%` })
      },
    })
  })
}
```

---

## Legacy APIs

These APIs are available for backwards compatibility but `scoped()` is preferred.

### componentScope

Creates a manual cleanup collector with effect scope.

```ts
function componentScope(): ComponentScopeResult

interface ComponentScopeResult {
  onCleanup: <T extends Cleanup>(cleanup: T) => T
  cleanup: Cleanup
  scope: EffectScope
}
```

**Deprecated**: Use `scoped()` instead.

### cleanupCollector

Creates a simple cleanup collector without effect scope.

```ts
function cleanupCollector(): {
  add: <T extends Cleanup>(cleanup: T) => T
  cleanup: Cleanup
}
```

**Deprecated**: Use `scoped()` with `onCleanup()` instead.

---

## Best Practices

### Always Use scoped() for Components

```ts
// Good
function MyComponent() {
  return scoped(() => {
    // ...
  })
}

// Avoid - manual cleanup tracking is error-prone
function MyComponent() {
  const cleanups: Cleanup[] = []
  // ...
  return () => cleanups.forEach(c => c())
}
```

### Register Cleanups Immediately

```ts
// Good - cleanup registered right after resource creation
const interval = setInterval(() => tick(), 1000)
onCleanup(() => clearInterval(interval))

// Risky - cleanup might be forgotten if code changes
const interval = setInterval(() => tick(), 1000)
// ... other code ...
onCleanup(() => clearInterval(interval))
```

### Don't Forget Async Cleanups

```ts
function AsyncComponent() {
  return scoped(() => {
    let cancelled = false

    async function loadData() {
      const result = await fetch('/api/data')
      if (cancelled) return // Check if unmounted
      // ... use result
    }

    onCleanup(() => {
      cancelled = true
    })

    loadData()
  })
}
```

### Test Cleanup Behavior

```ts
// Verify cleanups work correctly
const cleanup = MyComponent()
// ... verify component works
cleanup()
// ... verify resources are released (no memory leaks, timers stopped)
```

---

## See Also

- [Animation Primitives](../animation/cycle-pulse.md) - Animations that auto-clean up
- [Reactivity Concepts](../concepts/reactivity.md) - How effects and signals work
- [Box Component](../components/box.md) - Container component with children
