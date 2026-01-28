/**
 * TUI Framework - Box Primitive
 *
 * Container component with flexbox layout, borders, and background.
 * Children inherit parent context automatically.
 *
 * REACTIVITY: Props are passed directly to setSource() to preserve reactive links.
 * Don't extract values before binding - that breaks the connection!
 *
 * Usage:
 * ```ts
 * const width = signal(40)
 * box({
 *   width,  // Reactive! Changes to width.value update the UI
 *   height: 10,  // Static
 *   border: 1,
 *   children: () => {
 *     text({ content: 'Hello!' })
 *   }
 * })
 * ```
 */

import { ComponentType } from '../types'
import {
  allocateIndex,
  releaseIndex,
  getCurrentParentIndex,
  pushParentContext,
  popParentContext,
} from '../engine/registry'
import {
  pushCurrentComponent,
  popCurrentComponent,
  runMountCallbacks,
} from '../engine/lifecycle'
import { cleanupIndex as cleanupKeyboardListeners, onFocused } from '../state/keyboard'
import { registerFocusCallbacks, focus as focusComponent } from '../state/focus'
import { onComponent as onMouseComponent } from '../state/mouse'
import { getVariantStyle } from '../state/theme'
import { getActiveScope } from './scope'
import { enumSource } from './utils'

// Import arrays
import * as core from '../engine/arrays/core'
import * as taffy from '../engine/arrays/taffy'
import * as visual from '../engine/arrays/visual'
import * as interaction from '../engine/arrays/interaction'

// Import types
import type { BoxProps, Cleanup } from './types'

// =============================================================================
// HELPERS - Enum conversions
// =============================================================================

/** Convert flex direction string to number (matches taffy-native) */
function flexDirectionToNum(dir: string | undefined): number {
  switch (dir) {
    case 'row': return taffy.FLEX_DIRECTION_ROW
    case 'column': return taffy.FLEX_DIRECTION_COLUMN
    case 'row-reverse': return taffy.FLEX_DIRECTION_ROW_REVERSE
    case 'column-reverse': return taffy.FLEX_DIRECTION_COLUMN_REVERSE
    default: return taffy.FLEX_DIRECTION_COLUMN // default is column for TUI
  }
}

/** Convert flex wrap string to number */
function flexWrapToNum(wrap: string | undefined): number {
  switch (wrap) {
    case 'wrap': return taffy.FLEX_WRAP_WRAP
    case 'wrap-reverse': return taffy.FLEX_WRAP_WRAP_REVERSE
    default: return taffy.FLEX_WRAP_NOWRAP
  }
}

/** Convert justify content string to number */
function justifyToNum(justify: string | undefined): number {
  switch (justify) {
    case 'center': return taffy.JUSTIFY_CENTER
    case 'flex-end': return taffy.JUSTIFY_FLEX_END
    case 'space-between': return taffy.JUSTIFY_SPACE_BETWEEN
    case 'space-around': return taffy.JUSTIFY_SPACE_AROUND
    case 'space-evenly': return taffy.JUSTIFY_SPACE_EVENLY
    default: return taffy.JUSTIFY_FLEX_START
  }
}

/** Convert align items string to number */
function alignToNum(align: string | undefined): number {
  switch (align) {
    case 'flex-start': return taffy.ALIGN_FLEX_START
    case 'center': return taffy.ALIGN_CENTER
    case 'flex-end': return taffy.ALIGN_FLEX_END
    case 'baseline': return taffy.ALIGN_BASELINE
    default: return taffy.ALIGN_STRETCH
  }
}

/** Convert align-self string to number (0 = auto) */
function alignSelfToNum(align: string | undefined): number {
  switch (align) {
    case 'stretch': return taffy.ALIGN_SELF_STRETCH
    case 'flex-start': return taffy.ALIGN_SELF_FLEX_START
    case 'center': return taffy.ALIGN_SELF_CENTER
    case 'flex-end': return taffy.ALIGN_SELF_FLEX_END
    case 'baseline': return taffy.ALIGN_SELF_BASELINE
    default: return taffy.ALIGN_SELF_AUTO
  }
}

/** Convert border prop to width (0 or 1) */
function borderToWidth(
  prop: number | (() => number) | { value: number } | undefined
): number | (() => number) {
  if (prop === undefined) return 0
  if (typeof prop === 'number') return prop > 0 ? 1 : 0
  if (typeof prop === 'function') {
    return () => (prop as () => number)() > 0 ? 1 : 0
  }
  if (typeof prop === 'object' && prop !== null && 'value' in prop) {
    return () => (prop as { value: number }).value > 0 ? 1 : 0
  }
  return 0
}

// =============================================================================
// BOX COMPONENT
// =============================================================================

/**
 * Create a box container component.
 *
 * Boxes are the building blocks of layouts. They can:
 * - Have borders and backgrounds
 * - Use flexbox for child layout
 * - Contain other components as children
 * - Scroll their content
 *
 * Pass signals directly for reactive props - they stay connected!
 */
export function box(props: BoxProps = {}): Cleanup {
  const index = allocateIndex(props.id)

  // Track current component for lifecycle hooks
  pushCurrentComponent(index)

  // ==========================================================================
  // CORE - Always needed
  // ==========================================================================
  core.componentType[index] = ComponentType.BOX
  core.parentIndex.setSource(index, getCurrentParentIndex())

  // ==========================================================================
  // TAFFY LAYOUT ARRAYS - Direct binding to TypedArrays
  // ==========================================================================

  // Parent hierarchy (for tree structure)
  taffy.parentIndex.setSource(index, getCurrentParentIndex())

  // Visibility
  if (props.visible !== undefined) {
    taffy.visible.setSource(index, props.visible)
    core.visible.setSource(index, props.visible)
  }

  // Flex container properties
  if (props.flexDirection !== undefined) {
    taffy.flexDirection.setSource(index, enumSource(props.flexDirection, flexDirectionToNum))
  }
  if (props.flexWrap !== undefined) {
    taffy.flexWrap.setSource(index, enumSource(props.flexWrap, flexWrapToNum))
  }
  if (props.justifyContent !== undefined) {
    taffy.justifyContent.setSource(index, enumSource(props.justifyContent, justifyToNum))
  }
  if (props.alignItems !== undefined) {
    taffy.alignItems.setSource(index, enumSource(props.alignItems, alignToNum))
  }

  // Flex item properties
  if (props.grow !== undefined) {
    taffy.grow.setSource(index, props.grow)
  }
  if (props.shrink !== undefined) {
    taffy.shrink.setSource(index, props.shrink)
  }
  if (props.flexBasis !== undefined) {
    taffy.basis.setSource(index, taffy.dimensionSource(props.flexBasis))
  }
  if (props.alignSelf !== undefined) {
    taffy.alignSelf.setSource(index, enumSource(props.alignSelf, alignSelfToNum))
  }

  // Dimensions
  if (props.width !== undefined) {
    taffy.width.setSource(index, taffy.dimensionSource(props.width))
  }
  if (props.height !== undefined) {
    taffy.height.setSource(index, taffy.dimensionSource(props.height))
  }
  if (props.minWidth !== undefined) {
    taffy.minWidth.setSource(index, taffy.dimensionSource(props.minWidth))
  }
  if (props.maxWidth !== undefined) {
    taffy.maxWidth.setSource(index, taffy.dimensionSource(props.maxWidth))
  }
  if (props.minHeight !== undefined) {
    taffy.minHeight.setSource(index, taffy.dimensionSource(props.minHeight))
  }
  if (props.maxHeight !== undefined) {
    taffy.maxHeight.setSource(index, taffy.dimensionSource(props.maxHeight))
  }

  // Spacing - Margin
  if (props.margin !== undefined) {
    taffy.marginTop.setSource(index, props.marginTop ?? props.margin)
    taffy.marginRight.setSource(index, props.marginRight ?? props.margin)
    taffy.marginBottom.setSource(index, props.marginBottom ?? props.margin)
    taffy.marginLeft.setSource(index, props.marginLeft ?? props.margin)
  } else {
    if (props.marginTop !== undefined) taffy.marginTop.setSource(index, props.marginTop)
    if (props.marginRight !== undefined) taffy.marginRight.setSource(index, props.marginRight)
    if (props.marginBottom !== undefined) taffy.marginBottom.setSource(index, props.marginBottom)
    if (props.marginLeft !== undefined) taffy.marginLeft.setSource(index, props.marginLeft)
  }

  // Spacing - Padding
  if (props.padding !== undefined) {
    taffy.paddingTop.setSource(index, props.paddingTop ?? props.padding)
    taffy.paddingRight.setSource(index, props.paddingRight ?? props.padding)
    taffy.paddingBottom.setSource(index, props.paddingBottom ?? props.padding)
    taffy.paddingLeft.setSource(index, props.paddingLeft ?? props.padding)
  } else {
    if (props.paddingTop !== undefined) taffy.paddingTop.setSource(index, props.paddingTop)
    if (props.paddingRight !== undefined) taffy.paddingRight.setSource(index, props.paddingRight)
    if (props.paddingBottom !== undefined) taffy.paddingBottom.setSource(index, props.paddingBottom)
    if (props.paddingLeft !== undefined) taffy.paddingLeft.setSource(index, props.paddingLeft)
  }

  // Spacing - Gap
  if (props.gap !== undefined) {
    taffy.gap.setSource(index, props.gap)
  }

  // Border widths for layout spacing
  if (props.border !== undefined) {
    const bw = borderToWidth(props.border)
    taffy.borderTop.setSource(index, bw)
    taffy.borderRight.setSource(index, bw)
    taffy.borderBottom.setSource(index, bw)
    taffy.borderLeft.setSource(index, bw)
  }
  if (props.borderTop !== undefined) taffy.borderTop.setSource(index, borderToWidth(props.borderTop))
  if (props.borderRight !== undefined) taffy.borderRight.setSource(index, borderToWidth(props.borderRight))
  if (props.borderBottom !== undefined) taffy.borderBottom.setSource(index, borderToWidth(props.borderBottom))
  if (props.borderLeft !== undefined) taffy.borderLeft.setSource(index, borderToWidth(props.borderLeft))

  // ==========================================================================
  // INTERACTION - Focusable handling
  // ==========================================================================
  const shouldBeFocusable = props.focusable || (props.overflow === 'scroll' && props.focusable !== false)
  if (shouldBeFocusable) {
    interaction.focusable.setSource(index, 1)
    if (props.tabIndex !== undefined) interaction.tabIndex.setSource(index, props.tabIndex)
  }

  // ==========================================================================
  // FOCUS CALLBACKS & KEYBOARD HANDLER
  // ==========================================================================
  let unsubKeyboard: (() => void) | undefined
  let unsubFocusCallbacks: (() => void) | undefined

  if (shouldBeFocusable) {
    if (props.onKey) {
      unsubKeyboard = onFocused(index, props.onKey)
    }
    if (props.onFocus || props.onBlur) {
      unsubFocusCallbacks = registerFocusCallbacks(index, {
        onFocus: props.onFocus,
        onBlur: props.onBlur,
      })
    }
  }

  // ==========================================================================
  // MOUSE HANDLERS
  // ==========================================================================
  let unsubMouse: (() => void) | undefined
  const hasMouseHandlers = props.onMouseDown || props.onMouseUp || props.onClick || props.onMouseEnter || props.onMouseLeave || props.onScroll

  if (shouldBeFocusable || hasMouseHandlers) {
    unsubMouse = onMouseComponent(index, {
      onMouseDown: props.onMouseDown,
      onMouseUp: props.onMouseUp,
      onClick: (event) => {
        if (shouldBeFocusable) {
          focusComponent(index)
        }
        return props.onClick?.(event)
      },
      onMouseEnter: props.onMouseEnter,
      onMouseLeave: props.onMouseLeave,
      onScroll: props.onScroll,
    })
  }

  // ==========================================================================
  // VISUAL - Colors and borders (for frameBuffer rendering)
  // ==========================================================================
  if (props.variant && props.variant !== 'default') {
    const variant = props.variant
    if (props.fg !== undefined) {
      visual.fgColor.setSource(index, props.fg)
    } else {
      visual.fgColor.setSource(index, () => getVariantStyle(variant).fg)
    }
    if (props.bg !== undefined) {
      visual.bgColor.setSource(index, props.bg)
    } else {
      visual.bgColor.setSource(index, () => getVariantStyle(variant).bg)
    }
    if (props.borderColor !== undefined) {
      visual.borderColor.setSource(index, props.borderColor)
    } else {
      visual.borderColor.setSource(index, () => getVariantStyle(variant).border)
    }
  } else {
    if (props.fg !== undefined) visual.fgColor.setSource(index, props.fg)
    if (props.bg !== undefined) visual.bgColor.setSource(index, props.bg)
    if (props.borderColor !== undefined) visual.borderColor.setSource(index, props.borderColor)
  }
  if (props.opacity !== undefined) visual.opacity.setSource(index, props.opacity)

  // Border style for rendering
  if (props.border !== undefined) visual.borderStyle.setSource(index, props.border)
  if (props.borderTop !== undefined) visual.borderTop.setSource(index, props.borderTop)
  if (props.borderRight !== undefined) visual.borderRight.setSource(index, props.borderRight)
  if (props.borderBottom !== undefined) visual.borderBottom.setSource(index, props.borderBottom)
  if (props.borderLeft !== undefined) visual.borderLeft.setSource(index, props.borderLeft)

  // Render children with this box as parent context
  if (props.children) {
    pushParentContext(index)
    try {
      props.children()
    } finally {
      popParentContext()
    }
  }

  // Component setup complete - run lifecycle callbacks
  popCurrentComponent()
  runMountCallbacks(index)

  // Cleanup function
  const cleanup = () => {
    unsubFocusCallbacks?.()
    unsubMouse?.()
    unsubKeyboard?.()
    cleanupKeyboardListeners(index)
    releaseIndex(index)
  }

  // Auto-register with active scope if one exists
  const scope = getActiveScope()
  if (scope) {
    scope.cleanups.push(cleanup)
  }

  return cleanup
}
