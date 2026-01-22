# spark-tui Component Primitives

This document details the component primitives and their implementation patterns from the TypeScript reference.

## Component Lifecycle

Every component follows this lifecycle:

```typescript
export function box(props: BoxProps = {}): Cleanup {
  // 1. ALLOCATE - Get index from registry
  const index = allocateIndex(props.id)

  // 2. CREATE FLEXNODE - Persistent layout object
  const flexNode = createFlexNode(index)

  // 3. TRACK - Push to lifecycle stack for hooks
  pushCurrentComponent(index)

  // 4. CORE SETUP - Type, parent, visibility
  core.componentType[index] = ComponentType.BOX
  core.parentIndex.setSource(index, getCurrentParentIndex())

  // 5. BIND PROPS - To FlexNode slots and arrays
  if (props.width !== undefined) flexNode.width.source = props.width
  // ... more bindings

  // 6. REGISTER HANDLERS - Keyboard, mouse, focus callbacks
  const unsubKeyboard = onFocused(index, handleKeyEvent)
  const unsubMouse = onMouseComponent(index, { onClick, ... })

  // 7. RENDER CHILDREN - With this component as parent context
  if (props.children) {
    pushParentContext(index)
    try {
      props.children()
    } finally {
      popParentContext()
    }
  }

  // 8. COMPLETE - Run mount callbacks
  popCurrentComponent()
  runMountCallbacks(index)

  // 9. RETURN CLEANUP - Unsubscribe and release
  const cleanup = () => {
    unsubMouse?.()
    unsubKeyboard?.()
    releaseIndex(index)  // Recursive! Cleans up children
  }

  // Auto-register with active scope
  const scope = getActiveScope()
  if (scope) scope.cleanups.push(cleanup)

  return cleanup
}
```

## Box (Container)

The fundamental container component. Can have children, borders, backgrounds, and handles events.

### Props Interface

```typescript
interface BoxProps {
  // Identity
  id?: string

  // Visibility
  visible?: boolean | Signal<boolean> | (() => boolean)

  // Layout - Container
  flexDirection?: 'column' | 'row' | 'column-reverse' | 'row-reverse'
  flexWrap?: 'nowrap' | 'wrap' | 'wrap-reverse'
  justifyContent?: 'flex-start' | 'center' | 'flex-end' | 'space-between' | 'space-around' | 'space-evenly'
  alignItems?: 'stretch' | 'flex-start' | 'center' | 'flex-end' | 'baseline'

  // Layout - Item
  grow?: number
  shrink?: number
  flexBasis?: Dimension
  alignSelf?: 'auto' | 'stretch' | 'flex-start' | 'center' | 'flex-end' | 'baseline'

  // Dimensions
  width?: Dimension
  height?: Dimension
  minWidth?: Dimension
  maxWidth?: Dimension
  minHeight?: Dimension
  maxHeight?: Dimension

  // Spacing
  margin?: number
  marginTop?: number
  marginRight?: number
  marginBottom?: number
  marginLeft?: number
  padding?: number
  paddingTop?: number
  paddingRight?: number
  paddingBottom?: number
  paddingLeft?: number
  gap?: number

  // Borders
  border?: number  // Border style enum (0=none, 1=single, 2=double, etc.)
  borderTop?: number
  borderRight?: number
  borderBottom?: number
  borderLeft?: number
  borderColor?: RGBA

  // Colors
  fg?: RGBA
  bg?: RGBA
  opacity?: number

  // Theming
  variant?: Variant

  // Scrolling
  overflow?: 'visible' | 'hidden' | 'scroll' | 'auto'

  // Interaction
  focusable?: boolean
  tabIndex?: number
  zIndex?: number

  // Callbacks
  onFocus?: () => void
  onBlur?: () => void
  onKey?: (event: KeyboardEvent) => boolean
  onMouseDown?: (event: MouseEvent) => void | boolean
  onMouseUp?: (event: MouseEvent) => void | boolean
  onClick?: (event: MouseEvent) => void | boolean
  onMouseEnter?: (event: MouseEvent) => void
  onMouseLeave?: (event: MouseEvent) => void
  onScroll?: (event: MouseEvent) => void | boolean

  // Children
  children?: () => void
}
```

### Dimension Type

```typescript
type Dimension = number | string
// 0        → 'auto' (content-determined)
// 10       → 10 cells (pixels in terminal units)
// '50%'    → 50% of parent
// '100%'   → Full parent width/height
```

### Border Styles

```typescript
enum BorderStyle {
  None = 0,
  Single = 1,        // ┌─┐│└─┘
  Double = 2,        // ╔═╗║╚═╝
  Rounded = 3,       // ╭─╮│╰─╯
  Heavy = 4,         // ┏━┓┃┗━┛
  DoubleSingle = 5,  // ╓─╖║╙─╜
  SingleDouble = 6,  // ╒═╕│╘═╛
  Classic = 7,       // +--+|+--+
  Dashed = 8,        // ┌╌┐╎└╌┘
  Dotted = 9,        // ┌┄┐┆└┄┘
  Block = 10,        // ████████
}
```

### Implementation Details

```typescript
// Focusable handling - auto-focusable for scrollable containers
const shouldBeFocusable = props.focusable || (props.overflow === 'scroll' && props.focusable !== false)
if (shouldBeFocusable) {
  interaction.focusable.setSource(index, 1)
  if (props.tabIndex !== undefined) {
    interaction.tabIndex.setSource(index, props.tabIndex)
  }
}

// Click-to-focus pattern
unsubMouse = onMouseComponent(index, {
  onClick: (event) => {
    if (shouldBeFocusable) {
      focusComponent(index)  // Focus on click
    }
    return props.onClick?.(event)
  },
  // ... other handlers
})

// Variant colors - reactive to theme changes
if (props.variant && props.variant !== 'default') {
  const variant = props.variant
  visual.fgColor.setSource(index, () => getVariantStyle(variant).fg)
  visual.bgColor.setSource(index, () => getVariantStyle(variant).bg)
  visual.borderColor.setSource(index, () => getVariantStyle(variant).border)
}
```

## Text (Display)

Pure display component for text content. Cannot have children.

### Props Interface

```typescript
interface TextProps {
  // Identity
  id?: string

  // Content - REQUIRED (the whole point!)
  content: string | number | Signal<string | number> | (() => string | number)

  // Visibility
  visible?: boolean | Signal<boolean> | (() => boolean)

  // Text styling
  attrs?: number  // Attr.BOLD | Attr.ITALIC, etc.
  align?: 'left' | 'center' | 'right'
  wrap?: 'wrap' | 'nowrap' | 'truncate'

  // Layout - Item (text is always a flex item, never container)
  grow?: number
  shrink?: number
  flexBasis?: Dimension
  alignSelf?: 'auto' | 'stretch' | 'flex-start' | 'center' | 'flex-end'

  // Dimensions
  width?: Dimension
  height?: Dimension
  minWidth?: Dimension
  maxWidth?: Dimension
  minHeight?: Dimension
  maxHeight?: Dimension

  // Spacing
  padding?: number
  paddingTop?: number
  paddingRight?: number
  paddingBottom?: number
  paddingLeft?: number

  // Colors
  fg?: RGBA
  bg?: RGBA
  opacity?: number

  // Theming
  variant?: Variant

  // Mouse events (text can be clickable)
  onMouseDown?: (event: MouseEvent) => void | boolean
  onMouseUp?: (event: MouseEvent) => void | boolean
  onClick?: (event: MouseEvent) => void | boolean
  onMouseEnter?: (event: MouseEvent) => void
  onMouseLeave?: (event: MouseEvent) => void
  onScroll?: (event: MouseEvent) => void | boolean
}
```

### Text Attributes

```typescript
enum Attr {
  NONE        = 0,
  BOLD        = 1 << 0,  // 1
  DIM         = 1 << 1,  // 2
  ITALIC      = 1 << 2,  // 4
  UNDERLINE   = 1 << 3,  // 8
  BLINK       = 1 << 4,  // 16
  INVERSE     = 1 << 5,  // 32
  HIDDEN      = 1 << 6,  // 64
  STRIKETHROUGH = 1 << 7,  // 128
}

// Usage: attrs: Attr.BOLD | Attr.UNDERLINE
```

### Content Conversion

Numbers are automatically converted to strings:

```typescript
function contentToStringSource(content: TextProps['content']): string | (() => string) {
  // Getter function
  if (typeof content === 'function') {
    return () => String(content())
  }
  // Signal/derived with .value
  if (content !== null && typeof content === 'object' && 'value' in content) {
    return () => String((content as { value: string | number }).value)
  }
  // Static value
  return String(content)
}
```

### Wrap Modes

```typescript
// textWrap array values
0 = nowrap    // Single line, may overflow
1 = wrap      // Wrap at container width
2 = truncate  // Single line with ellipsis (...)
```

## Input (Text Entry)

Single-line text input with full cursor support.

### Props Interface

```typescript
interface InputProps {
  // Identity
  id?: string

  // Value - REQUIRED (two-way binding)
  value: WritableSignal<string>  // signal('') - mutable!

  // Visibility
  visible?: boolean | Signal<boolean> | (() => boolean)

  // Input-specific
  placeholder?: string
  password?: boolean
  maskChar?: string  // Default: '•'
  maxLength?: number
  autoFocus?: boolean

  // Cursor configuration
  cursor?: {
    style?: 'block' | 'bar' | 'underline'
    char?: string  // Custom character
    blink?: boolean | BlinkConfig
  }

  // Dimensions
  width?: Dimension
  height?: Dimension
  minWidth?: Dimension
  maxWidth?: Dimension
  minHeight?: Dimension
  maxHeight?: Dimension

  // Spacing
  padding?: number
  paddingTop?: number
  paddingRight?: number
  paddingBottom?: number
  paddingLeft?: number
  margin?: number
  marginTop?: number
  marginRight?: number
  marginBottom?: number
  marginLeft?: number

  // Borders
  border?: number
  borderTop?: number
  borderRight?: number
  borderBottom?: number
  borderLeft?: number
  borderColor?: RGBA

  // Colors
  fg?: RGBA
  bg?: RGBA
  opacity?: number

  // Theming
  variant?: Variant

  // Callbacks
  onChange?: (value: string) => void
  onSubmit?: (value: string) => void
  onCancel?: () => void
  onFocus?: () => void
  onBlur?: () => void

  // Mouse events
  onMouseDown?: (event: MouseEvent) => void | boolean
  onMouseUp?: (event: MouseEvent) => void | boolean
  onClick?: (event: MouseEvent) => void | boolean
  onMouseEnter?: (event: MouseEvent) => void
  onMouseLeave?: (event: MouseEvent) => void
  onScroll?: (event: MouseEvent) => void | boolean

  // Focus
  tabIndex?: number
}

interface BlinkConfig {
  enabled?: boolean
  fps?: number      // Default: 2 (500ms cycle)
  altChar?: string  // Character for "off" phase
}
```

### Keyboard Handling

```typescript
const handleKeyEvent = (event: KeyboardEvent): boolean => {
  const val = getValue()
  const pos = Math.min(cursorPos.value, val.length)  // Clamp to valid range

  switch (event.key) {
    // Navigation
    case 'ArrowLeft':
      if (pos > 0) cursorPos.value = pos - 1
      return true

    case 'ArrowRight':
      if (pos < val.length) cursorPos.value = pos + 1
      return true

    case 'Home':
      cursorPos.value = 0
      return true

    case 'End':
      cursorPos.value = val.length
      return true

    // Deletion
    case 'Backspace':
      if (pos > 0) {
        const newVal = val.slice(0, pos - 1) + val.slice(pos)
        setValue(newVal)
        cursorPos.value = pos - 1
        props.onChange?.(newVal)
      }
      return true

    case 'Delete':
      if (pos < val.length) {
        const newVal = val.slice(0, pos) + val.slice(pos + 1)
        setValue(newVal)
        props.onChange?.(newVal)
      }
      return true

    // Submission
    case 'Enter':
      props.onSubmit?.(val)
      return true

    case 'Escape':
      props.onCancel?.()
      return true

    // Character input
    default:
      if (event.key.length === 1 && !event.modifiers.ctrl && !event.modifiers.alt) {
        if (maxLen > 0 && val.length >= maxLen) return true
        const newVal = val.slice(0, pos) + event.key + val.slice(pos)
        setValue(newVal)
        cursorPos.value = pos + 1
        props.onChange?.(newVal)
        return true
      }
      return false
  }
}
```

### Display Text Handling

```typescript
// Display text getter - reactive!
const getDisplayText = () => {
  const val = getValue()
  if (val.length === 0 && props.placeholder) {
    return props.placeholder
  }
  return props.password ? maskChar.repeat(val.length) : val
}

textArrays.textContent.setSource(index, getDisplayText)
```

### Cursor System

```typescript
// Create drawn cursor
const cursor = createCursor(index, {
  style: cursorConfig.style ?? 'block',
  char: cursorConfig.char,
  blink: blinkEnabled,
  fps: blinkFps,
  altChar: blinkConfig?.altChar,
})

// Cursor position - clamped to value length
interaction.cursorPosition.setSource(index, () => Math.min(cursorPos.value, getValue().length))

// Cleanup
cleanup = () => {
  cursor.dispose()
  // ...
}
```

## Control Flow Components

### show(condition, render, else)

Conditional rendering:

```typescript
function show<T>(
  condition: T | Signal<T> | (() => T),
  render: (value: T) => void,
  elseRender?: () => void
): Cleanup {
  const scope = effectScope()

  effect(() => {
    scope.dispose()  // Clean up previous render

    const value = unwrap(condition)
    if (value) {
      scope.run(() => render(value))
    } else if (elseRender) {
      scope.run(() => elseRender())
    }
  })

  return () => scope.dispose()
}
```

### each(items, render, options)

List rendering with fine-grained updates:

```typescript
function each<T>(
  items: T[] | Signal<T[]> | (() => T[]),
  render: (item: T, index: number) => void,
  options?: { key?: (item: T) => string | number }
): Cleanup {
  const scopes = new Map<string | number, EffectScope>()

  effect(() => {
    const list = unwrap(items)
    const seenKeys = new Set<string | number>()

    list.forEach((item, i) => {
      const key = options?.key ? options.key(item) : i
      seenKeys.add(key)

      if (!scopes.has(key)) {
        const scope = effectScope()
        scope.run(() => render(item, i))
        scopes.set(key, scope)
      }
    })

    // Clean up removed items
    for (const [key, scope] of scopes) {
      if (!seenKeys.has(key)) {
        scope.dispose()
        scopes.delete(key)
      }
    }
  })

  return () => {
    for (const scope of scopes.values()) {
      scope.dispose()
    }
    scopes.clear()
  }
}
```

### when(promise, handlers)

Async handling:

```typescript
function when<T>(
  promise: Promise<T> | Signal<Promise<T>> | (() => Promise<T>),
  handlers: {
    pending?: () => void
    then?: (value: T) => void
    catch?: (error: Error) => void
  }
): Cleanup {
  const scope = effectScope()

  effect(() => {
    scope.dispose()
    const p = unwrap(promise)

    // Show pending state
    if (handlers.pending) {
      scope.run(() => handlers.pending!())
    }

    p.then(value => {
      scope.dispose()
      if (handlers.then) {
        scope.run(() => handlers.then!(value))
      }
    }).catch(error => {
      scope.dispose()
      if (handlers.catch) {
        scope.run(() => handlers.catch!(error))
      }
    })
  })

  return () => scope.dispose()
}
```

## Usage Patterns

### Reactive Props

```typescript
const width = signal(40)
const visible = signal(true)

box({
  width,        // Stays reactive!
  visible,      // Stays reactive!
  height: 10,   // Static value
  children: () => {
    text({ content: 'Hello' })
  }
})

// Later changes update UI automatically
width.set(60)
visible.set(false)
```

### Computed Content

```typescript
const count = signal(0)

text({
  content: () => `Count: ${count.value}`,  // Computed getter
})

// Or with derived
const message = derived(() => `Count: ${count.value}`)
text({ content: message })
```

### Two-Way Binding (Input)

```typescript
const name = signal('')

input({
  value: name,  // Two-way binding
  placeholder: 'Enter name...',
  onSubmit: (val) => console.log('Submitted:', val),
})

// Read current value
console.log(name.value)

// Set programmatically
name.set('John')
```

### Variant Theming

```typescript
box({
  variant: 'primary',  // Uses theme.primary colors
  border: 1,
  children: () => {
    text({ content: 'Primary Button' })
  }
})

// When theme changes, all variant components update automatically
setTheme('dracula')
```

### List with Keys

```typescript
const items = signal([
  { id: 1, name: 'Item 1' },
  { id: 2, name: 'Item 2' },
])

each(
  items,
  (item) => {
    box({
      children: () => {
        text({ content: item.name })
      }
    })
  },
  { key: item => item.id }  // Stable identity for updates
)
```
