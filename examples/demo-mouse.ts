/**
 * SparkTUI Mouse Interaction Playground
 *
 * Demonstrates full mouse support:
 * - Live mouse position tracking
 * - Hover effects with color changes
 * - Click counters
 * - Drag visualization (mouse down state)
 * - Colorful button grid
 * - Tooltip simulation
 *
 * Run: bun run examples/demo-mouse.ts
 */

import { signal, derived } from '@rlabs-inc/signals'
import { mount } from '../ts/engine/mount'
import { box } from '../ts/primitives/box'
import { text } from '../ts/primitives/text'
import {
  mouseX,
  mouseY,
  isMouseDown,
  getButtonName,
  onGlobalMouse,
} from '../ts/state/mouse'

// =============================================================================
// COLORS
// =============================================================================

const COLORS = {
  bg: { r: 20, g: 20, b: 35, a: 255 },
  cardBg: { r: 30, g: 30, b: 50, a: 255 },
  border: { r: 60, g: 80, b: 120, a: 255 },
  title: { r: 100, g: 200, b: 255, a: 255 },
  subtitle: { r: 140, g: 140, b: 180, a: 255 },
  text: { r: 200, g: 200, b: 220, a: 255 },
  dim: { r: 80, g: 80, b: 100, a: 255 },
  accent: { r: 100, g: 220, b: 180, a: 255 },
  warning: { r: 255, g: 180, b: 80, a: 255 },
  error: { r: 255, g: 100, b: 100, a: 255 },
  success: { r: 100, g: 255, b: 150, a: 255 },
}

// Color palette for buttons
const PALETTE = [
  { r: 255, g: 87, b: 51, a: 255 },   // Red-orange
  { r: 255, g: 165, b: 0, a: 255 },   // Orange
  { r: 255, g: 220, b: 0, a: 255 },   // Yellow
  { r: 50, g: 205, b: 50, a: 255 },   // Lime green
  { r: 0, g: 191, b: 255, a: 255 },   // Deep sky blue
  { r: 138, g: 43, b: 226, a: 255 },  // Blue violet
  { r: 255, g: 20, b: 147, a: 255 },  // Deep pink
  { r: 255, g: 255, b: 255, a: 255 }, // White
]

// =============================================================================
// REACTIVE STATE
// =============================================================================

// Click counters for three boxes
const counter1 = signal(0)
const counter2 = signal(0)
const counter3 = signal(0)

// Hover states for four boxes
const hovered1 = signal(false)
const hovered2 = signal(false)
const hovered3 = signal(false)
const hovered4 = signal(false)

// Selected color from palette
const selectedColorIndex = signal(0)

// Tooltip state
const tooltipVisible = signal(false)
const tooltipText = signal('')

// Track last button pressed
const lastButton = signal('none')

// Track mouse state description
const mouseState = derived(() => {
  if (isMouseDown.value) return 'PRESSING'
  if (hovered1.value || hovered2.value || hovered3.value || hovered4.value) return 'HOVERING'
  return 'IDLE'
})

// Global mouse handler to track button
onGlobalMouse((event) => {
  if (event.button !== undefined) {
    lastButton.value = getButtonName(event)
  }
})

// =============================================================================
// MOUNT APP
// =============================================================================

const cols = process.stdout.columns || 80
const rows = process.stdout.rows || 24

const { unmount, setMode, getMode } = mount(() => {
  // Root container - full terminal
  box({
    width: cols,
    height: rows,
    flexDirection: 'column',
    bg: COLORS.bg,
    children: () => {
      // Header
      box({
        width: '100%',
        flexDirection: 'row',
        justifyContent: 'center',
        padding: 1,
        border: 1,
        borderColor: COLORS.border,
        bg: COLORS.cardBg,
        children: () => {
          text({
            content: 'Mouse Interaction Playground',
            fg: COLORS.title,
          })
        },
      })

      // Main content area
      box({
        grow: 1,
        flexDirection: 'column',
        padding: 1,
        gap: 1,
        children: () => {
          // Status row
          box({
            flexDirection: 'row',
            gap: 2,
            children: () => {
              // Mouse position display
              box({
                flexDirection: 'row',
                children: () => {
                  text({ content: 'Mouse Position: ', fg: COLORS.subtitle })
                  text({
                    content: derived(() => `(${mouseX.value}, ${mouseY.value})`),
                    fg: COLORS.accent,
                  })
                },
              })

              // Button display
              box({
                flexDirection: 'row',
                children: () => {
                  text({ content: 'Button: ', fg: COLORS.subtitle })
                  text({
                    content: derived(() => lastButton.value),
                    fg: COLORS.warning,
                  })
                },
              })
            },
          })

          // State display
          box({
            flexDirection: 'row',
            children: () => {
              text({ content: 'State: ', fg: COLORS.subtitle })
              text({
                content: derived(() => `[${mouseState.value}]`),
                fg: derived(() => {
                  switch (mouseState.value) {
                    case 'PRESSING': return COLORS.error
                    case 'HOVERING': return COLORS.success
                    default: return COLORS.dim
                  }
                }),
              })
              text({
                content: derived(() => isMouseDown.value ? '  (mouse down!)' : ''),
                fg: COLORS.warning,
              })
            },
          })

          // Hover boxes section
          box({
            flexDirection: 'column',
            marginTop: 1,
            children: () => {
              text({ content: 'Hover over these boxes:', fg: COLORS.subtitle })
              box({
                flexDirection: 'row',
                gap: 1,
                marginTop: 1,
                children: () => {
                  // Hover box 1
                  box({
                    width: 12,
                    height: 4,
                    border: 1,
                    borderColor: COLORS.border,
                    justifyContent: 'center',
                    alignItems: 'center',
                    bg: derived(() => hovered1.value
                      ? { r: 80, g: 200, b: 120, a: 255 }
                      : COLORS.cardBg
                    ),
                    onMouseEnter: () => { hovered1.value = true },
                    onMouseLeave: () => { hovered1.value = false },
                    children: () => {
                      text({
                        content: derived(() => hovered1.value ? 'Hi!' : 'Hover'),
                        fg: derived(() => hovered1.value ? COLORS.bg : COLORS.text),
                      })
                      text({
                        content: 'Me!',
                        fg: derived(() => hovered1.value ? COLORS.bg : COLORS.text),
                      })
                    },
                  })

                  // Hover box 2
                  box({
                    width: 12,
                    height: 4,
                    border: 1,
                    borderColor: COLORS.border,
                    justifyContent: 'center',
                    alignItems: 'center',
                    bg: derived(() => hovered2.value
                      ? { r: 100, g: 150, b: 255, a: 255 }
                      : COLORS.cardBg
                    ),
                    onMouseEnter: () => { hovered2.value = true },
                    onMouseLeave: () => { hovered2.value = false },
                    children: () => {
                      text({
                        content: derived(() => hovered2.value ? 'Hello!' : 'Hover'),
                        fg: derived(() => hovered2.value ? COLORS.bg : COLORS.text),
                      })
                      text({
                        content: 'Me!',
                        fg: derived(() => hovered2.value ? COLORS.bg : COLORS.text),
                      })
                    },
                  })

                  // Hover box 3
                  box({
                    width: 12,
                    height: 4,
                    border: 1,
                    borderColor: COLORS.border,
                    justifyContent: 'center',
                    alignItems: 'center',
                    bg: derived(() => hovered3.value
                      ? { r: 255, g: 200, b: 100, a: 255 }
                      : COLORS.cardBg
                    ),
                    onMouseEnter: () => { hovered3.value = true },
                    onMouseLeave: () => { hovered3.value = false },
                    children: () => {
                      text({
                        content: derived(() => hovered3.value ? 'Yes!' : 'Hover'),
                        fg: derived(() => hovered3.value ? COLORS.bg : COLORS.text),
                      })
                      text({
                        content: 'Me!',
                        fg: derived(() => hovered3.value ? COLORS.bg : COLORS.text),
                      })
                    },
                  })

                  // Hover box 4
                  box({
                    width: 12,
                    height: 4,
                    border: 1,
                    borderColor: COLORS.border,
                    justifyContent: 'center',
                    alignItems: 'center',
                    bg: derived(() => hovered4.value
                      ? { r: 255, g: 120, b: 180, a: 255 }
                      : COLORS.cardBg
                    ),
                    onMouseEnter: () => { hovered4.value = true },
                    onMouseLeave: () => { hovered4.value = false },
                    children: () => {
                      text({
                        content: derived(() => hovered4.value ? 'Wow!' : 'Hover'),
                        fg: derived(() => hovered4.value ? COLORS.bg : COLORS.text),
                      })
                      text({
                        content: 'Me!',
                        fg: derived(() => hovered4.value ? COLORS.bg : COLORS.text),
                      })
                    },
                  })
                },
              })
            },
          })

          // Click counters section
          box({
            flexDirection: 'column',
            marginTop: 1,
            children: () => {
              text({ content: 'Click counters:', fg: COLORS.subtitle })
              box({
                flexDirection: 'row',
                gap: 1,
                marginTop: 1,
                children: () => {
                  // Counter box 1
                  box({
                    width: 12,
                    height: 4,
                    border: 1,
                    borderColor: COLORS.success,
                    justifyContent: 'center',
                    alignItems: 'center',
                    bg: COLORS.cardBg,
                    onClick: () => { counter1.value++ },
                    children: () => {
                      text({
                        content: derived(() => `${counter1.value}`),
                        fg: COLORS.success,
                      })
                      text({ content: 'clicks', fg: COLORS.dim })
                    },
                  })

                  // Counter box 2
                  box({
                    width: 12,
                    height: 4,
                    border: 1,
                    borderColor: COLORS.warning,
                    justifyContent: 'center',
                    alignItems: 'center',
                    bg: COLORS.cardBg,
                    onClick: () => { counter2.value++ },
                    children: () => {
                      text({
                        content: derived(() => `${counter2.value}`),
                        fg: COLORS.warning,
                      })
                      text({ content: 'clicks', fg: COLORS.dim })
                    },
                  })

                  // Counter box 3
                  box({
                    width: 12,
                    height: 4,
                    border: 1,
                    borderColor: COLORS.error,
                    justifyContent: 'center',
                    alignItems: 'center',
                    bg: COLORS.cardBg,
                    onClick: () => { counter3.value++ },
                    children: () => {
                      text({
                        content: derived(() => `${counter3.value}`),
                        fg: COLORS.error,
                      })
                      text({ content: 'clicks', fg: COLORS.dim })
                    },
                  })
                },
              })
            },
          })

          // Color palette section
          box({
            flexDirection: 'column',
            marginTop: 1,
            children: () => {
              text({ content: 'Color palette (click to select):', fg: COLORS.subtitle })
              box({
                flexDirection: 'row',
                gap: 1,
                marginTop: 1,
                children: () => {
                  // Create palette buttons
                  PALETTE.forEach((color, i) => {
                    box({
                      width: 4,
                      height: 2,
                      bg: color,
                      border: derived(() => selectedColorIndex.value === i ? 1 : 0),
                      borderColor: { r: 255, g: 255, b: 255, a: 255 },
                      onClick: () => { selectedColorIndex.value = i },
                      onMouseEnter: () => {
                        tooltipText.value = `#${color.r.toString(16).padStart(2, '0')}${color.g.toString(16).padStart(2, '0')}${color.b.toString(16).padStart(2, '0')}`.toUpperCase()
                        tooltipVisible.value = true
                      },
                      onMouseLeave: () => {
                        tooltipVisible.value = false
                      },
                    })
                  })
                },
              })

              // Selected color display
              box({
                flexDirection: 'row',
                marginTop: 1,
                children: () => {
                  text({ content: 'Selected: ', fg: COLORS.subtitle })
                  text({
                    content: derived(() => {
                      const c = PALETTE[selectedColorIndex.value]
                      return `#${c.r.toString(16).padStart(2, '0')}${c.g.toString(16).padStart(2, '0')}${c.b.toString(16).padStart(2, '0')}`.toUpperCase()
                    }),
                    fg: derived(() => PALETTE[selectedColorIndex.value]),
                  })
                },
              })

              // Tooltip (shows hex on hover)
              box({
                visible: tooltipVisible,
                flexDirection: 'row',
                marginTop: 1,
                children: () => {
                  text({ content: 'Hovering: ', fg: COLORS.dim })
                  text({
                    content: tooltipText,
                    fg: COLORS.accent,
                  })
                },
              })
            },
          })

          // Drag area
          box({
            flexDirection: 'column',
            marginTop: 1,
            children: () => {
              text({ content: 'Drag test area (hold mouse button):', fg: COLORS.subtitle })
              box({
                width: 40,
                height: 5,
                marginTop: 1,
                border: 1,
                borderColor: derived(() => isMouseDown.value ? COLORS.accent : COLORS.border),
                bg: derived(() => isMouseDown.value
                  ? { r: 50, g: 80, b: 60, a: 255 }
                  : COLORS.cardBg
                ),
                justifyContent: 'center',
                alignItems: 'center',
                children: () => {
                  text({
                    content: derived(() => isMouseDown.value
                      ? `Dragging at (${mouseX.value}, ${mouseY.value})`
                      : 'Click and hold to drag'
                    ),
                    fg: derived(() => isMouseDown.value ? COLORS.success : COLORS.dim),
                  })
                },
              })
            },
          })
        },
      })

      // Footer
      box({
        width: '100%',
        flexDirection: 'row',
        justifyContent: 'center',
        padding: 1,
        border: 1,
        borderColor: COLORS.border,
        bg: COLORS.cardBg,
        children: () => {
          text({
            content: 'Press Ctrl+C to exit',
            fg: COLORS.dim,
          })
        },
      })
    },
  })
}, {
  mode: 'fullscreen',
})

console.log('[demo-mouse] App mounted')

// Keep process alive
await new Promise(() => {})
