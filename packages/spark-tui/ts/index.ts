/**
 * SparkTUI - Hybrid TUI Framework
 *
 * All Rust benefits without borrowing a single thing.
 *
 * TypeScript primitives, Rust engine, SharedArrayBuffer bridge.
 * Pure reactive - no loops, no polling, no fixed FPS.
 */

// Re-export signals for convenience
export { signal, derived, effect, batch, state } from '@rlabs-inc/signals'

// =============================================================================
// MOUNT API - Entry point for SparkTUI apps
// =============================================================================
export {
  mount,
  mountSync,
  mountForTest,
  isMounted,
  getRenderMode,
  type MountOptions,
  type MountHandle,
  type MountRenderMode,
} from './engine/mount'

// =============================================================================
// PRIMITIVES - Building blocks for terminal UIs
// =============================================================================
export {
  box,
  text,
  input,
  each,
  show,
  when,
  scoped,
  onCleanup,
  cycle,
  pulse,
  Frames,
} from './primitives'

export type {
  BoxProps,
  TextProps,
  InputProps,
  CursorConfig,
  CursorStyle,
  BlinkConfig,
  Cleanup,
  MouseProps,
  AnimationOptions,
  CycleOptions,
  PulseOptions,
} from './primitives'

// =============================================================================
// KEYBOARD & EVENTS - Ergonomic event handling
// =============================================================================
export {
  // Key checkers
  isEnter,
  isSpace,
  isEscape,
  isArrowKey,
  isFunctionKey,
  isChar,
  isKeyPress,
  isKeyRepeat,
  isKeyRelease,
  // Modifier checkers
  hasCtrl,
  hasAlt,
  hasShift,
  hasMeta,
  // Key info
  getChar,
  getKeyName,
  // Key codes
  KEY_ENTER,
  KEY_TAB,
  KEY_BACKSPACE,
  KEY_ESCAPE,
  KEY_DELETE,
  KEY_SPACE,
  KEY_UP,
  KEY_DOWN,
  KEY_LEFT,
  KEY_RIGHT,
  KEY_HOME,
  KEY_END,
  KEY_PAGE_UP,
  KEY_PAGE_DOWN,
  // Types
  type KeyEvent,
  type MouseEvent,
  type ScrollEvent,
  type FocusEvent,
  type SparkEvent,
} from './engine/events'

// =============================================================================
// THEME - Reactive styling system
// =============================================================================
export {
  t,              // Reactive colors: t.primary, t.error, t.textMuted, etc.
  themes,         // Theme presets: dracula, nord, catppuccin, etc.
  setTheme,       // Switch theme: setTheme('dracula') or setTheme({ primary: '#ff0000' })
  getThemeNames,  // List available: ['terminal', 'dracula', 'nord', ...]
} from './state/theme'

// =============================================================================
// TEXT STYLING - Shorthand constants for clean syntax
// =============================================================================
// Usage: text({ content: 'Hello', bold, underline, fg: t.error })
export const bold = true
export const dim = true
export const italic = true
export const underline = true
export const blink = true
export const inverse = true
export const hidden = true
export const strikethrough = true

// For power users who need the raw bitmask
export { Attr } from './types'

// =============================================================================
// TYPES - Color and layout types
// =============================================================================
export type { RGBA, ColorInput } from './types'
export { parseColor, TERMINAL_DEFAULT, ansiColor } from './types/color'
