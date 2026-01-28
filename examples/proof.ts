/**
 * SparkTUI Proof of Concept
 *
 * Proves the round-trip:
 * 1. TS creates SharedArrayBuffer (v2, ~2MB)
 * 2. TS writes box layout data (as primitives would)
 * 3. Rust receives pointer via FFI
 * 4. Rust reads the data, runs Taffy, writes computed layout back
 * 5. TS reads the output
 * 6. TS writes visual/text/interaction data, verifies structure
 *
 * This is NOT how the real system works (Rust renders to terminal directly).
 * This just proves the shared memory contract is correct on both sides.
 */

import { ptr } from 'bun:ffi'
import { loadEngine } from '../ts/bridge/ffi'
import {
  createSharedBuffer,
  setNodeMeta,
  setNodeFloat,
  setNodeColor,
  setNodeInteraction,
  setNodeParent,
  setTerminalSize,
  setNodeCount,
  setNodeText,
  getNodeText,
  getNodeOutput,
  getNodeMeta,
  getNodeColor,
  getNodeInteraction,
  markDirty,
  packColor,
  unpackColor,
  COMPONENT_BOX,
  COMPONENT_TEXT,
  META_COMPONENT_TYPE,
  META_VISIBLE,
  META_FLEX_DIRECTION,
  META_BORDER_STYLE,
  META_OPACITY,
  META_Z_INDEX,
  META_TEXT_ALIGN,
  META_TEXT_WRAP,
  META_FOCUSABLE,
  META_DIRTY_FLAGS,
  FLOAT_WIDTH,
  FLOAT_HEIGHT,
  FLOAT_PADDING_TOP,
  FLOAT_PADDING_LEFT,
  FLOAT_GROW,
  COLOR_FG,
  COLOR_BG,
  COLOR_BORDER,
  INTERACT_TAB_INDEX,
  INTERACT_SCROLL_X,
  INTERACT_CURSOR_POS,
  DIRTY_LAYOUT,
  DIRTY_VISUAL,
  DIRTY_TEXT,
  TOTAL_BUFFER_SIZE,
  HEADER_TEXT_POOL_WRITE_PTR,
} from '../ts/bridge/shared-buffer'

// =============================================================================
// SETUP
// =============================================================================

console.log('=== SparkTUI Proof of Concept (v2 Buffer) ===\n')
console.log(`Buffer size: ${(TOTAL_BUFFER_SIZE / 1024).toFixed(1)}KB (~${(TOTAL_BUFFER_SIZE / 1024 / 1024).toFixed(1)}MB)`)

// Create shared buffer
const views = createSharedBuffer()
console.log(`SharedArrayBuffer created: ${views.buffer.byteLength} bytes`)

// Load Rust engine
const engine = loadEngine()
console.log('Rust engine loaded')

// Pass buffer pointer to Rust
const bufferPtr = ptr(views.buffer)
const initResult = engine.init(bufferPtr, views.buffer.byteLength)
console.log(`Engine initialized: ${initResult === 0 ? 'OK' : 'FAILED'}`)

// =============================================================================
// TEST 1: Layout round-trip (same as v1)
// =============================================================================

console.log('\n--- Test 1: Layout Round-Trip ---\n')

setTerminalSize(views, 80, 24)

// Root box: full width, 10 rows, row direction, padding
const ROOT = 0
setNodeMeta(views, ROOT, META_COMPONENT_TYPE, COMPONENT_BOX)
setNodeMeta(views, ROOT, META_VISIBLE, 1)
setNodeMeta(views, ROOT, META_FLEX_DIRECTION, 1) // row
setNodeFloat(views, ROOT, FLOAT_WIDTH, 80)
setNodeFloat(views, ROOT, FLOAT_HEIGHT, 10)
setNodeFloat(views, ROOT, FLOAT_PADDING_TOP, 1)
setNodeFloat(views, ROOT, FLOAT_PADDING_LEFT, 2)

// Child 1: fixed 20 wide
const CHILD1 = 1
setNodeMeta(views, CHILD1, META_COMPONENT_TYPE, COMPONENT_BOX)
setNodeMeta(views, CHILD1, META_VISIBLE, 1)
setNodeFloat(views, CHILD1, FLOAT_WIDTH, 20)
setNodeFloat(views, CHILD1, FLOAT_HEIGHT, 5)
setNodeParent(views, CHILD1, ROOT)

// Child 2: flex-grow: 1 (fills remaining space)
const CHILD2 = 2
setNodeMeta(views, CHILD2, META_COMPONENT_TYPE, COMPONENT_BOX)
setNodeMeta(views, CHILD2, META_VISIBLE, 1)
setNodeFloat(views, CHILD2, FLOAT_GROW, 1)
setNodeFloat(views, CHILD2, FLOAT_HEIGHT, 5)
setNodeParent(views, CHILD2, ROOT)

setNodeCount(views, 3)

const nodesLaidOut = engine.computeLayout()
console.log(`Nodes laid out: ${nodesLaidOut}`)

const root = getNodeOutput(views, ROOT)
const c1 = getNodeOutput(views, CHILD1)
const c2 = getNodeOutput(views, CHILD2)

const checks: [string, boolean][] = [
  ['Root width = 80', root.w === 80],
  ['Root height = 10', root.h === 10],
  ['Child1 width = 20', c1.w === 20],
  ['Child1 x = 2 (padding_left)', c1.x === 2],
  ['Child1 y = 1 (padding_top)', c1.y === 1],
  ['Child2 grows to fill', c2.w === 80 - 2 - 20],
  ['Child2 after child1', c2.x === 2 + 20],
]

// =============================================================================
// TEST 2: Color section
// =============================================================================

console.log('\n--- Test 2: Color Section ---\n')

const red = packColor(255, 0, 0)
const green = packColor(0, 255, 0, 128)
setNodeColor(views, ROOT, COLOR_FG, red)
setNodeColor(views, ROOT, COLOR_BG, green)
setNodeColor(views, ROOT, COLOR_BORDER, packColor(100, 100, 100))

const fgRead = getNodeColor(views, ROOT, COLOR_FG)
const bgRead = getNodeColor(views, ROOT, COLOR_BG)
const fgUnpacked = unpackColor(fgRead)
const bgUnpacked = unpackColor(bgRead)

checks.push(
  ['FG color packed correctly', fgUnpacked.r === 255 && fgUnpacked.g === 0 && fgUnpacked.b === 0 && fgUnpacked.a === 255],
  ['BG color with alpha', bgUnpacked.r === 0 && bgUnpacked.g === 255 && bgUnpacked.b === 0 && bgUnpacked.a === 128],
  ['Border color set', getNodeColor(views, ROOT, COLOR_BORDER) !== 0],
  ['Unset color = 0 (inherit)', getNodeColor(views, CHILD1, COLOR_FG) === 0],
)

// =============================================================================
// TEST 3: Text pool
// =============================================================================

console.log('\n--- Test 3: Text Pool ---\n')

setNodeText(views, ROOT, 'Hello, SparkTUI!')
setNodeText(views, CHILD1, 'Child 1 text')
setNodeText(views, CHILD2, 'Unicode: 日本語テキスト')

const rootText = getNodeText(views, ROOT)
const c1Text = getNodeText(views, CHILD1)
const c2Text = getNodeText(views, CHILD2)

console.log(`  Root text: "${rootText}"`)
console.log(`  Child1 text: "${c1Text}"`)
console.log(`  Child2 text: "${c2Text}"`)
console.log(`  Text pool write ptr: ${views.header[HEADER_TEXT_POOL_WRITE_PTR]}`)

checks.push(
  ['Root text round-trip', rootText === 'Hello, SparkTUI!'],
  ['Child1 text round-trip', c1Text === 'Child 1 text'],
  ['Unicode text round-trip', c2Text === 'Unicode: 日本語テキスト'],
  ['Text pool write ptr advanced', views.header[HEADER_TEXT_POOL_WRITE_PTR] > 0],
)

// =============================================================================
// TEST 4: Visual metadata
// =============================================================================

console.log('\n--- Test 4: Visual Metadata ---\n')

setNodeMeta(views, ROOT, META_BORDER_STYLE, 3) // rounded
setNodeMeta(views, ROOT, META_OPACITY, 200)     // ~78% opacity
setNodeMeta(views, ROOT, META_Z_INDEX, 200)     // above default

checks.push(
  ['Border style set', getNodeMeta(views, ROOT, META_BORDER_STYLE) === 3],
  ['Opacity set', getNodeMeta(views, ROOT, META_OPACITY) === 200],
  ['Z-index set', getNodeMeta(views, ROOT, META_Z_INDEX) === 200],
  ['Default opacity = 255', getNodeMeta(views, CHILD1, META_OPACITY) === 255],
  ['Default z-index = 128', getNodeMeta(views, CHILD1, META_Z_INDEX) === 128],
)

// =============================================================================
// TEST 5: Text metadata
// =============================================================================

console.log('\n--- Test 5: Text Metadata ---\n')

setNodeMeta(views, CHILD1, META_COMPONENT_TYPE, COMPONENT_TEXT)
setNodeMeta(views, CHILD1, META_TEXT_ALIGN, 1) // center
setNodeMeta(views, CHILD1, META_TEXT_WRAP, 2)  // truncate

checks.push(
  ['Text align set', getNodeMeta(views, CHILD1, META_TEXT_ALIGN) === 1],
  ['Text wrap set', getNodeMeta(views, CHILD1, META_TEXT_WRAP) === 2],
  ['Default text wrap = 1 (wrap)', getNodeMeta(views, CHILD2, META_TEXT_WRAP) === 1],
)

// =============================================================================
// TEST 6: Interaction section
// =============================================================================

console.log('\n--- Test 6: Interaction Section ---\n')

setNodeMeta(views, CHILD1, META_FOCUSABLE, 1)
setNodeInteraction(views, CHILD1, INTERACT_TAB_INDEX, 0)
setNodeInteraction(views, CHILD1, INTERACT_CURSOR_POS, 5)
setNodeInteraction(views, ROOT, INTERACT_SCROLL_X, 10)

checks.push(
  ['Focusable set', getNodeMeta(views, CHILD1, META_FOCUSABLE) === 1],
  ['Tab index set', getNodeInteraction(views, CHILD1, INTERACT_TAB_INDEX) === 0],
  ['Cursor position set', getNodeInteraction(views, CHILD1, INTERACT_CURSOR_POS) === 5],
  ['Scroll X set', getNodeInteraction(views, ROOT, INTERACT_SCROLL_X) === 10],
  ['Default tab index = -1', getNodeInteraction(views, CHILD2, INTERACT_TAB_INDEX) === -1],
)

// =============================================================================
// TEST 7: Dirty flags
// =============================================================================

console.log('\n--- Test 7: Dirty Flags ---\n')

markDirty(views, ROOT, DIRTY_LAYOUT | DIRTY_VISUAL)
markDirty(views, CHILD1, DIRTY_TEXT)

const rootDirty = getNodeMeta(views, ROOT, META_DIRTY_FLAGS)
const c1Dirty = getNodeMeta(views, CHILD1, META_DIRTY_FLAGS)

checks.push(
  ['Root dirty: layout+visual', (rootDirty & DIRTY_LAYOUT) !== 0 && (rootDirty & DIRTY_VISUAL) !== 0],
  ['Root not dirty: text', (rootDirty & DIRTY_TEXT) === 0],
  ['Child1 dirty: text', (c1Dirty & DIRTY_TEXT) !== 0],
  ['Child2 not dirty', getNodeMeta(views, CHILD2, META_DIRTY_FLAGS) === 0],
)

// =============================================================================
// TEST 8: Buffer size
// =============================================================================

console.log('\n--- Test 8: Buffer Size ---\n')

checks.push(
  ['Buffer size ~2MB', TOTAL_BUFFER_SIZE > 2_000_000 && TOTAL_BUFFER_SIZE < 2_200_000],
  ['Buffer version = 2', views.header[0] === 2],
)

// =============================================================================
// RESULTS
// =============================================================================

console.log('\n--- Verification ---\n')

let passed = 0
for (const [name, ok] of checks) {
  console.log(`  ${ok ? '✓' : '✗'} ${name}`)
  if (ok) passed++
}

console.log(`\n${passed}/${checks.length} checks passed`)

// Cleanup
engine.close()

if (passed === checks.length) {
  console.log('\n🎉 SharedArrayBuffer v2 contract VERIFIED! All sections work.\n')
} else {
  console.log('\nSome checks failed - debug needed.\n')
  process.exit(1)
}
