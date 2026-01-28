/**
 * SparkTUI — Hello World
 *
 * The first thing we ever render to the terminal:
 *
 *   TS writes props → SharedArrayBuffer → Rust engine starts →
 *   reactive graph evaluates → layout → framebuffer → diff render → terminal
 *
 * No FFI calls after init. No loops. No polling.
 * Data written BEFORE init gets picked up on the engine's first effect evaluation.
 */

import { ptr } from 'bun:ffi'
import { loadEngine } from '../ts/bridge/ffi'
import {
  createSharedBuffer,
  setTerminalSize,
  setNodeText,
  markDirty,
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
} from '../ts/bridge/shared-buffer'

// =============================================================================
// 1. CREATE SHARED BUFFER
// =============================================================================

const views = createSharedBuffer()

// Terminal size — use actual terminal dimensions
const cols = process.stdout.columns || 80
const rows = process.stdout.rows || 24
setTerminalSize(views, cols, rows)

// =============================================================================
// 2. WRITE COMPONENT TREE
// =============================================================================

// Node 0: Root container (full terminal, column direction, centered)
views.u8[U8_COMPONENT_TYPE][0] = COMPONENT_BOX
views.u8[U8_VISIBLE][0] = 1
views.u8[U8_FLEX_DIRECTION][0] = 1  // column
views.u8[U8_JUSTIFY_CONTENT][0] = 2 // center
views.u8[U8_ALIGN_ITEMS][0] = 2     // center
views.f32[F32_WIDTH][0] = cols
views.f32[F32_HEIGHT][0] = rows
views.i32[I32_PARENT_INDEX][0] = -1  // root
views.u32[U32_BG_COLOR][0] = packColor(20, 20, 40, 255) // dark blue-ish bg

// Node 1: Inner box (bordered card)
views.u8[U8_COMPONENT_TYPE][1] = COMPONENT_BOX
views.u8[U8_VISIBLE][1] = 1
views.u8[U8_FLEX_DIRECTION][1] = 1  // column
views.u8[U8_JUSTIFY_CONTENT][1] = 2 // center
views.u8[U8_ALIGN_ITEMS][1] = 2     // center
views.f32[F32_WIDTH][1] = 40
views.f32[F32_HEIGHT][1] = 7
views.f32[F32_PADDING_TOP][1] = 1
views.f32[F32_PADDING_BOTTOM][1] = 1
views.f32[F32_PADDING_LEFT][1] = 2
views.f32[F32_PADDING_RIGHT][1] = 2
views.i32[I32_PARENT_INDEX][1] = 0

// Border
views.u8[U8_BORDER_STYLE][1] = 2     // rounded
views.u8[U8_BORDER_TOP_WIDTH][1] = 1
views.u8[U8_BORDER_RIGHT_WIDTH][1] = 1
views.u8[U8_BORDER_BOTTOM_WIDTH][1] = 1
views.u8[U8_BORDER_LEFT_WIDTH][1] = 1
views.u32[U32_BORDER_COLOR][1] = packColor(100, 180, 255, 255) // light blue border
views.u32[U32_BG_COLOR][1] = packColor(30, 30, 60, 255)        // slightly lighter bg
views.u32[U32_FG_COLOR][1] = packColor(255, 255, 255, 255)     // white text

// Node 2: Title text
views.u8[U8_COMPONENT_TYPE][2] = COMPONENT_TEXT
views.u8[U8_VISIBLE][2] = 1
views.i32[I32_PARENT_INDEX][2] = 1
views.u32[U32_FG_COLOR][2] = packColor(100, 220, 255, 255) // cyan
setNodeText(views, 2, 'SparkTUI')

// Node 3: Subtitle text
views.u8[U8_COMPONENT_TYPE][3] = COMPONENT_TEXT
views.u8[U8_VISIBLE][3] = 1
views.i32[I32_PARENT_INDEX][3] = 1
views.u32[U32_FG_COLOR][3] = packColor(180, 180, 200, 255) // muted text
setNodeText(views, 3, 'Hello from the hybrid frontier')

// Node 4: Footer text
views.u8[U8_COMPONENT_TYPE][4] = COMPONENT_TEXT
views.u8[U8_VISIBLE][4] = 1
views.i32[I32_PARENT_INDEX][4] = 1
views.u32[U32_FG_COLOR][4] = packColor(80, 80, 120, 255) // dimmed
setNodeText(views, 4, 'Press Ctrl+C to exit')

// =============================================================================
// 3. SET HEADER + DIRTY FLAGS
// =============================================================================

views.header[HEADER_NODE_COUNT] = 5

// Mark all nodes dirty (layout + visual + text + hierarchy)
const ALL_DIRTY = DIRTY_LAYOUT | DIRTY_VISUAL | DIRTY_TEXT | DIRTY_HIERARCHY
for (let i = 0; i < 5; i++) {
  markDirty(views, i, ALL_DIRTY)
}

// =============================================================================
// 4. START ENGINE
// =============================================================================
//
// spark_init() starts the engine thread which:
//   1. Enters fullscreen (alt screen, raw mode, mouse reporting)
//   2. Creates reactive graph (generation → layout → framebuffer → render)
//   3. Initial effect evaluation picks up our data and renders
//   4. Blocks on channel for stdin/wake events
//
// Data written BEFORE init is picked up on the first reactive evaluation.
// After this call, the terminal belongs to Rust.

const engine = loadEngine()
const result = engine.init(ptr(views.buffer), views.buffer.byteLength)

if (result !== 0) {
  console.error(`[hello-world] Engine init failed: ${result}`)
  process.exit(1)
}

// =============================================================================
// 5. STAY ALIVE
// =============================================================================
//
// The engine thread handles stdin (Ctrl+C → exit).
// We just need to keep the process alive.

await new Promise(() => {})
