/**
 * REAL Wake Latency Benchmark
 *
 * Tests the ACTUAL wake mechanism: adaptive spin-wait with backoff.
 * Previous benchmark used pure spin_loop (mode 1) which was misleading.
 *
 * This test simulates what actually happens in the engine:
 * - Rust thread spins on a wake flag with 3-phase adaptive backoff
 * - TS sets the flag
 * - Measures actual round-trip detection time
 *
 * Phase 1 (idle < 64 iterations): spin_loop() — ns
 * Phase 2 (idle 64-255): yield_now() — μs
 * Phase 3 (idle >= 256): sleep(50μs) — macOS timer granularity applies
 */

import { dlopen, FFIType, ptr } from 'bun:ffi'
import { join } from 'path'

console.log('=== REAL Adaptive Spin-Wait Latency Benchmark ===')
console.log('Testing actual engine wake mechanism, not pure spin loop\n')

const libPath = join(import.meta.dir, '../rust/target/release/libspark_tui_engine.dylib')
const lib = dlopen(libPath, {
  spark_test_adaptive_wake: {
    args: [FFIType.ptr],
    returns: FFIType.u32,
  },
})

// Buffer layout (24 bytes):
//   [0]:  u32 — wake flag (TS sets to 1)
//   [1]:  u32 — result (Rust sets to 1 when detected)
//   [2]:  u32 — latency in microseconds (Rust-measured)
//   [3]:  u32 — phase when detected (1=spin, 2=yield, 3=sleep)
//   [4]:  u32 — iteration count when detected
//   [5]:  u32 — reserved

// =========================================================================
// Test 1: Hot detection (set flag immediately, thread still in phase 1)
// =========================================================================

console.log('--- Test 1: Hot detection (no idle time) ---\n')

{
  const sab = new SharedArrayBuffer(24)
  const u32 = new Uint32Array(sab)
  const u8 = new Uint8Array(sab)

  lib.symbols.spark_test_adaptive_wake(ptr(u8))
  await Bun.sleep(1) // minimal — thread should be in phase 1

  const t0 = performance.now()
  Atomics.store(u32 as any, 0, 1)

  while (Atomics.load(u32 as any, 1) === 0) {}
  const t1 = performance.now()

  const rtUs = (t1 - t0) * 1000
  const rustUs = Atomics.load(u32 as any, 2)
  const phase = Atomics.load(u32 as any, 3)
  const iters = Atomics.load(u32 as any, 4)

  console.log(`  Round-trip: ${rtUs.toFixed(1)}μs`)
  console.log(`  Rust-measured: ${rustUs}μs`)
  console.log(`  Detected in phase: ${phase} (iterations: ${iters})`)
}

// =========================================================================
// Test 2: Wake after various idle periods
// =========================================================================

console.log('\n--- Test 2: Wake-from-idle at various durations ---\n')

const idleTimes = [1, 5, 10, 20, 50, 100, 200, 500, 1000]

for (const idleMs of idleTimes) {
  const sab = new SharedArrayBuffer(24)
  const u32 = new Uint32Array(sab)
  const u8 = new Uint8Array(sab)

  lib.symbols.spark_test_adaptive_wake(ptr(u8))
  await Bun.sleep(idleMs)

  const t0 = performance.now()
  Atomics.store(u32 as any, 0, 1)

  while (Atomics.load(u32 as any, 1) === 0) {}
  const t1 = performance.now()

  const rtUs = (t1 - t0) * 1000
  const rustUs = Atomics.load(u32 as any, 2)
  const phase = Atomics.load(u32 as any, 3)
  const iters = Atomics.load(u32 as any, 4)

  console.log(`  After ${String(idleMs).padStart(4)}ms idle: ${String(rtUs.toFixed(0)).padStart(6)}μs RT | ${String(rustUs).padStart(6)}μs Rust | phase ${phase} (iter ${iters})`)
}

// =========================================================================
// Test 3: Burst after idle (measures re-spin speed)
// =========================================================================

console.log('\n--- Test 3: Burst detection after 500ms idle ---\n')

const burstResults: number[] = []

for (let i = 0; i < 10; i++) {
  const sab = new SharedArrayBuffer(24)
  const u32 = new Uint32Array(sab)
  const u8 = new Uint8Array(sab)

  lib.symbols.spark_test_adaptive_wake(ptr(u8))
  await Bun.sleep(500) // long idle — thread will be in phase 3

  const t0 = performance.now()
  Atomics.store(u32 as any, 0, 1)

  while (Atomics.load(u32 as any, 1) === 0) {}
  const t1 = performance.now()

  burstResults.push((t1 - t0) * 1000)
}

burstResults.sort((a, b) => a - b)
const min = burstResults[0]
const max = burstResults[burstResults.length - 1]
const median = burstResults[Math.floor(burstResults.length / 2)]
const avg = burstResults.reduce((a, b) => a + b, 0) / burstResults.length

console.log(`  Min:    ${min.toFixed(0)}μs`)
console.log(`  Median: ${median.toFixed(0)}μs`)
console.log(`  Avg:    ${avg.toFixed(0)}μs`)
console.log(`  Max:    ${max.toFixed(0)}μs`)

// =========================================================================
// Summary
// =========================================================================

console.log('\n=== Context ===\n')
console.log(`  1 frame @ 60fps:    16,667μs`)
console.log(`  1 frame @ 120fps:    8,333μs`)
console.log(`  Human perception:  ~10,000μs`)
console.log(`  macOS timer floor:  ~1,000μs`)

lib.close()
process.exit(0)
