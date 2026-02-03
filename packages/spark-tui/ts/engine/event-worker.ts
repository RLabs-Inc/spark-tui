/**
 * SparkTUI Event Worker
 *
 * Runs in a separate thread, blocking on spark_wait_for_events().
 * Posts "events" message to main thread when Rust signals events are ready.
 *
 * This allows:
 * - Main thread: never blocks, runs animations/timers normally
 * - Worker thread: blocks with 0% CPU, instant wake on input
 */

declare var self: Worker

import { dlopen, FFIType } from 'bun:ffi'

// FFI symbols - only need waitForEvents
const symbols = {
  spark_wait_for_events: {
    args: [] as const,
    returns: FFIType.void,
  },
} as const

let lib: ReturnType<typeof dlopen<typeof symbols>> | null = null
let running = true

// Handle messages from main thread
self.onmessage = (event: MessageEvent) => {
  const { type, libPath } = event.data

  if (type === 'start') {
    // Load the FFI library
    lib = dlopen(libPath, symbols)

    // Start the blocking event loop
    eventLoop()
  } else if (type === 'stop') {
    running = false
  }
}

function eventLoop(): void {
  while (running && lib) {
    // Block until Rust signals events are ready (0% CPU)
    lib.symbols.spark_wait_for_events()

    if (!running) break

    // Notify main thread that events are ready
    postMessage('events')
  }
}
