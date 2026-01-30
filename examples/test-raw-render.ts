/**
 * Raw render test - bypass TS primitives, test Rust pipeline directly
 */

import { ptr } from 'bun:ffi'
import { join } from 'path'
import {
  createAoSBuffer,
  setTerminalSize,
  setNodeText,
  setNodeCount,
  packColor,
  createNodeWriter,
  COMPONENT_BOX,
  COMPONENT_TEXT,
  DIRTY_LAYOUT,
  DIRTY_VISUAL,
  DIRTY_TEXT,
  DIRTY_HIERARCHY,
} from '../ts/bridge/shared-buffer-aos'
import { loadEngine } from '../ts/bridge/ffi'

// Build
console.log('[test] Building...')
Bun.spawnSync({
  cmd: ['cargo', 'build', '--release'],
  cwd: join(import.meta.dir, '../rust'),
  stdout: 'inherit',
  stderr: 'inherit',
})

// Create buffer
console.log('[test] Creating buffer...')
const buf = createAoSBuffer()

// Terminal size
const cols = process.stdout.columns || 80
const rows = process.stdout.rows || 24
setTerminalSize(buf, cols, rows)
console.log(`[test] Terminal: ${cols}x${rows}`)

const ALL_DIRTY = DIRTY_LAYOUT | DIRTY_VISUAL | DIRTY_TEXT | DIRTY_HIERARCHY

// Node 0: Root (full terminal, dark blue bg)
const root = createNodeWriter(buf, 0)
root.componentType = COMPONENT_BOX
root.visible = 1
root.width = cols
root.height = rows
root.parentIndex = -1
root.bgColor = packColor(30, 30, 46, 255) // dark blue
root.markDirty(ALL_DIRTY)

// Node 1: Text "Hello SparkTUI!"
const text = createNodeWriter(buf, 1)
text.componentType = COMPONENT_TEXT
text.visible = 1
text.parentIndex = 0
text.fgColor = packColor(255, 255, 255, 255) // white
setNodeText(buf, 1, 'Hello SparkTUI!')
text.markDirty(ALL_DIRTY)

// Set node count
setNodeCount(buf, 2)

// Debug: verify dirty flags were written
const view = new DataView(buf.buffer)
const node0DirtyOffset = 256 + 0 * 256 + 172 // HEADER + node*STRIDE + U_DIRTY_FLAGS
const node1DirtyOffset = 256 + 1 * 256 + 172
console.log(`[test] Dirty flags: node0=${view.getUint8(node0DirtyOffset).toString(2).padStart(8,'0')}, node1=${view.getUint8(node1DirtyOffset).toString(2).padStart(8,'0')}`)
console.log(`[test] Created 2 nodes`)

// Load engine
console.log('[test] Loading engine...')
const engine = loadEngine()

console.log('[test] Calling spark_init...')
const result = engine.init(ptr(buf.buffer), buf.buffer.byteLength)
console.log(`[test] spark_init returned: ${result}`)

if (result !== 0) {
  console.error('[test] Engine init failed!')
  process.exit(1)
}

console.log('[test] Engine running. Press Ctrl+C to exit.')

// Wait
await new Promise(() => {})
