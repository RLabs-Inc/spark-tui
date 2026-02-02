# box

> Container component with flexbox/grid layout, borders, and background.

The `box` component is the fundamental building block for layout in SparkTUI. It provides CSS-like flexbox and grid layout capabilities, borders, backgrounds, and serves as a container for other components.

## Import

```ts
import { box } from 'spark-tui'
// or
import { box } from '../ts/primitives'
```

## Signature

```ts
function box(props?: BoxProps): Cleanup
```

## Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| props | `BoxProps` | No | `{}` | Configuration object for the box |

Returns a `Cleanup` function that unmounts the component when called.

## Examples

### Basic Usage

```ts
import { box, text } from 'spark-tui'

// Simple container with children
box({
  width: 40,
  height: 10,
  border: 1,
  children: () => {
    text({ content: 'Hello, SparkTUI!' })
  }
})
```

### Flexbox Layout

```ts
import { box, text } from 'spark-tui'

// Row layout with centered items
box({
  flexDirection: 'row',
  justifyContent: 'space-between',
  alignItems: 'center',
  padding: 1,
  gap: 2,
  children: () => {
    box({ width: 10, height: 3, bg: { r: 255, g: 0, b: 0, a: 255 } })
    box({ width: 10, height: 3, bg: { r: 0, g: 255, b: 0, a: 255 } })
    box({ width: 10, height: 3, bg: { r: 0, g: 0, b: 255, a: 255 } })
  }
})
```

### With Signals (Reactive)

```ts
import { signal, derived } from '@rlabs-inc/signals'
import { box, text } from 'spark-tui'

const width = signal(40)
const isExpanded = signal(false)

// Reactive dimensions and colors
box({
  width,  // Automatically updates when signal changes
  height: () => isExpanded.value ? 20 : 10,  // Inline derived
  bg: derived(() => isExpanded.value
    ? { r: 50, g: 100, b: 200, a: 255 }
    : { r: 30, g: 30, b: 40, a: 255 }
  ),
  border: 1,
  children: () => {
    text({ content: () => `Width: ${width.value}` })
  }
})

// Later: update triggers re-render
width.value = 60
isExpanded.value = true
```

### Grid Layout

```ts
import { box, text } from 'spark-tui'

// 3-column grid with fractional units
box({
  display: 'grid',
  gridTemplateColumns: ['1fr', '2fr', '1fr'],
  gridTemplateRows: ['auto', 'auto'],
  gap: 1,
  children: () => {
    text({ content: 'Cell 1' })
    text({ content: 'Cell 2 (wider)' })
    text({ content: 'Cell 3' })
    text({ content: 'Cell 4', gridColumnStart: 1, gridColumnEnd: 'span 2' })
    text({ content: 'Cell 5' })
  }
})
```

### Scrollable Container

```ts
import { box, text, each } from 'spark-tui'

const items = Array.from({ length: 100 }, (_, i) => `Line ${i + 1}`)

// Scrollable with keyboard navigation
box({
  width: 40,
  height: 10,
  overflow: 'scroll',
  border: 1,
  focusable: true,  // Enables keyboard scrolling when focused
  children: () => {
    each(
      () => items,
      (getItem) => text({ content: () => getItem() })
    )
  }
})
```

### Interactive Button

```ts
import { signal } from '@rlabs-inc/signals'
import { box, text } from 'spark-tui'
import { isEnter, isSpace } from 'spark-tui/events'

const count = signal(0)

box({
  width: 15,
  height: 3,
  bg: { r: 80, g: 160, b: 80, a: 255 },
  border: 1,
  focusable: true,
  justifyContent: 'center',
  alignItems: 'center',
  onClick: () => { count.value++ },
  onKey: (event) => {
    if (isEnter(event) || isSpace(event)) {
      count.value++
      return true  // Consume the event
    }
  },
  children: () => {
    text({ content: () => `Count: ${count.value}` })
  }
})
```

### Styled with Variants

```ts
import { box, text } from 'spark-tui'

// Using theme variants for consistent styling
box({
  variant: 'primary',
  padding: 2,
  border: 1,
  children: () => {
    text({ content: 'Primary styled box' })
  }
})

box({
  variant: 'error',
  padding: 2,
  border: 1,
  children: () => {
    text({ content: 'Error styled box' })
  }
})
```

## Props Reference

### Core Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `id` | `string` | auto-generated | Optional component identifier |
| `visible` | `Reactive<boolean>` | `true` | Whether the box is visible |
| `children` | `() => void` | - | Function that renders child components |
| `variant` | `Variant` | `'default'` | Theme variant for automatic styling |

### Display & Layout Mode

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `display` | `Reactive<'flex' \| 'grid' \| 'none'>` | `'flex'` | Layout mode |

### Dimension Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `width` | `Reactive<Dimension>` | auto | Width in cells or percentage (e.g., `50`, `'100%'`) |
| `height` | `Reactive<Dimension>` | auto | Height in cells or percentage |
| `minWidth` | `Reactive<Dimension>` | - | Minimum width constraint |
| `maxWidth` | `Reactive<Dimension>` | - | Maximum width constraint |
| `minHeight` | `Reactive<Dimension>` | - | Minimum height constraint |
| `maxHeight` | `Reactive<Dimension>` | - | Maximum height constraint |

### Flexbox Container Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `flexDirection` | `Reactive<'column' \| 'row' \| 'column-reverse' \| 'row-reverse'>` | `'column'` | Main axis direction |
| `flexWrap` | `Reactive<'nowrap' \| 'wrap' \| 'wrap-reverse'>` | `'nowrap'` | Whether items wrap |
| `justifyContent` | `Reactive<'flex-start' \| 'center' \| 'flex-end' \| 'space-between' \| 'space-around' \| 'space-evenly'>` | `'flex-start'` | Main axis alignment |
| `alignItems` | `Reactive<'stretch' \| 'flex-start' \| 'center' \| 'flex-end' \| 'baseline'>` | `'stretch'` | Cross axis alignment |
| `alignContent` | `Reactive<'flex-start' \| 'center' \| 'flex-end' \| 'space-between' \| 'space-around' \| 'space-evenly' \| 'stretch'>` | `'flex-start'` | Multi-line cross axis alignment |

### Flexbox Item Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `grow` | `Reactive<number>` | `0` | Flex grow factor |
| `shrink` | `Reactive<number>` | `1` | Flex shrink factor |
| `flexBasis` | `Reactive<number>` | auto | Initial main size |
| `alignSelf` | `Reactive<'auto' \| 'stretch' \| 'flex-start' \| 'center' \| 'flex-end' \| 'baseline'>` | `'auto'` | Override `alignItems` for this item |

### Grid Container Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `gridTemplateColumns` | `Reactive<GridTemplate>` | - | Column track definitions |
| `gridTemplateRows` | `Reactive<GridTemplate>` | - | Row track definitions |
| `gridAutoFlow` | `Reactive<'row' \| 'column' \| 'row dense' \| 'column dense'>` | `'row'` | Auto-placement algorithm |
| `gridAutoColumns` | `Reactive<GridTrackSize>` | - | Size of auto-generated columns |
| `gridAutoRows` | `Reactive<GridTrackSize>` | - | Size of auto-generated rows |
| `justifyItems` | `Reactive<'start' \| 'end' \| 'center' \| 'stretch'>` | `'start'` | Default justify for grid items |

### Grid Item Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `gridColumnStart` | `Reactive<GridLine>` | `'auto'` | Column start line (1-based or `'span N'`) |
| `gridColumnEnd` | `Reactive<GridLine>` | `'auto'` | Column end line |
| `gridRowStart` | `Reactive<GridLine>` | `'auto'` | Row start line |
| `gridRowEnd` | `Reactive<GridLine>` | `'auto'` | Row end line |
| `justifySelf` | `Reactive<'auto' \| 'start' \| 'end' \| 'center' \| 'stretch'>` | `'auto'` | Override `justifyItems` for this item |

### Spacing Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `padding` | `Reactive<number>` | `0` | Padding on all sides |
| `paddingTop` | `Reactive<number>` | - | Top padding (overrides `padding`) |
| `paddingRight` | `Reactive<number>` | - | Right padding |
| `paddingBottom` | `Reactive<number>` | - | Bottom padding |
| `paddingLeft` | `Reactive<number>` | - | Left padding |
| `margin` | `Reactive<number>` | `0` | Margin on all sides |
| `marginTop` | `Reactive<number>` | - | Top margin (overrides `margin`) |
| `marginRight` | `Reactive<number>` | - | Right margin |
| `marginBottom` | `Reactive<number>` | - | Bottom margin |
| `marginLeft` | `Reactive<number>` | - | Left margin |
| `gap` | `Reactive<number>` | `0` | Gap between children |
| `rowGap` | `Reactive<number>` | - | Gap between rows (overrides `gap`) |
| `columnGap` | `Reactive<number>` | - | Gap between columns (overrides `gap`) |

### Style Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `fg` | `Reactive<ColorInput>` | inherited | Foreground (text) color |
| `bg` | `Reactive<ColorInput>` | transparent | Background color |
| `opacity` | `Reactive<number>` | `1` | Opacity (0-1) |
| `zIndex` | `Reactive<number>` | `0` | Stacking order |
| `overflow` | `Reactive<'visible' \| 'hidden' \| 'scroll' \| 'auto'>` | `'visible'` | Overflow behavior |

**ColorInput** accepts flexible formats: hex strings (`'#ff0000'`, `'#f00'`), CSS names (`'red'`, `'dodgerblue'`), RGB/RGBA (`'rgb(255,0,0)'`, `'rgba(255,0,0,0.5)'`), HSL (`'hsl(0,100%,50%)'`), OKLCH (`'oklch(0.7 0.15 30)'`), integers (`0xff0000`), RGBA objects, or `null` for terminal default.

### Border Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `border` | `Reactive<number>` | `0` | Border style (0=none, 1=single, 2=double, 3=rounded, etc.) |
| `borderTop` | `Reactive<number>` | - | Top border style |
| `borderRight` | `Reactive<number>` | - | Right border style |
| `borderBottom` | `Reactive<number>` | - | Bottom border style |
| `borderLeft` | `Reactive<number>` | - | Left border style |
| `borderColor` | `Reactive<ColorInput>` | - | Border color (see ColorInput above) |

**Border Style Values:**
- `0` - None
- `1` - Single (default box drawing)
- `2` - Double
- `3` - Rounded
- `4` - Bold
- `5` - Dashed
- `6` - Dotted
- `7` - ASCII (`-`, `|`, `+`)
- `8` - Block (filled)

### Interaction Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `focusable` | `Reactive<boolean>` | `false` | Whether the box can receive focus |
| `tabIndex` | `Reactive<number>` | `-1` | Tab order (-1 = not in tab order) |

### Event Props

| Prop | Type | Description |
|------|------|-------------|
| `onKey` | `(event: KeyEvent) => boolean \| void` | Keyboard handler (fires when focused). Return `true` to consume. |
| `onFocus` | `() => void` | Called when box receives focus |
| `onBlur` | `() => void` | Called when box loses focus |
| `onClick` | `(event: MouseEvent) => void \| boolean` | Called on click (down + up). Return `true` to consume. |
| `onMouseDown` | `(event: MouseEvent) => void \| boolean` | Called on mouse button down |
| `onMouseUp` | `(event: MouseEvent) => void \| boolean` | Called on mouse button up |
| `onMouseEnter` | `(event: MouseEvent) => void` | Called when mouse enters box |
| `onMouseLeave` | `(event: MouseEvent) => void` | Called when mouse leaves box |
| `onScroll` | `(event: ScrollEvent) => void` | Called on scroll (mouse wheel or keyboard) |

## Types

### GridTrackSize

```ts
type GridTrackSize =
  | number           // Fixed size in terminal cells
  | `${number}%`     // Percentage of container
  | `${number}fr`    // Fractional unit
  | 'auto'           // Auto sizing
  | 'min-content'    // Minimum content size
  | 'max-content'    // Maximum content size
```

### GridLine

```ts
type GridLine = number | `span ${number}` | 'auto'
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
box({ bg: '#ff0000' })           // Hex
box({ bg: 'dodgerblue' })        // CSS name
box({ bg: 'oklch(0.7 0.15 300)' }) // OKLCH (perceptually uniform)
box({ bg: 0xff0000 })            // Integer
box({ bg: null })                // Terminal default
```

### Dimension

```ts
type Dimension = number | `${number}%`
```

## Notes

- **Flexbox default**: The default `flexDirection` is `'column'` (TUI convention), unlike CSS which defaults to `'row'`.
- **Auto-focusable scrolling**: When `overflow: 'scroll'` is set, the box automatically becomes focusable for keyboard scrolling.
- **Reactive props**: Any prop marked `Reactive<T>` accepts static values, signals, deriveds, bindings, or getter functions.
- **Children function**: The `children` prop is a function, not JSX. This enables proper scope management.

## See Also

- [text](./text.md) - Text display component
- [input](./input.md) - Text input component
- [each](../control-flow/each.md) - List rendering
- [show](../control-flow/show.md) - Conditional rendering
- [Theming](../theming/index.md) - Theme system and variants
