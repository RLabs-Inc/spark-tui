/**
 * SparkTUI — Live Stress Test
 *
 * Full pipeline stress test: TS writes → SharedArrayBuffer → wake →
 * Rust engine → layout → framebuffer → diff render → terminal.
 *
 * Displays live stats in the TUI itself. Press Ctrl+C to exit with summary.
 *
 * Tests:
 *   Phase 1: Color animation (visual-only, tests smart skip path)
 *   Phase 2: Layout animation (size changes, full pipeline)
 *   Phase 3: Text thrashing (rapid text content updates)
 *   Phase 4: Node scaling (grow tree from 10 to 1000 nodes)
 *   Phase 5: Burst fire (rapid-fire updates, tests wake coalescing)
 *
 * Run: bun run examples/bench-stress.ts
 */

import { ptr } from 'bun:ffi'
import { loadEngine } from '../ts/bridge/ffi'
import {
  createSharedBuffer,
  setTerminalSize,
  setNodeText,
  markDirty,
  notifyRust,
  packColor,
  COMPONENT_BOX,
  COMPONENT_TEXT,
  U8_COMPONENT_TYPE,
  U8_VISIBLE,
  U8_FLEX_DIRECTION,
  U8_ALIGN_ITEMS,
  U8_JUSTIFY_CONTENT,
  U8_BORDER_STYLE,
  U8_BORDER_TOP_WIDTH,
  U8_BORDER_RIGHT_WIDTH,
  U8_BORDER_BOTTOM_WIDTH,
  U8_BORDER_LEFT_WIDTH,
  U32_FG_COLOR,
  U32_BG_COLOR,
  U32_BORDER_COLOR,
  I32_PARENT_INDEX,
  F32_WIDTH,
  F32_HEIGHT,
  F32_PADDING_TOP,
  F32_PADDING_RIGHT,
  F32_PADDING_BOTTOM,
  F32_PADDING_LEFT,
  F32_GROW,
  DIRTY_LAYOUT,
  DIRTY_VISUAL,
  DIRTY_TEXT,
  DIRTY_HIERARCHY,
  HEADER_NODE_COUNT,
  HEADER_RENDER_COUNT,
} from '../ts/bridge/shared-buffer'

// =============================================================================
// SETUP
// =============================================================================

const views = createSharedBuffer()
const cols = process.stdout.columns || 120
const rows = process.stdout.rows || 40
setTerminalSize(views, cols, rows)

const ALL_DIRTY = DIRTY_LAYOUT | DIRTY_VISUAL | DIRTY_TEXT | DIRTY_HIERARCHY

// =============================================================================
// BUILD INITIAL TREE
// =============================================================================

// Node 0: Root (full screen)
views.u8[U8_COMPONENT_TYPE][0] = COMPONENT_BOX
views.u8[U8_VISIBLE][0] = 1
views.u8[U8_FLEX_DIRECTION][0] = 1  // column
views.f32[F32_WIDTH][0] = cols
views.f32[F32_HEIGHT][0] = rows
views.i32[I32_PARENT_INDEX][0] = -1
views.u32[U32_BG_COLOR][0] = packColor(15, 15, 30, 255)
markDirty(views, 0, ALL_DIRTY)

// Node 1: Header bar
views.u8[U8_COMPONENT_TYPE][1] = COMPONENT_BOX
views.u8[U8_VISIBLE][1] = 1
views.u8[U8_FLEX_DIRECTION][1] = 0  // row
views.u8[U8_ALIGN_ITEMS][1] = 2     // center
views.f32[F32_WIDTH][1] = cols
views.f32[F32_HEIGHT][1] = 3
views.f32[F32_PADDING_LEFT][1] = 2
views.i32[I32_PARENT_INDEX][1] = 0
views.u32[U32_BG_COLOR][1] = packColor(30, 30, 60, 255)
views.u32[U32_BORDER_COLOR][1] = packColor(60, 60, 120, 255)
views.u8[U8_BORDER_STYLE][1] = 1
views.u8[U8_BORDER_BOTTOM_WIDTH][1] = 1
markDirty(views, 1, ALL_DIRTY)

// Node 2: Title
views.u8[U8_COMPONENT_TYPE][2] = COMPONENT_TEXT
views.u8[U8_VISIBLE][2] = 1
views.i32[I32_PARENT_INDEX][2] = 1
views.u32[U32_FG_COLOR][2] = packColor(100, 220, 255, 255)
setNodeText(views, 2, 'SparkTUI Stress Test')
markDirty(views, 2, ALL_DIRTY)

// Node 3: Stats line
views.u8[U8_COMPONENT_TYPE][3] = COMPONENT_TEXT
views.u8[U8_VISIBLE][3] = 1
views.f32[F32_PADDING_LEFT][3] = 4
views.i32[I32_PARENT_INDEX][3] = 1
views.u32[U32_FG_COLOR][3] = packColor(180, 180, 200, 255)
setNodeText(views, 3, 'Starting...')
markDirty(views, 3, ALL_DIRTY)

// Node 4: Content area
views.u8[U8_COMPONENT_TYPE][4] = COMPONENT_BOX
views.u8[U8_VISIBLE][4] = 1
views.u8[U8_FLEX_DIRECTION][4] = 1  // column
views.f32[F32_GROW][4] = 1
views.f32[F32_WIDTH][4] = cols
views.f32[F32_PADDING_TOP][4] = 1
views.f32[F32_PADDING_LEFT][4] = 2
views.f32[F32_PADDING_RIGHT][4] = 2
views.i32[I32_PARENT_INDEX][4] = 0
views.u32[U32_BG_COLOR][4] = packColor(15, 15, 30, 255)
markDirty(views, 4, ALL_DIRTY)

// Node 5: Phase label
views.u8[U8_COMPONENT_TYPE][5] = COMPONENT_TEXT
views.u8[U8_VISIBLE][5] = 1
views.i32[I32_PARENT_INDEX][5] = 4
views.u32[U32_FG_COLOR][5] = packColor(255, 200, 100, 255) // amber
setNodeText(views, 5, '')
markDirty(views, 5, ALL_DIRTY)

// Nodes 6-15: Dynamic content area (10 boxes for stress test visuals)
const DYNAMIC_START = 6
const DYNAMIC_COUNT = 10

for (let i = 0; i < DYNAMIC_COUNT; i++) {
  const n = DYNAMIC_START + i
  views.u8[U8_COMPONENT_TYPE][n] = COMPONENT_BOX
  views.u8[U8_VISIBLE][n] = 1
  views.u8[U8_FLEX_DIRECTION][n] = 0 // row
  views.f32[F32_HEIGHT][n] = 2
  views.f32[F32_WIDTH][n] = cols - 4
  views.f32[F32_GROW][n] = 0
  views.i32[I32_PARENT_INDEX][n] = 4
  views.u32[U32_BG_COLOR][n] = packColor(25 + i * 3, 25, 50 + i * 2, 255)
  markDirty(views, n, ALL_DIRTY)
}

// Node 16: Result text
const RESULT_NODE = DYNAMIC_START + DYNAMIC_COUNT
views.u8[U8_COMPONENT_TYPE][RESULT_NODE] = COMPONENT_TEXT
views.u8[U8_VISIBLE][RESULT_NODE] = 1
views.i32[I32_PARENT_INDEX][RESULT_NODE] = 4
views.u32[U32_FG_COLOR][RESULT_NODE] = packColor(120, 255, 120, 255) // green
setNodeText(views, RESULT_NODE, '')
markDirty(views, RESULT_NODE, ALL_DIRTY)

const TOTAL_BASE_NODES = RESULT_NODE + 1
views.header[HEADER_NODE_COUNT] = TOTAL_BASE_NODES

// =============================================================================
// START ENGINE
// =============================================================================

const engine = loadEngine()
const initResult = engine.init(ptr(views.buffer), views.buffer.byteLength)
if (initResult !== 0) {
  console.error(`Engine init failed: ${initResult}`)
  process.exit(1)
}

// Give engine time to render initial frame
await Bun.sleep(200)

// =============================================================================
// HELPERS
// =============================================================================

function readRenderCount(): number {
  return Atomics.load(views.header as any, HEADER_RENDER_COUNT)
}

function updateStats(text: string) {
  setNodeText(views, 3, text)
  markDirty(views, 3, DIRTY_TEXT | DIRTY_VISUAL)
}

function updatePhase(text: string) {
  setNodeText(views, 5, text)
  markDirty(views, 5, DIRTY_TEXT | DIRTY_VISUAL)
}

function updateResult(text: string) {
  setNodeText(views, RESULT_NODE, text)
  markDirty(views, RESULT_NODE, DIRTY_TEXT | DIRTY_VISUAL)
}

function wake() {
  notifyRust(views)
}

interface PhaseResult {
  name: string
  updates: number
  renders: number
  durationMs: number
  fps: number
  updatesPerSec: number
}

const results: PhaseResult[] = []

async function runPhase(
  name: string,
  durationMs: number,
  intervalMs: number,
  updateFn: (frame: number) => void,
): Promise<PhaseResult> {
  updatePhase(`Phase: ${name}`)
  wake()
  await Bun.sleep(100)

  const rendersBefore = readRenderCount()
  const t0 = performance.now()
  let frame = 0

  while (performance.now() - t0 < durationMs) {
    updateFn(frame)
    wake()
    frame++

    const elapsed = performance.now() - t0
    const renders = readRenderCount() - rendersBefore
    const fps = renders / (elapsed / 1000)
    updateStats(`${name} | Frame: ${frame} | Renders: ${renders} | FPS: ${fps.toFixed(0)} | ${((durationMs - elapsed) / 1000).toFixed(1)}s left`)

    if (intervalMs > 0) {
      await Bun.sleep(intervalMs)
    } else {
      // Yield to let Rust process
      await Bun.sleep(0)
    }
  }

  const t1 = performance.now()
  const duration = t1 - t0
  const rendersAfter = readRenderCount()
  const totalRenders = rendersAfter - rendersBefore
  const fps = totalRenders / (duration / 1000)
  const updatesPerSec = frame / (duration / 1000)

  const result: PhaseResult = { name, updates: frame, renders: totalRenders, durationMs: duration, fps, updatesPerSec }
  results.push(result)

  updateResult(`${name}: ${totalRenders} renders in ${(duration / 1000).toFixed(1)}s = ${fps.toFixed(0)} FPS (${frame} updates, ${updatesPerSec.toFixed(0)} upd/s)`)
  wake()
  await Bun.sleep(500)

  return result
}

// =============================================================================
// PHASE 1: Color Animation (visual-only, no layout recomputation)
// =============================================================================

await runPhase('Color Animation', 5000, 16, (frame) => {
  for (let i = 0; i < DYNAMIC_COUNT; i++) {
    const n = DYNAMIC_START + i
    const hue = (frame * 3 + i * 25) % 360
    const r = Math.floor(128 + 127 * Math.sin(hue * Math.PI / 180))
    const g = Math.floor(128 + 127 * Math.sin((hue + 120) * Math.PI / 180))
    const b = Math.floor(128 + 127 * Math.sin((hue + 240) * Math.PI / 180))
    views.u32[U32_BG_COLOR][n] = packColor(r, g, b, 255)
    markDirty(views, n, DIRTY_VISUAL) // visual only — no layout
  }
})

// =============================================================================
// PHASE 2: Layout Animation (size changes, full pipeline)
// =============================================================================

await runPhase('Layout Animation', 5000, 16, (frame) => {
  for (let i = 0; i < DYNAMIC_COUNT; i++) {
    const n = DYNAMIC_START + i
    const w = Math.floor(20 + (cols - 24) * (0.5 + 0.5 * Math.sin((frame * 0.05) + i * 0.5)))
    views.f32[F32_WIDTH][n] = w
    markDirty(views, n, DIRTY_LAYOUT | DIRTY_VISUAL) // layout + visual
  }
})

// =============================================================================
// PHASE 3: Text Thrashing (rapid text content updates)
// =============================================================================

await runPhase('Text Thrashing', 5000, 16, (frame) => {
  // Reset text pool to prevent overflow
  if (frame % 100 === 0) {
    views.header[7] = 0 // HEADER_TEXT_POOL_WRITE_PTR
  }

  for (let i = 0; i < DYNAMIC_COUNT; i++) {
    const n = DYNAMIC_START + i
    // Change component to TEXT for this phase
    if (frame === 0) {
      views.u8[U8_COMPONENT_TYPE][n] = COMPONENT_TEXT
      views.u32[U32_FG_COLOR][n] = packColor(200, 200, 220, 255)
      views.u32[U32_BG_COLOR][n] = packColor(25 + i * 3, 25, 50 + i * 2, 255)
    }
    const bar = '\u2588'.repeat(Math.floor(5 + 15 * (0.5 + 0.5 * Math.sin(frame * 0.1 + i))))
    setNodeText(views, n, `Row ${i}: ${bar} [${frame}]`)
    markDirty(views, n, DIRTY_TEXT | DIRTY_VISUAL)
  }
})

// Restore dynamic nodes to boxes
for (let i = 0; i < DYNAMIC_COUNT; i++) {
  const n = DYNAMIC_START + i
  views.u8[U8_COMPONENT_TYPE][n] = COMPONENT_BOX
  markDirty(views, n, ALL_DIRTY)
}

// =============================================================================
// PHASE 4: Node Scaling (grow tree from base to 500 nodes)
// =============================================================================

await runPhase('Node Scaling', 5000, 50, (frame) => {
  const targetNodes = Math.min(500, TOTAL_BASE_NODES + frame * 5)
  const currentNodes = views.header[HEADER_NODE_COUNT]

  // Add nodes incrementally
  for (let n = currentNodes; n < targetNodes; n++) {
    views.u8[U8_COMPONENT_TYPE][n] = COMPONENT_BOX
    views.u8[U8_VISIBLE][n] = 1
    views.i32[I32_PARENT_INDEX][n] = 4 // all children of content area
    views.f32[F32_HEIGHT][n] = 1
    views.f32[F32_WIDTH][n] = Math.floor(10 + Math.random() * (cols - 20))
    views.u32[U32_BG_COLOR][n] = packColor(
      20 + Math.floor(Math.random() * 40),
      20 + Math.floor(Math.random() * 20),
      40 + Math.floor(Math.random() * 40),
      255
    )
    markDirty(views, n, ALL_DIRTY)
  }

  views.header[HEADER_NODE_COUNT] = targetNodes
  updateStats(`Node Scaling | Nodes: ${targetNodes} | Frame: ${frame}`)
})

// =============================================================================
// PHASE 5: Burst Fire (rapid updates, zero sleep, tests wake coalescing)
// =============================================================================

// Reset to base nodes
views.header[HEADER_NODE_COUNT] = TOTAL_BASE_NODES
for (let i = TOTAL_BASE_NODES; i < 500; i++) {
  views.u8[U8_COMPONENT_TYPE][i] = 0 // NONE
}

await runPhase('Burst Fire', 3000, 0, (frame) => {
  // Zero-interval updates — as fast as JS can push
  const n = DYNAMIC_START + (frame % DYNAMIC_COUNT)
  views.u32[U32_BG_COLOR][n] = packColor(
    Math.floor(Math.random() * 255),
    Math.floor(Math.random() * 255),
    Math.floor(Math.random() * 255),
    255
  )
  markDirty(views, n, DIRTY_VISUAL)
})

// =============================================================================
// SUMMARY
// =============================================================================

updatePhase('COMPLETE')

const summary = results.map(r =>
  `${r.name.padEnd(20)} ${String(r.fps.toFixed(0)).padStart(5)} FPS | ${String(r.renders).padStart(5)} renders | ${String(r.updates).padStart(7)} updates | ${String(r.updatesPerSec.toFixed(0)).padStart(7)} upd/s`
).join('\n')

updateResult(summary)
updateStats('All phases complete. Press Ctrl+C to exit.')
wake()

// Keep alive
await new Promise(() => {})
