/**
 * SparkTUI Mount API (v3 Buffer)
 *
 * The main entry point for SparkTUI applications.
 * Handles bridge initialization, engine loading, event listener, and cleanup.
 *
 * PURELY REACTIVE: No loops. Change propagates through the dependency graph.
 * The event listener uses Atomics.waitAsync - it SUSPENDS until Rust notifies.
 *
 * Two APIs:
 *   mount()     - async, blocks until exit (most users)
 *   mountSync() - sync, returns handle for manual control (power users, tests)
 */

import { initBridge, resetBridge, getBuffer } from '../bridge'
import {
  startEventListener,
  stopEventListener,
  registerExitHandler,
  cleanupAllHandlers,
} from './events'
import { scoped } from '../primitives/scope'
import {
  type SharedBuffer,
  setTerminalSize,
  setConfigFlags,
  setRenderMode,
  RenderMode,
  CONFIG_DEFAULT,
  CONFIG_EXIT_ON_CTRL_C,
  CONFIG_TAB_NAVIGATION,
  CONFIG_MOUSE_ENABLED,
} from '../bridge/shared-buffer'
import { loadEngine, getLibPath, type SparkEngine } from '../bridge/ffi'
import { ptr } from 'bun:ffi'
import type { Cleanup } from '../primitives/types'

// =============================================================================
// TYPES
// =============================================================================

/**
 * Render mode for the application.
 *
 * - `fullscreen`: Clears screen, uses alternate buffer (default)
 * - `inline`: Renders within terminal flow, respects scroll
 * - `append`: Appends output without clearing
 */
export type MountRenderMode = 'fullscreen' | 'inline' | 'append'

export interface MountOptions {
  /** Render mode: fullscreen (default), inline, or append */
  mode?: MountRenderMode

  /** Terminal width (auto-detected if not specified) */
  width?: number

  /** Terminal height (auto-detected if not specified) */
  height?: number

  /** Disable Ctrl+C exit handling (default: enabled) */
  disableCtrlC?: boolean

  /** Disable Tab focus navigation (default: enabled) */
  disableTabNavigation?: boolean

  /** Disable mouse support (default: enabled) */
  disableMouse?: boolean

  /** Callback when app is unmounted */
  onUnmount?: () => void

  /** Use noop notifier (for testing without Rust) */
  noopNotifier?: boolean

  /** Maximum number of nodes (default: 10,000) */
  maxNodes?: number

  /** Text pool size in bytes (default: 10MB) */
  textPoolSize?: number
}

export interface MountHandle {
  /** Unmount the app and clean up */
  unmount(): void

  /** Get the shared buffer for direct access */
  buffer: SharedBuffer

  /** Get the Rust engine for direct access */
  engine: SparkEngine

  /** Switch render mode at runtime */
  setMode(mode: MountRenderMode): void

  /** Get current render mode */
  getMode(): MountRenderMode

  /** Block until the app exits (for power users who use mountSync) */
  waitForExit(): Promise<void>
}

// =============================================================================
// STATE
// =============================================================================

let currentCleanup: Cleanup | null = null
let currentMode: MountRenderMode = 'fullscreen'
let mounted = false
let exitUnsubscribe: Cleanup | null = null
let currentEngine: SparkEngine | null = null
let exitResolver: (() => void) | null = null

// =============================================================================
// RENDER MODE
// =============================================================================

function renderModeToEnum(mode: MountRenderMode): RenderMode {
  switch (mode) {
    case 'fullscreen': return RenderMode.Diff
    case 'inline': return RenderMode.Inline
    case 'append': return RenderMode.Append
    default: return RenderMode.Diff
  }
}

function applyRenderMode(buffer: SharedBuffer, mode: MountRenderMode): void {
  setRenderMode(buffer, renderModeToEnum(mode))
  currentMode = mode
}

// =============================================================================
// TERMINAL SIZE
// =============================================================================

function getTerminalSize(): { width: number; height: number } {
  if (typeof process !== 'undefined' && process.stdout) {
    return {
      width: process.stdout.columns ?? 80,
      height: process.stdout.rows ?? 24,
    }
  }
  return { width: 80, height: 24 }
}

// =============================================================================
// MOUNT SYNC (Power users, tests)
// =============================================================================

/**
 * Mount a SparkTUI application synchronously.
 *
 * Returns a handle for manual control. Use this for:
 * - Tests that need to inspect state
 * - Power users who want fine-grained control
 * - Apps that need to do work after mounting before blocking
 *
 * For most apps, use `mount()` instead which handles everything.
 *
 * @param app - The app function that creates the UI
 * @param options - Mount options (render mode, terminal size, etc.)
 * @returns A handle to control the mounted app
 *
 * @example Power user pattern
 * ```ts
 * const app = mountSync(() => {
 *   box({ children: () => text({ content: 'Hello!' }) })
 * })
 * // Do something with app.buffer or app.engine
 * await app.waitForExit()
 * ```
 *
 * @example Test pattern
 * ```ts
 * const app = mountSync(() => { ... }, { noopNotifier: true })
 * // Inspect app.buffer
 * app.unmount()
 * ```
 */
export function mountSync(app: () => void, options: MountOptions = {}): MountHandle {
  if (mounted) {
    throw new Error('SparkTUI is already mounted. Call unmount() first.')
  }

  const {
    mode = 'fullscreen',
    width,
    height,
    disableCtrlC = false,
    disableTabNavigation = false,
    disableMouse = false,
    onUnmount,
    noopNotifier = false,
    maxNodes,
    textPoolSize,
  } = options

  // Load engine FIRST (we need engine.wake for the notifier)
  let engine: SparkEngine
  if (!noopNotifier) {
    engine = loadEngine()
    currentEngine = engine
  } else {
    // Create a noop engine for tests
    engine = {
      init: () => 0,
      bufferSize: () => 0,
      wake: () => { },
      waitForEvents: () => { },
      cleanup: () => { },
      close: () => { },
    }
  }

  // Initialize bridge with FFI wake function (~5ns vs 500-2000μs!)
  const { buffer } = initBridge({
    noopNotifier,
    maxNodes,
    textPoolSize,
    wakeFn: engine.wake,
  })

  // Set terminal size
  const termSize = getTerminalSize()
  setTerminalSize(buffer, width ?? termSize.width, height ?? termSize.height)

  // Set render mode in shared buffer
  applyRenderMode(buffer, mode)

  // Set config flags
  let flags = CONFIG_DEFAULT
  if (disableCtrlC) {
    flags &= ~CONFIG_EXIT_ON_CTRL_C
  }
  if (disableTabNavigation) {
    flags &= ~CONFIG_TAB_NAVIGATION
  }
  if (disableMouse) {
    flags &= ~CONFIG_MOUSE_ENABLED
  }
  setConfigFlags(buffer, flags)

  // Initialize Rust engine with the buffer
  if (!noopNotifier) {
    const result = engine.init(ptr(buffer.raw), buffer.raw.byteLength)
    if (result !== 0) {
      throw new Error(`SparkTUI engine init failed with code ${result}`)
    }
  }

  // Start event listener (worker-based - TRUE 0% CPU, non-blocking main thread)
  if (!noopNotifier) {
    startEventListener(buffer, getLibPath())
  }

  // Create exit promise that resolves when app exits
  const exitPromise = new Promise<void>((resolve) => {
    exitResolver = resolve
  })

  // Create the mount handle
  const handle: MountHandle = {
    unmount() {
      if (!mounted) return

      if (exitUnsubscribe) {
        exitUnsubscribe()
        exitUnsubscribe = null
      }

      stopEventListener()
      cleanupAllHandlers()

      if (currentCleanup) {
        currentCleanup()
        currentCleanup = null
      }

      // Cleanup engine
      if (currentEngine) {
        currentEngine.cleanup()
        currentEngine.close()
        currentEngine = null
      }

      resetBridge()

      mounted = false
      currentMode = 'fullscreen'

      // Resolve the exit promise
      if (exitResolver) {
        exitResolver()
        exitResolver = null
      }

      onUnmount?.()
    },

    buffer,
    engine,

    setMode(newMode: MountRenderMode) {
      applyRenderMode(buffer, newMode)
    },

    getMode() {
      return currentMode
    },

    waitForExit() {
      return exitPromise
    },
  }

  // Register exit handler (Ctrl+C, 'q', etc. from Rust)
  if (!disableCtrlC) {
    exitUnsubscribe = registerExitHandler(() => {
      handle.unmount()
      process.exit(0)
    })
  }

  // Run app in scoped context
  currentCleanup = scoped(() => {
    app()
  })

  mounted = true

  return handle
}

// =============================================================================
// MOUNT (Most users)
// =============================================================================

/**
 * Mount a SparkTUI application.
 *
 * This is THE entry point for SparkTUI apps. It handles everything:
 * - Bridge initialization (SharedArrayBuffer + reactive arrays)
 * - Rust engine loading and initialization
 * - Event listener startup (Atomics.waitAsync - reactive, not polling)
 * - Render mode configuration
 * - Terminal size detection
 * - Blocks until the app exits (Ctrl+C, 'q', etc.)
 * - Clean unmount with full cleanup
 *
 * @param app - The app function that creates the UI
 * @param options - Mount options (render mode, terminal size, etc.)
 * @returns Promise that resolves when the app exits
 *
 * @example Simple app
 * ```ts
 * await mount(() => {
 *   box({
 *     children: () => text({ content: 'Hello, SparkTUI!' })
 *   })
 * })
 * // App has exited
 * ```
 *
 * @example Fullscreen with options
 * ```ts
 * await mount(() => {
 *   box({ width: '100%', height: '100%', children: () => {
 *     text({ content: 'Fullscreen!' })
 *   }})
 * }, { mode: 'fullscreen', disableMouse: true })
 * ```
 *
 * @example Inline mode (renders within terminal flow)
 * ```ts
 * await mount(() => {
 *   box({ children: () => text({ content: 'Inline content' }) })
 * }, { mode: 'inline' })
 * ```
 */
export async function mount(app: () => void, options: MountOptions = {}): Promise<void> {
  const handle = mountSync(app, options)
  await handle.waitForExit()
}

// =============================================================================
// HELPERS
// =============================================================================

/** Check if SparkTUI is currently mounted */
export function isMounted(): boolean {
  return mounted
}

/** Get the current render mode */
export function getRenderMode(): MountRenderMode {
  return currentMode
}

/**
 * Convenience function for testing - mount and immediately get buffer access.
 * Automatically uses noopNotifier for testing without Rust engine.
 */
export function mountForTest(
  app: () => void,
  options: Omit<MountOptions, 'noopNotifier'> = {}
): MountHandle {
  return mountSync(app, { ...options, noopNotifier: true })
}
