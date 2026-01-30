/**
 * SparkTUI - Cursor Customization Showcase
 *
 * Demonstrates all cursor customization options:
 * - Standard styles (block, bar, underline)
 * - Blink speeds (slow to fast, and no blink)
 * - Custom characters (thin bar, thick bar, arrow, star)
 * - Colored cursors (green, red, rainbow animation)
 *
 * Navigation: [Tab] to move between inputs
 * Exit: Ctrl+C
 */

import { signal } from '@rlabs-inc/signals'
import { mount } from '../ts/engine/mount'
import { box, text, input, cycle } from '../ts/primitives'
import { packColor } from '../ts/bridge/shared-buffer-aos'

// =============================================================================
// COLORS
// =============================================================================

const colors = {
  // Background colors
  panelBg: packColor(25, 25, 35, 255),
  sectionBg: packColor(35, 35, 50, 255),
  inputBg: packColor(45, 45, 65, 255),

  // Text colors
  title: packColor(120, 200, 255, 255),
  heading: packColor(180, 180, 220, 255),
  label: packColor(150, 150, 180, 255),
  dimmed: packColor(100, 100, 130, 255),
  bright: packColor(240, 240, 255, 255),

  // Border colors
  border: packColor(80, 100, 140, 255),
  borderDim: packColor(60, 70, 100, 255),

  // Cursor colors
  green: packColor(0, 255, 100, 255),
  greenDim: packColor(0, 100, 40, 255),
  red: packColor(255, 80, 80, 255),
  redDim: packColor(100, 30, 30, 255),

  // Rainbow colors for animated cursor
  rainbow: [
    packColor(255, 0, 0, 255),     // Red
    packColor(255, 165, 0, 255),   // Orange
    packColor(255, 255, 0, 255),   // Yellow
    packColor(0, 255, 0, 255),     // Green
    packColor(0, 100, 255, 255),   // Blue
    packColor(150, 0, 255, 255),   // Purple
  ],
}

// =============================================================================
// MOUNT APP
// =============================================================================

const cols = process.stdout.columns || 80
const rows = process.stdout.rows || 24

const { unmount, setMode, getMode } = mount(() => {
  // Root container
  box({
    width: cols,
    height: rows,
    flexDirection: 'column',
    bg: colors.panelBg,
    padding: 1,
    children: () => {
      // Header
      box({
        width: '100%',
        border: 1,
        borderColor: colors.border,
        padding: 1,
        paddingLeft: 2,
        paddingRight: 2,
        flexDirection: 'column',
        alignItems: 'center',
        children: () => {
          text({ content: 'Cursor Styles Showcase', fg: colors.title })
        },
      })

      // Main content area
      box({
        width: '100%',
        grow: 1,
        flexDirection: 'row',
        gap: 1,
        marginTop: 1,
        children: () => {
          // Left column
          box({
            grow: 1,
            flexDirection: 'column',
            gap: 1,
            children: () => {
              // Standard Styles section
              box({
                border: 1,
                borderColor: colors.borderDim,
                bg: colors.sectionBg,
                padding: 1,
                flexDirection: 'column',
                children: () => {
                  text({ content: 'Standard Styles', fg: colors.heading, marginBottom: 1 })

                  // Block cursor
                  box({
                    flexDirection: 'row',
                    alignItems: 'center',
                    marginBottom: 1,
                    children: () => {
                      text({ content: 'Block:     ', fg: colors.label, width: 12 })
                      input({
                        value: signal('Hello World'),
                        width: 30,
                        bg: colors.inputBg,
                        fg: colors.bright,
                        cursor: { style: 'block' },
                      })
                    },
                  })

                  // Bar cursor
                  box({
                    flexDirection: 'row',
                    alignItems: 'center',
                    marginBottom: 1,
                    children: () => {
                      text({ content: 'Bar:       ', fg: colors.label, width: 12 })
                      input({
                        value: signal('VS Code style'),
                        width: 30,
                        bg: colors.inputBg,
                        fg: colors.bright,
                        cursor: { style: 'bar' },
                      })
                    },
                  })

                  // Underline cursor
                  box({
                    flexDirection: 'row',
                    alignItems: 'center',
                    children: () => {
                      text({ content: 'Underline: ', fg: colors.label, width: 12 })
                      input({
                        value: signal('Classic terminal'),
                        width: 30,
                        bg: colors.inputBg,
                        fg: colors.bright,
                        cursor: { style: 'underline' },
                      })
                    },
                  })
                },
              })

              // Blink Speeds section
              box({
                border: 1,
                borderColor: colors.borderDim,
                bg: colors.sectionBg,
                padding: 1,
                flexDirection: 'column',
                children: () => {
                  text({ content: 'Blink Speeds', fg: colors.heading, marginBottom: 1 })

                  // 1 FPS (slow)
                  box({
                    flexDirection: 'row',
                    alignItems: 'center',
                    marginBottom: 1,
                    children: () => {
                      text({ content: '1 FPS:     ', fg: colors.label, width: 12 })
                      input({
                        value: signal('Slow blink'),
                        width: 30,
                        bg: colors.inputBg,
                        fg: colors.bright,
                        cursor: { blink: { fps: 1 } },
                      })
                    },
                  })

                  // 2 FPS (normal)
                  box({
                    flexDirection: 'row',
                    alignItems: 'center',
                    marginBottom: 1,
                    children: () => {
                      text({ content: '2 FPS:     ', fg: colors.label, width: 12 })
                      input({
                        value: signal('Normal blink'),
                        width: 30,
                        bg: colors.inputBg,
                        fg: colors.bright,
                        cursor: { blink: { fps: 2 } },
                      })
                    },
                  })

                  // 4 FPS (fast)
                  box({
                    flexDirection: 'row',
                    alignItems: 'center',
                    marginBottom: 1,
                    children: () => {
                      text({ content: '4 FPS:     ', fg: colors.label, width: 12 })
                      input({
                        value: signal('Fast blink'),
                        width: 30,
                        bg: colors.inputBg,
                        fg: colors.bright,
                        cursor: { blink: { fps: 4 } },
                      })
                    },
                  })

                  // No blink
                  box({
                    flexDirection: 'row',
                    alignItems: 'center',
                    children: () => {
                      text({ content: 'No blink:  ', fg: colors.label, width: 12 })
                      input({
                        value: signal('Solid cursor'),
                        width: 30,
                        bg: colors.inputBg,
                        fg: colors.bright,
                        cursor: { blink: false },
                      })
                    },
                  })
                },
              })
            },
          })

          // Right column
          box({
            grow: 1,
            flexDirection: 'column',
            gap: 1,
            children: () => {
              // Custom Characters section
              box({
                border: 1,
                borderColor: colors.borderDim,
                bg: colors.sectionBg,
                padding: 1,
                flexDirection: 'column',
                children: () => {
                  text({ content: 'Custom Characters', fg: colors.heading, marginBottom: 1 })

                  // Thin bar
                  box({
                    flexDirection: 'row',
                    alignItems: 'center',
                    marginBottom: 1,
                    children: () => {
                      text({ content: 'Thin bar:  ', fg: colors.label, width: 12 })
                      input({
                        value: signal('Custom cursor'),
                        width: 30,
                        bg: colors.inputBg,
                        fg: colors.bright,
                        cursor: { char: '\u258F', blink: false },  // Left one-eighth block
                      })
                    },
                  })

                  // Thick bar
                  box({
                    flexDirection: 'row',
                    alignItems: 'center',
                    marginBottom: 1,
                    children: () => {
                      text({ content: 'Thick bar: ', fg: colors.label, width: 12 })
                      input({
                        value: signal('Half block'),
                        width: 30,
                        bg: colors.inputBg,
                        fg: colors.bright,
                        cursor: { char: '\u258C', blink: false },  // Left half block
                      })
                    },
                  })

                  // Arrow
                  box({
                    flexDirection: 'row',
                    alignItems: 'center',
                    marginBottom: 1,
                    children: () => {
                      text({ content: 'Arrow:     ', fg: colors.label, width: 12 })
                      input({
                        value: signal('Point right'),
                        width: 30,
                        bg: colors.inputBg,
                        fg: colors.bright,
                        cursor: { char: '\u25B6', blink: { fps: 2 } },  // Black right triangle
                      })
                    },
                  })

                  // Star
                  box({
                    flexDirection: 'row',
                    alignItems: 'center',
                    children: () => {
                      text({ content: 'Star:      ', fg: colors.label, width: 12 })
                      input({
                        value: signal('Fun cursor'),
                        width: 30,
                        bg: colors.inputBg,
                        fg: colors.bright,
                        cursor: { char: '\u2605', blink: { fps: 3 } },  // Black star
                      })
                    },
                  })
                },
              })

              // Colored Cursors section
              box({
                border: 1,
                borderColor: colors.borderDim,
                bg: colors.sectionBg,
                padding: 1,
                flexDirection: 'column',
                children: () => {
                  text({ content: 'Colored Cursors', fg: colors.heading, marginBottom: 1 })

                  // Green cursor (Matrix style)
                  box({
                    flexDirection: 'row',
                    alignItems: 'center',
                    marginBottom: 1,
                    children: () => {
                      text({ content: 'Green:     ', fg: colors.label, width: 12 })
                      input({
                        value: signal('Matrix style'),
                        width: 30,
                        bg: colors.inputBg,
                        fg: colors.bright,
                        cursor: {
                          fg: { r: 0, g: 255, b: 100, a: 255 },
                          bg: { r: 0, g: 100, b: 40, a: 255 },
                          blink: { fps: 2 },
                        },
                      })
                    },
                  })

                  // Red cursor (Error mode)
                  box({
                    flexDirection: 'row',
                    alignItems: 'center',
                    marginBottom: 1,
                    children: () => {
                      text({ content: 'Red:       ', fg: colors.label, width: 12 })
                      input({
                        value: signal('Error mode'),
                        width: 30,
                        bg: colors.inputBg,
                        fg: colors.bright,
                        cursor: {
                          fg: { r: 255, g: 80, b: 80, a: 255 },
                          bg: { r: 100, g: 30, b: 30, a: 255 },
                          blink: { fps: 4 },
                        },
                      })
                    },
                  })

                  // Rainbow cursor (Party mode!)
                  box({
                    flexDirection: 'row',
                    alignItems: 'center',
                    children: () => {
                      text({ content: 'Rainbow:   ', fg: colors.label, width: 12 })
                      input({
                        value: signal('Party mode!'),
                        width: 30,
                        bg: colors.inputBg,
                        fg: colors.bright,
                        cursor: {
                          fg: cycle(
                            [
                              { r: 255, g: 0, b: 0, a: 255 },
                              { r: 255, g: 165, b: 0, a: 255 },
                              { r: 255, g: 255, b: 0, a: 255 },
                              { r: 0, g: 255, b: 0, a: 255 },
                              { r: 0, g: 100, b: 255, a: 255 },
                              { r: 150, g: 0, b: 255, a: 255 },
                            ],
                            { fps: 8 }
                          ),
                          blink: false,  // No blink, just color change
                        },
                      })
                    },
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
        marginTop: 1,
        justifyContent: 'center',
        children: () => {
          text({ content: '[Tab] to navigate between inputs    [Ctrl+C] to exit', fg: colors.dimmed })
        },
      })
    },
  })
}, {
  mode: 'fullscreen',
})

console.log('[demo-cursors] App mounted')

// Keep process alive - Rust engine handles stdin (Ctrl+C exits)
await new Promise(() => {})
