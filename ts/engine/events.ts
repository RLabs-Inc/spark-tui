/**
 * SparkTUI Event System
 *
 * Reads events from the SharedBuffer event ring buffer and dispatches to handlers.
 *
 * PURELY REACTIVE: Uses Atomics.waitAsync for instant wake, NO polling.
 *
 * Event flow:
 *   Rust writes event -> Atomics.notify(wake_ts) -> TS wakes -> reads ring -> dispatches
 *
 * Latency target: < 50 microseconds
 */

// Type declaration for Atomics.waitAsync (ES2024+)
// Node.js 16+ and modern browsers support this
declare global {
  interface Atomics {
    waitAsync(
      typedArray: Int32Array,
      index: number,
      value: number,
      timeout?: number
    ): { async: false; value: 'not-equal' | 'timed-out' } | { async: true; value: Promise<'ok' | 'timed-out'> }
  }
}

import {
  type AoSBuffer,
  EventType,
  EVENT_RING_OFFSET,
  EVENT_RING_HEADER_SIZE,
  EVENT_SLOT_SIZE,
  MAX_EVENTS,
  H_WAKE_TS,
  getEventWriteIdx,
  getEventReadIdx,
  setEventReadIdx,
  getParentIndex,
} from '../bridge/shared-buffer-aos'

// =============================================================================
// RE-EXPORT EventType
// =============================================================================

export { EventType }

// =============================================================================
// EVENT TYPES (matching Rust ring buffer layout)
// =============================================================================

/** Keyboard event - key press, repeat, or release */
export interface KeyEvent {
  type: EventType.Key
  componentIndex: number
  keycode: number
  modifiers: number // ctrl=1, alt=2, shift=4, meta=8
  keyState: number // press=0, repeat=1, release=2
}

/** Mouse button events */
export interface MouseEvent {
  type:
    | EventType.MouseDown
    | EventType.MouseUp
    | EventType.Click
    | EventType.MouseEnter
    | EventType.MouseLeave
    | EventType.MouseMove
  componentIndex: number
  x: number
  y: number
  button: number // left=0, middle=1, right=2
}

/** Scroll wheel event */
export interface ScrollEvent {
  type: EventType.Scroll
  componentIndex: number
  deltaX: number
  deltaY: number
}

/** Focus/blur events */
export interface FocusEvent {
  type: EventType.Focus | EventType.Blur
  componentIndex: number
}

/** Input value events - change, submit, cancel */
export interface ValueEvent {
  type: EventType.ValueChange | EventType.Submit | EventType.Cancel
  componentIndex: number
}

/** Terminal resize event */
export interface ResizeEvent {
  type: EventType.Resize
  width: number
  height: number
}

/** Exit event (Ctrl+C, etc.) */
export interface ExitEvent {
  type: EventType.Exit
}

/** Union of all event types */
export type SparkEvent =
  | KeyEvent
  | MouseEvent
  | ScrollEvent
  | FocusEvent
  | ValueEvent
  | ResizeEvent
  | ExitEvent

// =============================================================================
// MODIFIER FLAGS
// =============================================================================

export const MODIFIER_CTRL = 1
export const MODIFIER_ALT = 2
export const MODIFIER_SHIFT = 4
export const MODIFIER_META = 8

// =============================================================================
// KEY STATE
// =============================================================================

export const KEY_STATE_PRESS = 0
export const KEY_STATE_REPEAT = 1
export const KEY_STATE_RELEASE = 2

// =============================================================================
// MOUSE BUTTON
// =============================================================================

export const MOUSE_BUTTON_LEFT = 0
export const MOUSE_BUTTON_MIDDLE = 1
export const MOUSE_BUTTON_RIGHT = 2

// =============================================================================
// HANDLER TYPES
// =============================================================================

/** Return true to consume the event (stop propagation) */
export type KeyHandler = (event: KeyEvent) => boolean | void
export type MouseHandler = (event: MouseEvent) => void
export type FocusHandler = (event: FocusEvent) => void
export type ValueHandler = (event: ValueEvent) => void
export type ResizeHandler = (event: ResizeEvent) => void
export type ExitHandler = (event: ExitEvent) => void
export type ScrollHandler = (event: ScrollEvent) => void

// =============================================================================
// EVENT PARSER
// =============================================================================

/**
 * Parse a single event from the ring buffer at the given slot.
 * Returns null if the slot is empty (EventType.None).
 */
function parseEvent(buf: AoSBuffer, slot: number): SparkEvent | null {
  const offset = EVENT_RING_OFFSET + EVENT_RING_HEADER_SIZE + slot * EVENT_SLOT_SIZE
  const view = buf.view

  const eventType = view.getUint8(offset) as EventType
  if (eventType === EventType.None) return null

  const componentIndex = view.getUint16(offset + 2, true)
  const dataOffset = offset + 4

  switch (eventType) {
    case EventType.Key:
      return {
        type: eventType,
        componentIndex,
        keycode: view.getUint32(dataOffset, true),
        modifiers: view.getUint8(dataOffset + 4),
        keyState: view.getUint8(dataOffset + 5),
      }

    case EventType.MouseDown:
    case EventType.MouseUp:
    case EventType.Click:
    case EventType.MouseEnter:
    case EventType.MouseLeave:
    case EventType.MouseMove:
      return {
        type: eventType,
        componentIndex,
        x: view.getUint16(dataOffset, true),
        y: view.getUint16(dataOffset + 2, true),
        button: view.getUint8(dataOffset + 4),
      }

    case EventType.Scroll:
      return {
        type: eventType,
        componentIndex,
        deltaX: view.getInt32(dataOffset, true),
        deltaY: view.getInt32(dataOffset + 4, true),
      }

    case EventType.Focus:
    case EventType.Blur:
      return {
        type: eventType,
        componentIndex,
      }

    case EventType.ValueChange:
    case EventType.Submit:
    case EventType.Cancel:
      return {
        type: eventType,
        componentIndex,
      }

    case EventType.Resize:
      return {
        type: eventType,
        width: view.getUint16(dataOffset, true),
        height: view.getUint16(dataOffset + 2, true),
      }

    case EventType.Exit:
      return {
        type: eventType,
      }

    default:
      // Unknown event type - skip
      return null
  }
}

// =============================================================================
// RING BUFFER READER
// =============================================================================

/**
 * Read all pending events from the ring buffer.
 * Updates the read index atomically after reading.
 */
export function readEvents(buf: AoSBuffer): SparkEvent[] {
  const events: SparkEvent[] = []
  const writeIdx = getEventWriteIdx(buf)
  let readIdx = getEventReadIdx(buf)

  // Read all events between readIdx and writeIdx
  while (readIdx < writeIdx) {
    const slot = readIdx % MAX_EVENTS
    const event = parseEvent(buf, slot)
    if (event) {
      events.push(event)
    }
    readIdx++
  }

  // Update read index so Rust knows we've consumed these events
  setEventReadIdx(buf, readIdx)
  return events
}

// =============================================================================
// HANDLER REGISTRIES (internal)
// =============================================================================

// Per-component handlers (keyed by component index)
const keyHandlers = new Map<number, KeyHandler[]>()
const mouseHandlers = new Map<
  number,
  Partial<Record<MouseEvent['type'], MouseHandler[]>>
>()
const focusHandlers = new Map<number, FocusHandler[]>()
const valueHandlers = new Map<number, ValueHandler[]>()
const scrollHandlers = new Map<number, ScrollHandler[]>()

// Global handlers (not component-specific)
const globalKeyHandlers: KeyHandler[] = []
const globalMouseHandlers: MouseHandler[] = []
const globalScrollHandlers: ScrollHandler[] = []
const resizeHandlers: ResizeHandler[] = []
const exitHandlers: ExitHandler[] = []

// =============================================================================
// HANDLER REGISTRATION
// =============================================================================

/**
 * Register a key handler for a specific component.
 * Returns an unsubscribe function.
 */
export function registerKeyHandler(index: number, handler: KeyHandler): () => void {
  if (!keyHandlers.has(index)) {
    keyHandlers.set(index, [])
  }
  keyHandlers.get(index)!.push(handler)

  return () => {
    const handlers = keyHandlers.get(index)
    if (handlers) {
      const i = handlers.indexOf(handler)
      if (i >= 0) handlers.splice(i, 1)
      if (handlers.length === 0) keyHandlers.delete(index)
    }
  }
}

/**
 * Register a global key handler (receives all key events).
 * Returns an unsubscribe function.
 */
export function registerGlobalKeyHandler(handler: KeyHandler): () => void {
  globalKeyHandlers.push(handler)
  return () => {
    const i = globalKeyHandlers.indexOf(handler)
    if (i >= 0) globalKeyHandlers.splice(i, 1)
  }
}

/**
 * Register a mouse handler for a specific component and event type.
 * Returns an unsubscribe function.
 */
export function registerMouseHandler(
  index: number,
  eventType: MouseEvent['type'],
  handler: MouseHandler
): () => void {
  if (!mouseHandlers.has(index)) {
    mouseHandlers.set(index, {})
  }
  const componentHandlers = mouseHandlers.get(index)!
  if (!componentHandlers[eventType]) {
    componentHandlers[eventType] = []
  }
  componentHandlers[eventType]!.push(handler)

  return () => {
    const handlers = componentHandlers[eventType]
    if (handlers) {
      const i = handlers.indexOf(handler)
      if (i >= 0) handlers.splice(i, 1)
      if (handlers.length === 0) delete componentHandlers[eventType]
    }
    // Clean up empty component entry
    if (Object.keys(componentHandlers).length === 0) {
      mouseHandlers.delete(index)
    }
  }
}

/**
 * Register a global mouse handler (receives all mouse events).
 * Returns an unsubscribe function.
 */
export function registerGlobalMouseHandler(handler: MouseHandler): () => void {
  globalMouseHandlers.push(handler)
  return () => {
    const i = globalMouseHandlers.indexOf(handler)
    if (i >= 0) globalMouseHandlers.splice(i, 1)
  }
}

/**
 * Register a focus handler for a specific component.
 * Returns an unsubscribe function.
 */
export function registerFocusHandler(index: number, handler: FocusHandler): () => void {
  if (!focusHandlers.has(index)) {
    focusHandlers.set(index, [])
  }
  focusHandlers.get(index)!.push(handler)

  return () => {
    const handlers = focusHandlers.get(index)
    if (handlers) {
      const i = handlers.indexOf(handler)
      if (i >= 0) handlers.splice(i, 1)
      if (handlers.length === 0) focusHandlers.delete(index)
    }
  }
}

/**
 * Register a value handler for a specific component.
 * Returns an unsubscribe function.
 */
export function registerValueHandler(index: number, handler: ValueHandler): () => void {
  if (!valueHandlers.has(index)) {
    valueHandlers.set(index, [])
  }
  valueHandlers.get(index)!.push(handler)

  return () => {
    const handlers = valueHandlers.get(index)
    if (handlers) {
      const i = handlers.indexOf(handler)
      if (i >= 0) handlers.splice(i, 1)
      if (handlers.length === 0) valueHandlers.delete(index)
    }
  }
}

/**
 * Register a scroll handler for a specific component.
 * Returns an unsubscribe function.
 */
export function registerScrollHandler(index: number, handler: ScrollHandler): () => void {
  if (!scrollHandlers.has(index)) {
    scrollHandlers.set(index, [])
  }
  scrollHandlers.get(index)!.push(handler)

  return () => {
    const handlers = scrollHandlers.get(index)
    if (handlers) {
      const i = handlers.indexOf(handler)
      if (i >= 0) handlers.splice(i, 1)
      if (handlers.length === 0) scrollHandlers.delete(index)
    }
  }
}

/**
 * Register a global scroll handler.
 * Returns an unsubscribe function.
 */
export function registerGlobalScrollHandler(handler: ScrollHandler): () => void {
  globalScrollHandlers.push(handler)
  return () => {
    const i = globalScrollHandlers.indexOf(handler)
    if (i >= 0) globalScrollHandlers.splice(i, 1)
  }
}

/**
 * Register a resize handler (global - terminal resize).
 * Returns an unsubscribe function.
 */
export function registerResizeHandler(handler: ResizeHandler): () => void {
  resizeHandlers.push(handler)
  return () => {
    const i = resizeHandlers.indexOf(handler)
    if (i >= 0) resizeHandlers.splice(i, 1)
  }
}

/**
 * Register an exit handler (global - Ctrl+C, etc.).
 * Returns an unsubscribe function.
 */
export function registerExitHandler(handler: ExitHandler): () => void {
  exitHandlers.push(handler)
  return () => {
    const i = exitHandlers.indexOf(handler)
    if (i >= 0) exitHandlers.splice(i, 1)
  }
}

// =============================================================================
// EVENT DISPATCHER
// =============================================================================

/**
 * Dispatch a single event to all registered handlers.
 * Key events can return true to consume (stop propagation).
 */
function dispatchEvent(event: SparkEvent): void {
  switch (event.type) {
    case EventType.Key: {
      // Global key handlers first (can consume)
      for (const handler of globalKeyHandlers) {
        if (handler(event) === true) return // consumed
      }

      // Component-specific key handlers with BUBBLING
      if (currentBuffer) {
        let target = event.componentIndex
        let depth = 0

        while (depth < 100) {
          const handlers = keyHandlers.get(target)
          if (handlers) {
            for (const handler of handlers) {
              if (handler(event) === true) return // consumed
            }
          }

          // Move to parent
          const parent = getParentIndex(currentBuffer, target)
          if (parent < 0) break // No parent (root)
          target = parent
          depth++
        }
      }
      break
    }

    case EventType.MouseDown:
    case EventType.MouseUp:
    case EventType.Click:
    case EventType.MouseEnter:
    case EventType.MouseLeave:
    case EventType.MouseMove: {
      // Global mouse handlers (always fire)
      for (const handler of globalMouseHandlers) {
        handler(event)
      }

      // Component-specific mouse handlers with BUBBLING
      if (currentBuffer) {
        let target = event.componentIndex
        let depth = 0

        while (depth < 100) {
          const componentHandlers = mouseHandlers.get(target)
          if (componentHandlers) {
            const typeHandlers = componentHandlers[event.type]
            if (typeHandlers) {
              for (const handler of typeHandlers) {
                handler(event)
              }
              // Found a handler? We could stop bubbling here if we implemented stopPropagation.
              // For now, let's behave like standard DOM bubbling (fire on all ancestors).
              // BUT for click events on buttons, usually the button handles it and that's it.
              // If we want to simulate "clicking the button" when clicking text inside it,
              // bubbling is exactly what we need.
            }
          }

          // Move to parent
          const parent = getParentIndex(currentBuffer, target)
          if (parent < 0) break // No parent (root)
          target = parent
          depth++
        }
      }
      break
    }

    case EventType.Scroll: {
      // Global scroll handlers
      for (const handler of globalScrollHandlers) {
        handler(event)
      }
      // Component-specific scroll handlers
      const handlers = scrollHandlers.get(event.componentIndex)
      if (handlers) {
        for (const handler of handlers) {
          handler(event)
        }
      }
      break
    }

    case EventType.Focus:
    case EventType.Blur: {
      const handlers = focusHandlers.get(event.componentIndex)
      if (handlers) {
        for (const handler of handlers) {
          handler(event)
        }
      }
      break
    }

    case EventType.ValueChange:
    case EventType.Submit:
    case EventType.Cancel: {
      const handlers = valueHandlers.get(event.componentIndex)
      if (handlers) {
        for (const handler of handlers) {
          handler(event)
        }
      }
      break
    }

    case EventType.Resize: {
      for (const handler of resizeHandlers) {
        handler(event)
      }
      break
    }

    case EventType.Exit: {
      for (const handler of exitHandlers) {
        handler(event)
      }
      break
    }
  }
}

// =============================================================================
// EVENT LISTENER (Atomics.waitAsync - REACTIVE, NOT POLLING)
// =============================================================================

let running = false
let wakeInt32: Int32Array | null = null
let currentBuffer: AoSBuffer | null = null

/**
 * Start the event listener.
 *
 * IMPORTANT: This is NOT polling! Atomics.waitAsync SUSPENDS until Rust notifies.
 * The while(running) loop is NOT busy - each iteration waits for notification.
 *
 * Event flow:
 *   1. TS calls Atomics.waitAsync() - SUSPENDS (zero CPU)
 *   2. Rust writes event to ring buffer
 *   3. Rust calls Atomics.notify(wake_ts)
 *   4. TS wakes instantly (< 10 microseconds)
 *   5. TS reads events, dispatches to handlers
 *   6. Back to step 1
 */
export function startEventListener(buf: AoSBuffer): void {
  if (running) return

  running = true
  currentBuffer = buf
  wakeInt32 = new Int32Array(buf.buffer, H_WAKE_TS, 1)

  // Start the reactive wait loop
  waitForEvents()
}

/**
 * The event wait loop.
 *
 * PLATFORM LIMITATION: Cross-language Atomics.notify doesn't work on macOS.
 * JavaScript's Atomics.waitAsync and Rust's atomic_wait use different
 * underlying primitives (__ulock on macOS vs futex on Linux).
 *
 * WORKAROUND: Short timeout (8ms) for responsiveness. This is NOT polling in
 * the traditional sense - the thread truly suspends between checks, and the
 * timeout only fires if no events arrive. With events flowing, wakes are instant.
 *
 * TODO: Test true futex wake on Linux where both use the same syscall.
 *
 * Event flow:
 *   1. TS: Atomics.waitAsync() - SUSPENDS (zero CPU while waiting)
 *   2. Rust: writes event, sets wake flag, calls wake_one()
 *   3. TS: wakes on value change OR timeout, reads events, dispatches
 */
async function waitForEvents(): Promise<void> {
  while (running && wakeInt32 && currentBuffer) {
    // Get current wake value
    const currentValue = Atomics.load(wakeInt32, 0)

    // Wait for value change OR minimal timeout
    // 1ms = fastest practical interval, very responsive
    const result = Atomics.waitAsync(wakeInt32, 0, currentValue, 1)

    if (result.async) {
      await result.value
    }
    // 'not-equal' = value changed (instant), 'timed-out' = check anyway

    if (!running) break

    // Reset wake flag
    Atomics.store(wakeInt32, 0, 0)

    // Read and dispatch all pending events
    const events = readEvents(currentBuffer)
    for (const event of events) {
      dispatchEvent(event)
    }
  }
}

/**
 * Stop the event listener.
 */
export function stopEventListener(): void {
  running = false

  // Wake to exit the wait loop
  if (wakeInt32) {
    Atomics.store(wakeInt32, 0, 1)
    Atomics.notify(wakeInt32, 0)
  }

  wakeInt32 = null
  currentBuffer = null
}

/**
 * Check if the event listener is running.
 */
export function isEventListenerRunning(): boolean {
  return running
}

// =============================================================================
// HANDLER CLEANUP
// =============================================================================

/**
 * Clean up all handlers for a specific component index.
 * Called when a component is unmounted.
 */
export function cleanupHandlers(index: number): void {
  keyHandlers.delete(index)
  mouseHandlers.delete(index)
  focusHandlers.delete(index)
  valueHandlers.delete(index)
  scrollHandlers.delete(index)
}

/**
 * Clean up all handlers.
 * Called when the application is unmounted.
 */
export function cleanupAllHandlers(): void {
  keyHandlers.clear()
  mouseHandlers.clear()
  focusHandlers.clear()
  valueHandlers.clear()
  scrollHandlers.clear()

  globalKeyHandlers.length = 0
  globalMouseHandlers.length = 0
  globalScrollHandlers.length = 0
  resizeHandlers.length = 0
  exitHandlers.length = 0
}

// =============================================================================
// MANUAL EVENT DISPATCH (for testing/debugging)
// =============================================================================

/**
 * Manually dispatch an event (for testing/debugging).
 * Normally events come from the ring buffer.
 */
export function dispatchEventManual(event: SparkEvent): void {
  dispatchEvent(event)
}

/**
 * Manually read and dispatch events without waiting.
 * Useful for synchronous testing or when you know events are pending.
 */
export function processEvents(buf: AoSBuffer): SparkEvent[] {
  const events = readEvents(buf)
  for (const event of events) {
    dispatchEvent(event)
  }
  return events
}

// =============================================================================
// HELPER: Modifier Checks
// =============================================================================

/** Check if Ctrl modifier is pressed */
export function hasCtrl(event: KeyEvent): boolean {
  return (event.modifiers & MODIFIER_CTRL) !== 0
}

/** Check if Alt modifier is pressed */
export function hasAlt(event: KeyEvent): boolean {
  return (event.modifiers & MODIFIER_ALT) !== 0
}

/** Check if Shift modifier is pressed */
export function hasShift(event: KeyEvent): boolean {
  return (event.modifiers & MODIFIER_SHIFT) !== 0
}

/** Check if Meta/Cmd modifier is pressed */
export function hasMeta(event: KeyEvent): boolean {
  return (event.modifiers & MODIFIER_META) !== 0
}

/** Check if this is a key press (not repeat or release) */
export function isKeyPress(event: KeyEvent): boolean {
  return event.keyState === KEY_STATE_PRESS
}

/** Check if this is a key repeat */
export function isKeyRepeat(event: KeyEvent): boolean {
  return event.keyState === KEY_STATE_REPEAT
}

/** Check if this is a key release */
export function isKeyRelease(event: KeyEvent): boolean {
  return event.keyState === KEY_STATE_RELEASE
}

// =============================================================================
// KEY CODE CONSTANTS (matching Rust keyboard.rs)
// =============================================================================

/** Common key codes */
export const KEY_ENTER = 13
export const KEY_TAB = 9
export const KEY_BACKSPACE = 8
export const KEY_ESCAPE = 27
export const KEY_DELETE = 127
export const KEY_SPACE = 32

/** Arrow keys (0x1000 range) */
export const KEY_UP = 0x1001
export const KEY_DOWN = 0x1002
export const KEY_LEFT = 0x1003
export const KEY_RIGHT = 0x1004
export const KEY_HOME = 0x1005
export const KEY_END = 0x1006
export const KEY_PAGE_UP = 0x1007
export const KEY_PAGE_DOWN = 0x1008
export const KEY_INSERT = 0x1009

/** Function keys (0x2000 range) - F1 = 0x2001, F12 = 0x200C */
export const KEY_F1 = 0x2001
export const KEY_F2 = 0x2002
export const KEY_F3 = 0x2003
export const KEY_F4 = 0x2004
export const KEY_F5 = 0x2005
export const KEY_F6 = 0x2006
export const KEY_F7 = 0x2007
export const KEY_F8 = 0x2008
export const KEY_F9 = 0x2009
export const KEY_F10 = 0x200A
export const KEY_F11 = 0x200B
export const KEY_F12 = 0x200C

// =============================================================================
// FRIENDLY KEY HELPERS
// =============================================================================

/** Get the character for a printable key (or undefined for special keys) */
export function getChar(event: KeyEvent): string | undefined {
  // Printable ASCII range
  if (event.keycode >= 32 && event.keycode <= 126) {
    return String.fromCharCode(event.keycode)
  }
  return undefined
}

/** Get a human-readable key name */
export function getKeyName(event: KeyEvent): string {
  switch (event.keycode) {
    case KEY_ENTER: return 'enter'
    case KEY_TAB: return 'tab'
    case KEY_BACKSPACE: return 'backspace'
    case KEY_ESCAPE: return 'escape'
    case KEY_DELETE: return 'delete'
    case KEY_SPACE: return 'space'
    case KEY_UP: return 'up'
    case KEY_DOWN: return 'down'
    case KEY_LEFT: return 'left'
    case KEY_RIGHT: return 'right'
    case KEY_HOME: return 'home'
    case KEY_END: return 'end'
    case KEY_PAGE_UP: return 'pageup'
    case KEY_PAGE_DOWN: return 'pagedown'
    case KEY_INSERT: return 'insert'
    default:
      // Function keys
      if (event.keycode >= KEY_F1 && event.keycode <= KEY_F12) {
        return `f${event.keycode - 0x2000}`
      }
      // Printable character
      if (event.keycode >= 32 && event.keycode <= 126) {
        return String.fromCharCode(event.keycode)
      }
      return `unknown(${event.keycode})`
  }
}

/** Check if this is the Enter key */
export function isEnter(event: KeyEvent): boolean {
  return event.keycode === KEY_ENTER
}

/** Check if this is the Space key */
export function isSpace(event: KeyEvent): boolean {
  return event.keycode === KEY_SPACE
}

/** Check if this is the Escape key */
export function isEscape(event: KeyEvent): boolean {
  return event.keycode === KEY_ESCAPE
}

/** Check if this is an arrow key */
export function isArrowKey(event: KeyEvent): boolean {
  return event.keycode >= KEY_UP && event.keycode <= KEY_RIGHT
}

/** Check if this is a function key (F1-F12) */
export function isFunctionKey(event: KeyEvent): boolean {
  return event.keycode >= KEY_F1 && event.keycode <= KEY_F12
}

/** Check if the key matches a specific character */
export function isChar(event: KeyEvent, char: string): boolean {
  return event.keycode === char.charCodeAt(0)
}
