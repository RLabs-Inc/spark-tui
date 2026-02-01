/**
 * SparkTUI - Wake Flag Notifier Bridge
 *
 * PURELY REACTIVE: No batching, no microtasks, no delays.
 * Every signal write immediately notifies Rust.
 *
 * The reactive flow:
 *   signal.value = x → SharedBuffer write → IMMEDIATE Atomics.notify → Rust wakes
 */

import { NoopNotifier } from '@rlabs-inc/signals'
import type { Notifier } from '@rlabs-inc/signals'
import type { SharedBuffer } from './shared-buffer'
import {
  H_WAKE_RUST,
  H_TS_NOTIFY_COUNT,
  H_TS_NOTIFY_TIMESTAMP,
} from './shared-buffer'

/**
 * Synchronous Atomics Notifier - NO BATCHING.
 *
 * Unlike AtomicsNotifier from @rlabs-inc/signals which batches via microtask,
 * this notifier calls Atomics.store + Atomics.notify IMMEDIATELY.
 *
 * This is critical for SparkTUI's pure reactive architecture:
 * - Every signal write triggers immediate notification
 * - Rust wakes instantly (within WakeWatcher's detection latency)
 * - No JavaScript event loop yielding required
 *
 * Also tracks instrumentation:
 * - H_TS_NOTIFY_COUNT: total notify calls
 * - H_TS_NOTIFY_TIMESTAMP: Unix microseconds for wake latency calculation
 */
class SyncAtomicsNotifier implements Notifier {
  private wakeFlag: Int32Array
  private index: number
  private view: DataView

  constructor(buf: SharedBuffer) {
    this.wakeFlag = buf.headerI32
    this.index = H_WAKE_RUST / 4
    this.view = buf.view
  }

  notify(): void {
    // Instrumentation: write Unix timestamp for Rust to calculate wake latency
    // Use Date.now() for reliable cross-runtime comparison with Rust SystemTime
    const nowUs = BigInt(Date.now()) * 1000n  // Convert ms to μs
    this.view.setBigUint64(H_TS_NOTIFY_TIMESTAMP, nowUs, true)

    // Instrumentation: increment notify count
    const count = this.view.getUint32(H_TS_NOTIFY_COUNT, true)
    this.view.setUint32(H_TS_NOTIFY_COUNT, (count + 1) >>> 0, true)

    // IMMEDIATE - no queueMicrotask, no batching
    Atomics.store(this.wakeFlag, this.index, 1)
    Atomics.notify(this.wakeFlag, this.index)
  }
}

/**
 * Create a silent notifier for testing (no cross-side notification).
 */
export function createNoopNotifier(): Notifier {
  return new NoopNotifier()
}

/**
 * Create a SYNCHRONOUS Notifier wired to the shared buffer's wake flag.
 *
 * Every SharedSlotBuffer write immediately calls:
 *   Atomics.store(wakeFlag, 1) + Atomics.notify
 *
 * Also tracks instrumentation (notify count, timestamp for latency measurement).
 *
 * This is pure reactivity - no batching, no delays.
 */
export function createWakeNotifier(buf: SharedBuffer): Notifier {
  return new SyncAtomicsNotifier(buf)
}
