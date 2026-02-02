# input

> Single-line text input with full reactivity, cursor customization, and keyboard navigation.

The `input` component provides a fully-featured text input field with two-way binding, cursor navigation, password masking, placeholder text, and extensive cursor customization options.

## Import

```ts
import { input } from 'spark-tui'
// or
import { input } from '../ts/primitives'
```

## Signature

```ts
function input(props: InputProps): Cleanup
```

## Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| props | `InputProps` | Yes | - | Configuration object (requires `value`) |

Returns a `Cleanup` function that unmounts the component when called.

## Examples

### Basic Usage

```ts
import { signal } from '@rlabs-inc/signals'
import { input } from 'spark-tui'

const name = signal('')

// Simple text input
input({
  value: name,
  placeholder: 'Enter your name...',
  width: 30,
})
```

### With Border and Styling

```ts
import { signal } from '@rlabs-inc/signals'
import { input } from 'spark-tui'

const email = signal('')

input({
  value: email,
  placeholder: 'you@example.com',
  width: 40,
  border: 1,
  borderColor: { r: 80, g: 160, b: 240, a: 255 },
  fg: { r: 255, g: 255, b: 255, a: 255 },
  padding: 0,
  paddingLeft: 1,
})
```

### Password Input

```ts
import { signal } from '@rlabs-inc/signals'
import { input } from 'spark-tui'

const password = signal('')

input({
  value: password,
  placeholder: 'Enter password...',
  password: true,           // Masks characters
  maskChar: '*',            // Custom mask character (default: bullet)
  width: 30,
  border: 1,
})
```

### With Event Handlers

```ts
import { signal } from '@rlabs-inc/signals'
import { box, text, input } from 'spark-tui'

const searchQuery = signal('')
const results = signal<string[]>([])

box({
  flexDirection: 'column',
  gap: 1,
  children: () => {
    input({
      value: searchQuery,
      placeholder: 'Search...',
      width: 40,
      border: 1,
      autoFocus: true,
      onChange: (value) => {
        // Live filtering as you type
        results.value = filterItems(value)
      },
      onSubmit: (value) => {
        // Triggered on Enter key
        console.log('Search submitted:', value)
        performSearch(value)
      },
      onCancel: () => {
        // Triggered on Escape key
        searchQuery.value = ''
      },
    })

    text({ content: () => `Results: ${results.value.length}` })
  }
})
```

### Focus Management

```ts
import { signal } from '@rlabs-inc/signals'
import { box, input, text } from 'spark-tui'

const username = signal('')
const password = signal('')
const currentField = signal<'username' | 'password'>('username')

// Input with focus callbacks
input({
  value: username,
  placeholder: 'Username',
  width: 30,
  autoFocus: true,
  onFocus: () => { currentField.value = 'username' },
  onBlur: () => { /* Optional cleanup */ },
  onSubmit: () => {
    // Move focus to next field (Tab navigation handled automatically)
  },
})

input({
  value: password,
  placeholder: 'Password',
  password: true,
  width: 30,
  onFocus: () => { currentField.value = 'password' },
})

text({
  content: () => `Current field: ${currentField.value}`,
  fg: { r: 150, g: 150, b: 180, a: 255 },
})
```

### Cursor Styles

```ts
import { signal } from '@rlabs-inc/signals'
import { box, text, input } from 'spark-tui'

// Block cursor (default)
input({
  value: signal('Block cursor'),
  width: 30,
  cursor: { style: 'block' },
})

// Bar cursor (VS Code style)
input({
  value: signal('Bar cursor'),
  width: 30,
  cursor: { style: 'bar' },
})

// Underline cursor
input({
  value: signal('Underline cursor'),
  width: 30,
  cursor: { style: 'underline' },
})
```

### Cursor Blink Configuration

```ts
import { signal } from '@rlabs-inc/signals'
import { input } from 'spark-tui'

// Slow blink (1 FPS = 1 second cycle)
input({
  value: signal('Slow blink'),
  width: 30,
  cursor: { blink: { fps: 1 } },
})

// Normal blink (2 FPS = 500ms cycle)
input({
  value: signal('Normal blink'),
  width: 30,
  cursor: { blink: { fps: 2 } },
})

// Fast blink (4 FPS = 250ms cycle)
input({
  value: signal('Fast blink'),
  width: 30,
  cursor: { blink: { fps: 4 } },
})

// No blink (solid cursor)
input({
  value: signal('No blink'),
  width: 30,
  cursor: { blink: false },
})
```

### Custom Cursor Characters

```ts
import { signal } from '@rlabs-inc/signals'
import { input } from 'spark-tui'

// Thin bar cursor
input({
  value: signal('Thin bar'),
  width: 30,
  cursor: { char: '\u258F', blink: false },  // Left 1/8 block
})

// Arrow cursor
input({
  value: signal('Arrow cursor'),
  width: 30,
  cursor: { char: '\u25B6', blink: { fps: 2 } },  // Right triangle
})

// Star cursor
input({
  value: signal('Fun cursor'),
  width: 30,
  cursor: { char: '\u2605', blink: { fps: 3 } },  // Black star
})
```

### Colored Cursors

```ts
import { signal } from '@rlabs-inc/signals'
import { input, cycle } from 'spark-tui'

// Green cursor (Matrix style)
input({
  value: signal('Matrix style'),
  width: 30,
  cursor: {
    fg: { r: 0, g: 255, b: 100, a: 255 },
    bg: { r: 0, g: 100, b: 40, a: 255 },
    blink: { fps: 2 },
  },
})

// Red cursor (Error mode)
input({
  value: signal('Error mode'),
  width: 30,
  cursor: {
    fg: { r: 255, g: 80, b: 80, a: 255 },
    bg: { r: 100, g: 30, b: 30, a: 255 },
    blink: { fps: 4 },
  },
})

// Rainbow cursor (animated color)
input({
  value: signal('Party mode!'),
  width: 30,
  cursor: {
    fg: cycle(
      [
        { r: 255, g: 0, b: 0, a: 255 },     // Red
        { r: 255, g: 165, b: 0, a: 255 },   // Orange
        { r: 255, g: 255, b: 0, a: 255 },   // Yellow
        { r: 0, g: 255, b: 0, a: 255 },     // Green
        { r: 0, g: 100, b: 255, a: 255 },   // Blue
        { r: 150, g: 0, b: 255, a: 255 },   // Purple
      ],
      { fps: 8 }
    ),
    blink: false,
  },
})
```

### Max Length Validation

```ts
import { signal } from '@rlabs-inc/signals'
import { box, text, input } from 'spark-tui'

const username = signal('')

box({
  flexDirection: 'column',
  gap: 1,
  children: () => {
    input({
      value: username,
      placeholder: 'Username (max 20 chars)',
      maxLength: 20,
      width: 30,
      border: 1,
    })

    text({
      content: () => `${username.value.length}/20`,
      fg: () => username.value.length >= 20
        ? { r: 255, g: 100, b: 100, a: 255 }
        : { r: 150, g: 150, b: 180, a: 255 },
    })
  }
})
```

### Form Example

```ts
import { signal, derived } from '@rlabs-inc/signals'
import { box, text, input, show } from 'spark-tui'

const username = signal('')
const email = signal('')
const password = signal('')

const isValid = derived(() =>
  username.value.length >= 3 &&
  email.value.includes('@') &&
  password.value.length >= 8
)

box({
  flexDirection: 'column',
  gap: 1,
  border: 1,
  padding: 2,
  children: () => {
    text({ content: 'Create Account', fg: { r: 255, g: 255, b: 255, a: 255 } })

    // Username field
    box({
      flexDirection: 'row',
      gap: 2,
      children: () => {
        text({ content: 'Username:', width: 10 })
        input({
          value: username,
          placeholder: 'Enter username...',
          width: 25,
          border: 1,
          autoFocus: true,
        })
      }
    })

    // Email field
    box({
      flexDirection: 'row',
      gap: 2,
      children: () => {
        text({ content: 'Email:', width: 10 })
        input({
          value: email,
          placeholder: 'you@example.com',
          width: 25,
          border: 1,
        })
      }
    })

    // Password field
    box({
      flexDirection: 'row',
      gap: 2,
      children: () => {
        text({ content: 'Password:', width: 10 })
        input({
          value: password,
          placeholder: 'Enter password...',
          password: true,
          width: 25,
          border: 1,
          cursor: { style: 'bar', blink: { fps: 2 } },
        })
      }
    })

    // Submit button
    show(
      () => isValid.value,
      () => text({ content: 'Ready to submit!', fg: { r: 100, g: 255, b: 100, a: 255 } }),
      () => text({ content: 'Please fill all fields', fg: { r: 150, g: 150, b: 150, a: 255 } })
    )
  }
})
```

## Props Reference

### Core Props

| Prop | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| `id` | `string` | No | auto-generated | Optional component identifier |
| `value` | `WritableSignal<string> \| Binding<string>` | **Yes** | - | Two-way bound value (must be writable) |
| `visible` | `Reactive<boolean>` | No | `true` | Whether the input is visible |
| `variant` | `Variant` | No | `'default'` | Theme variant for automatic styling |

### Input Behavior Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `placeholder` | `string` | - | Placeholder text when empty |
| `placeholderColor` | `Reactive<ColorInput>` | theme muted | Color of placeholder text |
| `autoFocus` | `boolean` | `false` | Focus input on mount |
| `maxLength` | `number` | unlimited | Maximum input length (0 = unlimited) |
| `password` | `boolean` | `false` | Mask characters |
| `maskChar` | `string` | `'bullet'` | Character used for password masking |

### Cursor Configuration

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `cursor` | `CursorConfig` | see below | Cursor appearance configuration |

#### CursorConfig

```ts
interface CursorConfig {
  style?: 'block' | 'bar' | 'underline'  // Default: 'block'
  char?: string                          // Custom cursor character
  blink?: boolean | BlinkConfig          // Blink configuration
  fg?: Reactive<ColorInput>              // Cursor foreground color
  bg?: Reactive<ColorInput>              // Cursor background color
}

interface BlinkConfig {
  enabled?: boolean    // Default: true
  fps?: number         // Blink rate (default: 2)
  altChar?: string     // Character shown on "off" phase
}
```

**Cursor color examples:**
```ts
input({
  value: name,
  cursor: {
    fg: '#00ff00',           // Green cursor text
    bg: 'rgba(0,255,0,0.3)', // Semi-transparent green background
  },
})
```

### Text Alignment

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `align` | `Reactive<'left' \| 'center' \| 'right'>` | `'left'` | Text alignment |
| `attrs` | `Reactive<CellAttrs>` | `Attr.NONE` | Text attributes (bold, etc.) |

### Dimension Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `width` | `Reactive<Dimension>` | auto | Width in cells or percentage |
| `height` | `Reactive<Dimension>` | `1` | Height (typically 1 for single-line) |
| `minWidth` | `Reactive<Dimension>` | - | Minimum width constraint |
| `maxWidth` | `Reactive<Dimension>` | - | Maximum width constraint |
| `minHeight` | `Reactive<Dimension>` | - | Minimum height constraint |
| `maxHeight` | `Reactive<Dimension>` | - | Maximum height constraint |

### Style Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `fg` | `Reactive<ColorInput>` | theme text | Foreground (text) color |
| `bg` | `Reactive<ColorInput>` | transparent | Background color |
| `opacity` | `Reactive<number>` | `1` | Opacity (0-1) |
| `zIndex` | `Reactive<number>` | `0` | Stacking order |

**ColorInput** accepts flexible formats: hex strings (`'#ff0000'`, `'#f00'`), CSS names (`'red'`, `'dodgerblue'`), RGB/RGBA (`'rgb(255,0,0)'`, `'rgba(255,0,0,0.5)'`), HSL (`'hsl(0,100%,50%)'`), OKLCH (`'oklch(0.7 0.15 30)'`), integers (`0xff0000`), RGBA objects, or `null` for terminal default.

### Border Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `border` | `Reactive<number>` | `0` | Border style |
| `borderTop` | `Reactive<number>` | - | Top border style |
| `borderRight` | `Reactive<number>` | - | Right border style |
| `borderBottom` | `Reactive<number>` | - | Bottom border style |
| `borderLeft` | `Reactive<number>` | - | Left border style |
| `borderColor` | `Reactive<ColorInput>` | - | Border color (see ColorInput above) |

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
| `gap` | `Reactive<number>` | - | Gap (for layout) |
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

### Interaction Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `focusable` | `Reactive<boolean>` | `true` | Always focusable (inputs require focus) |
| `tabIndex` | `Reactive<number>` | auto | Tab order |

### Event Props

| Prop | Type | Description |
|------|------|-------------|
| `onChange` | `(value: string) => void` | Called on every value change |
| `onSubmit` | `(value: string) => void` | Called on Enter key |
| `onCancel` | `() => void` | Called on Escape key |
| `onFocus` | `() => void` | Called when input receives focus |
| `onBlur` | `() => void` | Called when input loses focus |
| `onClick` | `(event: MouseEvent) => void \| boolean` | Called on click |
| `onMouseDown` | `(event: MouseEvent) => void \| boolean` | Called on mouse button down |
| `onMouseUp` | `(event: MouseEvent) => void \| boolean` | Called on mouse button up |
| `onMouseEnter` | `(event: MouseEvent) => void` | Called when mouse enters |
| `onMouseLeave` | `(event: MouseEvent) => void` | Called when mouse leaves |
| `onScroll` | `(event: ScrollEvent) => void` | Called on scroll events |

## Keyboard Navigation

The input component handles the following keys automatically:

| Key | Action |
|-----|--------|
| `Arrow Left` | Move cursor left |
| `Arrow Right` | Move cursor right |
| `Home` | Move cursor to start |
| `End` | Move cursor to end |
| `Backspace` | Delete character before cursor |
| `Delete` | Delete character at cursor |
| `Enter` | Trigger `onSubmit` callback |
| `Escape` | Trigger `onCancel` callback |
| Printable chars | Insert at cursor position |

Tab navigation between inputs is handled by the framework automatically.

## Types

### CursorStyle

```ts
type CursorStyle = 'block' | 'bar' | 'underline'
```

### BlinkConfig

```ts
interface BlinkConfig {
  enabled?: boolean    // Default: true when BlinkConfig is used
  altChar?: string     // Character to show on "off" phase
  fps?: number         // Blink rate in FPS (default: 2)
}
```

### CursorConfig

```ts
interface CursorConfig {
  style?: CursorStyle           // Cursor shape preset
  char?: string                 // Custom cursor character
  blink?: boolean | BlinkConfig // Blink configuration
  fg?: Reactive<RGBA>           // Cursor foreground color
  bg?: Reactive<RGBA>           // Cursor background color
}
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
input({
  value: name,
  fg: 'white',
  bg: '#1a1a2e',
  borderColor: 'dodgerblue',
})
```

## Notes

- **Value is required**: The `value` prop is mandatory and must be a writable signal or binding.
- **Always focusable**: Input components are always focusable (required for keyboard input).
- **Clicking focuses**: Clicking on an input automatically focuses it.
- **Tab navigation**: Tab and Shift+Tab navigation is handled automatically by the framework.
- **Single-line only**: This component is for single-line input. For multi-line, use a custom scrollable box with text.
- **Reactive cursor colors**: Cursor `fg` and `bg` can be reactive for animated effects.

## See Also

- [box](./box.md) - Container component
- [text](./text.md) - Text display component
- [cycle](../animation/cycle.md) - Frame animation for cursor effects
- [Focus Management](../events/focus.md) - Focus system documentation
- [Theming](../theming/index.md) - Theme system and variants
