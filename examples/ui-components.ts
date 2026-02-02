/**
 * SparkTUI UI Components Showcase
 *
 * Comprehensive demonstration of styled UI components:
 * - Buttons (all variants: primary, secondary, success, warning, error, etc.)
 * - Badges/Tags
 * - Cards with headers
 * - Alerts (info, warning, error, success)
 * - Progress indicators
 * - All with proper theme colors and styling
 *
 * Navigation:
 * - Tab: Cycle focus through buttons
 * - Enter/Space: Activate focused button
 * - 1-5: Switch theme presets
 * - Ctrl+C: Exit
 */

import { signal, derived } from '@rlabs-inc/signals'
import { mount } from '../ts/engine/mount'
import { box, text, each, cycle, Frames } from '../ts/primitives'
import {
  theme,
  themes,
  setTheme,
  t,
  resolveColor,
  getVariantStyle,
  type Variant,
} from '../ts/state/theme'
// oklch import not needed - using theme colors
import { on } from '../ts/state/keyboard'
import type { KeyEvent } from '../ts/state/keyboard'

// =============================================================================
// STATE
// =============================================================================

const focusedButton = signal(0)
const clickedButton = signal<string | null>(null)
const progressValue = signal(65)

// Quick theme presets
const quickThemes = ['terminal', 'dracula', 'catppuccin', 'tokyoNight', 'nord'] as const

// Button variants to showcase
const buttonVariants: Variant[] = [
  'primary', 'secondary', 'tertiary', 'accent',
  'success', 'warning', 'error', 'info',
  'muted', 'ghost', 'outline',
]

// =============================================================================
// REUSABLE COMPONENTS
// =============================================================================

/**
 * Styled button component
 */
function Button(props: {
  id: string
  label: string
  variant: Variant
  width?: number
  index: number
  onClick?: () => void
}) {
  const { id, label, variant, width = 12, index, onClick } = props
  const isFocused = () => focusedButton.value === index

  box({
    id,
    variant,
    width,
    height: 3,
    border: 1,
    justifyContent: 'center',
    alignItems: 'center',
    focusable: true,
    borderColor: () => isFocused() ? resolveColor(theme.textBright) : getVariantStyle(variant).border,
    onClick: () => {
      clickedButton.value = label
      onClick?.()
      // Reset after 1 second
      setTimeout(() => {
        if (clickedButton.value === label) {
          clickedButton.value = null
        }
      }, 1000)
    },
    children: () => {
      text({ content: label })
    },
  })
}

/**
 * Badge/Tag component
 */
function Badge(props: {
  id: string
  label: string
  variant: Variant
}) {
  const { id, label, variant } = props

  box({
    id,
    variant,
    height: 1,
    paddingLeft: 1,
    paddingRight: 1,
    justifyContent: 'center',
    alignItems: 'center',
    children: () => {
      text({ content: label })
    },
  })
}

/**
 * Card component with optional header
 */
function Card(props: {
  id: string
  title?: string
  width?: number | string
  children: () => void
}) {
  const { id, title, width = 30, children } = props

  box({
    id,
    width,
    flexDirection: 'column',
    border: 1,
    borderColor: t.textMuted,
    bg: t.surface,
    children: () => {
      if (title) {
        box({
          id: `${id}-header`,
          width: '100%',
          height: 1,
          paddingLeft: 1,
          paddingRight: 1,
          bg: t.bgMuted,
          borderColor: t.textMuted,
          children: () => {
            text({
              content: title,
              fg: t.textBright,
            })
          },
        })
      }
      box({
        id: `${id}-body`,
        padding: 1,
        flexDirection: 'column',
        children,
      })
    },
  })
}

/**
 * Alert component
 */
function Alert(props: {
  id: string
  message: string
  variant: 'success' | 'warning' | 'error' | 'info'
  icon?: string
}) {
  const { id, message, variant, icon } = props
  const icons = {
    success: '[OK]',
    warning: '[!!]',
    error: '[X]',
    info: '[i]',
  }

  box({
    id,
    variant,
    width: '100%',
    flexDirection: 'row',
    alignItems: 'center',
    gap: 1,
    padding: 1,
    border: 1,
    children: () => {
      text({
        content: icon || icons[variant],
      })
      text({
        content: message,
      })
    },
  })
}

/**
 * Progress bar component
 */
function ProgressBar(props: {
  id: string
  value: number | (() => number)
  width?: number
  variant?: Variant
  showLabel?: boolean
}) {
  const { id, value, width = 30, variant = 'primary', showLabel = true } = props
  const getValue = () => typeof value === 'function' ? value() : value

  box({
    id,
    width,
    height: 1,
    flexDirection: 'row',
    alignItems: 'center',
    gap: 1,
    children: () => {
      // Track
      box({
        id: `${id}-track`,
        width: width - 6,
        height: 1,
        flexDirection: 'row',
        border: 1,
        borderColor: t.textMuted,
        children: () => {
          const filledWidth = () => Math.round((getValue() / 100) * (width - 10))

          // Filled portion
          box({
            id: `${id}-filled`,
            width: filledWidth,
            height: 1,
            bg: () => getVariantStyle(variant).bg,
          })
        },
      })

      if (showLabel) {
        text({
          content: () => `${getValue()}%`,
          width: 5,
          fg: t.text,
        })
      }
    },
  })
}

// =============================================================================
// MOUNT APP
// =============================================================================

const cols = process.stdout.columns || 100
const rows = process.stdout.rows || 40

const { unmount } = mount(() => {
  // Keyboard navigation
  on((event: KeyEvent) => {
    const keycode = event.keycode

    // Tab - cycle focus
    if (keycode === 9) {
      focusedButton.value = (focusedButton.value + 1) % buttonVariants.length
      return true
    }

    // Number keys 1-5 for quick theme switching
    if (keycode >= 49 && keycode <= 53) {
      const themeIdx = keycode - 49
      setTheme(quickThemes[themeIdx]! as keyof typeof themes)
      return true
    }

    // +/= increase progress
    if (keycode === 43 || keycode === 61) {
      progressValue.value = Math.min(100, progressValue.value + 5)
      return true
    }

    // - decrease progress
    if (keycode === 45) {
      progressValue.value = Math.max(0, progressValue.value - 5)
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
        height: 3,
        flexDirection: 'row',
        justifyContent: 'space-between',
        alignItems: 'center',
        paddingLeft: 2,
        paddingRight: 2,
        border: 1,
        borderColor: t.primary,
        bg: t.surface,
        children: () => {
          text({
            content: ' UI Components Showcase ',
            fg: t.primary,
          })
          text({
            content: () => `Theme: ${theme.name}`,
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
        padding: 1,
        gap: 2,
        overflow: 'scroll',
        children: () => {
          // =====================================================================
          // LEFT COLUMN
          // =====================================================================
          box({
            id: 'left-column',
            width: '50%',
            flexDirection: 'column',
            gap: 1,
            children: () => {
              // -----------------------------------------------------------------
              // BUTTONS SECTION
              // -----------------------------------------------------------------
              Card({
                id: 'buttons-card',
                title: ' Buttons ',
                width: '100%',
                children: () => {
                  // Row 1: Primary variants
                  box({
                    id: 'btn-row-1',
                    flexDirection: 'row',
                    flexWrap: 'wrap',
                    gap: 1,
                    marginBottom: 1,
                    children: () => {
                      const row1 = ['primary', 'secondary', 'tertiary', 'accent'] as const
                      row1.forEach((v, i) => {
                        Button({
                          id: `btn-${v}`,
                          label: v,
                          variant: v,
                          width: 11,
                          index: i,
                        })
                      })
                    },
                  })

                  // Row 2: Semantic variants
                  box({
                    id: 'btn-row-2',
                    flexDirection: 'row',
                    flexWrap: 'wrap',
                    gap: 1,
                    marginBottom: 1,
                    children: () => {
                      const row2 = ['success', 'warning', 'error', 'info'] as const
                      row2.forEach((v, i) => {
                        Button({
                          id: `btn-${v}`,
                          label: v,
                          variant: v,
                          width: 11,
                          index: i + 4,
                        })
                      })
                    },
                  })

                  // Row 3: Style variants
                  box({
                    id: 'btn-row-3',
                    flexDirection: 'row',
                    flexWrap: 'wrap',
                    gap: 1,
                    children: () => {
                      const row3 = ['muted', 'ghost', 'outline'] as const
                      row3.forEach((v, i) => {
                        Button({
                          id: `btn-${v}`,
                          label: v,
                          variant: v,
                          width: 11,
                          index: i + 8,
                        })
                      })
                    },
                  })

                  // Click indicator
                  box({
                    id: 'click-indicator',
                    height: 1,
                    marginTop: 1,
                    children: () => {
                      text({
                        content: () => clickedButton.value
                          ? `Clicked: ${clickedButton.value}`
                          : 'Tab to navigate, Enter to click',
                        fg: () => clickedButton.value ? resolveColor(theme.success) : resolveColor(theme.textMuted),
                      })
                    },
                  })
                },
              })

              // -----------------------------------------------------------------
              // BADGES SECTION
              // -----------------------------------------------------------------
              Card({
                id: 'badges-card',
                title: ' Badges & Tags ',
                width: '100%',
                children: () => {
                  box({
                    id: 'badges-row',
                    flexDirection: 'row',
                    flexWrap: 'wrap',
                    gap: 1,
                    children: () => {
                      Badge({ id: 'badge-new', label: 'NEW', variant: 'success' })
                      Badge({ id: 'badge-beta', label: 'BETA', variant: 'warning' })
                      Badge({ id: 'badge-deprecated', label: 'DEPRECATED', variant: 'error' })
                      Badge({ id: 'badge-featured', label: 'FEATURED', variant: 'primary' })
                      Badge({ id: 'badge-pro', label: 'PRO', variant: 'accent' })
                      Badge({ id: 'badge-v2', label: 'v2.0', variant: 'info' })
                      Badge({ id: 'badge-free', label: 'FREE', variant: 'tertiary' })
                    },
                  })
                },
              })

              // -----------------------------------------------------------------
              // PROGRESS SECTION
              // -----------------------------------------------------------------
              Card({
                id: 'progress-card',
                title: ' Progress Indicators ',
                width: '100%',
                children: () => {
                  box({
                    id: 'progress-rows',
                    flexDirection: 'column',
                    gap: 1,
                    children: () => {
                      // Interactive progress
                      box({
                        id: 'progress-row-1',
                        flexDirection: 'row',
                        alignItems: 'center',
                        gap: 2,
                        children: () => {
                          text({ content: 'Progress:', width: 10, fg: t.textMuted })
                          ProgressBar({
                            id: 'progress-main',
                            value: () => progressValue.value,
                            width: 30,
                            variant: 'primary',
                          })
                        },
                      })

                      // Success progress
                      box({
                        id: 'progress-row-2',
                        flexDirection: 'row',
                        alignItems: 'center',
                        gap: 2,
                        children: () => {
                          text({ content: 'Complete:', width: 10, fg: t.textMuted })
                          ProgressBar({
                            id: 'progress-success',
                            value: 100,
                            width: 30,
                            variant: 'success',
                          })
                        },
                      })

                      // Warning progress
                      box({
                        id: 'progress-row-3',
                        flexDirection: 'row',
                        alignItems: 'center',
                        gap: 2,
                        children: () => {
                          text({ content: 'Warning:', width: 10, fg: t.textMuted })
                          ProgressBar({
                            id: 'progress-warning',
                            value: 45,
                            width: 30,
                            variant: 'warning',
                          })
                        },
                      })

                      // Animated loading
                      box({
                        id: 'loading-row',
                        flexDirection: 'row',
                        alignItems: 'center',
                        gap: 2,
                        children: () => {
                          text({ content: 'Loading:', width: 10, fg: t.textMuted })
                          text({
                            content: cycle(Frames.spinner, { fps: 10 }),
                            fg: t.primary,
                          })
                          text({
                            content: cycle(Frames.dots, { fps: 8 }),
                            fg: t.secondary,
                          })
                          text({
                            content: cycle(['   ', '.  ', '.. ', '...'], { fps: 3 }),
                            fg: t.textMuted,
                          })
                        },
                      })
                    },
                  })
                },
              })
            },
          })

          // =====================================================================
          // RIGHT COLUMN
          // =====================================================================
          box({
            id: 'right-column',
            grow: 1,
            flexDirection: 'column',
            gap: 1,
            children: () => {
              // -----------------------------------------------------------------
              // ALERTS SECTION
              // -----------------------------------------------------------------
              Card({
                id: 'alerts-card',
                title: ' Alerts ',
                width: '100%',
                children: () => {
                  box({
                    id: 'alerts-stack',
                    flexDirection: 'column',
                    gap: 1,
                    children: () => {
                      Alert({
                        id: 'alert-success',
                        variant: 'success',
                        message: 'Operation completed successfully!',
                      })
                      Alert({
                        id: 'alert-info',
                        variant: 'info',
                        message: 'New updates are available.',
                      })
                      Alert({
                        id: 'alert-warning',
                        variant: 'warning',
                        message: 'Your session will expire in 5 minutes.',
                      })
                      Alert({
                        id: 'alert-error',
                        variant: 'error',
                        message: 'Failed to save changes. Please try again.',
                      })
                    },
                  })
                },
              })

              // -----------------------------------------------------------------
              // SAMPLE CARDS
              // -----------------------------------------------------------------
              Card({
                id: 'sample-card-1',
                title: ' User Profile ',
                width: '100%',
                children: () => {
                  box({
                    id: 'profile-content',
                    flexDirection: 'column',
                    gap: 1,
                    children: () => {
                      box({
                        id: 'profile-row-1',
                        flexDirection: 'row',
                        gap: 2,
                        children: () => {
                          text({ content: 'Name:', width: 8, fg: t.textMuted })
                          text({ content: 'John Developer', fg: t.text })
                        },
                      })
                      box({
                        id: 'profile-row-2',
                        flexDirection: 'row',
                        gap: 2,
                        children: () => {
                          text({ content: 'Role:', width: 8, fg: t.textMuted })
                          text({ content: 'Admin', fg: t.text })
                          Badge({ id: 'role-badge', label: 'PRO', variant: 'accent' })
                        },
                      })
                      box({
                        id: 'profile-row-3',
                        flexDirection: 'row',
                        gap: 2,
                        children: () => {
                          text({ content: 'Status:', width: 8, fg: t.textMuted })
                          text({
                            content: cycle(['Online', 'Online.', 'Online..', 'Online...'], { fps: 2 }),
                            fg: t.success,
                          })
                        },
                      })
                    },
                  })
                },
              })

              // -----------------------------------------------------------------
              // THEME QUICK SWITCH
              // -----------------------------------------------------------------
              Card({
                id: 'theme-card',
                title: ' Quick Theme Switch (1-5) ',
                width: '100%',
                children: () => {
                  box({
                    id: 'theme-buttons',
                    flexDirection: 'row',
                    flexWrap: 'wrap',
                    gap: 1,
                    children: () => {
                      quickThemes.forEach((themeName, i) => {
                        box({
                          id: `theme-btn-${themeName}`,
                          width: 12,
                          height: 2,
                          border: 1,
                          borderColor: () =>
                            theme.name === themeName
                              ? resolveColor(theme.primary)
                              : resolveColor(theme.textMuted),
                          bg: () =>
                            theme.name === themeName
                              ? resolveColor(theme.primary)
                              : resolveColor(theme.surface),
                          fg: () =>
                            theme.name === themeName
                              ? resolveColor(theme.textBright)
                              : resolveColor(theme.text),
                          justifyContent: 'center',
                          alignItems: 'center',
                          onClick: () => setTheme(themeName as keyof typeof themes),
                          children: () => {
                            text({ content: `${i + 1}:${themeName.slice(0, 7)}` })
                          },
                        })
                      })
                    },
                  })
                },
              })

              // -----------------------------------------------------------------
              // COLOR SWATCHES
              // -----------------------------------------------------------------
              Card({
                id: 'colors-card',
                title: ' Current Theme Colors ',
                width: '100%',
                children: () => {
                  box({
                    id: 'color-swatches',
                    flexDirection: 'row',
                    flexWrap: 'wrap',
                    gap: 1,
                    children: () => {
                      const colors = [
                        { key: 'primary', label: 'P' },
                        { key: 'secondary', label: 'S' },
                        { key: 'tertiary', label: 'T' },
                        { key: 'accent', label: 'A' },
                        { key: 'success', label: 'OK' },
                        { key: 'warning', label: 'W' },
                        { key: 'error', label: 'E' },
                        { key: 'info', label: 'I' },
                      ]

                      for (const color of colors) {
                        box({
                          id: `swatch-${color.key}`,
                          width: 4,
                          height: 2,
                          bg: () => resolveColor((theme as any)[color.key]),
                          justifyContent: 'center',
                          alignItems: 'center',
                          border: 1,
                          borderColor: t.textMuted,
                          children: () => {
                            text({
                              content: color.label,
                              fg: t.textBright,
                            })
                          },
                        })
                      }
                    },
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
            content: 'Tab: Focus | 1-5: Themes | +/-: Progress | Ctrl+C: Exit',
            fg: t.textMuted,
          })
        },
      })
    },
  })
}, {
  mode: 'fullscreen',
})

console.log('[ui-components] App mounted')

// Keep process alive
await new Promise(() => {})
