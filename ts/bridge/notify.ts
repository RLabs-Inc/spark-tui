/**
 * SparkTUI - Wake Flag Notifier Bridge
 *
 * Creates an AtomicsNotifier wired to the shared buffer's wake flag.
 * Used by reactive-arrays.ts to notify Rust when shared memory changes.
 */

import { AtomicsNotifier, NoopNotifier } from '@rlabs-inc/signals'
import type { Notifier } from '@rlabs-inc/signals'
import type { SharedBuffer } from './shared-buffer'
import { H_WAKE_RUST } from './shared-buffer'

/**
 * Create a silent notifier for testing (no cross-side notification).
 */
export function createNoopNotifier(): Notifier {
  return new NoopNotifier()
}

/**
 * Create a Notifier wired to the shared buffer's wake flag.
 *
 * When any SharedSlotBuffer writes to shared memory, the notifier
 * batches via microtask and then: Atomics.store(wakeFlag, 1) + Atomics.notify.
 * This wakes the Rust side's sleepy thread.
 */
export function createWakeNotifier(buf: SharedBuffer): Notifier {
  // SharedBuffer.headerI32 is already an Int32Array view of the header
  return new AtomicsNotifier(buf.headerI32, H_WAKE_RUST / 4) // offset in i32 units
}
