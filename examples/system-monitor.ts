/**
 * SparkTUI - System Monitor
 *
 * A professional system stats display demonstrating:
 * - CPU usage bar (simulated)
 * - Memory usage with used/total display
 * - Network activity (up/down arrows with speeds)
 * - Process count
 * - Uptime display
 * - Updates every second
 *
 * This is PURELY REACTIVE. No render loops, no polling.
 * Signals update UI automatically through the reactive graph.
 *
 * Run: bun run examples/system-monitor.ts
 */

import { signal, derived } from '@rlabs-inc/signals'
import { mount } from '../ts/engine/mount'
import { box, text, each, cycle, pulse, Frames } from '../ts/primitives'
import { onCleanup } from '../ts/primitives/scope'
import { packColor } from '../ts/bridge/shared-buffer'

// =============================================================================
// COLORS
// =============================================================================

const colors = {
  // Backgrounds
  bgDark: packColor(18, 18, 24, 255),
  bgCard: packColor(28, 28, 38, 255),
  bgHeader: packColor(35, 35, 50, 255),
  bgProgress: packColor(40, 40, 55, 255),

  // Text
  textPrimary: packColor(240, 240, 250, 255),
  textSecondary: packColor(160, 160, 180, 255),
  textMuted: packColor(100, 100, 120, 255),
  textDim: packColor(70, 70, 90, 255),

  // Accents
  cyan: packColor(80, 200, 255, 255),
  green: packColor(80, 220, 140, 255),
  yellow: packColor(255, 200, 80, 255),
  red: packColor(255, 100, 100, 255),
  orange: packColor(255, 150, 80, 255),
  purple: packColor(180, 130, 255, 255),
  blue: packColor(100, 150, 255, 255),

  // Borders
  borderDim: packColor(50, 50, 70, 255),
  borderAccent: packColor(80, 140, 200, 255),
}

// =============================================================================
// STATE
// =============================================================================

// CPU metrics (per-core)
const cpuCores = signal([45, 32, 67, 28, 55, 41, 38, 52])
const cpuOverall = derived(() => {
  const cores = cpuCores.value
  return Math.round(cores.reduce((a, b) => a + b, 0) / cores.length)
})

// Memory metrics
const memoryUsed = signal(8.4) // GB
const memoryTotal = signal(16.0) // GB
const memoryPercent = derived(() =>
  Math.round((memoryUsed.value / memoryTotal.value) * 100)
)

// Swap metrics
const swapUsed = signal(1.2) // GB
const swapTotal = signal(4.0) // GB

// Disk metrics
const diskUsed = signal(234.5) // GB
const diskTotal = signal(512.0) // GB
const diskPercent = derived(() =>
  Math.round((diskUsed.value / diskTotal.value) * 100)
)

// Network metrics
const networkUp = signal(1.24) // MB/s
const networkDown = signal(5.67) // MB/s
const networkUpHistory = signal<number[]>([])
const networkDownHistory = signal<number[]>([])

// Process metrics
const processCount = signal(312)
const threadCount = signal(1847)
const runningProcesses = signal(4)

// System info
const uptime = signal(0) // seconds
const loadAvg = signal([1.24, 0.98, 0.87])

// =============================================================================
// HELPERS
// =============================================================================

function formatUptime(seconds: number): string {
  const days = Math.floor(seconds / 86400)
  const hours = Math.floor((seconds % 86400) / 3600)
  const mins = Math.floor((seconds % 3600) / 60)
  const secs = seconds % 60

  if (days > 0) {
    return `${days}d ${hours}h ${mins}m`
  }
  if (hours > 0) {
    return `${hours}h ${mins}m ${secs}s`
  }
  if (mins > 0) {
    return `${mins}m ${secs}s`
  }
  return `${secs}s`
}

function formatBytes(gb: number): string {
  return `${gb.toFixed(1)} GB`
}

function formatSpeed(mbps: number): string {
  if (mbps >= 1) {
    return `${mbps.toFixed(2)} MB/s`
  }
  return `${(mbps * 1024).toFixed(0)} KB/s`
}

function progressBar(percent: number, width: number = 20): string {
  const filled = Math.floor((percent / 100) * width)
  const empty = width - filled
  return '\u2588'.repeat(filled) + '\u2591'.repeat(empty)
}

function miniSparkline(history: number[], width: number = 10): string {
  if (history.length === 0) return '\u2581'.repeat(width)
  const chars = '\u2581\u2582\u2583\u2584\u2585\u2586\u2587\u2588'
  const max = Math.max(...history, 1)
  const recent = history.slice(-width)
  const padded = [...Array(width - recent.length).fill(0), ...recent]
  return padded.map(v => chars[Math.min(7, Math.floor((v / max) * 7))]!).join('')
}

function getUsageColor(percent: number): number {
  if (percent >= 90) return colors.red
  if (percent >= 70) return colors.yellow
  if (percent >= 50) return colors.orange
  return colors.green
}

// =============================================================================
// SIMULATION
// =============================================================================

function startSimulation(): () => void {
  let uptimeCounter = Math.floor(Math.random() * 100000)
  uptime.value = uptimeCounter

  const interval = setInterval(() => {
    // Update uptime
    uptimeCounter++
    uptime.value = uptimeCounter

    // Fluctuate CPU cores
    cpuCores.value = cpuCores.value.map(core =>
      Math.max(5, Math.min(100, core + (Math.random() - 0.5) * 20))
    )

    // Fluctuate memory (slower changes)
    memoryUsed.value = Math.max(
      2,
      Math.min(memoryTotal.value - 1, memoryUsed.value + (Math.random() - 0.5) * 0.5)
    )

    // Fluctuate swap
    swapUsed.value = Math.max(
      0,
      Math.min(swapTotal.value, swapUsed.value + (Math.random() - 0.5) * 0.1)
    )

    // Fluctuate disk (very slow)
    diskUsed.value = Math.max(
      50,
      Math.min(diskTotal.value - 10, diskUsed.value + (Math.random() - 0.5) * 0.5)
    )

    // Network fluctuation
    const newUp = Math.max(0.01, networkUp.value + (Math.random() - 0.5) * 0.5)
    const newDown = Math.max(0.05, networkDown.value + (Math.random() - 0.5) * 2)
    networkUp.value = newUp
    networkDown.value = newDown

    // Update history
    networkUpHistory.value = [...networkUpHistory.value.slice(-19), newUp]
    networkDownHistory.value = [...networkDownHistory.value.slice(-19), newDown]

    // Fluctuate process counts
    processCount.value = Math.max(
      100,
      Math.min(500, processCount.value + Math.floor((Math.random() - 0.5) * 10))
    )
    threadCount.value = Math.max(
      500,
      Math.min(3000, threadCount.value + Math.floor((Math.random() - 0.5) * 50))
    )
    runningProcesses.value = Math.max(
      1,
      Math.min(20, runningProcesses.value + Math.floor((Math.random() - 0.5) * 3))
    )

    // Update load average
    loadAvg.value = loadAvg.value.map(v =>
      Math.max(0.1, Math.min(8, v + (Math.random() - 0.5) * 0.2))
    )
  }, 1000)

  return () => clearInterval(interval)
}

// =============================================================================
// COMPONENTS
// =============================================================================

function MetricCard(props: {
  title: string
  width?: number
  children: () => void
}) {
  box({
    width: props.width ?? 30,
    border: 1,
    borderColor: colors.borderDim,
    bg: colors.bgCard,
    padding: 1,
    flexDirection: 'column',
    children: () => {
      text({ content: props.title, fg: colors.textSecondary })
      props.children()
    },
  })
}

function ProgressMetric(props: {
  label: string
  percent: () => number
  detail: () => string
  barWidth?: number
}) {
  box({
    flexDirection: 'column',
    marginTop: 1,
    children: () => {
      // Label and percentage
      box({
        flexDirection: 'row',
        justifyContent: 'space-between',
        children: () => {
          text({ content: props.label, fg: colors.textMuted })
          text({
            content: derived(() => `${props.percent()}%`),
            fg: derived(() => getUsageColor(props.percent())),
          })
        },
      })
      // Progress bar
      text({
        content: derived(() => progressBar(props.percent(), props.barWidth ?? 20)),
        fg: derived(() => getUsageColor(props.percent())),
      })
      // Detail
      text({
        content: derived(props.detail),
        fg: colors.textDim,
      })
    },
  })
}

function CPUCoreGrid() {
  box({
    flexDirection: 'row',
    flexWrap: 'wrap',
    gap: 1,
    marginTop: 1,
    children: () => {
      for (let i = 0; i < 8; i++) {
        box({
          width: 12,
          flexDirection: 'column',
          children: () => {
            text({
              content: `Core ${i}`,
              fg: colors.textDim,
            })
            text({
              content: derived(() => {
                const pct = Math.round(cpuCores.value[i] ?? 0)
                return progressBar(pct, 8) + ` ${pct}%`
              }),
              fg: derived(() => getUsageColor(cpuCores.value[i] ?? 0)),
            })
          },
        })
      }
    },
  })
}

// =============================================================================
// MAIN APP
// =============================================================================

const cols = process.stdout.columns || 80
const rows = process.stdout.rows || 24

mount(() => {
  // Start simulation
  const stopSimulation = startSimulation()
  onCleanup(stopSimulation)

  // Pulsing live indicator
  const livePulse = pulse({ fps: 2 })

  // Root container
  box({
    width: cols,
    height: rows,
    flexDirection: 'column',
    bg: colors.bgDark,
    children: () => {
      // ===== HEADER =====
      box({
        width: '100%',
        height: 3,
        flexDirection: 'row',
        justifyContent: 'space-between',
        alignItems: 'center',
        border: 1,
        borderColor: colors.borderAccent,
        bg: colors.bgHeader,
        paddingLeft: 2,
        paddingRight: 2,
        children: () => {
          // Title
          box({
            flexDirection: 'row',
            gap: 2,
            alignItems: 'center',
            children: () => {
              text({ content: 'System Monitor', fg: colors.cyan })
              text({
                content: derived(() => livePulse.value ? '\u25CF' : '\u25CB'),
                fg: colors.green,
              })
            },
          })

          // System info
          box({
            flexDirection: 'row',
            gap: 3,
            alignItems: 'center',
            children: () => {
              text({
                content: derived(() => `Uptime: ${formatUptime(uptime.value)}`),
                fg: colors.textSecondary,
              })
              text({
                content: derived(() =>
                  `Load: ${loadAvg.value.map(v => v.toFixed(2)).join(' ')}`
                ),
                fg: colors.textMuted,
              })
            },
          })
        },
      })

      // ===== MAIN CONTENT =====
      box({
        width: '100%',
        grow: 1,
        flexDirection: 'row',
        padding: 1,
        gap: 1,
        children: () => {
          // LEFT COLUMN - CPU & Memory
          box({
            flexDirection: 'column',
            gap: 1,
            grow: 1,
            children: () => {
              // CPU Overview
              MetricCard({
                title: 'CPU Usage',
                children: () => {
                  // Overall CPU
                  box({
                    flexDirection: 'row',
                    alignItems: 'center',
                    gap: 2,
                    marginTop: 1,
                    children: () => {
                      text({
                        content: cycle(Frames.spinner, { fps: 10 }),
                        fg: colors.cyan,
                      })
                      text({
                        content: derived(() => `Overall: ${cpuOverall.value}%`),
                        fg: derived(() => getUsageColor(cpuOverall.value)),
                      })
                      text({
                        content: derived(() => progressBar(cpuOverall.value, 15)),
                        fg: derived(() => getUsageColor(cpuOverall.value)),
                      })
                    },
                  })

                  // Per-core grid
                  CPUCoreGrid()
                },
              })

              // Memory
              MetricCard({
                title: 'Memory',
                children: () => {
                  ProgressMetric({
                    label: 'RAM',
                    percent: () => memoryPercent.value,
                    detail: () =>
                      `${formatBytes(memoryUsed.value)} / ${formatBytes(memoryTotal.value)}`,
                  })
                  ProgressMetric({
                    label: 'Swap',
                    percent: () => Math.round((swapUsed.value / swapTotal.value) * 100),
                    detail: () =>
                      `${formatBytes(swapUsed.value)} / ${formatBytes(swapTotal.value)}`,
                  })
                },
              })
            },
          })

          // RIGHT COLUMN - Network, Disk, Processes
          box({
            flexDirection: 'column',
            gap: 1,
            grow: 1,
            children: () => {
              // Network
              MetricCard({
                title: 'Network',
                children: () => {
                  box({
                    flexDirection: 'column',
                    marginTop: 1,
                    children: () => {
                      // Upload
                      box({
                        flexDirection: 'row',
                        gap: 2,
                        children: () => {
                          text({ content: '\u2191 Up', fg: colors.green })
                          text({
                            content: derived(() => formatSpeed(networkUp.value)),
                            fg: colors.textPrimary,
                          })
                          text({
                            content: derived(() => miniSparkline(networkUpHistory.value)),
                            fg: colors.green,
                          })
                        },
                      })
                      // Download
                      box({
                        flexDirection: 'row',
                        gap: 2,
                        marginTop: 1,
                        children: () => {
                          text({ content: '\u2193 Dn', fg: colors.cyan })
                          text({
                            content: derived(() => formatSpeed(networkDown.value)),
                            fg: colors.textPrimary,
                          })
                          text({
                            content: derived(() => miniSparkline(networkDownHistory.value)),
                            fg: colors.cyan,
                          })
                        },
                      })
                    },
                  })
                },
              })

              // Disk
              MetricCard({
                title: 'Disk Usage',
                children: () => {
                  ProgressMetric({
                    label: 'Root (/)',
                    percent: () => diskPercent.value,
                    detail: () =>
                      `${formatBytes(diskUsed.value)} / ${formatBytes(diskTotal.value)}`,
                  })
                },
              })

              // Processes
              MetricCard({
                title: 'Processes',
                children: () => {
                  box({
                    flexDirection: 'column',
                    marginTop: 1,
                    gap: 1,
                    children: () => {
                      box({
                        flexDirection: 'row',
                        justifyContent: 'space-between',
                        children: () => {
                          text({ content: 'Total', fg: colors.textMuted })
                          text({
                            content: processCount,
                            fg: colors.textPrimary,
                          })
                        },
                      })
                      box({
                        flexDirection: 'row',
                        justifyContent: 'space-between',
                        children: () => {
                          text({ content: 'Running', fg: colors.textMuted })
                          text({
                            content: runningProcesses,
                            fg: colors.green,
                          })
                        },
                      })
                      box({
                        flexDirection: 'row',
                        justifyContent: 'space-between',
                        children: () => {
                          text({ content: 'Threads', fg: colors.textMuted })
                          text({
                            content: threadCount,
                            fg: colors.textSecondary,
                          })
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

      // ===== FOOTER =====
      box({
        width: '100%',
        height: 2,
        flexDirection: 'row',
        justifyContent: 'space-between',
        alignItems: 'center',
        paddingLeft: 2,
        paddingRight: 2,
        borderTop: 1,
        borderColor: colors.borderDim,
        children: () => {
          text({
            content: 'SparkTUI System Monitor - Real-time metrics',
            fg: colors.textMuted,
          })
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

console.log('[system-monitor] App mounted')

// Keep process alive
await new Promise(() => {})
