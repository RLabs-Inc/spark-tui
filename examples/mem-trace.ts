/**
 * Memory tracing test
 *
 * Tracks Bun's memory usage at specific points to identify where growth occurs.
 * Run: bun examples/mem-trace.ts
 */

import { signal } from '@rlabs-inc/signals'
import { box, text } from '../ts/primitives'
import { mountSync } from '../ts/engine'

function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms))
}

function getMemoryMB(): number {
  // Force garbage collection if available
  if (typeof Bun !== 'undefined' && (Bun as any).gc) {
    (Bun as any).gc(true)
  }
  return Math.round(process.memoryUsage().rss / 1024 / 1024)
}

console.log('Memory Trace Test')
console.log('=================\n')

const memBefore = getMemoryMB()
console.log(`[0] Before mount: ${memBefore} MB`)

const counter = signal(0)

const app = mountSync(() => {
  box({
    children: () => {
      text({ content: counter })
    },
  })
}, { mode: 'inline' })

await sleep(500)
const memAfterMount = getMemoryMB()
console.log(`[1] After mount + initial render: ${memAfterMount} MB (+${memAfterMount - memBefore})`)

// Phase 1: 100 updates, measure
console.log('\n--- Phase 1: 100 updates ---')
for (let i = 0; i < 100; i++) {
  counter.value = i
  await sleep(0)
}
await sleep(500)
const memAfter100 = getMemoryMB()
console.log(`[2] After 100 updates: ${memAfter100} MB (+${memAfter100 - memAfterMount})`)

// Phase 2: 1000 updates
console.log('\n--- Phase 2: 1000 updates ---')
for (let i = 0; i < 1000; i++) {
  counter.value = 100 + i
  await sleep(0)
}
await sleep(500)
const memAfter1000 = getMemoryMB()
console.log(`[3] After 1000 updates: ${memAfter1000} MB (+${memAfter1000 - memAfter100})`)

// Phase 3: 5000 updates
console.log('\n--- Phase 3: 5000 updates ---')
for (let i = 0; i < 5000; i++) {
  counter.value = 1100 + i
  await sleep(0)
}
await sleep(500)
const memAfter5000 = getMemoryMB()
console.log(`[4] After 5000 updates: ${memAfter5000} MB (+${memAfter5000 - memAfter1000})`)

// Phase 4: IDLE for 3 seconds
console.log('\n--- Phase 4: 3 second IDLE ---')
console.log('No updates, memory should stay stable...')
const memBeforeIdle = getMemoryMB()
await sleep(3000)
const memAfterIdle = getMemoryMB()
console.log(`[5] After 3s idle: ${memAfterIdle} MB (+${memAfterIdle - memBeforeIdle})`)

if (memAfterIdle > memBeforeIdle + 10) {
  console.log('\n⚠️  WARNING: Memory grew during IDLE - this is the leak!')
} else {
  console.log('\n✓ Memory stable during idle')
}

// Phase 5: Another burst
console.log('\n--- Phase 5: 5000 more updates ---')
for (let i = 0; i < 5000; i++) {
  counter.value = 6100 + i
  await sleep(0)
}
await sleep(500)
const memAfterBurst2 = getMemoryMB()
console.log(`[6] After another 5000 updates: ${memAfterBurst2} MB (+${memAfterBurst2 - memAfterIdle})`)

// Phase 6: Final IDLE
console.log('\n--- Phase 6: 3 second final IDLE ---')
const memBeforeFinalIdle = getMemoryMB()
await sleep(3000)
const memFinal = getMemoryMB()
console.log(`[7] After final 3s idle: ${memFinal} MB (+${memFinal - memBeforeFinalIdle})`)

console.log('\n=== Summary ===')
console.log(`Total memory growth: ${memFinal - memBefore} MB`)
console.log(`Growth per 1000 updates: ${Math.round((memAfter5000 - memAfter100) / 4.9 * 10) / 10} MB`)

app.unmount()
console.log('\nUnmounted. Exiting...')
process.exit(0)
