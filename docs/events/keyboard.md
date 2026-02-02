# Keyboard Events

> Handle keyboard input with reactive signals and type-safe event handlers.

SparkTUI's keyboard system is **purely reactive** - there are no polling loops or event queues. When a key is pressed, the event propagates through the reactive graph instantly.

## Import

```ts
import {
  // Event types
  type KeyEvent,
  EventType,

  // Handler types
  type KeyHandler,

  // Registration functions
  registerKeyHandler,
  registerGlobalKeyHandler,

  // Key state checks
  isKeyPress,
  isKeyRepeat,
  isKeyRelease,

  // Modifier checks
  hasCtrl,
  hasAlt,
  hasShift,
  hasMeta,

  // Key identification
  isEnter,
  isSpace,
  isEscape,
  isArrowKey,
  isFunctionKey,
  isChar,
  getChar,
  getKeyName,

  // Key code constants
  KEY_ENTER,
  KEY_TAB,
  KEY_BACKSPACE,
  KEY_ESCAPE,
  KEY_DELETE,
  KEY_SPACE,
  KEY_UP,
  KEY_DOWN,
  KEY_LEFT,
  KEY_RIGHT,
  KEY_HOME,
  KEY_END,
  KEY_PAGE_UP,
  KEY_PAGE_DOWN,
  KEY_INSERT,
  KEY_F1,
  KEY_F2,
  KEY_F3,
  KEY_F4,
  KEY_F5,
  KEY_F6,
  KEY_F7,
  KEY_F8,
  KEY_F9,
  KEY_F10,
  KEY_F11,
  KEY_F12,

  // Modifier constants
  MODIFIER_CTRL,
  MODIFIER_ALT,
  MODIFIER_SHIFT,
  MODIFIER_META,

  // Key state constants
  KEY_STATE_PRESS,
  KEY_STATE_REPEAT,
  KEY_STATE_RELEASE,
} from 'spark-tui'

// Reactive keyboard state (from ts/state/keyboard.ts)
import {
  lastEvent,      // Signal<KeyEvent | null>
  lastKey,        // Derived<string | null>
  modifiers,      // Derived<{ctrl, alt, shift, meta}>
  on,             // Global key handler
  onKey,          // Handler for specific key
  onFocused,      // Handler for focused component
  matchesKey,     // Check key combinations
  isPress,        // Check if key press (not repeat/release)
  isRepeat,       // Check if key repeat
  isRelease,      // Check if key release
} from 'spark-tui'
```

## Event Types

### KeyEvent

The `KeyEvent` interface represents a keyboard event:

```ts
interface KeyEvent {
  type: EventType.Key
  componentIndex: number  // Index of the focused component
  keycode: number         // Key code (see constants)
  modifiers: number       // Bitmask: ctrl=1, alt=2, shift=4, meta=8
  keyState: number        // 0=press, 1=repeat, 2=release
}
```

### KeyHandler

Handlers can return `true` to stop event propagation:

```ts
type KeyHandler = (event: KeyEvent) => boolean | void
```

## Key Code Constants

### Common Keys

| Constant | Value | Description |
|----------|-------|-------------|
| `KEY_ENTER` | `13` | Enter/Return key |
| `KEY_TAB` | `9` | Tab key |
| `KEY_BACKSPACE` | `8` | Backspace key |
| `KEY_ESCAPE` | `27` | Escape key |
| `KEY_DELETE` | `127` | Delete key |
| `KEY_SPACE` | `32` | Spacebar |

### Navigation Keys

| Constant | Value | Description |
|----------|-------|-------------|
| `KEY_UP` | `0x1001` | Up arrow |
| `KEY_DOWN` | `0x1002` | Down arrow |
| `KEY_LEFT` | `0x1003` | Left arrow |
| `KEY_RIGHT` | `0x1004` | Right arrow |
| `KEY_HOME` | `0x1005` | Home key |
| `KEY_END` | `0x1006` | End key |
| `KEY_PAGE_UP` | `0x1007` | Page Up |
| `KEY_PAGE_DOWN` | `0x1008` | Page Down |
| `KEY_INSERT` | `0x1009` | Insert key |

### Function Keys

| Constant | Value | Description |
|----------|-------|-------------|
| `KEY_F1` | `0x2001` | F1 key |
| `KEY_F2` | `0x2002` | F2 key |
| `KEY_F3` | `0x2003` | F3 key |
| `KEY_F4` | `0x2004` | F4 key |
| `KEY_F5` | `0x2005` | F5 key |
| `KEY_F6` | `0x2006` | F6 key |
| `KEY_F7` | `0x2007` | F7 key |
| `KEY_F8` | `0x2008` | F8 key |
| `KEY_F9` | `0x2009` | F9 key |
| `KEY_F10` | `0x200A` | F10 key |
| `KEY_F11` | `0x200B` | F11 key |
| `KEY_F12` | `0x200C` | F12 key |

### Modifier Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `MODIFIER_CTRL` | `1` | Control key modifier |
| `MODIFIER_ALT` | `2` | Alt key modifier |
| `MODIFIER_SHIFT` | `4` | Shift key modifier |
| `MODIFIER_META` | `8` | Meta/Command key modifier |

### Key State Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `KEY_STATE_PRESS` | `0` | Initial key press |
| `KEY_STATE_REPEAT` | `1` | Key held down (repeat) |
| `KEY_STATE_RELEASE` | `2` | Key released |

## Helper Functions

### Key Identification

| Function | Signature | Description |
|----------|-----------|-------------|
| `isEnter` | `(event: KeyEvent) => boolean` | Check if Enter key |
| `isSpace` | `(event: KeyEvent) => boolean` | Check if Space key |
| `isEscape` | `(event: KeyEvent) => boolean` | Check if Escape key |
| `isArrowKey` | `(event: KeyEvent) => boolean` | Check if any arrow key |
| `isFunctionKey` | `(event: KeyEvent) => boolean` | Check if F1-F12 |
| `isChar` | `(event: KeyEvent, char: string) => boolean` | Check if specific character |
| `getChar` | `(event: KeyEvent) => string \| undefined` | Get printable character (32-126) |
| `getKeyName` | `(event: KeyEvent) => string` | Get human-readable key name |

### Modifier Checks

| Function | Signature | Description |
|----------|-----------|-------------|
| `hasCtrl` | `(event: KeyEvent) => boolean` | Check if Ctrl is held |
| `hasAlt` | `(event: KeyEvent) => boolean` | Check if Alt is held |
| `hasShift` | `(event: KeyEvent) => boolean` | Check if Shift is held |
| `hasMeta` | `(event: KeyEvent) => boolean` | Check if Meta/Cmd is held |

### Key State Checks

| Function | Signature | Description |
|----------|-----------|-------------|
| `isKeyPress` | `(event: KeyEvent) => boolean` | Check if initial press |
| `isKeyRepeat` | `(event: KeyEvent) => boolean` | Check if key repeat |
| `isKeyRelease` | `(event: KeyEvent) => boolean` | Check if key release |
| `isPress` | `(event: KeyEvent) => boolean` | Alias for isKeyPress |
| `isRepeat` | `(event: KeyEvent) => boolean` | Alias for isKeyRepeat |
| `isRelease` | `(event: KeyEvent) => boolean` | Alias for isKeyRelease |

### Key Combination Matching

| Function | Signature | Description |
|----------|-----------|-------------|
| `matchesKey` | `(event: KeyEvent, combo: string) => boolean` | Match key combination like "Ctrl+S" |

## Reactive State

SparkTUI exposes keyboard state as reactive signals:

### `lastEvent`

The most recent keyboard event:

```ts
import { lastEvent } from 'spark-tui'
import { effect } from '@rlabs-inc/signals'

effect(() => {
  const event = lastEvent.value
  if (event) {
    console.log('Key pressed:', getKeyName(event))
  }
})
```

### `lastKey`

The last key pressed as a string (printable characters only):

```ts
import { lastKey } from 'spark-tui'

effect(() => {
  const key = lastKey.value
  if (key) {
    console.log('Character:', key)
  }
})
```

### `modifiers`

Current modifier key state:

```ts
import { modifiers } from 'spark-tui'

effect(() => {
  const { ctrl, alt, shift, meta } = modifiers.value
  console.log('Modifiers:', { ctrl, alt, shift, meta })
})
```

## Handler Registration

### Global Key Handlers

Handle all keyboard events regardless of focus:

```ts
import { on } from 'spark-tui'

// Global handler - return true to consume
const unsub = on((event) => {
  if (isEscape(event)) {
    closeModal()
    return true // Stop propagation
  }
})

// Cleanup when done
unsub()
```

### Specific Key Handlers

Handle a specific key globally:

```ts
import { onKey } from 'spark-tui'

// Handle Enter key globally
const unsub = onKey('Enter', () => {
  submitForm()
  return true
})

// Handle specific character
const unsub2 = onKey('q', () => {
  quit()
  return true
})
```

### Focused Component Handlers

Handle keys when a specific component has focus:

```ts
import { onFocused } from 'spark-tui'

// Inside a component
const unsub = onFocused(componentIndex, (event) => {
  if (isEnter(event)) {
    handleSubmit()
    return true
  }
})
```

### Component `onKey` Prop

The `box` and `input` primitives accept an `onKey` prop:

```ts
import { box } from 'spark-tui'

box({
  focusable: true,
  onKey: (event) => {
    if (hasCtrl(event) && isChar(event, 's')) {
      save()
      return true // Consume event
    }
  },
  children: () => {
    text({ content: 'Press Ctrl+S to save' })
  },
})
```

## Event Propagation

Keyboard events bubble from the focused component up to the root:

1. Global handlers are called first
2. If not consumed, the focused component's handler is called
3. If not consumed, parent handlers are called (bubbling up)
4. Return `true` from any handler to stop propagation

```ts
// Parent box handles shortcuts
box({
  onKey: (event) => {
    // This runs if child doesn't consume the event
    if (hasCtrl(event) && isChar(event, 'z')) {
      undo()
      return true
    }
  },
  children: () => {
    // Child input handles typing
    input({
      value: textSignal,
      // Input handles text entry, but Ctrl+Z bubbles up
    })
  },
})
```

## Examples

### Vim-style Navigation

```ts
import { box, text } from 'spark-tui'
import { signal } from '@rlabs-inc/signals'

const selectedIndex = signal(0)

box({
  focusable: true,
  onKey: (event) => {
    if (!isPress(event)) return // Only handle press, not repeat

    const char = getChar(event)
    switch (char) {
      case 'j': // Down
        selectedIndex.value++
        return true
      case 'k': // Up
        selectedIndex.value--
        return true
      case 'g': // Top (gg in vim, single g here)
        selectedIndex.value = 0
        return true
      case 'G': // Bottom
        selectedIndex.value = items.length - 1
        return true
    }

    // Also handle arrow keys
    if (event.keycode === KEY_DOWN) {
      selectedIndex.value++
      return true
    }
    if (event.keycode === KEY_UP) {
      selectedIndex.value--
      return true
    }
  },
})
```

### Key Combination Shortcuts

```ts
import { on, matchesKey } from 'spark-tui'

// Register global shortcuts
on((event) => {
  if (matchesKey(event, 'Ctrl+S')) {
    save()
    return true
  }
  if (matchesKey(event, 'Ctrl+Shift+Z')) {
    redo()
    return true
  }
  if (matchesKey(event, 'Alt+F4')) {
    quit()
    return true
  }
})
```

### Modal Dialog with Escape

```ts
import { box, text, on, isEscape } from 'spark-tui'
import { signal } from '@rlabs-inc/signals'

const showModal = signal(false)

// Global escape handler
on((event) => {
  if (isEscape(event) && showModal.value) {
    showModal.value = false
    return true
  }
})

// In your app
when(showModal, () => {
  box({
    // Modal content
    onKey: (event) => {
      if (isEnter(event)) {
        confirm()
        showModal.value = false
        return true
      }
    },
  })
})
```

### Arrow Key List Navigation

```ts
import { box, each, text } from 'spark-tui'
import { signal } from '@rlabs-inc/signals'

const items = ['Apple', 'Banana', 'Cherry', 'Date']
const selected = signal(0)

box({
  focusable: true,
  onKey: (event) => {
    if (!isKeyPress(event)) return

    switch (event.keycode) {
      case KEY_UP:
        selected.value = Math.max(0, selected.value - 1)
        return true
      case KEY_DOWN:
        selected.value = Math.min(items.length - 1, selected.value + 1)
        return true
      case KEY_HOME:
        selected.value = 0
        return true
      case KEY_END:
        selected.value = items.length - 1
        return true
      case KEY_ENTER:
        selectItem(items[selected.value])
        return true
    }
  },
  children: () => {
    each(items, (item, index) => {
      text({
        content: () => `${index === selected.value ? '>' : ' '} ${item}`,
        fg: () => index === selected.value ? { r: 255, g: 255, b: 0 } : null,
      })
    })
  },
})
```

### Text Input with Key Filtering

```ts
import { input } from 'spark-tui'
import { signal } from '@rlabs-inc/signals'

const value = signal('')

input({
  value,
  onKey: (event) => {
    const char = getChar(event)

    // Only allow digits
    if (char && !/[0-9]/.test(char)) {
      return true // Consume (block) non-digit input
    }

    // Allow control keys to pass through
  },
})
```

## See Also

- [Mouse Events](./mouse.md) - Mouse clicks, scrolling, and hover
- [Focus Management](../concepts/focus.md) - Managing component focus
- [Box Component](../components/box.md) - Container with keyboard support
- [Input Component](../components/input.md) - Text input with keyboard handling
