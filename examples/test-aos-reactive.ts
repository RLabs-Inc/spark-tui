/**
 * Test AoS Reactive Arrays (Fast Path)
 *
 * Verifies direct DataView writes work with repeat().
 */

import { signal, repeat } from '@rlabs-inc/signals'
import { initBridgeAoS, getAoSArrays, getAoSBuffer } from '../ts/bridge'
import { getF32, F_WIDTH, F_HEIGHT, getU8, U_FLEX_DIRECTION } from '../ts/bridge/shared-buffer-aos'

// Initialize AoS bridge
const { buffer, arrays } = initBridgeAoS({ noopNotifier: true })

console.log('=== AoS Reactive Arrays Test (Fast Path) ===\n')

// Test 1: Direct write via arrays
console.log('Test 1: Direct write via arrays.width.set()')
arrays.width.set(0, 100)
arrays.height.set(0, 50)
arrays.flexDirection.set(0, 1) // column

// Read back via low-level API
const w = getF32(buffer, 0, F_WIDTH)
const h = getF32(buffer, 0, F_HEIGHT)
const fd = getU8(buffer, 0, U_FLEX_DIRECTION)

console.log(`  Node 0: width=${w}, height=${h}, flexDirection=${fd}`)
console.log(`  Expected: width=100, height=50, flexDirection=1`)
console.log(`  ${w === 100 && h === 50 && fd === 1 ? '✓ PASS' : '✗ FAIL'}\n`)

// Test 2: repeat() with static value
console.log('Test 2: repeat() with static value')
const dispose1 = repeat(200, arrays.width, 1)
const w1 = getF32(buffer, 1, F_WIDTH)
console.log(`  Node 1: width=${w1}`)
console.log(`  Expected: width=200`)
console.log(`  ${w1 === 200 ? '✓ PASS' : '✗ FAIL'}\n`)

// Test 3: repeat() with signal (reactive)
console.log('Test 3: repeat() with signal (reactive)')
const widthSignal = signal(300)
const dispose2 = repeat(widthSignal, arrays.width, 2)

const w2_before = getF32(buffer, 2, F_WIDTH)
console.log(`  Node 2 before: width=${w2_before}`)

widthSignal.value = 400
const w2_after = getF32(buffer, 2, F_WIDTH)
console.log(`  Node 2 after signal change: width=${w2_after}`)
console.log(`  Expected: 300 → 400`)
console.log(`  ${w2_before === 300 && w2_after === 400 ? '✓ PASS' : '✗ FAIL'}\n`)

// Test 4: repeat() with getter (reactive)
console.log('Test 4: repeat() with getter (reactive)')
const baseWidth = signal(100)
const dispose3 = repeat(() => baseWidth.value * 2, arrays.width, 3)

const w3_before = getF32(buffer, 3, F_WIDTH)
console.log(`  Node 3 before: width=${w3_before}`)

baseWidth.value = 150
const w3_after = getF32(buffer, 3, F_WIDTH)
console.log(`  Node 3 after signal change: width=${w3_after}`)
console.log(`  Expected: 200 → 300`)
console.log(`  ${w3_before === 200 && w3_after === 300 ? '✓ PASS' : '✗ FAIL'}\n`)

// Test 5: Multiple nodes, verify isolation
console.log('Test 5: Multiple nodes isolation')
arrays.width.set(10, 1000)
arrays.width.set(11, 1100)
arrays.width.set(12, 1200)

const w10 = getF32(buffer, 10, F_WIDTH)
const w11 = getF32(buffer, 11, F_WIDTH)
const w12 = getF32(buffer, 12, F_WIDTH)

console.log(`  Node 10: ${w10}, Node 11: ${w11}, Node 12: ${w12}`)
console.log(`  Expected: 1000, 1100, 1200`)
console.log(`  ${w10 === 1000 && w11 === 1100 && w12 === 1200 ? '✓ PASS' : '✗ FAIL'}\n`)

// Cleanup
dispose1()
dispose2()
dispose3()

console.log('=== All tests complete ===')
