/**
 * SparkTUI API Showcase
 *
 * A comprehensive demonstration of the SparkTUI user API, organized by category.
 * Each section shows correct usage patterns derived from the actual implementation.
 *
 * Sections:
 * 1. Signals & Reactivity - signal, derived, getters
 * 2. Primitives - box, text, input
 * 3. Control Flow - show, each (with correct cleanup patterns!)
 * 4. Layout - flexbox, dimensions, spacing
 * 5. Styling - borders, colors, variants
 * 6. Themes - 13 presets, theme switching
 * 7. Animations - cycle, Frames
 * 8. Keyboard & Focus
 * 9. Mouse Events
 * 10. Scroll
 *
 * Navigation: Tab to cycle sections, Ctrl+C to exit
 */

import { signal, derived } from '@rlabs-inc/signals'
import { mount } from '../ts/engine/mount'
import { box, text, input, show, each, cycle, Frames } from '../ts/primitives'
import { packColor } from '../ts/bridge/shared-buffer'
import { setTheme, t, themes } from '../ts/state/theme'
import { on, isPress } from '../ts/state/keyboard'
import type { KeyEvent } from '../ts/state/keyboard'

// Helper to get printable character from keycode
function getChar(event: KeyEvent): string | null {
  const code = event.keycode
  if (code >= 32 && code <= 126) {
    return String.fromCharCode(code)
  }
  return null
}

// =============================================================================
// COLORS
// =============================================================================

const colors = {
  bg: packColor(20, 20, 28, 255),
  bgSection: packColor(30, 30, 42, 255),
  bgHighlight: packColor(45, 45, 65, 255),
  border: packColor(70, 70, 100, 255),
  borderActive: packColor(120, 140, 255, 255),
  text: packColor(220, 220, 235, 255),
  textMuted: packColor(120, 120, 150, 255),
  textBright: packColor(255, 255, 255, 255),
  accent: packColor(140, 170, 255, 255),
  success: packColor(120, 220, 150, 255),
  warning: packColor(255, 200, 100, 255),
  error: packColor(255, 120, 120, 255),
}

// =============================================================================
// STATE
// =============================================================================

const currentSection = signal(0)
const sectionNames = [
  'Signals',
  'Primitives',
  'Control Flow',
  'Layout',
  'Styling',
  'Themes',
  'Animations',
  'Keyboard',
  'Mouse',
  'Scroll',
]

// =============================================================================
// MOUNT APP
// =============================================================================

const cols = process.stdout.columns || 100
const rows = process.stdout.rows || 30

mount(() => {
  // ─────────────────────────────────────────────────────────────────────────────
  // KEYBOARD HANDLER
  // ─────────────────────────────────────────────────────────────────────────────
  on((event: KeyEvent) => {
    if (!isPress(event)) return false

    // Tab: next section
    if (event.keycode === 9) {
      currentSection.value = (currentSection.value + 1) % sectionNames.length
      return true
    }

    // Shift+Tab: previous section (ESC [ Z)
    if (event.keycode === 0x1b5b5a) {
      currentSection.value = (currentSection.value - 1 + sectionNames.length) % sectionNames.length
      return true
    }

    // Number keys 1-9, 0 for section selection
    const char = getChar(event)
    if (char && char >= '0' && char <= '9') {
      const num = char === '0' ? 9 : parseInt(char) - 1
      if (num < sectionNames.length) {
        currentSection.value = num
        return true
      }
    }

    return false
  })

  // ─────────────────────────────────────────────────────────────────────────────
  // ROOT CONTAINER
  // ─────────────────────────────────────────────────────────────────────────────
  box({
    id: 'root',
    width: cols,
    height: rows,
    flexDirection: 'column',
    bg: colors.bg,
    children: () => {
      // ─────────────────────────────────────────────────────────────────────────
      // HEADER
      // ─────────────────────────────────────────────────────────────────────────
      box({
        id: 'header',
        width: '100%',
        height: 3,
        flexDirection: 'row',
        justifyContent: 'space-between',
        alignItems: 'center',
        paddingLeft: 2,
        paddingRight: 2,
        bg: colors.bgSection,
        borderBottom: 1,
        borderColor: colors.border,
        children: () => {
          text({ content: 'SparkTUI API Showcase', fg: colors.accent })
          text({
            content: () => `Section ${currentSection.value + 1}/${sectionNames.length}: ${sectionNames[currentSection.value]}`,
            fg: colors.textMuted,
          })
        },
      })

      // ─────────────────────────────────────────────────────────────────────────
      // NAV BAR
      // ─────────────────────────────────────────────────────────────────────────
      box({
        id: 'nav',
        width: '100%',
        height: 1,
        flexDirection: 'row',
        gap: 2,
        paddingLeft: 2,
        bg: colors.bgSection,
        children: () => {
          for (let i = 0; i < sectionNames.length; i++) {
            const idx = i
            box({
              id: `nav-${i}`,
              children: () => {
                text({
                  content: `${(i + 1) % 10}:${sectionNames[i]}`,
                  fg: () => currentSection.value === idx ? colors.accent : colors.textMuted,
                })
              },
            })
          }
        },
      })

      // ─────────────────────────────────────────────────────────────────────────
      // CONTENT AREA
      // ─────────────────────────────────────────────────────────────────────────
      box({
        id: 'content',
        width: '100%',
        grow: 1,
        padding: 2,
        overflow: 'scroll',
        children: () => {
          // SECTION 0: SIGNALS
          show(
            () => currentSection.value === 0,
            () => SignalsSection()
          )

          // SECTION 1: PRIMITIVES
          show(
            () => currentSection.value === 1,
            () => PrimitivesSection()
          )

          // SECTION 2: CONTROL FLOW
          show(
            () => currentSection.value === 2,
            () => ControlFlowSection()
          )

          // SECTION 3: LAYOUT
          show(
            () => currentSection.value === 3,
            () => LayoutSection()
          )

          // SECTION 4: STYLING
          show(
            () => currentSection.value === 4,
            () => StylingSection()
          )

          // SECTION 5: THEMES
          show(
            () => currentSection.value === 5,
            () => ThemesSection()
          )

          // SECTION 6: ANIMATIONS
          show(
            () => currentSection.value === 6,
            () => AnimationsSection()
          )

          // SECTION 7: KEYBOARD
          show(
            () => currentSection.value === 7,
            () => KeyboardSection()
          )

          // SECTION 8: MOUSE
          show(
            () => currentSection.value === 8,
            () => MouseSection()
          )

          // SECTION 9: SCROLL
          show(
            () => currentSection.value === 9,
            () => ScrollSection()
          )
        },
      })

      // ─────────────────────────────────────────────────────────────────────────
      // FOOTER
      // ─────────────────────────────────────────────────────────────────────────
      box({
        id: 'footer',
        width: '100%',
        height: 1,
        flexDirection: 'row',
        justifyContent: 'center',
        bg: colors.bgSection,
        borderTop: 1,
        borderColor: colors.border,
        children: () => {
          text({
            content: 'Tab: Next section | 1-0: Jump to section | Ctrl+C: Exit',
            fg: colors.textMuted,
          })
        },
      })
    },
  })
}, { mode: 'fullscreen' })

// =============================================================================
// SECTION 0: SIGNALS & REACTIVITY
// =============================================================================

function SignalsSection() {
  // Local state for this section
  const counter = signal(0)
  const doubled = derived(() => counter.value * 2)
  const message = signal('Hello SparkTUI!')

  // Auto-increment counter
  const interval = setInterval(() => {
    counter.value++
  }, 1000)

  // Return cleanup from root box
  return box({
    id: 'signals-section',
    flexDirection: 'column',
    gap: 1,
    children: () => {
      text({ content: '1. SIGNALS & REACTIVITY', fg: colors.accent })
      text({ content: 'Reactive state management with fine-grained updates', fg: colors.textMuted })

      box({
        marginTop: 1,
        flexDirection: 'column',
        gap: 1,
        border: 1,
        borderColor: colors.border,
        padding: 1,
        children: () => {
          // signal() - writable reactive value
          text({ content: 'signal() - Writable reactive value:', fg: colors.textBright })
          text({ content: () => `  counter.value = ${counter.value}`, fg: colors.text })
          text({ content: () => `  message.value = "${message.value}"`, fg: colors.text })

          // derived() - computed value
          text({ content: 'derived() - Computed from signals:', fg: colors.textBright, marginTop: 1 })
          text({ content: () => `  doubled = counter * 2 = ${doubled.value}`, fg: colors.text })

          // Getter function - inline reactive
          text({ content: 'Getter () => - Inline reactive expression:', fg: colors.textBright, marginTop: 1 })
          text({ content: () => `  tripled = ${counter.value * 3}`, fg: colors.text })
        },
      })

      text({
        content: 'All three forms work with any prop: width, height, color, content, etc.',
        fg: colors.textMuted,
        marginTop: 1,
      })
    },
  })
}

// =============================================================================
// SECTION 1: PRIMITIVES
// =============================================================================

function PrimitivesSection() {
  const inputValue = signal('Type here...')

  return box({
    id: 'primitives-section',
    flexDirection: 'column',
    gap: 1,
    children: () => {
      text({ content: '2. PRIMITIVES', fg: colors.accent })
      text({ content: 'box(), text(), input() - All return Cleanup functions', fg: colors.textMuted })

      // BOX
      box({
        marginTop: 1,
        flexDirection: 'column',
        border: 1,
        borderColor: colors.border,
        padding: 1,
        children: () => {
          text({ content: 'box() - Container with flexbox layout', fg: colors.textBright })

          box({
            flexDirection: 'row',
            gap: 2,
            marginTop: 1,
            children: () => {
              box({
                width: 15,
                height: 3,
                bg: colors.bgHighlight,
                border: 1,
                borderColor: colors.border,
                justifyContent: 'center',
                alignItems: 'center',
                children: () => text({ content: 'Box 1' }),
              })
              box({
                width: 15,
                height: 3,
                bg: colors.bgHighlight,
                border: 1,
                borderColor: colors.accent,
                justifyContent: 'center',
                alignItems: 'center',
                children: () => text({ content: 'Box 2' }),
              })
            },
          })
        },
      })

      // TEXT
      box({
        marginTop: 1,
        flexDirection: 'column',
        border: 1,
        borderColor: colors.border,
        padding: 1,
        children: () => {
          text({ content: 'text() - Text content (required: content prop)', fg: colors.textBright })

          box({
            flexDirection: 'column',
            gap: 1,
            marginTop: 1,
            children: () => {
              text({ content: 'Normal text', fg: colors.text })
              text({ content: 'Colored text', fg: colors.success })
              text({ content: 'Bold text', fg: colors.accent, attrs: { bold: true } })
              text({ content: 'Italic text', fg: colors.warning, attrs: { italic: true } })
              text({ content: 'Underlined', fg: colors.error, attrs: { underline: true } })
            },
          })
        },
      })

      // INPUT
      box({
        marginTop: 1,
        flexDirection: 'column',
        border: 1,
        borderColor: colors.border,
        padding: 1,
        children: () => {
          text({ content: 'input() - Text input (required: value prop)', fg: colors.textBright })

          box({
            flexDirection: 'row',
            alignItems: 'center',
            gap: 2,
            marginTop: 1,
            children: () => {
              text({ content: 'Value:', fg: colors.text })
              input({
                id: 'demo-input',
                value: inputValue,
                width: 30,
                border: 1,
                borderColor: colors.border,
                bg: colors.bgHighlight,
                fg: colors.text,
                paddingLeft: 1,
                paddingRight: 1,
              })
            },
          })

          text({
            content: () => `Current value: "${inputValue.value}"`,
            fg: colors.textMuted,
            marginTop: 1,
          })
        },
      })
    },
  })
}

// =============================================================================
// SECTION 2: CONTROL FLOW
// =============================================================================

function ControlFlowSection() {
  const showContent = signal(true)
  const items = signal([
    { id: '1', name: 'Apple' },
    { id: '2', name: 'Banana' },
    { id: '3', name: 'Cherry' },
  ])

  return box({
    id: 'control-flow-section',
    flexDirection: 'column',
    gap: 1,
    children: () => {
      text({ content: '3. CONTROL FLOW', fg: colors.accent })
      text({ content: 'show() and each() - Conditional and list rendering', fg: colors.textMuted })

      // SHOW
      box({
        marginTop: 1,
        flexDirection: 'column',
        border: 1,
        borderColor: colors.border,
        padding: 1,
        children: () => {
          text({ content: 'show(condition, renderFn) - Conditional rendering', fg: colors.textBright })
          text({
            content: 'CRITICAL: renderFn must RETURN the cleanup from root component!',
            fg: colors.warning,
          })

          box({
            flexDirection: 'row',
            alignItems: 'center',
            gap: 2,
            marginTop: 1,
            children: () => {
              box({
                width: 10,
                height: 1,
                bg: colors.bgHighlight,
                border: 1,
                borderColor: colors.border,
                justifyContent: 'center',
                alignItems: 'center',
                focusable: true,
                onClick: () => { showContent.value = !showContent.value },
                children: () => text({ content: 'Toggle' }),
              })

              text({ content: () => `showContent = ${showContent.value}`, fg: colors.textMuted })
            },
          })

          box({
            marginTop: 1,
            height: 3,
            children: () => {
              // Correct pattern: renderFn returns the cleanup
              show(
                () => showContent.value,
                () => {
                  return box({
                    bg: colors.success,
                    padding: 1,
                    children: () => text({ content: 'Content is visible!', fg: colors.bg }),
                  })
                }
              )

              // Else branch also returns cleanup
              show(
                () => !showContent.value,
                () => {
                  return box({
                    bg: colors.error,
                    padding: 1,
                    children: () => text({ content: 'Content is hidden!', fg: colors.bg }),
                  })
                }
              )
            },
          })
        },
      })

      // EACH
      box({
        marginTop: 1,
        flexDirection: 'column',
        border: 1,
        borderColor: colors.border,
        padding: 1,
        children: () => {
          text({ content: 'each(items, renderFn, {key}) - List rendering', fg: colors.textBright })
          text({
            content: 'renderFn receives (getItem, key) and must RETURN cleanup!',
            fg: colors.warning,
          })

          box({
            flexDirection: 'row',
            gap: 2,
            marginTop: 1,
            children: () => {
              box({
                width: 8,
                height: 1,
                bg: colors.bgHighlight,
                border: 1,
                borderColor: colors.border,
                justifyContent: 'center',
                focusable: true,
                onClick: () => {
                  const id = String(Date.now())
                  items.value = [...items.value, { id, name: `Item ${id.slice(-4)}` }]
                },
                children: () => text({ content: 'Add' }),
              })

              box({
                width: 10,
                height: 1,
                bg: colors.bgHighlight,
                border: 1,
                borderColor: colors.border,
                justifyContent: 'center',
                focusable: true,
                onClick: () => {
                  if (items.value.length > 0) {
                    items.value = items.value.slice(0, -1)
                  }
                },
                children: () => text({ content: 'Remove' }),
              })

              text({ content: () => `Count: ${items.value.length}`, fg: colors.textMuted })
            },
          })

          box({
            flexDirection: 'row',
            gap: 1,
            marginTop: 1,
            flexWrap: 'wrap',
            children: () => {
              // Correct pattern: renderFn returns the cleanup
              each(
                () => items.value,
                (getItem, key) => {
                  return box({
                    id: `item-${key}`,
                    bg: colors.bgHighlight,
                    border: 1,
                    borderColor: colors.border,
                    paddingLeft: 1,
                    paddingRight: 1,
                    children: () => {
                      text({ content: () => getItem().name, fg: colors.text })
                    },
                  })
                },
                { key: (item) => item.id }
              )
            },
          })
        },
      })
    },
  })
}

// =============================================================================
// SECTION 3: LAYOUT
// =============================================================================

function LayoutSection() {
  return box({
    id: 'layout-section',
    flexDirection: 'column',
    gap: 1,
    children: () => {
      text({ content: '4. LAYOUT', fg: colors.accent })
      text({ content: 'Flexbox layout powered by Taffy', fg: colors.textMuted })

      // Flex Direction
      box({
        marginTop: 1,
        flexDirection: 'column',
        border: 1,
        borderColor: colors.border,
        padding: 1,
        children: () => {
          text({ content: 'flexDirection: row | column', fg: colors.textBright })

          box({
            flexDirection: 'row',
            gap: 2,
            marginTop: 1,
            children: () => {
              // Row
              box({
                flexDirection: 'column',
                children: () => {
                  text({ content: 'row:', fg: colors.textMuted })
                  box({
                    flexDirection: 'row',
                    gap: 1,
                    children: () => {
                      for (let i = 1; i <= 3; i++) {
                        box({
                          width: 4,
                          height: 2,
                          bg: colors.bgHighlight,
                          border: 1,
                          borderColor: colors.border,
                          justifyContent: 'center',
                          alignItems: 'center',
                          children: () => text({ content: String(i) }),
                        })
                      }
                    },
                  })
                },
              })

              // Column
              box({
                flexDirection: 'column',
                children: () => {
                  text({ content: 'column:', fg: colors.textMuted })
                  box({
                    flexDirection: 'column',
                    gap: 0,
                    children: () => {
                      for (let i = 1; i <= 3; i++) {
                        box({
                          width: 8,
                          height: 1,
                          bg: colors.bgHighlight,
                          border: 1,
                          borderColor: colors.border,
                          justifyContent: 'center',
                          children: () => text({ content: String(i) }),
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

      // Justify Content
      box({
        marginTop: 1,
        flexDirection: 'column',
        border: 1,
        borderColor: colors.border,
        padding: 1,
        children: () => {
          text({ content: 'justifyContent: flex-start | center | flex-end | space-between', fg: colors.textBright })

          const justifyOptions = ['flex-start', 'center', 'flex-end', 'space-between'] as const

          box({
            flexDirection: 'column',
            gap: 1,
            marginTop: 1,
            children: () => {
              for (const justify of justifyOptions) {
                box({
                  flexDirection: 'row',
                  alignItems: 'center',
                  gap: 1,
                  children: () => {
                    text({ content: `${justify}:`.padEnd(16), fg: colors.textMuted })
                    box({
                      width: 40,
                      flexDirection: 'row',
                      justifyContent: justify,
                      bg: colors.bgHighlight,
                      border: 1,
                      borderColor: colors.border,
                      children: () => {
                        for (let i = 0; i < 3; i++) {
                          box({
                            width: 4,
                            height: 1,
                            bg: colors.accent,
                            justifyContent: 'center',
                            children: () => text({ content: '[]', fg: colors.bg }),
                          })
                        }
                      },
                    })
                  },
                })
              }
            },
          })
        },
      })

      // Grow
      box({
        marginTop: 1,
        flexDirection: 'column',
        border: 1,
        borderColor: colors.border,
        padding: 1,
        children: () => {
          text({ content: 'grow: 0 | 1 | 2 ... - Flex grow factor', fg: colors.textBright })

          box({
            flexDirection: 'row',
            width: 60,
            marginTop: 1,
            bg: colors.bgHighlight,
            border: 1,
            borderColor: colors.border,
            children: () => {
              box({
                grow: 1,
                height: 2,
                bg: colors.success,
                justifyContent: 'center',
                alignItems: 'center',
                children: () => text({ content: 'grow:1', fg: colors.bg }),
              })
              box({
                grow: 2,
                height: 2,
                bg: colors.warning,
                justifyContent: 'center',
                alignItems: 'center',
                children: () => text({ content: 'grow:2', fg: colors.bg }),
              })
              box({
                grow: 1,
                height: 2,
                bg: colors.error,
                justifyContent: 'center',
                alignItems: 'center',
                children: () => text({ content: 'grow:1', fg: colors.bg }),
              })
            },
          })
        },
      })
    },
  })
}

// =============================================================================
// SECTION 4: STYLING
// =============================================================================

function StylingSection() {
  return box({
    id: 'styling-section',
    flexDirection: 'column',
    gap: 1,
    children: () => {
      text({ content: '5. STYLING', fg: colors.accent })
      text({ content: 'Borders, colors, and visual properties', fg: colors.textMuted })

      // Borders
      box({
        marginTop: 1,
        flexDirection: 'column',
        border: 1,
        borderColor: colors.border,
        padding: 1,
        children: () => {
          text({ content: 'Borders: border, borderTop/Right/Bottom/Left, borderColor', fg: colors.textBright })

          box({
            flexDirection: 'row',
            gap: 2,
            marginTop: 1,
            children: () => {
              // Full border
              box({
                width: 12,
                height: 3,
                border: 1,
                borderColor: colors.accent,
                justifyContent: 'center',
                alignItems: 'center',
                children: () => text({ content: 'border:1' }),
              })

              // Top only
              box({
                width: 12,
                height: 3,
                borderTop: 1,
                borderColor: colors.success,
                justifyContent: 'center',
                alignItems: 'center',
                children: () => text({ content: 'borderTop' }),
              })

              // Left + Right
              box({
                width: 12,
                height: 3,
                borderLeft: 1,
                borderRight: 1,
                borderColor: colors.warning,
                justifyContent: 'center',
                alignItems: 'center',
                children: () => text({ content: 'L+R' }),
              })

              // Thick border
              box({
                width: 12,
                height: 3,
                border: 2,
                borderColor: colors.error,
                justifyContent: 'center',
                alignItems: 'center',
                children: () => text({ content: 'border:2' }),
              })
            },
          })
        },
      })

      // Colors
      box({
        marginTop: 1,
        flexDirection: 'column',
        border: 1,
        borderColor: colors.border,
        padding: 1,
        children: () => {
          text({ content: 'Colors: fg, bg (packColor or ANSI 0-255)', fg: colors.textBright })

          box({
            flexDirection: 'row',
            gap: 1,
            marginTop: 1,
            children: () => {
              // Color swatches
              const swatches = [
                { bg: colors.accent, label: 'accent' },
                { bg: colors.success, label: 'success' },
                { bg: colors.warning, label: 'warning' },
                { bg: colors.error, label: 'error' },
              ]

              for (const swatch of swatches) {
                box({
                  width: 10,
                  height: 2,
                  bg: swatch.bg,
                  justifyContent: 'center',
                  alignItems: 'center',
                  children: () => text({ content: swatch.label, fg: colors.bg }),
                })
              }
            },
          })
        },
      })

      // Variants
      box({
        marginTop: 1,
        flexDirection: 'column',
        border: 1,
        borderColor: colors.border,
        padding: 1,
        children: () => {
          text({ content: 'Variants: 14 semantic styles from theme', fg: colors.textBright })

          box({
            flexDirection: 'row',
            gap: 1,
            marginTop: 1,
            flexWrap: 'wrap',
            children: () => {
              const variants = ['primary', 'secondary', 'success', 'warning', 'error', 'info', 'muted'] as const

              for (const v of variants) {
                box({
                  variant: v,
                  width: 10,
                  height: 2,
                  border: 1,
                  justifyContent: 'center',
                  alignItems: 'center',
                  children: () => text({ content: v }),
                })
              }
            },
          })
        },
      })
    },
  })
}

// =============================================================================
// SECTION 5: THEMES
// =============================================================================

function ThemesSection() {
  const themeNames = Object.keys(themes)
  const currentThemeIdx = signal(0)

  return box({
    id: 'themes-section',
    flexDirection: 'column',
    gap: 1,
    children: () => {
      text({ content: '6. THEMES', fg: colors.accent })
      text({ content: '13 preset themes + reactive theme colors via t.*', fg: colors.textMuted })

      box({
        marginTop: 1,
        flexDirection: 'column',
        border: 1,
        borderColor: colors.border,
        padding: 1,
        children: () => {
          text({ content: 'Click a theme to apply:', fg: colors.textBright })

          box({
            flexDirection: 'row',
            gap: 1,
            marginTop: 1,
            flexWrap: 'wrap',
            children: () => {
              for (let i = 0; i < themeNames.length; i++) {
                const idx = i
                const name = themeNames[i]!
                box({
                  width: 12,
                  height: 1,
                  bg: () => currentThemeIdx.value === idx ? colors.accent : colors.bgHighlight,
                  fg: () => currentThemeIdx.value === idx ? colors.bg : colors.text,
                  border: 1,
                  borderColor: colors.border,
                  justifyContent: 'center',
                  focusable: true,
                  onClick: () => {
                    currentThemeIdx.value = idx
                    setTheme(name as keyof typeof themes)
                  },
                  children: () => text({ content: name }),
                })
              }
            },
          })
        },
      })

      // Theme-reactive sample
      box({
        marginTop: 1,
        flexDirection: 'column',
        border: 1,
        borderColor: t.border,
        bg: t.surface,
        padding: 1,
        children: () => {
          text({ content: 'Theme-reactive box (uses t.* colors):', fg: t.text })

          box({
            flexDirection: 'row',
            gap: 2,
            marginTop: 1,
            children: () => {
              box({
                variant: 'primary',
                padding: 1,
                border: 1,
                children: () => text({ content: 'Primary' }),
              })
              box({
                variant: 'success',
                padding: 1,
                border: 1,
                children: () => text({ content: 'Success' }),
              })
              box({
                variant: 'error',
                padding: 1,
                border: 1,
                children: () => text({ content: 'Error' }),
              })
            },
          })
        },
      })
    },
  })
}

// =============================================================================
// SECTION 6: ANIMATIONS
// =============================================================================

function AnimationsSection() {
  const animationActive = signal(true)

  return box({
    id: 'animations-section',
    flexDirection: 'column',
    gap: 1,
    children: () => {
      text({ content: '7. ANIMATIONS', fg: colors.accent })
      text({ content: 'cycle() returns a signal that updates on interval', fg: colors.textMuted })

      // Built-in frames
      box({
        marginTop: 1,
        flexDirection: 'column',
        border: 1,
        borderColor: colors.border,
        padding: 1,
        children: () => {
          text({ content: 'Frames.* - 8 built-in animation sets:', fg: colors.textBright })

          box({
            flexDirection: 'row',
            gap: 3,
            marginTop: 1,
            children: () => {
              const frameTypes: Array<[string, readonly string[]]> = [
                ['spinner', Frames.spinner],
                ['dots', Frames.dots],
                ['line', Frames.line],
                ['bar', Frames.bar],
                ['bounce', Frames.bounce],
                ['arrow', Frames.arrow],
                ['pulse', Frames.pulse],
              ]

              for (const [name, frames] of frameTypes) {
                box({
                  flexDirection: 'column',
                  alignItems: 'center',
                  width: 8,
                  children: () => {
                    text({
                      content: cycle(frames, { fps: 10, active: animationActive }),
                      fg: colors.accent,
                    })
                    text({ content: name, fg: colors.textMuted })
                  },
                })
              }
            },
          })
        },
      })

      // FPS comparison
      box({
        marginTop: 1,
        flexDirection: 'column',
        border: 1,
        borderColor: colors.border,
        padding: 1,
        children: () => {
          text({ content: 'FPS control:', fg: colors.textBright })

          box({
            flexDirection: 'row',
            gap: 3,
            marginTop: 1,
            children: () => {
              for (const fps of [2, 8, 16, 24]) {
                box({
                  flexDirection: 'column',
                  alignItems: 'center',
                  children: () => {
                    text({
                      content: cycle(Frames.spinner, { fps, active: animationActive }),
                      fg: colors.text,
                    })
                    text({ content: `${fps} fps`, fg: colors.textMuted })
                  },
                })
              }
            },
          })
        },
      })

      // Pause/resume
      box({
        marginTop: 1,
        flexDirection: 'row',
        alignItems: 'center',
        gap: 2,
        children: () => {
          box({
            width: 12,
            height: 1,
            bg: colors.bgHighlight,
            border: 1,
            borderColor: colors.border,
            justifyContent: 'center',
            focusable: true,
            onClick: () => { animationActive.value = !animationActive.value },
            children: () => text({ content: () => animationActive.value ? 'Pause' : 'Resume' }),
          })
          text({
            content: () => `Animations: ${animationActive.value ? 'Running' : 'Paused'}`,
            fg: colors.textMuted,
          })
        },
      })
    },
  })
}

// =============================================================================
// SECTION 7: KEYBOARD
// =============================================================================

function KeyboardSection() {
  const lastKey = signal('(none)')
  const keyCount = signal(0)

  // Section-specific key handler
  on((event: KeyEvent) => {
    if (currentSection.value !== 7) return false
    if (!isPress(event)) return false

    const char = getChar(event)
    if (char) {
      lastKey.value = char
    } else {
      // Special keys
      if (event.keycode === 0x1b5b41) lastKey.value = 'ArrowUp'
      else if (event.keycode === 0x1b5b42) lastKey.value = 'ArrowDown'
      else if (event.keycode === 0x1b5b43) lastKey.value = 'ArrowRight'
      else if (event.keycode === 0x1b5b44) lastKey.value = 'ArrowLeft'
      else if (event.keycode === 13) lastKey.value = 'Enter'
      else if (event.keycode === 27) lastKey.value = 'Escape'
      else if (event.keycode === 32) lastKey.value = 'Space'
      else lastKey.value = `keycode:${event.keycode}`
    }
    keyCount.value++

    return false // Don't consume, let other handlers see it
  })

  return box({
    id: 'keyboard-section',
    flexDirection: 'column',
    gap: 1,
    children: () => {
      text({ content: '8. KEYBOARD & FOCUS', fg: colors.accent })
      text({ content: 'on() for global keys, onKey on focusable boxes', fg: colors.textMuted })

      box({
        marginTop: 1,
        flexDirection: 'column',
        border: 1,
        borderColor: colors.border,
        padding: 1,
        children: () => {
          text({ content: 'Press any key:', fg: colors.textBright })

          box({
            flexDirection: 'row',
            gap: 4,
            marginTop: 1,
            children: () => {
              text({ content: () => `Last key: ${lastKey.value}`, fg: colors.text })
              text({ content: () => `Total: ${keyCount.value}`, fg: colors.textMuted })
            },
          })
        },
      })

      box({
        marginTop: 1,
        flexDirection: 'column',
        border: 1,
        borderColor: colors.border,
        padding: 1,
        children: () => {
          text({ content: 'KeyEvent structure:', fg: colors.textBright })
          text({ content: '  keycode: number (Unicode codepoint or special)', fg: colors.textMuted })
          text({ content: '  modifiers: CTRL(1) | ALT(2) | SHIFT(4) | META(8)', fg: colors.textMuted })
          text({ content: '  keyState: PRESS(0) | REPEAT(1) | RELEASE(2)', fg: colors.textMuted })
        },
      })

      box({
        marginTop: 1,
        flexDirection: 'column',
        border: 1,
        borderColor: colors.border,
        padding: 1,
        children: () => {
          text({ content: 'Focus: focusable prop + onFocus/onBlur callbacks', fg: colors.textBright })

          box({
            flexDirection: 'row',
            gap: 2,
            marginTop: 1,
            children: () => {
              for (let i = 1; i <= 3; i++) {
                const idx = i
                box({
                  id: `focus-box-${i}`,
                  width: 15,
                  height: 3,
                  bg: colors.bgHighlight,
                  border: 1,
                  borderColor: colors.border,
                  justifyContent: 'center',
                  alignItems: 'center',
                  focusable: true,
                  onFocus: () => { lastKey.value = `Focus ${idx}` },
                  onBlur: () => { lastKey.value = `Blur ${idx}` },
                  children: () => text({ content: `Focusable ${i}` }),
                })
              }
            },
          })
        },
      })
    },
  })
}

// =============================================================================
// SECTION 8: MOUSE
// =============================================================================

function MouseSection() {
  const clickCount = signal(0)
  const lastEvent = signal('(none)')
  const hovered = signal(false)

  return box({
    id: 'mouse-section',
    flexDirection: 'column',
    gap: 1,
    children: () => {
      text({ content: '9. MOUSE EVENTS', fg: colors.accent })
      text({ content: 'onClick, onMouseDown/Up, onMouseEnter/Leave', fg: colors.textMuted })

      box({
        marginTop: 1,
        flexDirection: 'column',
        border: 1,
        borderColor: colors.border,
        padding: 1,
        children: () => {
          text({ content: 'Click the boxes:', fg: colors.textBright })

          box({
            flexDirection: 'row',
            gap: 2,
            marginTop: 1,
            children: () => {
              // Click counter
              box({
                width: 15,
                height: 3,
                bg: colors.bgHighlight,
                border: 1,
                borderColor: colors.accent,
                justifyContent: 'center',
                alignItems: 'center',
                onClick: () => {
                  clickCount.value++
                  lastEvent.value = 'click'
                },
                children: () => text({ content: () => `Clicks: ${clickCount.value}` }),
              })

              // Hover box
              box({
                width: 15,
                height: 3,
                bg: () => hovered.value ? colors.accent : colors.bgHighlight,
                border: 1,
                borderColor: colors.border,
                justifyContent: 'center',
                alignItems: 'center',
                onMouseEnter: () => {
                  hovered.value = true
                  lastEvent.value = 'mouseEnter'
                },
                onMouseLeave: () => {
                  hovered.value = false
                  lastEvent.value = 'mouseLeave'
                },
                children: () => text({
                  content: () => hovered.value ? 'Hovered!' : 'Hover me',
                  fg: () => hovered.value ? colors.bg : colors.text,
                }),
              })

              // Down/Up box
              box({
                width: 15,
                height: 3,
                bg: colors.bgHighlight,
                border: 1,
                borderColor: colors.border,
                justifyContent: 'center',
                alignItems: 'center',
                onMouseDown: () => { lastEvent.value = 'mouseDown' },
                onMouseUp: () => { lastEvent.value = 'mouseUp' },
                children: () => text({ content: 'Down/Up' }),
              })
            },
          })

          text({
            content: () => `Last event: ${lastEvent.value}`,
            fg: colors.textMuted,
            marginTop: 1,
          })
        },
      })

      box({
        marginTop: 1,
        flexDirection: 'column',
        border: 1,
        borderColor: colors.border,
        padding: 1,
        children: () => {
          text({ content: 'MouseEvent structure:', fg: colors.textBright })
          text({ content: '  x, y: number (cell coordinates)', fg: colors.textMuted })
          text({ content: '  button: 0=left, 1=middle, 2=right', fg: colors.textMuted })
          text({ content: '  componentIndex: target component', fg: colors.textMuted })
        },
      })
    },
  })
}

// =============================================================================
// SECTION 9: SCROLL
// =============================================================================

function ScrollSection() {
  return box({
    id: 'scroll-section',
    flexDirection: 'column',
    gap: 1,
    children: () => {
      text({ content: '10. SCROLL', fg: colors.accent })
      text({ content: "overflow: 'scroll' or 'auto' makes container scrollable", fg: colors.textMuted })

      box({
        marginTop: 1,
        flexDirection: 'row',
        gap: 2,
        children: () => {
          // Vertical scroll
          box({
            flexDirection: 'column',
            children: () => {
              text({ content: 'Vertical (arrow keys):', fg: colors.textBright })

              box({
                width: 30,
                height: 8,
                overflow: 'scroll',
                border: 1,
                borderColor: colors.border,
                bg: colors.bgHighlight,
                focusable: true,
                flexDirection: 'column',
                children: () => {
                  for (let i = 1; i <= 20; i++) {
                    text({ content: `Line ${i}: Scroll content...`, fg: colors.text })
                  }
                },
              })
            },
          })

          // Horizontal scroll
          box({
            flexDirection: 'column',
            children: () => {
              text({ content: 'Horizontal:', fg: colors.textBright })

              box({
                width: 30,
                height: 8,
                overflow: 'scroll',
                border: 1,
                borderColor: colors.border,
                bg: colors.bgHighlight,
                focusable: true,
                flexDirection: 'column',
                children: () => {
                  for (let i = 1; i <= 10; i++) {
                    text({
                      content: `Line ${i}: ${'This is a very long line that extends beyond the visible area '.repeat(2)}`,
                      fg: colors.text,
                    })
                  }
                },
              })
            },
          })
        },
      })

      box({
        marginTop: 1,
        flexDirection: 'column',
        border: 1,
        borderColor: colors.border,
        padding: 1,
        children: () => {
          text({ content: 'Scroll controls:', fg: colors.textBright })
          text({ content: '  Arrow keys: Scroll by 1', fg: colors.textMuted })
          text({ content: '  Page Up/Down: Scroll by viewport', fg: colors.textMuted })
          text({ content: '  Home/End: Jump to start/end', fg: colors.textMuted })
          text({ content: '  Mouse wheel: Scroll by 3', fg: colors.textMuted })
        },
      })
    },
  })
}

// =============================================================================
// KEEP ALIVE
// =============================================================================

console.log('[api-showcase] App mounted - Tab to switch sections, Ctrl+C to exit')
await new Promise(() => {})
