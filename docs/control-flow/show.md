# show

> Conditional rendering. Shows or hides components based on a reactive boolean condition. When the condition changes, components are automatically created or destroyed.

## Import

```ts
import { show } from 'spark-tui/primitives';
```

## Signature

```ts
function show(
  conditionGetter: () => boolean,
  renderFn: () => Cleanup,
  elseFn?: () => Cleanup
): Cleanup
```

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `conditionGetter` | `() => boolean` | Yes | A getter function that returns `true` or `false`. This creates a reactive dependency - when the condition changes, `show` re-evaluates. |
| `renderFn` | `() => Cleanup` | Yes | Called when the condition is `true`. Renders the "truthy" content. Must return a cleanup function. |
| `elseFn` | `() => Cleanup` | No | Called when the condition is `false`. Renders the "falsy" content. If omitted, nothing renders when the condition is false. |

## Return Value

Returns a `Cleanup` function. Call it to unmount the currently rendered branch and stop reactive tracking.

```ts
const cleanup = show(
  () => isVisible.value,
  () => text({ content: 'Visible!' }),
)

// Later, to unmount:
cleanup()
```

## Examples

### Basic Usage

Show or hide content based on a signal:

```ts
import { signal } from '@rlabs-inc/signals'
import { show, text, box } from 'spark-tui/primitives'

const isLoggedIn = signal(false)

box({
  children: () => {
    show(
      () => isLoggedIn.value,
      () => {
        text({ content: 'Welcome back!' })
        return () => {} // cleanup
      }
    )
  },
})

// Toggle login state
isLoggedIn.value = true  // Shows "Welcome back!"
isLoggedIn.value = false // Hides it
```

### With Else Branch

Render different content for true/false:

```ts
import { signal } from '@rlabs-inc/signals'
import { show, text, box } from 'spark-tui/primitives'
import { t } from 'spark-tui/theme'

const showDetails = signal(false)

box({
  children: () => {
    show(
      () => showDetails.value,
      () => {
        // True branch: expanded view
        box({
          flexDirection: 'column',
          gap: 1,
          children: () => {
            text({ content: 'Full Details', fg: t.primary })
            text({ content: 'Name: John Doe' })
            text({ content: 'Email: john@example.com' })
            text({ content: 'Role: Admin' })
          },
        })
        return () => {}
      },
      () => {
        // False branch: collapsed view
        text({ content: 'Click to expand...', fg: t.textMuted })
        return () => {}
      }
    )
  },
})
```

### Toggle Button Pattern

Common UI pattern for show/hide toggles:

```ts
import { signal } from '@rlabs-inc/signals'
import { show, text, box } from 'spark-tui/primitives'
import { t } from 'spark-tui/theme'
import { isEnter, isSpace } from 'spark-tui/events'

const isExpanded = signal(false)

box({
  flexDirection: 'column',
  gap: 1,
  children: () => {
    // Toggle button
    box({
      flexDirection: 'row',
      gap: 1,
      focusable: true,
      onClick: () => { isExpanded.value = !isExpanded.value },
      onKey: (e) => {
        if (isEnter(e) || isSpace(e)) {
          isExpanded.value = !isExpanded.value
          return true
        }
      },
      children: () => {
        text({
          content: () => isExpanded.value ? '[-]' : '[+]',
          fg: t.primary,
        })
        text({ content: 'Details' })
      },
    })

    // Conditional content
    show(
      () => isExpanded.value,
      () => {
        box({
          padding: 1,
          border: 1,
          borderColor: t.primary,
          children: () => {
            text({ content: 'Here are the details...' })
          },
        })
        return () => {}
      }
    )
  },
})
```

### Loading State

Show loading indicator while data loads:

```ts
import { signal } from '@rlabs-inc/signals'
import { show, text, box, cycle, Frames } from 'spark-tui/primitives'
import { t } from 'spark-tui/theme'

const isLoading = signal(true)
const data = signal<string | null>(null)

box({
  children: () => {
    show(
      () => isLoading.value,
      () => {
        // Loading state
        box({
          flexDirection: 'row',
          gap: 1,
          children: () => {
            text({ content: cycle(Frames.spinner, { fps: 12 }) })
            text({ content: 'Loading...', fg: t.textMuted })
          },
        })
        return () => {}
      },
      () => {
        // Loaded state
        text({ content: () => `Data: ${data.value}` })
        return () => {}
      }
    )
  },
})

// Simulate data load
setTimeout(() => {
  data.value = 'Hello from the server!'
  isLoading.value = false
}, 2000)
```

### Nested Conditions

Chain `show` calls for complex logic:

```ts
import { signal } from '@rlabs-inc/signals'
import { show, text, box } from 'spark-tui/primitives'
import { t } from 'spark-tui/theme'

type Status = 'idle' | 'loading' | 'success' | 'error'

const status = signal<Status>('idle')
const errorMessage = signal('')

box({
  children: () => {
    show(
      () => status.value === 'loading',
      () => {
        text({ content: 'Loading...' })
        return () => {}
      },
      () => {
        // Not loading - check other states
        show(
          () => status.value === 'error',
          () => {
            text({ content: () => `Error: ${errorMessage.value}`, fg: t.error })
            return () => {}
          },
          () => {
            show(
              () => status.value === 'success',
              () => {
                text({ content: 'Success!', fg: t.success })
                return () => {}
              },
              () => {
                text({ content: 'Ready', fg: t.textMuted })
                return () => {}
              }
            )
            return () => {}
          }
        )
        return () => {}
      }
    )
  },
})
```

### Form Validation

Show validation errors conditionally:

```ts
import { signal, derived } from '@rlabs-inc/signals'
import { show, text, box, input } from 'spark-tui/primitives'
import { t } from 'spark-tui/theme'

const email = signal('')

const isValidEmail = derived(() => {
  const value = email.value
  return value.length === 0 || value.includes('@')
})

const showError = derived(() => {
  return email.value.length > 0 && !isValidEmail.value
})

box({
  flexDirection: 'column',
  gap: 1,
  children: () => {
    // Input field
    box({
      flexDirection: 'row',
      gap: 1,
      children: () => {
        text({ content: 'Email:' })
        input({
          value: email,
          width: 30,
          placeholder: 'user@example.com',
        })
      },
    })

    // Validation error
    show(
      () => showError.value,
      () => {
        text({
          content: 'Please enter a valid email address',
          fg: t.error,
        })
        return () => {}
      }
    )
  },
})
```

### Feature Flags

Toggle features based on configuration:

```ts
import { signal } from '@rlabs-inc/signals'
import { show, text, box } from 'spark-tui/primitives'

interface FeatureFlags {
  newDashboard: boolean
  betaFeatures: boolean
  debugMode: boolean
}

const features = signal<FeatureFlags>({
  newDashboard: true,
  betaFeatures: false,
  debugMode: process.env.NODE_ENV === 'development',
})

box({
  flexDirection: 'column',
  children: () => {
    // New dashboard (enabled)
    show(
      () => features.value.newDashboard,
      () => {
        box({
          children: () => text({ content: 'New Dashboard Component' }),
        })
        return () => {}
      },
      () => {
        box({
          children: () => text({ content: 'Legacy Dashboard' }),
        })
        return () => {}
      }
    )

    // Debug panel (dev only)
    show(
      () => features.value.debugMode,
      () => {
        box({
          border: 1,
          borderColor: { r: 255, g: 165, b: 0, a: 255 },
          children: () => text({ content: 'Debug: ON' }),
        })
        return () => {}
      }
    )
  },
})
```

## How It Works

The `show` primitive manages conditional rendering through reactive effects:

1. **Initial Render**: When `show` is called, it immediately evaluates `conditionGetter()` and calls either `renderFn` or `elseFn` based on the result.

2. **Reactive Tracking**: The condition getter is read inside a reactive effect. Any signals accessed inside `conditionGetter()` are tracked as dependencies.

3. **Condition Change**: When a tracked dependency changes:
   - The effect re-runs and evaluates the new condition
   - If the condition changed (true to false, or false to true):
     - The current branch's cleanup function is called
     - The new branch's render function is called
   - If the condition stayed the same, nothing happens (short-circuit)

4. **Cleanup**: When `show` itself is cleaned up (e.g., parent unmounts), it calls the current branch's cleanup function.

### Reactive Flow

```
conditionGetter() evaluated
  |
  v
Condition changed since last render?
  |
  +-- No  -> Do nothing (skip)
  |
  +-- Yes -> Call current cleanup()
             |
             v
             Condition is true?
               |
               +-- Yes -> Call renderFn(), store cleanup
               +-- No  -> Call elseFn() if exists, store cleanup
```

### Render vs Hide

`show` completely **unmounts** components when the condition becomes false. This is different from just hiding them visually:

- Components are destroyed and recreated (not hidden)
- Component state is lost when hidden
- Event listeners are removed
- Resources are freed

If you need to preserve component state while hidden, consider using the `visible` prop instead:

```ts
// Components destroyed on hide (show)
show(
  () => isVisible.value,
  () => expensiveComponent()
)

// Components preserved but invisible (visible prop)
box({
  visible: isVisible,
  children: () => expensiveComponent(),
})
```

## Common Patterns

### Guard Clauses

Early return pattern for auth/permissions:

```ts
show(
  () => !isAuthenticated.value,
  () => {
    text({ content: 'Please log in to continue' })
    return () => {}
  },
  () => {
    // Main app content only if authenticated
    mainAppContent()
    return () => {}
  }
)
```

### Combined with each

Conditional items in a list:

```ts
each(
  () => items.value,
  (getItem, key) => {
    return show(
      () => getItem().isActive,
      () => {
        text({ content: () => getItem().name })
        return () => {}
      }
    )
  },
  { key: (item) => item.id }
)
```

### Multiple Independent Conditions

Multiple `show` calls render independently:

```ts
box({
  children: () => {
    show(
      () => showHeader.value,
      () => { header(); return () => {} }
    )

    mainContent()

    show(
      () => showFooter.value,
      () => { footer(); return () => {} }
    )
  },
})
```

## See Also

- [each](./each.md) - List rendering
- [when](./when.md) - Async/promise handling
- [Reactivity Concepts](/docs/concepts/reactivity.md) - Understanding signals and effects
