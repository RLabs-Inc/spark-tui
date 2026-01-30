/**
 * SparkTUI - Wake Flag Notifier Bridge
 *
 * Creates an AtomicsNotifier wired to the AoS buffer's wake flag.
 * Used by reactive-arrays-aos.ts to notify Rust when shared memory changes.
 */

import { AtomicsNotifier, NoopNotifier } from '@rlabs-inc/signals'
import type { Notifier } from '@rlabs-inc/signals'
import type { AoSBuffer } from './shared-buffer-aos'
import { H_WAKE_FLAG } from './shared-buffer-aos'

/**
 * Create a silent notifier for testing (no cross-side notification).
 */
export function createNoopNotifier(): Notifier {
  return new NoopNotifier()
}

/**
 * Create a Notifier wired to the AoS buffer's wake flag.
 *
 * When any SharedSlotBuffer writes to shared memory, the notifier
 * batches via microtask and then: Atomics.store(wakeFlag, 1) + Atomics.notify.
 * This wakes the Rust side's sleepy thread.
 */
export function createWakeNotifierAoS(buf: AoSBuffer): Notifier {
  // Create an Int32Array view of just the header for atomic operations
  const headerI32 = new Int32Array(buf.buffer, 0, 64) // 256 bytes / 4 = 64 i32s
  return new AtomicsNotifier(headerI32, H_WAKE_FLAG / 4) // offset in i32 units
}
