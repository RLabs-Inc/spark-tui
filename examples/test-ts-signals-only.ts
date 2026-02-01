/**
 * TEST 5: TS Signals Only (@rlabs-inc/signals)
 *
 * Pure TypeScript signals - NO Rust, NO SAB, NO FFI.
 * Tests the signals package in isolation.
 *
 * Run: bun examples/test-ts-signals-only.ts
 * Watch: Activity Monitor → bun process → Memory
 */

import { signal, derived, effect } from '@rlabs-inc/signals'

console.log('╔════════════════════════════════════════════════════════════╗')
console.log('║  TEST: TS Signals Only (@rlabs-inc/signals)                ║')
console.log('║                                                            ║')
console.log('║  Pure TypeScript - NO Rust, NO SAB, NO FFI                 ║')
console.log('║  Just signals, deriveds, and effects.                      ║')
console.log('║                                                            ║')
console.log('║  Watch memory in Activity Monitor!                         ║')
console.log('╚════════════════════════════════════════════════════════════╝\n')

// Create a reactive chain similar to what SparkTUI uses
const counter = signal(0)
const text = signal('Hello')

// Derived chain (like layout → framebuffer)
const doubled = derived(() => counter.value * 2)
const message = derived(() => `${text.value}: ${doubled.value}`)

// Effect (like render effect)
let effectRuns = 0
const stopEffect = effect(() => {
  const m = message.value
  const _ = m.length  // Touch the value
  effectRuns++
})

console.log('Reactive chain created:')
console.log('  counter → doubled → message → effect')
console.log('  Initial effect runs:', effectRuns, '\n')

// ═══════════════════════════════════════════════════════════════
// PHASE 1: 1,000,000 signal updates
// ═══════════════════════════════════════════════════════════════

console.log('▶ PHASE 1: 1,000,000 signal updates')
console.log('  Each triggers: signal → derived → derived → effect\n')

const start = Date.now()
for (let i = 0; i < 1_000_000; i++) {
  counter.value = i

  if (i % 100_000 === 0) {
    console.log(`  ${i.toLocaleString()} updates... effect runs: ${effectRuns}`)
  }
}
const elapsed = Date.now() - start

console.log(`\n  Done! ${elapsed}ms for 1M updates`)
console.log(`  Effect ran ${effectRuns} times`)
console.log(`  Rate: ${(1_000_000 / elapsed * 1000).toFixed(0)} updates/sec\n`)

// ═══════════════════════════════════════════════════════════════
// PHASE 2: Sleep
// ═══════════════════════════════════════════════════════════════

console.log('▶ PHASE 2: SLEEP 15 seconds')
console.log('  ┌─────────────────────────────────────────────────┐')
console.log('  │  WATCH MEMORY NOW!                              │')
console.log('  │  If memory grows = @rlabs-inc/signals leaking   │')
console.log('  └─────────────────────────────────────────────────┘\n')

for (let i = 0; i < 15; i++) {
  await Bun.sleep(1000)
  console.log(`  Sleep: ${i + 1} of 15 seconds...`)
}

// ═══════════════════════════════════════════════════════════════
// PHASE 3: Another burst with text changes too
// ═══════════════════════════════════════════════════════════════

console.log('\n▶ PHASE 3: 1,000,000 more updates (counter + text)\n')

const start2 = Date.now()
for (let i = 0; i < 1_000_000; i++) {
  counter.value = i
  if (i % 10 === 0) {
    text.value = `Text${i}`  // Change text every 10 iterations
  }

  if (i % 100_000 === 0) {
    console.log(`  ${(1_000_000 + i).toLocaleString()} updates... effect runs: ${effectRuns}`)
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
stopEffect()

console.log('\n╔════════════════════════════════════════════════════════════╗')
console.log('║  TEST COMPLETE - 2,000,000 signal updates                  ║')
console.log('║                                                            ║')
console.log('║  Total effect runs:', effectRuns.toString().padStart(10), '                         ║')
console.log('║                                                            ║')
console.log('║  If memory stayed stable → TS signals are fine             ║')
console.log('║  If memory grew → @rlabs-inc/signals is leaking            ║')
console.log('╚════════════════════════════════════════════════════════════╝\n')

process.exit(0)
