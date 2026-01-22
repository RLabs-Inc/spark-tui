# spark-tui Systems

This document details the runtime systems that power the TUI framework.

## Theme System

The theme system provides reactive, semantic colors with full terminal integration.

### Color Types

```typescript
type ThemeColor = null | number | string

// null       → Terminal default (respects user's terminal theme)
// 0-15       → ANSI 16-color (respects terminal palette)
// 16-255     → Extended 256-color palette
// >255       → RGB as 0xRRGGBB
// string     → CSS color (hex, rgb, oklch, etc.)
```

### Theme State

```typescript
// Reactive state object
export const theme = state({
  // Main palette (ANSI defaults respect terminal theme)
  primary: 12,      // bright blue
  secondary: 13,    // bright magenta
  tertiary: 14,     // bright cyan
  accent: 11,       // bright yellow

  // Semantic colors
  success: 2,       // green
  warning: 3,       // yellow
  error: 1,         // red
  info: 6,          // cyan

  // Text colors
  text: null,       // terminal default
  textMuted: 8,     // bright black/gray
  textDim: 8,
  textDisabled: 8,
  textBright: 15,   // bright white

  // Background colors
  background: null,     // terminal default
  backgroundMuted: null,
  surface: null,
  overlay: null,

  // Border colors
  border: 7,            // white
  borderFocus: 12,      // primary color

  name: 'terminal',
  description: 'Uses terminal default colors',
})
```

### Theme Presets (13 total)

```typescript
export const themes = {
  terminal,      // ANSI colors (default)
  dracula,       // Dark with vivid colors (OKLCH)
  nord,          // Arctic, bluish
  monokai,       // Vibrant syntax-highlighting
  solarized,     // Precision color scheme
  catppuccin,    // Soothing pastel
  gruvbox,       // Retro groove
  tokyoNight,    // Tokyo city lights
  oneDark,       // Atom's iconic theme
  rosePine,      // Natural pine vibes
  kanagawa,      // Hokusai wave painting
  everforest,    // Comfortable green-tinted
  nightOwl,      // Accessibility-focused
}
```

### Applying Themes

```typescript
import { setTheme, getThemeNames } from 'spark-tui/theme'

// By name
setTheme('dracula')

// Custom partial
setTheme({
  primary: 0xFF5733,
  secondary: 'oklch(0.7 0.15 200)',
})

// Get available themes
const names = getThemeNames()  // ['terminal', 'dracula', ...]
```

### Easy Theme Access (t.*)

```typescript
import { t } from 'spark-tui/theme'

box({
  borderColor: t.primary,  // Reactive! Updates on theme change
  fg: t.text,
  bg: t.surface,
})

// Each is a derived that resolves the theme color
t.primary     // → derived(() => resolveColor(theme.primary))
t.secondary
t.success
t.error
// ... all 18 properties
```

### Variant Styles (15 variants)

```typescript
type Variant =
  | 'default'
  | 'primary' | 'secondary' | 'tertiary' | 'accent'
  | 'success' | 'warning' | 'error' | 'info'
  | 'muted' | 'surface' | 'elevated'
  | 'ghost' | 'outline'

// Usage
box({ variant: 'primary' })  // Primary background, contrast text

// Each variant defines fg, bg, border, borderFocus
interface VariantStyle {
  fg: RGBA
  bg: RGBA
  border: RGBA
  borderFocus: RGBA
}
```

### Contrast Calculation

For RGB themes, automatic WCAG AA (4.5:1) contrast:

```typescript
function getContrastFg(desiredFg: RGBA, bg: RGBA): RGBA {
  // ANSI colors: trust terminal
  if (isAnsiColor(bg) || isAnsiColor(desiredFg)) {
    return desiredFg
  }
  // RGB colors: calculate OKLCH contrast
  return adjustLightnessForContrast(desiredFg, bg, 4.5)
}
```

### ANSI Escape Codes

```typescript
// Generate ANSI codes for rendering
toAnsiFg(theme.primary)  // '\x1b[38;2;...m' or '\x1b[94m'
toAnsiBg(theme.error)    // '\x1b[48;2;...m' or '\x1b[41m'

// Terminal default
toAnsiFg(null)  // '\x1b[39m' (reset fg)
toAnsiBg(null)  // '\x1b[49m' (reset bg)
```

## Focus System

Manages keyboard navigation and focus state.

### Core State

```typescript
export const focusedIndex = signal<number>(-1)  // -1 = nothing focused

// Derived queries
export const hasFocus = derived(() => focusedIndex.value >= 0)
export const focusableIndices = derived(getFocusableIndices)
```

### Focus Callbacks

Callbacks fire at the source (state change) not via effects:

```typescript
// Multiple registrations per index supported
export function registerFocusCallbacks(index: number, callbacks: FocusCallbacks): () => void {
  let list = focusCallbackRegistry.get(index)
  if (!list) {
    list = []
    focusCallbackRegistry.set(index, list)
  }
  list.push(callbacks)

  return () => { /* remove from list */ }
}

// Internal: fires callbacks when focus changes
function setFocusWithCallbacks(newIndex: number): void {
  const oldIndex = focusedIndex.value
  if (oldIndex === newIndex) return

  // Fire onBlur for ALL callbacks on old focus
  if (oldIndex >= 0) {
    const callbacks = focusCallbackRegistry.get(oldIndex)
    if (callbacks) {
      for (const cb of callbacks) cb.onBlur?.()
    }
  }

  focusedIndex.value = newIndex

  // Fire onFocus for ALL callbacks on new focus
  if (newIndex >= 0) {
    const callbacks = focusCallbackRegistry.get(newIndex)
    if (callbacks) {
      for (const cb of callbacks) cb.onFocus?.()
    }
  }
}
```

### Tab Navigation

```typescript
// Get focusables sorted by tabIndex
function getFocusableIndices(): number[] {
  const result: number[] = []

  for (const i of getAllocatedIndices()) {
    const isFocusable = unwrap(focusable[i])
    const isVisible = unwrap(visible[i])
    const isActuallyVisible = isVisible !== 0 && isVisible !== false

    if (isFocusable && isActuallyVisible) {
      result.push(i)
    }
  }

  // Sort by tabIndex (same tabIndex → allocation order)
  result.sort((a, b) => {
    const tabA = unwrap(tabIndex[a]) ?? 0
    const tabB = unwrap(tabIndex[b]) ?? 0
    if (tabA !== tabB) return tabA < tabB ? -1 : 1
    return a - b
  })

  return result
}

// Navigation with wrap-around
function findNextFocusable(fromIndex: number, direction: 1 | -1): number {
  const focusables = getFocusableIndices()
  if (focusables.length === 0) return -1

  const currentPos = focusables.indexOf(fromIndex)
  if (currentPos === -1) {
    return direction === 1 ? focusables[0]! : focusables[focusables.length - 1]!
  }

  const nextPos = (currentPos + direction + focusables.length) % focusables.length
  return focusables[nextPos]!
}

export function focusNext(): boolean {
  const next = findNextFocusable(focusedIndex.value, 1)
  if (next !== -1 && next !== focusedIndex.value) {
    saveFocusToHistory()
    setFocusWithCallbacks(next)
    return true
  }
  return false
}
```

### Focus Trap (Modals)

```typescript
const focusTrapStack: number[] = []

export function pushFocusTrap(containerIndex: number): void {
  focusTrapStack.push(containerIndex)
}

export function popFocusTrap(): number | undefined {
  return focusTrapStack.pop()
}

export function isFocusTrapped(): boolean {
  return focusTrapStack.length > 0
}
```

### Focus History

```typescript
interface FocusHistoryEntry {
  index: number
  id: string | undefined  // For recycling detection
}

const focusHistory: FocusHistoryEntry[] = []
const MAX_HISTORY = 10

export function saveFocusToHistory(): void {
  const current = focusedIndex.value
  if (current >= 0) {
    const id = getId(current)
    focusHistory.push({ index: current, id })
    if (focusHistory.length > MAX_HISTORY) {
      focusHistory.shift()
    }
  }
}

export function restoreFocusFromHistory(): boolean {
  while (focusHistory.length > 0) {
    const entry = focusHistory.pop()!
    // Verify index wasn't recycled for different component
    if (getId(entry.index) !== entry.id) continue
    // Check still valid and focusable
    if (unwrap(focusable[entry.index]) && isVisible(entry.index)) {
      setFocusWithCallbacks(entry.index)
      return true
    }
  }
  return false
}
```

## Keyboard System

Handles keyboard events with multiple dispatch levels.

### Event Types

```typescript
interface Modifiers {
  ctrl: boolean
  alt: boolean
  shift: boolean
  meta: boolean
}

type KeyState = 'press' | 'repeat' | 'release'

interface KeyboardEvent {
  key: string           // 'a', 'Enter', 'ArrowUp', etc.
  modifiers: Modifiers
  state: KeyState
  raw?: string          // Raw escape sequence
}

type KeyHandler = (event: KeyboardEvent) => void | boolean
```

### Handler Registry

```typescript
// Three levels of handlers
const globalHandlers = new Set<KeyHandler>()              // keyboard.on()
const keyHandlers = new Map<string, Set<() => boolean>>() // keyboard.onKey('Enter', ...)
const focusedHandlers = new Map<number, Set<KeyHandler>>()// keyboard.onFocused(index, ...)
```

### Dispatch Priority

1. **Global shortcuts** (Ctrl+C, Tab) - handled by global-keys.ts
2. **Focused component handlers** - component-specific input handling
3. **Key-specific handlers** - onKey('Enter', ...)
4. **Global handlers** - on(event => ...)
5. **Framework defaults** - arrow scrolling, Page Up/Down

```typescript
function handleKeyboardEvent(event: KeyboardEvent): void {
  // 1. Global shortcuts
  if (event.key === 'c' && event.modifiers.ctrl) {
    cleanup()
    process.exit(0)
  }

  // Skip non-press events for shortcuts
  if (event.state !== 'press') {
    dispatchFocused(focusedIndex.value, event)
    dispatchKeyboard(event)
    return
  }

  // 2. Tab navigation
  if (event.key === 'Tab' && !event.modifiers.ctrl && !event.modifiers.alt) {
    event.modifiers.shift ? focusPrevious() : focusNext()
    return
  }

  // 3. Focused component handlers
  if (dispatchFocused(focusedIndex.value, event)) return

  // 4. User handlers (keyboard.onKey, keyboard.on)
  if (dispatchKeyboard(event)) return

  // 5. Framework defaults (scrolling)
  if (event.key === 'ArrowUp' && handleArrowScroll('up')) return
  if (event.key === 'ArrowDown' && handleArrowScroll('down')) return
  // ... etc
}
```

## Mouse System

Handles mouse events with HitGrid for coordinate lookup.

### HitGrid

O(1) coordinate → component lookup:

```typescript
export class HitGrid {
  private grid: Int16Array
  private _width: number
  private _height: number

  constructor(width: number, height: number) {
    this._width = width
    this._height = height
    this.grid = new Int16Array(width * height).fill(-1)
  }

  get(x: number, y: number): number {
    if (x < 0 || x >= this._width || y < 0 || y >= this._height) return -1
    return this.grid[y * this._width + x]!
  }

  fillRect(x: number, y: number, w: number, h: number, componentIndex: number): void {
    // Fill rectangle with component index
  }

  clear(): void {
    this.grid.fill(-1)
  }
}

// Global instance
export const hitGrid = new HitGrid(80, 24)
```

### Mouse State

```typescript
export const lastMouseEvent = signal<MouseEvent | null>(null)
export const mouseX = signal(0)
export const mouseY = signal(0)
export const isMouseDown = signal(false)
```

### Event Dispatch

```typescript
export function dispatch(event: MouseEvent): boolean {
  // Fill componentIndex from HitGrid
  event.componentIndex = hitGrid.get(event.x, event.y)

  // Update reactive state
  lastMouseEvent.value = event
  mouseX.value = event.x
  mouseY.value = event.y

  // Handle hover (enter/leave)
  if (event.componentIndex !== hoveredComponent) {
    // Fire onMouseLeave for previous
    if (hoveredComponent >= 0) {
      const prevHandlers = componentHandlers.get(hoveredComponent)
      prevHandlers?.onMouseLeave?.(event)
      interaction.hovered.setValue(hoveredComponent, 0)
    }
    // Fire onMouseEnter for new
    if (event.componentIndex >= 0) {
      const handlers = componentHandlers.get(event.componentIndex)
      handlers?.onMouseEnter?.(event)
      interaction.hovered.setValue(event.componentIndex, 1)
    }
    hoveredComponent = event.componentIndex
  }

  // Handle click detection (press + release on same component)
  if (event.action === 'down') {
    pressedComponent = event.componentIndex
    // ...
  }
  if (event.action === 'up') {
    if (pressedComponent === event.componentIndex) {
      // Fire onClick!
    }
    pressedComponent = -1
  }
}
```

### Mouse Tracking (ANSI)

```typescript
const ENABLE_MOUSE = '\x1b[?1000h\x1b[?1002h\x1b[?1003h\x1b[?1006h'
const DISABLE_MOUSE = '\x1b[?1000l\x1b[?1002l\x1b[?1003l\x1b[?1006l'

export function enableTracking(): void {
  if (trackingEnabled) return
  trackingEnabled = true
  process.stdout.write(ENABLE_MOUSE)
}
```

## Scroll System

Manages scrollable containers and scroll behavior.

### Scroll State

```typescript
// User state (interaction arrays)
scrollOffsetX[index]  // Current X scroll position
scrollOffsetY[index]  // Current Y scroll position

// Computed by layout (layoutDerived)
scrollable[index]     // 1 = scrollable, 0 = not
maxScrollX[index]     // Maximum X scroll
maxScrollY[index]     // Maximum Y scroll
```

### Scroll Operations

```typescript
export function scrollBy(index: number, deltaX: number, deltaY: number): boolean {
  if (!isScrollable(index)) return false

  const current = getScrollOffset(index)
  const max = getMaxScroll(index)

  const newX = Math.max(0, Math.min(current.x + deltaX, max.x))
  const newY = Math.max(0, Math.min(current.y + deltaY, max.y))

  if (newX === current.x && newY === current.y) {
    return false  // At boundary
  }

  setScrollOffset(index, newX, newY)
  return true
}

export function scrollToTop(index: number): void {
  setScrollOffset(index, getScrollOffset(index).x, 0)
}

export function scrollToBottom(index: number): void {
  setScrollOffset(index, getScrollOffset(index).x, getMaxScroll(index).y)
}
```

### Scroll Chaining

When at boundary, try parent:

```typescript
export function scrollByWithChaining(
  index: number,
  deltaX: number,
  deltaY: number,
  getParent?: (i: number) => number
): boolean {
  // Try to scroll this component
  if (scrollBy(index, deltaX, deltaY)) {
    return true
  }

  // At boundary - try parent
  if (getParent) {
    const parent = getParent(index)
    if (parent >= 0 && isScrollable(parent)) {
      return scrollByWithChaining(parent, deltaX, deltaY, getParent)
    }
  }

  return false
}
```

### Keyboard Scroll

```typescript
// Constants
export const LINE_SCROLL = 1
export const WHEEL_SCROLL = 3
export const PAGE_SCROLL_FACTOR = 0.9

export function handleArrowScroll(direction: 'up' | 'down' | 'left' | 'right'): boolean {
  const scrollable = getFocusedScrollable()
  if (scrollable < 0) return false

  switch (direction) {
    case 'up': return scrollBy(scrollable, 0, -LINE_SCROLL)
    case 'down': return scrollBy(scrollable, 0, LINE_SCROLL)
    case 'left': return scrollBy(scrollable, -LINE_SCROLL, 0)
    case 'right': return scrollBy(scrollable, LINE_SCROLL, 0)
  }
}
```

### Mouse Wheel

```typescript
export function handleWheelScroll(x: number, y: number, direction: string): boolean {
  // First try element under cursor
  let scrollable = findScrollableAt(x, y)

  // Fallback to focused scrollable
  if (scrollable < 0) {
    scrollable = getFocusedScrollable()
  }

  if (scrollable < 0) return false

  switch (direction) {
    case 'up': return scrollBy(scrollable, 0, -WHEEL_SCROLL)
    case 'down': return scrollBy(scrollable, 0, WHEEL_SCROLL)
    // ...
  }
}
```

### Scroll Into View

```typescript
export function scrollIntoView(
  childIndex: number,
  scrollableIndex: number,
  childY: number,
  childHeight: number,
  viewportHeight: number
): void {
  if (!isScrollable(scrollableIndex)) return

  const current = getScrollOffset(scrollableIndex)
  const viewportTop = current.y
  const viewportBottom = viewportTop + viewportHeight

  // Already visible?
  if (childY >= viewportTop && childY + childHeight <= viewportBottom) {
    return
  }

  // Scroll minimally to make visible
  if (childY < viewportTop) {
    setScrollOffset(scrollableIndex, current.x, childY)
  } else if (childY + childHeight > viewportBottom) {
    setScrollOffset(scrollableIndex, current.x, childY + childHeight - viewportHeight)
  }
}
```

## Drawn Cursor System

For input components - cursor is a character rendered into the buffer.

### Cursor Types

```typescript
type DrawnCursorStyle = 'block' | 'bar' | 'underline'

const CURSOR_CHARS: Record<DrawnCursorStyle, number> = {
  block: 0,        // Special: inverse block (swap fg/bg)
  bar: 0x2502,     // │ vertical line
  underline: 0x5F, // _ underscore
}
```

### Blink Animation

Shared clocks per FPS to minimize timers:

```typescript
interface BlinkRegistry {
  phase: Signal<boolean>      // true = visible
  interval: ReturnType<typeof setInterval> | null
  subscribers: number
}

const blinkRegistry = new Map<number, BlinkRegistry>()

function subscribeToBlink(fps: number): () => void {
  const registry = getBlinkClock(fps)
  registry.subscribers++

  // Start interval if first subscriber
  if (registry.subscribers === 1 && !registry.interval) {
    const ms = Math.floor(1000 / fps / 2)  // Half cycle
    registry.interval = setInterval(() => {
      registry.phase.value = !registry.phase.value
    }, ms)
  }

  return () => {
    registry.subscribers--
    if (registry.subscribers === 0 && registry.interval) {
      clearInterval(registry.interval)
      registry.interval = null
      registry.phase.value = true  // Reset to visible
    }
  }
}
```

### Cursor Creation

```typescript
export function createCursor(index: number, config: DrawnCursorConfig = {}): DrawnCursor {
  const { style = 'block', blink = true, fps = 2 } = config

  // Set cursor arrays
  interaction.cursorChar.setSource(index, CURSOR_CHARS[style])
  interaction.cursorBlinkFps.setSource(index, blink ? fps : 0)

  // Focus integration - start/stop blink
  const unsubscribeFocus = registerFocusCallbacks(index, {
    onFocus: () => {
      if (blink) unsubscribeBlink = subscribeToBlink(fps)
    },
    onBlur: () => {
      unsubscribeBlink?.()
      unsubscribeBlink = null
    },
  })

  // Visibility as reactive getter
  interaction.cursorVisible.setSource(index, () => {
    if (focusedIndex.value !== index) return 1  // Always visible when not focused
    if (!blink) return 1                         // No blink = always visible
    return getBlinkClock(fps).phase.value ? 1 : 0
  })

  return {
    setPosition: (pos) => interaction.cursorPosition.setSource(index, pos),
    getPosition: () => interaction.cursorPosition[index] || 0,
    show: () => manualVisible.value = true,
    hide: () => manualVisible.value = false,
    isVisible: () => (interaction.cursorVisible[index] ?? 1) === 1,
    dispose: () => disposeCursor(index),
  }
}
```

## Global Keys Integration

Central wiring of all input systems:

```typescript
export function initialize(options?: {
  onCleanup?: () => void
  exitOnCtrlC?: boolean
  enableMouse?: boolean
}): void {
  // Initialize input system with handlers
  input.initialize(handleKeyboardEvent, handleMouseEvent)

  // Enable mouse tracking
  if (options?.enableMouse !== false) {
    mouse.enableTracking()
  }
}

export function cleanup(): void {
  mouse.cleanup()
  keyboard.cleanup()
  input.cleanup()

  // Show cursor
  process.stdout.write('\x1b[?25h')

  cleanupCallback?.()
}
```
