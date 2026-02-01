/**
 * TEST 2: SharedArrayBuffer Only
 *
 * Creates and uses SharedArrayBuffer repeatedly.
 * NO Rust, NO FFI involved.
 *
 * If this leaks → Bun's SAB implementation is the problem
 * If stable → SAB is fine, look elsewhere
 *
 * Run: bun examples/test-sab-only.ts
 * Watch: Activity Monitor → bun process → Memory
 */

console.log('╔════════════════════════════════════════════════════════════╗')
console.log('║  TEST: SharedArrayBuffer Only (No Rust, No FFI)            ║')
console.log('║                                                            ║')
console.log('║  Creating 20MB SAB and doing 1M Atomics operations.        ║')
console.log('║                                                            ║')
console.log('║  Watch memory in Activity Monitor!                         ║')
console.log('╚════════════════════════════════════════════════════════════╝\n')

// Create a 20MB SharedArrayBuffer (like SparkTUI does)
const BUFFER_SIZE = 20 * 1024 * 1024  // 20MB
console.log(`Creating ${BUFFER_SIZE / 1024 / 1024}MB SharedArrayBuffer...\n`)

const sab = new SharedArrayBuffer(BUFFER_SIZE)
const int32View = new Int32Array(sab)
const uint8View = new Uint8Array(sab)

console.log('▶ PHASE 1: 1,000,000 Atomics operations')
console.log('  Store + Load + Notify cycle...\n')

const start = Date.now()
for (let i = 0; i < 1_000_000; i++) {
  // Simulate what SparkTUI does: store wake flag + notify
  Atomics.store(int32View, 0, 1)
  Atomics.notify(int32View, 0, 1)
  Atomics.store(int32View, 0, 0)

  // Also write to random locations like real usage
  const offset = (i * 17) % (BUFFER_SIZE - 4)
  uint8View[offset] = i & 0xFF

  if (i % 100_000 === 0) {
    console.log(`  ${i.toLocaleString()} operations...`)
  }
}
const elapsed = Date.now() - start

console.log(`\n  Done! ${elapsed}ms for 1M Atomics cycles`)
console.log(`  Rate: ${(1_000_000 / elapsed * 1000).toFixed(0)} ops/sec\n`)

// Sleep and watch memory
console.log('▶ PHASE 2: SLEEP 15 seconds')
console.log('  ┌─────────────────────────────────────────────────┐')
console.log('  │  WATCH MEMORY NOW!                              │')
console.log('  │  If it grew during Phase 1 = SAB LEAK           │')
console.log('  │  Memory should be ~20MB (just the buffer).      │')
console.log('  └─────────────────────────────────────────────────┘\n')

for (let i = 0; i < 15; i++) {
  await Bun.sleep(1000)
  console.log(`  Sleep: ${i + 1} of 15 seconds...`)
}

// Another burst
console.log('\n▶ PHASE 3: Another 1,000,000 Atomics operations\n')

const start2 = Date.now()
for (let i = 0; i < 1_000_000; i++) {
  Atomics.store(int32View, 0, 1)
  Atomics.notify(int32View, 0, 1)
  Atomics.store(int32View, 0, 0)

  const offset = (i * 17) % (BUFFER_SIZE - 4)
  uint8View[offset] = i & 0xFF

  if (i % 100_000 === 0) {
    console.log(`  ${(1_000_000 + i).toLocaleString()} operations...`)
  }
}
const elapsed2 = Date.now() - start2
console.log(`\n  Done! ${elapsed2}ms`)

// Final sleep
console.log('\n▶ PHASE 4: FINAL SLEEP 15 seconds\n')

for (let i = 0; i < 15; i++) {
  await Bun.sleep(1000)
  console.log(`  Sleep: ${i + 1} of 15 seconds...`)
}

console.log('\n╔════════════════════════════════════════════════════════════╗')
console.log('║  TEST COMPLETE - 2,000,000 Atomics operations              ║')
console.log('║                                                            ║')
console.log('║  If memory stayed at ~20MB → SAB is NOT the leak           ║')
console.log('║  If memory grew → SharedArrayBuffer IS leaking             ║')
console.log('╚════════════════════════════════════════════════════════════╝\n')

process.exit(0)
