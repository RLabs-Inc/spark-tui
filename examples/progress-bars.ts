/**
 * SparkTUI - Progress Bar Showcase
 *
 * A beautiful showcase of progress bar styles:
 * - Block style (solid unicode blocks)
 * - Dots style (braille patterns)
 * - ASCII style (classic hash/dash)
 * - Gradient style (color-changing based on progress)
 * - Determinate and indeterminate (animated)
 * - Multiple concurrent progress bars
 * - Percentage and ETA display
 *
 * Run: bun run examples/progress-bars.ts
 */

import { signal, derived } from '@rlabs-inc/signals'
import { mount } from '../ts/engine/mount'
import { box } from '../ts/primitives/box'
import { text } from '../ts/primitives/text'
import { onCleanup } from '../ts/primitives/scope'
import { cycle, pulse } from '../ts/primitives/animation'
import type { RGBA } from '../ts/types'

// =============================================================================
// COLORS
// =============================================================================

function rgba(r: number, g: number, b: number, a: number = 255): RGBA {
  return { r, g, b, a }
}

const colors = {
  bgDark: rgba(16, 16, 22),
  bgCard: rgba(24, 24, 32),
  bgPanel: rgba(32, 32, 42),
  bgBar: rgba(40, 40, 55),

  textPrimary: rgba(235, 235, 245),
  textSecondary: rgba(160, 160, 180),
  textMuted: rgba(100, 100, 120),

  cyan: rgba(80, 200, 255),
  green: rgba(80, 220, 140),
  yellow: rgba(255, 200, 80),
  red: rgba(255, 100, 100),
  orange: rgba(255, 160, 80),
  purple: rgba(180, 130, 255),
  blue: rgba(100, 150, 255),
  pink: rgba(255, 130, 180),

  borderDim: rgba(50, 50, 65),
  borderAccent: rgba(80, 140, 200),
}

// Gradient colors for progress
const gradientColors: RGBA[] = [
  rgba(255, 80, 80),   // 0%  - Red
  rgba(255, 120, 80),  // 20% - Orange-Red
  rgba(255, 180, 80),  // 40% - Orange
  rgba(255, 220, 80),  // 60% - Yellow
  rgba(180, 220, 80),  // 80% - Yellow-Green
  rgba(80, 220, 120),  // 100% - Green
]

// =============================================================================
// PROGRESS BAR RENDERERS
// =============================================================================

/** Block-style progress bar: █████░░░░░ */
function blockBar(pct: number, width: number = 20): string {
  const clamped = Math.max(0, Math.min(100, pct))
  const filled = Math.floor((clamped / 100) * width)
  const partial = ((clamped / 100) * width) - filled

  const blocks = ['▏', '▎', '▍', '▌', '▋', '▊', '▉', '█']
  let bar = '█'.repeat(filled)

  if (partial > 0 && filled < width) {
    const partialIndex = Math.floor(partial * 8)
    bar += blocks[partialIndex] || ''
    bar += '░'.repeat(Math.max(0, width - filled - 1))
  } else {
    bar += '░'.repeat(Math.max(0, width - filled))
  }

  return bar.slice(0, width)
}

/** Dots-style progress bar using braille: ⣿⣿⣿⣿⡀⠀⠀⠀ */
function dotsBar(pct: number, width: number = 20): string {
  const clamped = Math.max(0, Math.min(100, pct))
  const filled = Math.floor((clamped / 100) * width)
  const partial = ((clamped / 100) * width) - filled

  const brailleFull = '⣿'
  const brailleEmpty = '⠀'
  const braillePartial = ['⡀', '⡄', '⡆', '⡇', '⣇', '⣧', '⣷']

  let bar = brailleFull.repeat(filled)

  if (partial > 0 && filled < width) {
    const partialIndex = Math.floor(partial * 7)
    bar += braillePartial[partialIndex] || brailleEmpty
    bar += brailleEmpty.repeat(Math.max(0, width - filled - 1))
  } else {
    bar += brailleEmpty.repeat(Math.max(0, width - filled))
  }

  return bar.slice(0, width)
}

/** ASCII-style progress bar: [####----] */
function asciiBar(pct: number, width: number = 20): string {
  const clamped = Math.max(0, Math.min(100, pct))
  const filled = Math.floor((clamped / 100) * width)
  return '#'.repeat(filled) + '-'.repeat(width - filled)
}

/** Thin bar with rounded ends: ●━━━━━━━━○○○○ */
function thinBar(pct: number, width: number = 20): string {
  const clamped = Math.max(0, Math.min(100, pct))
  const filled = Math.floor((clamped / 100) * width)
  return '●' + '━'.repeat(Math.max(0, filled - 1)) + '○'.repeat(Math.max(0, width - filled))
}

/** Smooth bar with half blocks: ████▌░░░░░ */
function smoothBar(pct: number, width: number = 20): string {
  const clamped = Math.max(0, Math.min(100, pct))
  const filled = Math.floor((clamped / 100) * width * 2) // Double precision
  const fullBlocks = Math.floor(filled / 2)
  const halfBlock = filled % 2 === 1

  let bar = '█'.repeat(fullBlocks)
  if (halfBlock) bar += '▌'
  bar += '░'.repeat(Math.max(0, width - fullBlocks - (halfBlock ? 1 : 0)))

  return bar.slice(0, width)
}

/** Get gradient color based on percentage */
function gradientColor(pct: number): RGBA {
  const clamped = Math.max(0, Math.min(100, pct))
  const index = (clamped / 100) * (gradientColors.length - 1)
  const lower = Math.floor(index)
  const upper = Math.ceil(index)
  const t = index - lower

  if (lower === upper) return gradientColors[lower]!

  const c1 = gradientColors[lower]!
  const c2 = gradientColors[upper]!

  return {
    r: Math.round(c1.r + (c2.r - c1.r) * t),
    g: Math.round(c1.g + (c2.g - c1.g) * t),
    b: Math.round(c1.b + (c2.b - c1.b) * t),
    a: 255,
  }
}

/** Spinner frames for indeterminate progress */
const spinnerFrames = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏']

/** Bouncing bar for indeterminate progress */
function bouncingBar(pos: number, width: number = 20): string {
  const bounceWidth = 4
  const position = Math.floor(pos % ((width - bounceWidth) * 2))
  const actualPos = position < (width - bounceWidth)
    ? position
    : (width - bounceWidth) * 2 - position

  return '░'.repeat(actualPos) + '▓'.repeat(bounceWidth) + '░'.repeat(Math.max(0, width - bounceWidth - actualPos))
}

/** Format time as MM:SS */
function formatTime(seconds: number): string {
  const mins = Math.floor(seconds / 60)
  const secs = Math.floor(seconds % 60)
  return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`
}

// =============================================================================
// STATE
// =============================================================================

// Determinate progress values
const downloadProgress = signal(0)
const uploadProgress = signal(0)
const installProgress = signal(0)
const compileProgress = signal(0)

// Multiple file downloads
interface Download {
  name: string
  progress: number
  size: string
  speed: number
}

const downloads = signal<Download[]>([
  { name: 'package-1.tar.gz', progress: 0, size: '24.5 MB', speed: 1.2 },
  { name: 'package-2.tar.gz', progress: 0, size: '12.3 MB', speed: 2.1 },
  { name: 'package-3.tar.gz', progress: 0, size: '8.7 MB', speed: 3.4 },
  { name: 'package-4.tar.gz', progress: 0, size: '45.2 MB', speed: 0.8 },
])

// Indeterminate animation position
const bouncePos = signal(0)

// Elapsed time for ETA calculation
const startTime = Date.now()
const elapsedTime = signal(0)

// =============================================================================
// SIMULATION
// =============================================================================

function startSimulation(): () => void {
  // Main progress bars - different speeds
  const mainInterval = setInterval(() => {
    if (downloadProgress.value < 100) {
      downloadProgress.value = Math.min(100, downloadProgress.value + Math.random() * 3)
    }
    if (uploadProgress.value < 100) {
      uploadProgress.value = Math.min(100, uploadProgress.value + Math.random() * 1.5)
    }
    if (installProgress.value < 100) {
      installProgress.value = Math.min(100, installProgress.value + Math.random() * 2)
    }
    if (compileProgress.value < 100) {
      compileProgress.value = Math.min(100, compileProgress.value + Math.random() * 2.5)
    }

    // Reset when all complete
    if (downloadProgress.value >= 100 && uploadProgress.value >= 100 &&
        installProgress.value >= 100 && compileProgress.value >= 100) {
      setTimeout(() => {
        downloadProgress.value = 0
        uploadProgress.value = 0
        installProgress.value = 0
        compileProgress.value = 0
      }, 1500)
    }
  }, 100)

  // Multiple downloads
  const downloadInterval = setInterval(() => {
    const updated = downloads.value.map(d => ({
      ...d,
      progress: d.progress < 100
        ? Math.min(100, d.progress + d.speed * Math.random() * 2)
        : d.progress,
    }))

    // Reset completed downloads
    const allComplete = updated.every(d => d.progress >= 100)
    if (allComplete) {
      setTimeout(() => {
        downloads.value = downloads.value.map(d => ({ ...d, progress: 0 }))
      }, 1500)
    } else {
      downloads.value = updated
    }
  }, 100)

  // Bounce animation for indeterminate
  const bounceInterval = setInterval(() => {
    bouncePos.value = bouncePos.value + 1
  }, 50)

  // Update elapsed time
  const timeInterval = setInterval(() => {
    elapsedTime.value = Math.floor((Date.now() - startTime) / 1000)
  }, 1000)

  return () => {
    clearInterval(mainInterval)
    clearInterval(downloadInterval)
    clearInterval(bounceInterval)
    clearInterval(timeInterval)
  }
}

// =============================================================================
// COMPONENTS
// =============================================================================

function Section(props: { title: string; children: () => void }) {
  box({
    width: '100%',
    flexDirection: 'column',
    border: 1,
    borderColor: colors.borderDim,
    bg: colors.bgCard,
    marginBottom: 1,
    children: () => {
      // Title
      box({
        width: '100%',
        height: 1,
        paddingLeft: 1,
        bg: colors.bgPanel,
        children: () => {
          text({ content: props.title, fg: colors.cyan })
        },
      })
      // Content
      box({
        padding: 1,
        flexDirection: 'column',
        gap: 1,
        children: props.children,
      })
    },
  })
}

function ProgressRow(props: {
  label: string
  progress: () => number
  barFn: (pct: number, width: number) => string
  width?: number
  showPct?: boolean
  showEta?: boolean
  color?: RGBA | (() => RGBA)
}) {
  const barWidth = props.width ?? 25

  box({
    flexDirection: 'row',
    alignItems: 'center',
    gap: 1,
    children: () => {
      // Label
      text({ content: props.label.padEnd(12), fg: colors.textSecondary })

      // Bar
      text({
        content: derived(() => props.barFn(props.progress(), barWidth)),
        fg: props.color ?? colors.cyan,
      })

      // Percentage
      if (props.showPct !== false) {
        text({
          content: derived(() => `${Math.round(props.progress()).toString().padStart(3)}%`),
          fg: colors.textPrimary,
        })
      }

      // ETA
      if (props.showEta) {
        text({
          content: derived(() => {
            const pct = props.progress()
            if (pct >= 100) return 'Done!'
            if (pct <= 0) return ''
            const elapsed = elapsedTime.value
            const estimated = (elapsed / pct) * 100
            const remaining = estimated - elapsed
            return `ETA: ${formatTime(remaining)}`
          }),
          fg: colors.textMuted,
        })
      }
    },
  })
}

// =============================================================================
// MAIN
// =============================================================================

const cols = process.stdout.columns || 80
const rows = process.stdout.rows || 24

await mount(() => {
  const spinner = cycle(spinnerFrames, { fps: 12 })

  box({
    width: cols,
    height: rows,
    flexDirection: 'column',
    bg: colors.bgDark,
    padding: 1,
    children: () => {
      // Title
      box({
        width: '100%',
        height: 3,
        flexDirection: 'row',
        justifyContent: 'space-between',
        alignItems: 'center',
        border: 1,
        borderColor: colors.borderAccent,
        bg: colors.bgCard,
        paddingLeft: 2,
        paddingRight: 2,
        children: () => {
          box({
            flexDirection: 'row',
            gap: 1,
            children: () => {
              text({ content: spinner, fg: colors.cyan })
              text({ content: 'Progress Bar Showcase', fg: colors.cyan })
            },
          })
          text({
            content: derived(() => `Elapsed: ${formatTime(elapsedTime.value)}`),
            fg: colors.textSecondary,
          })
        },
      })

      // Content
      box({
        width: '100%',
        grow: 1,
        flexDirection: 'column',
        padding: 1,
        children: () => {
          // Section 1: Different Styles
          Section({
            title: '  Progress Bar Styles',
            children: () => {
              ProgressRow({
                label: 'Block',
                progress: () => downloadProgress.value,
                barFn: blockBar,
                color: colors.green,
              })
              ProgressRow({
                label: 'Dots',
                progress: () => downloadProgress.value,
                barFn: dotsBar,
                color: colors.purple,
              })
              ProgressRow({
                label: 'ASCII',
                progress: () => downloadProgress.value,
                barFn: asciiBar,
                color: colors.yellow,
              })
              ProgressRow({
                label: 'Thin',
                progress: () => downloadProgress.value,
                barFn: thinBar,
                color: colors.cyan,
              })
              ProgressRow({
                label: 'Smooth',
                progress: () => downloadProgress.value,
                barFn: smoothBar,
                color: colors.pink,
              })
            },
          })

          // Section 2: Gradient + ETA
          Section({
            title: '  Gradient Progress (with ETA)',
            children: () => {
              ProgressRow({
                label: 'Download',
                progress: () => downloadProgress.value,
                barFn: blockBar,
                showEta: true,
                color: derived(() => gradientColor(downloadProgress.value)),
              })
              ProgressRow({
                label: 'Upload',
                progress: () => uploadProgress.value,
                barFn: blockBar,
                showEta: true,
                color: derived(() => gradientColor(uploadProgress.value)),
              })
            },
          })

          // Section 3: Concurrent Downloads
          Section({
            title: '  Concurrent Downloads',
            children: () => {
              const dl = downloads.value
              for (const d of dl) {
                box({
                  flexDirection: 'row',
                  alignItems: 'center',
                  gap: 1,
                  children: () => {
                    text({
                      content: d.name.slice(0, 18).padEnd(18),
                      fg: colors.textSecondary,
                    })
                    text({
                      content: derived(() => {
                        const current = downloads.value.find(x => x.name === d.name)
                        return blockBar(current?.progress ?? 0, 20)
                      }),
                      fg: derived(() => {
                        const current = downloads.value.find(x => x.name === d.name)
                        return gradientColor(current?.progress ?? 0)
                      }),
                    })
                    text({
                      content: derived(() => {
                        const current = downloads.value.find(x => x.name === d.name)
                        return `${Math.round(current?.progress ?? 0).toString().padStart(3)}%`
                      }),
                      fg: colors.textPrimary,
                    })
                    text({
                      content: d.size.padStart(10),
                      fg: colors.textMuted,
                    })
                  },
                })
              }
            },
          })

          // Section 4: Indeterminate
          Section({
            title: '  Indeterminate Progress',
            children: () => {
              box({
                flexDirection: 'row',
                alignItems: 'center',
                gap: 1,
                children: () => {
                  text({ content: 'Loading'.padEnd(12), fg: colors.textSecondary })
                  text({
                    content: derived(() => bouncingBar(bouncePos.value, 25)),
                    fg: colors.cyan,
                  })
                  text({
                    content: 'Please wait...',
                    fg: colors.textMuted,
                  })
                },
              })

              box({
                flexDirection: 'row',
                alignItems: 'center',
                gap: 1,
                children: () => {
                  text({ content: 'Processing'.padEnd(12), fg: colors.textSecondary })
                  text({
                    content: cycle(['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'], { fps: 12 }),
                    fg: colors.orange,
                  })
                  text({
                    content: ' Analyzing data...',
                    fg: colors.textMuted,
                  })
                },
              })

              box({
                flexDirection: 'row',
                alignItems: 'center',
                gap: 1,
                children: () => {
                  text({ content: 'Searching'.padEnd(12), fg: colors.textSecondary })
                  text({
                    content: cycle(['[    ]', '[=   ]', '[==  ]', '[=== ]', '[ ===]', '[  ==]', '[   =]', '[    ]'], { fps: 4 }),
                    fg: colors.green,
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
        height: 1,
        justifyContent: 'center',
        children: () => {
          text({
            content: 'Press Ctrl+C to exit',
            fg: colors.textMuted,
          })
        },
      })
    },
  })

  const stopSimulation = startSimulation()
  onCleanup(stopSimulation)
}, {
  mode: 'fullscreen',
})
