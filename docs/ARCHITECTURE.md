# spark-tui Architecture Deep Dive

This document provides a comprehensive reference of the TypeScript TUI implementation patterns that must be faithfully ported to Rust.

## Core Architecture: Reactive Parallel Arrays

The entire framework is built on two foundational patterns that work together:

### 1. Parallel Arrays (ECS Pattern)

Components are **NOT objects**. They are indices into columnar arrays:

```
Index 0: Box  (parent=-1, width=80, height=24, visible=true,  fg=white, ...)
Index 1: Text (parent=0,  width=auto, height=1, visible=true, fg=blue,  ...)
Index 2: Box  (parent=0,  width=40,  height=10, visible=true, fg=white, ...)
Index 3: Text (parent=2,  width=auto, height=1, visible=true, fg=green, ...)
```

This enables:
- **Cache-friendly iteration** - Layout loops over contiguous memory
- **Efficient reactivity** - Each cell is a stable Slot that never moves
- **No object allocation** - Adding a component just writes to existing arrays

### 2. SlotArray - Reactive Parallel Arrays

Each array is a `SlotArray<T>` where every cell is a reactive `Slot<T>`:

```typescript
// TypeScript implementation
export function slotArray<T>(defaultValue: T): SlotArray<T> {
  return new Proxy([], {
    get(target, prop) {
      const index = Number(prop)
      if (Number.isInteger(index)) {
        // Lazy creation - slot only exists when accessed
        if (!target[index]) {
          target[index] = slot(defaultValue)
        }
        // Auto-unwrap .value when reading
        return target[index].value
      }
      // ... methods like setSource, clear
    }
  })
}
```

Key insight: **Reading `array[i]` returns the unwrapped value and creates a reactive dependency**. The computed value that reads it will re-run when the slot's source changes.

## The Two-Path Reactive Pipeline

**This is crucial architecture that enables efficient updates:**

```
                    ┌─────────────────────────────────────────┐
                    │          Component Props Changed         │
                    └─────────────────┬───────────────────────┘
                                      │
                           ┌──────────┴──────────┐
                           │   What changed?     │
                           └──────────┬──────────┘
                                      │
              ┌───────────────────────┼───────────────────────┐
              │                       │                       │
   ┌──────────▼──────────┐ ┌─────────▼──────────┐ ┌─────────▼──────────┐
   │ Layout-related prop │ │  Non-layout prop   │ │  Visual-only prop  │
   │ (width, flexGrow,   │ │  (textContent,     │ │  (fgColor, bgColor,│
   │  padding, margin)   │ │   visible)         │ │   borderColor)     │
   └──────────┬──────────┘ └─────────┬──────────┘ └─────────┬──────────┘
              │                       │                       │
              ▼                       ▼                       │
   ┌───────────────────┐   ┌───────────────────┐             │
   │  layoutDerived    │   │  layoutDerived    │             │
   │  RE-COMPUTES      │   │  SEES NO CHANGE   │             │
   │  (Taffy flexbox)  │   │  (returns cache)  │             │
   └─────────┬─────────┘   └─────────┬─────────┘             │
              │                       │                       │
              ▼                       ▼                       ▼
   ┌─────────────────────────────────────────────────────────────┐
   │                    frameBufferDerived                       │
   │           RE-COMPUTES (reads layout + arrays)               │
   └──────────────────────────────┬──────────────────────────────┘
                                  │
                                  ▼
   ┌─────────────────────────────────────────────────────────────┐
   │                       Render Effect                         │
   │              (DiffRenderer → terminal)                      │
   └─────────────────────────────────────────────────────────────┘
```

**Why this matters:**
- Changing `fgColor` skips layout computation entirely
- Changing `width` runs full pipeline
- Layout is expensive (flexbox algorithm), framebuffer is cheap (just drawing)

## Component Registry

The registry manages the component lifecycle and index allocation:

```typescript
// Maps
const idToIndex = new Map<string, number>()      // "myBox" → 5
const indexToId = new Map<number, string>()      // 5 → "myBox"
const allocatedIndices = new ReactiveSet<number>() // {0, 1, 2, 3, 5} (4 freed)

// Pool for O(1) reuse
const freeIndices: number[] = []
let nextIndex = 0

export function allocateIndex(id?: string): number {
  const componentId = id ?? `c${idCounter++}`

  // Reuse from pool or allocate new
  const index = freeIndices.length > 0
    ? freeIndices.pop()!
    : nextIndex++

  // Register bidirectional mapping
  idToIndex.set(componentId, index)
  indexToId.set(index, componentId)
  allocatedIndices.add(index)  // ReactiveSet triggers deriveds!

  ensureAllCapacity(index)  // Grow arrays if needed
  return index
}

export function releaseIndex(index: number): void {
  // RECURSIVE! Release children first
  for (const childIndex of allocatedIndices) {
    if (parentIndex[childIndex] === index) {
      releaseIndex(childIndex)
    }
  }

  runDestroyCallbacks(index)
  destroyFlexNode(index)

  // Clean up
  idToIndex.delete(id)
  indexToId.delete(index)
  allocatedIndices.delete(index)
  clearAllAtIndex(index)

  freeIndices.push(index)  // Return to pool
}
```

### Parent Context Stack

For nested component creation, a stack tracks the current parent:

```typescript
const parentStack: number[] = []

export function pushParentContext(index: number): void {
  parentStack.push(index)
}

export function popParentContext(): void {
  parentStack.pop()
}

export function getCurrentParentIndex(): number {
  return parentStack.length > 0 ? parentStack[parentStack.length - 1] : -1
}

// Usage in box primitive:
core.parentIndex.setSource(index, getCurrentParentIndex())

if (props.children) {
  pushParentContext(index)
  try {
    props.children()  // Children get this box as parent
  } finally {
    popParentContext()
  }
}
```

## FlexNode - Persistent Layout Object

Each component gets one FlexNode for its entire lifetime:

```typescript
export class FlexNode {
  readonly index: number

  // Container properties (5)
  flexDirection: Slot<number> = slot(0)   // 0=column, 1=row, 2=col-reverse, 3=row-reverse
  flexWrap: Slot<number> = slot(0)        // 0=nowrap, 1=wrap, 2=wrap-reverse
  justifyContent: Slot<number> = slot(0)  // 0=start, 1=center, 2=end, 3-5=space-*
  alignItems: Slot<number> = slot(0)      // 0=stretch, 1=start, 2=center, 3=end, 4=baseline
  alignContent: Slot<number> = slot(0)

  // Item properties (5)
  flexGrow: Slot<number> = slot(0)
  flexShrink: Slot<number> = slot(1)  // Default shrink!
  flexBasis: Slot<Dimension> = slot(0)
  alignSelf: Slot<number> = slot(0)
  order: Slot<number> = slot(0)

  // Dimensions (6)
  width: Slot<Dimension> = slot(0)   // 0=auto, number=px, string='50%'
  height: Slot<Dimension> = slot(0)
  minWidth: Slot<Dimension> = slot(0)
  maxWidth: Slot<Dimension> = slot(0)
  minHeight: Slot<Dimension> = slot(0)
  maxHeight: Slot<Dimension> = slot(0)

  // Spacing (11)
  marginTop: Slot<number> = slot(0)
  marginRight: Slot<number> = slot(0)
  marginBottom: Slot<number> = slot(0)
  marginLeft: Slot<number> = slot(0)
  paddingTop: Slot<number> = slot(0)
  paddingRight: Slot<number> = slot(0)
  paddingBottom: Slot<number> = slot(0)
  paddingLeft: Slot<number> = slot(0)
  gap: Slot<number> = slot(0)
  rowGap: Slot<number> = slot(0)
  columnGap: Slot<number> = slot(0)

  // Border (4)
  borderTop: Slot<number> = slot(0)    // 0=none, 1=has border
  borderRight: Slot<number> = slot(0)
  borderBottom: Slot<number> = slot(0)
  borderLeft: Slot<number> = slot(0)

  // Other (2)
  overflow: Slot<number> = slot(0)     // 0=visible, 1=hidden, 2=scroll
  position: Slot<number> = slot(0)     // 0=relative, 1=absolute
}
```

**Critical Pattern:** Props bind directly to Slots:

```typescript
// In box.ts - preserves reactivity!
if (props.width !== undefined) {
  flexNode.width.source = props.width  // Direct binding
}
if (props.flexDirection !== undefined) {
  flexNode.flexDirection.source = enumSource(props.flexDirection, flexDirectionToNum)
}
```

The `enumSource` helper converts string enums to numbers reactively:

```typescript
function enumSource<T>(
  prop: T | (() => T) | { value: T },
  converter: (v: T) => number
): number | (() => number) {
  if (typeof prop === 'function') {
    return () => converter((prop as () => T)())
  }
  if (prop !== null && typeof prop === 'object' && 'value' in prop) {
    return () => converter((prop as { value: T }).value)
  }
  return converter(prop)
}
```

## Layout Pipeline

### layoutDerived

The core layout computation is a `derived` that reads FlexNode slots:

```typescript
export const layoutDerived = derived(() => {
  // Reading these creates reactive dependencies
  const width = terminalWidth.value
  const height = terminalHeight.value
  const mode = renderMode.value
  const indices = getAllocatedIndices()  // ReactiveSet!

  // Compute flexbox layout (reads FlexNode.*.value = more dependencies)
  return computeLayoutFlexbox(width, height, indices, mode === 'fullscreen')
})
```

The flexbox computation reads every FlexNode slot:

```typescript
function layoutContainer(containerIndex, allIndices, ...) {
  const node = getFlexNode(containerIndex)

  // Reading .value creates dependency - changes trigger re-layout
  const containerStyle = {
    flexDirection: mapFlexDirection(node.flexDirection.value),
    flexWrap: mapFlexWrap(node.flexWrap.value),
    justifyContent: mapJustifyContent(node.justifyContent.value),
    // ... all 33 properties
  }

  // Run W3C flexbox spec algorithm
  const result = computeFlexLayout(input)

  // Extract to output arrays
  for (const childIndex of children) {
    outX[childIndex] = result.items[i].x
    outY[childIndex] = result.items[i].y
    outW[childIndex] = result.items[i].width
    outH[childIndex] = result.items[i].height
  }
}
```

### frameBufferDerived

The framebuffer reads layout results and visual arrays:

```typescript
export const frameBufferDerived = derived(() => {
  // Read layout (creates dependency on layout changes)
  const computed = layoutDerived.value

  // Create framebuffer
  const fb = createFrameBuffer(computed.contentWidth, computed.contentHeight)

  // Render each component (reads visual arrays = more dependencies)
  for (const index of getAllocatedIndices()) {
    renderComponent(fb, index, computed)
  }

  return fb
})
```

### Render Effect

Single effect renders to terminal:

```typescript
const diffRenderer = new DiffRenderer()

effect(() => {
  const fb = frameBufferDerived.value
  diffRenderer.render(fb)
})
```

## Array Categories

### Core Arrays (always needed)

```typescript
export const componentType: ComponentType[] = []  // Plain array (never changes after create)
export const parentIndex = slotArray<number>(-1)  // Reactive
export const visible = slotArray<number | boolean>(1)  // 1=visible, 0/false=hidden
export const componentId = slotArray<string>('')  // Optional ID
```

### Layout Arrays (FlexNode owns most, some shared)

```typescript
// In FlexNode (layout computation reads these)
flexDirection, flexWrap, justifyContent, alignItems, alignContent
flexGrow, flexShrink, flexBasis, alignSelf, order
width, height, minWidth, maxWidth, minHeight, maxHeight
marginTop/Right/Bottom/Left, paddingTop/Right/Bottom/Left
gap, rowGap, columnGap
borderTop/Right/Bottom/Left, overflow, position

// In arrays (framebuffer reads these for rendering)
export const zIndex = slotArray<number>(0)
```

### Visual Arrays (framebuffer reads directly)

```typescript
export const fgColor = slotArray<RGBA>(TERMINAL_DEFAULT)
export const bgColor = slotArray<RGBA>(TERMINAL_DEFAULT)
export const borderStyle = slotArray<number>(0)  // Border style enum
export const borderTop = slotArray<number>(0)    // Per-side styles
export const borderRight = slotArray<number>(0)
export const borderBottom = slotArray<number>(0)
export const borderLeft = slotArray<number>(0)
export const borderColor = slotArray<RGBA>(TERMINAL_DEFAULT)
export const opacity = slotArray<number>(255)
```

### Text Arrays (shared - layout measures, framebuffer renders)

```typescript
export const textContent = slotArray<string>('')
export const textAlign = slotArray<number>(0)     // 0=left, 1=center, 2=right
export const textWrap = slotArray<number>(1)      // 0=nowrap, 1=wrap, 2=truncate
export const textAttrs = slotArray<number>(Attr.NONE)  // Bold, italic, etc.
```

### Interaction Arrays (user state)

```typescript
export const focusable = slotArray<number>(0)     // 0=not focusable, 1=focusable
export const tabIndex = slotArray<number>(0)      // Sort order for Tab cycling
export const focusedIndex = signal(-1)            // Currently focused component
export const cursorPosition = slotArray<number>(0)
export const cursorChar = slotArray<number>(0)    // Cursor character codepoint
export const cursorVisible = slotArray<number>(1)
export const cursorBlinkFps = slotArray<number>(2)
export const scrollOffsetX = slotArray<number>(0)
export const scrollOffsetY = slotArray<number>(0)
export const hovered = slotArray<number>(0)       // Mouse hover state
export const pressed = slotArray<number>(0)       // Mouse pressed state
```

### Spacing Arrays (framebuffer reads for content area calculation)

```typescript
// Duplicated in arrays for framebuffer to read
// FlexNode has these too for layout computation
export const paddingTop = slotArray<number>(0)
export const paddingRight = slotArray<number>(0)
export const paddingBottom = slotArray<number>(0)
export const paddingLeft = slotArray<number>(0)
```

## Dimension Type

Used for width, height, flex-basis, and constraints:

```typescript
type Dimension = number | string
// 0        = auto (content-determined)
// number>0 = pixels (terminal cells)
// '50%'    = percentage of parent
```

## Cleanup and Memory Management

### Index Release Chain

```
releaseIndex(parent)
    ├── Find children where parentIndex[child] === parent
    ├── releaseIndex(child1)  // Recursive!
    ├── releaseIndex(child2)
    ├── runDestroyCallbacks(parent)
    ├── destroyFlexNode(parent)  // Disconnects all slots
    ├── clearAllAtIndex(parent)  // Clears array values
    └── freeIndices.push(parent) // Return to pool
```

### Auto-Cleanup on Empty

When all components are released, everything resets:

```typescript
if (allocatedIndices.size === 0) {
  resetAllArrays()     // Clear all parallel arrays
  resetTitanArrays()   // Clear layout output arrays
  freeIndices.length = 0
  nextIndex = 0
}
```

## Important Patterns

### 1. Bind Props Directly

```typescript
// CORRECT - signal stays connected
flexNode.width.source = props.width

// WRONG - extracts value, breaks reactivity
flexNode.width.source = props.width.get()
```

### 2. Only Set If Defined

```typescript
// Only bind if prop was passed (don't override defaults)
if (props.width !== undefined) {
  flexNode.width.source = props.width
}
```

### 3. Shorthand With Override

```typescript
// padding shorthand
if (props.padding !== undefined) {
  flexNode.paddingTop.source = props.paddingTop ?? props.padding
  flexNode.paddingRight.source = props.paddingRight ?? props.padding
  flexNode.paddingBottom.source = props.paddingBottom ?? props.padding
  flexNode.paddingLeft.source = props.paddingLeft ?? props.padding
}
```

### 4. Border Width vs Style

Border has both **width** (for layout) and **style** (for rendering):

```typescript
// FlexNode gets width (0 or 1) for spacing calculation
if (props.border !== undefined) {
  const widthSource = () => props.border > 0 ? 1 : 0
  flexNode.borderTop.source = widthSource
  flexNode.borderRight.source = widthSource
  flexNode.borderBottom.source = widthSource
  flexNode.borderLeft.source = widthSource
}

// Visual array gets style enum for rendering
if (props.border !== undefined) {
  visual.borderStyle.setSource(index, props.border)
}
```

### 5. Reactive Enum Conversion

```typescript
// Convert string enums to numbers reactively
flexNode.flexDirection.source = enumSource(
  props.flexDirection,  // 'row' | 'column' | signal | getter
  flexDirectionToNum    // Converter function
)
```

## Summary

The architecture can be summarized as:

1. **Components are indices** into parallel SlotArrays
2. **Each index gets a FlexNode** with 33 reactive Slot properties
3. **Props bind directly to Slots** - never extract values
4. **Layout changes** trigger full pipeline (layoutDerived → frameBufferDerived → render)
5. **Visual changes** skip layout (frameBufferDerived → render only)
6. **Cleanup is recursive** - releasing parent releases children
7. **Index pooling** enables O(1) allocation/release
