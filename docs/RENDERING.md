# spark-tui Rendering Pipeline

This document covers the rendering pipeline, frame buffer structure, and the three rendering modes.

## Pipeline Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                     Component Tree                              │
│  (Components call primitives, create indices, bind to arrays)   │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Parallel Arrays + FlexNodes                  │
│     (SlotArrays with reactive cells, FlexNode Slot properties)  │
└────────────────────────────┬────────────────────────────────────┘
                             │
              ┌──────────────┴──────────────┐
              │                             │
    Layout-related change          Visual-only change
              │                             │
              ▼                             │
┌─────────────────────────┐                 │
│     layoutDerived       │                 │
│  (Flexbox → positions)  │                 │
└────────────┬────────────┘                 │
              │                             │
              ▼                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                    frameBufferDerived                           │
│           (Layout + arrays → 2D cell grid)                      │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Render Effect                              │
│              (DiffRenderer → terminal)                          │
└─────────────────────────────────────────────────────────────────┘
```

## Frame Buffer Structure

### Cell Type

```typescript
interface Cell {
  char: number       // Unicode codepoint (0 = empty/continuation)
  fg: RGBA           // Foreground color
  bg: RGBA           // Background color
  attrs: CellAttrs   // Text attributes (bold, italic, etc.)
}

interface RGBA {
  r: number  // 0-255 or -1 for terminal default
  g: number
  b: number
  a: number
}

// Special marker for terminal default color
const TERMINAL_DEFAULT: RGBA = { r: -1, g: -1, b: -1, a: -1 }

// Text attributes (bitflags)
enum Attr {
  NONE        = 0,
  BOLD        = 1 << 0,
  DIM         = 1 << 1,
  ITALIC      = 1 << 2,
  UNDERLINE   = 1 << 3,
  BLINK       = 1 << 4,
  INVERSE     = 1 << 5,
  HIDDEN      = 1 << 6,
  STRIKETHROUGH = 1 << 7,
}
```

### FrameBuffer

```typescript
interface FrameBuffer {
  width: number
  height: number
  cells: Cell[][]  // [y][x] for row-major access
}

function createFrameBuffer(width: number, height: number): FrameBuffer {
  const cells: Cell[][] = []
  for (let y = 0; y < height; y++) {
    const row: Cell[] = []
    for (let x = 0; x < width; x++) {
      row.push({
        char: 0x20,  // Space
        fg: TERMINAL_DEFAULT,
        bg: TERMINAL_DEFAULT,
        attrs: Attr.NONE,
      })
    }
    cells.push(row)
  }
  return { width, height, cells }
}
```

## Layout Derived

Computes positions and sizes using flexbox:

```typescript
export const layoutDerived = derived(() => {
  // Reading these creates reactive dependencies
  const width = terminalWidth.value
  const height = terminalHeight.value
  const mode = renderMode.value
  const indices = getAllocatedIndices()  // ReactiveSet!

  // Full flexbox computation (reads FlexNode.*.value = more dependencies)
  return computeLayoutFlexbox(
    width,
    height,
    indices,
    mode === 'fullscreen'  // constrainHeight
  )
})

interface ComputedLayout {
  x: number[]           // Component X positions
  y: number[]           // Component Y positions
  width: number[]       // Component widths
  height: number[]      // Component heights
  scrollable: number[]  // Scrollable flags
  maxScrollX: number[]  // Max scroll X
  maxScrollY: number[]  // Max scroll Y
  contentWidth: number  // Total content width
  contentHeight: number // Total content height
}
```

## FrameBuffer Derived

Renders component tree to cell grid:

```typescript
export const frameBufferDerived = derived(() => {
  // Read layout (creates dependency)
  const computed = layoutDerived.value

  // Determine buffer dimensions based on render mode
  const mode = renderMode.value
  const bufferWidth = mode === 'fullscreen'
    ? terminalWidth.value
    : computed.contentWidth
  const bufferHeight = mode === 'fullscreen'
    ? terminalHeight.value
    : computed.contentHeight

  // Create buffer
  const fb = createFrameBuffer(bufferWidth, bufferHeight)

  // Build HitGrid for mouse lookup
  hitGrid.resize(bufferWidth, bufferHeight)
  hitGrid.clear()

  // Z-sort components
  const sorted = sortByZIndex(getAllocatedIndices(), computed)

  // Render each component
  for (const index of sorted) {
    renderComponent(fb, index, computed)
  }

  return fb
})
```

### Component Rendering

```typescript
function renderComponent(fb: FrameBuffer, index: number, computed: ComputedLayout): void {
  const type = componentType[index]
  const x = computed.x[index]!
  const y = computed.y[index]!
  const w = computed.width[index]!
  const h = computed.height[index]!

  // Skip invisible
  const vis = visible[index]
  if (vis === 0 || vis === false) return

  // Get colors (reads visual arrays = dependencies)
  const fg = fgColor[index] ?? TERMINAL_DEFAULT
  const bg = bgColor[index] ?? TERMINAL_DEFAULT

  // Fill background
  fillRect(fb, x, y, w, h, bg)

  // Register in HitGrid for mouse events
  hitGrid.fillRect(x, y, w, h, index)

  // Render borders (if any)
  const border = borderStyle[index]
  if (border > 0) {
    renderBorder(fb, x, y, w, h, border, borderColor[index] ?? fg)
  }

  // Render content based on type
  switch (type) {
    case ComponentType.TEXT:
    case ComponentType.INPUT:
      renderText(fb, index, x, y, w, h, fg, bg, computed)
      break
    case ComponentType.BOX:
      // Box content is children (rendered separately in z-order)
      break
  }
}
```

### Text Rendering

```typescript
function renderText(
  fb: FrameBuffer,
  index: number,
  x: number, y: number, w: number, h: number,
  fg: RGBA, bg: RGBA,
  computed: ComputedLayout
): void {
  const content = textContent[index] ?? ''
  const align = textAlign[index] ?? 0
  const wrap = textWrap[index] ?? 1
  const attrs = textAttrs[index] ?? Attr.NONE

  // Calculate content area (inside padding/border)
  const padTop = paddingTop[index] ?? 0
  const padRight = paddingRight[index] ?? 0
  const padBottom = paddingBottom[index] ?? 0
  const padLeft = paddingLeft[index] ?? 0
  const borderW = borderStyle[index] > 0 ? 1 : 0

  const contentX = x + padLeft + borderW
  const contentY = y + padTop + borderW
  const contentW = w - padLeft - padRight - borderW * 2
  const contentH = h - padTop - padBottom - borderW * 2

  // Handle text wrapping
  let lines: string[]
  if (wrap === 0) {  // nowrap
    lines = [content]
  } else if (wrap === 2) {  // truncate
    lines = [truncateWithEllipsis(content, contentW)]
  } else {  // wrap
    lines = wrapText(content, contentW)
  }

  // Render each line
  for (let lineIdx = 0; lineIdx < lines.length && lineIdx < contentH; lineIdx++) {
    const line = lines[lineIdx]!
    const lineY = contentY + lineIdx

    // Calculate X based on alignment
    let lineX = contentX
    if (align === 1) {  // center
      lineX = contentX + Math.floor((contentW - line.length) / 2)
    } else if (align === 2) {  // right
      lineX = contentX + contentW - line.length
    }

    // Write characters
    for (let i = 0; i < line.length; i++) {
      const cellX = lineX + i
      if (cellX >= 0 && cellX < fb.width && lineY >= 0 && lineY < fb.height) {
        const cell = fb.cells[lineY]![cellX]!
        cell.char = line.codePointAt(i) ?? 0x20
        cell.fg = fg
        cell.bg = bg
        cell.attrs = attrs
      }
    }
  }

  // Render cursor for INPUT components
  if (componentType[index] === ComponentType.INPUT) {
    renderCursor(fb, index, contentX, contentY, fg, bg)
  }
}
```

### Cursor Rendering

```typescript
function renderCursor(
  fb: FrameBuffer,
  index: number,
  contentX: number,
  contentY: number,
  fg: RGBA, bg: RGBA
): void {
  // Only render if visible
  const visible = cursorVisible[index]
  if (visible !== 1) return

  // Only render if focused
  if (focusedIndex.value !== index) return

  const pos = cursorPosition[index] ?? 0
  const cursorChar = cursorChar[index] ?? 0
  const cellX = contentX + pos
  const cellY = contentY

  if (cellX >= 0 && cellX < fb.width && cellY >= 0 && cellY < fb.height) {
    const cell = fb.cells[cellY]![cellX]!

    if (cursorChar === 0) {
      // Block cursor = swap fg/bg
      const temp = cell.fg
      cell.fg = cell.bg
      cell.bg = temp
    } else {
      // Custom cursor character
      cell.char = cursorChar
    }
  }
}
```

## Differential Renderer

The "Terminal GPU" - outputs only changed cells:

```typescript
export class DiffRenderer {
  private output = new OutputBuffer()
  private cellRenderer = new StatefulCellRenderer()
  private previousBuffer: FrameBuffer | null = null

  render(buffer: FrameBuffer): boolean {
    const prev = this.previousBuffer
    let hasChanges = false

    // Begin synchronized output (prevents flicker)
    this.output.write(ansi.beginSync)

    // Reset cell renderer state
    this.cellRenderer.reset()

    // Render only changed cells
    for (let y = 0; y < buffer.height; y++) {
      for (let x = 0; x < buffer.width; x++) {
        const cell = buffer.cells[y]![x]!

        // Skip if unchanged
        if (prev && y < prev.height && x < prev.width) {
          const prevCell = prev.cells[y]![x]!
          if (cellEqual(cell, prevCell)) continue
        }

        hasChanges = true
        this.cellRenderer.render(this.output, x, y, cell)
      }
    }

    // End synchronized output
    this.output.write(ansi.endSync)

    // Flush to terminal
    this.output.flushSync()

    // Store for next diff
    this.previousBuffer = buffer

    return hasChanges
  }
}
```

### Stateful Cell Renderer

Minimizes ANSI output by tracking state:

```typescript
class StatefulCellRenderer {
  private lastFg: RGBA | null = null
  private lastBg: RGBA | null = null
  private lastAttrs: CellAttrs = Attr.NONE
  private lastX = -1
  private lastY = -1

  render(output: OutputBuffer, x: number, y: number, cell: Cell): void {
    // Move cursor only if not sequential
    if (y !== this.lastY || x !== this.lastX + 1) {
      output.write(ansi.moveTo(x + 1, y + 1))  // ANSI is 1-indexed
    }

    // Reset + new attrs if changed
    if (cell.attrs !== this.lastAttrs) {
      output.write(ansi.reset)
      if (cell.attrs !== Attr.NONE) {
        output.write(ansi.attrs(cell.attrs))
      }
      // After reset, colors need re-emit
      this.lastFg = null
      this.lastBg = null
      this.lastAttrs = cell.attrs
    }

    // Foreground only if changed
    if (!this.lastFg || !rgbaEqual(cell.fg, this.lastFg)) {
      output.write(ansi.fg(cell.fg))
      this.lastFg = cell.fg
    }

    // Background only if changed
    if (!this.lastBg || !rgbaEqual(cell.bg, this.lastBg)) {
      output.write(ansi.bg(cell.bg))
      this.lastBg = cell.bg
    }

    // Output character
    output.write(String.fromCodePoint(cell.char))

    this.lastX = x
    this.lastY = y
  }
}
```

## ANSI Escape Codes

```typescript
export const ansi = {
  // Cursor movement
  moveTo: (x: number, y: number) => `\x1b[${y};${x}H`,
  moveUp: (n: number) => `\x1b[${n}A`,
  moveDown: (n: number) => `\x1b[${n}B`,

  // Clearing
  clearScreen: '\x1b[2J',
  clearTerminal: '\x1b[2J\x1b[3J\x1b[H',
  eraseLines: (n: number) => `\x1b[${n}M`,

  // Colors
  fg: (c: RGBA) => c.r === -1 ? '\x1b[39m' : `\x1b[38;2;${c.r};${c.g};${c.b}m`,
  bg: (c: RGBA) => c.r === -1 ? '\x1b[49m' : `\x1b[48;2;${c.r};${c.g};${c.b}m`,

  // Attributes
  reset: '\x1b[0m',
  attrs: (a: number) => {
    const codes: string[] = []
    if (a & Attr.BOLD) codes.push('1')
    if (a & Attr.DIM) codes.push('2')
    if (a & Attr.ITALIC) codes.push('3')
    if (a & Attr.UNDERLINE) codes.push('4')
    if (a & Attr.BLINK) codes.push('5')
    if (a & Attr.INVERSE) codes.push('7')
    if (a & Attr.HIDDEN) codes.push('8')
    if (a & Attr.STRIKETHROUGH) codes.push('9')
    return codes.length > 0 ? `\x1b[${codes.join(';')}m` : ''
  },

  // Synchronized output (prevents flicker)
  beginSync: '\x1b[?2026h',
  endSync: '\x1b[?2026l',

  // Cursor visibility
  hideCursor: '\x1b[?25l',
  showCursor: '\x1b[?25h',

  // Cursor save/restore
  saveCursor: '\x1b[s',
  restoreCursor: '\x1b[u',

  // Alternate screen buffer
  enterAltScreen: '\x1b[?1049h',
  exitAltScreen: '\x1b[?1049l',
}
```

## Three Rendering Modes

### Fullscreen Mode

Uses alternate screen buffer, fixed terminal dimensions, differential rendering.

```typescript
// Setup
process.stdout.write(ansi.enterAltScreen)
process.stdout.write(ansi.hideCursor)
process.stdout.write(ansi.clearScreen)

// Render
const fb = frameBufferDerived.value  // Terminal dimensions
diffRenderer.render(fb)

// Cleanup
process.stdout.write(ansi.exitAltScreen)
process.stdout.write(ansi.showCursor)
```

**Characteristics:**
- Buffer size = terminal size
- Uses alternate screen buffer (content preserved on exit)
- Differential rendering (only changed cells)
- Fixed layout (height constrained)

### Inline Mode

Normal buffer, content-determined height, full rebuild each frame.

```typescript
export class InlineRenderer {
  private previousOutput = ''

  render(buffer: FrameBuffer): void {
    const output = this.buildOutput(buffer)

    // Skip if unchanged
    if (output === this.previousOutput) return

    // Clear and redraw
    this.output.write(ansi.beginSync)
    this.output.write(ansi.clearTerminal + output)
    this.output.write(ansi.endSync)
    this.output.flushSync()

    this.previousOutput = output
  }

  private buildOutput(buffer: FrameBuffer): string {
    const chunks: string[] = []

    for (let y = 0; y < buffer.height; y++) {
      if (y > 0) chunks.push('\n')
      for (let x = 0; x < buffer.width; x++) {
        this.renderCell(chunks, buffer.cells[y]![x]!)
      }
    }

    chunks.push(ansi.reset)
    chunks.push('\n')
    return chunks.join('')
  }
}
```

**Characteristics:**
- Buffer size = content size
- Normal screen buffer (content stays after exit)
- Full rebuild each frame (simpler than diff for variable height)
- Content-determined layout (height unconstrained)

### Append Mode (For CLIs)

Active region that updates + frozen history above.

```typescript
interface AppendModeState {
  frozenLines: string[]     // Lines that won't change
  activeHeight: number      // Current active region height
}

// When user submits command, freeze current output
function freezeActiveRegion(): void {
  // Move active content to frozen
  const activeOutput = renderActiveRegion()
  frozenLines.push(...activeOutput.split('\n'))

  // Clear active region
  clearActiveRegion()
}

// Render active region (overwrites previous)
function renderActiveRegion(buffer: FrameBuffer): void {
  // Move up to overwrite previous active content
  if (previousHeight > 0) {
    output.write(ansi.moveUp(previousHeight))
    output.write(ansi.carriageReturn)
  }

  // Render buffer
  for (let y = 0; y < buffer.height; y++) {
    if (y > 0) output.write('\n')
    for (let x = 0; x < buffer.width; x++) {
      renderCell(output, buffer.cells[y]![x]!)
    }
  }

  // Clear any leftover lines from previous (taller) render
  if (buffer.height < previousHeight) {
    for (let i = buffer.height; i < previousHeight; i++) {
      output.write('\n')
      output.write(ansi.eraseLine)
    }
  }

  previousHeight = buffer.height
}
```

**Characteristics:**
- Two-zone: frozen history + active region
- Active region updates in place
- History scrolls normally
- Ideal for CLI tools with command history

## Render Mode Selection

```typescript
type RenderMode = 'fullscreen' | 'inline' | 'append'

const renderMode = signal<RenderMode>('fullscreen')

// In mount()
export function mount(render: () => void, options?: MountOptions) {
  const mode = options?.mode ?? 'fullscreen'
  renderMode.value = mode

  // Choose renderer
  const renderer = mode === 'fullscreen'
    ? new DiffRenderer()
    : mode === 'inline'
      ? new InlineRenderer()
      : new AppendRenderer()

  // Setup based on mode
  if (mode === 'fullscreen') {
    process.stdout.write(ansi.enterAltScreen)
    process.stdout.write(ansi.hideCursor)
  }

  // Render effect
  effect(() => {
    const fb = frameBufferDerived.value
    renderer.render(fb)
  })

  // Cleanup
  return () => {
    if (mode === 'fullscreen') {
      process.stdout.write(ansi.exitAltScreen)
    }
    process.stdout.write(ansi.showCursor)
    process.stdout.write(ansi.reset)
  }
}
```

## Output Buffering

All output is batched for single syscall:

```typescript
export class OutputBuffer {
  private chunks: string[] = []
  private totalLength = 0

  write(str: string): void {
    if (str.length === 0) return
    this.chunks.push(str)
    this.totalLength += str.length
  }

  flushSync(): void {
    if (this.totalLength === 0) return
    const output = this.chunks.join('')
    this.clear()
    process.stdout.write(output)
  }

  // For Bun: async variant
  async flush(): Promise<void> {
    if (this.totalLength === 0) return
    const output = this.chunks.join('')
    this.clear()
    await Bun.write(Bun.stdout, output)
  }
}
```

## Terminal Size Tracking

```typescript
export const terminalWidth = signal(process.stdout.columns || 80)
export const terminalHeight = signal(process.stdout.rows || 24)

// Track resize events
process.stdout.on('resize', () => {
  terminalWidth.value = process.stdout.columns
  terminalHeight.value = process.stdout.rows

  // For fullscreen: invalidate diff renderer
  diffRenderer.invalidate()

  // Resize hitgrid
  hitGrid.resize(terminalWidth.value, terminalHeight.value)
})
```

## Performance Considerations

### Diff Rendering Optimizations

1. **Stateful cell renderer** - Only emits changed attributes/colors
2. **Sequential cursor tracking** - Skips cursor move for adjacent cells
3. **Synchronized output** - Wraps in begin/end sync for flicker-free
4. **Batched output** - Single syscall per frame

### Layout Caching

- layoutDerived caches result until dependencies change
- Visual-only changes (colors) skip layout entirely
- Only layout-affecting changes trigger re-computation

### Memory Efficiency

- FrameBuffer reuses same structure (avoid allocation per frame)
- HitGrid uses Int16Array (2 bytes per cell)
- Previous buffer stored for diff (trade memory for speed)
