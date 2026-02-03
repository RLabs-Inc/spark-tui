/**
 * SparkTUI Animation Stress Test
 *
 * Performance showcase demonstrating 50+ simultaneous animations
 * running smoothly. Shows spinners, color cycling, and moving elements
 * all powered by SparkTUI's reactive signal system.
 *
 * Key concepts demonstrated:
 * - 50+ spinners at various speeds
 * - Color cycling animations
 * - Position-based animations
 * - All animations sharing clock timers efficiently
 * - Performance stats showing smooth operation
 *
 * Controls:
 * - +/- or Up/Down: Adjust animation count
 * - Space: Pause/Resume all animations
 * - 1-5: Adjust global speed
 * - Ctrl+C: Exit
 */

import { signal, derived } from '@rlabs-inc/signals'
import { mount } from '../ts/engine/mount'
import { box, text, each, cycle, Frames } from '../ts/primitives'
import { packColor } from '../ts/bridge/shared-buffer'
import { on, isPress, lastKey } from '../ts/state/keyboard'
import type { KeyEvent } from '../ts/state/keyboard'

// =============================================================================
// COLORS
// =============================================================================

const colors = {
  bg: packColor(12, 12, 18, 255),
  bgCard: packColor(22, 22, 32, 255),
  bgPanel: packColor(28, 28, 40, 255),
  border: packColor(60, 60, 90, 255),
  borderActive: packColor(100, 140, 255, 255),
  text: packColor(220, 220, 235, 255),
  textMuted: packColor(100, 100, 130, 255),
  textAccent: packColor(140, 170, 255, 255),
  textSuccess: packColor(100, 220, 140, 255),
  textWarning: packColor(255, 200, 100, 255),
  textError: packColor(255, 120, 120, 255),
}

// Rainbow colors for animations
const rainbow = [
  packColor(255, 100, 100, 255),
  packColor(255, 160, 100, 255),
  packColor(255, 220, 100, 255),
  packColor(180, 255, 100, 255),
  packColor(100, 255, 160, 255),
  packColor(100, 220, 255, 255),
  packColor(100, 140, 255, 255),
  packColor(160, 100, 255, 255),
  packColor(255, 100, 220, 255),
]

// Spinner text colors
const spinnerColors = [
  packColor(255, 120, 120, 255),
  packColor(255, 200, 120, 255),
  packColor(200, 255, 120, 255),
  packColor(120, 255, 200, 255),
  packColor(120, 200, 255, 255),
  packColor(200, 120, 255, 255),
]

// =============================================================================
// STATE
// =============================================================================

const animationCount = signal(50)
const globalPaused = signal(false)
const speedMultiplier = signal(1)

// FPS tracking
const fps = signal(0)
let fpsFrames = 0
let lastFpsTime = performance.now()

// Animation count
const totalAnimations = derived(() => {
  // Spinners + color boxes + moving elements
  return animationCount.value + 12 + 8
})

// Generate spinner items
const spinnerItems = derived(() => {
  const count = animationCount.value
  const result: Array<{ id: string; index: number; type: number }> = []
  for (let i = 0; i < count; i++) {
    result.push({
      id: `spinner-${i}`,
      index: i,
      type: i % 7, // Different spinner types
    })
  }
  return result
})

// Get spinner frames by type
function getSpinnerFrames(type: number): readonly string[] {
  switch (type) {
    case 0: return Frames.spinner
    case 1: return Frames.dots
    case 2: return Frames.line
    case 3: return Frames.bar
    case 4: return Frames.bounce
    case 5: return Frames.arrow
    case 6: return Frames.pulse
    default: return Frames.spinner
  }
}

// Update FPS
function updateFps() {
  fpsFrames++
  const now = performance.now()
  if (now - lastFpsTime >= 1000) {
    fps.value = fpsFrames
    fpsFrames = 0
    lastFpsTime = now
  }
}

// FPS updater
setInterval(() => {
  if (!globalPaused.value) {
    updateFps()
  }
}, 16)

// =============================================================================
// MOUNT APP
// =============================================================================

const cols = process.stdout.columns || 120
const rows = process.stdout.rows || 40

mount(() => {
  // ─────────────────────────────────────────────────────────────────────────────
  // KEYBOARD HANDLER
  // ─────────────────────────────────────────────────────────────────────────────
  on((event: KeyEvent) => {
    if (!isPress(event)) return false

    const char = lastKey.value

    // Adjust count
    if (char === '+' || char === '=' || event.keycode === 0x1b5b41) { // Up arrow
      animationCount.value = Math.min(100, animationCount.value + 10)
      return true
    }
    if (char === '-' || event.keycode === 0x1b5b42) { // Down arrow
      animationCount.value = Math.max(10, animationCount.value - 10)
      return true
    }

    // Pause/Resume
    if (char === ' ') {
      globalPaused.value = !globalPaused.value
      return true
    }

    // Speed control
    if (char === '1') {
      speedMultiplier.value = 0.5
      return true
    }
    if (char === '2') {
      speedMultiplier.value = 1
      return true
    }
    if (char === '3') {
      speedMultiplier.value = 2
      return true
    }
    if (char === '4') {
      speedMultiplier.value = 4
      return true
    }
    if (char === '5') {
      speedMultiplier.value = 8
      return true
    }

    return false
  })

  // ─────────────────────────────────────────────────────────────────────────────
  // ROOT CONTAINER
  // ─────────────────────────────────────────────────────────────────────────────
  box({
    id: 'root',
    width: '100%',
    height: '100%',
    flexDirection: 'column',
    bg: colors.bg,
    children: () => {
      // ─────────────────────────────────────────────────────────────────────────
      // HEADER
      // ─────────────────────────────────────────────────────────────────────────
      box({
        id: 'header',
        width: '100%',
        // height: 5,
        flexDirection: 'column',
        bg: colors.bgCard,
        border: 1,
        borderColor: colors.border,
        padding: 1,
        children: () => {
          box({
            flexDirection: 'row',
            width: '100%',
            justifyContent: 'space-between',
            children: () => {
              box({
                flexDirection: 'row',
                gap: 2,
                children: () => {
                  text({
                    content: cycle(Frames.dots, { fps: 12, active: () => !globalPaused.value }),
                    fg: colors.textSuccess,
                  })
                  text({ content: 'SparkTUI Animation Stress Test', fg: colors.textAccent })
                  text({
                    content: () => globalPaused.value ? ' [PAUSED]' : '',
                    fg: colors.textWarning,
                  })
                },
              })
              text({
                content: () => `FPS: ${fps.value}`,
                fg: () => fps.value >= 30 ? colors.textSuccess : fps.value >= 15 ? colors.textWarning : colors.textError,
              })
            },
          })

          box({
            flexDirection: 'row',
            width: '100%',
            // flexWrap: 'wrap',
            gap: 4,
            marginTop: 1,
            children: () => {
              text({
                content: () => `Spinners: ${animationCount.value}`,
                fg: colors.text,
              })
              text({
                content: () => `Total Animations: ${totalAnimations.value}`,
                fg: colors.textAccent,
              })
              text({
                content: () => `Speed: ${speedMultiplier.value}x`,
                fg: colors.text,
              })
              text({
                content: 'All sharing clock timers efficiently',
                fg: colors.textMuted,
              })
            },
          })
        },
      })

      // ─────────────────────────────────────────────────────────────────────────
      // MAIN CONTENT
      // ─────────────────────────────────────────────────────────────────────────
      box({
        width: '100%',  // TEST: bound main content to root
        grow: 1,
        flexDirection: 'row',
        // flexWrap: 'wrap',  // removed - we want row layout for left/right panels
        padding: 1,
        gap: 1,
        children: () => {
          // ─────────────────────────────────────────────────────────────────────
          // LEFT PANEL: Spinner Grid
          // ─────────────────────────────────────────────────────────────────────
          box({
            // grow: 1,  // TEST: grow to fill space left by right panel
            flexDirection: 'column',
            width: '100%',
            border: 1,
            borderColor: colors.border,
            bg: colors.bgPanel,
            children: () => {
              // Panel header
              box({
                width: '100%',  // TEST: fill parent width
                height: 1,
                bg: colors.bgCard,
                paddingLeft: 1,
                children: () => {
                  text({
                    content: () => `Spinner Grid (${animationCount.value} spinners)`,
                    fg: colors.textAccent,
                  })
                },
              })

              // Spinner grid container
              box({
                width: '100%',  // TEST: bound to left panel width
                // grow: 1,
                padding: 1,
                children: () => {
                  // Spinner wrapper - flexWrap makes spinners wrap within bounded width
                  box({
                    width: '100%',
                    flexDirection: 'row',
                    // flexWrap: 'wrap',  // restored - wrapping works with width: 100%
                    overflow: 'scroll',
                    gap: 1,
                    children: () => {
                      each(
                        () => spinnerItems.value,
                        (getItem, key) => {
                          const item = getItem()
                          const colorIdx = item.index % spinnerColors.length
                          const frames = getSpinnerFrames(item.type)
                          const baseFps = 8 + (item.index % 8) // 8-15 fps

                          return box({
                            id: `spinner-box-${key}`,
                            width: 3,
                            height: 1,
                            flexWrap: 'wrap',
                            justifyContent: 'center',
                            alignItems: 'center',
                            children: () => {
                              text({
                                content: cycle(frames, {
                                  fps: baseFps * speedMultiplier.value,
                                  active: () => !globalPaused.value,
                                }),
                                fg: spinnerColors[colorIdx],
                              })
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

          // ─────────────────────────────────────────────────────────────────────
          // RIGHT PANEL: Color & Movement
          // ─────────────────────────────────────────────────────────────────────
          box({
            width: 30,
            flexDirection: 'column',
            gap: 1,
            children: () => {
              // Color Cycling Section
              box({
                flexDirection: 'column',
                border: 1,
                borderColor: colors.border,
                bg: colors.bgPanel,
                children: () => {
                  box({
                    height: 1,
                    bg: colors.bgCard,
                    paddingLeft: 1,
                    children: () => {
                      text({ content: 'Color Cycling (12)', fg: colors.textAccent })
                    },
                  })

                  box({
                    flexDirection: 'row',
                    flexWrap: 'wrap',
                    padding: 1,
                    gap: 1,
                    children: () => {
                      for (let i = 0; i < 12; i++) {
                        // Offset colors for wave effect
                        const offsetRainbow = [...rainbow.slice(i % rainbow.length), ...rainbow.slice(0, i % rainbow.length)]
                        box({
                          width: 4,
                          height: 2,
                          bg: cycle(offsetRainbow, {
                            fps: 2 * speedMultiplier.value,
                            active: () => !globalPaused.value,
                          }),
                          justifyContent: 'center',
                          alignItems: 'center',
                          children: () => {
                            text({ content: '    ', fg: colors.text })
                          },
                        })
                      }
                    },
                  })
                },
              })

              // Border Cycling Section
              box({
                flexDirection: 'column',
                border: 1,
                borderColor: cycle(rainbow, {
                  fps: 3 * speedMultiplier.value,
                  active: () => !globalPaused.value,
                }),
                bg: colors.bgPanel,
                children: () => {
                  box({
                    height: 1,
                    bg: colors.bgCard,
                    paddingLeft: 1,
                    children: () => {
                      text({ content: 'Border Cycling', fg: colors.textAccent })
                    },
                  })

                  box({
                    padding: 1,
                    justifyContent: 'center',
                    alignItems: 'center',
                    children: () => {
                      text({
                        content: 'Rainbow Border!',
                        fg: cycle(rainbow, {
                          fps: 4 * speedMultiplier.value,
                          active: () => !globalPaused.value,
                        }),
                      })
                    },
                  })
                },
              })

              // Multi-element Animation
              box({
                flexDirection: 'column',
                border: 1,
                borderColor: colors.border,
                bg: colors.bgPanel,
                children: () => {
                  box({
                    height: 1,
                    bg: colors.bgCard,
                    paddingLeft: 1,
                    children: () => {
                      text({ content: 'Bouncing Elements (8)', fg: colors.textAccent })
                    },
                  })

                  box({
                    flexDirection: 'row',
                    padding: 1,
                    gap: 1,
                    justifyContent: 'center',
                    children: () => {
                      const bounceChars = ['_', '.', '-', "'", '"', "'", '-', '.']
                      for (let i = 0; i < 8; i++) {
                        const offset = (i * 2) % bounceChars.length
                        const chars = [...bounceChars.slice(offset), ...bounceChars.slice(0, offset)]
                        text({
                          content: cycle(chars, {
                            fps: 6 * speedMultiplier.value,
                            active: () => !globalPaused.value,
                          }),
                          fg: spinnerColors[i % spinnerColors.length],
                        })
                      }
                    },
                  })
                },
              })

              // Progress Bars
              box({
                flexDirection: 'column',
                border: 1,
                borderColor: colors.border,
                bg: colors.bgPanel,
                children: () => {
                  box({
                    height: 1,
                    bg: colors.bgCard,
                    paddingLeft: 1,
                    children: () => {
                      text({ content: 'Animated Progress', fg: colors.textAccent })
                    },
                  })

                  box({
                    flexDirection: 'column',
                    padding: 1,
                    gap: 0,
                    children: () => {
                      const progressStates = [
                        '[          ]',
                        '[=         ]',
                        '[==        ]',
                        '[===       ]',
                        '[====      ]',
                        '[=====     ]',
                        '[======    ]',
                        '[=======   ]',
                        '[========  ]',
                        '[========= ]',
                        '[==========]',
                      ]
                      for (let i = 0; i < 3; i++) {
                        const offset = (i * 3) % progressStates.length
                        const states = [...progressStates.slice(offset), ...progressStates.slice(0, offset)]
                        text({
                          content: cycle(states, {
                            fps: 3 * speedMultiplier.value,
                            active: () => !globalPaused.value,
                          }),
                          fg: spinnerColors[i * 2],
                        })
                      }
                    },
                  })
                },
              })

              // Stats
              box({
                grow: 1,
                flexDirection: 'column',
                border: 1,
                borderColor: colors.border,
                bg: colors.bgPanel,
                padding: 1,
                children: () => {
                  text({ content: 'How it works:', fg: colors.textAccent })
                  text({ content: '- cycle() is a signal source', fg: colors.textMuted })
                  text({ content: '- Same FPS share clock timer', fg: colors.textMuted })
                  text({ content: '- NO render loops', fg: colors.textMuted })
                  text({ content: '- Purely reactive updates', fg: colors.textMuted })
                },
              })
            },
          })
        },
      })

      // ─────────────────────────────────────────────────────────────────────────
      // CONTROLS
      // ─────────────────────────────────────────────────────────────────────────
      box({
        id: 'controls',
        width: '100%',
        height: 1,
        bg: colors.bgCard,
        flexDirection: 'row',
        justifyContent: 'center',
        gap: 3,
        children: () => {
          text({ content: '[+/-] Adjust count', fg: colors.textMuted })
          text({ content: '[Space] Pause', fg: colors.textMuted })
          text({ content: '[1-5] Speed', fg: colors.textMuted })
          text({ content: '[Ctrl+C] Exit', fg: colors.textMuted })
        },
      })
    },
  })
}, { mode: 'fullscreen' })

console.log('[animation-stress] App mounted - +/- to adjust, Space to pause')
await new Promise(() => { })
