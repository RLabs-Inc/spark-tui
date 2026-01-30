/**
 * SparkTUI - Drawn Cursor State Module
 *
 * Manages cursor appearance in input fields. The cursor blink uses the
 * `pulse()` animation primitive - timing is handled entirely in TypeScript.
 *
 * CRITICAL: Cursor blink is TS signal-based, NOT Rust timing.
 * - `pulse({ fps: 2 })` creates a signal that toggles true/false
 * - The signal value flows through `repeat()` to SharedBuffer
 * - Rust reads the value and renders accordingly
 * - Rust has ZERO timing logic for cursor blink
 *
 * @example Simple blinking cursor
 * ```ts
 * const cursor = createCursor(index, {
 *   style: 'bar',
 *   visible: pulse({ fps: 2 }),  // Blink at 2 FPS
 * })
 * ```
 *
 * @example Custom appearance
 * ```ts
 * const cursor = createCursor(index, {
 *   style: 'block',
 *   char: '\u2588',  // Full block
 *   fg: 0xFFFFFFFF,
 *   bg: 0xFF0000FF,
 *   visible: true,  // Always visible, no blink
 * })
 * ```
 *
 * @example Animated colors
 * ```ts
 * const cursor = createCursor(index, {
 *   visible: pulse({ fps: 2 }),
 *   fg: cycle([red, orange, yellow], { fps: 4 }),  // Color cycle
 * })
 * ```
 */

import { signal, repeat } from '@rlabs-inc/signals'
import type { WritableSignal, ReadableSignal } from '@rlabs-inc/signals'
import { pulse } from '../primitives/animation'
import { getAoSArrays } from '../bridge'

// =============================================================================
// TYPES
// =============================================================================

export type CursorStyle = 'block' | 'bar' | 'underline'

export interface CursorConfig {
  /** Cursor style: block (default), bar, or underline */
  style?: CursorStyle | ReadableSignal<CursorStyle> | (() => CursorStyle)

  /** Custom character to display as cursor (UTF-32 codepoint) */
  char?: string | ReadableSignal<string> | (() => string)

  /** Cursor visibility - can be animated! Use pulse() for blink */
  visible?: boolean | ReadableSignal<boolean> | (() => boolean)

  /** Foreground color (ARGB packed) - can be animated! */
  fg?: number | ReadableSignal<number> | (() => number)

  /** Background color (ARGB packed) - can be animated! */
  bg?: number | ReadableSignal<number> | (() => number)
}

export interface CursorHandle {
  /** Update cursor position (0-based index in text) */
  setPosition(pos: number): void

  /** Get current cursor position */
  getPosition(): number

  /** Update visibility (overrides config.visible if static) */
  setVisible(visible: boolean): void

  /** Check if cursor is currently visible */
  isVisible(): boolean

  /** Dispose cursor resources and stop animations */
  dispose(): void
}

// =============================================================================
// STYLE CONVERSION
// =============================================================================

/**
 * Convert cursor style string to numeric value for SharedBuffer.
 * - 0 = block (default)
 * - 1 = bar (thin vertical line)
 * - 2 = underline
 */
function styleToNum(style: CursorStyle | undefined): number {
  switch (style) {
    case 'bar': return 1
    case 'underline': return 2
    default: return 0 // block
  }
}

/**
 * Convert character to UTF-32 codepoint.
 * Default is full block character (U+2588).
 */
function charToCodepoint(char: string | undefined): number {
  if (!char || char.length === 0) return 0x2588 // Default: full block
  return char.codePointAt(0) ?? 0x2588
}

/**
 * Check if a value is reactive (signal or getter function)
 */
function isReactive<T>(value: T | ReadableSignal<T> | (() => T)): boolean {
  return typeof value === 'function' || (value !== null && typeof value === 'object' && 'value' in (value as any))
}

/**
 * Unwrap a value that might be a signal or getter
 */
function unwrap<T>(value: T | ReadableSignal<T> | (() => T)): T {
  if (typeof value === 'function') return (value as () => T)()
  if (value !== null && typeof value === 'object' && 'value' in (value as object)) {
    return (value as ReadableSignal<T>).value
  }
  return value as T
}

// =============================================================================
// CREATE CURSOR
// =============================================================================

/**
 * Create a cursor for an input component.
 *
 * The cursor visibility can be animated using pulse() for blink effect.
 * All cursor properties are written to SharedBuffer and Rust renders them.
 *
 * @param index - Component index in the SharedBuffer
 * @param config - Cursor configuration (style, char, visible, colors)
 * @returns CursorHandle for controlling the cursor
 *
 * @example Simple blinking cursor
 * ```ts
 * const cursor = createCursor(index, {
 *   style: 'bar',
 *   visible: pulse({ fps: 2 }),  // Blink at 2 FPS
 * })
 * ```
 *
 * @example Custom appearance
 * ```ts
 * const cursor = createCursor(index, {
 *   style: 'block',
 *   char: '\u2588',
 *   fg: 0xFFFFFFFF,
 *   bg: 0xFF0000FF,
 *   visible: true,  // Always visible, no blink
 * })
 * ```
 */
export function createCursor(index: number, config: CursorConfig = {}): CursorHandle {
  const arrays = getAoSArrays()
  const disposals: (() => void)[] = []

  // Position signal (internal) - allows programmatic control
  const positionSignal = signal(0)
  disposals.push(repeat(positionSignal, arrays.cursorPosition, index))

  // --------------------------------------------------------------------------
  // STYLE
  // --------------------------------------------------------------------------
  if (config.style !== undefined) {
    if (typeof config.style === 'string') {
      // Static style
      arrays.cursorStyle.set(index, styleToNum(config.style))
    } else {
      // Reactive style
      disposals.push(repeat(
        () => styleToNum(unwrap(config.style!)),
        arrays.cursorStyle,
        index
      ))
    }
  }

  // --------------------------------------------------------------------------
  // CHARACTER
  // --------------------------------------------------------------------------
  if (config.char !== undefined) {
    if (typeof config.char === 'string') {
      // Static character
      arrays.cursorChar.set(index, charToCodepoint(config.char))
    } else {
      // Reactive character
      disposals.push(repeat(
        () => charToCodepoint(unwrap(config.char!)),
        arrays.cursorChar,
        index
      ))
    }
  }

  // --------------------------------------------------------------------------
  // VISIBILITY (the key reactive property - uses pulse() for blink!)
  // --------------------------------------------------------------------------
  // Internal signal for manual visibility control
  const visibilitySignal = signal(true)
  let usingDefaultBlink = false

  if (config.visible !== undefined) {
    if (typeof config.visible === 'boolean') {
      // Static visibility - use internal signal for setVisible() control
      visibilitySignal.value = config.visible
      disposals.push(repeat(() => visibilitySignal.value ? 1 : 0, arrays.cursorFlags, index))
    } else {
      // Reactive visibility (could be pulse() signal!)
      disposals.push(repeat(
        () => {
          const v = unwrap(config.visible!)
          return v ? 1 : 0
        },
        arrays.cursorFlags,
        index
      ))
    }
  } else {
    // Default: blinking cursor at 2 FPS
    usingDefaultBlink = true
    const blinkSignal = pulse({ fps: 2 })
    disposals.push(repeat(() => blinkSignal.value ? 1 : 0, arrays.cursorFlags, index))
  }

  // --------------------------------------------------------------------------
  // FOREGROUND COLOR
  // --------------------------------------------------------------------------
  if (config.fg !== undefined) {
    if (typeof config.fg === 'number') {
      // Static color
      arrays.cursorFg.set(index, config.fg)
    } else {
      // Reactive color (can be animated with cycle()!)
      disposals.push(repeat(
        () => unwrap(config.fg!),
        arrays.cursorFg,
        index
      ))
    }
  }

  // --------------------------------------------------------------------------
  // BACKGROUND COLOR
  // --------------------------------------------------------------------------
  if (config.bg !== undefined) {
    if (typeof config.bg === 'number') {
      // Static color
      arrays.cursorBg.set(index, config.bg)
    } else {
      // Reactive color (can be animated with cycle()!)
      disposals.push(repeat(
        () => unwrap(config.bg!),
        arrays.cursorBg,
        index
      ))
    }
  }

  // --------------------------------------------------------------------------
  // HANDLE
  // --------------------------------------------------------------------------
  return {
    setPosition(pos: number): void {
      positionSignal.value = pos
    },

    getPosition(): number {
      return positionSignal.value
    },

    setVisible(visible: boolean): void {
      // Only works for static visibility or default blink
      // For reactive visibility, the signal controls it
      visibilitySignal.value = visible
    },

    isVisible(): boolean {
      // Read from internal signal (may not reflect reactive visibility)
      return visibilitySignal.value
    },

    dispose(): void {
      for (const d of disposals) d()
      disposals.length = 0
    },
  }
}

// =============================================================================
// PRESET CONFIGURATIONS
// =============================================================================

/**
 * Preset cursor configurations for common use cases.
 *
 * @example Using a preset
 * ```ts
 * const cursor = createCursor(index, CursorPresets.bar)
 * ```
 *
 * @example Extending a preset
 * ```ts
 * const cursor = createCursor(index, {
 *   ...CursorPresets.bar,
 *   fg: 0xFF00FFFF,  // Cyan
 * })
 * ```
 */
export const CursorPresets = {
  /** Standard blinking block cursor (like vim normal mode) */
  block: { style: 'block' as const, visible: pulse({ fps: 2 }) },

  /** Bar cursor (like VS Code, thin vertical line) */
  bar: { style: 'bar' as const, visible: pulse({ fps: 2 }) },

  /** Underline cursor (like classic terminals) */
  underline: { style: 'underline' as const, visible: pulse({ fps: 2 }) },

  /** Always visible, no blink */
  solid: { visible: true },

  /** Hidden cursor */
  hidden: { visible: false },
} as const

// =============================================================================
// NOTE: Types and functions are exported inline above via 'export' keywords.
// =============================================================================
