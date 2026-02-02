# text

> Display text with styling, alignment, and wrapping.

The `text` component renders text content in the terminal with support for colors, alignment, text wrapping, and text attributes (bold, italic, underline, etc.).

## Import

```ts
import { text } from 'spark-tui'
// or
import { text } from '../ts/primitives'
```

## Signature

```ts
function text(props: TextProps): Cleanup
```

## Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| props | `TextProps` | Yes | - | Configuration object (requires `content`) |

Returns a `Cleanup` function that unmounts the component when called.

## Examples

### Basic Usage

```ts
import { text } from 'spark-tui'

// Simple static text
text({ content: 'Hello, SparkTUI!' })

// With colors
text({
  content: 'Colored text',
  fg: { r: 80, g: 200, b: 255, a: 255 },  // Cyan
  bg: { r: 30, g: 30, b: 40, a: 255 },    // Dark background
})
```

### With Signals (Reactive)

```ts
import { signal, derived } from '@rlabs-inc/signals'
import { text } from 'spark-tui'

const count = signal(0)
const message = signal('Hello')

// Direct signal binding
text({ content: count })  // Displays "0", updates automatically

// Inline derived (getter function)
text({ content: () => `Count is: ${count.value}` })

// Named derived
const status = derived(() =>
  count.value > 10 ? 'High' : 'Low'
)
text({ content: status })

// Reactive colors
text({
  content: message,
  fg: derived(() => count.value > 5
    ? { r: 255, g: 100, b: 100, a: 255 }  // Red when high
    : { r: 100, g: 255, b: 100, a: 255 }  // Green when low
  ),
})
```

### Text Alignment

```ts
import { box, text } from 'spark-tui'

box({
  width: 40,
  border: 1,
  flexDirection: 'column',
  children: () => {
    text({ content: 'Left aligned (default)', align: 'left' })
    text({ content: 'Center aligned', align: 'center' })
    text({ content: 'Right aligned', align: 'right' })
  }
})
```

### Text Wrapping

```ts
import { box, text } from 'spark-tui'

const longText = 'This is a very long piece of text that will need to wrap to fit within the container.'

box({
  width: 30,
  border: 1,
  flexDirection: 'column',
  gap: 1,
  children: () => {
    // Wraps to multiple lines (default)
    text({ content: longText, wrap: 'wrap' })

    // Single line, no wrap
    text({ content: longText, wrap: 'nowrap' })

    // Truncates with ellipsis
    text({ content: longText, wrap: 'truncate' })
  }
})
```

### Text Attributes

```ts
import { text } from 'spark-tui'
import { Attr } from 'spark-tui/types'

// Bold text
text({ content: 'Bold text', attrs: Attr.BOLD })

// Italic text
text({ content: 'Italic text', attrs: Attr.ITALIC })

// Underlined text
text({ content: 'Underlined text', attrs: Attr.UNDERLINE })

// Combined attributes (bitwise OR)
text({
  content: 'Bold and underlined',
  attrs: Attr.BOLD | Attr.UNDERLINE,
})

// All attributes available:
// Attr.NONE, Attr.BOLD, Attr.DIM, Attr.ITALIC,
// Attr.UNDERLINE, Attr.BLINK, Attr.INVERSE,
// Attr.HIDDEN, Attr.STRIKETHROUGH
```

### Styled with Variants

```ts
import { text } from 'spark-tui'

// Using theme variants for consistent styling
text({ content: 'Primary text', variant: 'primary' })
text({ content: 'Success message', variant: 'success' })
text({ content: 'Warning message', variant: 'warning' })
text({ content: 'Error message', variant: 'error' })
text({ content: 'Info text', variant: 'info' })
```

### In Grid Layout

```ts
import { box, text } from 'spark-tui'

box({
  display: 'grid',
  gridTemplateColumns: ['1fr', '2fr'],
  gap: 1,
  children: () => {
    text({ content: 'Label:' })
    text({ content: 'Value' })

    text({ content: 'Spanning:', gridColumnStart: 1, gridColumnEnd: 'span 2' })
  }
})
```

### With Mouse Events

```ts
import { signal } from '@rlabs-inc/signals'
import { text } from 'spark-tui'

const isHovered = signal(false)

text({
  content: 'Hover over me!',
  fg: () => isHovered.value
    ? { r: 255, g: 255, b: 0, a: 255 }   // Yellow on hover
    : { r: 200, g: 200, b: 200, a: 255 }, // Gray normally
  onMouseEnter: () => { isHovered.value = true },
  onMouseLeave: () => { isHovered.value = false },
  onClick: () => { console.log('Clicked!') },
})
```

### Dashboard Example

```ts
import { signal, derived } from '@rlabs-inc/signals'
import { box, text } from 'spark-tui'

const cpu = signal(45)
const memory = signal(62)

function MetricDisplay(label: string, value: { readonly value: number }) {
  box({
    flexDirection: 'row',
    gap: 2,
    children: () => {
      text({ content: `${label}:`, fg: { r: 150, g: 150, b: 180, a: 255 } })
      text({
        content: derived(() => `${value.value}%`),
        fg: derived(() => value.value > 80
          ? { r: 255, g: 100, b: 100, a: 255 }  // Red when high
          : { r: 100, g: 255, b: 100, a: 255 }  // Green when normal
        ),
      })
    }
  })
}

box({
  flexDirection: 'column',
  gap: 1,
  children: () => {
    MetricDisplay('CPU', cpu)
    MetricDisplay('Memory', memory)
  }
})
```

## Props Reference

### Core Props

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `id` | `string` | No | auto-generated | Optional component identifier |
| `content` | `Reactive<string \| number>` | **Yes** | - | Text content to display |
| `visible` | `Reactive<boolean>` | No | `true` | Whether the text is visible |
| `variant` | `Variant` | No | `'default'` | Theme variant for automatic styling |

### Text Styling Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `align` | `Reactive<'left' \| 'center' \| 'right'>` | `'left'` | Text alignment within container |
| `wrap` | `Reactive<'wrap' \| 'nowrap' \| 'truncate'>` | `'wrap'` | Text wrapping behavior |
| `attrs` | `Reactive<CellAttrs>` | `Attr.NONE` | Text attributes (bold, italic, etc.) |

### Dimension Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `width` | `Reactive<Dimension>` | auto | Width in cells or percentage |
| `height` | `Reactive<Dimension>` | auto | Height in cells or percentage |
| `minWidth` | `Reactive<Dimension>` | - | Minimum width constraint |
| `maxWidth` | `Reactive<Dimension>` | - | Maximum width constraint |
| `minHeight` | `Reactive<Dimension>` | - | Minimum height constraint |
| `maxHeight` | `Reactive<Dimension>` | - | Maximum height constraint |

### Style Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `fg` | `Reactive<ColorInput>` | inherited | Foreground (text) color |
| `bg` | `Reactive<ColorInput>` | transparent | Background color |
| `opacity` | `Reactive<number>` | `1` | Opacity (0-1) |
| `zIndex` | `Reactive<number>` | `0` | Stacking order |

**ColorInput** accepts flexible formats: hex strings (`'#ff0000'`, `'#f00'`), CSS names (`'red'`, `'dodgerblue'`), RGB/RGBA (`'rgb(255,0,0)'`, `'rgba(255,0,0,0.5)'`), HSL (`'hsl(0,100%,50%)'`), OKLCH (`'oklch(0.7 0.15 30)'`), integers (`0xff0000`), RGBA objects, or `null` for terminal default.

### Spacing Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `padding` | `Reactive<number>` | `0` | Padding on all sides |
| `paddingTop` | `Reactive<number>` | - | Top padding |
| `paddingRight` | `Reactive<number>` | - | Right padding |
| `paddingBottom` | `Reactive<number>` | - | Bottom padding |
| `paddingLeft` | `Reactive<number>` | - | Left padding |
| `margin` | `Reactive<number>` | `0` | Margin on all sides |
| `marginTop` | `Reactive<number>` | - | Top margin |
| `marginRight` | `Reactive<number>` | - | Right margin |
| `marginBottom` | `Reactive<number>` | - | Bottom margin |
| `marginLeft` | `Reactive<number>` | - | Left margin |
| `gap` | `Reactive<number>` | `0` | Gap (for layout purposes) |
| `rowGap` | `Reactive<number>` | - | Row gap |
| `columnGap` | `Reactive<number>` | - | Column gap |

### Flexbox Item Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `grow` | `Reactive<number>` | `0` | Flex grow factor |
| `shrink` | `Reactive<number>` | `1` | Flex shrink factor |
| `flexBasis` | `Reactive<number>` | auto | Initial main size |
| `alignSelf` | `Reactive<'auto' \| 'stretch' \| 'flex-start' \| 'center' \| 'flex-end' \| 'baseline'>` | `'auto'` | Cross-axis alignment override |

### Grid Item Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `gridColumnStart` | `Reactive<GridLine>` | `'auto'` | Column start line |
| `gridColumnEnd` | `Reactive<GridLine>` | `'auto'` | Column end line |
| `gridRowStart` | `Reactive<GridLine>` | `'auto'` | Row start line |
| `gridRowEnd` | `Reactive<GridLine>` | `'auto'` | Row end line |
| `justifySelf` | `Reactive<'auto' \| 'start' \| 'end' \| 'center' \| 'stretch'>` | `'auto'` | Justify override for grid |

### Mouse Event Props

| Prop | Type | Description |
|------|------|-------------|
| `onClick` | `(event: MouseEvent) => void \| boolean` | Called on click. Return `true` to consume. |
| `onMouseDown` | `(event: MouseEvent) => void \| boolean` | Called on mouse button down |
| `onMouseUp` | `(event: MouseEvent) => void \| boolean` | Called on mouse button up |
| `onMouseEnter` | `(event: MouseEvent) => void` | Called when mouse enters |
| `onMouseLeave` | `(event: MouseEvent) => void` | Called when mouse leaves |
| `onScroll` | `(event: ScrollEvent) => void` | Called on scroll events |

## Types

### CellAttrs (Attr)

```ts
const Attr = {
  NONE: 0,
  BOLD: 1 << 0,        // 1
  DIM: 1 << 1,         // 2
  ITALIC: 1 << 2,      // 4
  UNDERLINE: 1 << 3,   // 8
  BLINK: 1 << 4,       // 16
  INVERSE: 1 << 5,     // 32
  HIDDEN: 1 << 6,      // 64
  STRIKETHROUGH: 1 << 7, // 128
} as const

type CellAttrs = number  // Combine with bitwise OR
```

### Variant

```ts
type Variant =
  | 'default'
  | 'primary'
  | 'secondary'
  | 'success'
  | 'warning'
  | 'error'
  | 'info'
  | 'ghost'
  | 'outline'
```

### ColorInput

```ts
type ColorInput =
  | RGBA                      // { r: 255, g: 0, b: 0, a: 255 }
  | string                    // '#ff0000', 'red', 'rgb(255,0,0)', 'oklch(0.7 0.15 30)'
  | number                    // 0xff0000
  | null                      // Terminal default

interface RGBA {
  r: number  // 0-255
  g: number  // 0-255
  b: number  // 0-255
  a: number  // 0-255 (255 = fully opaque)
}
```

**Examples:**
```ts
text({ fg: 'red' })              // CSS name
text({ fg: '#00ff00' })          // Hex
text({ fg: 'oklch(0.8 0.12 250)' }) // OKLCH
text({ fg: 0x00ffff })           // Integer (cyan)
```

### Dimension

```ts
type Dimension = number | `${number}%`
```

### GridLine

```ts
type GridLine = number | `span ${number}` | 'auto'
```

## Notes

- **Content is required**: Unlike `box`, the `content` prop is mandatory for `text`.
- **Numbers auto-convert**: Passing a number to `content` (e.g., `content: 42`) automatically converts it to a string.
- **Unicode support**: Full Unicode support including emoji, CJK characters, and combining characters. Width calculation is Unicode-aware.
- **Reactive props**: Any prop marked `Reactive<T>` accepts static values, signals, deriveds, bindings, or getter functions.
- **No children**: `text` does not support `children`. It only renders its `content`.
- **Inherits colors**: If `fg` is not specified, text inherits the foreground color from its parent.

## See Also

- [box](./box.md) - Container component
- [input](./input.md) - Text input component
- [Attr Constants](../api-reference/types.md#attr) - Text attribute constants
- [Theming](../theming/index.md) - Theme system and variants
