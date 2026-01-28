/**
 * TUI Framework - Text Primitive
 *
 * Display text with styling, alignment, and wrapping.
 *
 * REACTIVITY: Props are passed directly to setSource() to preserve reactive links.
 * Don't extract values before binding - that breaks the connection!
 *
 * Usage:
 * ```ts
 * // Static text
 * text({ content: 'Hello, World!' })
 *
 * // Reactive text - pass the signal directly!
 * const message = signal('Hello')
 * text({ content: message })
 * message.value = 'Updated!'  // UI reacts automatically!
 *
 * // Reactive with derived
 * const count = signal(0)
 * const countText = derived(() => `Count: ${count.value}`)
 * text({ content: countText })
 * ```
 */

import { derived } from '@rlabs-inc/signals'
import { ComponentType } from '../types'
import { allocateIndex, releaseIndex, getCurrentParentIndex } from '../engine/registry'
import { measureTextWidth } from '../pipeline/layout/utils/text-measure'
import {
  pushCurrentComponent,
  popCurrentComponent,
  runMountCallbacks,
} from '../engine/lifecycle'
import { cleanupIndex as cleanupKeyboardListeners } from '../state/keyboard'
import { onComponent as onMouseComponent } from '../state/mouse'
import { getVariantStyle } from '../state/theme'
import { getActiveScope } from './scope'
import { enumSource } from './utils'

// Import arrays
import * as core from '../engine/arrays/core'
import * as taffy from '../engine/arrays/taffy'
import * as visual from '../engine/arrays/visual'
import * as textArrays from '../engine/arrays/text'

// Import types
import type { TextProps, Cleanup } from './types'

// =============================================================================
// HELPERS - Enum conversions
// =============================================================================

/** Convert align string to number */
function alignToNum(align: string | undefined): number {
  switch (align) {
    case 'center': return 1
    case 'right': return 2
    default: return 0 // left
  }
}

/** Convert wrap string to number */
function wrapToNum(wrap: string | undefined): number {
  switch (wrap) {
    case 'nowrap': return 0
    case 'truncate': return 2
    default: return 1 // wrap
  }
}

/** Convert align-self string to number (0 = auto) */
function alignSelfToNum(alignSelf: string | undefined): number {
  switch (alignSelf) {
    case 'stretch': return taffy.ALIGN_SELF_STRETCH
    case 'flex-start': return taffy.ALIGN_SELF_FLEX_START
    case 'center': return taffy.ALIGN_SELF_CENTER
    case 'flex-end': return taffy.ALIGN_SELF_FLEX_END
    case 'baseline': return taffy.ALIGN_SELF_BASELINE
    default: return taffy.ALIGN_SELF_AUTO
  }
}

/**
 * Convert content prop (string | number) to string source for setSource.
 * Handles: static values, signals, and getters.
 */
function contentToStringSource(
  content: TextProps['content']
): string | (() => string) {
  // Getter function - wrap to convert
  if (typeof content === 'function') {
    return () => String(content())
  }
  // Signal/binding/derived with .value
  if (content !== null && typeof content === 'object' && 'value' in content) {
    const reactive = content as { value: string | number }
    return () => String(reactive.value)
  }
  // Static value
  return String(content)
}

// =============================================================================
// TEXT COMPONENT
// =============================================================================

/**
 * Create a text display component.
 *
 * Pass signals directly for reactive content - they stay connected!
 * The pipeline reads via unwrap() which tracks dependencies.
 *
 * Supports all theme variants:
 * - Core: default, primary, secondary, tertiary, accent
 * - Status: success, warning, error, info
 * - Surface: muted, surface, elevated, ghost, outline
 */
export function text(props: TextProps): Cleanup {
  const index = allocateIndex(props.id)

  // Track current component for lifecycle hooks
  pushCurrentComponent(index)

  // ==========================================================================
  // CORE - Always needed
  // ==========================================================================
  core.componentType[index] = ComponentType.TEXT
  core.parentIndex.setSource(index, getCurrentParentIndex())

  // Visibility
  if (props.visible !== undefined) {
    taffy.visible.setSource(index, props.visible)
    core.visible.setSource(index, props.visible)
  }

  // ==========================================================================
  // TAFFY LAYOUT ARRAYS - Direct binding to TypedArrays
  // ==========================================================================

  // Parent hierarchy (for tree structure)
  taffy.parentIndex.setSource(index, getCurrentParentIndex())

  // Flex item properties (text is always a flex item, never a container)
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

  // Dimensions - auto-calculate from content if not provided
  // Convert content source to a getter we can use for measurement
  const contentSource = contentToStringSource(props.content)
  const getContent = typeof contentSource === 'function' ? contentSource : () => contentSource

  if (props.width !== undefined) {
    taffy.width.setSource(index, taffy.dimensionSource(props.width))
  } else {
    // Auto-width: measure content width (reactive - updates when content changes)
    taffy.width.setSource(index, () => measureTextWidth(getContent()))
  }

  if (props.height !== undefined) {
    taffy.height.setSource(index, taffy.dimensionSource(props.height))
  } else {
    // Auto-height: count lines in content (newlines)
    taffy.height.setSource(index, () => {
      const content = getContent()
      return content.split('\n').length
    })
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

  // ==========================================================================
  // TEXT CONTENT - Always needed (this is a text component!)
  // ==========================================================================
  textArrays.textContent.setSource(index, contentToStringSource(props.content))

  // Text styling
  if (props.attrs !== undefined) textArrays.textAttrs.setSource(index, props.attrs)
  if (props.align !== undefined) textArrays.textAlign.setSource(index, enumSource(props.align, alignToNum))
  if (props.wrap !== undefined) textArrays.textWrap.setSource(index, enumSource(props.wrap, wrapToNum))

  // ==========================================================================
  // VISUAL - Colors with variant support
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
  } else {
    if (props.fg !== undefined) visual.fgColor.setSource(index, props.fg)
    if (props.bg !== undefined) visual.bgColor.setSource(index, props.bg)
  }
  if (props.opacity !== undefined) visual.opacity.setSource(index, props.opacity)

  // ==========================================================================
  // MOUSE HANDLERS
  // ==========================================================================
  let unsubMouse: (() => void) | undefined

  if (props.onMouseDown || props.onMouseUp || props.onClick || props.onMouseEnter || props.onMouseLeave || props.onScroll) {
    unsubMouse = onMouseComponent(index, {
      onMouseDown: props.onMouseDown,
      onMouseUp: props.onMouseUp,
      onClick: props.onClick,
      onMouseEnter: props.onMouseEnter,
      onMouseLeave: props.onMouseLeave,
      onScroll: props.onScroll,
    })
  }

  // Component setup complete - run lifecycle callbacks
  popCurrentComponent()
  runMountCallbacks(index)

  // Cleanup function
  const cleanup = () => {
    unsubMouse?.()
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
