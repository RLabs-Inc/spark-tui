/**
 * SparkTUI - Stopwatch
 *
 * A precision stopwatch featuring:
 * - Start/Stop/Reset controls
 * - Lap times tracking
 * - Millisecond precision display
 * - Keyboard shortcuts
 * - Theme cycling
 *
 * Controls:
 *   Space  Start/Stop
 *   r      Reset
 *   l      Record lap
 *   c      Clear laps
 *   t      Cycle theme
 *   q      Quit
 *   Ctrl+C Quit
 *
 * Run: bun run examples/stopwatch.ts
 */

import { signal, derived } from '@rlabs-inc/signals'
import { mount } from '../ts/engine'
import { box, text, cycle, Frames } from '../ts/primitives'
import { t, themes, setTheme, getThemeNames } from '../ts/state/theme'
import { getChar, isSpace, isEnter } from '../ts/engine/events'
import { onCleanup } from '../ts/primitives/scope'

// =============================================================================
// STATE
// =============================================================================

const isRunning = signal(false)
const elapsedMs = signal(0)
const laps = signal<number[]>([])
const themeNames = getThemeNames()
const themeIndex = signal(0)
const currentThemeName = derived(() => themeNames[themeIndex.value])

// =============================================================================
// DERIVED VALUES
// =============================================================================

const hours = derived(() => Math.floor(elapsedMs.value / 3600000))
const minutes = derived(() => Math.floor((elapsedMs.value % 3600000) / 60000))
const seconds = derived(() => Math.floor((elapsedMs.value % 60000) / 1000))
const centiseconds = derived(() => Math.floor((elapsedMs.value % 1000) / 10))

const timeDisplay = derived(() => {
  const h = String(hours.value).padStart(2, '0')
  const m = String(minutes.value).padStart(2, '0')
  const s = String(seconds.value).padStart(2, '0')
  const cs = String(centiseconds.value).padStart(2, '0')
  return `${h}:${m}:${s}.${cs}`
})

const statusText = derived(() => isRunning.value ? 'RUNNING' : 'STOPPED')

// =============================================================================
// HELPERS
// =============================================================================

function cycleTheme() {
  themeIndex.value = (themeIndex.value + 1) % themeNames.length
  setTheme(themeNames[themeIndex.value] as keyof typeof themes)
}

function formatLapTime(ms: number): string {
  const h = Math.floor(ms / 3600000)
  const m = Math.floor((ms % 3600000) / 60000)
  const s = Math.floor((ms % 60000) / 1000)
  const cs = Math.floor((ms % 1000) / 10)
  return `${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}.${String(cs).padStart(2, '0')}`
}

function toggleRunning() {
  isRunning.value = !isRunning.value
}

function reset() {
  isRunning.value = false
  elapsedMs.value = 0
}

function recordLap() {
  if (isRunning.value && elapsedMs.value > 0) {
    laps.value = [...laps.value, elapsedMs.value]
  }
}

function clearLaps() {
  laps.value = []
}

// =============================================================================
// APP
// =============================================================================

await mount(() => {
  // Timer update interval
  let interval: ReturnType<typeof setInterval> | null = null
  let lastTick = Date.now()

  const startTimer = () => {
    if (interval) return
    lastTick = Date.now()
    interval = setInterval(() => {
      const now = Date.now()
      elapsedMs.value += now - lastTick
      lastTick = now
    }, 10) // Update every 10ms for smooth display
  }

  const stopTimer = () => {
    if (interval) {
      clearInterval(interval)
      interval = null
    }
  }

  // React to running state changes
  const checkRunning = () => {
    if (isRunning.value) {
      startTimer()
    } else {
      stopTimer()
    }
  }

  // Initial check
  checkRunning()

  // Watch for changes using derived
  const runningWatcher = derived(() => {
    checkRunning()
    return isRunning.value
  })
  // Force evaluation
  runningWatcher.value

  onCleanup(() => {
    stopTimer()
  })

  // Spinner for running state
  const spinner = cycle(Frames.spinner, { fps: 12, active: isRunning })

  // Root container
  box({
    width: '100%',
    height: '100%',
    flexDirection: 'column',
    justifyContent: 'center',
    alignItems: 'center',
    children: () => {
      // Main card
      box({
        width: 50,
        flexDirection: 'column',
        alignItems: 'center',
        border: 3,
        borderColor: t.primary,
        padding: 2,
        gap: 1,
        children: () => {
          // Title with status
          box({
            flexDirection: 'row',
            gap: 2,
            alignItems: 'center',
            children: () => {
              text({ content: 'Stopwatch', fg: t.primary })
              text({
                content: derived(() => isRunning.value ? spinner.value : ' '),
                fg: t.success,
              })
              text({
                content: statusText,
                fg: derived(() => isRunning.value ? t.success.value : t.warning.value),
              })
            },
          })

          // Large time display
          box({
            width: '100%',
            padding: 2,
            border: 1,
            borderColor: t.textMuted,
            justifyContent: 'center',
            alignItems: 'center',
            children: () => {
              text({
                content: timeDisplay,
                fg: t.textBright,
              })
            },
          })

          // Control buttons row
          box({
            width: '100%',
            flexDirection: 'row',
            justifyContent: 'space-around',
            marginTop: 1,
            children: () => {
              // Start/Stop button
              box({
                width: 12,
                height: 3,
                justifyContent: 'center',
                alignItems: 'center',
                border: 1,
                borderColor: derived(() => isRunning.value ? t.warning.value : t.success.value),
                focusable: true,
                onClick: toggleRunning,
                onKey: (key) => {
                  if (isEnter(key) || isSpace(key)) {
                    toggleRunning()
                    return true
                  }
                },
                children: () => {
                  text({
                    content: derived(() => isRunning.value ? '  Stop  ' : ' Start  '),
                    fg: derived(() => isRunning.value ? t.warning.value : t.success.value),
                  })
                },
              })

              // Lap button
              box({
                width: 12,
                height: 3,
                justifyContent: 'center',
                alignItems: 'center',
                border: 1,
                borderColor: t.info,
                focusable: true,
                onClick: recordLap,
                onKey: (key) => {
                  if (isEnter(key) || isSpace(key)) {
                    recordLap()
                    return true
                  }
                },
                children: () => {
                  text({ content: '  Lap   ', fg: t.info })
                },
              })

              // Reset button
              box({
                width: 12,
                height: 3,
                justifyContent: 'center',
                alignItems: 'center',
                border: 1,
                borderColor: t.error,
                focusable: true,
                onClick: reset,
                onKey: (key) => {
                  if (isEnter(key) || isSpace(key)) {
                    reset()
                    return true
                  }
                },
                children: () => {
                  text({ content: ' Reset  ', fg: t.error })
                },
              })
            },
          })

          // Lap times section
          box({
            width: '100%',
            flexDirection: 'column',
            marginTop: 1,
            children: () => {
              box({
                flexDirection: 'row',
                justifyContent: 'space-between',
                width: '100%',
                children: () => {
                  text({ content: 'Lap Times', fg: t.textMuted })
                  text({
                    content: derived(() => laps.value.length > 0 ? `[c] clear (${laps.value.length})` : ''),
                    fg: t.textDim,
                  })
                },
              })

              // Lap list (show last 5)
              box({
                width: '100%',
                height: 6,
                flexDirection: 'column',
                overflow: 'hidden',
                border: 1,
                borderColor: t.textDim,
                padding: 1,
                marginTop: 1,
                children: () => {
                  const lapList = laps.value
                  if (lapList.length === 0) {
                    text({ content: 'No laps recorded', fg: t.textDim })
                  } else {
                    // Show last 4 laps (most recent first)
                    const recentLaps = lapList.slice(-4).reverse()
                    for (let i = 0; i < recentLaps.length; i++) {
                      const lapNum = lapList.length - i
                      const lapTime = recentLaps[i]!
                      // Calculate split time (difference from previous lap)
                      const prevLap = i < lapList.length - 1 ? lapList[lapList.length - i - 2]! : 0
                      const split = lapTime - prevLap

                      box({
                        flexDirection: 'row',
                        justifyContent: 'space-between',
                        width: '100%',
                        children: () => {
                          text({
                            content: `Lap ${String(lapNum).padStart(2)}`,
                            fg: t.textMuted,
                          })
                          text({
                            content: formatLapTime(lapTime),
                            fg: t.textBright,
                          })
                          text({
                            content: `+${formatLapTime(split)}`,
                            fg: t.success,
                          })
                        },
                      })
                    }
                  }
                },
              })
            },
          })

          // Help text
          text({
            content: 'Space: start/stop | r: reset | l: lap | c: clear',
            fg: t.textMuted,
          })
          text({
            content: derived(() => `t: theme (${currentThemeName.value}) | q: quit`),
            fg: t.textDim,
          })
        },
      })
    },

    // Global keyboard handler
    onKey: (key) => {
      const ch = getChar(key)

      if (isSpace(key)) {
        toggleRunning()
        return true
      }

      if (ch === 'r' || ch === 'R') {
        reset()
        return true
      }

      if (ch === 'l' || ch === 'L') {
        recordLap()
        return true
      }

      if (ch === 'c' || ch === 'C') {
        clearLaps()
        return true
      }

      if (ch === 't' || ch === 'T') {
        cycleTheme()
        return true
      }

      if (ch === 'q' || ch === 'Q') {
        process.exit(0)
      }
    },
  })
})
