/**
 * SparkTUI Mouse State
 *
 * Reactive mouse state exposed as signals.
 * State updates when events arrive from the dispatcher.
 *
 * PURELY REACTIVE: No polling, no intervals.
 * Signals update when the event dispatcher routes mouse events.
 */

import { signal, derived } from '@rlabs-inc/signals'
import type { MouseEvent as SparkMouseEvent, ScrollEvent } from '../engine/events'
import {
  EventType,
  registerMouseHandler,
  registerGlobalMouseHandler,
  registerScrollHandler,
  registerGlobalScrollHandler,
  MOUSE_BUTTON_LEFT,
  MOUSE_BUTTON_MIDDLE,
  MOUSE_BUTTON_RIGHT,
} from '../engine/events'

// Re-export types and constants for convenience
export type { MouseEvent } from '../engine/events'
export type { ScrollEvent } from '../engine/events'
export { MOUSE_BUTTON_LEFT, MOUSE_BUTTON_MIDDLE, MOUSE_BUTTON_RIGHT }

// =============================================================================
// REACTIVE STATE
// =============================================================================

/** Internal signal for last mouse event - updated by event dispatcher */
const lastMouseEventSignal = signal<SparkMouseEvent | null>(null)

/** Internal signal for mouse X position */
const mouseXSignal = signal(0)

/** Internal signal for mouse Y position */
const mouseYSignal = signal(0)

/** Internal signal for mouse button state */
const isMouseDownSignal = signal(false)

/**
 * Last mouse event received.
 * Reactive signal - updates on any mouse action.
 */
export const lastMouseEvent = lastMouseEventSignal

/**
 * Current mouse X position.
 * Reactive signal - updates on mouse move.
 */
export const mouseX = mouseXSignal

/**
 * Current mouse Y position.
 * Reactive signal - updates on mouse move.
 */
export const mouseY = mouseYSignal

/**
 * Whether mouse button is currently pressed.
 * Reactive signal - updates on mouse down/up.
 */
export const isMouseDown = isMouseDownSignal

/**
 * Current mouse position as {x, y} object.
 * Derived from mouseX and mouseY.
 */
export const mousePosition = derived(() => ({
  x: mouseXSignal.value,
  y: mouseYSignal.value,
}))

// =============================================================================
// INTERNAL UPDATE FUNCTION
// =============================================================================

/**
 * Called by event dispatcher when a mouse event arrives.
 * Updates all mouse state signals.
 * @internal
 */
export function _updateMouseState(event: SparkMouseEvent): void {
  lastMouseEventSignal.value = event
  mouseXSignal.value = event.x
  mouseYSignal.value = event.y

  if (event.type === EventType.MouseDown) {
    isMouseDownSignal.value = true
  } else if (event.type === EventType.MouseUp) {
    isMouseDownSignal.value = false
  }
}

// =============================================================================
// PUBLIC API - COMPONENT HANDLERS
// =============================================================================

/**
 * Mouse handlers that can be registered per component.
 */
export interface MouseHandlers {
  onMouseDown?: (event: SparkMouseEvent) => void
  onMouseUp?: (event: SparkMouseEvent) => void
  onClick?: (event: SparkMouseEvent) => void
  onMouseEnter?: (event: SparkMouseEvent) => void
  onMouseLeave?: (event: SparkMouseEvent) => void
  onScroll?: (event: ScrollEvent) => void
}

/**
 * Register mouse handlers for a specific component.
 *
 * @param index - Component index
 * @param handlers - Object with mouse event handlers
 * @returns Unsubscribe function
 *
 * @example
 * ```ts
 * import { onComponent } from './state/mouse'
 *
 * // Inside a component
 * const unsub = onComponent(index, {
 *   onClick: (event) => {
 *     console.log('Clicked at', event.x, event.y)
 *   },
 *   onMouseEnter: () => {
 *     setHovered(true)
 *   },
 *   onMouseLeave: () => {
 *     setHovered(false)
 *   },
 * })
 * ```
 */
export function onComponent(index: number, handlers: MouseHandlers): () => void {
  const unsubscribers: (() => void)[] = []

  // Register each handler for its specific event type
  if (handlers.onMouseDown) {
    unsubscribers.push(
      registerMouseHandler(index, EventType.MouseDown, handlers.onMouseDown)
    )
  }
  if (handlers.onMouseUp) {
    unsubscribers.push(
      registerMouseHandler(index, EventType.MouseUp, handlers.onMouseUp)
    )
  }
  if (handlers.onClick) {
    unsubscribers.push(
      registerMouseHandler(index, EventType.Click, handlers.onClick)
    )
  }
  if (handlers.onMouseEnter) {
    unsubscribers.push(
      registerMouseHandler(index, EventType.MouseEnter, handlers.onMouseEnter)
    )
  }
  if (handlers.onMouseLeave) {
    unsubscribers.push(
      registerMouseHandler(index, EventType.MouseLeave, handlers.onMouseLeave)
    )
  }
  if (handlers.onScroll) {
    unsubscribers.push(
      registerScrollHandler(index, handlers.onScroll)
    )
  }

  // Return combined unsubscribe function
  return () => {
    for (const unsub of unsubscribers) {
      unsub()
    }
  }
}

// =============================================================================
// PUBLIC API - GLOBAL HANDLERS
// =============================================================================

/**
 * Register a global click handler.
 * Called for all click events regardless of target component.
 *
 * @param handler - Called when any click occurs
 * @returns Unsubscribe function
 *
 * @example
 * ```ts
 * import { onGlobalClick } from './state/mouse'
 *
 * // Close dropdown when clicking outside
 * const unsub = onGlobalClick((event) => {
 *   if (!isInsideDropdown(event.x, event.y)) {
 *     closeDropdown()
 *   }
 * })
 * ```
 */
export function onGlobalClick(handler: (event: SparkMouseEvent) => void): () => void {
  return registerGlobalMouseHandler((event) => {
    if (event.type === EventType.Click) {
      handler(event)
    }
  })
}

/**
 * Register a global mouse handler.
 * Receives ALL mouse events (move, down, up, etc.).
 *
 * @param handler - Called on any mouse event
 * @returns Unsubscribe function
 */
export function onGlobalMouse(handler: (event: SparkMouseEvent) => void): () => void {
  return registerGlobalMouseHandler(handler)
}

/**
 * Register a global mouse move handler.
 * Useful for drag operations or cursor tracking.
 *
 * Note: This uses the reactive mousePosition signal internally.
 * For most cases, prefer using `mousePosition` signal directly.
 *
 * @param handler - Called on mouse move
 * @returns Unsubscribe function
 */
export function onGlobalMove(handler: (x: number, y: number) => void): () => void {
  return registerGlobalMouseHandler((event) => {
    if (event.type === EventType.MouseMove) {
      handler(event.x, event.y)
    }
  })
}

/**
 * Register a global scroll handler.
 *
 * @param handler - Called on any scroll event
 * @returns Unsubscribe function
 */
export function onGlobalScroll(handler: (event: ScrollEvent) => void): () => void {
  return registerGlobalScrollHandler(handler)
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/**
 * Check if a point is within a bounding box.
 * Useful for hit testing in custom components.
 */
export function isPointInBounds(
  x: number,
  y: number,
  bounds: { x: number; y: number; width: number; height: number }
): boolean {
  return (
    x >= bounds.x &&
    x < bounds.x + bounds.width &&
    y >= bounds.y &&
    y < bounds.y + bounds.height
  )
}

/**
 * Get the mouse button name from event.
 */
export function getButtonName(event: SparkMouseEvent): 'left' | 'middle' | 'right' {
  switch (event.button) {
    case MOUSE_BUTTON_LEFT: return 'left'
    case MOUSE_BUTTON_MIDDLE: return 'middle'
    case MOUSE_BUTTON_RIGHT: return 'right'
    default: return 'left'
  }
}

/**
 * Check if the event was a left click.
 */
export function isLeftButton(event: SparkMouseEvent): boolean {
  return event.button === MOUSE_BUTTON_LEFT
}

/**
 * Check if the event was a middle click.
 */
export function isMiddleButton(event: SparkMouseEvent): boolean {
  return event.button === MOUSE_BUTTON_MIDDLE
}

/**
 * Check if the event was a right click.
 */
export function isRightButton(event: SparkMouseEvent): boolean {
  return event.button === MOUSE_BUTTON_RIGHT
}
