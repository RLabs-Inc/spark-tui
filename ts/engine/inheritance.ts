/**
 * TUI Framework - Color and Style Inheritance
 *
 * Utilities for walking up the component tree to inherit colors and styles.
 * Used when a component has null colors (meaning "inherit from parent").
 */

import type { RGBA } from '../types'
import { TERMINAL_DEFAULT } from '../types/color'
import { getArrays, isInitialized } from '../bridge'
import { unpackColor } from '../bridge/shared-buffer'

/**
 * Get inherited foreground color by walking up the parent tree.
 * Returns TERMINAL_DEFAULT if no explicit color is found.
 */
export function getInheritedFg(index: number): RGBA {
  if (!isInitialized()) return TERMINAL_DEFAULT

  const arrays = getArrays()
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

  const arrays = getArrays()
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
 */
export function getInheritedBorderColor(index: number, _side: 'top' | 'right' | 'bottom' | 'left'): RGBA {
  if (!isInitialized()) return TERMINAL_DEFAULT

  const arrays = getArrays()

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

  const arrays = getArrays()

  // Check per-side colors first, fall back to unified
  const unifiedPacked = arrays.borderColor.get(index)
  const unified = unifiedPacked !== 0 ? unpackColor(unifiedPacked) : fg

  const topPacked = arrays.borderTopColor.get(index)
  const rightPacked = arrays.borderRightColor.get(index)
  const bottomPacked = arrays.borderBottomColor.get(index)
  const leftPacked = arrays.borderLeftColor.get(index)

  return {
    top: topPacked !== 0 ? unpackColor(topPacked) : unified,
    right: rightPacked !== 0 ? unpackColor(rightPacked) : unified,
    bottom: bottomPacked !== 0 ? unpackColor(bottomPacked) : unified,
    left: leftPacked !== 0 ? unpackColor(leftPacked) : unified,
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

  const arrays = getArrays()

  return {
    top: arrays.borderWidthTop.get(index),
    right: arrays.borderWidthRight.get(index),
    bottom: arrays.borderWidthBottom.get(index),
    left: arrays.borderWidthLeft.get(index),
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

  const arrays = getArrays()
  let opacity = 1
  let current: number = index

  while (current >= 0) {
    const nodeOpacity = arrays.opacity.get(current)
    // opacity is stored as f32 (0.0-1.0), default is 1.0
    if (nodeOpacity > 0 && nodeOpacity < 1) {
      opacity *= nodeOpacity
    }
    current = arrays.parentIndex.get(current)
  }

  return opacity
}
