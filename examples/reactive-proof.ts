/**
 * SparkTUI — Reactive Pipeline Proof
 *
 * Proves the FULL reactive chain:
 *
 *   signal → repeat() → SharedSlotBuffer → SharedArrayBuffer → Rust layout → output
 *
 * Tests:
 * 1. Static props: box with fixed dimensions → Rust computes layout → correct output
 * 2. Reactive props: change a signal → repeat() fires → SharedArrayBuffer updates → re-layout
 * 3. Text content: static and reactive text → text pool round-trip
 * 4. Nested children: parent/child hierarchy → correct computed positions
 * 5. Cleanup: dispose → index released → node marked NONE
 */

import { signal } from '@rlabs-inc/signals'
import { ptr } from 'bun:ffi'
import { initBridge, getViews, getArrays } from '../ts/bridge'
import { loadEngine } from '../ts/bridge/ffi'
import {
  getNodeOutput,
  getNodeText,
  setTerminalSize,
  COMPONENT_NONE,
  COMPONENT_BOX,
  COMPONENT_TEXT,
  U8_COMPONENT_TYPE,
  U32_FG_COLOR,
  I32_PARENT_INDEX,
  F32_WIDTH,
  F32_HEIGHT,
  F32_GROW,
  F32_PADDING_TOP,
  F32_PADDING_LEFT,
} from '../ts/bridge/shared-buffer'

// =============================================================================
// SETUP
// =============================================================================

console.log('=== SparkTUI Reactive Pipeline Proof ===\n')

// Init bridge (SharedArrayBuffer + ReactiveArrays + NoopNotifier for testing)
const { views, arrays } = initBridge({ noopNotifier: true })
console.log(`SharedArrayBuffer: ${views.buffer.byteLength} bytes`)

// Load Rust engine
const engine = loadEngine()
const initResult = engine.init(ptr(views.buffer), views.buffer.byteLength)
console.log(`Rust engine: ${initResult === 0 ? 'OK' : 'FAILED'}`)

// Set terminal size
setTerminalSize(views, 80, 24)

const checks: [string, boolean][] = []

// =============================================================================
// TEST 1: Direct SharedSlotBuffer writes (bypass primitives, prove arrays work)
// =============================================================================

console.log('\n--- Test 1: SharedSlotBuffer Direct Writes ---\n')

// Write a root box: 80x10, row direction, padding
arrays.componentType.set(0, COMPONENT_BOX)
arrays.visible.set(0, 1)
arrays.flexDirection.set(0, 0)  // row
arrays.width.set(0, 80)
arrays.height.set(0, 10)
arrays.paddingTop.set(0, 1)
arrays.paddingLeft.set(0, 2)
arrays.parentIndex.set(0, -1)

// Child 1: fixed 20 wide
arrays.componentType.set(1, COMPONENT_BOX)
arrays.visible.set(1, 1)
arrays.width.set(1, 20)
arrays.height.set(1, 5)
arrays.parentIndex.set(1, 0)

// Child 2: flex-grow: 1
arrays.componentType.set(2, COMPONENT_BOX)
arrays.visible.set(2, 1)
arrays.grow.set(2, 1)
arrays.height.set(2, 5)
arrays.parentIndex.set(2, 0)

// Update node count in header
views.header[1] = 3  // HEADER_NODE_COUNT

// Verify SharedArrayBuffer has the values
const rawWidth0 = views.f32[F32_WIDTH][0]
const rawHeight0 = views.f32[F32_HEIGHT][0]
const rawPadTop0 = views.f32[F32_PADDING_TOP][0]
const rawPadLeft0 = views.f32[F32_PADDING_LEFT][0]
const rawGrow2 = views.f32[F32_GROW][2]
const rawParent1 = views.i32[I32_PARENT_INDEX][1]
const rawCompType0 = views.u8[U8_COMPONENT_TYPE][0]

console.log(`  Root: ${rawWidth0}x${rawHeight0}, pad(${rawPadTop0},${rawPadLeft0}), type=${rawCompType0}`)
console.log(`  Child1: ${views.f32[F32_WIDTH][1]}x${views.f32[F32_HEIGHT][1]}, parent=${rawParent1}`)
console.log(`  Child2: grow=${rawGrow2}, parent=${views.i32[I32_PARENT_INDEX][2]}`)

checks.push(
  ['SharedSlotBuffer → SharedArrayBuffer: width', rawWidth0 === 80],
  ['SharedSlotBuffer → SharedArrayBuffer: height', rawHeight0 === 10],
  ['SharedSlotBuffer → SharedArrayBuffer: paddingTop', rawPadTop0 === 1],
  ['SharedSlotBuffer → SharedArrayBuffer: paddingLeft', rawPadLeft0 === 2],
  ['SharedSlotBuffer → SharedArrayBuffer: grow', rawGrow2 === 1],
  ['SharedSlotBuffer → SharedArrayBuffer: parentIndex', rawParent1 === 0],
  ['SharedSlotBuffer → SharedArrayBuffer: componentType', rawCompType0 === COMPONENT_BOX],
)

// =============================================================================
// TEST 2: Rust Layout Computation
// =============================================================================

console.log('\n--- Test 2: Rust Layout ---\n')

const nodesLaidOut = engine.computeLayout()
console.log(`  Nodes laid out: ${nodesLaidOut}`)

const root = getNodeOutput(views, 0)
const c1 = getNodeOutput(views, 1)
const c2 = getNodeOutput(views, 2)

console.log(`  Root: (${root.x},${root.y}) ${root.w}x${root.h}`)
console.log(`  Child1: (${c1.x},${c1.y}) ${c1.w}x${c1.h}`)
console.log(`  Child2: (${c2.x},${c2.y}) ${c2.w}x${c2.h}`)

checks.push(
  ['Layout: root width = 80', root.w === 80],
  ['Layout: root height = 10', root.h === 10],
  ['Layout: child1 width = 20', c1.w === 20],
  ['Layout: child1 x = 2 (padding_left)', c1.x === 2],
  ['Layout: child1 y = 1 (padding_top)', c1.y === 1],
  ['Layout: child2 grows to fill', c2.w === 80 - 2 - 20],
  ['Layout: child2 after child1', c2.x === 2 + 20],
)

// =============================================================================
// TEST 3: Reactive Signal → SharedArrayBuffer → Re-layout
// =============================================================================

console.log('\n--- Test 3: Reactive Signal → Re-layout ---\n')

// Create a signal and use repeat() to wire it
const widthSignal = signal(30)
import { repeat } from '@rlabs-inc/signals'

// Wire signal to child1's width
const disposeRepeat = repeat(widthSignal, arrays.width, 1)

// Signal should have written initial value
const afterRepeat = views.f32[F32_WIDTH][1]
console.log(`  After repeat(30): child1 width = ${afterRepeat}`)
checks.push(['repeat() writes initial value', afterRepeat === 30])

// Re-layout with new width
engine.computeLayout()
const c1After = getNodeOutput(views, 1)
const c2After = getNodeOutput(views, 2)
console.log(`  Layout: child1=${c1After.w}, child2=${c2After.w}`)
checks.push(
  ['Re-layout: child1 = 30', c1After.w === 30],
  ['Re-layout: child2 fills remaining', c2After.w === 80 - 2 - 30],
)

// Now change the signal → repeat() fires inline → SharedArrayBuffer updates
widthSignal.value = 40
const afterChange = views.f32[F32_WIDTH][1]
console.log(`  After signal change to 40: raw = ${afterChange}`)
checks.push(['Signal change → SharedArrayBuffer update', afterChange === 40])

// Re-layout
engine.computeLayout()
const c1Final = getNodeOutput(views, 1)
const c2Final = getNodeOutput(views, 2)
console.log(`  Layout: child1=${c1Final.w}, child2=${c2Final.w}`)
checks.push(
  ['Reactive re-layout: child1 = 40', c1Final.w === 40],
  ['Reactive re-layout: child2 fills remaining', c2Final.w === 80 - 2 - 40],
)

// Cleanup
disposeRepeat()

// =============================================================================
// TEST 4: Reactive Getter → SharedArrayBuffer
// =============================================================================

console.log('\n--- Test 4: Getter (Inline Derived) ---\n')

const baseWidth = signal(10)
const disposeGetter = repeat(() => baseWidth.value * 3, arrays.width, 1)

const getterInitial = views.f32[F32_WIDTH][1]
console.log(`  After repeat(() => 10 * 3): width = ${getterInitial}`)
checks.push(['Getter initial: 10 * 3 = 30', getterInitial === 30])

baseWidth.value = 15
const getterAfter = views.f32[F32_WIDTH][1]
console.log(`  After signal = 15: width = ${getterAfter}`)
checks.push(['Getter reactive: 15 * 3 = 45', getterAfter === 45])

disposeGetter()

// =============================================================================
// TEST 5: Color Packing via SharedSlotBuffer
// =============================================================================

console.log('\n--- Test 5: Color Packing ---\n')

import { packColor, unpackColor } from '../ts/bridge/shared-buffer'

const red = packColor(255, 0, 0, 255)
arrays.fgColor.set(0, red)

const rawFg = views.u32[U32_FG_COLOR][0]
const unpacked = unpackColor(rawFg)
console.log(`  Packed red: 0x${rawFg.toString(16)}, unpacked: r=${unpacked.r} g=${unpacked.g} b=${unpacked.b} a=${unpacked.a}`)

checks.push(
  ['Color: packed correctly', rawFg === red],
  ['Color: red channel', unpacked.r === 255],
  ['Color: green channel', unpacked.g === 0],
  ['Color: alpha channel', unpacked.a === 255],
)

// =============================================================================
// TEST 6: Text Pool via SharedSlotBuffer
// =============================================================================

console.log('\n--- Test 6: Text Pool ---\n')

import { setNodeText } from '../ts/bridge/shared-buffer'

// Static text
setNodeText(views, 0, 'Hello, SparkTUI!')
const text0 = getNodeText(views, 0)
console.log(`  Static text: "${text0}"`)
checks.push(['Static text round-trip', text0 === 'Hello, SparkTUI!'])

// Unicode text
setNodeText(views, 1, 'Unicode: 日本語')
const text1 = getNodeText(views, 1)
console.log(`  Unicode text: "${text1}"`)
checks.push(['Unicode text round-trip', text1 === 'Unicode: 日本語'])

// Reactive text via repeater (simulates what text.ts does)
const textSignal = signal('Initial')

// Simulate writeTextToPool pattern: readFn encodes + writes pool + returns offset
const textEncoder = new TextEncoder()
const disposeText = repeat(
  () => {
    const str = String(textSignal.value)
    const encoded = textEncoder.encode(str)
    const writePtr = views.header[7] // HEADER_TEXT_POOL_WRITE_PTR
    const capacity = views.header[8] // HEADER_TEXT_POOL_CAPACITY
    if (writePtr + encoded.length > capacity) return writePtr // skip if full
    views.textPool.set(encoded, writePtr)
    views.u32[11][2] = encoded.length // U32_TEXT_LENGTH for node 2
    views.header[7] = writePtr + encoded.length
    return writePtr
  },
  arrays.textOffset,
  2
)

const textInit = getNodeText(views, 2)
console.log(`  Reactive text initial: "${textInit}"`)
checks.push(['Reactive text initial', textInit === 'Initial'])

// Change signal → repeater fires inline → pool updates
textSignal.value = 'Changed!'
const textAfter = getNodeText(views, 2)
console.log(`  Reactive text after change: "${textAfter}"`)
checks.push(['Reactive text after change', textAfter === 'Changed!'])

disposeText()

// =============================================================================
// TEST 7: Cleanup — dispose releases node
// =============================================================================

console.log('\n--- Test 7: Cleanup ---\n')

// Set node 2 to NONE (simulates what registry does on release)
arrays.componentType.set(2, COMPONENT_NONE)
const releasedType = views.u8[U8_COMPONENT_TYPE][2]
console.log(`  After release: componentType = ${releasedType}`)
checks.push(['Cleanup: componentType = NONE', releasedType === COMPONENT_NONE])

// =============================================================================
// RESULTS
// =============================================================================

console.log('\n--- Results ---\n')

let passed = 0
for (const [name, ok] of checks) {
  console.log(`  ${ok ? '✓' : '✗'} ${name}`)
  if (ok) passed++
}

console.log(`\n${passed}/${checks.length} checks passed`)

engine.close()

if (passed === checks.length) {
  console.log('\n🎉 Reactive pipeline VERIFIED! Signal → repeat() → SharedArrayBuffer → Rust layout → output.\n')
} else {
  console.log('\nSome checks failed — debug needed.\n')
  process.exit(1)
}
