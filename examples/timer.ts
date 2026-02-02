/**
 * SparkTUI - Countdown Timer
 *
 * A countdown timer featuring:
 * - Input fields for minutes and seconds
 * - Visual progress bar
 * - Color changes as time runs out (green -> yellow -> red)
 * - Bell notification when timer completes
 * - Preset quick-start buttons
 * - Theme cycling
 *
 * Controls:
 *   Space  Start/Pause
 *   r      Reset
 *   1-5    Quick presets (1/2/5/10/15 min)
 *   +      Add 1 minute
 *   -      Subtract 1 minute
 *   t      Cycle theme
 *   q      Quit
 *   Ctrl+C Quit
 *
 * Run: bun run examples/timer.ts
 */

import { signal, derived } from '@rlabs-inc/signals'
import { mount } from '../ts/engine'
import { box, text, input, cycle, pulse, Frames } from '../ts/primitives'
import { t, themes, setTheme, getThemeNames } from '../ts/state/theme'
import { getChar, isSpace, isEnter } from '../ts/engine/events'
import { onCleanup } from '../ts/primitives/scope'

// =============================================================================
// STATE
// =============================================================================

const initialMinutes = signal(5)
const initialSeconds = signal(0)
const remainingMs = signal(5 * 60 * 1000) // Start with 5 minutes
const isRunning = signal(false)
const isComplete = signal(false)
const themeNames = getThemeNames()
const themeIndex = signal(0)
const currentThemeName = derived(() => themeNames[themeIndex.value])

// =============================================================================
// DERIVED VALUES
// =============================================================================

const totalInitialMs = derived(() => (initialMinutes.value * 60 + initialSeconds.value) * 1000)

const minutes = derived(() => Math.floor(remainingMs.value / 60000))
const seconds = derived(() => Math.floor((remainingMs.value % 60000) / 1000))

const timeDisplay = derived(() => {
  const m = String(minutes.value).padStart(2, '0')
  const s = String(seconds.value).padStart(2, '0')
  return `${m}:${s}`
})

const progressPercent = derived(() => {
  const total = totalInitialMs.value
  if (total === 0) return 0
  return Math.max(0, Math.min(100, (remainingMs.value / total) * 100))
})

const progressBarWidth = 40

const progressBar = derived(() => {
  const filled = Math.floor((progressPercent.value / 100) * progressBarWidth)
  const empty = progressBarWidth - filled
  return '\u2588'.repeat(filled) + '\u2591'.repeat(empty)
})

// Color based on remaining percentage
const timerColor = derived(() => {
  const pct = progressPercent.value
  if (pct > 50) return t.success.value
  if (pct > 20) return t.warning.value
  return t.error.value
})

const statusText = derived(() => {
  if (isComplete.value) return 'COMPLETE!'
  if (isRunning.value) return 'RUNNING'
  return 'PAUSED'
})

// =============================================================================
// HELPERS
// =============================================================================

function cycleTheme() {
  themeIndex.value = (themeIndex.value + 1) % themeNames.length
  setTheme(themeNames[themeIndex.value] as keyof typeof themes)
}

function setPreset(mins: number) {
  isRunning.value = false
  isComplete.value = false
  initialMinutes.value = mins
  initialSeconds.value = 0
  remainingMs.value = mins * 60 * 1000
}

function toggleRunning() {
  if (isComplete.value) return
  if (remainingMs.value <= 0) return
  isRunning.value = !isRunning.value
}

function reset() {
  isRunning.value = false
  isComplete.value = false
  remainingMs.value = totalInitialMs.value
}

function addMinute() {
  if (!isRunning.value) {
    initialMinutes.value = Math.min(99, initialMinutes.value + 1)
    remainingMs.value = totalInitialMs.value
  } else {
    remainingMs.value = Math.min(99 * 60 * 1000, remainingMs.value + 60000)
  }
}

function subtractMinute() {
  if (!isRunning.value) {
    initialMinutes.value = Math.max(0, initialMinutes.value - 1)
    remainingMs.value = totalInitialMs.value
  } else {
    remainingMs.value = Math.max(0, remainingMs.value - 60000)
  }
}

function ringBell() {
  // Bell character - terminal will beep
  process.stdout.write('\x07')
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
      const elapsed = now - lastTick
      lastTick = now
      remainingMs.value = Math.max(0, remainingMs.value - elapsed)

      if (remainingMs.value <= 0) {
        isRunning.value = false
        isComplete.value = true
        stopTimer()
        ringBell()
      }
    }, 100)
  }

  const stopTimer = () => {
    if (interval) {
      clearInterval(interval)
      interval = null
    }
  }

  // React to running state
  const checkRunning = () => {
    if (isRunning.value && !isComplete.value) {
      startTimer()
    } else {
      stopTimer()
    }
  }

  // Initial check
  checkRunning()

  // Watch for changes
  const runningWatcher = derived(() => {
    checkRunning()
    return isRunning.value
  })
  runningWatcher.value

  onCleanup(() => stopTimer())

  // Animations
  const spinner = cycle(Frames.spinner, { fps: 12, active: isRunning })
  const completeBlink = pulse({ fps: 2, active: isComplete })

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
        borderColor: derived(() => isComplete.value ? t.success.value : t.primary.value),
        padding: 2,
        gap: 1,
        children: () => {
          // Title with status
          box({
            flexDirection: 'row',
            gap: 2,
            alignItems: 'center',
            children: () => {
              text({ content: 'Countdown Timer', fg: t.primary })
              text({
                content: derived(() => isRunning.value ? spinner.value : ' '),
                fg: timerColor,
              })
              text({
                content: statusText,
                fg: derived(() => {
                  if (isComplete.value) return completeBlink.value ? t.success.value : t.bgMuted.value
                  if (isRunning.value) return timerColor.value
                  return t.warning.value
                }),
              })
            },
          })

          // Large time display
          box({
            width: '100%',
            padding: 2,
            border: 1,
            borderColor: timerColor,
            justifyContent: 'center',
            alignItems: 'center',
            children: () => {
              text({
                content: timeDisplay,
                fg: timerColor,
              })
            },
          })

          // Progress bar
          box({
            width: '100%',
            flexDirection: 'column',
            alignItems: 'center',
            marginTop: 1,
            children: () => {
              text({
                content: progressBar,
                fg: timerColor,
              })
              text({
                content: derived(() => `${Math.round(progressPercent.value)}%`),
                fg: t.textMuted,
              })
            },
          })

          // Preset buttons row
          box({
            width: '100%',
            flexDirection: 'row',
            justifyContent: 'center',
            gap: 1,
            marginTop: 1,
            children: () => {
              const presets = [
                { key: '1', mins: 1, label: '1m' },
                { key: '2', mins: 2, label: '2m' },
                { key: '3', mins: 5, label: '5m' },
                { key: '4', mins: 10, label: '10m' },
                { key: '5', mins: 15, label: '15m' },
              ]

              for (const preset of presets) {
                box({
                  width: 6,
                  height: 3,
                  justifyContent: 'center',
                  alignItems: 'center',
                  border: 1,
                  borderColor: t.textDim,
                  focusable: true,
                  onClick: () => setPreset(preset.mins),
                  onKey: (key) => {
                    if (isEnter(key) || isSpace(key)) {
                      setPreset(preset.mins)
                      return true
                    }
                  },
                  children: () => {
                    text({ content: preset.label, fg: t.textMuted })
                  },
                })
              }
            },
          })

          // Control buttons row
          box({
            width: '100%',
            flexDirection: 'row',
            justifyContent: 'space-around',
            marginTop: 1,
            children: () => {
              // -1 min button
              box({
                width: 8,
                height: 3,
                justifyContent: 'center',
                alignItems: 'center',
                border: 1,
                borderColor: t.warning,
                focusable: true,
                onClick: subtractMinute,
                onKey: (key) => {
                  if (isEnter(key) || isSpace(key)) {
                    subtractMinute()
                    return true
                  }
                },
                children: () => {
                  text({ content: ' -1m ', fg: t.warning })
                },
              })

              // Start/Pause button
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
                    content: derived(() => isRunning.value ? ' Pause  ' : ' Start  '),
                    fg: derived(() => isRunning.value ? t.warning.value : t.success.value),
                  })
                },
              })

              // Reset button
              box({
                width: 10,
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
                  text({ content: ' Reset ', fg: t.error })
                },
              })

              // +1 min button
              box({
                width: 8,
                height: 3,
                justifyContent: 'center',
                alignItems: 'center',
                border: 1,
                borderColor: t.success,
                focusable: true,
                onClick: addMinute,
                onKey: (key) => {
                  if (isEnter(key) || isSpace(key)) {
                    addMinute()
                    return true
                  }
                },
                children: () => {
                  text({ content: ' +1m ', fg: t.success })
                },
              })
            },
          })

          // Help text
          text({
            content: 'Space: start/pause | r: reset | +/-: adjust',
            fg: t.textMuted,
          })
          text({
            content: '1-5: presets | t: theme | q: quit',
            fg: t.textMuted,
          })
          text({
            content: derived(() => `Theme: ${currentThemeName.value}`),
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

      if (ch === '+' || ch === '=') {
        addMinute()
        return true
      }

      if (ch === '-' || ch === '_') {
        subtractMinute()
        return true
      }

      // Presets 1-5
      if (ch === '1') { setPreset(1); return true }
      if (ch === '2') { setPreset(2); return true }
      if (ch === '3') { setPreset(5); return true }
      if (ch === '4') { setPreset(10); return true }
      if (ch === '5') { setPreset(15); return true }

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
