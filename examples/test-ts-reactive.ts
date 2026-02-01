/**
 * TEST 4: TS Reactive Layer Only
 *
 * Tests the TypeScript signals + ReactiveArrays + Notifier.
 * NO UI primitives (box, text).
 * Engine running.
 *
 * This isolates whether the leak is in the TS reactive layer
 * or in the UI primitives.
 *
 * Run: bun examples/test-ts-reactive.ts
 * Watch: Activity Monitor → bun process → Memory
 */

import { signal, effect } from '@rlabs-inc/signals'
import { initBridge, getBuffer, getArrays, getNotifier } from '../ts/bridge'
import { loadEngine } from '../ts/bridge/ffi'
import { ptr } from 'bun:ffi'
import { setTerminalSize, setRenderMode, RenderMode } from '../ts/bridge/shared-buffer'

console.log('╔════════════════════════════════════════════════════════════╗')
console.log('║  TEST: TS Reactive Layer (No UI Primitives)                ║')
console.log('║                                                            ║')
console.log('║  - TS signals (@rlabs-inc/signals)                         ║')
console.log('║  - ReactiveArrays + SharedSlotBuffer                       ║')
console.log('║  - AtomicsNotifier                                         ║')
console.log('║  - Rust engine running                                     ║')
console.log('║  - NO box(), text() primitives                             ║')
console.log('║                                                            ║')
console.log('║  Watch memory in Activity Monitor!                         ║')
console.log('╚════════════════════════════════════════════════════════════╝\n')

// Step 1: Initialize bridge (creates SAB + reactive arrays + notifier)
console.log('Step 1: Initializing bridge...')
const { buffer, arrays, notifier } = initBridge()
console.log('  Buffer size:', buffer.raw.byteLength / 1024 / 1024, 'MB')
console.log('  Done.\n')

// Step 2: Configure buffer
console.log('Step 2: Configuring buffer...')
setTerminalSize(buffer, 80, 24)
setRenderMode(buffer, RenderMode.Inline)
console.log('  Done.\n')

// Step 3: Start Rust engine
console.log('Step 3: Starting Rust engine...')
const engine = loadEngine()
const result = engine.init(ptr(buffer.raw), buffer.raw.byteLength)
console.log('  spark_init() returned:', result)
if (result !== 0) {
  console.log('  WARNING: Engine init failed!')
}
console.log('  Done.\n')

// Wait for engine
await Bun.sleep(500)

// Step 4: Create TS signals and effects that write to reactive arrays
console.log('Step 4: Creating TS signals + effects...\n')

const counter = signal(0)
const textContent = signal('Hello')

// Create an effect that writes to the reactive arrays when signals change
// This simulates what primitives do internally
let effectRuns = 0
const stopEffect = effect(() => {
  const c = counter.value
  const t = textContent.value

  // Write to reactive arrays (simulating what primitives do)
  // This triggers the notifier → wake flag → Rust wakes
  if (arrays.nodeCount.length > 0) {
    // Just touch the arrays to trigger reactivity
    const _ = arrays.nodeCount.get(0)
  }

  effectRuns++
})

console.log('  Effect created. Initial run count:', effectRuns, '\n')

// ═══════════════════════════════════════════════════════════════
// PHASE 1: Rapid signal updates
// ═══════════════════════════════════════════════════════════════

console.log('▶ PHASE 1: 500,000 signal updates')
console.log('  Each update triggers: signal → effect → array write → notifier\n')

const start = Date.now()
for (let i = 0; i < 500_000; i++) {
  counter.value = i

  if (i % 50_000 === 0) {
    console.log(`  ${i.toLocaleString()} updates... effect runs: ${effectRuns}`)
  }
}
const elapsed = Date.now() - start

console.log(`\n  Done! ${elapsed}ms for 500K signal updates`)
console.log(`  Effect ran ${effectRuns} times`)
console.log(`  Rate: ${(500_000 / elapsed * 1000).toFixed(0)} updates/sec\n`)

// ═══════════════════════════════════════════════════════════════
// PHASE 2: Sleep
// ═══════════════════════════════════════════════════════════════

console.log('▶ PHASE 2: SLEEP 15 seconds')
console.log('  ┌─────────────────────────────────────────────────┐')
console.log('  │  WATCH MEMORY NOW!                              │')
console.log('  │  If memory grows = TS reactive layer leaking    │')
console.log('  └─────────────────────────────────────────────────┘\n')

for (let i = 0; i < 15; i++) {
  await Bun.sleep(1000)
  console.log(`  Sleep: ${i + 1} of 15 seconds...`)
}

// ═══════════════════════════════════════════════════════════════
// PHASE 3: Another burst
// ═══════════════════════════════════════════════════════════════

console.log('\n▶ PHASE 3: Another 500,000 signal updates\n')

const start2 = Date.now()
for (let i = 0; i < 500_000; i++) {
  counter.value = 500_000 + i
  textContent.value = `Text ${i}`

  if (i % 50_000 === 0) {
    console.log(`  ${(500_000 + i).toLocaleString()} updates... effect runs: ${effectRuns}`)
  }
}
const elapsed2 = Date.now() - start2
console.log(`\n  Done! ${elapsed2}ms`)
console.log(`  Total effect runs: ${effectRuns}\n`)

// ═══════════════════════════════════════════════════════════════
// PHASE 4: Final sleep
// ═══════════════════════════════════════════════════════════════

console.log('▶ PHASE 4: FINAL SLEEP 15 seconds\n')

for (let i = 0; i < 15; i++) {
  await Bun.sleep(1000)
  console.log(`  Sleep: ${i + 1} of 15 seconds...`)
}

// Cleanup
console.log('\n▶ Cleanup...')
stopEffect()
engine.cleanup()
engine.close()

console.log('\n╔════════════════════════════════════════════════════════════╗')
console.log('║  TEST COMPLETE                                             ║')
console.log('║                                                            ║')
console.log('║  If memory stayed stable → TS reactive layer is fine       ║')
console.log('║  If memory grew → Leak is in signals/arrays/notifier       ║')
console.log('╚════════════════════════════════════════════════════════════╝\n')

process.exit(0)
