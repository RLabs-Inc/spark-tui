/**
 * SparkTUI Animation Showcase
 *
 * A beautiful demo showcasing all animation primitives:
 * - Built-in spinner frames (spinner, dots, line, bar, clock, bounce, arrow, pulse)
 * - FPS comparison (same animation at different speeds)
 * - Color cycling (rainbow backgrounds)
 * - Pulse effects (blinking indicators)
 * - Combined animations (spinner + color together)
 *
 * This demonstrates that animations are SIGNAL SOURCES, not render loops.
 * setInterval updates signal values, which propagate reactively through
 * the system. There is NO fixed FPS rendering - Rust renders when data changes.
 */

import { mount } from '../ts/engine/mount'
import { box, text, cycle, Frames } from '../ts/primitives'
import { packColor } from '../ts/bridge/shared-buffer'

// =============================================================================
// COLORS
// =============================================================================

const colors = {
  // Background colors
  bgDark: packColor(18, 18, 24, 255),
  bgCard: packColor(28, 28, 38, 255),
  bgSection: packColor(35, 35, 50, 255),

  // Border colors
  borderPrimary: packColor(100, 140, 200, 255),
  borderDim: packColor(60, 60, 80, 255),
  borderHighlight: packColor(140, 180, 255, 255),

  // Text colors
  textPrimary: packColor(240, 240, 250, 255),
  textSecondary: packColor(160, 160, 180, 255),
  textDim: packColor(100, 100, 120, 255),
  textAccent: packColor(120, 200, 255, 255),
  textSuccess: packColor(100, 220, 140, 255),
  textWarning: packColor(255, 200, 100, 255),
  textError: packColor(255, 120, 120, 255),

  // Rainbow for color cycling
  rainbow: [
    packColor(255, 100, 100, 255), // red
    packColor(255, 180, 100, 255), // orange
    packColor(255, 255, 100, 255), // yellow
    packColor(100, 255, 140, 255), // green
    packColor(100, 200, 255, 255), // blue
    packColor(180, 140, 255, 255), // purple
  ],

  // Indicator colors
  online: packColor(100, 220, 140, 255),
  offline: packColor(100, 100, 120, 255),
  recording: packColor(255, 100, 100, 255),
  loading: packColor(255, 200, 100, 255),
}

// =============================================================================
// MOUNT APP
// =============================================================================

const cols = process.stdout.columns || 80
const rows = process.stdout.rows || 24

const { unmount, setMode, getMode } = mount(() => {
  // ─────────────────────────────────────────────────────────────────────────
  // ROOT CONTAINER
  // ─────────────────────────────────────────────────────────────────────────
  box({
    width: cols,
    height: rows,
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'flex-start',
    bg: colors.bgDark,
    padding: 1,
    children: () => {
      // ─────────────────────────────────────────────────────────────────────
      // TITLE CARD
      // ─────────────────────────────────────────────────────────────────────
      box({
        width: 60,
        flexDirection: 'column',
        alignItems: 'center',
        border: 1,
        borderColor: colors.borderHighlight,
        bg: colors.bgCard,
        padding: 1,
        children: () => {
          text({
            content: '  SparkTUI Animation Showcase  ',
            fg: colors.textAccent,
          })
          text({
            content: 'Reactive animations powered by signals',
            fg: colors.textDim,
          })
        },
      })

      // ─────────────────────────────────────────────────────────────────────
      // SECTION 1: BUILT-IN SPINNERS
      // ─────────────────────────────────────────────────────────────────────
      box({
        width: 60,
        flexDirection: 'column',
        border: 1,
        borderColor: colors.borderDim,
        bg: colors.bgSection,
        padding: 1,
        marginTop: 1,
        children: () => {
          text({ content: 'Built-in Spinners', fg: colors.textSecondary })

          box({
            flexDirection: 'row',
            justifyContent: 'space-around',
            width: '100%',
            marginTop: 1,
            children: () => {
              // Each spinner type with label
              const spinnerTypes: Array<[string, readonly string[]]> = [
                ['spinner', Frames.spinner],
                ['dots', Frames.dots],
                ['line', Frames.line],
                ['bar', Frames.bar],
                ['bounce', Frames.bounce],
                ['arrow', Frames.arrow],
                ['pulse', Frames.pulse],
              ]

              for (const [name, frames] of spinnerTypes) {
                box({
                  flexDirection: 'column',
                  alignItems: 'center',
                  width: 7,
                  children: () => {
                    text({
                      content: cycle(frames, { fps: 10 }),
                      fg: colors.textAccent,
                    })
                    text({
                      content: name,
                      fg: colors.textDim,
                    })
                  },
                })
              }
            },
          })
        },
      })

      // ─────────────────────────────────────────────────────────────────────
      // SECTION 2: FPS COMPARISON
      // ─────────────────────────────────────────────────────────────────────
      box({
        width: 60,
        flexDirection: 'column',
        border: 1,
        borderColor: colors.borderDim,
        bg: colors.bgSection,
        padding: 1,
        marginTop: 1,
        children: () => {
          text({ content: 'FPS Comparison (same spinner)', fg: colors.textSecondary })

          box({
            flexDirection: 'row',
            justifyContent: 'space-around',
            width: '100%',
            marginTop: 1,
            children: () => {
              const fpsValues = [2, 4, 8, 12, 24]

              for (const fps of fpsValues) {
                box({
                  flexDirection: 'column',
                  alignItems: 'center',
                  width: 10,
                  children: () => {
                    text({
                      content: cycle(Frames.spinner, { fps }),
                      fg: colors.textPrimary,
                    })
                    text({
                      content: `${fps} fps`,
                      fg: colors.textDim,
                    })
                  },
                })
              }
            },
          })
        },
      })

      // ─────────────────────────────────────────────────────────────────────
      // SECTION 3: COLOR CYCLING
      // ─────────────────────────────────────────────────────────────────────
      box({
        width: 60,
        flexDirection: 'column',
        border: 1,
        borderColor: colors.borderDim,
        bg: colors.bgSection,
        padding: 1,
        marginTop: 1,
        children: () => {
          text({ content: 'Color Cycling (2 fps)', fg: colors.textSecondary })

          box({
            flexDirection: 'row',
            justifyContent: 'center',
            gap: 1,
            width: '100%',
            marginTop: 1,
            children: () => {
              // Each box cycles through rainbow colors, offset by index
              for (let i = 0; i < 6; i++) {
                // Offset each color box to create wave effect
                const offsetColors = [
                  ...colors.rainbow.slice(i),
                  ...colors.rainbow.slice(0, i),
                ]
                box({
                  width: 6,
                  height: 2,
                  bg: cycle(offsetColors, { fps: 2 }),
                  alignItems: 'center',
                  justifyContent: 'center',
                  children: () => {
                    text({ content: '    ', fg: colors.textPrimary })
                  },
                })
              }
            },
          })
        },
      })

      // ─────────────────────────────────────────────────────────────────────
      // SECTION 4: PULSE INDICATORS
      // ─────────────────────────────────────────────────────────────────────
      box({
        width: 60,
        flexDirection: 'column',
        border: 1,
        borderColor: colors.borderDim,
        bg: colors.bgSection,
        padding: 1,
        marginTop: 1,
        children: () => {
          text({ content: 'Pulse Indicators', fg: colors.textSecondary })

          box({
            flexDirection: 'row',
            justifyContent: 'space-around',
            width: '100%',
            marginTop: 1,
            children: () => {
              // Online indicator (slow pulse)
              box({
                flexDirection: 'row',
                alignItems: 'center',
                gap: 1,
                children: () => {
                  text({
                    content: cycle(['●', '○'], { fps: 1 }),
                    fg: colors.online,
                  })
                  text({ content: 'Online', fg: colors.textPrimary })
                },
              })

              // Offline indicator (no animation)
              box({
                flexDirection: 'row',
                alignItems: 'center',
                gap: 1,
                children: () => {
                  text({ content: '○', fg: colors.offline })
                  text({ content: 'Offline', fg: colors.textDim })
                },
              })

              // Recording indicator (fast blink)
              box({
                flexDirection: 'row',
                alignItems: 'center',
                gap: 1,
                children: () => {
                  text({
                    content: cycle(['●', ' '], { fps: 2 }),
                    fg: colors.recording,
                  })
                  text({ content: 'Recording', fg: colors.textPrimary })
                },
              })

              // Loading indicator (spinner + label)
              box({
                flexDirection: 'row',
                alignItems: 'center',
                gap: 1,
                children: () => {
                  text({
                    content: cycle(Frames.dots, { fps: 10 }),
                    fg: colors.loading,
                  })
                  text({ content: 'Loading', fg: colors.textPrimary })
                },
              })
            },
          })
        },
      })

      // ─────────────────────────────────────────────────────────────────────
      // SECTION 5: COMBINED ANIMATIONS
      // ─────────────────────────────────────────────────────────────────────
      box({
        width: 60,
        flexDirection: 'column',
        border: 1,
        borderColor: colors.borderDim,
        bg: colors.bgSection,
        padding: 1,
        marginTop: 1,
        children: () => {
          text({ content: 'Combined Animations', fg: colors.textSecondary })

          // Create combined animation signals before using them
          const processingSpinner = cycle(Frames.spinner, { fps: 12 })
          const processingBg = cycle(colors.rainbow, { fps: 2 })
          const clockFace = cycle(Frames.clock, { fps: 1 })
          const clockBorder = cycle(colors.rainbow, { fps: 3 })

          box({
            flexDirection: 'row',
            justifyContent: 'space-around',
            width: '100%',
            marginTop: 1,
            children: () => {
              // Spinner + color cycling background
              box({
                width: 20,
                height: 3,
                bg: processingBg,
                alignItems: 'center',
                justifyContent: 'center',
                border: 1,
                borderColor: colors.borderPrimary,
                children: () => {
                  text({
                    content: () => ` ${processingSpinner.value} Processing `,
                    fg: colors.bgDark,
                  })
                },
              })

              // Clock with cycling border
              box({
                width: 20,
                height: 3,
                bg: colors.bgCard,
                alignItems: 'center',
                justifyContent: 'center',
                border: 1,
                borderColor: clockBorder,
                children: () => {
                  text({
                    content: clockFace,
                    fg: colors.textPrimary,
                  })
                  text({
                    content: ' Time flies',
                    fg: colors.textSecondary,
                  })
                },
              })
            },
          })
        },
      })

      // ─────────────────────────────────────────────────────────────────────
      // FOOTER
      // ─────────────────────────────────────────────────────────────────────
      box({
        width: 60,
        flexDirection: 'row',
        justifyContent: 'center',
        marginTop: 1,
        children: () => {
          text({
            content: 'Press Ctrl+C to exit',
            fg: colors.textDim,
          })
        },
      })
    },
  })
}, {
  mode: 'fullscreen',
})

console.log('[demo-spinners] App mounted')

// Keep process alive - Rust handles stdin (Ctrl+C to exit)
await new Promise(() => { })
