/**
 * Debug: Test wake mechanism directly
 */

import { signal } from '@rlabs-inc/signals'
import { box, text } from '../ts/primitives'
import { mountSync } from '../ts/engine'
import { getRenderCount, wakeRust } from '../ts/bridge/shared-buffer'

const counter = signal(0)

const handle = mountSync(() => {
  box({
    width: 30,
    height: 5,
    children: () => {
      text({ content: counter })
    }
  })
}, { mode: 'inline' })

await Bun.sleep(300)

console.log('Initial render count:', getRenderCount(handle.buffer))

// Test 1: Direct wakeRust call
console.log('\nCalling wakeRust() directly...')
wakeRust(handle.buffer)
await Bun.sleep(100)
console.log('After wakeRust + 100ms:', getRenderCount(handle.buffer))

// Test 2: Signal update
console.log('\nUpdating counter.value...')
counter.value = 42
await Bun.sleep(100)
console.log('After counter=42 + 100ms:', getRenderCount(handle.buffer))

// Test 3: Check wake flag manually
console.log('\nChecking wake flag state...')
const wakeIdx = 64 / 4  // H_WAKE_RUST / 4
const flagBefore = Atomics.load(handle.buffer.headerI32, wakeIdx)
console.log('Wake flag before store:', flagBefore)

Atomics.store(handle.buffer.headerI32, wakeIdx, 1)
const flagAfter = Atomics.load(handle.buffer.headerI32, wakeIdx)
console.log('Wake flag after store:', flagAfter)

await Bun.sleep(100)
const flagConsumed = Atomics.load(handle.buffer.headerI32, wakeIdx)
console.log('Wake flag after 100ms (should be 0 if Rust consumed):', flagConsumed)
console.log('Render count now:', getRenderCount(handle.buffer))

handle.unmount()
process.exit(0)
