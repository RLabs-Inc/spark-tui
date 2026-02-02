# Core Types

> Type definitions for colors, dimensions, borders, text attributes, grid layout, and reactive props.

## Import

```ts
import {
  // Color types and utilities
  type RGBA,
  type ColorInput,
  parseColor,
  rgba,
  Colors,
  TERMINAL_DEFAULT,
  ansiColor,
  isAnsiColor,
  isTerminalDefault,
  oklch,
  rgbToOklch,

  // Color manipulation
  rgbaEqual,
  rgbaBlend,
  rgbaLerp,
  dim,
  brighten,

  // Dimension types
  type Dimension,
  type ParsedDimension,
  parseDimension,

  // Cell and text attributes
  Attr,
  type CellAttrs,
  type Cell,

  // Border styles
  BorderStyle,
  BorderChars,
  type BorderStyleValue,

  // Grid types
  type GridTrackSize,
  type GridTemplate,
  type GridAutoFlow,
  type GridLine,

  // Reactive types
  type Reactive,
} from 'spark-tui';
```

## ColorInput

The `ColorInput` type accepts flexible color formats for all color props (`fg`, `bg`, `borderColor`, etc.):

### Type Definition

```ts
type ColorInput =
  | RGBA                      // Object: { r: 255, g: 0, b: 0, a: 255 }
  | string                    // Hex, CSS name, rgb(), hsl(), oklch()
  | number                    // Integer: 0xff0000 (0xRRGGBB format)
  | null;                     // Terminal default color
```

### Supported Formats

| Format | Example | Description |
|--------|---------|-------------|
| RGBA object | `{ r: 255, g: 0, b: 0, a: 255 }` | Direct RGBA values (0-255) |
| Hex (6-digit) | `'#ff0000'` | Standard hex color |
| Hex (3-digit) | `'#f00'` | Shorthand hex (expands to `#ff0000`) |
| Hex (8-digit) | `'#ff0000ff'` | Hex with alpha |
| CSS name | `'red'`, `'dodgerblue'`, `'transparent'` | All CSS named colors |
| RGB | `'rgb(255, 0, 0)'` | CSS rgb() function |
| RGBA | `'rgba(255, 0, 0, 0.5)'` | CSS rgba() with alpha (0-1) |
| HSL | `'hsl(0, 100%, 50%)'` | CSS hsl() function |
| HSLA | `'hsla(0, 100%, 50%, 0.5)'` | CSS hsla() with alpha |
| OKLCH | `'oklch(0.7 0.15 30)'` | Perceptually uniform color space |
| Integer | `0xff0000` | 0xRRGGBB format |
| null | `null` | Use terminal default |

### Usage Examples

```ts
import { box, text } from 'spark-tui';

// All of these are equivalent ways to set red
box({ bg: '#ff0000' });                    // Hex string
box({ bg: '#f00' });                       // Short hex
box({ bg: 'red' });                        // CSS name
box({ bg: 'rgb(255, 0, 0)' });             // RGB string
box({ bg: 'hsl(0, 100%, 50%)' });          // HSL string
box({ bg: { r: 255, g: 0, b: 0, a: 255 }});// RGBA object
box({ bg: 0xff0000 });                     // Integer

// Semi-transparent colors
box({ bg: 'rgba(255, 0, 0, 0.5)' });       // 50% opacity
box({ bg: '#ff000080' });                  // 50% opacity (hex)
box({ bg: { r: 255, g: 0, b: 0, a: 128 }});// 50% opacity (object)

// OKLCH for perceptually uniform colors
box({ bg: 'oklch(0.7 0.15 30)' });         // Warm red
box({ bg: 'oklch(0.7 0.15 200)' });        // Teal
box({ bg: 'oklch(0.7 0.15 300)' });        // Purple

// CSS named colors
text({ fg: 'dodgerblue' });
text({ fg: 'coral' });
text({ fg: 'mediumseagreen' });

// Terminal default (inherits from terminal settings)
box({ bg: null });
text({ fg: null });
```

### Why OKLCH?

OKLCH is a perceptually uniform color space, meaning:
- Equal steps in lightness look equally different to the human eye
- Hue rotation maintains consistent perceived brightness
- Great for generating color palettes programmatically

```ts
// Generate a rainbow with consistent perceived brightness
const hues = [0, 60, 120, 180, 240, 300];
const rainbow = hues.map(h => `oklch(0.7 0.15 ${h})`);

// All these colors appear equally "bright" to the eye
box({ bg: 'oklch(0.7 0.15 0)' });    // Red
box({ bg: 'oklch(0.7 0.15 120)' });  // Green
box({ bg: 'oklch(0.7 0.15 240)' });  // Blue
```

## RGBA

### Interface

```ts
interface RGBA {
  r: number;  // Red (0-255)
  g: number;  // Green (0-255)
  b: number;  // Blue (0-255)
  a: number;  // Alpha (0-255, 255 = opaque)
}
```

Using integers for exact comparison - no floating point epsilon needed.

### Creating RGBA Colors

```ts
import { rgba, parseColor, oklch, ansiColor } from 'spark-tui';

// Direct construction
const red: RGBA = { r: 255, g: 0, b: 0, a: 255 };

// Using rgba() helper
const green = rgba(0, 255, 0);       // Alpha defaults to 255
const semiRed = rgba(255, 0, 0, 128); // 50% opacity

// Parsing from various formats
parseColor('#ff5500');           // Hex
parseColor('rgb(255, 85, 0)');   // CSS rgb()
parseColor('red');               // CSS named color
parseColor(0xff5500);            // Integer (0xRRGGBB)

// OKLCH colors (perceptually uniform)
oklch(0.7, 0.15, 200);           // L, C, H
oklch(0.7, 0.15, 200, 128);      // With alpha

// ANSI colors (use terminal palette)
ansiColor(1);   // ANSI red
ansiColor(12);  // ANSI bright blue
```

### Special Color Values

```ts
import { TERMINAL_DEFAULT, ansiColor, Colors } from 'spark-tui';

// Terminal default - lets terminal choose
const defaultFg = TERMINAL_DEFAULT;  // { r: -1, g: -1, b: -1, a: 255 }

// ANSI color marker - uses terminal's color palette
const ansiRed = ansiColor(1);        // { r: -2, g: 1, b: 0, a: 255 }

// Built-in color presets
Colors.BLACK      // { r: 0, g: 0, b: 0, a: 255 }
Colors.WHITE      // { r: 255, g: 255, b: 255, a: 255 }
Colors.RED        // { r: 255, g: 0, b: 0, a: 255 }
Colors.GREEN      // { r: 0, g: 255, b: 0, a: 255 }
Colors.BLUE       // { r: 0, g: 0, b: 255, a: 255 }
Colors.YELLOW     // { r: 255, g: 255, b: 0, a: 255 }
Colors.CYAN       // { r: 0, g: 255, b: 255, a: 255 }
Colors.MAGENTA    // { r: 255, g: 0, b: 255, a: 255 }
Colors.GRAY       // { r: 128, g: 128, b: 128, a: 255 }
Colors.TRANSPARENT // { r: 0, g: 0, b: 0, a: 0 }
```

### Color Detection

```ts
import { isTerminalDefault, isAnsiColor, getAnsiIndex } from 'spark-tui';

isTerminalDefault(TERMINAL_DEFAULT);  // true
isTerminalDefault(Colors.RED);        // false

isAnsiColor(ansiColor(12));           // true
isAnsiColor(Colors.RED);              // false

getAnsiIndex(ansiColor(12));          // 12
```

### `parseColor()` Function

Parses any color input to RGBA:

```ts
function parseColor(input: string | number | RGBA): RGBA
```

| Input | Example | Result |
|-------|---------|--------|
| Hex string | `'#ff5500'` | `{ r: 255, g: 85, b: 0, a: 255 }` |
| RGB string | `'rgb(255, 85, 0)'` | `{ r: 255, g: 85, b: 0, a: 255 }` |
| RGBA string | `'rgba(255, 85, 0, 0.5)'` | `{ r: 255, g: 85, b: 0, a: 128 }` |
| HSL string | `'hsl(20, 100%, 50%)'` | Converted to RGB |
| OKLCH string | `'oklch(0.7 0.15 200)'` | Converted to RGB |
| Named color | `'red'` | `{ r: 255, g: 0, b: 0, a: 255 }` |
| Integer | `0xff5500` | `{ r: 255, g: 85, b: 0, a: 255 }` |
| `'transparent'` | - | `{ r: 0, g: 0, b: 0, a: 0 }` |
| `'inherit'` | - | `TERMINAL_DEFAULT` |
| RGBA object | `{ r: 255, g: 0, b: 0, a: 255 }` | Passthrough |

### Color Manipulation

```ts
import { rgbaBlend, rgbaLerp, dim, brighten, rgbaEqual } from 'spark-tui';

// Alpha blending (src over dst)
const blended = rgbaBlend(
  { r: 255, g: 0, b: 0, a: 128 },  // 50% red
  { r: 0, g: 0, b: 255, a: 255 }   // Blue background
);

// Linear interpolation
const midpoint = rgbaLerp(Colors.RED, Colors.BLUE, 0.5);

// Brightness adjustment
const darker = dim(Colors.RED, 0.5);       // 50% brightness
const lighter = brighten(Colors.RED, 1.5); // 150% brightness

// Equality check
rgbaEqual(Colors.RED, Colors.RED);  // true
```

### OKLCH Functions

OKLCH provides perceptually uniform color manipulation:

```ts
import { oklch, rgbToOklch } from 'spark-tui';

// Create color from OKLCH
const color = oklch(0.7, 0.15, 200);  // L=0.7, C=0.15, H=200

// Convert RGB to OKLCH
const { l, c, h } = rgbToOklch(Colors.RED);
// l: lightness (0-1)
// c: chroma (0-~0.4)
// h: hue (0-360)
```

## Dimension

### Type Definition

```ts
type Dimension = number | `${number}%`;
```

| Value | Meaning | Example |
|-------|---------|---------|
| `number` | Absolute value in terminal cells | `width: 50` (50 characters) |
| `'N%'` | Percentage of parent | `width: '100%'` (full parent width) |
| `0` | Auto-size based on content | `width: 0` |

### ParsedDimension Interface

```ts
interface ParsedDimension {
  value: number;
  isPercent: boolean;
}
```

### `parseDimension()` Function

```ts
function parseDimension(dim: Dimension | undefined | null): ParsedDimension
```

```ts
import { parseDimension } from 'spark-tui';

parseDimension(50);       // { value: 50, isPercent: false }
parseDimension('100%');   // { value: 100, isPercent: true }
parseDimension(null);     // { value: 0, isPercent: false }
```

## Attr (Text Attributes)

### Bitfield Constants

```ts
const Attr = {
  NONE: 0,
  BOLD: 1 << 0,          // 1
  DIM: 1 << 1,           // 2
  ITALIC: 1 << 2,        // 4
  UNDERLINE: 1 << 3,     // 8
  BLINK: 1 << 4,         // 16
  INVERSE: 1 << 5,       // 32
  HIDDEN: 1 << 6,        // 64
  STRIKETHROUGH: 1 << 7, // 128
} as const;

type CellAttrs = number;
```

### Usage

```ts
import { Attr } from 'spark-tui';
import { text } from 'spark-tui';

// Single attribute
text({ content: 'Bold text', attrs: Attr.BOLD });

// Combined attributes (bitwise OR)
text({
  content: 'Bold and italic',
  attrs: Attr.BOLD | Attr.ITALIC
});

// Check for attribute
const hasUnderline = (attrs: number) => (attrs & Attr.UNDERLINE) !== 0;
```

## Cell

The atomic unit of terminal rendering:

```ts
interface Cell {
  /** Unicode codepoint (32 for space) */
  char: number;
  /** Foreground color */
  fg: RGBA;
  /** Background color */
  bg: RGBA;
  /** Attribute flags (bold, italic, etc.) */
  attrs: CellAttrs;
}
```

## BorderStyle

### Constants

```ts
const BorderStyle = {
  NONE: 0,
  SINGLE: 1,        // --- | ┌ ┐ └ ┘
  DOUBLE: 2,        // === || ╔ ╗ ╚ ╝
  ROUNDED: 3,       // --- | ╭ ╮ ╰ ╯
  BOLD: 4,          // === || ┏ ┓ ┗ ┛
  DASHED: 5,        // ┄┄┄ ┆ ┌ ┐ └ ┘
  DOTTED: 6,        // ··· · · · · ·
  ASCII: 7,         // --- | + + + +
  BLOCK: 8,         // ### # # # # #
  DOUBLE_HORZ: 9,   // === | ╒ ╕ ╘ ╛
  DOUBLE_VERT: 10,  // --- || ╓ ╖ ╙ ╜
} as const;

type BorderStyleValue = (typeof BorderStyle)[keyof typeof BorderStyle];
```

### Border Characters

```ts
// BorderChars[style] = [horizontal, vertical, topLeft, topRight, bottomRight, bottomLeft]
const BorderChars: Record<number, readonly [string, string, string, string, string, string]> = {
  [BorderStyle.SINGLE]:      ['─', '│', '┌', '┐', '┘', '└'],
  [BorderStyle.DOUBLE]:      ['═', '║', '╔', '╗', '╝', '╚'],
  [BorderStyle.ROUNDED]:     ['─', '│', '╭', '╮', '╯', '╰'],
  [BorderStyle.BOLD]:        ['━', '┃', '┏', '┓', '┛', '┗'],
  [BorderStyle.DASHED]:      ['┄', '┆', '┌', '┐', '┘', '└'],
  [BorderStyle.DOTTED]:      ['·', '·', '·', '·', '·', '·'],
  [BorderStyle.ASCII]:       ['-', '|', '+', '+', '+', '+'],
  [BorderStyle.BLOCK]:       ['█', '█', '█', '█', '█', '█'],
  [BorderStyle.DOUBLE_HORZ]: ['═', '│', '╒', '╕', '╛', '╘'],
  [BorderStyle.DOUBLE_VERT]: ['─', '║', '╓', '╖', '╜', '╙'],
};
```

### Usage

```ts
import { BorderStyle, box } from 'spark-tui';

box({
  border: BorderStyle.ROUNDED,
  borderColor: { r: 100, g: 150, b: 200, a: 255 },
  children: () => { /* ... */ }
});
```

### Visual Reference

```
SINGLE       DOUBLE       ROUNDED      BOLD
┌──────┐     ╔══════╗     ╭──────╮     ┏━━━━━━┓
│      │     ║      ║     │      │     ┃      ┃
└──────┘     ╚══════╝     ╰──────╯     ┗━━━━━━┛

DASHED       DOTTED       ASCII        BLOCK
┌┄┄┄┄┄┄┐     ··········     +------+     ████████
┆      ┆     ·        ·     |      |     █      █
└┄┄┄┄┄┄┘     ··········     +------+     ████████
```

## Grid Types

### GridTrackSize

Defines how a grid track (row or column) is sized:

```ts
type GridTrackSize =
  | number                    // Fixed size in terminal cells
  | `${number}%`              // Percentage of container
  | `${number}fr`             // Fractional unit
  | 'auto'                    // Auto sizing
  | 'min-content'             // Minimum content size
  | 'max-content';            // Maximum content size
```

### GridTemplate

Array of track sizes for rows or columns:

```ts
type GridTemplate = GridTrackSize[];
```

### GridAutoFlow

Controls how auto-placed items flow in the grid:

```ts
type GridAutoFlow = 'row' | 'column' | 'row dense' | 'column dense';
```

### GridLine

Position within the grid:

```ts
type GridLine = number | `span ${number}` | 'auto';
```

### Grid Usage Example

```ts
import { box, text } from 'spark-tui';

box({
  display: 'grid',
  gridTemplateColumns: ['1fr', '2fr', '1fr'],  // 3 columns, middle is 2x
  gridTemplateRows: [5, 'auto', 5],            // Header, content, footer
  gap: 1,
  children: () => {
    // Header spanning all columns
    box({
      gridColumnStart: 1,
      gridColumnEnd: 4,  // Spans columns 1-3
      children: () => text({ content: 'Header' })
    });

    // Sidebar
    box({
      gridRowStart: 2,
      children: () => text({ content: 'Sidebar' })
    });

    // Main content
    box({
      gridColumnStart: 2,
      gridRowStart: 2,
      children: () => text({ content: 'Content' })
    });

    // Right panel
    box({
      gridColumnStart: 3,
      gridRowStart: 2,
      children: () => text({ content: 'Panel' })
    });

    // Footer spanning all columns
    box({
      gridColumnStart: 1,
      gridColumnEnd: 4,
      gridRowStart: 3,
      children: () => text({ content: 'Footer' })
    });
  }
});
```

## Reactive<T>

Makes a prop value accept both static values and reactive sources:

```ts
type Reactive<T> = T | WritableSignal<T> | Binding<T> | ReadonlyBinding<T> | (() => T);
```

### Usage

```ts
import { state, derived } from '@rlabs-inc/signals';
import { box, text } from 'spark-tui';

const width = state(50);
const doubled = derived(() => width.value * 2);

box({
  // Static value
  height: 10,

  // Signal
  width: width,

  // Derived
  padding: derived(() => width.value > 40 ? 2 : 1),

  // Inline getter
  opacity: () => width.value / 100,

  children: () => {
    text({
      // Reactive content
      content: () => `Width: ${width.value}`
    });
  }
});
```

## Cursor Types

### CursorShape

```ts
type CursorShape = 'block' | 'underline' | 'bar';
```

### Cursor Interface

```ts
interface Cursor {
  x: number;
  y: number;
  shape: CursorShape;
  visible: boolean;
  blinking: boolean;
}
```

## Input Events

### KeyEvent

```ts
interface Modifiers {
  ctrl: boolean;
  alt: boolean;
  shift: boolean;
  meta: boolean;
}

type KeyState = 'press' | 'release' | 'repeat';

interface KeyEvent {
  key: string;           // 'a', 'Enter', 'ArrowUp', etc.
  modifiers: Modifiers;
  state: KeyState;
}
```

### MouseEvent

```ts
type MouseButton = 'left' | 'middle' | 'right' | 'none';
type MouseAction = 'down' | 'up' | 'move' | 'scroll';

interface MouseEvent {
  x: number;              // Cell X position (0-indexed)
  y: number;              // Cell Y position (0-indexed)
  button: MouseButton;
  action: MouseAction;
  scrollDelta?: number;   // -1 up, 1 down (when action is 'scroll')
  modifiers: Modifiers;
}
```

### ResizeEvent

```ts
interface ResizeEvent {
  width: number;
  height: number;
}
```

## Component Types

Internal type identifiers:

```ts
const ComponentType = {
  NONE: 0,
  BOX: 1,
  TEXT: 2,
  INPUT: 3,
  SELECT: 4,
  PROGRESS: 5,
  CANVAS: 6,
} as const;

type ComponentTypeValue = (typeof ComponentType)[keyof typeof ComponentType];
```

## Scroll State

```ts
interface ScrollState {
  x: number;     // Current horizontal scroll
  y: number;     // Current vertical scroll
  maxX: number;  // Maximum horizontal scroll
  maxY: number;  // Maximum vertical scroll
}
```

## See Also

- [Theming System](/docs/theming/themes.md)
- [Box Component](/docs/components/box.md)
- [Text Component](/docs/components/text.md)
- [Input Component](/docs/components/input.md)
- [Grid Layout](/docs/concepts/grid-layout.md)
