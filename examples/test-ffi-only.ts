/**
 * TEST 1: FFI Calls Only
 *
 * Loads the Rust dylib and calls spark_wake() a million times.
 * NO SharedArrayBuffer involved.
 *
 * If this leaks → Bun's FFI layer is the problem
 * If stable → FFI is fine, look elsewhere
 *
 * Run: bun examples/test-ffi-only.ts
 * Watch: Activity Monitor → bun process → Memory
 */

import { dlopen, FFIType } from 'bun:ffi'
import { join } from 'path'

const LIB_PATH = join(import.meta.dir, '../rust/target/release/libspark_tui_engine.dylib')

console.log('╔════════════════════════════════════════════════════════════╗')
console.log('║  TEST: FFI Calls Only (No SharedArrayBuffer)               ║')
console.log('║                                                            ║')
console.log('║  Loading Rust dylib and calling spark_buffer_size()        ║')
console.log('║  1 million times.                                          ║')
console.log('║                                                            ║')
console.log('║  Watch memory in Activity Monitor!                         ║')
console.log('╚════════════════════════════════════════════════════════════╝\n')

// Load the library
console.log(`Loading: ${LIB_PATH}\n`)

const lib = dlopen(LIB_PATH, {
  spark_buffer_size: {
    args: [] as const,
    returns: FFIType.u32,
  },
})

// We can't call spark_wake() without init, so use spark_buffer_size() instead
// It's a simple FFI call that returns a number

console.log('▶ PHASE 1: 1,000,000 FFI calls')
console.log('  Calling spark_buffer_size() repeatedly...\n')

const start = Date.now()
for (let i = 0; i < 1_000_000; i++) {
  lib.symbols.spark_buffer_size()

  if (i % 100_000 === 0) {
    console.log(`  ${i.toLocaleString()} calls...`)
  }
}
const elapsed = Date.now() - start

console.log(`\n  Done! ${elapsed}ms for 1M calls`)
console.log(`  Rate: ${(1_000_000 / elapsed * 1000).toFixed(0)} calls/sec\n`)

// Sleep and watch memory
console.log('▶ PHASE 2: SLEEP 15 seconds')
console.log('  ┌─────────────────────────────────────────────────┐')
console.log('  │  WATCH MEMORY NOW!                              │')
console.log('  │  If it grew during Phase 1 = FFI LEAK           │')
console.log('  │  Memory should be flat/low.                     │')
console.log('  └─────────────────────────────────────────────────┘\n')

for (let i = 0; i < 15; i++) {
  await Bun.sleep(1000)
  console.log(`  Sleep: ${i + 1} of 15 seconds...`)
}

// Another burst
console.log('\n▶ PHASE 3: Another 1,000,000 FFI calls\n')

const start2 = Date.now()
for (let i = 0; i < 1_000_000; i++) {
  lib.symbols.spark_buffer_size()

  if (i % 100_000 === 0) {
    console.log(`  ${(1_000_000 + i).toLocaleString()} calls...`)
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
console.log('║  TEST COMPLETE - 2,000,000 FFI calls                       ║')
console.log('║                                                            ║')
console.log('║  If memory stayed low → FFI is NOT the leak                ║')
console.log('║  If memory grew → FFI/dlopen IS leaking                    ║')
console.log('╚════════════════════════════════════════════════════════════╝\n')

lib.close()
process.exit(0)
