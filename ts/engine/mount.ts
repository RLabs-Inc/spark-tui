/**
 * SparkTUI Mount API
 *
 * The main entry point for SparkTUI applications.
 * Handles bridge initialization, event listener, render mode, and cleanup.
 *
 * PURELY REACTIVE: No loops. Change propagates through the dependency graph.
 * The event listener uses Atomics.waitAsync - it SUSPENDS until Rust notifies.
 */

import { initBridgeAoS, resetBridge } from '../bridge'
import {
  startEventListener,
  stopEventListener,
  registerExitHandler,
  cleanupAllHandlers,
} from './events'
import { scoped } from '../primitives/scope'
import {
  setConfigFlags,
  setTerminalSize,
  CONFIG_DEFAULT,
  CONFIG_EXIT_ON_CTRL_C,
  CONFIG_TAB_NAVIGATION,
  CONFIG_MOUSE_ENABLED,
  H_RENDER_MODE,
  type AoSBuffer,
} from '../bridge/shared-buffer-aos'
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
export type RenderMode = 'fullscreen' | 'inline' | 'append'

export interface MountOptions {
  /** Render mode: fullscreen (default), inline, or append */
  mode?: RenderMode

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
}

export interface MountHandle {
  /** Unmount the app and clean up */
  unmount(): void

  /** Get the AoS buffer for direct access */
  buffer: AoSBuffer

  /** Switch render mode at runtime */
  setMode(mode: RenderMode): void

  /** Get current render mode */
  getMode(): RenderMode
}

// =============================================================================
// STATE
// =============================================================================

let currentCleanup: Cleanup | null = null
let currentMode: RenderMode = 'fullscreen'
let mounted = false
let exitUnsubscribe: Cleanup | null = null

// =============================================================================
// RENDER MODE
// =============================================================================

function renderModeToNum(mode: RenderMode): number {
  switch (mode) {
    case 'fullscreen':
      return 0
    case 'inline':
      return 1
    case 'append':
      return 2
    default:
      return 0
  }
}

function applyRenderMode(buffer: AoSBuffer, mode: RenderMode): void {
  buffer.view.setUint32(H_RENDER_MODE, renderModeToNum(mode), true)
  currentMode = mode
}

// =============================================================================
// TERMINAL SIZE
// =============================================================================

function getTerminalSize(): { width: number; height: number } {
  // Node.js environment
  if (typeof process !== 'undefined' && process.stdout) {
    return {
      width: process.stdout.columns ?? 80,
      height: process.stdout.rows ?? 24,
    }
  }
  // Fallback
  return { width: 80, height: 24 }
}

// =============================================================================
// MOUNT
// =============================================================================

/**
 * Mount a SparkTUI application.
 *
 * This is THE entry point for SparkTUI apps. It handles:
 * - Bridge initialization (SharedArrayBuffer + reactive arrays + notifier)
 * - Event listener startup (Atomics.waitAsync - reactive, not polling)
 * - Render mode configuration
 * - Terminal size detection
 * - Clean unmount with full cleanup
 *
 * @param app - The app function that creates the UI
 * @param options - Mount options (render mode, terminal size, etc.)
 * @returns A handle to control the mounted app
 *
 * @example Fullscreen app (default)
 * ```ts
 * const { unmount } = mount(() => {
 *   box({ width: '100%', height: '100%', children: () => {
 *     text({ content: 'Hello, SparkTUI!' })
 *   }})
 * })
 *
 * // Later: unmount()
 * ```
 *
 * @example Inline mode (renders within terminal flow)
 * ```ts
 * mount(() => {
 *   box({ children: () => {
 *     text({ content: 'Inline content' })
 *   }})
 * }, { mode: 'inline', height: 10 })
 * ```
 *
 * @example With custom config
 * ```ts
 * mount(() => {
 *   // ... app UI
 * }, {
 *   mode: 'fullscreen',
 *   disableCtrlC: false,
 *   disableMouse: false,
 *   onUnmount: () => console.log('App unmounted'),
 * })
 * ```
 *
 * @example Testing without Rust
 * ```ts
 * mount(() => {
 *   // ... app UI
 * }, { noopNotifier: true })
 * ```
 */
export function mount(app: () => void, options: MountOptions = {}): MountHandle {
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
  } = options

  // Initialize bridge (SharedArrayBuffer + reactive arrays + notifier)
  const { buffer, arrays, notifier } = initBridgeAoS({ noopNotifier })

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

  // Start event listener (Atomics.waitAsync - REACTIVE, NOT POLLING)
  // This SUSPENDS until Rust notifies via Atomics.notify
  if (!noopNotifier) {
    startEventListener(buffer)
  }

  // Create the mount handle first so exitHandler can reference it
  const handle: MountHandle = {
    unmount() {
      if (!mounted) return

      // Unsubscribe exit handler
      if (exitUnsubscribe) {
        exitUnsubscribe()
        exitUnsubscribe = null
      }

      // Stop event listener
      stopEventListener()

      // Clean up all event handlers
      cleanupAllHandlers()

      // Dispose scope (cleans up all primitives and effects)
      if (currentCleanup) {
        currentCleanup()
        currentCleanup = null
      }

      // Reset bridge state (allows re-mounting)
      resetBridge()

      mounted = false
      currentMode = 'fullscreen'

      // Call user callback
      onUnmount?.()
    },

    buffer,

    setMode(newMode: RenderMode) {
      applyRenderMode(buffer, newMode)
    },

    getMode() {
      return currentMode
    },
  }

  // Register Ctrl+C handler (if enabled)
  if (!disableCtrlC) {
    exitUnsubscribe = registerExitHandler(() => {
      handle.unmount()
      process.exit(0)
    })
  }

  // Run app in scoped context
  // scoped() tracks all effects and cleanups, returns a master cleanup function
  currentCleanup = scoped(() => {
    app()
  })

  mounted = true

  return handle
}

// =============================================================================
// HELPERS
// =============================================================================

/** Check if SparkTUI is currently mounted */
export function isMounted(): boolean {
  return mounted
}

/** Get the current render mode */
export function getRenderMode(): RenderMode {
  return currentMode
}

/**
 * Convenience function for testing - mount and immediately get buffer access.
 * Automatically uses noopNotifier for testing without Rust.
 */
export function mountForTest(
  app: () => void,
  options: Omit<MountOptions, 'noopNotifier'> = {}
): MountHandle {
  return mount(app, { ...options, noopNotifier: true })
}
