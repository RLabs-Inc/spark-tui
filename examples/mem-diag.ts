/**
 * Memory leak diagnostic test
 *
 * Logs wake events and frame counts to understand what's happening.
 * Run: bun examples/mem-diag.ts
 */

import { signal } from '@rlabs-inc/signals'
import { box, text } from '../ts/primitives'
import { mountSync } from '../ts/engine'

function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms))
}

console.log('Memory Leak Diagnostic Test')
console.log('============================\n')

const counter = signal(0)

const app = mountSync(() => {
  box({
    children: () => {
      text({ content: counter })
    },
  })
}, { mode: 'inline' })

// Get the shared buffer for diagnostics
const sharedBuffer = (app as any).sharedBuffer

// Wait for initial render
await sleep(500)

console.log('Phase 1: 3 seconds of rapid updates')
console.log('=====================================')
const duration1 = 3000
let updates = 0
let start = Date.now()
let lastRenderCount = sharedBuffer?.renderCount?.() ?? 0

// Log every second during updates
const logInterval = setInterval(() => {
  const now = Date.now()
  const elapsed = ((now - start) / 1000).toFixed(1)
  const currentRenderCount = sharedBuffer?.renderCount?.() ?? 0
  const framesThisSecond = currentRenderCount - lastRenderCount
  lastRenderCount = currentRenderCount

  console.log(`[${elapsed}s] Updates: ${updates}, Frames rendered this second: ${framesThisSecond}`)
}, 1000)

while (Date.now() - start < duration1) {
  counter.value = updates++
  await sleep(0)
}

clearInterval(logInterval)
console.log(`\nPhase 1 complete: ${updates} updates in 3 seconds`)
console.log(`Total frames rendered: ${sharedBuffer?.renderCount?.() ?? 'unknown'}`)

// Phase 2: NO updates, just watch memory
console.log('\nPhase 2: 5 seconds of IDLE (no updates)')
console.log('==========================================')
console.log('Memory should stay stable. Watch Activity Monitor.')

const idleStart = Date.now()
const idleDuration = 5000
let idleRenderCount = sharedBuffer?.renderCount?.() ?? 0

// Check every second during idle
const idleInterval = setInterval(() => {
  const now = Date.now()
  const elapsed = ((now - idleStart) / 1000).toFixed(1)
  const currentRenderCount = sharedBuffer?.renderCount?.() ?? 0
  const newFrames = currentRenderCount - idleRenderCount

  // If newFrames > 0, something is triggering renders during idle!
  if (newFrames > 0) {
    console.log(`[IDLE ${elapsed}s] WARNING: ${newFrames} NEW frames rendered during idle!`)
  } else {
    console.log(`[IDLE ${elapsed}s] No new frames (expected)`)
  }
  idleRenderCount = currentRenderCount
}, 1000)

await sleep(idleDuration)
clearInterval(idleInterval)

console.log('\nPhase 2 complete.')
console.log(`Final render count: ${sharedBuffer?.renderCount?.() ?? 'unknown'}`)

app.unmount()
console.log('\nDone - process should exit cleanly')
process.exit(0)
