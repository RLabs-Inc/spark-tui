/**
 * Bun FFI bridge to the Rust SparkTUI engine.
 *
 * Loads the cdylib and exposes typed functions.
 */

import { dlopen, FFIType, ptr } from 'bun:ffi'
import { join } from 'path'

// =============================================================================
// LIBRARY LOADING
// =============================================================================

const LIB_NAME = process.platform === 'darwin'
  ? 'libspark_tui_engine.dylib'
  : process.platform === 'win32'
    ? 'spark_tui_engine.dll'
    : 'libspark_tui_engine.so'

/** Path to the compiled Rust library */
function getLibPath(): string {
  // Look in rust/target/release/ relative to project root
  return join(import.meta.dir, '../../rust/target/release', LIB_NAME)
}

/** FFI symbol definitions */
const symbols = {
  spark_init: {
    args: [FFIType.ptr, FFIType.u32] as const,
    returns: FFIType.u32,
  },
  spark_compute_layout: {
    args: [] as const,
    returns: FFIType.u32,
  },
  spark_buffer_size: {
    args: [] as const,
    returns: FFIType.u32,
  },
} as const

export interface SparkEngine {
  /** Initialize with SharedArrayBuffer pointer. Returns 0 on success. */
  init(bufferPtr: ReturnType<typeof ptr>, bufferLen: number): number
  /** Compute layout from shared buffer data. Returns number of nodes laid out. */
  computeLayout(): number
  /** Get required buffer size. */
  bufferSize(): number
  /** Close the library. */
  close(): void
}

/**
 * Load the Rust engine library.
 *
 * @param libPath - Override path to the .dylib/.so. Defaults to rust/target/release/
 */
export function loadEngine(libPath?: string): SparkEngine {
  const path = libPath ?? getLibPath()
  const lib = dlopen(path, symbols)

  return {
    init(bufferPtr, bufferLen) {
      return lib.symbols.spark_init(bufferPtr, bufferLen)
    },
    computeLayout() {
      return lib.symbols.spark_compute_layout()
    },
    bufferSize() {
      return lib.symbols.spark_buffer_size()
    },
    close() {
      lib.close()
    },
  }
}
