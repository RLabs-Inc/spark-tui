/**
 * SparkTUI - System Dashboard
 *
 * A beautiful system monitoring dashboard showcasing data visualization:
 * - Multiple panels (CPU, Memory, Network, Disk)
 * - Animated progress bars using block characters
 * - Sparkline charts using braille characters
 * - Real-time updates with simulated data
 * - Theme-aware colors
 *
 * This is PURELY REACTIVE. No render loops, no polling.
 * Signals update UI automatically through the reactive graph.
 *
 * Run: bun run examples/dashboard.ts
 */

import { signal, derived } from '@rlabs-inc/signals'
import { mount } from '../ts/engine/mount'
import { box } from '../ts/primitives/box'
import { text } from '../ts/primitives/text'
import { onCleanup } from '../ts/primitives/scope'
import { cycle, pulse, Frames } from '../ts/primitives/animation'
import { t } from '../ts/state/theme'
import type { RGBA } from '../ts/types'

// =============================================================================
// COLORS - Semantic palette
// =============================================================================

function rgba(r: number, g: number, b: number, a: number = 255): RGBA {
  return { r, g, b, a }
}

const colors = {
  // Backgrounds
  bgDark: rgba(16, 16, 22),
  bgCard: rgba(24, 24, 32),
  bgPanel: rgba(32, 32, 42),

  // Text
  textPrimary: rgba(235, 235, 245),
  textSecondary: rgba(160, 160, 180),
  textMuted: rgba(100, 100, 120),

  // Semantic colors
  cyan: rgba(80, 200, 255),
  green: rgba(80, 220, 140),
  yellow: rgba(255, 200, 80),
  red: rgba(255, 100, 100),
  orange: rgba(255, 160, 80),
  purple: rgba(180, 130, 255),
  blue: rgba(100, 150, 255),
  pink: rgba(255, 130, 180),

  // Border
  borderDim: rgba(50, 50, 65),
  borderAccent: rgba(80, 140, 200),
}

// =============================================================================
// METRICS DATA - Simulated system data
// =============================================================================

// System metrics
const cpuUsage = signal(42)
const memoryUsage = signal(65)
const diskUsage = signal(78)
const networkUp = signal(2.4)
const networkDown = signal(8.7)
const cpuTemp = signal(58)

// History arrays for sparklines (last 30 values)
const cpuHistory = signal<number[]>([45, 48, 52, 47, 43, 46, 50, 55, 52, 48, 44, 42, 45, 48, 52, 55, 58, 54, 50, 47, 44, 42, 45, 48, 50, 52, 48, 44, 42, 45])
const memHistory = signal<number[]>([60, 62, 64, 63, 65, 67, 68, 66, 65, 64, 63, 65, 67, 69, 68, 66, 65, 64, 66, 68, 67, 65, 64, 63, 65, 67, 68, 66, 65, 64])
const netHistory = signal<number[]>([5, 8, 12, 9, 6, 4, 7, 11, 15, 12, 8, 5, 6, 9, 13, 10, 7, 5, 8, 12, 14, 11, 8, 6, 7, 10, 13, 9, 6, 5])

// Process list
interface ProcessInfo {
  name: string
  cpu: number
  mem: number
  pid: number
}

const processes = signal<ProcessInfo[]>([
  { name: 'rust-analyzer', cpu: 12.4, mem: 340, pid: 1234 },
  { name: 'node', cpu: 8.2, mem: 180, pid: 5678 },
  { name: 'bun', cpu: 5.1, mem: 95, pid: 9012 },
  { name: 'vscode', cpu: 4.8, mem: 520, pid: 3456 },
  { name: 'chrome', cpu: 3.2, mem: 890, pid: 7890 },
])

// Live clock
const clock = signal(new Date().toLocaleTimeString())

// =============================================================================
// VISUALIZATION HELPERS
// =============================================================================

/** Progress bar using block characters */
const BLOCKS = ['▏', '▎', '▍', '▌', '▋', '▊', '▉', '█']

function progressBar(value: number, width: number = 12): string {
  const clamped = Math.max(0, Math.min(100, value))
  const fullBlocks = Math.floor((clamped / 100) * width)
  const partialBlock = Math.floor(((clamped / 100) * width - fullBlocks) * 8)

  let bar = '█'.repeat(fullBlocks)
  if (partialBlock > 0 && fullBlocks < width) {
    bar += BLOCKS[partialBlock - 1]
  }
  const remaining = width - fullBlocks - (partialBlock > 0 ? 1 : 0)
  bar += '░'.repeat(Math.max(0, remaining))

  return bar
}

/** Braille sparkline chart */
const BRAILLE_PATTERNS = [
  '⣀', '⣤', '⣶', '⣿',  // Bottom to top for single height
]

function sparkline(history: number[], width: number = 20): string {
  if (history.length === 0) return '⣀'.repeat(width)

  // Take last 'width' values, or pad if needed
  const values = history.slice(-width)
  while (values.length < width) values.unshift(values[0] ?? 0)

  const max = Math.max(...values, 1)
  const min = Math.min(...values)
  const range = max - min || 1

  return values.map(v => {
    const normalized = (v - min) / range
    const index = Math.min(3, Math.floor(normalized * 4))
    return BRAILLE_PATTERNS[index]
  }).join('')
}

/** More detailed braille sparkline using 2-height patterns */
const BRAILLE_2H = [
  ['⡀', '⡄', '⡆', '⡇'],  // Row 1 (low)
  ['⣀', '⣄', '⣆', '⣇'],  // Row 2
  ['⣠', '⣤', '⣦', '⣧'],  // Row 3
  ['⣰', '⣴', '⣶', '⣷'],  // Row 4 (high)
]

function detailedSparkline(history: number[]): string {
  if (history.length === 0) return ''

  const max = Math.max(...history, 1)
  return history.map(v => {
    const level = Math.min(15, Math.floor((v / max) * 16))
    const row = Math.floor(level / 4)
    const col = level % 4
    return BRAILLE_2H[row]?.[col] ?? '⣀'
  }).join('')
}

/** Format bytes */
function formatBytes(mb: number): string {
  return `${mb.toFixed(1)} MB/s`
}

/** Get color based on usage percentage */
function usageColor(pct: number): RGBA {
  if (pct >= 90) return colors.red
  if (pct >= 75) return colors.orange
  if (pct >= 50) return colors.yellow
  return colors.green
}

// =============================================================================
// SIMULATION
// =============================================================================

function startSimulation(): () => void {
  // Update metrics every 500ms
  const metricsInterval = setInterval(() => {
    // CPU fluctuates more
    cpuUsage.value = Math.max(5, Math.min(95, cpuUsage.value + (Math.random() - 0.5) * 20))

    // Memory changes slowly
    memoryUsage.value = Math.max(30, Math.min(90, memoryUsage.value + (Math.random() - 0.5) * 5))

    // Disk very slow changes
    if (Math.random() < 0.1) {
      diskUsage.value = Math.max(50, Math.min(95, diskUsage.value + (Math.random() - 0.5) * 2))
    }

    // Network fluctuates
    networkUp.value = Math.max(0.1, networkUp.value + (Math.random() - 0.5) * 2)
    networkDown.value = Math.max(0.5, networkDown.value + (Math.random() - 0.5) * 4)

    // Temperature
    cpuTemp.value = Math.max(40, Math.min(85, cpuTemp.value + (Math.random() - 0.5) * 5))

    // Update histories
    cpuHistory.value = [...cpuHistory.value.slice(-29), cpuUsage.value]
    memHistory.value = [...memHistory.value.slice(-29), memoryUsage.value]
    netHistory.value = [...netHistory.value.slice(-29), networkDown.value]
  }, 500)

  // Update clock every second
  const clockInterval = setInterval(() => {
    clock.value = new Date().toLocaleTimeString()
  }, 1000)

  // Update processes occasionally
  const processInterval = setInterval(() => {
    const procs = [...processes.value]
    for (const p of procs) {
      p.cpu = Math.max(0.1, Math.min(30, p.cpu + (Math.random() - 0.5) * 3))
    }
    // Sort by CPU
    procs.sort((a, b) => b.cpu - a.cpu)
    processes.value = procs
  }, 2000)

  return () => {
    clearInterval(metricsInterval)
    clearInterval(clockInterval)
    clearInterval(processInterval)
  }
}

// =============================================================================
// COMPONENTS
// =============================================================================

/** Panel with title and content */
function Panel(props: { title: string; width?: number | string; height?: number | string; grow?: number; children: () => void }) {
  box({
    width: props.width,
    height: props.height,
    grow: props.grow,
    flexDirection: 'column',
    border: 1,
    borderColor: colors.borderDim,
    bg: colors.bgCard,
    children: () => {
      // Title bar
      box({
        width: '100%',
        height: 1,
        paddingLeft: 1,
        paddingRight: 1,
        bg: colors.bgPanel,
        children: () => {
          text({ content: props.title, fg: colors.cyan })
        },
      })
      // Content
      box({
        grow: 1,
        padding: 1,
        flexDirection: 'column',
        children: props.children,
      })
    },
  })
}

/** Metric row with label, bar, and value */
function MetricRow(props: {
  label: string
  value: () => number
  suffix?: string
  width?: number
}) {
  const barWidth = props.width ?? 12

  box({
    flexDirection: 'row',
    gap: 1,
    height: 1,
    alignItems: 'center',
    children: () => {
      // Label
      text({ content: props.label.padEnd(8), fg: colors.textSecondary })

      // Progress bar
      text({
        content: derived(() => progressBar(props.value(), barWidth)),
        fg: derived(() => usageColor(props.value())),
      })

      // Value
      text({
        content: derived(() => `${Math.round(props.value()).toString().padStart(3)}${props.suffix ?? '%'}`),
        fg: colors.textPrimary,
      })
    },
  })
}

// =============================================================================
// MAIN DASHBOARD
// =============================================================================

const cols = process.stdout.columns || 80
const rows = process.stdout.rows || 24

await mount(() => {
  const livePulse = pulse({ fps: 2 })

  // Root container
  box({
    width: cols,
    height: rows,
    flexDirection: 'column',
    bg: colors.bgDark,
    children: () => {
      // Header
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
          // Title
          box({
            flexDirection: 'row',
            gap: 1,
            alignItems: 'center',
            children: () => {
              text({
                content: cycle(Frames.spinner, { fps: 10 }),
                fg: colors.cyan,
              })
              text({ content: 'System Monitor', fg: colors.cyan })
            },
          })

          // Live indicator + clock
          box({
            flexDirection: 'row',
            gap: 2,
            alignItems: 'center',
            children: () => {
              text({
                content: derived(() => livePulse.value ? '●' : '○'),
                fg: colors.green,
              })
              text({ content: 'LIVE', fg: colors.green })
              text({ content: clock, fg: colors.textSecondary })
            },
          })
        },
      })

      // Main content area
      box({
        width: '100%',
        grow: 1,
        flexDirection: 'row',
        gap: 1,
        padding: 1,
        children: () => {
          // Left column - System metrics
          box({
            width: '50%',
            flexDirection: 'column',
            gap: 1,
            children: () => {
              // CPU Panel
              Panel({
                title: '  CPU',
                height: 7,
                children: () => {
                  MetricRow({ label: 'Usage', value: () => cpuUsage.value })
                  MetricRow({ label: 'Temp', value: () => cpuTemp.value, suffix: '°C' })
                  box({ height: 1 })
                  text({ content: 'History:', fg: colors.textMuted })
                  text({
                    content: derived(() => sparkline(cpuHistory.value, 30)),
                    fg: colors.cyan,
                  })
                },
              })

              // Memory Panel
              Panel({
                title: '  Memory',
                height: 6,
                children: () => {
                  MetricRow({ label: 'Used', value: () => memoryUsage.value })
                  box({ height: 1 })
                  text({ content: 'History:', fg: colors.textMuted })
                  text({
                    content: derived(() => sparkline(memHistory.value, 30)),
                    fg: colors.purple,
                  })
                },
              })

              // Disk Panel
              Panel({
                title: '  Disk',
                height: 4,
                children: () => {
                  MetricRow({ label: '/dev/sda', value: () => diskUsage.value })
                },
              })
            },
          })

          // Right column - Network & Processes
          box({
            width: '50%',
            flexDirection: 'column',
            gap: 1,
            children: () => {
              // Network Panel
              Panel({
                title: '  Network',
                height: 8,
                children: () => {
                  box({
                    flexDirection: 'row',
                    gap: 2,
                    children: () => {
                      text({ content: '↑', fg: colors.green })
                      text({
                        content: derived(() => formatBytes(networkUp.value)),
                        fg: colors.textPrimary,
                      })
                      text({ content: '↓', fg: colors.cyan })
                      text({
                        content: derived(() => formatBytes(networkDown.value)),
                        fg: colors.textPrimary,
                      })
                    },
                  })
                  box({ height: 1 })
                  text({ content: 'Traffic:', fg: colors.textMuted })
                  text({
                    content: derived(() => detailedSparkline(netHistory.value)),
                    fg: colors.green,
                  })
                },
              })

              // Processes Panel
              Panel({
                title: '  Top Processes',
                grow: 1,
                children: () => {
                  // Header row
                  box({
                    flexDirection: 'row',
                    children: () => {
                      text({ content: 'NAME'.padEnd(15), fg: colors.textMuted })
                      text({ content: 'CPU%'.padStart(8), fg: colors.textMuted })
                      text({ content: 'MEM'.padStart(8), fg: colors.textMuted })
                    },
                  })

                  // Process rows
                  box({
                    flexDirection: 'column',
                    gap: 0,
                    children: () => {
                      const procs = processes.value
                      for (let i = 0; i < Math.min(5, procs.length); i++) {
                        const p = procs[i]!
                        box({
                          flexDirection: 'row',
                          children: () => {
                            text({ content: p.name.padEnd(15), fg: colors.textPrimary })
                            text({
                              content: p.cpu.toFixed(1).padStart(8),
                              fg: p.cpu > 10 ? colors.orange : colors.textSecondary,
                            })
                            text({
                              content: `${p.mem}MB`.padStart(8),
                              fg: colors.textSecondary,
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

  // Start simulation and register cleanup
  const stopSimulation = startSimulation()
  onCleanup(stopSimulation)
}, {
  mode: 'fullscreen',
})
