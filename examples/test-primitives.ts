/**
 * TEST 6: UI Primitives (box, text)
 *
 * Tests the full primitive layer:
 * - mountSync() with real engine
 * - box() and text() creating nodes
 * - Signal updates triggering re-renders
 *
 * This is closest to the real benchmark.
 *
 * Run: bun examples/test-primitives.ts
 * Watch: Activity Monitor → bun process → Memory
 */

import { signal } from '@rlabs-inc/signals'
import { box, text } from '../ts/primitives'
import { mountSync } from '../ts/engine'

console.log('╔════════════════════════════════════════════════════════════╗')
console.log('║  TEST: UI Primitives (Full Stack)                          ║')
console.log('║                                                            ║')
console.log('║  - mountSync() with Rust engine                            ║')
console.log('║  - box() and text() primitives                             ║')
console.log('║  - Signal updates → SharedBuffer → Rust render             ║')
console.log('║                                                            ║')
console.log('║  This is the FULL SparkTUI stack.                          ║')
console.log('║  Watch memory in Activity Monitor!                         ║')
console.log('╚════════════════════════════════════════════════════════════╝\n')

// Create signals
const counter = signal(0)
const content = signal('Hello SparkTUI!')

console.log('Step 1: Mounting app with primitives...\n')

// Mount a simple app
const app = mountSync(() => {
  box({
    flexDirection: 'column',
    children: () => {
      text({ content: counter })
      text({ content: content })
    },
  })
}, { mode: 'inline' })

console.log('  Mounted! Engine running.\n')

// Wait for initial render
await Bun.sleep(500)

// ═══════════════════════════════════════════════════════════════
// PHASE 1: Signal updates
// ═══════════════════════════════════════════════════════════════

console.log('▶ PHASE 1: 100,000 signal updates')
console.log('  Each triggers full reactive chain → Rust render\n')

const start = Date.now()
for (let i = 0; i < 100_000; i++) {
  counter.value = i

  if (i % 10_000 === 0) {
    console.log(`  ${i.toLocaleString()} updates...`)
    await Bun.sleep(0) // Yield to allow renders
  }
}
const elapsed = Date.now() - start

console.log(`\n  Done! ${elapsed}ms for 100K updates`)
console.log(`  Rate: ${(100_000 / elapsed * 1000).toFixed(0)} updates/sec\n`)

// ═══════════════════════════════════════════════════════════════
// PHASE 2: Sleep
// ═══════════════════════════════════════════════════════════════

console.log('▶ PHASE 2: SLEEP 15 seconds')
console.log('  ┌─────────────────────────────────────────────────┐')
console.log('  │  WATCH MEMORY NOW!                              │')
console.log('  │  If memory grows = LEAK IN PRIMITIVES           │')
console.log('  └─────────────────────────────────────────────────┘\n')

for (let i = 0; i < 15; i++) {
  await Bun.sleep(1000)
  console.log(`  Sleep: ${i + 1} of 15 seconds...`)
}

// ═══════════════════════════════════════════════════════════════
// PHASE 3: More updates with text changes
// ═══════════════════════════════════════════════════════════════

console.log('\n▶ PHASE 3: 100,000 more updates (counter + text)\n')

const start2 = Date.now()
for (let i = 0; i < 100_000; i++) {
  counter.value = 100_000 + i
  if (i % 100 === 0) {
    content.value = `Update ${i}`
  }

  if (i % 10_000 === 0) {
    console.log(`  ${(100_000 + i).toLocaleString()} updates...`)
    await Bun.sleep(0)
  }
}
const elapsed2 = Date.now() - start2

console.log(`\n  Done! ${elapsed2}ms\n`)

// ═══════════════════════════════════════════════════════════════
// PHASE 4: Final sleep
// ═══════════════════════════════════════════════════════════════

console.log('▶ PHASE 4: FINAL SLEEP 15 seconds\n')

for (let i = 0; i < 15; i++) {
  await Bun.sleep(1000)
  console.log(`  Sleep: ${i + 1} of 15 seconds...`)
}

// Cleanup
console.log('\n▶ Cleanup: Unmounting...')
app.unmount()

console.log('\n╔════════════════════════════════════════════════════════════╗')
console.log('║  TEST COMPLETE - 200,000 primitive updates                 ║')
console.log('║                                                            ║')
console.log('║  If memory stayed stable → Primitives are fine             ║')
console.log('║  If memory grew → LEAK IS IN PRIMITIVES/RENDERING          ║')
console.log('╚════════════════════════════════════════════════════════════╝\n')

process.exit(0)
