/**
 * SparkTUI - Network Status Monitor
 *
 * A professional network monitoring dashboard demonstrating:
 * - Connection status indicator
 * - Ping latency display
 * - Packet loss percentage
 * - Bandwidth usage graph (ASCII sparklines)
 * - Connection history
 * - Simulated network data
 *
 * This is PURELY REACTIVE. No render loops, no polling.
 * Signals update UI automatically through the reactive graph.
 *
 * Run: bun run examples/network-status.ts
 */

import { signal, derived } from '@rlabs-inc/signals'
import { mount } from '../ts/engine/mount'
import { box, text, each, show, cycle, pulse, Frames } from '../ts/primitives'
import { onCleanup } from '../ts/primitives/scope'
import { packColor } from '../ts/bridge/shared-buffer'

// =============================================================================
// TYPES
// =============================================================================

type ConnectionStatus = 'connected' | 'degraded' | 'disconnected'

interface ConnectionEvent {
  id: string
  timestamp: string
  type: 'connect' | 'disconnect' | 'latency_spike' | 'packet_loss' | 'recovered'
  message: string
}

interface Endpoint {
  name: string
  host: string
  status: ConnectionStatus
  latency: number
  packetLoss: number
}

// =============================================================================
// COLORS
// =============================================================================

const colors = {
  // Backgrounds
  bgDark: packColor(18, 18, 24, 255),
  bgCard: packColor(28, 28, 38, 255),
  bgHeader: packColor(35, 35, 50, 255),
  bgGraph: packColor(22, 22, 30, 255),

  // Text
  textPrimary: packColor(240, 240, 250, 255),
  textSecondary: packColor(160, 160, 180, 255),
  textMuted: packColor(100, 100, 120, 255),
  textDim: packColor(70, 70, 90, 255),

  // Status
  connected: packColor(80, 220, 140, 255),
  degraded: packColor(255, 200, 80, 255),
  disconnected: packColor(255, 100, 100, 255),

  // Graph colors
  upload: packColor(80, 200, 255, 255),
  download: packColor(180, 130, 255, 255),
  latency: packColor(255, 180, 100, 255),

  // Borders
  borderDim: packColor(50, 50, 70, 255),
  borderAccent: packColor(80, 140, 200, 255),
}

// =============================================================================
// STATE
// =============================================================================

// Overall connection
const connectionStatus = signal<ConnectionStatus>('connected')
const currentLatency = signal(23)
const packetLoss = signal(0.0)

// Bandwidth
const uploadSpeed = signal(2.4) // MB/s
const downloadSpeed = signal(12.7) // MB/s
const uploadHistory = signal<number[]>([])
const downloadHistory = signal<number[]>([])
const latencyHistory = signal<number[]>([])

// Stats
const totalPacketsSent = signal(0)
const totalPacketsReceived = signal(0)
const packetsLost = signal(0)
const connectionUptime = signal(0)

// Endpoints being monitored
const endpoints = signal<Endpoint[]>([
  { name: 'Gateway', host: '192.168.1.1', status: 'connected', latency: 1, packetLoss: 0 },
  { name: 'DNS Primary', host: '8.8.8.8', status: 'connected', latency: 12, packetLoss: 0 },
  { name: 'DNS Secondary', host: '8.8.4.4', status: 'connected', latency: 15, packetLoss: 0 },
  { name: 'API Server', host: 'api.example.com', status: 'connected', latency: 45, packetLoss: 0 },
  { name: 'CDN', host: 'cdn.example.com', status: 'connected', latency: 23, packetLoss: 0 },
])

// Connection event history
let eventIdCounter = 0
const connectionHistory = signal<ConnectionEvent[]>([])

// =============================================================================
// HELPERS
// =============================================================================

function getTimestamp(): string {
  return new Date().toLocaleTimeString('en-US', { hour12: false })
}

function getStatusColor(status: ConnectionStatus): number {
  switch (status) {
    case 'connected': return colors.connected
    case 'degraded': return colors.degraded
    case 'disconnected': return colors.disconnected
  }
}

function getStatusIcon(status: ConnectionStatus): string {
  switch (status) {
    case 'connected': return '\u25CF'
    case 'degraded': return '\u25D0'
    case 'disconnected': return '\u25CB'
  }
}

function getStatusLabel(status: ConnectionStatus): string {
  switch (status) {
    case 'connected': return 'Connected'
    case 'degraded': return 'Degraded'
    case 'disconnected': return 'Disconnected'
  }
}

function formatSpeed(mbps: number): string {
  if (mbps >= 1) return `${mbps.toFixed(2)} MB/s`
  return `${(mbps * 1024).toFixed(0)} KB/s`
}

function formatLatency(ms: number): string {
  return `${ms}ms`
}

function formatPacketLoss(percent: number): string {
  return `${percent.toFixed(1)}%`
}

function formatUptime(seconds: number): string {
  const hours = Math.floor(seconds / 3600)
  const mins = Math.floor((seconds % 3600) / 60)
  const secs = seconds % 60
  if (hours > 0) return `${hours}h ${mins}m ${secs}s`
  if (mins > 0) return `${mins}m ${secs}s`
  return `${secs}s`
}

function sparkline(history: number[], width: number = 30): string {
  if (history.length === 0) return '\u2581'.repeat(width)
  const chars = '\u2581\u2582\u2583\u2584\u2585\u2586\u2587\u2588'
  const max = Math.max(...history, 1)
  const recent = history.slice(-width)
  const padded = [...Array(width - recent.length).fill(0), ...recent]
  return padded.map(v => chars[Math.min(7, Math.floor((v / max) * 7))]!).join('')
}

function addEvent(type: ConnectionEvent['type'], message: string): void {
  const event: ConnectionEvent = {
    id: `event-${eventIdCounter++}`,
    timestamp: getTimestamp(),
    type,
    message,
  }
  // Keep last 20 events
  connectionHistory.value = [event, ...connectionHistory.value.slice(0, 19)]
}

function getEventColor(type: ConnectionEvent['type']): number {
  switch (type) {
    case 'connect': return colors.connected
    case 'disconnect': return colors.disconnected
    case 'latency_spike': return colors.degraded
    case 'packet_loss': return colors.degraded
    case 'recovered': return colors.connected
  }
}

function getEventIcon(type: ConnectionEvent['type']): string {
  switch (type) {
    case 'connect': return '\u2191'
    case 'disconnect': return '\u2193'
    case 'latency_spike': return '\u26A0'
    case 'packet_loss': return '\u2718'
    case 'recovered': return '\u2714'
  }
}

// =============================================================================
// SIMULATION
// =============================================================================

function startSimulation(): () => void {
  let uptimeCounter = 0
  let packetCounter = { sent: 0, received: 0, lost: 0 }

  // Add initial connect event
  addEvent('connect', 'Network connection established')

  const interval = setInterval(() => {
    uptimeCounter++
    connectionUptime.value = uptimeCounter

    // Update packet counts
    const newSent = Math.floor(Math.random() * 100) + 50
    packetCounter.sent += newSent
    const lossRate = packetLoss.value / 100
    const actualReceived = Math.floor(newSent * (1 - lossRate))
    packetCounter.received += actualReceived
    packetCounter.lost += (newSent - actualReceived)

    totalPacketsSent.value = packetCounter.sent
    totalPacketsReceived.value = packetCounter.received
    packetsLost.value = packetCounter.lost

    // Fluctuate latency
    const newLatency = Math.max(5, Math.min(500, currentLatency.value + (Math.random() - 0.5) * 20))
    currentLatency.value = Math.round(newLatency)
    latencyHistory.value = [...latencyHistory.value.slice(-29), newLatency]

    // Fluctuate bandwidth
    const newUp = Math.max(0.1, uploadSpeed.value + (Math.random() - 0.5) * 1)
    const newDown = Math.max(0.5, downloadSpeed.value + (Math.random() - 0.5) * 3)
    uploadSpeed.value = newUp
    downloadSpeed.value = newDown
    uploadHistory.value = [...uploadHistory.value.slice(-29), newUp]
    downloadHistory.value = [...downloadHistory.value.slice(-29), newDown]

    // Random packet loss (mostly 0, occasionally spikes)
    if (Math.random() < 0.05) {
      const newLoss = Math.random() * 5
      packetLoss.value = newLoss
      if (newLoss > 2) {
        addEvent('packet_loss', `Packet loss spike: ${newLoss.toFixed(1)}%`)
      }
    } else {
      packetLoss.value = Math.max(0, packetLoss.value - 0.1)
    }

    // Update overall status based on latency and packet loss
    if (currentLatency.value > 200 || packetLoss.value > 5) {
      if (connectionStatus.value !== 'degraded') {
        connectionStatus.value = 'degraded'
        addEvent('latency_spike', `Network degraded: ${currentLatency.value}ms latency`)
      }
    } else if (currentLatency.value > 500 || packetLoss.value > 20) {
      if (connectionStatus.value !== 'disconnected') {
        connectionStatus.value = 'disconnected'
        addEvent('disconnect', 'Network connection lost')
      }
    } else {
      if (connectionStatus.value !== 'connected') {
        const wasDisconnected = connectionStatus.value === 'disconnected'
        connectionStatus.value = 'connected'
        addEvent('recovered', wasDisconnected ? 'Connection restored' : 'Network stabilized')
      }
    }

    // Update endpoints
    endpoints.value = endpoints.value.map(ep => ({
      ...ep,
      latency: Math.max(1, Math.min(300, ep.latency + (Math.random() - 0.5) * 10)),
      packetLoss: Math.random() < 0.1 ? Math.random() * 3 : 0,
      status: (ep.latency > 100 || ep.packetLoss > 2) ? 'degraded' :
        (ep.latency > 200 || ep.packetLoss > 10) ? 'disconnected' : 'connected',
    }))
  }, 1000)

  return () => clearInterval(interval)
}

// =============================================================================
// COMPONENTS
// =============================================================================

function StatusCard() {
  box({
    width: '100%',
    border: 1,
    borderColor: derived(() => {
      const status = connectionStatus.value
      return status === 'connected' ? colors.connected :
        status === 'degraded' ? colors.degraded : colors.disconnected
    }),
    bg: colors.bgCard,
    padding: 1,
    flexDirection: 'row',
    justifyContent: 'space-around',
    alignItems: 'center',
    children: () => {
      // Status indicator
      box({
        flexDirection: 'column',
        alignItems: 'center',
        children: () => {
          const statusPulse = pulse({ fps: 2 })
          text({
            content: derived(() =>
              connectionStatus.value === 'connected' ?
                (statusPulse.value ? '\u25CF' : '\u25CB') :
                getStatusIcon(connectionStatus.value)
            ),
            fg: derived(() => getStatusColor(connectionStatus.value)),
          })
          text({
            content: derived(() => getStatusLabel(connectionStatus.value)),
            fg: derived(() => getStatusColor(connectionStatus.value)),
          })
        },
      })

      // Latency
      box({
        flexDirection: 'column',
        alignItems: 'center',
        children: () => {
          text({ content: 'Latency', fg: colors.textSecondary })
          text({
            content: derived(() => formatLatency(currentLatency.value)),
            fg: derived(() =>
              currentLatency.value > 100 ? colors.degraded :
                currentLatency.value > 200 ? colors.disconnected : colors.connected
            ),
          })
        },
      })

      // Packet Loss
      box({
        flexDirection: 'column',
        alignItems: 'center',
        children: () => {
          text({ content: 'Packet Loss', fg: colors.textSecondary })
          text({
            content: derived(() => formatPacketLoss(packetLoss.value)),
            fg: derived(() =>
              packetLoss.value > 5 ? colors.disconnected :
                packetLoss.value > 1 ? colors.degraded : colors.connected
            ),
          })
        },
      })

      // Uptime
      box({
        flexDirection: 'column',
        alignItems: 'center',
        children: () => {
          text({ content: 'Uptime', fg: colors.textSecondary })
          text({
            content: derived(() => formatUptime(connectionUptime.value)),
            fg: colors.textPrimary,
          })
        },
      })
    },
  })
}

function BandwidthGraph() {
  box({
    width: '100%',
    border: 1,
    borderColor: colors.borderDim,
    bg: colors.bgCard,
    padding: 1,
    flexDirection: 'column',
    children: () => {
      text({ content: 'Bandwidth', fg: colors.textSecondary })

      // Upload
      box({
        flexDirection: 'row',
        gap: 2,
        marginTop: 1,
        children: () => {
          text({ content: '\u2191 Up  ', fg: colors.upload })
          text({
            content: derived(() => formatSpeed(uploadSpeed.value).padEnd(12)),
            fg: colors.textPrimary,
          })
          text({
            content: derived(() => sparkline(uploadHistory.value)),
            fg: colors.upload,
          })
        },
      })

      // Download
      box({
        flexDirection: 'row',
        gap: 2,
        marginTop: 1,
        children: () => {
          text({ content: '\u2193 Down', fg: colors.download })
          text({
            content: derived(() => formatSpeed(downloadSpeed.value).padEnd(12)),
            fg: colors.textPrimary,
          })
          text({
            content: derived(() => sparkline(downloadHistory.value)),
            fg: colors.download,
          })
        },
      })

      // Latency graph
      box({
        flexDirection: 'row',
        gap: 2,
        marginTop: 1,
        children: () => {
          text({ content: '\u2248 Ping', fg: colors.latency })
          text({
            content: derived(() => formatLatency(currentLatency.value).padEnd(12)),
            fg: colors.textPrimary,
          })
          text({
            content: derived(() => sparkline(latencyHistory.value)),
            fg: colors.latency,
          })
        },
      })
    },
  })
}

function EndpointList() {
  box({
    width: '100%',
    border: 1,
    borderColor: colors.borderDim,
    bg: colors.bgCard,
    padding: 1,
    flexDirection: 'column',
    children: () => {
      text({ content: 'Monitored Endpoints', fg: colors.textSecondary })

      each(
        () => endpoints.value,
        (getEndpoint, key) => {
          box({
            flexDirection: 'row',
            justifyContent: 'space-between',
            marginTop: 1,
            children: () => {
              // Status icon and name
              box({
                flexDirection: 'row',
                gap: 1,
                width: 20,
                children: () => {
                  text({
                    content: () => getStatusIcon(getEndpoint().status),
                    fg: () => getStatusColor(getEndpoint().status),
                  })
                  text({
                    content: () => getEndpoint().name,
                    fg: colors.textPrimary,
                  })
                },
              })
              // Host
              text({
                content: () => getEndpoint().host,
                fg: colors.textMuted,
                width: 20,
              })
              // Latency
              text({
                content: () => `${Math.round(getEndpoint().latency)}ms`,
                fg: () =>
                  getEndpoint().latency > 100 ? colors.degraded :
                    getEndpoint().latency > 200 ? colors.disconnected : colors.textSecondary,
                width: 8,
                align: 'right',
              })
            },
          })
          return () => {}
        },
        { key: ep => ep.name }
      )
    },
  })
}

function EventHistory() {
  box({
    width: '100%',
    grow: 1,
    overflow: 'scroll',
    border: 1,
    borderColor: colors.borderDim,
    bg: colors.bgGraph,
    padding: 1,
    flexDirection: 'column',
    focusable: true,
    children: () => {
      text({ content: 'Connection History', fg: colors.textSecondary })

      each(
        () => connectionHistory.value,
        (getEvent, key) => {
          box({
            flexDirection: 'row',
            gap: 2,
            marginTop: 1,
            children: () => {
              text({
                content: () => getEvent().timestamp,
                fg: colors.textMuted,
              })
              text({
                content: () => getEventIcon(getEvent().type),
                fg: () => getEventColor(getEvent().type),
              })
              text({
                content: () => getEvent().message,
                fg: colors.textPrimary,
              })
            },
          })
          return () => {}
        },
        { key: ev => ev.id }
      )

      show(
        () => connectionHistory.value.length === 0,
        () => {
          text({
            content: 'No events recorded yet...',
            fg: colors.textMuted,
            marginTop: 1,
          })
          return () => {}
        }
      )
    },
  })
}

function PacketStats() {
  box({
    width: '100%',
    border: 1,
    borderColor: colors.borderDim,
    bg: colors.bgCard,
    padding: 1,
    flexDirection: 'row',
    justifyContent: 'space-around',
    children: () => {
      box({
        flexDirection: 'column',
        alignItems: 'center',
        children: () => {
          text({ content: 'Sent', fg: colors.textMuted })
          text({
            content: derived(() => totalPacketsSent.value.toLocaleString()),
            fg: colors.upload,
          })
        },
      })
      box({
        flexDirection: 'column',
        alignItems: 'center',
        children: () => {
          text({ content: 'Received', fg: colors.textMuted })
          text({
            content: derived(() => totalPacketsReceived.value.toLocaleString()),
            fg: colors.download,
          })
        },
      })
      box({
        flexDirection: 'column',
        alignItems: 'center',
        children: () => {
          text({ content: 'Lost', fg: colors.textMuted })
          text({
            content: derived(() => packetsLost.value.toLocaleString()),
            fg: derived(() => packetsLost.value > 0 ? colors.disconnected : colors.textSecondary),
          })
        },
      })
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
          box({
            flexDirection: 'row',
            gap: 2,
            alignItems: 'center',
            children: () => {
              text({
                content: cycle(Frames.dots, { fps: 10 }),
                fg: colors.upload,
              })
              text({ content: 'Network Status Monitor', fg: colors.upload })
            },
          })

          text({
            content: derived(() => new Date().toLocaleTimeString()),
            fg: colors.textSecondary,
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
          // LEFT COLUMN
          box({
            flexDirection: 'column',
            gap: 1,
            width: '50%',
            children: () => {
              StatusCard()
              BandwidthGraph()
              PacketStats()
            },
          })

          // RIGHT COLUMN
          box({
            flexDirection: 'column',
            gap: 1,
            grow: 1,
            children: () => {
              EndpointList()
              EventHistory()
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
            content: 'SparkTUI Network Monitor - Real-time network diagnostics',
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

console.log('[network-status] App mounted')

// Keep process alive
await new Promise(() => {})
