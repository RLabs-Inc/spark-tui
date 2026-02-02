# when

> Async rendering for promises. Shows different content based on promise state: pending, resolved (then), or rejected (catch). When the promise settles, the appropriate handler renders automatically.

## Import

```ts
import { when } from 'spark-tui/primitives';
```

## Signature

```ts
function when<T>(
  promiseGetter: () => Promise<T>,
  options: WhenOptions<T>
): Cleanup

interface WhenOptions<T> {
  pending?: () => Cleanup
  then: (value: T) => Cleanup
  catch?: (error: Error) => Cleanup
}
```

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `promiseGetter` | `() => Promise<T>` | Yes | A getter function that returns a Promise. This creates a reactive dependency - when a new promise is returned, `when` handles the new promise. |
| `options.pending` | `() => Cleanup` | No | Called immediately while the promise is pending. Renders loading state. |
| `options.then` | `(value: T) => Cleanup` | Yes | Called when the promise resolves. Receives the resolved value. Must return a cleanup function. |
| `options.catch` | `(error: Error) => Cleanup` | No | Called when the promise rejects. Receives the error. If omitted, errors are logged to console. |

## Return Value

Returns a `Cleanup` function. Call it to cancel pending promise handling and unmount any rendered content.

```ts
const cleanup = when(
  () => fetchData(),
  {
    pending: () => text({ content: 'Loading...' }),
    then: (data) => text({ content: data }),
  }
)

// Later, to cancel and unmount:
cleanup()
```

## Examples

### Basic Usage

Fetch and display data:

```ts
import { when, text, box } from 'spark-tui/primitives'

async function fetchUser(): Promise<string> {
  const response = await fetch('/api/user')
  const data = await response.json()
  return data.name
}

box({
  children: () => {
    when(
      () => fetchUser(),
      {
        pending: () => {
          text({ content: 'Loading user...' })
          return () => {}
        },
        then: (name) => {
          text({ content: `Hello, ${name}!` })
          return () => {}
        },
        catch: (error) => {
          text({ content: `Error: ${error.message}` })
          return () => {}
        },
      }
    )
  },
})
```

### With Loading Spinner

Show an animated spinner while loading:

```ts
import { when, text, box, cycle, Frames } from 'spark-tui/primitives'
import { t } from 'spark-tui/theme'

interface ApiData {
  title: string
  description: string
}

async function fetchData(): Promise<ApiData> {
  const response = await fetch('/api/data')
  return response.json()
}

box({
  children: () => {
    when(
      () => fetchData(),
      {
        pending: () => {
          box({
            flexDirection: 'row',
            gap: 1,
            children: () => {
              text({ content: cycle(Frames.spinner, { fps: 12 }), fg: t.primary })
              text({ content: 'Fetching data...', fg: t.textMuted })
            },
          })
          return () => {}
        },
        then: (data) => {
          box({
            flexDirection: 'column',
            gap: 1,
            border: 1,
            borderColor: t.primary,
            padding: 1,
            children: () => {
              text({ content: data.title, fg: t.primary })
              text({ content: data.description })
            },
          })
          return () => {}
        },
        catch: (error) => {
          box({
            border: 1,
            borderColor: t.error,
            padding: 1,
            children: () => {
              text({ content: 'Failed to load data', fg: t.error })
              text({ content: error.message, fg: t.textMuted })
            },
          })
          return () => {}
        },
      }
    )
  },
})
```

### Reactive Promise

Re-fetch when a signal changes:

```ts
import { signal } from '@rlabs-inc/signals'
import { when, text, box, input } from 'spark-tui/primitives'
import { t } from 'spark-tui/theme'

const searchQuery = signal('')

interface SearchResult {
  id: string
  name: string
}

async function search(query: string): Promise<SearchResult[]> {
  if (!query) return []
  const response = await fetch(`/api/search?q=${encodeURIComponent(query)}`)
  return response.json()
}

box({
  flexDirection: 'column',
  gap: 1,
  children: () => {
    // Search input
    box({
      flexDirection: 'row',
      gap: 1,
      children: () => {
        text({ content: 'Search:' })
        input({
          value: searchQuery,
          width: 30,
          placeholder: 'Type to search...',
        })
      },
    })

    // Results - re-fetches when searchQuery changes
    when(
      () => search(searchQuery.value),
      {
        pending: () => {
          text({ content: 'Searching...', fg: t.textMuted })
          return () => {}
        },
        then: (results) => {
          if (results.length === 0) {
            text({ content: 'No results found', fg: t.textMuted })
          } else {
            box({
              flexDirection: 'column',
              children: () => {
                for (const result of results) {
                  text({ content: `- ${result.name}` })
                }
              },
            })
          }
          return () => {}
        },
        catch: (error) => {
          text({ content: `Search failed: ${error.message}`, fg: t.error })
          return () => {}
        },
      }
    )
  },
})
```

### Chained Promises

Handle dependent async operations:

```ts
import { signal } from '@rlabs-inc/signals'
import { when, text, box } from 'spark-tui/primitives'
import { t } from 'spark-tui/theme'

const userId = signal<string | null>(null)

interface User {
  id: string
  name: string
}

interface UserProfile {
  bio: string
  avatar: string
}

async function fetchUserWithProfile(): Promise<{ user: User; profile: UserProfile }> {
  const id = userId.value
  if (!id) throw new Error('No user selected')

  const userResponse = await fetch(`/api/users/${id}`)
  const user = await userResponse.json()

  const profileResponse = await fetch(`/api/users/${id}/profile`)
  const profile = await profileResponse.json()

  return { user, profile }
}

box({
  children: () => {
    when(
      () => fetchUserWithProfile(),
      {
        pending: () => {
          text({ content: 'Loading user and profile...' })
          return () => {}
        },
        then: ({ user, profile }) => {
          box({
            flexDirection: 'column',
            gap: 1,
            children: () => {
              text({ content: user.name, fg: t.primary })
              text({ content: profile.bio })
            },
          })
          return () => {}
        },
        catch: (error) => {
          text({ content: `Failed: ${error.message}`, fg: t.error })
          return () => {}
        },
      }
    )
  },
})
```

### Without Pending State

Skip the loading state for fast operations:

```ts
import { when, text, box } from 'spark-tui/primitives'

async function getConfig(): Promise<{ theme: string }> {
  // Fast local operation
  return { theme: 'dark' }
}

box({
  children: () => {
    when(
      () => getConfig(),
      {
        // No pending - content appears when ready
        then: (config) => {
          text({ content: `Theme: ${config.theme}` })
          return () => {}
        },
      }
    )
  },
})
```

### Error Recovery

Allow users to retry failed operations:

```ts
import { signal } from '@rlabs-inc/signals'
import { when, text, box } from 'spark-tui/primitives'
import { t } from 'spark-tui/theme'
import { isEnter, isSpace } from 'spark-tui/events'

// Retry counter to force new promise
const retryCount = signal(0)

async function fetchData(): Promise<string> {
  const response = await fetch('/api/data')
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`)
  }
  return response.text()
}

box({
  children: () => {
    when(
      // Reading retryCount creates dependency - incrementing it refetches
      () => {
        retryCount.value  // Track for reactivity
        return fetchData()
      },
      {
        pending: () => {
          text({ content: 'Loading...' })
          return () => {}
        },
        then: (data) => {
          text({ content: data })
          return () => {}
        },
        catch: (error) => {
          box({
            flexDirection: 'column',
            gap: 1,
            children: () => {
              text({ content: `Error: ${error.message}`, fg: t.error })

              // Retry button
              box({
                focusable: true,
                onClick: () => { retryCount.value++ },
                onKey: (e) => {
                  if (isEnter(e) || isSpace(e)) {
                    retryCount.value++
                    return true
                  }
                },
                children: () => {
                  text({ content: '[Retry]', fg: t.primary })
                },
              })
            },
          })
          return () => {}
        },
      }
    )
  },
})
```

### Timeout Handling

Add timeout to promise operations:

```ts
import { when, text, box } from 'spark-tui/primitives'
import { t } from 'spark-tui/theme'

function withTimeout<T>(promise: Promise<T>, ms: number): Promise<T> {
  return Promise.race([
    promise,
    new Promise<never>((_, reject) =>
      setTimeout(() => reject(new Error('Request timed out')), ms)
    ),
  ])
}

async function slowFetch(): Promise<string> {
  const response = await fetch('/api/slow')
  return response.text()
}

box({
  children: () => {
    when(
      () => withTimeout(slowFetch(), 5000),
      {
        pending: () => {
          text({ content: 'Loading (5s timeout)...' })
          return () => {}
        },
        then: (data) => {
          text({ content: data })
          return () => {}
        },
        catch: (error) => {
          text({
            content: error.message === 'Request timed out'
              ? 'Request timed out. Please try again.'
              : `Error: ${error.message}`,
            fg: t.error,
          })
          return () => {}
        },
      }
    )
  },
})
```

## How It Works

The `when` primitive manages async rendering through promise handling:

1. **Initial Setup**: When `when` is called, it immediately:
   - Calls `promiseGetter()` to get the initial promise
   - Renders the `pending` handler (if provided)
   - Attaches `.then()` and `.catch()` handlers to the promise

2. **Promise Settles**: When the promise resolves or rejects:
   - Cleans up the current rendered content
   - Calls the appropriate handler (`then` or `catch`)
   - Stores the new cleanup function

3. **New Promise**: If `promiseGetter` returns a different promise (e.g., due to signal change):
   - The old promise is "staled" - its handlers won't render
   - The new promise becomes active
   - Pending state is shown again
   - Process repeats

4. **Cleanup**: When `when` itself is cleaned up:
   - Current promise is staled (handlers won't fire)
   - Current rendered content is cleaned up

### Promise Staleness

`when` handles race conditions gracefully. If a new promise is created before the old one settles:

```
Promise A starts (shows pending)
  |
  v
Promise B starts (A is now "stale", shows pending)
  |
  v
Promise A resolves (ignored - A is stale)
  |
  v
Promise B resolves (renders B's result)
```

This prevents the UI from showing stale data when rapid changes occur.

### Reactive Flow

```
promiseGetter() called
  |
  v
Promise same as current?
  |
  +-- Yes -> Do nothing
  |
  +-- No  -> Stale current promise
             Render pending() if exists
             |
             v
             Promise settles
             |
             +-- Still current promise?
                   |
                   +-- No  -> Ignore (stale)
                   +-- Yes -> Cleanup current
                              Render then() or catch()
```

### Error Handling

If `catch` is not provided, rejected promises log to console:

```ts
when(
  () => failingPromise(),
  {
    then: (value) => text({ content: value }),
    // No catch - errors logged with: [when] Unhandled promise rejection: Error
  }
)
```

Always provide a `catch` handler in production for better UX.

## Common Patterns

### Parallel Promises

Load multiple resources simultaneously:

```ts
interface CombinedData {
  user: User
  settings: Settings
}

async function loadAll(): Promise<CombinedData> {
  const [user, settings] = await Promise.all([
    fetchUser(),
    fetchSettings(),
  ])
  return { user, settings }
}

when(
  () => loadAll(),
  {
    pending: () => text({ content: 'Loading...' }),
    then: ({ user, settings }) => renderDashboard(user, settings),
    catch: (error) => text({ content: `Failed: ${error.message}` }),
  }
)
```

### Conditional Fetching

Only fetch when needed:

```ts
const shouldFetch = signal(false)

when(
  () => {
    if (!shouldFetch.value) {
      return Promise.resolve(null)
    }
    return fetchData()
  },
  {
    then: (data) => {
      if (data === null) {
        text({ content: 'Click to load' })
      } else {
        text({ content: data })
      }
      return () => {}
    },
  }
)
```

### Transform Results

Process data before rendering:

```ts
interface RawData {
  items: { name: string; value: number }[]
}

interface ProcessedData {
  total: number
  average: number
  items: string[]
}

async function fetchAndProcess(): Promise<ProcessedData> {
  const response = await fetch('/api/data')
  const raw: RawData = await response.json()

  const total = raw.items.reduce((sum, item) => sum + item.value, 0)
  const average = total / raw.items.length

  return {
    total,
    average,
    items: raw.items.map(item => item.name),
  }
}

when(
  () => fetchAndProcess(),
  {
    then: (data) => {
      box({
        flexDirection: 'column',
        children: () => {
          text({ content: `Total: ${data.total}` })
          text({ content: `Average: ${data.average.toFixed(2)}` })
        },
      })
      return () => {}
    },
  }
)
```

## See Also

- [each](./each.md) - List rendering
- [show](./show.md) - Conditional rendering
- [Reactivity Concepts](/docs/concepts/reactivity.md) - Understanding signals and effects
