# Mouse Events

> Handle mouse clicks, movement, and scroll events with reactive signals.

SparkTUI's mouse system is **purely reactive** - there are no polling loops or event queues. When the mouse moves or a button is clicked, the event propagates through the reactive graph instantly.

## Import

```ts
import {
  // Event types
  type MouseEvent,
  type ScrollEvent,
  EventType,

  // Handler types
  type MouseHandler,
  type ScrollHandler,

  // Registration functions
  registerMouseHandler,
  registerGlobalMouseHandler,
  registerScrollHandler,
  registerGlobalScrollHandler,

  // Mouse button constants
  MOUSE_BUTTON_LEFT,
  MOUSE_BUTTON_MIDDLE,
  MOUSE_BUTTON_RIGHT,
} from 'spark-tui'

// Reactive mouse state (from ts/state/mouse.ts)
import {
  lastMouseEvent,    // Signal<MouseEvent | null>
  mouseX,            // Signal<number>
  mouseY,            // Signal<number>
  mousePosition,     // Derived<{x, y}>
  isMouseDown,       // Signal<boolean>
  onComponent,       // Component mouse handlers
  onGlobalClick,     // Global click handler
  onGlobalMouse,     // All mouse events globally
  onGlobalMove,      // Mouse move events
  onGlobalScroll,    // Scroll events globally
  isPointInBounds,   // Hit testing helper
  getButtonName,     // Get button name string
  isLeftButton,      // Check left button
  isMiddleButton,    // Check middle button
  isRightButton,     // Check right button
} from 'spark-tui'
```

## Event Types

### MouseEvent

The `MouseEvent` interface represents mouse button and movement events:

```ts
interface MouseEvent {
  type:
    | EventType.MouseDown
    | EventType.MouseUp
    | EventType.Click
    | EventType.MouseEnter
    | EventType.MouseLeave
    | EventType.MouseMove
  componentIndex: number  // Index of the target component
  x: number               // X position in terminal cells
  y: number               // Y position in terminal cells
  button: number          // 0=left, 1=middle, 2=right
}
```

### ScrollEvent

The `ScrollEvent` interface represents mouse scroll wheel events:

```ts
interface ScrollEvent {
  type: EventType.Scroll
  componentIndex: number  // Index of the target component
  deltaX: number          // Horizontal scroll amount
  deltaY: number          // Vertical scroll amount (negative = up, positive = down)
}
```

### MouseHandlers Interface

The component handler registration interface:

```ts
interface MouseHandlers {
  onMouseDown?: (event: MouseEvent) => void
  onMouseUp?: (event: MouseEvent) => void
  onClick?: (event: MouseEvent) => void
  onMouseEnter?: (event: MouseEvent) => void
  onMouseLeave?: (event: MouseEvent) => void
  onScroll?: (event: ScrollEvent) => void
}
```

## Mouse Button Constants

| Constant | Value | Description |
|----------|-------|-------------|
| `MOUSE_BUTTON_LEFT` | `0` | Left mouse button |
| `MOUSE_BUTTON_MIDDLE` | `1` | Middle mouse button (scroll wheel click) |
| `MOUSE_BUTTON_RIGHT` | `2` | Right mouse button |

## Helper Functions

### Button Identification

| Function | Signature | Description |
|----------|-----------|-------------|
| `isLeftButton` | `(event: MouseEvent) => boolean` | Check if left button |
| `isMiddleButton` | `(event: MouseEvent) => boolean` | Check if middle button |
| `isRightButton` | `(event: MouseEvent) => boolean` | Check if right button |
| `getButtonName` | `(event: MouseEvent) => 'left' \| 'middle' \| 'right'` | Get button name string |

### Hit Testing

| Function | Signature | Description |
|----------|-----------|-------------|
| `isPointInBounds` | `(x, y, bounds) => boolean` | Check if point is inside bounding box |

```ts
// bounds: { x: number, y: number, width: number, height: number }
if (isPointInBounds(event.x, event.y, buttonBounds)) {
  handleButtonClick()
}
```

## Reactive State

SparkTUI exposes mouse state as reactive signals:

### `mouseX` and `mouseY`

Current mouse position in terminal cells:

```ts
import { mouseX, mouseY } from 'spark-tui'
import { effect } from '@rlabs-inc/signals'

effect(() => {
  console.log(`Mouse at (${mouseX.value}, ${mouseY.value})`)
})
```

### `mousePosition`

Combined position as an object (derived from mouseX and mouseY):

```ts
import { mousePosition } from 'spark-tui'

effect(() => {
  const { x, y } = mousePosition.value
  updateCursorDisplay(x, y)
})
```

### `isMouseDown`

Whether any mouse button is currently pressed:

```ts
import { isMouseDown } from 'spark-tui'

effect(() => {
  if (isMouseDown.value) {
    startDragging()
  } else {
    stopDragging()
  }
})
```

### `lastMouseEvent`

The most recent mouse event:

```ts
import { lastMouseEvent } from 'spark-tui'

effect(() => {
  const event = lastMouseEvent.value
  if (event && event.type === EventType.Click) {
    console.log(`Clicked at (${event.x}, ${event.y})`)
  }
})
```

## Handler Registration

### Component Mouse Handlers

Register handlers for a specific component:

```ts
import { onComponent } from 'spark-tui'

const unsub = onComponent(componentIndex, {
  onClick: (event) => {
    console.log('Clicked at', event.x, event.y)
  },
  onMouseEnter: (event) => {
    setHovered(true)
  },
  onMouseLeave: (event) => {
    setHovered(false)
  },
  onScroll: (event) => {
    scrollY.value += event.deltaY
  },
})

// Cleanup when done
unsub()
```

### Global Click Handler

Handle all clicks regardless of target:

```ts
import { onGlobalClick } from 'spark-tui'

// Close dropdown when clicking outside
const unsub = onGlobalClick((event) => {
  if (!isPointInBounds(event.x, event.y, dropdownBounds)) {
    closeDropdown()
  }
})
```

### Global Mouse Handler

Receive all mouse events:

```ts
import { onGlobalMouse } from 'spark-tui'

const unsub = onGlobalMouse((event) => {
  switch (event.type) {
    case EventType.MouseDown:
      console.log('Mouse down')
      break
    case EventType.MouseUp:
      console.log('Mouse up')
      break
    case EventType.MouseMove:
      console.log(`Move to (${event.x}, ${event.y})`)
      break
  }
})
```

### Global Move Handler

Track mouse movement:

```ts
import { onGlobalMove } from 'spark-tui'

const unsub = onGlobalMove((x, y) => {
  updateCrosshair(x, y)
})
```

### Global Scroll Handler

Handle scroll events globally:

```ts
import { onGlobalScroll } from 'spark-tui'

const unsub = onGlobalScroll((event) => {
  if (event.deltaY < 0) {
    scrollUp()
  } else {
    scrollDown()
  }
})
```

## Component Props

The `box`, `text`, and `input` primitives accept mouse event props:

### MouseProps Interface

```ts
interface MouseProps {
  /** Called on mouse down. Return true to consume event. */
  onMouseDown?: (event: MouseEvent) => void | boolean
  /** Called on mouse up. Return true to consume event. */
  onMouseUp?: (event: MouseEvent) => void | boolean
  /** Called on click. Return true to consume event. */
  onClick?: (event: MouseEvent) => void | boolean
  /** Called when mouse enters this component */
  onMouseEnter?: (event: MouseEvent) => void
  /** Called when mouse leaves this component */
  onMouseLeave?: (event: MouseEvent) => void
  /** Called on scroll over this component. */
  onScroll?: (event: ScrollEvent) => void
}
```

### Using Props in Components

```ts
import { box, text } from 'spark-tui'
import { signal } from '@rlabs-inc/signals'

const isHovered = signal(false)
const clickCount = signal(0)

box({
  width: 20,
  height: 3,
  border: 1,
  bg: () => isHovered.value ? { r: 50, g: 50, b: 100 } : null,
  onClick: (event) => {
    clickCount.value++
    console.log(`Clicked with ${getButtonName(event)} button`)
  },
  onMouseEnter: () => {
    isHovered.value = true
  },
  onMouseLeave: () => {
    isHovered.value = false
  },
  children: () => {
    text({ content: () => `Clicks: ${clickCount.value}` })
  },
})
```

## Event Propagation

Mouse events bubble from the target component up to the root:

1. Global handlers are called first
2. The target component's handler is called
3. Parent handlers are called (bubbling up)
4. Events propagate through the component tree based on hit testing

```ts
// Outer box receives bubbled events
box({
  onClick: (event) => {
    console.log('Outer clicked (bubbled)')
  },
  children: () => {
    // Inner box receives direct clicks
    box({
      onClick: (event) => {
        console.log('Inner clicked')
        // Event continues to bubble to outer box
      },
    })
  },
})
```

## Examples

### Interactive Button

```ts
import { box, text } from 'spark-tui'
import { signal } from '@rlabs-inc/signals'

function Button(label: string, onClick: () => void) {
  const isHovered = signal(false)
  const isPressed = signal(false)

  box({
    width: label.length + 4,
    height: 3,
    border: 1,
    justifyContent: 'center',
    alignItems: 'center',
    bg: () => {
      if (isPressed.value) return { r: 30, g: 30, b: 80 }
      if (isHovered.value) return { r: 60, g: 60, b: 120 }
      return { r: 40, g: 40, b: 100 }
    },
    onMouseEnter: () => { isHovered.value = true },
    onMouseLeave: () => {
      isHovered.value = false
      isPressed.value = false
    },
    onMouseDown: () => { isPressed.value = true },
    onMouseUp: () => { isPressed.value = false },
    onClick: () => onClick(),
    children: () => {
      text({ content: label })
    },
  })
}

Button('Click Me', () => console.log('Button clicked!'))
```

### Scrollable List

```ts
import { box, each, text } from 'spark-tui'
import { signal, derived } from '@rlabs-inc/signals'

const items = Array.from({ length: 100 }, (_, i) => `Item ${i + 1}`)
const scrollOffset = signal(0)
const visibleCount = 10

const visibleItems = derived(() => {
  const start = scrollOffset.value
  return items.slice(start, start + visibleCount)
})

box({
  width: 30,
  height: visibleCount + 2, // +2 for border
  border: 1,
  overflow: 'hidden',
  onScroll: (event) => {
    const newOffset = scrollOffset.value + Math.sign(event.deltaY)
    scrollOffset.value = Math.max(0, Math.min(items.length - visibleCount, newOffset))
  },
  children: () => {
    each(visibleItems, (item, index) => {
      text({ content: item })
    })
  },
})
```

### Drag and Drop

```ts
import { box, text } from 'spark-tui'
import { signal } from '@rlabs-inc/signals'
import { onGlobalMouse, mouseX, mouseY, isMouseDown } from 'spark-tui'

const dragX = signal(5)
const dragY = signal(5)
let isDragging = false
let offsetX = 0
let offsetY = 0

// Draggable box
box({
  width: 10,
  height: 5,
  border: 1,
  // Position using margin (or absolute positioning if supported)
  marginLeft: dragX,
  marginTop: dragY,
  onMouseDown: (event) => {
    isDragging = true
    offsetX = event.x - dragX.value
    offsetY = event.y - dragY.value
  },
  children: () => {
    text({ content: 'Drag me' })
  },
})

// Global handler for drag tracking
onGlobalMouse((event) => {
  if (event.type === EventType.MouseUp) {
    isDragging = false
  }
  if (event.type === EventType.MouseMove && isDragging) {
    dragX.value = event.x - offsetX
    dragY.value = event.y - offsetY
  }
})
```

### Hover Tooltip

```ts
import { box, text, show } from 'spark-tui'
import { signal } from '@rlabs-inc/signals'

const showTooltip = signal(false)
const tooltipX = signal(0)
const tooltipY = signal(0)

// Target element
box({
  width: 10,
  height: 3,
  border: 1,
  onMouseEnter: (event) => {
    tooltipX.value = event.x + 2
    tooltipY.value = event.y + 1
    showTooltip.value = true
  },
  onMouseLeave: () => {
    showTooltip.value = false
  },
  children: () => {
    text({ content: 'Hover me' })
  },
})

// Tooltip (would need absolute positioning support)
show(showTooltip, () => {
  box({
    marginLeft: tooltipX,
    marginTop: tooltipY,
    padding: 1,
    bg: { r: 40, g: 40, b: 40 },
    children: () => {
      text({ content: 'This is a tooltip!' })
    },
  })
})
```

### Click Outside to Close

```ts
import { box, text, show, when } from 'spark-tui'
import { signal } from '@rlabs-inc/signals'
import { onGlobalClick, isPointInBounds } from 'spark-tui'

const isOpen = signal(false)
const menuBounds = { x: 10, y: 5, width: 20, height: 10 }

// Menu trigger
box({
  onClick: () => { isOpen.value = true },
  children: () => {
    text({ content: 'Open Menu' })
  },
})

// Close on outside click
onGlobalClick((event) => {
  if (isOpen.value && !isPointInBounds(event.x, event.y, menuBounds)) {
    isOpen.value = false
  }
})

// Menu
show(isOpen, () => {
  box({
    width: menuBounds.width,
    height: menuBounds.height,
    marginLeft: menuBounds.x,
    marginTop: menuBounds.y,
    border: 1,
    children: () => {
      text({ content: 'Menu Item 1' })
      text({ content: 'Menu Item 2' })
      text({ content: 'Menu Item 3' })
    },
  })
})
```

### Right-Click Context Menu

```ts
import { box, text, show } from 'spark-tui'
import { signal } from '@rlabs-inc/signals'

const showContext = signal(false)
const contextX = signal(0)
const contextY = signal(0)

box({
  width: '100%',
  height: '100%',
  onMouseDown: (event) => {
    if (isRightButton(event)) {
      contextX.value = event.x
      contextY.value = event.y
      showContext.value = true
    } else {
      showContext.value = false
    }
  },
  children: () => {
    text({ content: 'Right-click anywhere for context menu' })

    show(showContext, () => {
      box({
        marginLeft: contextX,
        marginTop: contextY,
        width: 15,
        border: 1,
        bg: { r: 30, g: 30, b: 30 },
        children: () => {
          box({
            onClick: () => { copy(); showContext.value = false },
            onMouseEnter: () => setHighlight(0),
            children: () => text({ content: 'Copy' }),
          })
          box({
            onClick: () => { paste(); showContext.value = false },
            onMouseEnter: () => setHighlight(1),
            children: () => text({ content: 'Paste' }),
          })
          box({
            onClick: () => { del(); showContext.value = false },
            onMouseEnter: () => setHighlight(2),
            children: () => text({ content: 'Delete' }),
          })
        },
      })
    })
  },
})
```

## Focus on Click

When a focusable component is clicked, it automatically receives focus:

```ts
import { box, input } from 'spark-tui'
import { signal } from '@rlabs-inc/signals'

const value = signal('')

// Clicking the input focuses it automatically
input({
  value,
  // focusable is implicitly true for input
  onClick: (event) => {
    console.log('Input clicked and focused')
  },
})

// Focusable boxes also focus on click
box({
  focusable: true,
  onClick: (event) => {
    // Component receives focus when clicked
    console.log('Box clicked and focused')
  },
})
```

## See Also

- [Keyboard Events](./keyboard.md) - Keyboard input handling
- [Focus Management](../concepts/focus.md) - Managing component focus
- [Box Component](../components/box.md) - Container with mouse support
- [Input Component](../components/input.md) - Text input with mouse handling
