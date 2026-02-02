/**
 * SparkTUI Theme Switcher
 *
 * Beautiful interactive theme showcase demonstrating:
 * - All 13 built-in themes with live preview
 * - Color palette visualization (primary, secondary, success, etc.)
 * - Text styles (text, textMuted, textDim, textBright)
 * - Arrow key navigation with instant theme switching
 *
 * Navigation:
 * - Up/Down: Cycle through themes
 * - Enter: Apply theme
 * - Ctrl+C: Exit
 */

import { signal, derived } from '@rlabs-inc/signals'
import { mount } from '../ts/engine/mount'
import { box, text, each } from '../ts/primitives'
import {
  theme,
  themes,
  setTheme,
  getThemeNames,
  t,
  resolveColor,
} from '../ts/state/theme'
import { on } from '../ts/state/keyboard'
import type { KeyEvent } from '../ts/state/keyboard'

// =============================================================================
// STATE
// =============================================================================

const themeNames = getThemeNames()
const selectedIndex = signal(0)
const currentThemeName = derived(() => themeNames[selectedIndex.value]!)

// Color categories for display
const mainColors = [
  { key: 'primary', label: 'Primary' },
  { key: 'secondary', label: 'Secondary' },
  { key: 'tertiary', label: 'Tertiary' },
  { key: 'accent', label: 'Accent' },
] as const

const semanticColors = [
  { key: 'success', label: 'Success' },
  { key: 'warning', label: 'Warning' },
  { key: 'error', label: 'Error' },
  { key: 'info', label: 'Info' },
] as const

const textColors = [
  { key: 'text', label: 'Text' },
  { key: 'textMuted', label: 'Muted' },
  { key: 'textDim', label: 'Dim' },
  { key: 'textBright', label: 'Bright' },
] as const

const bgColors = [
  { key: 'background', label: 'Background' },
  { key: 'backgroundMuted', label: 'BG Muted' },
  { key: 'surface', label: 'Surface' },
  { key: 'overlay', label: 'Overlay' },
] as const

// =============================================================================
// HELPERS
// =============================================================================

function colorSwatch(color: { key: string; label: string }, width: number = 12) {
  box({
    id: `swatch-${color.key}`,
    width,
    height: 3,
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    border: 1,
    borderColor: t.textMuted,
    bg: () => resolveColor((theme as any)[color.key]),
    children: () => {
      text({
        content: color.label,
        fg: () => {
          // Smart contrast: use dark text on light colors, light text on dark
          const c = (theme as any)[color.key]
          return c === null ? resolveColor(theme.textBright) : resolveColor(theme.textBright)
        },
      })
    },
  })
}

function textStyleRow(color: { key: string; label: string }) {
  box({
    id: `text-style-${color.key}`,
    flexDirection: 'row',
    alignItems: 'center',
    gap: 2,
    height: 1,
    children: () => {
      text({
        content: `${color.label}:`,
        width: 10,
        fg: t.textMuted,
      })
      text({
        content: 'The quick brown fox jumps over the lazy dog',
        fg: () => resolveColor((theme as any)[color.key]),
      })
    },
  })
}

// =============================================================================
// MOUNT APP
// =============================================================================

const cols = process.stdout.columns || 100
const rows = process.stdout.rows || 35

const { unmount } = mount(() => {
  // Keyboard navigation
  on((event: KeyEvent) => {
    const keycode = event.keycode

    // Arrow Up
    if (keycode === 0x1b5b41) {
      selectedIndex.value = Math.max(0, selectedIndex.value - 1)
      setTheme(themeNames[selectedIndex.value]! as keyof typeof themes)
      return true
    }

    // Arrow Down
    if (keycode === 0x1b5b42) {
      selectedIndex.value = Math.min(themeNames.length - 1, selectedIndex.value + 1)
      setTheme(themeNames[selectedIndex.value]! as keyof typeof themes)
      return true
    }

    // Enter
    if (keycode === 13) {
      setTheme(themeNames[selectedIndex.value]! as keyof typeof themes)
      return true
    }

    return false
  })

  // Root container
  box({
    id: 'root',
    width: cols,
    height: rows,
    flexDirection: 'column',
    bg: t.bg,
    fg: t.text,
    children: () => {
      // =========================================================================
      // HEADER
      // =========================================================================
      box({
        id: 'header',
        width: '100%',
        height: 5,
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        border: 2,
        borderColor: t.primary,
        bg: t.surface,
        children: () => {
          text({
            content: ' SparkTUI Theme Switcher ',
            fg: t.primary,
          })
          text({
            content: () => `${themeNames.length} built-in themes - Arrow keys to browse`,
            fg: t.textMuted,
          })
        },
      })

      // =========================================================================
      // MAIN CONTENT
      // =========================================================================
      box({
        id: 'main',
        width: '100%',
        grow: 1,
        flexDirection: 'row',
        children: () => {
          // =====================================================================
          // LEFT PANEL: Theme List
          // =====================================================================
          box({
            id: 'theme-list',
            width: 24,
            height: '100%',
            flexDirection: 'column',
            border: 1,
            borderColor: t.textMuted,
            children: () => {
              text({
                content: ' Themes',
                fg: t.textBright,
                bg: t.surface,
                padding: 1,
              })

              each(
                () => themeNames.map((name, i) => ({ name, index: i })),
                (getItem, key) => {
                  return box({
                    id: `theme-item-${key}`,
                    width: '100%',
                    height: 1,
                    flexDirection: 'row',
                    paddingLeft: 1,
                    bg: () => {
                      const item = getItem()
                      return item.index === selectedIndex.value
                        ? resolveColor(theme.primary)
                        : resolveColor(theme.background)
                    },
                    fg: () => {
                      const item = getItem()
                      return item.index === selectedIndex.value
                        ? resolveColor(theme.textBright)
                        : resolveColor(theme.text)
                    },
                    onClick: () => {
                      const item = getItem()
                      selectedIndex.value = item.index
                      setTheme(item.name as keyof typeof themes)
                    },
                    children: () => {
                      text({
                        content: () => getItem().index === selectedIndex.value ? '>' : ' ',
                      })
                      text({
                        content: () => ` ${getItem().name}`,
                      })
                    },
                  })
                },
                { key: (item) => item.name }
              )
            },
          })

          // =====================================================================
          // RIGHT PANEL: Preview
          // =====================================================================
          box({
            id: 'preview-panel',
            grow: 1,
            height: '100%',
            flexDirection: 'column',
            padding: 1,
            children: () => {
              // Theme name display
              box({
                id: 'theme-name-display',
                width: '100%',
                height: 3,
                flexDirection: 'column',
                alignItems: 'center',
                justifyContent: 'center',
                marginBottom: 1,
                children: () => {
                  text({
                    content: () => `Theme: ${currentThemeName.value}`,
                    fg: t.textBright,
                  })
                  text({
                    content: () => (themes as any)[currentThemeName.value]?.description || '',
                    fg: t.textMuted,
                  })
                },
              })

              // -----------------------------------------------------------------
              // MAIN PALETTE
              // -----------------------------------------------------------------
              box({
                id: 'main-palette-section',
                width: '100%',
                flexDirection: 'column',
                marginBottom: 1,
                children: () => {
                  text({
                    content: 'Main Palette',
                    fg: t.textMuted,
                    marginBottom: 1,
                  })
                  box({
                    id: 'main-palette-row',
                    flexDirection: 'row',
                    gap: 1,
                    children: () => {
                      for (const color of mainColors) {
                        colorSwatch(color, 14)
                      }
                    },
                  })
                },
              })

              // -----------------------------------------------------------------
              // SEMANTIC COLORS
              // -----------------------------------------------------------------
              box({
                id: 'semantic-section',
                width: '100%',
                flexDirection: 'column',
                marginBottom: 1,
                children: () => {
                  text({
                    content: 'Semantic Colors',
                    fg: t.textMuted,
                    marginBottom: 1,
                  })
                  box({
                    id: 'semantic-row',
                    flexDirection: 'row',
                    gap: 1,
                    children: () => {
                      for (const color of semanticColors) {
                        colorSwatch(color, 14)
                      }
                    },
                  })
                },
              })

              // -----------------------------------------------------------------
              // BACKGROUND COLORS
              // -----------------------------------------------------------------
              box({
                id: 'bg-section',
                width: '100%',
                flexDirection: 'column',
                marginBottom: 1,
                children: () => {
                  text({
                    content: 'Background Colors',
                    fg: t.textMuted,
                    marginBottom: 1,
                  })
                  box({
                    id: 'bg-row',
                    flexDirection: 'row',
                    gap: 1,
                    children: () => {
                      for (const color of bgColors) {
                        colorSwatch(color, 14)
                      }
                    },
                  })
                },
              })

              // -----------------------------------------------------------------
              // TEXT STYLES
              // -----------------------------------------------------------------
              box({
                id: 'text-styles-section',
                width: '100%',
                flexDirection: 'column',
                border: 1,
                borderColor: t.textMuted,
                padding: 1,
                bg: t.surface,
                children: () => {
                  text({
                    content: 'Text Styles Preview',
                    fg: t.textBright,
                    marginBottom: 1,
                  })

                  for (const color of textColors) {
                    textStyleRow(color)
                  }
                },
              })
            },
          })
        },
      })

      // =========================================================================
      // FOOTER
      // =========================================================================
      box({
        id: 'footer',
        width: '100%',
        height: 1,
        flexDirection: 'row',
        justifyContent: 'center',
        alignItems: 'center',
        bg: t.surface,
        children: () => {
          text({
            content: 'Up/Down: Navigate | Enter: Apply | Ctrl+C: Exit',
            fg: t.textMuted,
          })
        },
      })
    },
  })
}, {
  mode: 'fullscreen',
})

console.log('[theme-switcher] App mounted')

// Keep process alive
await new Promise(() => {})
