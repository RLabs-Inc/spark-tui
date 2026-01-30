/**
 * TUI Framework - Color and Style Inheritance
 *
 * Utilities for walking up the component tree to inherit colors and styles.
 * Used when a component has null colors (meaning "inherit from parent").
 */

import type { RGBA } from '../types'
import { TERMINAL_DEFAULT } from '../types/color'
import { getAoSArrays, isInitialized } from '../bridge'
import { unpackColor } from '../bridge/shared-buffer-aos'

/**
 * Get inherited foreground color by walking up the parent tree.
 * Returns TERMINAL_DEFAULT if no explicit color is found.
 */
export function getInheritedFg(index: number): RGBA {
  if (!isInitialized()) return TERMINAL_DEFAULT

  const arrays = getAoSArrays()
  let current: number = index

  while (current >= 0) {
    const fgPacked = arrays.fgColor.get(current)
    if (fgPacked !== 0) return unpackColor(fgPacked)
    const parent = arrays.parentIndex.get(current)
    if (parent < 0) break
    current = parent
  }

  return TERMINAL_DEFAULT
}

/**
 * Get inherited background color by walking up the parent tree.
 * Returns TERMINAL_DEFAULT if no explicit color is found.
 */
export function getInheritedBg(index: number): RGBA {
  if (!isInitialized()) return TERMINAL_DEFAULT

  const arrays = getAoSArrays()
  let current: number = index

  while (current >= 0) {
    const bgPacked = arrays.bgColor.get(current)
    if (bgPacked !== 0) return unpackColor(bgPacked)
    const parent = arrays.parentIndex.get(current)
    if (parent < 0) break
    current = parent
  }

  return TERMINAL_DEFAULT
}

/**
 * Get inherited border color.
 * Falls back to unified border color, then foreground color.
 * Note: Per-side border colors not yet in AoS layout.
 */
export function getInheritedBorderColor(index: number, _side: 'top' | 'right' | 'bottom' | 'left'): RGBA {
  if (!isInitialized()) return TERMINAL_DEFAULT

  const arrays = getAoSArrays()

  // Try unified border color
  const borderPacked = arrays.borderColor.get(index)
  if (borderPacked !== 0) return unpackColor(borderPacked)

  // Fall back to foreground color
  return getInheritedFg(index)
}

/**
 * Get all four border colors for a component.
 */
export function getBorderColors(index: number): {
  top: RGBA
  right: RGBA
  bottom: RGBA
  left: RGBA
} {
  const fg = getInheritedFg(index)

  if (!isInitialized()) {
    return { top: fg, right: fg, bottom: fg, left: fg }
  }

  const arrays = getAoSArrays()
  const borderPacked = arrays.borderColor.get(index)
  const fallback = borderPacked !== 0 ? unpackColor(borderPacked) : fg

  // Note: Per-side border colors not yet in AoS layout, use unified
  return {
    top: fallback,
    right: fallback,
    bottom: fallback,
    left: fallback,
  }
}

/**
 * Get all four border styles for a component.
 * Falls back to the unified borderStyle if per-side not set.
 */
export function getBorderStyles(index: number): {
  top: number
  right: number
  bottom: number
  left: number
} {
  if (!isInitialized()) {
    return { top: 0, right: 0, bottom: 0, left: 0 }
  }

  const arrays = getAoSArrays()

  return {
    top: arrays.borderTopWidth.get(index),
    right: arrays.borderRightWidth.get(index),
    bottom: arrays.borderBottomWidth.get(index),
    left: arrays.borderLeftWidth.get(index),
  }
}

/**
 * Check if a component has any border.
 */
export function hasBorder(index: number): boolean {
  const styles = getBorderStyles(index)
  return styles.top > 0 || styles.right > 0 || styles.bottom > 0 || styles.left > 0
}

/**
 * Get effective opacity by multiplying down the parent chain.
 */
export function getEffectiveOpacity(index: number): number {
  if (!isInitialized()) return 1

  const arrays = getAoSArrays()
  let opacity = 1
  let current: number = index

  while (current >= 0) {
    const nodeOpacity = arrays.opacity.get(current)
    if (nodeOpacity !== 0 && nodeOpacity !== 255) {
      // opacity is stored as u8 (0-255), convert to 0-1
      opacity *= nodeOpacity / 255
    }
    current = arrays.parentIndex.get(current)
  }

  return opacity
}
