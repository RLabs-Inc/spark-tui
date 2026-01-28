/**
 * Adaptive Spin-Wait Latency Benchmark
 *
 * Measures ROUND-TRIP latency: TS writes flag → Rust detects → Rust writes ack → TS detects.
 * Actual one-way detection ≈ round-trip / 2.
 *
 * Also measures what happens after idle (tests the backoff behavior).
 */

import { dlopen, FFIType, ptr } from 'bun:ffi'
import { join } from 'path'

console.log('=== Adaptive Spin-Wait Latency Benchmark ===\n')

const libPath = join(import.meta.dir, '../rust/target/release/libspark_tui_engine.dylib')
const lib = dlopen(libPath, {
  spark_test_atomic_wait: {
    args: [FFIType.ptr, FFIType.u32],
    returns: FFIType.u32,
  },
})

// =========================================================================
// Test 1: Rapid-fire round-trip (spin loop already hot)
// =========================================================================

console.log('--- Test 1: Rapid-fire detection (hot spin) ---\n')

{
  const sab = new SharedArrayBuffer(16)
  const i32 = new Int32Array(sab)

  // Start Rust spin thread
  lib.symbols.spark_test_atomic_wait(ptr(new Uint8Array(sab)), 1)
  await Bun.sleep(50) // ensure thread is spinning

  // Now the Rust thread is spinning on i32[0], waiting for != 0
  // When it detects, it writes result to i32[1]
  // We can measure round-trip by checking i32[1] right after writing i32[0]

  // Fire and measure how quickly Rust responds
  const t0 = performance.now()
  Atomics.store(i32, 0, 1)

  // Tight poll for Rust response
  let polls = 0
  while (Atomics.load(i32, 1) === 0) {
    polls++
  }
  const t1 = performance.now()

  const roundTripMs = t1 - t0
  const rustLatency = Atomics.load(i32, 2) // Rust-measured from thread start, includes sleep
  console.log(`  Round-trip: ${(roundTripMs * 1000).toFixed(0)}μs (${polls} polls)`)
  console.log(`  One-way estimate: ~${(roundTripMs * 500).toFixed(0)}μs`)
}

// =========================================================================
// Test 2: Multiple rapid fires (measures consistency)
// =========================================================================

console.log('\n--- Test 2: Burst detection (20 iterations) ---\n')

const roundTrips: number[] = []

for (let i = 0; i < 20; i++) {
  const sab = new SharedArrayBuffer(16)
  const i32 = new Int32Array(sab)

  lib.symbols.spark_test_atomic_wait(ptr(new Uint8Array(sab)), 1)
  await Bun.sleep(5) // minimal wait

  const t0 = performance.now()
  Atomics.store(i32, 0, 1)

  while (Atomics.load(i32, 1) === 0) {}
  const t1 = performance.now()

  const us = (t1 - t0) * 1000
  roundTrips.push(us)
}

roundTrips.sort((a, b) => a - b)
const min = roundTrips[0]
const max = roundTrips[roundTrips.length - 1]
const median = roundTrips[Math.floor(roundTrips.length / 2)]
const avg = roundTrips.reduce((a, b) => a + b, 0) / roundTrips.length

console.log(`  Min:    ${min.toFixed(1)}μs`)
console.log(`  Median: ${median.toFixed(1)}μs`)
console.log(`  Avg:    ${avg.toFixed(1)}μs`)
console.log(`  Max:    ${max.toFixed(1)}μs`)

// =========================================================================
// Test 3: After idle (measures worst-case backoff)
// =========================================================================

console.log('\n--- Test 3: Wake-from-idle latency ---\n')

const idleLatencies: number[] = []
const idleTimes = [10, 50, 100, 500, 1000] // ms of idle before wake

for (const idleMs of idleTimes) {
  const sab = new SharedArrayBuffer(16)
  const i32 = new Int32Array(sab)

  lib.symbols.spark_test_atomic_wait(ptr(new Uint8Array(sab)), 1)

  // Let the spin loop run for idleMs (it will back off during this time)
  await Bun.sleep(idleMs)

  const t0 = performance.now()
  Atomics.store(i32, 0, 1)

  while (Atomics.load(i32, 1) === 0) {}
  const t1 = performance.now()

  const us = (t1 - t0) * 1000
  idleLatencies.push(us)
  console.log(`  After ${idleMs}ms idle: ${us.toFixed(1)}μs`)
}

// =========================================================================
// Summary
// =========================================================================

console.log('\n=== Summary ===\n')
console.log(`  Hot spin detection:  ${median.toFixed(1)}μs median`)
console.log(`  After 1s idle:       ${idleLatencies[idleLatencies.length - 1].toFixed(1)}μs`)
console.log(`  1 frame @ 60fps:     16,667μs`)
console.log(`  Human perception:    ~10,000μs`)

lib.close()
process.exit(0)
