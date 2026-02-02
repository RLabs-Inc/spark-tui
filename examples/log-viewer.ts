/**
 * SparkTUI - Live Log Viewer
 *
 * A professional log viewer demonstrating:
 * - Scrollable log area with auto-scroll to bottom
 * - Log levels with semantic colors (INFO=blue, WARN=yellow, ERROR=red)
 * - Filter by log level
 * - Clear logs button
 * - Timestamp prefix on all entries
 * - Simulated log generation
 *
 * This is PURELY REACTIVE. No render loops, no polling.
 * Signals update UI automatically through the reactive graph.
 *
 * Run: bun run examples/log-viewer.ts
 */

import { signal, derived } from '@rlabs-inc/signals'
import { mount } from '../ts/engine/mount'
import { box, text, each, show, cycle, Frames } from '../ts/primitives'
import { onCleanup } from '../ts/primitives/scope'
import { packColor } from '../ts/bridge/shared-buffer'

// =============================================================================
// TYPES
// =============================================================================

type LogLevel = 'INFO' | 'WARN' | 'ERROR' | 'DEBUG'

interface LogEntry {
  id: string
  timestamp: string
  level: LogLevel
  message: string
}

// =============================================================================
// COLORS
// =============================================================================

const colors = {
  // Backgrounds
  bgDark: packColor(18, 18, 24, 255),
  bgCard: packColor(28, 28, 38, 255),
  bgHeader: packColor(35, 35, 50, 255),
  bgLogArea: packColor(22, 22, 30, 255),
  bgButton: packColor(50, 50, 70, 255),
  bgButtonHover: packColor(70, 70, 90, 255),
  bgFilterActive: packColor(60, 100, 140, 255),

  // Text
  textPrimary: packColor(240, 240, 250, 255),
  textSecondary: packColor(160, 160, 180, 255),
  textMuted: packColor(100, 100, 120, 255),
  textDim: packColor(70, 70, 90, 255),

  // Log levels
  info: packColor(100, 180, 255, 255),
  warn: packColor(255, 200, 100, 255),
  error: packColor(255, 100, 100, 255),
  debug: packColor(160, 160, 180, 255),

  // Borders
  borderDim: packColor(50, 50, 70, 255),
  borderAccent: packColor(80, 140, 200, 255),
}

// =============================================================================
// STATE
// =============================================================================

let logIdCounter = 0
const logs = signal<LogEntry[]>([])
const filter = signal<LogLevel | 'ALL'>('ALL')
const autoScroll = signal(true)
const logCount = signal(0)
const isPaused = signal(false)

// Filtered logs derived from logs and filter
const filteredLogs = derived(() => {
  const allLogs = logs.value
  const currentFilter = filter.value
  if (currentFilter === 'ALL') return allLogs
  return allLogs.filter(log => log.level === currentFilter)
})

// =============================================================================
// HELPERS
// =============================================================================

function getTimestamp(): string {
  const now = new Date()
  return now.toLocaleTimeString('en-US', { hour12: false }) +
    '.' + String(now.getMilliseconds()).padStart(3, '0')
}

function getLevelColor(level: LogLevel): number {
  switch (level) {
    case 'INFO': return colors.info
    case 'WARN': return colors.warn
    case 'ERROR': return colors.error
    case 'DEBUG': return colors.debug
  }
}

function getLevelBadge(level: LogLevel): string {
  switch (level) {
    case 'INFO': return '[INFO ]'
    case 'WARN': return '[WARN ]'
    case 'ERROR': return '[ERROR]'
    case 'DEBUG': return '[DEBUG]'
  }
}

function addLog(level: LogLevel, message: string): void {
  if (isPaused.value) return

  const entry: LogEntry = {
    id: `log-${logIdCounter++}`,
    timestamp: getTimestamp(),
    level,
    message,
  }

  // Keep last 500 logs
  const current = logs.value
  if (current.length >= 500) {
    logs.value = [...current.slice(-499), entry]
  } else {
    logs.value = [...current, entry]
  }
  logCount.value = logs.value.length
}

function clearLogs(): void {
  logs.value = []
  logCount.value = 0
  logIdCounter = 0
}

// =============================================================================
// LOG SIMULATION
// =============================================================================

const infoMessages = [
  'User authenticated successfully',
  'Database connection established',
  'Cache refreshed with 1,234 entries',
  'API request processed in 23ms',
  'Session created for user_abc123',
  'Background job completed: email_queue',
  'Webhook delivered to https://example.com',
  'Configuration reloaded',
  'Metrics exported to monitoring service',
  'Health check passed',
]

const warnMessages = [
  'Response time exceeded threshold (>200ms)',
  'Memory usage at 75% - consider scaling',
  'Rate limit approaching for IP 192.168.1.1',
  'Deprecated API endpoint accessed',
  'Connection pool running low',
  'Retry attempt 2/3 for external service',
  'Cache miss rate above 10%',
  'Slow query detected (>100ms)',
]

const errorMessages = [
  'Failed to connect to database: timeout',
  'Authentication failed for user xyz',
  'Request validation failed: missing required field',
  'External service returned 503',
  'Unhandled exception in worker thread',
  'Permission denied for resource /admin',
  'Rate limit exceeded for client app_123',
  'Memory allocation failed',
]

const debugMessages = [
  'Entering function processRequest()',
  'Variable state: { count: 42, status: "pending" }',
  'Query executed: SELECT * FROM users WHERE id = 1',
  'Event emitted: user.created',
  'Middleware chain: auth -> validate -> handler',
  'Cache key generated: user:profile:123',
]

function startLogSimulation(): () => void {
  // Generate logs at varying intervals
  const intervals: ReturnType<typeof setInterval>[] = []

  // Regular info logs (every 500-1500ms)
  intervals.push(setInterval(() => {
    const msg = infoMessages[Math.floor(Math.random() * infoMessages.length)]!
    addLog('INFO', msg)
  }, 500 + Math.random() * 1000))

  // Warning logs (every 2-5 seconds)
  intervals.push(setInterval(() => {
    const msg = warnMessages[Math.floor(Math.random() * warnMessages.length)]!
    addLog('WARN', msg)
  }, 2000 + Math.random() * 3000))

  // Error logs (every 4-8 seconds)
  intervals.push(setInterval(() => {
    const msg = errorMessages[Math.floor(Math.random() * errorMessages.length)]!
    addLog('ERROR', msg)
  }, 4000 + Math.random() * 4000))

  // Debug logs (every 1-2 seconds)
  intervals.push(setInterval(() => {
    const msg = debugMessages[Math.floor(Math.random() * debugMessages.length)]!
    addLog('DEBUG', msg)
  }, 1000 + Math.random() * 1000))

  return () => {
    for (const interval of intervals) {
      clearInterval(interval)
    }
  }
}

// =============================================================================
// COMPONENTS
// =============================================================================

function FilterButton(props: { level: LogLevel | 'ALL'; label: string }) {
  box({
    border: 1,
    borderColor: derived(() =>
      filter.value === props.level ? colors.borderAccent : colors.borderDim
    ),
    bg: derived(() =>
      filter.value === props.level ? colors.bgFilterActive : colors.bgButton
    ),
    paddingLeft: 1,
    paddingRight: 1,
    focusable: true,
    onClick: () => {
      filter.value = props.level
    },
    children: () => {
      text({
        content: props.label,
        fg: derived(() =>
          filter.value === props.level ? colors.textPrimary : colors.textSecondary
        ),
      })
    },
  })
}

function LogLine(getEntry: () => LogEntry) {
  box({
    flexDirection: 'row',
    gap: 1,
    width: '100%',
    children: () => {
      // Timestamp
      text({
        content: () => getEntry().timestamp,
        fg: colors.textMuted,
      })
      // Level badge
      text({
        content: () => getLevelBadge(getEntry().level),
        fg: () => getLevelColor(getEntry().level),
      })
      // Message
      text({
        content: () => getEntry().message,
        fg: colors.textPrimary,
        grow: 1,
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
  // Start log simulation
  const stopSimulation = startLogSimulation()
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
          // Title with spinner
          box({
            flexDirection: 'row',
            gap: 2,
            alignItems: 'center',
            children: () => {
              text({
                content: cycle(Frames.dots, { fps: 10 }),
                fg: colors.info,
              })
              text({ content: 'Log Viewer', fg: colors.textPrimary })
              text({
                content: derived(() => `(${logCount.value} entries)`),
                fg: colors.textMuted,
              })
            },
          })

          // Controls
          box({
            flexDirection: 'row',
            gap: 2,
            alignItems: 'center',
            children: () => {
              // Pause/Resume button
              box({
                border: 1,
                borderColor: derived(() =>
                  isPaused.value ? colors.warn : colors.borderDim
                ),
                bg: colors.bgButton,
                paddingLeft: 1,
                paddingRight: 1,
                focusable: true,
                onClick: () => {
                  isPaused.value = !isPaused.value
                },
                children: () => {
                  text({
                    content: derived(() => isPaused.value ? 'Resume' : 'Pause'),
                    fg: derived(() =>
                      isPaused.value ? colors.warn : colors.textSecondary
                    ),
                  })
                },
              })

              // Auto-scroll toggle
              box({
                border: 1,
                borderColor: derived(() =>
                  autoScroll.value ? colors.borderAccent : colors.borderDim
                ),
                bg: derived(() =>
                  autoScroll.value ? colors.bgFilterActive : colors.bgButton
                ),
                paddingLeft: 1,
                paddingRight: 1,
                focusable: true,
                onClick: () => {
                  autoScroll.value = !autoScroll.value
                },
                children: () => {
                  text({
                    content: derived(() =>
                      autoScroll.value ? 'Auto-scroll: ON' : 'Auto-scroll: OFF'
                    ),
                    fg: colors.textSecondary,
                  })
                },
              })

              // Clear button
              box({
                border: 1,
                borderColor: colors.error,
                bg: colors.bgButton,
                paddingLeft: 1,
                paddingRight: 1,
                focusable: true,
                onClick: clearLogs,
                children: () => {
                  text({ content: 'Clear', fg: colors.error })
                },
              })
            },
          })
        },
      })

      // ===== FILTER BAR =====
      box({
        width: '100%',
        height: 3,
        flexDirection: 'row',
        alignItems: 'center',
        gap: 2,
        padding: 1,
        bg: colors.bgCard,
        children: () => {
          text({ content: 'Filter:', fg: colors.textSecondary })
          FilterButton({ level: 'ALL', label: 'All' })
          FilterButton({ level: 'INFO', label: 'Info' })
          FilterButton({ level: 'WARN', label: 'Warn' })
          FilterButton({ level: 'ERROR', label: 'Error' })
          FilterButton({ level: 'DEBUG', label: 'Debug' })

          // Spacer
          box({ grow: 1, children: () => {} })

          // Filtered count
          text({
            content: derived(() => {
              const total = logs.value.length
              const filtered = filteredLogs.value.length
              if (filter.value === 'ALL') return `Showing all ${total} entries`
              return `Showing ${filtered} of ${total} entries`
            }),
            fg: colors.textMuted,
          })
        },
      })

      // ===== LOG AREA =====
      box({
        width: '100%',
        grow: 1,
        overflow: 'scroll',
        border: 1,
        borderColor: colors.borderDim,
        bg: colors.bgLogArea,
        padding: 1,
        flexDirection: 'column',
        focusable: true,
        children: () => {
          // Show "no logs" message when empty
          show(
            () => filteredLogs.value.length === 0,
            () => {
              text({
                content: 'No log entries to display. Logs will appear here...',
                fg: colors.textMuted,
              })
              return () => {}
            },
            () => {
              // Log entries using each()
              each(
                () => filteredLogs.value,
                (getEntry, key) => {
                  LogLine(getEntry)
                  return () => {}
                },
                { key: entry => entry.id }
              )
              return () => {}
            }
          )
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
          // Level legend
          box({
            flexDirection: 'row',
            gap: 3,
            children: () => {
              text({ content: 'INFO', fg: colors.info })
              text({ content: 'WARN', fg: colors.warn })
              text({ content: 'ERROR', fg: colors.error })
              text({ content: 'DEBUG', fg: colors.debug })
            },
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

console.log('[log-viewer] App mounted')

// Keep process alive
await new Promise(() => {})
