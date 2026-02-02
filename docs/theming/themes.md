# Theming System

> A reactive theme system with semantic colors, OKLCH support, and terminal default fallbacks.

## Import

```ts
import {
  theme,
  setTheme,
  themes,
  getThemeNames,
  t,
  resolvedTheme,
  resolveColor,
  getVariantStyle,
  variantStyle
} from 'spark-tui';
```

## Overview

SparkTUI's theming system is **fully reactive**. When you change a theme color, all components using that color automatically re-render. No manual updates needed.

The theme system supports three color formats:
- **Terminal default** (`null`): Respects user's terminal color scheme
- **ANSI colors** (0-255): Uses terminal's color palette
- **RGB colors** (0xRRGGBB or CSS strings): Exact color specification

## The `theme` Object

The `theme` object is a reactive state that holds all theme colors:

```ts
import { theme } from 'spark-tui';

// Read current theme colors
console.log(theme.primary);  // 12 (ANSI bright blue)
console.log(theme.text);     // null (terminal default)

// Modify theme reactively
theme.primary = 0x7aa2f7;    // Change to RGB color
theme.success = 'oklch(0.8 0.2 140)';  // Use OKLCH
```

### Theme Properties

#### Main Palette

| Property | Default | Description |
|----------|---------|-------------|
| `primary` | `12` | Primary brand color (ANSI bright blue) |
| `secondary` | `13` | Secondary accent (ANSI bright magenta) |
| `tertiary` | `14` | Tertiary color (ANSI bright cyan) |
| `accent` | `11` | Highlights (ANSI bright yellow) |

#### Semantic Colors

| Property | Default | Description |
|----------|---------|-------------|
| `success` | `2` | Success/positive states (ANSI green) |
| `warning` | `3` | Warning/caution states (ANSI yellow) |
| `error` | `1` | Error/danger states (ANSI red) |
| `info` | `6` | Informational states (ANSI cyan) |

#### Text Colors

| Property | Default | Description |
|----------|---------|-------------|
| `text` | `null` | Primary text (terminal default) |
| `textMuted` | `8` | Muted text (ANSI bright black/gray) |
| `textDim` | `8` | Dimmed text |
| `textDisabled` | `8` | Disabled text |
| `textBright` | `15` | Emphasized text (ANSI bright white) |

#### Background Colors

| Property | Default | Description |
|----------|---------|-------------|
| `background` | `null` | Primary background (terminal default) |
| `backgroundMuted` | `null` | Muted background |
| `surface` | `null` | Surface (cards, panels) |
| `overlay` | `null` | Overlay (modals) |

#### Metadata

| Property | Description |
|----------|-------------|
| `name` | Theme name string |
| `description` | Theme description |

## The `t` Object (Easy Theme Access)

The `t` object provides reactive deriveds for each theme color, resolved to RGBA:

```ts
import { t } from 'spark-tui';
import { box, text } from 'spark-tui';

// Use theme colors directly in components
box({
  bg: t.surface,
  borderColor: t.primary,
  children: () => {
    text({
      content: 'Themed text',
      fg: t.text
    });
  }
});
```

### Available `t` Properties

```ts
// Main palette
t.primary
t.secondary
t.tertiary
t.accent

// Semantic
t.success
t.warning
t.error
t.info

// Text
t.text
t.textMuted
t.textDim
t.textDisabled
t.textBright

// Backgrounds
t.bg         // Maps to theme.background
t.bgMuted    // Maps to theme.backgroundMuted
t.surface
t.overlay
```

## `setTheme()` Function

Apply a built-in theme or a custom theme object:

```ts
import { setTheme } from 'spark-tui';

// Apply built-in theme by name
setTheme('dracula');

// Apply custom theme (partial update)
setTheme({
  primary: 0x7aa2f7,
  secondary: 0xbb9af7,
  background: 0x1a1b26,
});

// Mix built-in with overrides
setTheme('catppuccin');
theme.primary = 0xff0000;  // Override just primary
```

## Built-in Themes

### `terminal` (Default)

Uses ANSI colors to respect the user's terminal theme. Best choice for CLI tools that should blend with the user's environment.

```ts
setTheme('terminal');
```

### `dracula`

Dark theme with vivid, high-contrast colors. Uses OKLCH for perceptually uniform colors.

```ts
setTheme('dracula');
// primary: 'oklch(0.75 0.15 300)' (purple)
// background: 0x282a36
```

### `nord`

Arctic, bluish color scheme with a calm aesthetic.

```ts
setTheme('nord');
// primary: 'oklch(0.8 0.08 210)' (frost cyan)
// background: 0x2e3440
```

### `monokai`

Vibrant syntax-highlighting inspired theme.

```ts
setTheme('monokai');
// primary: 'oklch(0.65 0.25 350)' (pink)
// background: 0x272822
```

### `solarized`

Precision color scheme with carefully balanced contrast.

```ts
setTheme('solarized');
// primary: 0x268bd2 (blue)
// background: 0x002b36 (base03)
```

### `catppuccin`

Soothing pastel theme (Mocha variant).

```ts
setTheme('catppuccin');
// primary: 0x89b4fa (blue)
// background: 0x1e1e2e (base)
```

### `gruvbox`

Retro groove color scheme with warm tones.

```ts
setTheme('gruvbox');
// primary: 0x458588 (blue)
// background: 0x282828
```

### `tokyoNight`

Clean, dark theme inspired by Tokyo city lights.

```ts
setTheme('tokyoNight');
// primary: 0x7aa2f7 (blue)
// background: 0x1a1b26
```

### `oneDark`

Atom's iconic dark theme.

```ts
setTheme('oneDark');
// primary: 0x61afef (blue)
// background: 0x282c34
```

### `rosePine`

All natural pine, faux fur and a bit of soho vibes.

```ts
setTheme('rosePine');
// primary: 0x9ccfd8 (foam)
// background: 0x191724 (base)
```

### `kanagawa`

Theme inspired by Katsushika Hokusai's famous wave painting.

```ts
setTheme('kanagawa');
// primary: 0x7e9cd8 (crystalBlue)
// background: 0x1f1f28 (sumiInk1)
```

### `everforest`

Comfortable green-tinted theme.

```ts
setTheme('everforest');
// primary: 0x7fbbb3 (aqua)
// background: 0x2d353b (bg_dim)
```

### `nightOwl`

Designed with accessibility in mind.

```ts
setTheme('nightOwl');
// primary: 0x82aaff (blue)
// background: 0x011627
```

## `getThemeNames()` Function

Get a list of all available built-in theme names:

```ts
import { getThemeNames } from 'spark-tui';

const themes = getThemeNames();
// ['terminal', 'dracula', 'nord', 'monokai', 'solarized',
//  'catppuccin', 'gruvbox', 'tokyoNight', 'oneDark',
//  'rosePine', 'kanagawa', 'everforest', 'nightOwl']
```

## `resolveColor()` Function

Resolve a theme color to RGBA:

```ts
import { resolveColor } from 'spark-tui';

// Terminal default -> special marker
resolveColor(null);        // { r: -1, g: -1, b: -1, a: 255 }

// ANSI colors -> ANSI marker
resolveColor(12);          // { r: -2, g: 12, b: 0, a: 255 }

// RGB hex -> RGBA
resolveColor(0xff5500);    // { r: 255, g: 85, b: 0, a: 255 }

// CSS color string -> RGBA
resolveColor('oklch(0.7 0.15 200)');  // { r: 93, g: 178, b: 191, a: 255 }
```

## `resolvedTheme` Derived

A derived that contains all theme colors resolved to RGBA. Useful for color blending or manual calculations:

```ts
import { resolvedTheme } from 'spark-tui';

const colors = resolvedTheme.value;
console.log(colors.primary);   // RGBA object
console.log(colors.text);      // RGBA object
```

## Variants

Variants provide pre-configured color combinations for common UI patterns:

```ts
import { getVariantStyle, variantStyle } from 'spark-tui';
import type { Variant } from 'spark-tui';

// Available variants
type Variant =
  | 'default'
  | 'primary' | 'secondary' | 'tertiary' | 'accent'
  | 'success' | 'warning' | 'error' | 'info'
  | 'muted' | 'surface' | 'elevated'
  | 'ghost' | 'outline';
```

### `getVariantStyle()` Function

Get variant colors for the current theme:

```ts
import { getVariantStyle } from 'spark-tui';

const style = getVariantStyle('primary');
// Returns: { fg, bg, border, borderFocus }

box({
  fg: style.fg,
  bg: style.bg,
  borderColor: style.border,
});
```

### `variantStyle()` Derived

Create a reactive derived for a variant:

```ts
import { variantStyle } from 'spark-tui';

const primaryStyle = variantStyle('primary');

// Use in component - automatically updates with theme changes
box({
  fg: primaryStyle.value.fg,
  bg: primaryStyle.value.bg,
});
```

### Variant Descriptions

| Variant | Description |
|---------|-------------|
| `default` | Standard colors (text on background) |
| `primary` | Primary brand (bright text on primary bg) |
| `secondary` | Secondary accent colors |
| `tertiary` | Tertiary accent colors |
| `accent` | Highlight/accent (dark text on accent bg) |
| `success` | Success state (bright text on success bg) |
| `warning` | Warning state (dark text on warning bg) |
| `error` | Error state (bright text on error bg) |
| `info` | Informational state |
| `muted` | Subdued appearance |
| `surface` | Card/panel styling |
| `elevated` | Elevated surface with shadow effect |
| `ghost` | Transparent background |
| `outline` | Outlined style (transparent bg, colored border) |

## Examples

### Theme Switcher

```ts
import { mount, box, text, setTheme, getThemeNames, theme } from 'spark-tui';
import { state } from '@rlabs-inc/signals';

await mount(() => {
  const currentTheme = state(0);
  const themes = getThemeNames();

  box({
    onKey: (e) => {
      if (e.key === 'n') {
        currentTheme.value = (currentTheme.value + 1) % themes.length;
        setTheme(themes[currentTheme.value]);
      }
    },
    focusable: true,
    children: () => {
      text({ content: () => `Theme: ${theme.name} (press 'n' for next)` });
    }
  });
});
```

### Custom Theme

```ts
import { mount, setTheme, box, text, t } from 'spark-tui';

// Create a custom theme
setTheme({
  name: 'myTheme',
  description: 'My custom theme',
  primary: 'oklch(0.7 0.2 250)',
  secondary: 'oklch(0.7 0.2 320)',
  text: 0xe0e0e0,
  background: 0x1a1a2e,
  surface: 0x2a2a4e,
});

await mount(() => {
  box({
    bg: t.surface,
    padding: 2,
    border: 1,
    borderColor: t.primary,
    children: () => {
      text({ content: 'Custom themed content!', fg: t.text });
    }
  });
});
```

### Using Variants

```ts
import { mount, box, text, getVariantStyle } from 'spark-tui';

await mount(() => {
  box({ flexDirection: 'row', gap: 2, children: () => {
    for (const variant of ['primary', 'success', 'error', 'warning'] as const) {
      const style = getVariantStyle(variant);
      box({
        fg: style.fg,
        bg: style.bg,
        padding: 1,
        border: 1,
        borderColor: style.border,
        children: () => {
          text({ content: variant });
        }
      });
    }
  }});
});
```

## Color Value Types

### ThemeColor Type

```ts
type ThemeColor = null | number | string;
```

| Value | Meaning | Example |
|-------|---------|---------|
| `null` | Terminal default | `theme.text = null` |
| `0-15` | ANSI color index | `theme.error = 1` (red) |
| `16-255` | Extended 256-color | `theme.accent = 208` (orange) |
| `> 255` | RGB color | `theme.primary = 0x7aa2f7` |
| `string` | CSS color | `theme.success = 'oklch(0.8 0.2 140)'` |

### OKLCH Colors

SparkTUI supports OKLCH colors for perceptually uniform color manipulation:

```ts
// OKLCH format: oklch(L C H) or oklch(L C H / A)
// L: Lightness (0-1)
// C: Chroma (0-0.4 typically)
// H: Hue (0-360 degrees)

theme.primary = 'oklch(0.7 0.15 200)';  // Nice cyan
theme.accent = 'oklch(0.85 0.2 90)';    // Vibrant yellow
```

## See Also

- [Colors and Color Utilities](/docs/api-reference/types.md#rgba)
- [Box Component](/docs/components/box.md) - Using theme colors in components
- [Text Component](/docs/components/text.md) - Text color styling
