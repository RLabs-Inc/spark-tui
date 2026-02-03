/**
 * TUI Framework - Animation Primitives
 *
 * Animation utilities for spinners, cursors, progress indicators, etc.
 * These are SIGNAL SOURCES, not render loops.
 *
 * The pattern:
 * 1. setInterval toggles a signal value periodically
 * 2. Signal change propagates through repeat() to SharedBuffer
 * 3. Rust wakes (via Atomics) and reads the new value
 * 4. Rust renders whatever value is there
 *
 * This is REACTIVE - the timing is just updating signal values.
 * There is NO fixed FPS rendering. Rust renders when data changes.
 *
 * @example Spinner animation
 * ```ts
 * text({ content: cycle(Frames.spinner, { fps: 12 }) })
 * ```
 *
 * @example Cursor blink
 * ```ts
 * input({ cursor: { visible: pulse({ fps: 2 }) } })
 * ```
 *
 * @example Color cycle
 * ```ts
 * box({ bg: cycle([red, green, blue], { fps: 2 }) })
 * ```
 */

import { signal, effect } from '@rlabs-inc/signals'
import type { WritableSignal, ReadableSignal } from '@rlabs-inc/signals'
import { getActiveScope } from './scope'

// =============================================================================
// TYPES
// =============================================================================

export interface AnimationOptions {
  /** Frames per second - how often the signal updates (default: 10) */
  fps?: number
  /** Whether animation is active. Can be a signal for reactive control. */
  active?: boolean | ReadableSignal<boolean> | (() => boolean)
  /** Start immediately or wait for first read (default: true) */
  autoStart?: boolean
}

export interface CycleOptions<T> extends AnimationOptions {
  /** Starting index in the frames array (default: 0) */
  startIndex?: number
}

export interface PulseOptions extends AnimationOptions {
  /** Duration in ms for 'on' state. If set, uses custom timing instead of even split. */
  onDuration?: number
}

// =============================================================================
// SHARED CLOCK REGISTRY
// Optimization: share timers across same-FPS animations
// =============================================================================

interface ClockEntry {
  interval: ReturnType<typeof setInterval>
  subscribers: Set<() => void>
  frameCount: number
}

const clocks = new Map<number, ClockEntry>()

/**
 * Get or create a shared clock for the given FPS.
 * Multiple animations at the same FPS share a single setInterval.
 */
function getOrCreateClock(fps: number): ClockEntry {
  if (!clocks.has(fps)) {
    const entry: ClockEntry = {
      interval: setInterval(() => {
        entry.frameCount++
        for (const sub of entry.subscribers) sub()
      }, 1000 / fps),
      subscribers: new Set(),
      frameCount: 0,
    }
    clocks.set(fps, entry)
  }
  return clocks.get(fps)!
}

/**
 * Release a clock subscription. Cleans up the interval when no subscribers remain.
 */
function releaseClock(fps: number, subscriber: () => void): void {
  const entry = clocks.get(fps)
  if (!entry) return
  entry.subscribers.delete(subscriber)
  if (entry.subscribers.size === 0) {
    clearInterval(entry.interval)
    clocks.delete(fps)
  }
}

// =============================================================================
// CYCLE - Core animation primitive
// =============================================================================

/**
 * Create a signal that cycles through values at the given FPS.
 *
 * This is a SIGNAL SOURCE, not a render loop. The signal updates
 * at the specified rate, and those updates propagate reactively
 * through the system.
 *
 * @example Spinner animation
 * ```ts
 * text({ content: cycle(Frames.spinner, { fps: 12 }) })
 * ```
 *
 * @example Color cycle
 * ```ts
 * box({ bg: cycle([red, green, blue], { fps: 2 }) })
 * ```
 *
 * @example Conditional animation
 * ```ts
 * const isLoading = signal(true)
 * text({ content: cycle(Frames.spinner, { fps: 12, active: isLoading }) })
 * // Animation pauses when isLoading is false
 * ```
 */
export function cycle<T>(frames: readonly T[], options: CycleOptions<T> = {}): WritableSignal<T> {
  const { fps = 10, active = true, startIndex = 0, autoStart = true } = options

  if (frames.length === 0) {
    throw new Error('cycle() requires at least one frame')
  }

  let currentIndex = startIndex % frames.length
  const sig = signal(frames[currentIndex]!)

  const tick = () => {
    currentIndex = (currentIndex + 1) % frames.length
    sig.value = frames[currentIndex]!
  }

  // Handle reactive active prop
  const isActive = (): boolean => {
    if (typeof active === 'boolean') return active
    if (typeof active === 'function') return active()
    return active.value
  }

  let cleanup: (() => void) | null = null

  const start = () => {
    if (cleanup) return
    const clock = getOrCreateClock(fps)
    clock.subscribers.add(tick)
    cleanup = () => releaseClock(fps, tick)
  }

  const stop = () => {
    cleanup?.()
    cleanup = null
  }

  // React to active changes if it's reactive
  if (typeof active !== 'boolean') {
    effect(() => {
      if (isActive()) start()
      else stop()
    })
  } else if (active && autoStart) {
    start()
  }

  // Auto-cleanup with scope
  const scope = getActiveScope()
  if (scope) {
    scope.cleanups.push(() => {
      stop()
    })
  }

  return sig
}

// =============================================================================
// PULSE - Boolean blink shorthand
// =============================================================================

/**
 * Create a signal that toggles between true/false (blink).
 *
 * Shorthand for cycle([true, false], options).
 *
 * @example Cursor blink at 2 FPS (500ms on, 500ms off)
 * ```ts
 * input({ cursor: { visible: pulse({ fps: 2 }) } })
 * ```
 *
 * @example Faster blink
 * ```ts
 * input({ cursor: { visible: pulse({ fps: 4 }) } })
 * ```
 *
 * @example Custom on/off timing (300ms on, 700ms off at 1 FPS)
 * ```ts
 * input({ cursor: { visible: pulse({ fps: 1, onDuration: 300 }) } })
 * ```
 */
export function pulse(options: PulseOptions = {}): WritableSignal<boolean> {
  const { fps = 2, onDuration, active = true, autoStart = true } = options

  if (onDuration !== undefined) {
    // Custom on/off timing - use dedicated setTimeout chain
    const period = 1000 / fps
    const sig = signal(true)

    // Handle reactive active prop
    const isActive = (): boolean => {
      if (typeof active === 'boolean') return active
      if (typeof active === 'function') return active()
      return active.value
    }

    let timeout: ReturnType<typeof setTimeout> | null = null
    let running = false

    const toggle = () => {
      if (!running) return
      sig.value = !sig.value
      const nextDuration = sig.value ? onDuration : period - onDuration
      timeout = setTimeout(toggle, Math.max(0, nextDuration))
    }

    const start = () => {
      if (running) return
      running = true
      sig.value = true
      timeout = setTimeout(toggle, onDuration)
    }

    const stop = () => {
      running = false
      if (timeout) {
        clearTimeout(timeout)
        timeout = null
      }
    }

    // React to active changes if it's reactive
    if (typeof active !== 'boolean') {
      effect(() => {
        if (isActive()) start()
        else stop()
      })
    } else if (active && autoStart) {
      start()
    }

    // Auto-cleanup with scope
    const scope = getActiveScope()
    if (scope) {
      scope.cleanups.push(() => {
        stop()
      })
    }

    return sig
  }

  // Standard even-split blink - delegate to cycle
  return cycle([true, false], { fps, active, autoStart })
}

// =============================================================================
// BUILT-IN FRAME SETS
// =============================================================================

/**
 * Built-in animation frame sets for common UI patterns.
 */
export const Frames = {
  /** Classic spinner: braille dots rotating */
  spinner: ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'] as const,

  /** Braille dots (vertical pattern) */
  dots: ['⣾', '⣽', '⣻', '⢿', '⡿', '⣟', '⣯', '⣷'] as const,

  /** Simple ASCII line spinner: -\|/ */
  line: ['-', '\\', '|', '/'] as const,

  /** Growing bar: thin to thick block */
  bar: ['▏', '▎', '▍', '▌', '▋', '▊', '▉', '█'] as const,

  /** Clock faces */
  clock: ['🕐', '🕑', '🕒', '🕓', '🕔', '🕕', '🕖', '🕗', '🕘', '🕙', '🕚', '🕛'] as const,

  /** Bouncing dot */
  bounce: ['⠁', '⠂', '⠄', '⠂'] as const,

  /** Rotating arrow */
  arrow: ['←', '↖', '↑', '↗', '→', '↘', '↓', '↙'] as const,

  /** Pulse ring */
  pulse: ['◯', '◔', '◑', '◕', '●', '◕', '◑', '◔'] as const,
} as const
