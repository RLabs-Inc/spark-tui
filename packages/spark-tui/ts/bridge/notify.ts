/**
 * SparkTUI - Wake Flag Notifier Bridge
 *
 * PURELY REACTIVE: No batching, no microtasks, no delays.
 * Every signal write immediately notifies Rust.
 *
 * The reactive flow:
 *   signal.value = x → SharedBuffer write → IMMEDIATE FFI spark_wake() → Rust wakes
 *
 * KEY INSIGHT (benchmarked):
 * - FFI call: ~5ns (FASTER than Atomics.notify which was ~14ns)
 * - Atomics.notify DOESN'T wake native Rust threads (JSC uses internal ParkingLot)
 * - FFI spark_wake() actually works + triggers ulock_wake for instant Rust wake
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
 * FFI-based Notifier - Calls spark_wake() directly.
 *
 * This replaces the broken Atomics.notify approach:
 * - Atomics.notify uses JSC's internal ParkingLot (can't wake native threads)
 * - FFI spark_wake() calls ulock_wake which actually wakes Rust
 *
 * Benchmarked performance:
 * - FFI call: ~5ns (constant)
 * - Atomics.store + notify: ~14ns (AND doesn't work!)
 * - FFI is 2.8x FASTER and actually works
 *
 * Also tracks instrumentation:
 * - H_TS_NOTIFY_COUNT: total notify calls
 * - H_TS_NOTIFY_TIMESTAMP: Unix microseconds for wake latency calculation
 */
class FFINotifier implements Notifier {
  private wakeFn: () => void
  private view: DataView
  private wakeFlag: Int32Array
  private wakeIndex: number

  constructor(buf: SharedBuffer, wakeFn: () => void) {
    this.wakeFn = wakeFn
    this.view = buf.view
    this.wakeFlag = buf.headerI32
    this.wakeIndex = H_WAKE_RUST / 4
  }

  notify(): void {
    // Instrumentation: write Unix timestamp for Rust to calculate wake latency
    // Use performance.timeOrigin + performance.now() for microsecond precision
    // (Date.now() only has millisecond resolution)
    const nowUs = BigInt(Math.floor((performance.timeOrigin + performance.now()) * 1000))
    this.view.setBigUint64(H_TS_NOTIFY_TIMESTAMP, nowUs, true)

    // Instrumentation: increment notify count
    const count = this.view.getUint32(H_TS_NOTIFY_COUNT, true)
    this.view.setUint32(H_TS_NOTIFY_COUNT, (count + 1) >>> 0, true)

    // Set wake flag in shared memory (Rust can also see this)
    Atomics.store(this.wakeFlag, this.wakeIndex, 1)

    // IMMEDIATE FFI call - ~5ns, actually wakes Rust!
    this.wakeFn()
  }
}

/**
 * Create a silent notifier for testing (no cross-side notification).
 */
export function createNoopNotifier(): Notifier {
  return new NoopNotifier()
}

/**
 * Create an FFI-based Notifier that calls spark_wake() directly.
 *
 * This is THE correct approach:
 * - FFI call: ~5ns (actually works!)
 * - Atomics.notify: ~14ns (doesn't wake native Rust threads)
 *
 * Every signal write immediately calls:
 *   Atomics.store(wakeFlag, 1) + FFI spark_wake()
 *
 * @param buf - The shared buffer
 * @param wakeFn - The FFI wake function (engine.wake from ffi.ts)
 */
export function createFFINotifier(buf: SharedBuffer, wakeFn: () => void): Notifier {
  return new FFINotifier(buf, wakeFn)
}
