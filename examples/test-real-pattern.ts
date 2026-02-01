/**
 * TEST 3: Real SparkTUI Pattern
 *
 * Exactly what SparkTUI does:
 * 1. Create SharedArrayBuffer (20MB)
 * 2. ONE dlopen + ONE spark_init() FFI call
 * 3. Then ONLY Atomics operations on SAB (no more FFI)
 *
 * This is the actual usage pattern. If this leaks, we've reproduced
 * the bug in minimal form.
 *
 * Run: bun examples/test-real-pattern.ts
 * Watch: Activity Monitor → bun process → Memory
 */

import { dlopen, FFIType, ptr } from 'bun:ffi'
import { join } from 'path'

const LIB_PATH = join(import.meta.dir, '../rust/target/release/libspark_tui_engine.dylib')

console.log('╔════════════════════════════════════════════════════════════╗')
console.log('║  TEST: Real SparkTUI Pattern                               ║')
console.log('║                                                            ║')
console.log('║  1. Create 20MB SharedArrayBuffer                          ║')
console.log('║  2. ONE dlopen + ONE spark_init() call                     ║')
console.log('║  3. Then ONLY Atomics operations (no more FFI)             ║')
console.log('║                                                            ║')
console.log('║  This is EXACTLY what SparkTUI does.                       ║')
console.log('║  Watch memory in Activity Monitor!                         ║')
console.log('╚════════════════════════════════════════════════════════════╝\n')

// Step 1: Create SharedArrayBuffer (like SparkTUI)
const BUFFER_SIZE = 20 * 1024 * 1024
console.log(`Step 1: Creating ${BUFFER_SIZE / 1024 / 1024}MB SharedArrayBuffer...`)
const sab = new SharedArrayBuffer(BUFFER_SIZE)
const int32View = new Int32Array(sab)
const uint8View = new Uint8Array(sab)

// Initialize header with some values
const view = new DataView(sab)
view.setUint32(0, 1, true)  // version
view.setUint32(4, 0, true)  // node_count
view.setUint32(8, 10000, true)  // max_nodes
view.setUint32(12, 80, true)  // terminal_width
view.setUint32(16, 24, true)  // terminal_height

console.log('  Done.\n')

// Step 2: Load dylib and call spark_init ONCE
console.log(`Step 2: Loading dylib and calling spark_init() ONCE...`)
console.log(`  Path: ${LIB_PATH}`)

const lib = dlopen(LIB_PATH, {
  spark_init: {
    args: [FFIType.ptr, FFIType.u32] as const,
    returns: FFIType.u32,
  },
  spark_cleanup: {
    args: [] as const,
    returns: FFIType.void,
  },
})

const result = lib.symbols.spark_init(ptr(sab), BUFFER_SIZE)
console.log(`  spark_init() returned: ${result}`)

if (result !== 0) {
  console.log('  WARNING: Init returned non-zero. Engine may have failed.')
  console.log('  Continuing anyway to test SAB operations...\n')
} else {
  console.log('  Engine started successfully!\n')
}

// Wake flag offset (from shared-buffer.ts H_WAKE_RUST)
const WAKE_FLAG_OFFSET = 32  // offset in bytes
const WAKE_FLAG_INDEX = WAKE_FLAG_OFFSET / 4  // offset in int32

console.log('Step 3: NO MORE FFI CALLS from here on.\n')
console.log('        Only Atomics operations on SharedArrayBuffer.\n')

// Wait for engine to initialize
await Bun.sleep(500)

// ═══════════════════════════════════════════════════════════════
// PHASE 1: Simulate signal updates via SAB writes
// ═══════════════════════════════════════════════════════════════

console.log('▶ PHASE 1: 500,000 "signal updates" via SAB + Atomics')
console.log('  Writing to buffer + setting wake flag...\n')

const start = Date.now()
for (let i = 0; i < 500_000; i++) {
  // Simulate writing node data (like a signal update)
  const nodeOffset = 256 + (i % 100) * 1024  // Node data area
  if (nodeOffset + 100 < BUFFER_SIZE) {
    uint8View[nodeOffset] = i & 0xFF
    uint8View[nodeOffset + 1] = (i >> 8) & 0xFF
  }

  // Set wake flag (this is what notifier does)
  Atomics.store(int32View, WAKE_FLAG_INDEX, 1)
  // Note: We don't call Atomics.notify because it doesn't wake Rust anyway
  // The adaptive spin watcher detects the flag change

  if (i % 50_000 === 0) {
    console.log(`  ${i.toLocaleString()} updates...`)
  }
}
const elapsed = Date.now() - start

console.log(`\n  Done! ${elapsed}ms for 500K updates`)
console.log(`  Rate: ${(500_000 / elapsed * 1000).toFixed(0)} updates/sec\n`)

// ═══════════════════════════════════════════════════════════════
// PHASE 2: Sleep - watch memory
// ═══════════════════════════════════════════════════════════════

console.log('▶ PHASE 2: SLEEP 15 seconds')
console.log('  ┌─────────────────────────────────────────────────┐')
console.log('  │  WATCH MEMORY NOW!                              │')
console.log('  │  Rust engine is running (wake watcher spinning) │')
console.log('  │  If memory grows during sleep = LEAK!           │')
console.log('  └─────────────────────────────────────────────────┘\n')

for (let i = 0; i < 15; i++) {
  await Bun.sleep(1000)
  console.log(`  Sleep: ${i + 1} of 15 seconds...`)
}

// ═══════════════════════════════════════════════════════════════
// PHASE 3: Another burst
// ═══════════════════════════════════════════════════════════════

console.log('\n▶ PHASE 3: Another 500,000 updates\n')

const start2 = Date.now()
for (let i = 0; i < 500_000; i++) {
  const nodeOffset = 256 + (i % 100) * 1024
  if (nodeOffset + 100 < BUFFER_SIZE) {
    uint8View[nodeOffset] = i & 0xFF
  }
  Atomics.store(int32View, WAKE_FLAG_INDEX, 1)

  if (i % 50_000 === 0) {
    console.log(`  ${(500_000 + i).toLocaleString()} updates...`)
  }
}
const elapsed2 = Date.now() - start2
console.log(`\n  Done! ${elapsed2}ms`)

// ═══════════════════════════════════════════════════════════════
// PHASE 4: Final sleep
// ═══════════════════════════════════════════════════════════════

console.log('\n▶ PHASE 4: FINAL SLEEP 15 seconds\n')

for (let i = 0; i < 15; i++) {
  await Bun.sleep(1000)
  console.log(`  Sleep: ${i + 1} of 15 seconds...`)
}

// ═══════════════════════════════════════════════════════════════
// Cleanup
// ═══════════════════════════════════════════════════════════════

console.log('\n▶ Cleanup: Calling spark_cleanup()...')
lib.symbols.spark_cleanup()
console.log('  Done.')

console.log('\n╔════════════════════════════════════════════════════════════╗')
console.log('║  TEST COMPLETE                                             ║')
console.log('║                                                            ║')
console.log('║  This was the EXACT SparkTUI pattern:                      ║')
console.log('║  - 1 FFI call (init)                                       ║')
console.log('║  - 1,000,000 SAB writes + Atomics.store                    ║')
console.log('║  - Rust engine running with adaptive spin                  ║')
console.log('║                                                            ║')
console.log('║  If memory stayed stable → Problem is elsewhere            ║')
console.log('║  If memory grew → We reproduced the leak!                  ║')
console.log('╚════════════════════════════════════════════════════════════╝\n')

lib.close()
process.exit(0)
