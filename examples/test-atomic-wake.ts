/**
 * Cross-Language Atomic Wake Test
 *
 * Tests FOUR mechanisms:
 *   Mode 0: atomic_wait::wait (libc++ on macOS)
 *   Mode 1: spin loop (control — always works)
 *   Mode 2: wait_on_address (os_sync_wait_on_address on macOS 14.4+)
 *   Mode 3: ecmascript_futex (ECMAScript memory model futex via os_sync_*)
 */

import { dlopen, FFIType, ptr } from 'bun:ffi'
import { join } from 'path'

console.log('=== Cross-Language Atomic Wake Test ===')
console.log('Does Atomics.notify() from Bun wake Rust atomic wait mechanisms?\n')

const libPath = join(import.meta.dir, '../rust/target/release/libspark_tui_engine.dylib')
const lib = dlopen(libPath, {
  spark_test_atomic_wait: {
    args: [FFIType.ptr, FFIType.u32],
    returns: FFIType.u32,
  },
})

async function runTest(
  name: string,
  mode: number,
  useNotify: boolean,
): Promise<{ woken: boolean; latency: number }> {
  console.log(`--- ${name} ---\n`)

  const sab = new SharedArrayBuffer(16)
  const i32 = new Int32Array(sab)
  const u8 = new Uint8Array(sab)

  const r = lib.symbols.spark_test_atomic_wait(ptr(u8), mode)
  console.log(`  Rust thread started: ${r === 0 ? 'OK' : 'FAILED'}`)
  console.log('  Waiting 500ms...')
  await Bun.sleep(500)

  if (useNotify) {
    console.log('  Firing: Atomics.store(1) + Atomics.notify()')
  } else {
    console.log('  Firing: Atomics.store(1) only')
  }
  Atomics.store(i32, 0, 1)
  if (useNotify) {
    const n = Atomics.notify(i32, 0)
    console.log(`  Atomics.notify returned: ${n}`)
  }

  const waitMs = mode === 1 ? 200 : 1000
  await Bun.sleep(waitMs)

  const result = Atomics.load(i32, 1)
  const latency = Atomics.load(i32, 2)
  const woken = result === 1

  console.log(`  Result: ${woken ? 'WOKEN' : 'NOT WOKEN'}`)
  if (woken) console.log(`  Latency: ${latency}μs`)
  console.log('')

  return { woken, latency }
}

// =========================================================================
// Tests
// =========================================================================

const t1 = await runTest('Test 1: atomic_wait (libc++) + store + notify', 0, true)
const t2 = await runTest('Test 2: wait_on_address (os_sync_*) + store + notify', 2, true)
const t3 = await runTest('Test 3: wait_on_address (os_sync_*) + store only', 2, false)
const t4 = await runTest('Test 4: spin loop (control) + store only', 1, false)
const t5 = await runTest('Test 5: ecmascript_futex + store + notify', 3, true)
const t6 = await runTest('Test 6: ecmascript_futex + store only', 3, false)

// =========================================================================
// Summary
// =========================================================================

console.log('=== Summary ===\n')
console.log(`  atomic_wait (libc++):           ${t1.woken ? `WOKEN (${t1.latency}μs)` : 'NOT WOKEN'}`)
console.log(`  wait_on_address (os_sync_*):    ${t2.woken ? `WOKEN (${t2.latency}μs)` : 'NOT WOKEN'}`)
console.log(`  wait_on_address (store only):   ${t3.woken ? `WOKEN (${t3.latency}μs)` : 'NOT WOKEN'}`)
console.log(`  spin loop (control):            ${t4.woken ? `WOKEN (${t4.latency}μs)` : 'NOT WOKEN'}`)
console.log(`  ecmascript_futex + notify:      ${t5.woken ? `WOKEN (${t5.latency}μs)` : 'NOT WOKEN'}`)
console.log(`  ecmascript_futex (store only):  ${t6.woken ? `WOKEN (${t6.latency}μs)` : 'NOT WOKEN'}`)

if (t5.woken) {
  console.log('\necmascript_futex WORKS cross-language!')
  console.log('  -> ECMAScript memory model futex is the solution.')
} else if (t2.woken) {
  console.log('\nos_sync_wait_on_address WORKS cross-language!')
  console.log('  -> Zero CPU blocking. No spinning needed.')
} else if (t1.woken) {
  console.log('\natomic_wait (libc++) works but not os_sync_*')
} else {
  console.log('\nNo mechanism wakes from Atomics.notify.')
  console.log('  -> Adaptive spin-wait is the correct solution.')
}

lib.close()
process.exit(0)
