/**
 * SparkTUI Theme Gallery
 *
 * Interactive theme explorer showcasing:
 * - 13 theme presets (Terminal, Dracula, Nord, Tokyo Night, Gruvbox, etc.)
 * - 14 variants per theme (default, primary, secondary, success, error, etc.)
 * - Color palette visualization
 * - Sample UI components
 *
 * Navigation:
 * - Up/Down arrows: Select theme
 * - Tab: Cycle through UI sections
 * - Enter: Apply selected theme
 * - Ctrl+C: Exit
 */

import { signal, derived } from '@rlabs-inc/signals'
import { mount } from '../ts/engine/mount'
import { box, text, input, each } from '../ts/primitives'
import {
  theme,
  themes,
  setTheme,
  getVariantStyle,
  t,
  resolveColor,
  type Variant,
} from '../ts/state/theme'
import { on } from '../ts/state/keyboard'
import type { KeyEvent } from '../ts/state/keyboard'

// =============================================================================
// THEME DATA
// =============================================================================

const themeNames = Object.keys(themes) as (keyof typeof themes)[]

const variants: Variant[] = [
  'default', 'primary', 'secondary', 'tertiary', 'accent',
  'success', 'warning', 'error', 'info',
  'muted', 'surface', 'elevated', 'ghost', 'outline',
]

const colorSlots = [
  { key: 'primary', label: 'Pri' },
  { key: 'secondary', label: 'Sec' },
  { key: 'tertiary', label: 'Ter' },
  { key: 'accent', label: 'Acc' },
  { key: 'success', label: 'Suc' },
  { key: 'warning', label: 'Wrn' },
  { key: 'error', label: 'Err' },
  { key: 'info', label: 'Inf' },
  { key: 'text', label: 'Txt' },
  { key: 'textMuted', label: 'Mut' },
  { key: 'background', label: 'BG0' },
  { key: 'backgroundMuted', label: 'BG1' },
  { key: 'surface', label: 'Srf' },
  { key: 'border', label: 'Brd' },
] as const

// =============================================================================
// STATE
// =============================================================================

const currentThemeIndex = signal(0)
const currentThemeName = derived(() => themeNames[currentThemeIndex.value]!)

// Sample form state
const username = signal('john_doe')
const password = signal('secret123')

// =============================================================================
// MOUNT APP
// =============================================================================

const cols = process.stdout.columns || 120
const rows = process.stdout.rows || 40

const { unmount, setMode, getMode } = mount(() => {
  // Keyboard handler
  on((event: KeyEvent) => {
    const keycode = event.keycode

    // Arrow Up (ESC [ A)
    if (keycode === 0x1b5b41) {
      currentThemeIndex.value = Math.max(0, currentThemeIndex.value - 1)
      setTheme(themeNames[currentThemeIndex.value]!)
      return true
    }

    // Arrow Down (ESC [ B)
    if (keycode === 0x1b5b42) {
      currentThemeIndex.value = Math.min(themeNames.length - 1, currentThemeIndex.value + 1)
      setTheme(themeNames[currentThemeIndex.value]!)
      return true
    }

    // Enter - confirm theme
    if (keycode === 13) {
      setTheme(themeNames[currentThemeIndex.value]!)
      return true
    }

    return false
  })

  // Root: full terminal
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
      height: 3,
      flexDirection: 'row',
      justifyContent: 'space-between',
      alignItems: 'center',
      padding: 1,
      border: 1,
      borderColor: t.border,
      bg: t.surface,
      children: () => {
        text({
          id: 'title',
          content: 'SparkTUI Theme Gallery',
          fg: t.textBright,
        })
        text({
          id: 'current-theme',
          content: () => `Current: ${currentThemeName.value}`,
          fg: t.primary,
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
        // THEME SELECTOR (LEFT PANEL)
        // =====================================================================
        box({
          id: 'theme-list-panel',
          width: 20,
          height: '100%',
          flexDirection: 'column',
          border: 1,
          borderColor: t.border,
          children: () => {
            text({
              id: 'themes-label',
              content: ' Themes:',
              fg: t.textMuted,
              padding: 1,
            })

            each(
              () => themeNames.map((name, i) => ({ name, index: i })),
              (getItem, key) => {
                return box({
                  id: `theme-${key}`,
                  width: '100%',
                  height: 1,
                  flexDirection: 'row',
                  alignItems: 'center',
                  paddingLeft: 1,
                  bg: () => {
                    const item = getItem()
                    return item.index === currentThemeIndex.value
                      ? resolveColor(theme.primary)
                      : resolveColor(theme.background)
                  },
                  fg: () => {
                    const item = getItem()
                    return item.index === currentThemeIndex.value
                      ? resolveColor(theme.textBright)
                      : resolveColor(theme.text)
                  },
                  onClick: () => {
                    const item = getItem()
                    currentThemeIndex.value = item.index
                    setTheme(item.name)
                  },
                  children: () => {
                    text({
                      id: `theme-indicator-${key}`,
                      content: () => getItem().index === currentThemeIndex.value ? '>' : ' ',
                    })
                    text({
                      id: `theme-name-${key}`,
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
        // RIGHT PANEL
        // =====================================================================
        box({
          id: 'right-panel',
          grow: 1,
          height: '100%',
          flexDirection: 'column',
          padding: 1,
          children: () => {
            // =================================================================
            // VARIANT GRID
            // =================================================================
            box({
              id: 'variants-section',
              width: '100%',
              flexDirection: 'column',
              children: () => {
                text({
                  id: 'variants-label',
                  content: 'Variants:',
                  fg: t.textMuted,
                  paddingBottom: 1,
                })

                // Grid rows (5 variants per row)
                box({
                  id: 'variant-grid',
                  width: '100%',
                  flexDirection: 'column',
                  gap: 1,
                  children: () => {
                    // Row 1: default, primary, secondary, tertiary, accent
                    box({
                      id: 'variant-row-1',
                      flexDirection: 'row',
                      gap: 2,
                      children: () => {
                        for (const v of variants.slice(0, 5)) {
                          box({
                            id: `variant-${v}`,
                            variant: v,
                            width: 12,
                            height: 3,
                            border: 1,
                            justifyContent: 'center',
                            alignItems: 'center',
                            children: () => {
                              text({ content: v })
                            },
                          })
                        }
                      },
                    })

                    // Row 2: success, warning, error, info
                    box({
                      id: 'variant-row-2',
                      flexDirection: 'row',
                      gap: 2,
                      children: () => {
                        for (const v of variants.slice(5, 9)) {
                          box({
                            id: `variant-${v}`,
                            variant: v,
                            width: 12,
                            height: 3,
                            border: 1,
                            justifyContent: 'center',
                            alignItems: 'center',
                            children: () => {
                              text({ content: v })
                            },
                          })
                        }
                      },
                    })

                    // Row 3: muted, surface, elevated, ghost, outline
                    box({
                      id: 'variant-row-3',
                      flexDirection: 'row',
                      gap: 2,
                      children: () => {
                        for (const v of variants.slice(9, 14)) {
                          box({
                            id: `variant-${v}`,
                            variant: v,
                            width: 12,
                            height: 3,
                            border: 1,
                            justifyContent: 'center',
                            alignItems: 'center',
                            children: () => {
                              text({ content: v })
                            },
                          })
                        }
                      },
                    })
                  },
                })
              },
            })

            // =================================================================
            // COLOR PALETTE
            // =================================================================
            box({
              id: 'palette-section',
              width: '100%',
              flexDirection: 'column',
              marginTop: 2,
              children: () => {
                text({
                  id: 'palette-label',
                  content: 'Color Palette:',
                  fg: t.textMuted,
                  paddingBottom: 1,
                })

                box({
                  id: 'palette-grid',
                  flexDirection: 'row',
                  flexWrap: 'wrap',
                  gap: 1,
                  children: () => {
                    for (const slot of colorSlots) {
                      box({
                        id: `palette-${slot.key}`,
                        width: 6,
                        height: 3,
                        flexDirection: 'column',
                        alignItems: 'center',
                        justifyContent: 'center',
                        border: 1,
                        borderColor: t.border,
                        bg: () => resolveColor((theme as any)[slot.key]),
                        children: () => {
                          text({
                            content: slot.label,
                            fg: () => {
                              // Use contrasting color for text
                              const bg = (theme as any)[slot.key]
                              return bg === null ? resolveColor(theme.text) : resolveColor(theme.textBright)
                            },
                          })
                        },
                      })
                    }
                  },
                })
              },
            })

            // =================================================================
            // SAMPLE UI
            // =================================================================
            box({
              id: 'sample-section',
              width: '100%',
              flexDirection: 'column',
              marginTop: 2,
              children: () => {
                text({
                  id: 'sample-label',
                  content: 'Sample UI:',
                  fg: t.textMuted,
                  paddingBottom: 1,
                })

                box({
                  id: 'sample-card',
                  width: 60,
                  flexDirection: 'column',
                  border: 1,
                  borderColor: t.border,
                  bg: t.surface,
                  padding: 1,
                  children: () => {
                    // Form row 1: Username
                    box({
                      id: 'form-row-1',
                      width: '100%',
                      flexDirection: 'row',
                      alignItems: 'center',
                      gap: 2,
                      marginBottom: 1,
                      children: () => {
                        text({
                          content: 'Username:',
                          width: 10,
                          fg: t.text,
                        })
                        input({
                          id: 'username-input',
                          value: username,
                          width: 25,
                          border: 1,
                          borderColor: t.border,
                          bg: t.bg,
                          fg: t.text,
                          padding: 0,
                          paddingLeft: 1,
                          paddingRight: 1,
                        })
                        text({
                          content: 'Valid',
                          fg: t.success,
                        })
                      },
                    })

                    // Form row 2: Password
                    box({
                      id: 'form-row-2',
                      width: '100%',
                      flexDirection: 'row',
                      alignItems: 'center',
                      gap: 2,
                      marginBottom: 1,
                      children: () => {
                        text({
                          content: 'Password:',
                          width: 10,
                          fg: t.text,
                        })
                        input({
                          id: 'password-input',
                          value: password,
                          password: true,
                          width: 25,
                          border: 1,
                          borderColor: t.border,
                          bg: t.bg,
                          fg: t.text,
                          padding: 0,
                          paddingLeft: 1,
                          paddingRight: 1,
                        })
                      },
                    })

                    // Buttons row
                    box({
                      id: 'button-row',
                      width: '100%',
                      flexDirection: 'row',
                      justifyContent: 'center',
                      gap: 2,
                      marginTop: 1,
                      children: () => {
                        box({
                          id: 'login-btn',
                          variant: 'primary',
                          width: 12,
                          height: 3,
                          border: 1,
                          justifyContent: 'center',
                          alignItems: 'center',
                          focusable: true,
                          children: () => {
                            text({ content: 'Login' })
                          },
                        })
                        box({
                          id: 'cancel-btn',
                          variant: 'muted',
                          width: 12,
                          height: 3,
                          border: 1,
                          justifyContent: 'center',
                          alignItems: 'center',
                          focusable: true,
                          children: () => {
                            text({ content: 'Cancel' })
                          },
                        })
                      },
                    })
                  },
                })
              },
            })

            // =================================================================
            // STATUS MESSAGES
            // =================================================================
            box({
              id: 'status-section',
              width: '100%',
              flexDirection: 'row',
              gap: 2,
              marginTop: 2,
              children: () => {
                box({
                  id: 'status-success',
                  variant: 'success',
                  padding: 1,
                  border: 1,
                  children: () => text({ content: 'Success: Operation completed' }),
                })
                box({
                  id: 'status-warning',
                  variant: 'warning',
                  padding: 1,
                  border: 1,
                  children: () => text({ content: 'Warning: Check your input' }),
                })
                box({
                  id: 'status-error',
                  variant: 'error',
                  padding: 1,
                  border: 1,
                  children: () => text({ content: 'Error: Something went wrong' }),
                })
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
          id: 'footer-text',
          content: 'Up/Down: Select theme | Enter: Apply | Ctrl+C: Exit',
          fg: t.textMuted,
        })
      },
    })
  },
})
}, {
  mode: 'fullscreen',
})

console.log('[demo-themes] App mounted')

// Keep process alive
await new Promise(() => {})
