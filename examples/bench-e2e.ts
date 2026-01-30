/**
 * SparkTUI END-TO-END Benchmark
 *
 * The REAL test: Measure from TS primitive prop change → terminal output
 *
 * Full reactive chain:
 *   TS signal.value = X
 *     → repeat() forwards to SharedSlotBuffer
 *     → SAB write
 *     → Atomics.notify (wake Rust)
 *     → Rust wakes from channel
 *     → Layout computation (Taffy)
 *     → Framebuffer computation
 *     → Diff render to terminal
 *   DONE
 */

import { signal, derived, effect } from '@rlabs-inc/signals'
import { dlopen, ptr, FFIType } from 'bun:ffi'
import {
  createAoSBuffer,
  setTerminalSize,
  HEADER_SIZE,
  STRIDE,
  H_WAKE_RUST,
} from '../ts/bridge/shared-buffer-aos'
import { createReactiveArraysAoS } from '../ts/bridge/reactive-arrays-aos'
import { createWakeNotifierAoS } from '../ts/bridge/notify'

const LIB_PATH = new URL('../rust/target/release/libspark_tui_engine.dylib', import.meta.url).pathname

const lib = dlopen(LIB_PATH, {
  spark_init: { args: [FFIType.ptr, FFIType.u32], returns: FFIType.i32 },
  spark_render: { args: [], returns: FFIType.i32 },
  spark_compute_layout: { args: [], returns: FFIType.i32 },
  spark_cleanup: { args: [], returns: FFIType.void },
})

console.log('═══════════════════════════════════════════════════════════════')
console.log('  SparkTUI END-TO-END Benchmark')
console.log('  From TS Signal Change → Terminal Output')
console.log('═══════════════════════════════════════════════════════════════')
console.log()

// =============================================================================
// SETUP: Full reactive bridge (like real app)
// =============================================================================

const buffer = createAoSBuffer()
const notifier = createWakeNotifierAoS(buffer)
const arrays = createReactiveArraysAoS(buffer, notifier)

setTerminalSize(buffer, 120, 40)

const result = lib.symbols.spark_init(ptr(buffer.buffer), buffer.buffer.byteLength)
if (result !== 0) {
  console.error('Failed to initialize')
  process.exit(1)
}

console.log('✓ Full reactive bridge initialized')
console.log('  - AoSBuffer: ✓')
console.log('  - WakeNotifier: ✓')
console.log('  - ReactiveArrays: ✓')
console.log('  - Rust Engine: ✓')
console.log()

// =============================================================================
// BENCHMARK 1: Signal → SAB Write (TS side only)
// =============================================================================

console.log('─────────────────────────────────────────────────────────────────')
console.log('BENCHMARK 1: Signal → SharedBuffer Write (TS reactive chain)')
console.log('─────────────────────────────────────────────────────────────────')

// Create a signal and wire it through the reactive system
const widthSignal = signal(50)
let writeCount = 0

// Simulate what repeat() does: forward signal changes to SharedSlotBuffer
const unsubscribe = effect(() => {
  const w = widthSignal.value
  arrays.width.set(0, w)  // This goes through SharedSlotBuffer → SAB → notifier
  writeCount++
})

const SIGNAL_ITERATIONS = 100000

const start1 = performance.now()
for (let i = 0; i < SIGNAL_ITERATIONS; i++) {
  widthSignal.value = 50 + (i % 50)
}
const elapsed1 = performance.now() - start1

console.log(`  ${SIGNAL_ITERATIONS.toLocaleString()} signal changes in ${elapsed1.toFixed(2)}ms`)
console.log(`  ${(SIGNAL_ITERATIONS / elapsed1 * 1000 / 1000).toFixed(0)}K signals/sec`)
console.log(`  ${(elapsed1 / SIGNAL_ITERATIONS * 1000).toFixed(2)}μs per signal→SAB`)
console.log()

unsubscribe()

// =============================================================================
// BENCHMARK 2: Signal → SAB → Rust Layout (TS + Rust, no render)
// =============================================================================

console.log('─────────────────────────────────────────────────────────────────')
console.log('BENCHMARK 2: Signal → SAB → Rust Layout (cross-language)')
console.log('─────────────────────────────────────────────────────────────────')

// Set up 100 nodes
buffer.view.setUint32(4, 100, true)
for (let i = 0; i < 100; i++) {
  const offset = HEADER_SIZE + i * STRIDE
  buffer.view.setInt32(offset + 180, i === 0 ? -1 : 0, true)
  buffer.view.setUint8(offset + 96, 1)
  buffer.view.setFloat32(offset + 0, 10, true)
  buffer.view.setFloat32(offset + 4, 1, true)
}

const widthSignal2 = signal(50)
const unsubscribe2 = effect(() => {
  arrays.width.set(0, widthSignal2.value)
})

const LAYOUT_ITERATIONS = 10000

const start2 = performance.now()
for (let i = 0; i < LAYOUT_ITERATIONS; i++) {
  widthSignal2.value = 50 + (i % 50)  // TS signal change → SAB
  lib.symbols.spark_compute_layout()   // Rust layout
}
const elapsed2 = performance.now() - start2

console.log(`  ${LAYOUT_ITERATIONS.toLocaleString()} signal→layout cycles in ${elapsed2.toFixed(2)}ms`)
console.log(`  ${(LAYOUT_ITERATIONS / elapsed2 * 1000 / 1000).toFixed(0)}K cycles/sec`)
console.log(`  ${(elapsed2 / LAYOUT_ITERATIONS * 1000).toFixed(2)}μs per full cycle`)
console.log()

unsubscribe2()

// =============================================================================
// BENCHMARK 3: Full E2E (Signal → SAB → Layout → Framebuffer → Render)
// =============================================================================

console.log('─────────────────────────────────────────────────────────────────')
console.log('BENCHMARK 3: FULL END-TO-END (Signal → Terminal)')
console.log('─────────────────────────────────────────────────────────────────')

const widthSignal3 = signal(50)
const unsubscribe3 = effect(() => {
  arrays.width.set(0, widthSignal3.value)
})

const E2E_ITERATIONS = 1000

const start3 = performance.now()
for (let i = 0; i < E2E_ITERATIONS; i++) {
  widthSignal3.value = 50 + (i % 50)  // TS signal change
  lib.symbols.spark_render()           // Full Rust pipeline
}
const elapsed3 = performance.now() - start3

console.log(`  ${E2E_ITERATIONS.toLocaleString()} full E2E cycles in ${elapsed3.toFixed(2)}ms`)
console.log(`  ${(E2E_ITERATIONS / elapsed3 * 1000 / 1000).toFixed(1)}K E2E/sec`)
console.log(`  ${(elapsed3 / E2E_ITERATIONS * 1000).toFixed(1)}μs per E2E cycle`)
console.log(`  ${(1000 / (elapsed3 / E2E_ITERATIONS)).toFixed(0)} potential FPS`)
console.log()

unsubscribe3()

// =============================================================================
// BENCHMARK 4: E2E at Scale (varying node counts)
// =============================================================================

console.log('─────────────────────────────────────────────────────────────────')
console.log('BENCHMARK 4: E2E at Scale (signal → terminal with N nodes)')
console.log('─────────────────────────────────────────────────────────────────')

const NODE_COUNTS = [10, 50, 100, 500, 1000, 5000, 10000]

for (const nodeCount of NODE_COUNTS) {
  // Set up nodes
  buffer.view.setUint32(4, nodeCount, true)
  for (let i = 0; i < nodeCount; i++) {
    const offset = HEADER_SIZE + i * STRIDE
    buffer.view.setInt32(offset + 180, i === 0 ? -1 : 0, true)
    buffer.view.setUint8(offset + 96, 1)
    buffer.view.setFloat32(offset + 0, 10 + (i % 20), true)
    buffer.view.setFloat32(offset + 4, 1, true)
  }

  const sig = signal(50)
  const unsub = effect(() => {
    arrays.width.set(0, sig.value)
  })

  // Warm up
  for (let i = 0; i < 10; i++) {
    sig.value = 50 + i
    lib.symbols.spark_render()
  }

  const iterations = nodeCount > 5000 ? 100 : 500
  const start = performance.now()
  for (let i = 0; i < iterations; i++) {
    sig.value = 50 + (i % 50)
    lib.symbols.spark_render()
  }
  const elapsed = performance.now() - start

  const avgUs = (elapsed / iterations) * 1000
  const fps = 1000 / (elapsed / iterations)
  const status60 = fps >= 60 ? '✓' : '✗'
  const status144 = fps >= 144 ? '✓' : '✗'

  console.log(`  ${nodeCount.toString().padStart(5)} nodes: ${avgUs.toFixed(1).padStart(7)}μs  (${fps.toFixed(0).padStart(6)} FPS)  60:${status60}  144:${status144}`)

  unsub()
}

console.log()

// =============================================================================
// BENCHMARK 5: Burst Updates (simulating rapid typing)
// =============================================================================

console.log('─────────────────────────────────────────────────────────────────')
console.log('BENCHMARK 5: Burst Updates (typing simulation, 100 nodes)')
console.log('─────────────────────────────────────────────────────────────────')

buffer.view.setUint32(4, 100, true)

const textSignal = signal('Hello')
const unsubText = effect(() => {
  // Simulate text content changing (like typing)
  const text = textSignal.value
  // Write to text pool (simplified)
  const encoder = new TextEncoder()
  const bytes = encoder.encode(text)
  const poolStart = HEADER_SIZE + 100000 * STRIDE  // After nodes
  for (let i = 0; i < bytes.length && i < 100; i++) {
    buffer.view.setUint8(poolStart + i, bytes[i])
  }
})

const BURST_SIZES = [10, 50, 100, 500, 1000]

for (const burstSize of BURST_SIZES) {
  const start = performance.now()

  for (let i = 0; i < burstSize; i++) {
    textSignal.value = `Typing char ${i}`
    lib.symbols.spark_render()
  }

  const elapsed = performance.now() - start
  const charsPerSec = (burstSize / elapsed) * 1000

  console.log(`  ${burstSize.toString().padStart(4)} char burst: ${elapsed.toFixed(2).padStart(7)}ms  (${charsPerSec.toFixed(0).padStart(5)} chars/sec)`)
}

unsubText()
console.log()

// =============================================================================
// SUMMARY
// =============================================================================

console.log('═══════════════════════════════════════════════════════════════')
console.log('  END-TO-END BENCHMARK COMPLETE')
console.log('═══════════════════════════════════════════════════════════════')
console.log()
console.log('  Full reactive chain measured:')
console.log('    TS signal.value = X')
console.log('      → effect() fires')
console.log('      → SharedSlotBuffer.set()')
console.log('      → DataView.setFloat32() to SAB')
console.log('      → notifier.notify() (batched)')
console.log('      → Atomics.store + Atomics.notify')
console.log('      → [Rust wakes - not measured in sync bench]')
console.log('      → spark_render() (layout + framebuffer + terminal)')
console.log('    DONE')
console.log()
console.log('  Note: True async wake latency adds ~1ms on macOS')
console.log('  On Linux with futex, could be <10μs')
console.log()

lib.symbols.spark_cleanup()
