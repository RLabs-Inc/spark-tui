/**
 * SparkTUI Gradient Demo
 *
 * Beautiful gradient effects showcasing OKLCH color space:
 * - Horizontal gradient using OKLCH hue rotation
 * - Vertical gradient (lightness variation)
 * - Animated rainbow using cycle()
 * - Perceptually uniform vs non-uniform comparison
 * - Multiple gradient styles
 *
 * OKLCH is perceptually uniform - colors appear evenly spaced
 * to human eyes, unlike RGB which can have muddy transitions.
 *
 * Controls:
 * - Space: Pause/resume animation
 * - +/-: Adjust animation speed
 * - Ctrl+C: Exit
 */

import { signal, derived } from '@rlabs-inc/signals'
import { mount } from '../ts/engine/mount'
import { box, text, cycle } from '../ts/primitives'
import { t, resolveColor, theme } from '../ts/state/theme'
import { oklch } from '../ts/types/color'
import { on } from '../ts/state/keyboard'
import type { KeyEvent } from '../ts/state/keyboard'

// =============================================================================
// STATE
// =============================================================================

const animationActive = signal(true)
const animationFps = signal(4)

// =============================================================================
// GRADIENT GENERATORS
// =============================================================================

// Generate OKLCH rainbow colors
function generateOklchRainbow(steps: number, lightness: number = 0.7, chroma: number = 0.15) {
  const colors = []
  for (let i = 0; i < steps; i++) {
    const hue = (i / steps) * 360
    colors.push(oklch(lightness, chroma, hue))
  }
  return colors
}

// Generate RGB rainbow colors (for comparison - shows non-uniformity)
function generateRgbRainbow(steps: number) {
  const colors = []
  for (let i = 0; i < steps; i++) {
    const t = i / steps
    // Simple HSL-like rainbow
    const h = t * 360
    const s = 1
    const l = 0.5

    // HSL to RGB conversion
    const c = (1 - Math.abs(2 * l - 1)) * s
    const x = c * (1 - Math.abs(((h / 60) % 2) - 1))
    const m = l - c / 2

    let r = 0, g = 0, b = 0
    if (h < 60) { r = c; g = x; b = 0 }
    else if (h < 120) { r = x; g = c; b = 0 }
    else if (h < 180) { r = 0; g = c; b = x }
    else if (h < 240) { r = 0; g = x; b = c }
    else if (h < 300) { r = x; g = 0; b = c }
    else { r = c; g = 0; b = x }

    colors.push({
      r: Math.round((r + m) * 255),
      g: Math.round((g + m) * 255),
      b: Math.round((b + m) * 255),
      a: 255,
    })
  }
  return colors
}

// Generate lightness gradient
function generateLightnessGradient(steps: number, hue: number = 270, chroma: number = 0.15) {
  const colors = []
  for (let i = 0; i < steps; i++) {
    const lightness = 0.2 + (i / steps) * 0.6 // 0.2 to 0.8
    colors.push(oklch(lightness, chroma, hue))
  }
  return colors
}

// Generate chroma gradient
function generateChromaGradient(steps: number, hue: number = 270, lightness: number = 0.7) {
  const colors = []
  for (let i = 0; i < steps; i++) {
    const chroma = (i / steps) * 0.3 // 0 to 0.3
    colors.push(oklch(lightness, chroma, hue))
  }
  return colors
}

// =============================================================================
// GRADIENT DISPLAY COMPONENTS
// =============================================================================

function horizontalGradient(
  id: string,
  colors: readonly any[],
  width: number,
  height: number = 2
) {
  box({
    id,
    width,
    height,
    flexDirection: 'row',
    children: () => {
      for (let i = 0; i < colors.length; i++) {
        box({
          id: `${id}-${i}`,
          width: 1,
          height,
          bg: colors[i],
        })
      }
    },
  })
}

function animatedGradient(
  id: string,
  width: number,
  height: number,
  lightness: number,
  chroma: number
) {
  // Create animated colors - each position cycles through hues with offset
  box({
    id,
    width,
    height,
    flexDirection: 'row',
    children: () => {
      for (let i = 0; i < width; i++) {
        // Each cell has its own color cycling signal, offset by position
        const hueOffset = (i / width) * 360
        const hueColors = []
        for (let h = 0; h < 36; h++) {
          const hue = (hueOffset + h * 10) % 360
          hueColors.push(oklch(lightness, chroma, hue))
        }

        box({
          id: `${id}-cell-${i}`,
          width: 1,
          height,
          bg: cycle(hueColors, { fps: animationFps.value, active: animationActive }),
        })
      }
    },
  })
}

// =============================================================================
// MOUNT APP
// =============================================================================

const cols = process.stdout.columns || 100
const rows = process.stdout.rows || 35
const gradientWidth = Math.min(60, cols - 20)

const { unmount } = mount(() => {
  // Keyboard controls
  on((event: KeyEvent) => {
    const keycode = event.keycode

    // Space - toggle animation
    if (keycode === 32) {
      animationActive.value = !animationActive.value
      return true
    }

    // + increase speed
    if (keycode === 43 || keycode === 61) {
      animationFps.value = Math.min(30, animationFps.value + 2)
      return true
    }

    // - decrease speed
    if (keycode === 45) {
      animationFps.value = Math.max(1, animationFps.value - 2)
      return true
    }

    return false
  })

  // Pre-generate static gradients
  const oklchRainbow = generateOklchRainbow(gradientWidth)
  const rgbRainbow = generateRgbRainbow(gradientWidth)
  const lightnessGradient = generateLightnessGradient(gradientWidth, 270) // Purple
  const chromaGradient = generateChromaGradient(gradientWidth, 30) // Orange
  const warmGradient = generateOklchRainbow(gradientWidth, 0.7, 0.2).slice(0, Math.floor(gradientWidth / 3))
  const coolGradient = generateOklchRainbow(gradientWidth, 0.6, 0.15).slice(Math.floor(gradientWidth / 2))

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
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        border: 1,
        borderColor: t.primary,
        bg: t.surface,
        children: () => {
          text({
            content: ' Gradient Demo - OKLCH Color Space ',
            fg: t.primary,
          })
        },
      })

      // =========================================================================
      // GRADIENT SECTIONS
      // =========================================================================
      box({
        id: 'main',
        width: '100%',
        grow: 1,
        flexDirection: 'column',
        padding: 1,
        gap: 1,
        overflow: 'scroll',
        children: () => {
          // -----------------------------------------------------------------
          // ANIMATED RAINBOW
          // -----------------------------------------------------------------
          box({
            id: 'animated-section',
            width: '100%',
            flexDirection: 'column',
            children: () => {
              box({
                id: 'animated-header',
                flexDirection: 'row',
                gap: 2,
                alignItems: 'center',
                children: () => {
                  text({
                    content: 'Animated Rainbow (OKLCH)',
                    fg: t.textBright,
                  })
                  text({
                    content: () => animationActive.value ? `[Running ${animationFps.value} fps]` : '[Paused]',
                    fg: () => animationActive.value ? resolveColor(theme.success) : resolveColor(theme.warning),
                  })
                },
              })
              animatedGradient('animated-rainbow', gradientWidth, 2, 0.7, 0.15)
            },
          })

          // -----------------------------------------------------------------
          // OKLCH vs RGB COMPARISON
          // -----------------------------------------------------------------
          box({
            id: 'comparison-section',
            width: '100%',
            flexDirection: 'column',
            marginTop: 1,
            children: () => {
              text({
                content: 'OKLCH vs RGB Rainbow Comparison',
                fg: t.textBright,
              })
              text({
                content: 'OKLCH is perceptually uniform - notice smoother transitions:',
                fg: t.textMuted,
              })

              box({
                id: 'oklch-row',
                flexDirection: 'row',
                alignItems: 'center',
                gap: 2,
                marginTop: 1,
                children: () => {
                  text({ content: 'OKLCH:', width: 8, fg: t.text })
                  horizontalGradient('oklch-rainbow', oklchRainbow, gradientWidth, 2)
                },
              })

              box({
                id: 'rgb-row',
                flexDirection: 'row',
                alignItems: 'center',
                gap: 2,
                children: () => {
                  text({ content: 'RGB:  ', width: 8, fg: t.text })
                  horizontalGradient('rgb-rainbow', rgbRainbow, gradientWidth, 2)
                },
              })

              text({
                content: '(RGB has uneven brightness - cyan/yellow appear brighter)',
                fg: t.textDim,
              })
            },
          })

          // -----------------------------------------------------------------
          // LIGHTNESS GRADIENT
          // -----------------------------------------------------------------
          box({
            id: 'lightness-section',
            width: '100%',
            flexDirection: 'column',
            marginTop: 1,
            children: () => {
              text({
                content: 'Lightness Gradient (constant hue & chroma)',
                fg: t.textBright,
              })
              box({
                id: 'lightness-row',
                flexDirection: 'row',
                alignItems: 'center',
                gap: 2,
                children: () => {
                  text({ content: 'L: 0.2', width: 8, fg: t.textDim })
                  horizontalGradient('lightness-gradient', lightnessGradient, gradientWidth, 2)
                  text({ content: '0.8', width: 4, fg: t.textBright })
                },
              })
            },
          })

          // -----------------------------------------------------------------
          // CHROMA GRADIENT
          // -----------------------------------------------------------------
          box({
            id: 'chroma-section',
            width: '100%',
            flexDirection: 'column',
            marginTop: 1,
            children: () => {
              text({
                content: 'Chroma Gradient (saturation - constant hue & lightness)',
                fg: t.textBright,
              })
              box({
                id: 'chroma-row',
                flexDirection: 'row',
                alignItems: 'center',
                gap: 2,
                children: () => {
                  text({ content: 'Gray', width: 8, fg: t.textDim })
                  horizontalGradient('chroma-gradient', chromaGradient, gradientWidth, 2)
                  text({ content: 'Vivid', width: 6, fg: t.primary })
                },
              })
            },
          })

          // -----------------------------------------------------------------
          // MULTI-ROW GRADIENT MATRIX
          // -----------------------------------------------------------------
          box({
            id: 'matrix-section',
            width: '100%',
            flexDirection: 'column',
            marginTop: 1,
            children: () => {
              text({
                content: 'Hue x Lightness Matrix',
                fg: t.textBright,
              })

              box({
                id: 'gradient-matrix',
                flexDirection: 'column',
                children: () => {
                  const lightnessLevels = [0.9, 0.75, 0.6, 0.45, 0.3]
                  for (const l of lightnessLevels) {
                    const row = generateOklchRainbow(gradientWidth, l, 0.15)
                    box({
                      id: `matrix-row-${l}`,
                      flexDirection: 'row',
                      alignItems: 'center',
                      gap: 1,
                      children: () => {
                        text({
                          content: `L=${l.toFixed(2)}`,
                          width: 7,
                          fg: t.textDim,
                        })
                        horizontalGradient(`matrix-gradient-${l}`, row, gradientWidth, 1)
                      },
                    })
                  }
                },
              })
            },
          })

          // -----------------------------------------------------------------
          // THEME-AWARE GRADIENTS
          // -----------------------------------------------------------------
          box({
            id: 'theme-gradients-section',
            width: '100%',
            flexDirection: 'column',
            marginTop: 1,
            children: () => {
              text({
                content: 'Theme-Aware Gradients',
                fg: t.textBright,
              })

              // Primary to secondary gradient
              box({
                id: 'primary-secondary',
                flexDirection: 'row',
                alignItems: 'center',
                gap: 2,
                marginTop: 1,
                children: () => {
                  text({ content: 'Primary', width: 10, fg: t.primary })

                  box({
                    id: 'ps-gradient',
                    width: 30,
                    height: 2,
                    flexDirection: 'row',
                    children: () => {
                      // Blend from primary to secondary
                      for (let i = 0; i < 30; i++) {
                        box({
                          id: `ps-${i}`,
                          width: 1,
                          height: 2,
                          bg: () => {
                            const primary = resolveColor(theme.primary)
                            const secondary = resolveColor(theme.secondary)
                            const t = i / 29
                            return {
                              r: Math.round(primary.r + (secondary.r - primary.r) * t),
                              g: Math.round(primary.g + (secondary.g - primary.g) * t),
                              b: Math.round(primary.b + (secondary.b - primary.b) * t),
                              a: 255,
                            }
                          },
                        })
                      }
                    },
                  })

                  text({ content: 'Secondary', width: 12, fg: t.secondary })
                },
              })

              // Success to error gradient
              box({
                id: 'success-error',
                flexDirection: 'row',
                alignItems: 'center',
                gap: 2,
                children: () => {
                  text({ content: 'Success', width: 10, fg: t.success })

                  box({
                    id: 'se-gradient',
                    width: 30,
                    height: 2,
                    flexDirection: 'row',
                    children: () => {
                      for (let i = 0; i < 30; i++) {
                        box({
                          id: `se-${i}`,
                          width: 1,
                          height: 2,
                          bg: () => {
                            const success = resolveColor(theme.success)
                            const error = resolveColor(theme.error)
                            const t = i / 29
                            return {
                              r: Math.round(success.r + (error.r - success.r) * t),
                              g: Math.round(success.g + (error.g - success.g) * t),
                              b: Math.round(success.b + (error.b - success.b) * t),
                              a: 255,
                            }
                          },
                        })
                      }
                    },
                  })

                  text({ content: 'Error', width: 8, fg: t.error })
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
            content: 'Space: Pause/Resume | +/-: Speed | Ctrl+C: Exit',
            fg: t.textMuted,
          })
        },
      })
    },
  })
}, {
  mode: 'fullscreen',
})

console.log('[gradient-demo] App mounted')

// Keep process alive
await new Promise(() => {})
