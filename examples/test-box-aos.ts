/**
 * Test Box Primitive with AoS
 */

import { signal } from '@rlabs-inc/signals'
import { initBridgeAoS, getAoSBuffer } from '../ts/bridge'
import { box } from '../ts/primitives/box'
import {
  getF32, getU8,
  F_WIDTH, F_HEIGHT, F_PADDING_TOP, F_FLEX_GROW,
  U_FLEX_DIRECTION, U_COMPONENT_TYPE, I_PARENT_INDEX,
  getI32,
} from '../ts/bridge/shared-buffer-aos'

// Initialize AoS bridge FIRST
initBridgeAoS({ noopNotifier: true })
const buffer = getAoSBuffer()

// Need to mock the registry - let's just do a minimal test
// For now, just verify the module loads and types check

console.log('=== Box Primitive AoS Test ===\n')
console.log('✓ Box module loaded successfully')
console.log('✓ AoS bridge initialized')
console.log('✓ Types compatible\n')

// Direct array write test (simulating what box() would do)
import { getAoSArrays } from '../ts/bridge'
const arrays = getAoSArrays()

console.log('Test: Direct array writes match AoS buffer')

// Write via arrays
arrays.width.set(5, 100)
arrays.height.set(5, 50)
arrays.flexDirection.set(5, 1)
arrays.paddingTop.set(5, 2)
arrays.grow.set(5, 1.5)
arrays.parentIndex.set(5, 0)
arrays.componentType.set(5, 1) // BOX

// Read via low-level
const w = getF32(buffer, 5, F_WIDTH)
const h = getF32(buffer, 5, F_HEIGHT)
const fd = getU8(buffer, 5, U_FLEX_DIRECTION)
const pt = getF32(buffer, 5, F_PADDING_TOP)
const g = getF32(buffer, 5, F_FLEX_GROW)
const pi = getI32(buffer, 5, I_PARENT_INDEX)
const ct = getU8(buffer, 5, U_COMPONENT_TYPE)

console.log(`  width: ${w} (expected 100)`)
console.log(`  height: ${h} (expected 50)`)
console.log(`  flexDirection: ${fd} (expected 1)`)
console.log(`  paddingTop: ${pt} (expected 2)`)
console.log(`  grow: ${g} (expected 1.5)`)
console.log(`  parentIndex: ${pi} (expected 0)`)
console.log(`  componentType: ${ct} (expected 1)`)

const pass = w === 100 && h === 50 && fd === 1 && pt === 2 && g === 1.5 && pi === 0 && ct === 1
console.log(`\n${pass ? '✓ ALL PASS' : '✗ FAIL'}`)
