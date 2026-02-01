/**
 * SparkTUI Event System (v3 Buffer)
 *
 * Reads events from the SharedBuffer event ring and dispatches to handlers.
 *
 * PURELY REACTIVE: Uses Atomics.waitAsync for instant wake, NO polling.
 *
 * Event flow:
 *   Rust writes event -> Atomics.notify(wake_ts) -> TS wakes -> reads ring -> dispatches
 */

import {
  type SharedBuffer,
  H_WAKE_TS,
  H_EVENT_WRITE_IDX,
  H_EVENT_READ_IDX,
  EVENT_RING_HEADER_SIZE,
  EVENT_SLOT_SIZE,
  MAX_EVENTS,
  getParentIndex,
} from '../bridge/shared-buffer'

// =============================================================================
// EVENT TYPES
// =============================================================================

/** Event type enum - must match Rust */
export const enum EventType {
  None = 0,
  Key = 1,
  MouseDown = 2,
  MouseUp = 3,
  Click = 4,
  MouseEnter = 5,
  MouseLeave = 6,
  MouseMove = 7,
  Scroll = 8,
  Focus = 9,
  Blur = 10,
  ValueChange = 11,
  Submit = 12,
  Cancel = 13,
  Exit = 14,
  Resize = 15,
}

/** Keyboard event */
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

/** Input value events */
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
// EVENT RING READER
// =============================================================================

/** Get event write index from header */
function getEventWriteIdx(buf: SharedBuffer): number {
  return buf.view.getUint32(H_EVENT_WRITE_IDX, true)
}

/** Get event read index from header */
function getEventReadIdx(buf: SharedBuffer): number {
  return buf.view.getUint32(H_EVENT_READ_IDX, true)
}

/** Set event read index in header */
function setEventReadIdx(buf: SharedBuffer, idx: number): void {
  buf.view.setUint32(H_EVENT_READ_IDX, idx, true)
}

/**
 * Parse a single event from the ring buffer at the given slot.
 */
function parseEvent(buf: SharedBuffer, slot: number): SparkEvent | null {
  const offset = buf.eventRingOffset + EVENT_RING_HEADER_SIZE + slot * EVENT_SLOT_SIZE
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
      return { type: eventType }

    default:
      return null
  }
}

/**
 * Read all pending events from the ring buffer.
 */
export function readEvents(buf: SharedBuffer): SparkEvent[] {
  const events: SparkEvent[] = []
  const writeIdx = getEventWriteIdx(buf)
  let readIdx = getEventReadIdx(buf)

  while (readIdx < writeIdx) {
    const slot = readIdx % MAX_EVENTS
    const event = parseEvent(buf, slot)
    if (event) events.push(event)
    readIdx++
  }

  setEventReadIdx(buf, readIdx)
  return events
}

// =============================================================================
// HANDLER REGISTRIES
// =============================================================================

const keyHandlers = new Map<number, KeyHandler[]>()
const mouseHandlers = new Map<number, Partial<Record<MouseEvent['type'], MouseHandler[]>>>()
const focusHandlers = new Map<number, FocusHandler[]>()
const valueHandlers = new Map<number, ValueHandler[]>()
const scrollHandlers = new Map<number, ScrollHandler[]>()

const globalKeyHandlers: KeyHandler[] = []
const globalMouseHandlers: MouseHandler[] = []
const globalScrollHandlers: ScrollHandler[] = []
const resizeHandlers: ResizeHandler[] = []
const exitHandlers: ExitHandler[] = []

// =============================================================================
// HANDLER REGISTRATION
// =============================================================================

export function registerKeyHandler(index: number, handler: KeyHandler): () => void {
  if (!keyHandlers.has(index)) keyHandlers.set(index, [])
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

export function registerGlobalKeyHandler(handler: KeyHandler): () => void {
  globalKeyHandlers.push(handler)
  return () => {
    const i = globalKeyHandlers.indexOf(handler)
    if (i >= 0) globalKeyHandlers.splice(i, 1)
  }
}

export function registerMouseHandler(
  index: number,
  eventType: MouseEvent['type'],
  handler: MouseHandler
): () => void {
  if (!mouseHandlers.has(index)) mouseHandlers.set(index, {})
  const componentHandlers = mouseHandlers.get(index)!
  if (!componentHandlers[eventType]) componentHandlers[eventType] = []
  componentHandlers[eventType]!.push(handler)

  return () => {
    const handlers = componentHandlers[eventType]
    if (handlers) {
      const i = handlers.indexOf(handler)
      if (i >= 0) handlers.splice(i, 1)
      if (handlers.length === 0) delete componentHandlers[eventType]
    }
    if (Object.keys(componentHandlers).length === 0) mouseHandlers.delete(index)
  }
}

export function registerGlobalMouseHandler(handler: MouseHandler): () => void {
  globalMouseHandlers.push(handler)
  return () => {
    const i = globalMouseHandlers.indexOf(handler)
    if (i >= 0) globalMouseHandlers.splice(i, 1)
  }
}

export function registerFocusHandler(index: number, handler: FocusHandler): () => void {
  if (!focusHandlers.has(index)) focusHandlers.set(index, [])
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

export function registerValueHandler(index: number, handler: ValueHandler): () => void {
  if (!valueHandlers.has(index)) valueHandlers.set(index, [])
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

export function registerScrollHandler(index: number, handler: ScrollHandler): () => void {
  if (!scrollHandlers.has(index)) scrollHandlers.set(index, [])
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

export function registerGlobalScrollHandler(handler: ScrollHandler): () => void {
  globalScrollHandlers.push(handler)
  return () => {
    const i = globalScrollHandlers.indexOf(handler)
    if (i >= 0) globalScrollHandlers.splice(i, 1)
  }
}

export function registerResizeHandler(handler: ResizeHandler): () => void {
  resizeHandlers.push(handler)
  return () => {
    const i = resizeHandlers.indexOf(handler)
    if (i >= 0) resizeHandlers.splice(i, 1)
  }
}

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

let currentBuffer: SharedBuffer | null = null

function dispatchEvent(event: SparkEvent): void {
  switch (event.type) {
    case EventType.Key: {
      for (const handler of globalKeyHandlers) {
        if (handler(event) === true) return
      }

      if (currentBuffer) {
        let target = event.componentIndex
        let depth = 0

        while (depth < 100) {
          const handlers = keyHandlers.get(target)
          if (handlers) {
            for (const handler of handlers) {
              if (handler(event) === true) return
            }
          }

          const parent = getParentIndex(currentBuffer, target)
          if (parent < 0) break
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
      for (const handler of globalMouseHandlers) {
        handler(event)
      }

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
            }
          }

          const parent = getParentIndex(currentBuffer, target)
          if (parent < 0) break
          target = parent
          depth++
        }
      }
      break
    }

    case EventType.Scroll: {
      for (const handler of globalScrollHandlers) {
        handler(event)
      }
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
// EVENT LISTENER (Atomics.waitAsync - REACTIVE)
// =============================================================================

let running = false
let wakeInt32: Int32Array | null = null

/**
 * Start the event listener.
 *
 * Uses Atomics.waitAsync - SUSPENDS until Rust notifies.
 * NOT polling. The while(running) loop waits for notification each iteration.
 */
export function startEventListener(buf: SharedBuffer): void {
  if (running) return

  running = true
  currentBuffer = buf
  wakeInt32 = new Int32Array(buf.raw, H_WAKE_TS, 1)

  waitForEvents()
}

async function waitForEvents(): Promise<void> {
  while (running && wakeInt32 && currentBuffer) {
    const currentValue = Atomics.load(wakeInt32, 0)

    // Wait for value change or minimal timeout (1ms for cross-platform compatibility)
    const result = Atomics.waitAsync(wakeInt32, 0, currentValue, 1)

    if (result.async) {
      await result.value
    }

    if (!running) break

    Atomics.store(wakeInt32, 0, 0)

    const events = readEvents(currentBuffer)
    for (const event of events) {
      dispatchEvent(event)
    }
  }
}

export function stopEventListener(): void {
  running = false

  if (wakeInt32) {
    Atomics.store(wakeInt32, 0, 1)
    Atomics.notify(wakeInt32, 0)
  }

  wakeInt32 = null
  currentBuffer = null
}

export function isEventListenerRunning(): boolean {
  return running
}

// =============================================================================
// HANDLER CLEANUP
// =============================================================================

export function cleanupHandlers(index: number): void {
  keyHandlers.delete(index)
  mouseHandlers.delete(index)
  focusHandlers.delete(index)
  valueHandlers.delete(index)
  scrollHandlers.delete(index)
}

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
// MANUAL EVENT DISPATCH (for testing)
// =============================================================================

export function dispatchEventManual(event: SparkEvent): void {
  dispatchEvent(event)
}

export function processEvents(buf: SharedBuffer): SparkEvent[] {
  const events = readEvents(buf)
  for (const event of events) {
    dispatchEvent(event)
  }
  return events
}

// =============================================================================
// MODIFIER CHECKS
// =============================================================================

export function hasCtrl(event: KeyEvent): boolean {
  return (event.modifiers & MODIFIER_CTRL) !== 0
}

export function hasAlt(event: KeyEvent): boolean {
  return (event.modifiers & MODIFIER_ALT) !== 0
}

export function hasShift(event: KeyEvent): boolean {
  return (event.modifiers & MODIFIER_SHIFT) !== 0
}

export function hasMeta(event: KeyEvent): boolean {
  return (event.modifiers & MODIFIER_META) !== 0
}

export function isKeyPress(event: KeyEvent): boolean {
  return event.keyState === KEY_STATE_PRESS
}

export function isKeyRepeat(event: KeyEvent): boolean {
  return event.keyState === KEY_STATE_REPEAT
}

export function isKeyRelease(event: KeyEvent): boolean {
  return event.keyState === KEY_STATE_RELEASE
}

// =============================================================================
// KEY CODE CONSTANTS
// =============================================================================

export const KEY_ENTER = 13
export const KEY_TAB = 9
export const KEY_BACKSPACE = 8
export const KEY_ESCAPE = 27
export const KEY_DELETE = 127
export const KEY_SPACE = 32

export const KEY_UP = 0x1001
export const KEY_DOWN = 0x1002
export const KEY_LEFT = 0x1003
export const KEY_RIGHT = 0x1004
export const KEY_HOME = 0x1005
export const KEY_END = 0x1006
export const KEY_PAGE_UP = 0x1007
export const KEY_PAGE_DOWN = 0x1008
export const KEY_INSERT = 0x1009

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
// KEY HELPERS
// =============================================================================

export function getChar(event: KeyEvent): string | undefined {
  if (event.keycode >= 32 && event.keycode <= 126) {
    return String.fromCharCode(event.keycode)
  }
  return undefined
}

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
      if (event.keycode >= KEY_F1 && event.keycode <= KEY_F12) {
        return `f${event.keycode - 0x2000}`
      }
      if (event.keycode >= 32 && event.keycode <= 126) {
        return String.fromCharCode(event.keycode)
      }
      return `unknown(${event.keycode})`
  }
}

export function isEnter(event: KeyEvent): boolean {
  return event.keycode === KEY_ENTER
}

export function isSpace(event: KeyEvent): boolean {
  return event.keycode === KEY_SPACE
}

export function isEscape(event: KeyEvent): boolean {
  return event.keycode === KEY_ESCAPE
}

export function isArrowKey(event: KeyEvent): boolean {
  return event.keycode >= KEY_UP && event.keycode <= KEY_RIGHT
}

export function isFunctionKey(event: KeyEvent): boolean {
  return event.keycode >= KEY_F1 && event.keycode <= KEY_F12
}

export function isChar(event: KeyEvent, char: string): boolean {
  return event.keycode === char.charCodeAt(0)
}
