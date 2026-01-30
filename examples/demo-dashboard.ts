/**
 * SparkTUI - Live Metrics Dashboard Demo
 *
 * A beautiful dashboard showcasing SparkTUI's reactive capabilities:
 * - Live-updating metrics (CPU, Memory, Network)
 * - Animated progress bars
 * - Pulsing status indicators
 * - ASCII sparklines
 * - Live clock
 *
 * This is PURELY REACTIVE. No render loops, no polling.
 * Signals update UI automatically through the reactive graph.
 *
 * Run: bun run examples/demo-dashboard.ts
 */

import { signal, derived, effect, effectScope } from '@rlabs-inc/signals'
import type { WritableSignal } from '@rlabs-inc/signals'
import { mount } from '../ts/engine/mount'
import { box } from '../ts/primitives/box'
import { text } from '../ts/primitives/text'
import { onCleanup } from '../ts/primitives/scope'
import { cycle, pulse, Frames } from '../ts/primitives/animation'

// =============================================================================
// COLORS - Semantic palette using RGBA objects
// =============================================================================

import type { RGBA } from '../ts/types'

/** Helper to create RGBA color */
function rgba(r: number, g: number, b: number, a: number = 255): RGBA {
  return { r, g, b, a }
}

const colors = {
  // Backgrounds
  bgDark: rgba(18, 18, 24),
  bgCard: rgba(28, 28, 38),
  bgCardHover: rgba(35, 35, 48),

  // Text
  textPrimary: rgba(240, 240, 250),
  textSecondary: rgba(160, 160, 180),
  textMuted: rgba(100, 100, 120),
  textDim: rgba(70, 70, 90),

  // Accent colors
  cyan: rgba(80, 200, 255),
  green: rgba(80, 220, 140),
  yellow: rgba(255, 200, 80),
  red: rgba(255, 100, 100),
  orange: rgba(255, 150, 80),
  purple: rgba(180, 130, 255),
  blue: rgba(100, 150, 255),

  // Border
  borderDim: rgba(50, 50, 70),
  borderAccent: rgba(80, 140, 200),
}

// =============================================================================
// SIMULATED METRICS - These would be real WebSocket/API data in production
// =============================================================================

// System metrics
const cpu = signal(45)
const memory = signal(62)
const networkUp = signal(1.2)
const networkDown = signal(3.4)

// Application metrics
const requestsPerSec = signal(1234)
const activeUsers = signal(892)
const errorCount = signal(3)

// History for sparkline
const requestHistory = signal<number[]>([])

// Services status
interface ServiceStatus {
  name: string
  status: 'running' | 'degraded' | 'down'
  uptime: number
  latency: number
}

const services = signal<ServiceStatus[]>([
  { name: 'API Gateway', status: 'running', uptime: 99.9, latency: 23 },
  { name: 'Database', status: 'running', uptime: 99.8, latency: 12 },
  { name: 'Cache', status: 'running', uptime: 100, latency: 2 },
  { name: 'Worker Queue', status: 'degraded', uptime: 95.2, latency: 145 },
])

// Recent events
interface LogEvent {
  time: string
  message: string
}

const recentEvents = signal<LogEvent[]>([
  { time: '12:34:52', message: 'User login: john@example.com' },
  { time: '12:34:48', message: 'API request: GET /users (23ms)' },
  { time: '12:34:45', message: 'Cache hit: session:abc123' },
])

// Live clock
const clock = signal(new Date().toLocaleTimeString())

// =============================================================================
// UTILITY FUNCTIONS
// =============================================================================

/** Create a progress bar string */
function progressBar(value: number, width: number = 10): string {
  const filled = Math.floor((value / 100) * width)
  const empty = width - filled
  return '\u2588'.repeat(filled) + '\u2591'.repeat(empty)
}

/** Format bytes per second */
function formatBps(mbps: number): string {
  return `${mbps.toFixed(1)} MB/s`
}

/** Format number with commas */
function formatNumber(n: number): string {
  return n.toLocaleString()
}

/** Create ASCII sparkline from history array */
function sparkline(history: number[]): string {
  if (history.length === 0) return ''
  const chars = '\u2581\u2582\u2583\u2584\u2585\u2586\u2587\u2588'
  const max = Math.max(...history, 1)
  return history.map(v => chars[Math.min(7, Math.floor((v / max) * 7))]!).join('')
}

/** Get status color based on service status */
function statusColor(status: ServiceStatus['status']): RGBA {
  switch (status) {
    case 'running': return colors.green
    case 'degraded': return colors.yellow
    case 'down': return colors.red
  }
}

/** Get status indicator */
function statusDot(status: ServiceStatus['status'], visible: boolean): string {
  if (status === 'running') return visible ? '\u25CF' : '\u25CB'
  if (status === 'degraded') return visible ? '\u25CF' : '\u25CB'
  return '\u25CF' // always solid for down
}

// =============================================================================
// SIMULATE LIVE DATA
// =============================================================================

function startSimulation(): () => void {
  // Update metrics every second
  const metricsInterval = setInterval(() => {
    // Fluctuate CPU
    cpu.value = Math.min(100, Math.max(0, cpu.value + (Math.random() - 0.5) * 15))

    // Fluctuate memory (slower changes)
    memory.value = Math.min(100, Math.max(0, memory.value + (Math.random() - 0.5) * 5))

    // Network fluctuation
    networkUp.value = Math.max(0.1, networkUp.value + (Math.random() - 0.5) * 0.5)
    networkDown.value = Math.max(0.1, networkDown.value + (Math.random() - 0.5) * 1)

    // Requests fluctuation
    const delta = Math.floor(Math.random() * 200) - 80
    requestsPerSec.value = Math.max(100, requestsPerSec.value + delta)

    // Active users fluctuation
    activeUsers.value = Math.max(10, activeUsers.value + Math.floor(Math.random() * 20) - 8)

    // Occasional error
    if (Math.random() < 0.1) {
      errorCount.value = Math.min(99, errorCount.value + 1)
    }

    // Update history for sparkline
    const history = [...requestHistory.value, requestsPerSec.value].slice(-40)
    requestHistory.value = history
  }, 1000)

  // Update clock every second
  const clockInterval = setInterval(() => {
    clock.value = new Date().toLocaleTimeString()
  }, 1000)

  // Occasional new events
  const eventInterval = setInterval(() => {
    const eventTypes = [
      () => `User login: user${Math.floor(Math.random() * 1000)}@example.com`,
      () => `API request: GET /api/v1/data (${Math.floor(Math.random() * 50)}ms)`,
      () => `Cache hit: session:${Math.random().toString(36).substring(7)}`,
      () => `Webhook received: order.created`,
      () => `Background job completed: email_queue`,
    ]
    const newEvent = {
      time: new Date().toLocaleTimeString().split(' ')[0]!,
      message: eventTypes[Math.floor(Math.random() * eventTypes.length)]!(),
    }
    recentEvents.value = [newEvent, ...recentEvents.value.slice(0, 2)]
  }, 3000)

  // Occasional service status changes
  const serviceInterval = setInterval(() => {
    const svc = [...services.value]
    const idx = Math.floor(Math.random() * svc.length)
    const s = svc[idx]!
    // Mostly stay running, occasional degraded, rare down
    const roll = Math.random()
    if (roll < 0.7) {
      s.status = 'running'
      s.uptime = Math.min(100, s.uptime + 0.1)
      s.latency = Math.max(1, s.latency - Math.floor(Math.random() * 5))
    } else if (roll < 0.95) {
      s.status = 'degraded'
      s.uptime = Math.max(90, s.uptime - 0.5)
      s.latency = s.latency + Math.floor(Math.random() * 30)
    } else {
      s.status = 'down'
      s.uptime = Math.max(80, s.uptime - 2)
    }
    services.value = svc
  }, 5000)

  return () => {
    clearInterval(metricsInterval)
    clearInterval(clockInterval)
    clearInterval(eventInterval)
    clearInterval(serviceInterval)
  }
}

// =============================================================================
// DASHBOARD COMPONENTS
// =============================================================================

/** Dashboard content builder - creates all dashboard UI elements */
function DashboardContent(livePulse: ReturnType<typeof pulse>) {
  // ===== HEADER =====
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
            text({ content: 'SparkTUI Dashboard', fg: colors.cyan })

            // Live indicator + clock
            box({
              flexDirection: 'row',
              gap: 2,
              alignItems: 'center',
              children: () => {
                // Pulsing live dot
                text({
                  content: derived(() => livePulse.value ? '\u25CF' : '\u25CB'),
                  fg: colors.green,
                })
                text({ content: 'Live', fg: colors.green })
                text({ content: clock, fg: colors.textSecondary })
              },
            })
          },
        })

        // ===== METRICS ROW =====
        box({
          width: '100%',
          height: 5,
          flexDirection: 'row',
          gap: 1,
          marginTop: 1,
          children: () => {
            // CPU Card
            MetricCard({
              title: 'CPU Usage',
              value: derived(() => `${Math.round(cpu.value)}%`),
              bar: derived(() => progressBar(cpu.value)),
              color: derived(() => cpu.value > 80 ? colors.red : cpu.value > 60 ? colors.yellow : colors.green),
            })

            // Memory Card
            MetricCard({
              title: 'Memory',
              value: derived(() => `${Math.round(memory.value)}%`),
              bar: derived(() => progressBar(memory.value)),
              color: derived(() => memory.value > 80 ? colors.red : memory.value > 60 ? colors.yellow : colors.green),
            })

            // Network Card
            box({
              grow: 1,
              height: '100%',
              border: 1,
              borderColor: colors.borderDim,
              bg: colors.bgCard,
              padding: 1,
              flexDirection: 'column',
              children: () => {
                text({ content: 'Network', fg: colors.textSecondary })
                text({
                  content: derived(() => `\u2191 ${formatBps(networkUp.value)}`),
                  fg: colors.cyan,
                })
                text({
                  content: derived(() => `\u2193 ${formatBps(networkDown.value)}`),
                  fg: colors.purple,
                })
              },
            })
          },
        })

        // ===== STATS + SPARKLINE ROW =====
        box({
          width: '100%',
          height: 4,
          border: 1,
          borderColor: colors.borderDim,
          bg: colors.bgCard,
          marginTop: 1,
          padding: 1,
          flexDirection: 'column',
          children: () => {
            // Stats row
            box({
              flexDirection: 'row',
              gap: 4,
              children: () => {
                text({
                  content: derived(() => `Requests/sec: ${formatNumber(requestsPerSec.value)}`),
                  fg: colors.textPrimary,
                })
                text({
                  content: derived(() => `Active Users: ${formatNumber(activeUsers.value)}`),
                  fg: colors.textPrimary,
                })
                text({
                  content: derived(() => `Errors: ${errorCount.value}`),
                  fg: derived(() => errorCount.value > 0 ? colors.red : colors.textMuted),
                })
              },
            })
            // Sparkline
            text({
              content: derived(() => sparkline(requestHistory.value)),
              fg: colors.cyan,
            })
          },
        })

        // ===== SERVICES =====
        box({
          width: '100%',
          grow: 1,
          border: 1,
          borderColor: colors.borderDim,
          bg: colors.bgCard,
          marginTop: 1,
          padding: 1,
          flexDirection: 'column',
          children: () => {
            text({ content: 'Services:', fg: colors.textSecondary, paddingBottom: 1 })

            // Service rows - static since we rebuild on change
            ServiceList()
          },
        })

        // ===== RECENT EVENTS =====
        box({
          width: '100%',
          height: 5,
          border: 1,
          borderColor: colors.borderDim,
          bg: colors.bgCard,
          marginTop: 1,
          padding: 1,
          flexDirection: 'column',
          children: () => {
            text({ content: 'Recent Events:', fg: colors.textSecondary })
            EventList()
          },
        })

  // ===== FOOTER =====
  box({
    width: '100%',
    height: 1,
    marginTop: 1,
    justifyContent: 'center',
    children: () => {
      text({
        content: 'Press Ctrl+C to exit',
        fg: colors.textDim,
      })
    },
  })
}

// Metric card component
interface MetricCardProps {
  title: string
  value: { readonly value: string } | (() => string)
  bar: { readonly value: string } | (() => string)
  color: { readonly value: RGBA } | (() => RGBA)
}

function MetricCard({ title, value, bar, color }: MetricCardProps) {
  box({
    grow: 1,
    height: '100%',
    border: 1,
    borderColor: colors.borderDim,
    bg: colors.bgCard,
    padding: 1,
    flexDirection: 'column',
    children: () => {
      text({ content: title, fg: colors.textSecondary })
      text({ content: bar, fg: color })
      text({ content: value, fg: colors.textPrimary })
    },
  })
}

// Service list with reactive updates
function ServiceList() {
  // Since we're using the primitives directly and they handle reactivity,
  // we need to use a derived that builds the text content
  const svc = services.value
  for (let i = 0; i < svc.length; i++) {
    const service = svc[i]!
    const statusPulse = pulse({ fps: 2 })

    box({
      flexDirection: 'row',
      gap: 2,
      height: 1,
      children: () => {
        // Status indicator (pulsing for non-running)
        text({
          content: derived(() =>
            service.status === 'running'
              ? '\u25CF'
              : statusPulse.value ? '\u25CF' : '\u25CB'
          ),
          fg: statusColor(service.status),
        })
        text({
          content: service.name.padEnd(14),
          fg: colors.textPrimary,
        })
        text({
          content: service.status.padEnd(10),
          fg: statusColor(service.status),
        })
        text({
          content: `${service.status === 'down' ? '\u2193' : '\u2191'} ${service.uptime.toFixed(1)}%`,
          fg: service.uptime >= 99 ? colors.green : service.uptime >= 95 ? colors.yellow : colors.red,
        })
        text({
          content: `${service.latency}ms avg`,
          fg: service.latency < 50 ? colors.green : service.latency < 100 ? colors.yellow : colors.red,
        })
      },
    })
  }
}

// Event list
function EventList() {
  const events = recentEvents.value
  for (let i = 0; i < Math.min(3, events.length); i++) {
    const event = events[i]!
    text({
      content: `[${event.time}] ${event.message}`,
      fg: colors.textMuted,
    })
  }
}

// =============================================================================
// MAIN
// =============================================================================

console.log('Initializing SparkTUI Dashboard...\n')

const cols = process.stdout.columns || 80
const rows = process.stdout.rows || 24

const { unmount, setMode, getMode } = mount(() => {
  // Pulsing indicator for "Live" status
  const livePulse = pulse({ fps: 2 })

  // Root container - full terminal, dark background
  box({
    width: cols,
    height: rows,
    flexDirection: 'column',
    bg: colors.bgDark,
    padding: 1,
    children: () => {
      // Call Dashboard content builder
      DashboardContent(livePulse)
    },
  })

  // Start simulation and register cleanup
  const stopSimulation = startSimulation()
  onCleanup(stopSimulation)
}, {
  mode: 'fullscreen',
})

console.log('[demo-dashboard] App mounted')

// Keep process alive - Rust engine handles stdin/Ctrl+C
await new Promise(() => {})
