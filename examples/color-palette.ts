/**
 * SparkTUI Color Palette Explorer
 *
 * Interactive color exploration showcasing:
 * - Full theme color grid
 * - OKLCH color picker simulation (L, C, H adjustment)
 * - Live color preview with hex/RGB/OKLCH display
 * - Color code display for copying
 *
 * Navigation:
 * - Tab: Switch between controls
 * - Arrow keys: Adjust values
 * - Number keys 1-8: Quick select color slots
 * - Ctrl+C: Exit
 */

import { signal, derived } from '@rlabs-inc/signals'
import { mount } from '../ts/engine/mount'
import { box, text, each } from '../ts/primitives'
import { theme, t, resolveColor } from '../ts/state/theme'
import { oklch, rgbToOklch } from '../ts/types/color'
import type { RGBA } from '../ts/types'
import { on } from '../ts/state/keyboard'
import type { KeyEvent } from '../ts/state/keyboard'

// =============================================================================
// STATE
// =============================================================================

// OKLCH picker state
const lightness = signal(0.7) // 0-1
const chroma = signal(0.15)   // 0-0.4
const hue = signal(270)       // 0-360

// Which control is focused
const focusedControl = signal<'L' | 'C' | 'H'>('H')

// Preview color derived from OKLCH
const previewColor = derived(() => oklch(lightness.value, chroma.value, hue.value))

// Color slots from theme
const colorSlots = [
  { key: 'primary', label: 'Primary' },
  { key: 'secondary', label: 'Secondary' },
  { key: 'tertiary', label: 'Tertiary' },
  { key: 'accent', label: 'Accent' },
  { key: 'success', label: 'Success' },
  { key: 'warning', label: 'Warning' },
  { key: 'error', label: 'Error' },
  { key: 'info', label: 'Info' },
  { key: 'text', label: 'Text' },
  { key: 'textMuted', label: 'Text Muted' },
  { key: 'textDim', label: 'Text Dim' },
  { key: 'textBright', label: 'Text Bright' },
  { key: 'background', label: 'Background' },
  { key: 'backgroundMuted', label: 'BG Muted' },
  { key: 'surface', label: 'Surface' },
  { key: 'overlay', label: 'Overlay' },
] as const

const selectedSlot = signal(0)

// =============================================================================
// HELPERS
// =============================================================================

function rgbaToHex(color: RGBA): string {
  if (color.r < 0) return 'terminal'
  const r = color.r.toString(16).padStart(2, '0')
  const g = color.g.toString(16).padStart(2, '0')
  const b = color.b.toString(16).padStart(2, '0')
  return `#${r}${g}${b}`.toUpperCase()
}

function rgbaToRgbString(color: RGBA): string {
  if (color.r < 0) return 'default'
  return `rgb(${color.r}, ${color.g}, ${color.b})`
}

function rgbaToOklchString(color: RGBA): string {
  if (color.r < 0) return 'n/a'
  const { l, c, h } = rgbToOklch(color)
  return `oklch(${l.toFixed(2)} ${c.toFixed(2)} ${Math.round(h)})`
}

function drawSlider(
  id: string,
  label: string,
  value: number,
  min: number,
  max: number,
  width: number,
  isFocused: boolean
) {
  const normalizedValue = (value - min) / (max - min)
  const filledWidth = Math.round(normalizedValue * (width - 4))

  box({
    id,
    width: width + 12,
    height: 1,
    flexDirection: 'row',
    alignItems: 'center',
    gap: 1,
    children: () => {
      text({
        content: `${label}:`,
        width: 3,
        fg: isFocused ? t.primary : t.textMuted,
      })

      // Slider track
      box({
        id: `${id}-track`,
        width,
        height: 1,
        flexDirection: 'row',
        border: 1,
        borderColor: isFocused ? t.primary : t.textMuted,
        children: () => {
          // Filled portion
          if (filledWidth > 0) {
            text({
              content: ''.padEnd(filledWidth, '#'),
              fg: t.primary,
            })
          }
          // Empty portion
          const emptyWidth = width - 4 - filledWidth
          if (emptyWidth > 0) {
            text({
              content: ''.padEnd(emptyWidth, '-'),
              fg: t.textDim,
            })
          }
        },
      })

      text({
        content: value.toFixed(2).padStart(5),
        width: 6,
        fg: t.text,
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

    // Tab - cycle focus
    if (keycode === 9) {
      const controls: Array<'L' | 'C' | 'H'> = ['L', 'C', 'H']
      const currentIdx = controls.indexOf(focusedControl.value)
      focusedControl.value = controls[(currentIdx + 1) % controls.length]!
      return true
    }

    // Arrow Up / Right - increase value
    if (keycode === 0x1b5b41 || keycode === 0x1b5b43) {
      switch (focusedControl.value) {
        case 'L':
          lightness.value = Math.min(1, lightness.value + 0.02)
          break
        case 'C':
          chroma.value = Math.min(0.4, chroma.value + 0.01)
          break
        case 'H':
          hue.value = (hue.value + 5) % 360
          break
      }
      return true
    }

    // Arrow Down / Left - decrease value
    if (keycode === 0x1b5b42 || keycode === 0x1b5b44) {
      switch (focusedControl.value) {
        case 'L':
          lightness.value = Math.max(0, lightness.value - 0.02)
          break
        case 'C':
          chroma.value = Math.max(0, chroma.value - 0.01)
          break
        case 'H':
          hue.value = (hue.value - 5 + 360) % 360
          break
      }
      return true
    }

    // Number keys 1-8 for color slot selection
    if (keycode >= 49 && keycode <= 56) {
      selectedSlot.value = keycode - 49
      // Load selected color into picker
      const slot = colorSlots[selectedSlot.value]
      if (slot) {
        const color = resolveColor((theme as any)[slot.key])
        if (color.r >= 0) {
          const oklchVal = rgbToOklch(color)
          lightness.value = oklchVal.l
          chroma.value = Math.min(0.4, oklchVal.c)
          hue.value = oklchVal.h
        }
      }
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
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        border: 1,
        borderColor: t.primary,
        bg: t.surface,
        children: () => {
          text({
            content: ' Color Palette Explorer ',
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
        padding: 1,
        gap: 2,
        children: () => {
          // =====================================================================
          // LEFT: Theme Color Grid
          // =====================================================================
          box({
            id: 'color-grid-panel',
            width: 50,
            flexDirection: 'column',
            border: 1,
            borderColor: t.textMuted,
            padding: 1,
            children: () => {
              text({
                content: 'Theme Colors (Press 1-8 to load)',
                fg: t.textBright,
                marginBottom: 1,
              })

              // Color grid - 4 columns
              box({
                id: 'color-grid',
                flexDirection: 'column',
                gap: 1,
                children: () => {
                  for (let row = 0; row < 4; row++) {
                    box({
                      id: `color-row-${row}`,
                      flexDirection: 'row',
                      gap: 1,
                      children: () => {
                        for (let col = 0; col < 4; col++) {
                          const idx = row * 4 + col
                          const slot = colorSlots[idx]
                          if (slot) {
                            const isSelected = () => selectedSlot.value === idx

                            box({
                              id: `color-swatch-${slot.key}`,
                              width: 11,
                              height: 3,
                              flexDirection: 'column',
                              alignItems: 'center',
                              justifyContent: 'center',
                              border: 1,
                              borderColor: () => isSelected() ? resolveColor(theme.primary) : resolveColor(theme.textMuted),
                              bg: () => resolveColor((theme as any)[slot.key]),
                              onClick: () => {
                                selectedSlot.value = idx
                                // Load into picker
                                const color = resolveColor((theme as any)[slot.key])
                                if (color.r >= 0) {
                                  const oklchVal = rgbToOklch(color)
                                  lightness.value = oklchVal.l
                                  chroma.value = Math.min(0.4, oklchVal.c)
                                  hue.value = oklchVal.h
                                }
                              },
                              children: () => {
                                text({
                                  content: slot.label.slice(0, 9),
                                  fg: t.textBright,
                                })
                              },
                            })
                          }
                        }
                      },
                    })
                  }
                },
              })

              // Selected color info
              box({
                id: 'selected-info',
                width: '100%',
                flexDirection: 'column',
                marginTop: 1,
                padding: 1,
                border: 1,
                borderColor: t.primary,
                bg: t.surface,
                children: () => {
                  text({
                    content: () => `Selected: ${colorSlots[selectedSlot.value]?.label || 'None'}`,
                    fg: t.textBright,
                  })
                  text({
                    content: () => {
                      const slot = colorSlots[selectedSlot.value]
                      if (!slot) return ''
                      const color = resolveColor((theme as any)[slot.key])
                      return `Hex: ${rgbaToHex(color)}`
                    },
                    fg: t.text,
                  })
                  text({
                    content: () => {
                      const slot = colorSlots[selectedSlot.value]
                      if (!slot) return ''
                      const color = resolveColor((theme as any)[slot.key])
                      return `RGB: ${rgbaToRgbString(color)}`
                    },
                    fg: t.text,
                  })
                  text({
                    content: () => {
                      const slot = colorSlots[selectedSlot.value]
                      if (!slot) return ''
                      const color = resolveColor((theme as any)[slot.key])
                      return `OKLCH: ${rgbaToOklchString(color)}`
                    },
                    fg: t.text,
                  })
                },
              })
            },
          })

          // =====================================================================
          // RIGHT: OKLCH Picker
          // =====================================================================
          box({
            id: 'oklch-picker',
            grow: 1,
            flexDirection: 'column',
            border: 1,
            borderColor: t.textMuted,
            padding: 1,
            children: () => {
              text({
                content: 'OKLCH Color Picker',
                fg: t.textBright,
                marginBottom: 1,
              })

              // Preview swatch
              box({
                id: 'preview-swatch',
                width: 30,
                height: 5,
                flexDirection: 'column',
                alignItems: 'center',
                justifyContent: 'center',
                border: 2,
                borderColor: t.primary,
                bg: previewColor,
                marginBottom: 1,
                children: () => {
                  text({
                    content: 'Preview',
                    fg: () => lightness.value > 0.5 ? { r: 0, g: 0, b: 0, a: 255 } : { r: 255, g: 255, b: 255, a: 255 },
                  })
                },
              })

              // Sliders
              box({
                id: 'sliders',
                flexDirection: 'column',
                gap: 1,
                marginBottom: 1,
                children: () => {
                  // Lightness slider
                  drawSlider(
                    'slider-l',
                    'L',
                    lightness.value,
                    0,
                    1,
                    20,
                    focusedControl.value === 'L'
                  )

                  // Chroma slider
                  drawSlider(
                    'slider-c',
                    'C',
                    chroma.value,
                    0,
                    0.4,
                    20,
                    focusedControl.value === 'C'
                  )

                  // Hue slider
                  drawSlider(
                    'slider-h',
                    'H',
                    hue.value,
                    0,
                    360,
                    20,
                    focusedControl.value === 'H'
                  )
                },
              })

              // Color values display
              box({
                id: 'color-values',
                flexDirection: 'column',
                padding: 1,
                border: 1,
                borderColor: t.textMuted,
                bg: t.surface,
                children: () => {
                  text({
                    content: 'Color Values:',
                    fg: t.textBright,
                    marginBottom: 1,
                  })
                  text({
                    content: () => `Hex:   ${rgbaToHex(previewColor.value)}`,
                    fg: t.text,
                  })
                  text({
                    content: () => `RGB:   ${rgbaToRgbString(previewColor.value)}`,
                    fg: t.text,
                  })
                  text({
                    content: () => `OKLCH: oklch(${lightness.value.toFixed(2)} ${chroma.value.toFixed(2)} ${Math.round(hue.value)})`,
                    fg: t.primary,
                  })
                },
              })

              // Hue rainbow preview
              box({
                id: 'hue-rainbow',
                width: 36,
                height: 2,
                flexDirection: 'row',
                marginTop: 1,
                children: () => {
                  for (let h = 0; h < 36; h++) {
                    const hueVal = h * 10
                    box({
                      id: `hue-${h}`,
                      width: 1,
                      height: 2,
                      bg: oklch(lightness.value, chroma.value, hueVal),
                    })
                  }
                },
              })
              text({
                content: 'Hue spectrum (0-360)',
                fg: t.textDim,
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
            content: 'Tab: Switch control | Arrows: Adjust | 1-8: Quick select | Ctrl+C: Exit',
            fg: t.textMuted,
          })
        },
      })
    },
  })
}, {
  mode: 'fullscreen',
})

console.log('[color-palette] App mounted')

// Keep process alive
await new Promise(() => {})
