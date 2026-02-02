/**
 * SparkTUI — Complete API Showcase & Test
 *
 * This file exercises EVERY user-facing API to verify it works.
 * It serves as both a test suite and documentation source.
 *
 * Sections:
 *   1. Signals & Reactivity
 *   2. Primitives (box, text, input)
 *   3. Control Flow (each, show, when)
 *   4. Layout (flexbox, dimensions, spacing)
 *   5. Styling (colors, borders, variants)
 *   6. Themes
 *   7. Events (keyboard, mouse, focus)
 *   8. Scroll
 *   9. Animation (cycle, pulse)
 *  10. Lifecycle (onMount, onDestroy)
 *
 * Controls:
 *   Tab / Shift+Tab    Navigate focus
 *   Arrow keys         Scroll (when focused on scrollable)
 *   Enter / Space      Activate button
 *   1-9                Switch demo section
 *   t                  Cycle theme
 *   q / Ctrl+C         Quit
 */

import { signal, derived, effect } from '@rlabs-inc/signals'
import {
  box,
  text,
  input,
  each,
  show,
  // when,  // TODO: Uncomment when async primitives ready
  cycle,
  Frames,
  scoped,
  onCleanup,
} from '../ts/primitives'
import { t, themes, setTheme, getThemeNames } from '../ts/state/theme'
import { mount } from '../ts/engine'
import { onMount, onDestroy } from '../ts/engine/lifecycle'
import { focus, focusedIndex } from '../ts/state/focus'
import { getBuffer } from '../ts/bridge'
import {
  isEnter,
  isSpace,
  getChar,
  hasCtrl,
  hasShift,
  type KeyEvent,
} from '../ts/engine/events'
import { BorderStyle, Attr } from '../ts/types'

// =============================================================================
// REACTIVE STATE
// =============================================================================

// Current demo section
const currentSection = signal(1)
const sectionNames = [
  '',
  '1. Signals & Reactivity',
  '2. Primitives',
  '3. Control Flow',
  '4. Layout',
  '5. Styling',
  '6. Themes',
  '7. Events',
  '8. Scroll',
  '9. Animation',
]

// Theme cycling
const themeNames = getThemeNames()
const themeIndex = signal(0)

// Section 1: Signals
const counter = signal(0)
const multiplied = derived(() => counter.value * 2)
const message = derived(() => `Count: ${counter.value}, Doubled: ${multiplied.value}`)

// Section 2: Primitives
const inputValue = signal('')
const inputValue2 = signal('')

// Section 3: Control Flow
const showConditional = signal(true)
const listItems = signal([
  { id: '1', name: 'Apple' },
  { id: '2', name: 'Banana' },
  { id: '3', name: 'Cherry' },
])

// Section 7: Events
const lastKey = signal('(none)')
const lastMouse = signal('(none)')
const clickCount = signal(0)

// Section 8: Scroll
const scrollContent = Array.from({ length: 50 }, (_, i) => `Line ${i + 1}`)

// Section 9: Animation
const animationActive = signal(true)

// =============================================================================
// HELPER: Section Header
// =============================================================================

function sectionHeader(title: string) {
  box({
    width: '100%',
    padding: 1,
    bg: t.surface,
    border: BorderStyle.DOUBLE,
    borderColor: t.primary,
    children: () => {
      text({ content: title, fg: t.primary, attrs: Attr.BOLD })
    },
  })
}

// =============================================================================
// SECTION 1: SIGNALS & REACTIVITY
// =============================================================================

function Section1_Signals() {
  return box({
    flexDirection: 'column',
    gap: 1,
    children: () => {
      sectionHeader('1. Signals & Reactivity')

      box({
        flexDirection: 'column',
        gap: 1,
        padding: 1,
        children: () => {
          // Static text
          text({ content: 'signal() creates reactive state', fg: t.textMuted })

          // Reactive counter display
          text({
            content: message, // Direct signal binding
            fg: t.text,
          })

          // Inline derived (getter function)
          text({
            content: () => `Squared: ${counter.value * counter.value}`,
            fg: t.text,
          })

          // Buttons
          box({
            flexDirection: 'row',
            gap: 2,
            children: () => {
              // Decrement
              box({
                width: 5,
                height: 1,
                bg: t.error,
                fg: t.textBright,
                focusable: true,
                justifyContent: 'center',
                onClick: () => { counter.value-- },
                onKey: (e) => {
                  if (isEnter(e) || isSpace(e)) {
                    counter.value--
                    return true
                  }
                },
                children: () => text({ content: ' - ' }),
              })

              // Increment
              box({
                width: 5,
                height: 1,
                bg: t.success,
                fg: t.textBright,
                focusable: true,
                justifyContent: 'center',
                onClick: () => { counter.value++ },
                onKey: (e) => {
                  if (isEnter(e) || isSpace(e)) {
                    counter.value++
                    return true
                  }
                },
                children: () => text({ content: ' + ' }),
              })
            },
          })

          text({ content: 'Click buttons or use Enter/Space when focused', fg: t.textMuted })
        },
      })
    },
  })
}

// =============================================================================
// SECTION 2: PRIMITIVES (box, text, input)
// =============================================================================

function Section2_Primitives() {
  return box({
    flexDirection: 'column',
    gap: 1,
    children: () => {
      sectionHeader('2. Primitives: box, text, input')

      box({
        flexDirection: 'column',
        gap: 1,
        padding: 1,
        children: () => {
          // Text with attributes
          text({ content: 'Normal text', fg: t.text })
          text({ content: 'Bold text', fg: t.text, attrs: Attr.BOLD })
          text({ content: 'Italic text', fg: t.text, attrs: Attr.ITALIC })
          text({ content: 'Underline text', fg: t.text, attrs: Attr.UNDERLINE })
          text({ content: 'Combined', fg: t.primary, attrs: Attr.BOLD | Attr.UNDERLINE })

          // Text alignment
          box({
            width: 40,
            border: 1,
            borderColor: t.textMuted,
            flexDirection: 'column',
            children: () => {
              text({ content: 'Left aligned', align: 'left', fg: t.text })
              text({ content: 'Center aligned', align: 'center', fg: t.text })
              text({ content: 'Right aligned', align: 'right', fg: t.text })
            },
          })

          // Input fields
          text({ content: 'Input fields:', fg: t.textMuted })

          box({
            flexDirection: 'row',
            gap: 2,
            children: () => {
              text({ content: 'Name:', fg: t.text })
              input({
                value: inputValue,
                width: 20,
                placeholder: 'Type here...',
                border: 1,
                borderColor: t.primary,
                onSubmit: (val) => { inputValue2.value = `Submitted: ${val}` },
              })
            },
          })

          // Show input result
          text({
            content: () => inputValue2.value || '(Press Enter to submit)',
            fg: t.textMuted,
          })
        },
      })
    },
  })
}

// =============================================================================
// SECTION 3: CONTROL FLOW (each, show)
// =============================================================================

function Section3_ControlFlow() {
  return box({
    flexDirection: 'column',
    gap: 1,
    children: () => {
      sectionHeader('3. Control Flow: each, show')

      box({
        flexDirection: 'column',
        gap: 1,
        padding: 1,
        children: () => {
          // show() demo
          text({ content: 'show() - Conditional rendering:', fg: t.textMuted })

          box({
            flexDirection: 'row',
            gap: 2,
            alignItems: 'center',
            children: () => {
              box({
                width: 10,
                height: 1,
                bg: t.primary,
                fg: t.textBright,
                focusable: true,
                justifyContent: 'center',
                onClick: () => { showConditional.value = !showConditional.value },
                onKey: (e) => {
                  if (isEnter(e) || isSpace(e)) {
                    showConditional.value = !showConditional.value
                    return true
                  }
                },
                children: () => text({ content: 'Toggle' }),
              })

              show(
                () => showConditional.value,
                () => text({ content: '✓ Visible!', fg: t.success }),
                () => text({ content: '✗ Hidden', fg: t.error })
              )
            },
          })

      // each() demo
      text({ content: 'each() - List rendering:', fg: t.textMuted })

      box({
        flexDirection: 'column',
        gap: 0,
        border: 1,
        borderColor: t.textMuted,
        padding: 1,
        children: () => {
          each(
            () => listItems.value,
            (getItem, key) => {
              return box({
                children: () => {
                  text({
                    content: () => `• ${getItem().name}`,
                    fg: t.text,
                  })
                },
              })
            },
            { key: (item) => item.id }
          )
        },
      })

      // Add/remove buttons
      box({
        flexDirection: 'row',
        gap: 2,
        children: () => {
          box({
            width: 12,
            height: 1,
            bg: t.success,
            fg: t.textBright,
            focusable: true,
            justifyContent: 'center',
            onClick: () => {
              const id = String(Date.now())
              listItems.value = [...listItems.value, { id, name: `Item ${id.slice(-4)}` }]
            },
            onKey: (e) => {
              if (isEnter(e) || isSpace(e)) {
                const id = String(Date.now())
                listItems.value = [...listItems.value, { id, name: `Item ${id.slice(-4)}` }]
                return true
              }
            },
            children: () => text({ content: 'Add Item' }),
          })

          box({
            width: 12,
            height: 1,
            bg: t.error,
            fg: t.textBright,
            focusable: true,
            justifyContent: 'center',
            onClick: () => {
              listItems.value = listItems.value.slice(0, -1)
            },
            onKey: (e) => {
              if (isEnter(e) || isSpace(e)) {
                listItems.value = listItems.value.slice(0, -1)
                return true
              }
            },
            children: () => text({ content: 'Remove' }),
          })
        },
      })
        },
      })
    },
  })
}

// =============================================================================
// SECTION 4: LAYOUT (flexbox, dimensions, spacing)
// =============================================================================

function Section4_Layout() {
  return box({
    flexDirection: 'column',
    gap: 1,
    children: () => {
      sectionHeader('4. Layout: Flexbox, Dimensions, Spacing')

      box({
        flexDirection: 'column',
        gap: 1,
        padding: 1,
    children: () => {
      // Flex direction
      text({ content: 'flexDirection: row vs column', fg: t.textMuted })

      box({
        flexDirection: 'row',
        gap: 2,
        children: () => {
          // Row
          box({
            flexDirection: 'row',
            gap: 1,
            border: 1,
            borderColor: t.primary,
            padding: 1,
            children: () => {
              text({ content: 'A', fg: t.text })
              text({ content: 'B', fg: t.text })
              text({ content: 'C', fg: t.text })
            },
          })

          // Column
          box({
            flexDirection: 'column',
            gap: 0,
            border: 1,
            borderColor: t.secondary,
            padding: 1,
            children: () => {
              text({ content: 'A', fg: t.text })
              text({ content: 'B', fg: t.text })
              text({ content: 'C', fg: t.text })
            },
          })
        },
      })

      // Justify content
      text({ content: 'justifyContent options:', fg: t.textMuted })

      const justifyOptions: Array<'flex-start' | 'center' | 'flex-end' | 'space-between'> = [
        'flex-start', 'center', 'flex-end', 'space-between'
      ]

      box({
        flexDirection: 'column',
        gap: 1,
        children: () => {
          for (const justify of justifyOptions) {
            box({
              flexDirection: 'row',
              gap: 1,
              children: () => {
                text({ content: justify.padEnd(15), fg: t.textMuted, width: 15 })
                box({
                  width: 30,
                  flexDirection: 'row',
                  justifyContent: justify,
                  border: 1,
                  borderColor: t.textMuted,
                  children: () => {
                    box({ width: 3, height: 1, bg: t.primary })
                    box({ width: 3, height: 1, bg: t.secondary })
                    box({ width: 3, height: 1, bg: t.success })
                  },
                })
              },
            })
          }
        },
      })

      // Grow
      text({ content: 'grow: flex items filling space', fg: t.textMuted })

      box({
        width: 50,
        flexDirection: 'row',
        border: 1,
        borderColor: t.textMuted,
        children: () => {
          box({ width: 10, height: 1, bg: t.error, children: () => text({ content: 'fixed' }) })
          box({ grow: 1, height: 1, bg: t.success, children: () => text({ content: 'grow:1' }) })
          box({ width: 10, height: 1, bg: t.warning, children: () => text({ content: 'fixed' }) })
        },
      })
        },
      })
    },
  })
}

// =============================================================================
// SECTION 5: STYLING (colors, borders, variants)
// =============================================================================

function Section5_Styling() {
  return box({
    flexDirection: 'column',
    gap: 1,
    children: () => {
      sectionHeader('5. Styling: Colors, Borders, Variants')

      box({
        flexDirection: 'column',
        gap: 1,
        padding: 1,
    children: () => {
      // Border styles
      text({ content: 'Border styles:', fg: t.textMuted })

      box({
        flexDirection: 'row',
        flexWrap: 'wrap',
        gap: 1,
        children: () => {
          const styles: Array<[string, number]> = [
            ['None', BorderStyle.NONE],
            ['Single', BorderStyle.SINGLE],
            ['Double', BorderStyle.DOUBLE],
            ['Rounded', BorderStyle.ROUNDED],
            ['Bold', BorderStyle.BOLD],
            ['Dashed', BorderStyle.DASHED],
            ['ASCII', BorderStyle.ASCII],
          ]

          for (const [name, style] of styles) {
            box({
              width: 10,
              height: 3,
              border: style,
              borderColor: t.primary,
              justifyContent: 'center',
              alignItems: 'center',
              children: () => text({ content: name, fg: t.text }),
            })
          }
        },
      })

      // Variants
      text({ content: 'Variants (semantic colors):', fg: t.textMuted })

      box({
        flexDirection: 'row',
        flexWrap: 'wrap',
        gap: 1,
        children: () => {
          const variants: Array<'primary' | 'secondary' | 'success' | 'warning' | 'error' | 'info'> = [
            'primary', 'secondary', 'success', 'warning', 'error', 'info'
          ]

          for (const variant of variants) {
            box({
              width: 12,
              height: 1,
              variant,
              justifyContent: 'center',
              children: () => text({ content: variant }),
            })
          }
        },
      })

      // Colors
      text({ content: 'Direct colors (theme reactive):', fg: t.textMuted })

      box({
        flexDirection: 'row',
        gap: 1,
        children: () => {
          box({ width: 8, height: 1, bg: t.primary, children: () => text({ content: 'primary' }) })
          box({ width: 10, height: 1, bg: t.secondary, children: () => text({ content: 'secondary' }) })
          box({ width: 8, height: 1, bg: t.success, children: () => text({ content: 'success' }) })
          box({ width: 8, height: 1, bg: t.warning, children: () => text({ content: 'warning' }) })
          box({ width: 6, height: 1, bg: t.error, children: () => text({ content: 'error' }) })
        },
      })
        },
      })
    },
  })
}

// =============================================================================
// SECTION 6: THEMES
// =============================================================================

function Section6_Themes() {
  return box({
    flexDirection: 'column',
    gap: 1,
    children: () => {
      sectionHeader('6. Themes')

      box({
        flexDirection: 'column',
        gap: 1,
        padding: 1,
        children: () => {
          text({ content: 'Press "t" to cycle themes', fg: t.textMuted })

          text({
            content: () => `Current theme: ${themeNames[themeIndex.value]}`,
            fg: t.primary,
            attrs: Attr.BOLD,
          })

          text({ content: 'Available themes:', fg: t.textMuted })

          box({
            flexDirection: 'row',
            flexWrap: 'wrap',
            gap: 1,
            children: () => {
              for (let i = 0; i < themeNames.length; i++) {
                const name = themeNames[i]
                box({
                  width: name.length + 2,
                  height: 1,
                  bg: () => themeIndex.value === i ? t.primary : t.surface,
                  fg: () => themeIndex.value === i ? t.textBright : t.text,
                  justifyContent: 'center',
                  children: () => text({ content: name }),
                })
              }
            },
          })

          // Theme color preview
          text({ content: 'Theme palette preview:', fg: t.textMuted })

          box({
            flexDirection: 'row',
            gap: 0,
            children: () => {
              box({ width: 4, height: 2, bg: t.primary })
              box({ width: 4, height: 2, bg: t.secondary })
              box({ width: 4, height: 2, bg: t.success })
              box({ width: 4, height: 2, bg: t.warning })
              box({ width: 4, height: 2, bg: t.error })
              box({ width: 4, height: 2, bg: t.info })
              box({ width: 4, height: 2, bg: t.surface })
              box({ width: 4, height: 2, bg: t.bg })
            },
          })
        },
      })
    },
  })
}

// =============================================================================
// SECTION 7: EVENTS (keyboard, mouse, focus)
// =============================================================================

function Section7_Events() {
  return box({
    flexDirection: 'column',
    gap: 1,
    children: () => {
      sectionHeader('7. Events: Keyboard, Mouse, Focus')

      box({
        flexDirection: 'column',
        gap: 1,
        padding: 1,
    children: () => {
      // Keyboard events
      text({ content: 'Keyboard events (press any key):', fg: t.textMuted })
      text({
        content: () => `Last key: ${lastKey.value}`,
        fg: t.text,
      })

      // Mouse events
      text({ content: 'Mouse events:', fg: t.textMuted })
      text({
        content: () => `Last mouse: ${lastMouse.value}`,
        fg: t.text,
      })

      // Click counter
      box({
        flexDirection: 'row',
        gap: 2,
        alignItems: 'center',
        children: () => {
          box({
            width: 15,
            height: 3,
            bg: t.primary,
            fg: t.textBright,
            focusable: true,
            justifyContent: 'center',
            alignItems: 'center',
            border: 1,
            borderColor: t.secondary,
            onClick: () => {
              clickCount.value++
              lastMouse.value = 'Click!'
            },
            onMouseEnter: () => { lastMouse.value = 'Enter' },
            onMouseLeave: () => { lastMouse.value = 'Leave' },
            onKey: (e) => {
              if (isEnter(e) || isSpace(e)) {
                clickCount.value++
                return true
              }
            },
            children: () => text({ content: 'Click Me!' }),
          })

          text({
            content: () => `Clicks: ${clickCount.value}`,
            fg: t.text,
          })
        },
      })

      // Focus demo
      text({ content: 'Focus (Tab to navigate):', fg: t.textMuted })

      box({
        flexDirection: 'row',
        gap: 1,
        children: () => {
          for (let i = 1; i <= 4; i++) {
            box({
              id: `focus-demo-${i}`,
              width: 10,
              height: 1,
              bg: t.surface,
              fg: t.text,
              border: 1,
              borderColor: t.textMuted,
              focusable: true,
              justifyContent: 'center',
              children: () => text({ content: `Box ${i}` }),
            })
          }
        },
      })

      text({
        content: () => `Focused index: ${focusedIndex.value}`,
        fg: t.textMuted,
      })
        },
      })
    },
  })
}

// =============================================================================
// SECTION 8: SCROLL
// =============================================================================

function Section8_Scroll() {
  return box({
    flexDirection: 'column',
    gap: 1,
    children: () => {
      sectionHeader('8. Scroll')

      box({
        flexDirection: 'column',
        gap: 1,
        padding: 1,
        children: () => {
          text({ content: 'Scrollable container (Tab to focus, arrows to scroll):', fg: t.textMuted })

          // Scrollable box
          box({
            width: 40,
            height: 8,
            overflow: 'scroll',
            border: 1,
            borderColor: t.primary,
            flexDirection: 'column',
            children: () => {
              for (const line of scrollContent) {
                text({ content: line, fg: t.text })
              }
            },
          })

          text({ content: 'Use Arrow keys, Page Up/Down, Home/End', fg: t.textMuted })
          text({ content: 'Mouse wheel also works in fullscreen mode', fg: t.textMuted })
        },
      })
    },
  })
}

// =============================================================================
// SECTION 9: ANIMATION
// =============================================================================

function Section9_Animation() {
  return box({
    flexDirection: 'column',
    gap: 1,
    children: () => {
      sectionHeader('9. Animation: cycle, pulse')

      box({
        flexDirection: 'column',
        gap: 1,
        padding: 1,
    children: () => {
      text({ content: 'cycle() - Frame-based animation:', fg: t.textMuted })

      box({
        flexDirection: 'row',
        gap: 3,
        children: () => {
          // Spinner
          box({
            flexDirection: 'row',
            gap: 1,
            children: () => {
              text({
                content: cycle(Frames.spinner, { fps: 12, active: animationActive }),
                fg: t.primary,
              })
              text({ content: 'Loading...', fg: t.text })
            },
          })

          // Dots
          box({
            flexDirection: 'row',
            gap: 1,
            children: () => {
              text({
                content: cycle(Frames.dots, { fps: 8, active: animationActive }),
                fg: t.secondary,
              })
              text({ content: 'Processing', fg: t.text })
            },
          })

          // Bounce
          box({
            flexDirection: 'row',
            gap: 1,
            children: () => {
              text({
                content: cycle(Frames.bounce, { fps: 6, active: animationActive }),
                fg: t.success,
              })
              text({ content: 'Bounce', fg: t.text })
            },
          })
        },
      })

      // Toggle animation
      box({
        flexDirection: 'row',
        gap: 2,
        alignItems: 'center',
        children: () => {
          box({
            width: 15,
            height: 1,
            bg: () => animationActive.value ? t.success : t.error,
            fg: t.textBright,
            focusable: true,
            justifyContent: 'center',
            onClick: () => { animationActive.value = !animationActive.value },
            onKey: (e) => {
              if (isEnter(e) || isSpace(e)) {
                animationActive.value = !animationActive.value
                return true
              }
            },
            children: () => text({
              content: () => animationActive.value ? 'Pause' : 'Resume',
            }),
          })

          text({
            content: () => `Animation: ${animationActive.value ? 'ON' : 'OFF'}`,
            fg: t.textMuted,
          })
        },
      })

      // Available frame sets
      text({ content: 'Available Frames:', fg: t.textMuted })
      text({
        content: 'spinner, dots, line, bar, clock, bounce, arrow, pulse',
        fg: t.text,
      })
        },
      })
    },
  })
}

// =============================================================================
// MAIN APP
// =============================================================================

await mount(() => {
  box({
    flexDirection: 'column',
    children: () => {
      // Header
      box({
        width: '100%',
        padding: 1,
        bg: t.primary,
        fg: t.textBright,
        justifyContent: 'center',
        children: () => {
          text({
            content: 'SparkTUI Complete API Showcase',
            attrs: Attr.BOLD,
          })
        },
      })

      // Navigation
      box({
        width: '100%',
        padding: 1,
        bg: t.surface,
        flexDirection: 'row',
        justifyContent: 'center',
        gap: 1,
        children: () => {
          text({ content: 'Sections:', fg: t.textMuted })
          for (let i = 1; i <= 9; i++) {
            box({
              width: 3,
              height: 1,
              bg: () => currentSection.value === i ? t.primary : null,
              fg: () => currentSection.value === i ? t.textBright : t.text,
              justifyContent: 'center',
              children: () => text({ content: String(i) }),
            })
          }
          text({ content: '| t:theme q:quit', fg: t.textMuted })
        },
      })

      // Current section content
      box({
        flexDirection: 'column',
        padding: 1,
        children: () => {
          show(() => currentSection.value === 1, Section1_Signals)
          show(() => currentSection.value === 2, Section2_Primitives)
          show(() => currentSection.value === 3, Section3_ControlFlow)
          show(() => currentSection.value === 4, Section4_Layout)
          show(() => currentSection.value === 5, Section5_Styling)
          show(() => currentSection.value === 6, Section6_Themes)
          show(() => currentSection.value === 7, Section7_Events)
          show(() => currentSection.value === 8, Section8_Scroll)
          show(() => currentSection.value === 9, Section9_Animation)
        },
      })

      // Footer
      box({
        width: '100%',
        padding: 1,
        bg: t.surface,
        justifyContent: 'center',
        children: () => {
          text({
            content: () => `Theme: ${themeNames[themeIndex.value]} | Section: ${sectionNames[currentSection.value]}`,
            fg: t.textMuted,
          })
        },
      })
    },

    // Global keyboard handler
    onKey: (key: KeyEvent) => {
      const ch = getChar(key)

      // Update last key display
      const mods: string[] = []
      if (hasCtrl(key)) mods.push('Ctrl')
      if (hasShift(key)) mods.push('Shift')
      const modStr = mods.length ? mods.join('+') + '+' : ''
      lastKey.value = `${modStr}${ch || `code:${key.keycode}`}`

      // Section navigation (1-9)
      if (ch! >= '1' && ch! <= '9') {
        currentSection.value = parseInt(ch!)
        return true
      }

      // Theme cycling
      if (ch === 't' || ch === 'T') {
        themeIndex.value = (themeIndex.value + 1) % themeNames.length
        setTheme(themeNames[themeIndex.value] as keyof typeof themes)
        return true
      }

      // Quit
      if (ch === 'q' || ch === 'Q') {
        process.exit(0)
      }

      return false
    },
  })
}, { mode: 'inline' })
