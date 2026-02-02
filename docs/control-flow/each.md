# each

> Reactive list rendering with fine-grained updates. Creates components for each item in an array, automatically adding/removing them as the array changes.

## Import

```ts
import { each } from 'spark-tui/primitives';
```

## Signature

```ts
function each<T>(
  itemsGetter: () => T[],
  renderFn: (getItem: () => T, key: string) => Cleanup,
  options: { key: (item: T) => string }
): Cleanup
```

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `itemsGetter` | `() => T[]` | Yes | A getter function that returns the array of items. This creates a reactive dependency - when the array changes, `each` re-evaluates. |
| `renderFn` | `(getItem: () => T, key: string) => Cleanup` | Yes | Called once per item. Receives a reactive getter for the item value and a stable key. Must return a cleanup function. |
| `options.key` | `(item: T) => string` | Yes | Function to extract a unique string key from each item. Keys must be unique within the list. |

### Render Function Arguments

The `renderFn` receives two arguments:

| Argument | Type | Description |
|----------|------|-------------|
| `getItem` | `() => T` | A reactive getter that returns the current item value. Call `getItem()` inside your components to read the item reactively. |
| `key` | `string` | The stable key for this item. Useful for tracking selection, building IDs, or debugging. |

## Return Value

Returns a `Cleanup` function. Call it to remove all rendered items and stop reactive tracking.

```ts
const cleanup = each(() => items.value, renderFn, { key: item => item.id })

// Later, to unmount:
cleanup()
```

## Examples

### Basic Usage

Render a simple list of text items:

```ts
import { signal } from '@rlabs-inc/signals'
import { each, text, box } from 'spark-tui/primitives'

interface Fruit {
  id: string
  name: string
}

const fruits = signal<Fruit[]>([
  { id: '1', name: 'Apple' },
  { id: '2', name: 'Banana' },
  { id: '3', name: 'Cherry' },
])

box({
  flexDirection: 'column',
  children: () => {
    each(
      () => fruits.value,
      (getItem, key) => {
        return text({
          content: () => `- ${getItem().name}`,
        })
      },
      { key: (item) => item.id }
    )
  },
})
```

### With Dynamic Data

Items update in place without recreating components:

```ts
import { signal } from '@rlabs-inc/signals'
import { each, text, box } from 'spark-tui/primitives'

interface Task {
  id: string
  title: string
  done: boolean
}

const tasks = signal<Task[]>([
  { id: 'a', title: 'Write docs', done: false },
  { id: 'b', title: 'Run tests', done: true },
])

box({
  flexDirection: 'column',
  children: () => {
    each(
      () => tasks.value,
      (getItem, key) => {
        // getItem() is reactive - when task updates, this re-renders
        return box({
          flexDirection: 'row',
          gap: 1,
          children: () => {
            text({
              content: () => getItem().done ? '[x]' : '[ ]',
            })
            text({
              content: () => getItem().title,
            })
          },
        })
      },
      { key: (task) => task.id }
    )
  },
})

// Toggle a task - only that item re-renders, not the whole list
function toggleTask(id: string) {
  tasks.value = tasks.value.map(t =>
    t.id === id ? { ...t, done: !t.done } : t
  )
}
```

### Add and Remove Items

```ts
import { signal } from '@rlabs-inc/signals'
import { each, text, box } from 'spark-tui/primitives'

interface Item {
  id: string
  name: string
}

const items = signal<Item[]>([
  { id: '1', name: 'Item 1' },
])

// Add a new item
function addItem() {
  const id = String(Date.now())
  items.value = [...items.value, { id, name: `Item ${id.slice(-4)}` }]
}

// Remove the last item
function removeItem() {
  items.value = items.value.slice(0, -1)
}

// Remove by ID
function removeById(id: string) {
  items.value = items.value.filter(item => item.id !== id)
}

box({
  flexDirection: 'column',
  gap: 1,
  children: () => {
    // List
    each(
      () => items.value,
      (getItem, key) => {
        return text({ content: () => getItem().name })
      },
      { key: (item) => item.id }
    )

    // Controls
    box({
      flexDirection: 'row',
      gap: 2,
      children: () => {
        box({
          focusable: true,
          onClick: addItem,
          children: () => text({ content: 'Add' }),
        })
        box({
          focusable: true,
          onClick: removeItem,
          children: () => text({ content: 'Remove' }),
        })
      },
    })
  },
})
```

### Selection Pattern

Use the stable key for tracking selection:

```ts
import { signal } from '@rlabs-inc/signals'
import { each, text, box } from 'spark-tui/primitives'
import { t } from 'spark-tui/theme'

interface ListItem {
  id: string
  label: string
}

const items = signal<ListItem[]>([
  { id: 'a', label: 'Option A' },
  { id: 'b', label: 'Option B' },
  { id: 'c', label: 'Option C' },
])

const selectedId = signal<string | null>(null)

box({
  flexDirection: 'column',
  children: () => {
    each(
      () => items.value,
      (getItem, key) => {
        // key is stable - use it for selection comparison
        return box({
          id: `item-${key}`,
          focusable: true,
          bg: () => selectedId.value === key ? t.primary.value : null,
          fg: () => selectedId.value === key ? t.textBright.value : t.text.value,
          onClick: () => { selectedId.value = key },
          children: () => {
            text({ content: () => getItem().label })
          },
        })
      },
      { key: (item) => item.id }
    )
  },
})
```

### Nested Lists

Each items can contain their own `each` for nested structures:

```ts
import { signal } from '@rlabs-inc/signals'
import { each, text, box } from 'spark-tui/primitives'

interface Category {
  id: string
  name: string
  items: { id: string; name: string }[]
}

const categories = signal<Category[]>([
  {
    id: 'fruits',
    name: 'Fruits',
    items: [
      { id: 'f1', name: 'Apple' },
      { id: 'f2', name: 'Banana' },
    ],
  },
  {
    id: 'veggies',
    name: 'Vegetables',
    items: [
      { id: 'v1', name: 'Carrot' },
      { id: 'v2', name: 'Broccoli' },
    ],
  },
])

box({
  flexDirection: 'column',
  gap: 1,
  children: () => {
    each(
      () => categories.value,
      (getCategory, categoryKey) => {
        return box({
          flexDirection: 'column',
          children: () => {
            // Category header
            text({ content: () => getCategory().name })

            // Nested items
            each(
              () => getCategory().items,
              (getItem, itemKey) => {
                return text({ content: () => `  - ${getItem().name}` })
              },
              { key: (item) => item.id }
            )
          },
        })
      },
      { key: (cat) => cat.id }
    )
  },
})
```

## How It Works

The `each` primitive implements fine-grained reactivity for lists:

1. **Initial Render**: On first run, `each` calls `itemsGetter()` to get the array. For each item, it extracts the key using `options.key()`, creates a signal containing that item, and calls `renderFn` with a getter that reads from that signal.

2. **Item Updates**: When the array changes, `each` compares keys:
   - **Existing items**: The item's signal is updated (not recreated). Components reading `getItem()` reactively re-render.
   - **New items**: A new signal is created and `renderFn` is called.
   - **Removed items**: The cleanup function from `renderFn` is called, and the signal is deleted.

3. **Fine-Grained Updates**: Because each item has its own signal, updating one item only affects components that read from that specific item. Other items don't re-render.

### Why Keys Matter

Keys must be:
- **Unique**: Duplicate keys cause warnings and undefined behavior
- **Stable**: Don't use array index as key if items can be reordered
- **Derived from data**: Typically use item IDs from your data model

```ts
// Good: unique ID from data
{ key: (item) => item.id }

// Good: composite key for uniqueness
{ key: (item) => `${item.type}-${item.id}` }

// Bad: array index (breaks on reorder)
{ key: (item, index) => String(index) }

// Bad: non-unique values
{ key: (item) => item.category }
```

### Reactive Flow

```
items signal changes
  |
  v
each() effect re-runs
  |
  +-- For each item: extract key
  |
  +-- Key exists?
  |     Yes -> Update item signal (fine-grained!)
  |     No  -> Create signal + call renderFn
  |
  +-- Keys not in new array?
        -> Call cleanup, delete signal
```

## Common Patterns

### Conditional Item Rendering

Combine with `show` for conditional items:

```ts
each(
  () => items.value,
  (getItem, key) => {
    return show(
      () => getItem().visible,
      () => text({ content: () => getItem().name }),
    )
  },
  { key: (item) => item.id }
)
```

### Derived Lists

Filter or transform lists reactively:

```ts
import { derived } from '@rlabs-inc/signals'

const allItems = signal([...])
const filter = signal('')

// Derived filtered list
const filteredItems = derived(() =>
  allItems.value.filter(item =>
    item.name.toLowerCase().includes(filter.value.toLowerCase())
  )
)

// each reads from derived
each(
  () => filteredItems.value,
  (getItem, key) => text({ content: () => getItem().name }),
  { key: (item) => item.id }
)
```

## See Also

- [show](./show.md) - Conditional rendering
- [when](./when.md) - Async/promise handling
- [Reactivity Concepts](/docs/concepts/reactivity.md) - Understanding signals and effects
