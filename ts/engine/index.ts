/**
 * TUI Framework - Engine
 *
 * The component engine using parallel arrays pattern.
 * Components allocate indices and write to arrays.
 * Deriveds read arrays and RETURN computed values.
 */

// Mount API - THE entry point for SparkTUI apps
export {
  mount,
  mountForTest,
  isMounted,
  getRenderMode,
  type MountOptions,
  type MountHandle,
  type RenderMode,
} from './mount'

// Registry
export {
  allocateIndex,
  releaseIndex,
  getIndex,
  getId,
  getAllocatedIndices,
  isAllocated,
  getAllocatedCount,
  getCurrentParentIndex,
  pushParentContext,
  popParentContext,
  resetRegistry,
} from './registry'

// Keyboard helpers - friendly API for key events
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
  // Key codes (for advanced use)
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
} from './events'

