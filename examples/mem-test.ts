/**
 * Memory diagnostic - isolate where the leak is
 */

import { signal } from '@rlabs-inc/signals'
import { box, text } from '../ts/primitives'
import { mountSync } from '../ts/engine'

function getMemoryMB(): number {
  return Math.round(process.memoryUsage().heapUsed / 1024 / 1024)
}

function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms))
}

console.log('Memory Diagnostic')
console.log('=================\n')

console.log(`Initial memory: ${getMemoryMB()}MB`)

// Test 1: Pure signal updates (no engine)
console.log('\n--- Test 1: Pure Signal Updates ---')
{
  const s = signal(0)
  const before = getMemoryMB()

  for (let i = 0; i < 100_000; i++) {
    s.value = i
  }

  // Force GC if available
  if (global.gc) global.gc()

  const after = getMemoryMB()
  console.log(`Before: ${before}MB, After: ${after}MB, Delta: ${after - before}MB`)
}

// Test 2: Mount only (no updates)
console.log('\n--- Test 2: Mount Only ---')
{
  const before = getMemoryMB()

  const counter = signal(0)
  const app = mountSync(() => {
    box({
      children: () => {
        text({ content: counter })
      },
    })
  }, { mode: 'inline' })

  await sleep(500) // Let it render

  const after = getMemoryMB()
  console.log(`Before: ${before}MB, After: ${after}MB, Delta: ${after - before}MB`)
  console.log(`(SharedBuffer is ~20MB, so this is expected)`)

  // Test 3: Signal updates WITH engine
  console.log('\n--- Test 3: Signal Updates With Engine ---')
  {
    const before = getMemoryMB()

    for (let i = 0; i < 1000; i++) {
      counter.value = i
      await sleep(0)
    }

    const after = getMemoryMB()
    console.log(`Before: ${before}MB, After: ${after}MB, Delta: ${after - before}MB`)
  }

  // Test 4: Burst updates (no await)
  console.log('\n--- Test 4: Burst Updates (no await) ---')
  {
    const before = getMemoryMB()

    for (let i = 0; i < 100_000; i++) {
      counter.value = i
    }
    await sleep(100)

    const after = getMemoryMB()
    console.log(`Before: ${before}MB, After: ${after}MB, Delta: ${after - before}MB`)
  }

  // Test 5: Many awaits
  console.log('\n--- Test 5: Many await sleep(0) ---')
  {
    const before = getMemoryMB()

    for (let i = 0; i < 10_000; i++) {
      await sleep(0)
    }

    const after = getMemoryMB()
    console.log(`Before: ${before}MB, After: ${after}MB, Delta: ${after - before}MB`)
  }

  app.unmount()
}

console.log(`\nFinal memory: ${getMemoryMB()}MB`)
process.exit(0)
