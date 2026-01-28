/**
 * SparkTUI - Wake Flag Notifier Bridge
 *
 * Creates an AtomicsNotifier wired to the SharedBuffer's wake flag.
 * Used by reactive-arrays.ts to notify Rust when shared memory changes.
 */

import { AtomicsNotifier, NoopNotifier } from '@rlabs-inc/signals'
import type { Notifier } from '@rlabs-inc/signals'
import type { SharedBufferViews } from './shared-buffer'
import { HEADER_WAKE_FLAG } from './shared-buffer'

/**
 * Create a Notifier wired to the SharedBuffer's wake flag.
 *
 * When any SharedSlotBuffer writes to shared memory, the notifier
 * batches via microtask and then: Atomics.store(wakeFlag, 1) + Atomics.notify.
 * This wakes the Rust side's sleepy thread.
 */
export function createWakeNotifier(views: SharedBufferViews): Notifier {
  return new AtomicsNotifier(views.headerI32, HEADER_WAKE_FLAG)
}

/**
 * Create a silent notifier for testing (no cross-side notification).
 */
export function createNoopNotifier(): Notifier {
  return new NoopNotifier()
}
