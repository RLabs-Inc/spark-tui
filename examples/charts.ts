/**
 * SparkTUI - ASCII Charts Collection
 *
 * A beautiful showcase of ASCII/Unicode chart types:
 * - Horizontal bar chart
 * - Vertical bar chart using block characters
 * - Line chart using braille
 * - Pie chart approximation
 * - All with labels and legends
 *
 * Run: bun run examples/charts.ts
 */

import { signal, derived } from '@rlabs-inc/signals'
import { mount } from '../ts/engine/mount'
import { box } from '../ts/primitives/box'
import { text } from '../ts/primitives/text'
import { onCleanup } from '../ts/primitives/scope'
import { cycle } from '../ts/primitives/animation'
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

  textPrimary: rgba(235, 235, 245),
  textSecondary: rgba(160, 160, 180),
  textMuted: rgba(100, 100, 120),

  // Chart colors
  chartColors: [
    rgba(80, 200, 255),   // Cyan
    rgba(80, 220, 140),   // Green
    rgba(255, 200, 80),   // Yellow
    rgba(255, 100, 100),  // Red
    rgba(180, 130, 255),  // Purple
    rgba(255, 160, 80),   // Orange
    rgba(255, 130, 180),  // Pink
    rgba(100, 180, 255),  // Blue
  ],

  borderDim: rgba(50, 50, 65),
  borderAccent: rgba(80, 140, 200),
}

// =============================================================================
// CHART DATA
// =============================================================================

interface DataPoint {
  label: string
  value: number
}

// Horizontal bar chart data - Programming languages
const languageData = signal<DataPoint[]>([
  { label: 'TypeScript', value: 85 },
  { label: 'Python', value: 72 },
  { label: 'Rust', value: 65 },
  { label: 'Go', value: 48 },
  { label: 'Java', value: 42 },
  { label: 'C++', value: 38 },
])

// Vertical bar chart data - Monthly sales
const salesData = signal<DataPoint[]>([
  { label: 'Jan', value: 45 },
  { label: 'Feb', value: 52 },
  { label: 'Mar', value: 68 },
  { label: 'Apr', value: 55 },
  { label: 'May', value: 78 },
  { label: 'Jun', value: 82 },
  { label: 'Jul', value: 95 },
  { label: 'Aug', value: 88 },
])

// Line chart data - Temperature over time
const tempHistory = signal<number[]>([
  22, 24, 25, 27, 30, 32, 35, 33, 30, 28, 25, 23,
  22, 21, 20, 22, 24, 28, 31, 34, 36, 33, 29, 26
])

// Pie chart data - Market share
const marketShare = signal<DataPoint[]>([
  { label: 'Product A', value: 35 },
  { label: 'Product B', value: 28 },
  { label: 'Product C', value: 20 },
  { label: 'Product D', value: 12 },
  { label: 'Other', value: 5 },
])

// =============================================================================
// CHART RENDERERS
// =============================================================================

/** Horizontal bar character blocks */
function horizontalBar(value: number, maxValue: number, width: number): string {
  const normalizedWidth = Math.floor((value / maxValue) * width)
  return '█'.repeat(normalizedWidth) + '░'.repeat(Math.max(0, width - normalizedWidth))
}

/** Vertical bar using block characters (bottom to top) */
const verticalBlocks = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█']

function verticalBarRow(data: DataPoint[], maxValue: number, row: number, totalRows: number): string {
  const threshold = (totalRows - row) / totalRows

  return data.map(d => {
    const normalizedValue = d.value / maxValue
    if (normalizedValue >= threshold) {
      // Calculate which block character to use for smooth gradient
      const diff = normalizedValue - threshold
      const rowHeight = 1 / totalRows
      if (diff >= rowHeight) {
        return '█'
      } else {
        const blockIndex = Math.floor((diff / rowHeight) * 8)
        return verticalBlocks[Math.min(7, blockIndex)] || '█'
      }
    }
    return ' '
  }).join('  ')
}

/** Braille line chart */
const braillePatterns: { [key: string]: string } = {
  // Row position (0-3) maps to braille dots
  '0': '⠀', '1': '⡀', '2': '⠄', '3': '⠂', '4': '⠁',
  '01': '⡄', '02': '⠤', '03': '⠒', '04': '⠑',
  '12': '⡤', '13': '⡒', '14': '⡑',
  '23': '⠖', '24': '⠕',
  '34': '⠓',
}

function brailleLine(data: number[], width: number): string[] {
  const lines: string[] = ['', '', '']
  const min = Math.min(...data)
  const max = Math.max(...data)
  const range = max - min || 1

  // Normalize data to 0-8 range (for 4 rows * 2 dots)
  const normalized = data.map(v => Math.floor(((v - min) / range) * 8))

  // Take pairs of values for braille characters
  for (let i = 0; i < width && i * 2 < data.length; i++) {
    const v1 = normalized[i * 2] ?? 0
    const v2 = normalized[i * 2 + 1] ?? v1

    // Convert to braille dots
    // Row 0 (top): dots at y=6,7
    // Row 1 (mid): dots at y=4,5
    // Row 2 (bot): dots at y=2,3
    // Row 3 (bottom): dots at y=0,1

    const row0 = v1 >= 6 || v2 >= 6
    const row1 = (v1 >= 4 && v1 < 6) || (v2 >= 4 && v2 < 6)
    const row2 = (v1 >= 2 && v1 < 4) || (v2 >= 2 && v2 < 4)

    // Simple character mapping
    lines[0] += row0 ? (v1 >= 7 || v2 >= 7 ? '⣿' : '⣤') : '⠀'
    lines[1] += row1 ? '⠤' : '⠀'
    lines[2] += row2 ? '⣀' : '⠀'
  }

  return lines
}

/** Simple sparkline for compact line display */
function sparkline(data: number[]): string {
  const min = Math.min(...data)
  const max = Math.max(...data)
  const range = max - min || 1
  const chars = '▁▂▃▄▅▆▇█'

  return data.map(v => {
    const normalized = (v - min) / range
    const index = Math.min(7, Math.floor(normalized * 8))
    return chars[index]
  }).join('')
}

/** Pie chart using segments (approximation) */
const pieSegments = ['○', '◔', '◑', '◕', '●']

function pieSlice(percentage: number, filled: boolean = true): string {
  if (percentage <= 0) return '○'
  if (percentage >= 100) return '●'

  const index = Math.min(4, Math.floor((percentage / 100) * 5))
  return pieSegments[index] || '○'
}

/** ASCII pie chart legend with percentages */
function pieLegend(data: DataPoint[]): Array<{ label: string; pct: number; color: RGBA }> {
  const total = data.reduce((sum, d) => sum + d.value, 0)
  return data.map((d, i) => ({
    label: d.label,
    pct: Math.round((d.value / total) * 100),
    color: colors.chartColors[i % colors.chartColors.length]!,
  }))
}

// =============================================================================
// SIMULATION
// =============================================================================

function startSimulation(): () => void {
  // Fluctuate data periodically
  const interval = setInterval(() => {
    // Update language data
    languageData.value = languageData.value.map(d => ({
      ...d,
      value: Math.max(10, Math.min(100, d.value + (Math.random() - 0.5) * 10)),
    }))

    // Update sales data
    salesData.value = salesData.value.map(d => ({
      ...d,
      value: Math.max(20, Math.min(100, d.value + (Math.random() - 0.5) * 15)),
    }))

    // Update temperature history (shift and add new)
    const temps = [...tempHistory.value.slice(1)]
    const lastTemp = temps[temps.length - 1] ?? 25
    temps.push(Math.max(15, Math.min(40, lastTemp + (Math.random() - 0.5) * 5)))
    tempHistory.value = temps

    // Update market share (small fluctuations)
    const shares = marketShare.value.map(d => ({
      ...d,
      value: Math.max(1, d.value + (Math.random() - 0.5) * 3),
    }))
    // Normalize to 100%
    const total = shares.reduce((sum, d) => sum + d.value, 0)
    marketShare.value = shares.map(d => ({
      ...d,
      value: Math.round((d.value / total) * 100),
    }))
  }, 2000)

  return () => clearInterval(interval)
}

// =============================================================================
// COMPONENTS
// =============================================================================

function ChartPanel(props: { title: string; height?: number; children: () => void }) {
  box({
    width: '100%',
    height: props.height,
    flexDirection: 'column',
    border: 1,
    borderColor: colors.borderDim,
    bg: colors.bgCard,
    marginBottom: 1,
    children: () => {
      box({
        width: '100%',
        height: 1,
        paddingLeft: 1,
        bg: colors.bgPanel,
        children: () => {
          text({ content: props.title, fg: colors.chartColors[0] })
        },
      })
      box({
        grow: 1,
        padding: 1,
        flexDirection: 'column',
        children: props.children,
      })
    },
  })
}

// =============================================================================
// MAIN
// =============================================================================

const cols = process.stdout.columns || 80
const rows = process.stdout.rows || 24

await mount(() => {
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
        justifyContent: 'center',
        alignItems: 'center',
        border: 1,
        borderColor: colors.borderAccent,
        bg: colors.bgCard,
        children: () => {
          text({ content: '  ASCII Charts Collection', fg: colors.chartColors[0] })
        },
      })

      // Main content - two columns
      box({
        width: '100%',
        grow: 1,
        flexDirection: 'row',
        gap: 1,
        marginTop: 1,
        children: () => {
          // Left column
          box({
            width: '50%',
            flexDirection: 'column',
            children: () => {
              // Horizontal Bar Chart
              ChartPanel({
                title: '  Horizontal Bar - Languages',
                height: 10,
                children: () => {
                  const data = languageData.value
                  const maxVal = Math.max(...data.map(d => d.value))

                  for (let i = 0; i < data.length; i++) {
                    const d = data[i]!
                    const color = colors.chartColors[i % colors.chartColors.length]!

                    box({
                      flexDirection: 'row',
                      gap: 1,
                      children: () => {
                        text({
                          content: d.label.padEnd(12),
                          fg: colors.textSecondary,
                        })
                        text({
                          content: derived(() => {
                            const current = languageData.value.find(x => x.label === d.label)
                            return horizontalBar(current?.value ?? 0, 100, 18)
                          }),
                          fg: color,
                        })
                        text({
                          content: derived(() => {
                            const current = languageData.value.find(x => x.label === d.label)
                            return `${Math.round(current?.value ?? 0)}%`
                          }),
                          fg: colors.textPrimary,
                        })
                      },
                    })
                  }
                },
              })

              // Sparkline Chart
              ChartPanel({
                title: '  Sparkline - Temperature',
                height: 5,
                children: () => {
                  box({
                    flexDirection: 'row',
                    gap: 2,
                    children: () => {
                      text({ content: 'Temp:', fg: colors.textSecondary })
                      text({
                        content: derived(() => sparkline(tempHistory.value)),
                        fg: colors.chartColors[1]!,
                      })
                    },
                  })
                  box({
                    flexDirection: 'row',
                    gap: 1,
                    marginTop: 1,
                    children: () => {
                      text({
                        content: derived(() => {
                          const min = Math.min(...tempHistory.value)
                          const max = Math.max(...tempHistory.value)
                          const avg = tempHistory.value.reduce((a, b) => a + b, 0) / tempHistory.value.length
                          return `Min: ${min.toFixed(0)}°  Max: ${max.toFixed(0)}°  Avg: ${avg.toFixed(1)}°`
                        }),
                        fg: colors.textMuted,
                      })
                    },
                  })
                },
              })

              // Pie Chart
              ChartPanel({
                title: '  Pie Chart - Market Share',
                height: 8,
                children: () => {
                  // Legend entries
                  const data = marketShare.value
                  const total = data.reduce((sum, d) => sum + d.value, 0)

                  for (let i = 0; i < data.length; i++) {
                    const d = data[i]!
                    const color = colors.chartColors[i % colors.chartColors.length]!
                    const pct = Math.round((d.value / total) * 100)

                    box({
                      flexDirection: 'row',
                      gap: 1,
                      children: () => {
                        // Colored indicator
                        text({
                          content: '█',
                          fg: color,
                        })
                        // Label
                        text({
                          content: d.label.padEnd(12),
                          fg: colors.textSecondary,
                        })
                        // Bar representation
                        text({
                          content: derived(() => {
                            const current = marketShare.value.find(x => x.label === d.label)
                            const currentTotal = marketShare.value.reduce((sum, x) => sum + x.value, 0)
                            const currentPct = Math.round(((current?.value ?? 0) / currentTotal) * 100)
                            return '█'.repeat(Math.floor(currentPct / 5)) + '░'.repeat(20 - Math.floor(currentPct / 5))
                          }),
                          fg: color,
                        })
                        // Percentage
                        text({
                          content: derived(() => {
                            const current = marketShare.value.find(x => x.label === d.label)
                            const currentTotal = marketShare.value.reduce((sum, x) => sum + x.value, 0)
                            return `${Math.round(((current?.value ?? 0) / currentTotal) * 100)}%`
                          }),
                          fg: colors.textPrimary,
                        })
                      },
                    })
                  }
                },
              })
            },
          })

          // Right column
          box({
            width: '50%',
            flexDirection: 'column',
            children: () => {
              // Vertical Bar Chart
              ChartPanel({
                title: '  Vertical Bar - Monthly Sales',
                height: 14,
                children: () => {
                  const data = salesData.value
                  const maxVal = Math.max(...data.map(d => d.value))
                  const chartHeight = 8

                  // Y-axis labels + chart rows
                  for (let row = 0; row < chartHeight; row++) {
                    const yValue = Math.round(maxVal * (1 - row / chartHeight))

                    box({
                      flexDirection: 'row',
                      children: () => {
                        // Y-axis label
                        text({
                          content: row === 0 ? `${yValue}`.padStart(3) :
                                   row === chartHeight - 1 ? '  0' : '   ',
                          fg: colors.textMuted,
                        })
                        text({ content: '│', fg: colors.borderDim })

                        // Bar columns
                        text({
                          content: derived(() => {
                            const currentData = salesData.value
                            const currentMax = Math.max(...currentData.map(d => d.value))
                            return verticalBarRow(currentData, currentMax, row, chartHeight)
                          }),
                          fg: colors.chartColors[0]!,
                        })
                      },
                    })
                  }

                  // X-axis
                  box({
                    flexDirection: 'row',
                    children: () => {
                      text({ content: '   └', fg: colors.borderDim })
                      text({
                        content: '─'.repeat(data.length * 3 - 1),
                        fg: colors.borderDim,
                      })
                    },
                  })

                  // X-axis labels
                  box({
                    flexDirection: 'row',
                    children: () => {
                      text({ content: '    ', fg: colors.textMuted })
                      text({
                        content: data.map(d => d.label).join(' '),
                        fg: colors.textMuted,
                      })
                    },
                  })
                },
              })

              // Multi-line Sparkline
              ChartPanel({
                title: '  Line Chart - Detailed View',
                height: 7,
                children: () => {
                  // Create a more detailed line using multiple rows
                  const data = tempHistory.value
                  const min = Math.min(...data)
                  const max = Math.max(...data)

                  // Scale label
                  box({
                    flexDirection: 'row',
                    gap: 1,
                    children: () => {
                      text({ content: `${max.toFixed(0)}°`, fg: colors.textMuted })
                    },
                  })

                  // Chart area
                  box({
                    children: () => {
                      text({
                        content: derived(() => {
                          const d = tempHistory.value
                          const min = Math.min(...d)
                          const max = Math.max(...d)
                          const range = max - min || 1
                          const chars = '▁▂▃▄▅▆▇█'

                          return d.map(v => {
                            const normalized = (v - min) / range
                            const index = Math.min(7, Math.floor(normalized * 8))
                            return chars[index]
                          }).join('')
                        }),
                        fg: colors.chartColors[2]!,
                      })
                    },
                  })

                  // Min label
                  box({
                    flexDirection: 'row',
                    gap: 1,
                    children: () => {
                      text({ content: `${min.toFixed(0)}°`, fg: colors.textMuted })
                    },
                  })

                  // Time axis
                  text({
                    content: '└─────────────────────────┘',
                    fg: colors.borderDim,
                  })
                  text({
                    content: '  0h                    24h',
                    fg: colors.textMuted,
                  })
                },
              })

              // Stats panel
              ChartPanel({
                title: '  Summary Stats',
                height: 5,
                children: () => {
                  box({
                    flexDirection: 'row',
                    gap: 2,
                    children: () => {
                      text({
                        content: cycle(['📊', '📈', '📉', '📊'], { fps: 1 }),
                        fg: colors.textPrimary,
                      })
                      text({
                        content: derived(() => {
                          const langs = languageData.value
                          const sales = salesData.value
                          const avgLang = langs.reduce((s, d) => s + d.value, 0) / langs.length
                          const avgSales = sales.reduce((s, d) => s + d.value, 0) / sales.length
                          return `Avg Lang: ${avgLang.toFixed(1)}%  Avg Sales: ${avgSales.toFixed(1)}`
                        }),
                        fg: colors.textSecondary,
                      })
                    },
                  })
                  text({
                    content: 'Data updates every 2 seconds',
                    fg: colors.textMuted,
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
