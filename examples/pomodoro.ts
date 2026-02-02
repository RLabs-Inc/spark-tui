/**
 * SparkTUI - Pomodoro Timer
 *
 * A beautiful Pomodoro technique timer demonstrating:
 * - Time-based reactive updates with signals
 * - Large centered displays
 * - State machine (work/break/idle)
 * - Session tracking
 * - Visual mode indicators with colors
 * - Sound notifications (terminal bell)
 *
 * Controls:
 *   Space/Enter  Start/Pause timer
 *   r            Reset current timer
 *   s            Skip to next session
 *   +/-          Adjust work time (+/- 5 min)
 *   q            Quit
 */

import { signal, derived, effect } from '@rlabs-inc/signals'
import { mount } from '../ts/engine/mount'
import { box, text, show, cycle, Frames } from '../ts/primitives'
import { packColor } from '../ts/bridge/shared-buffer'
import { on, isPress } from '../ts/state/keyboard'
import type { KeyEvent } from '../ts/state/keyboard'
import { getChar, isEnter, isSpace } from '../ts/engine/events'

// =============================================================================
// TYPES
// =============================================================================

type TimerState = 'idle' | 'work' | 'break'

// =============================================================================
// COLORS
// =============================================================================

const colors = {
  bg: packColor(20, 20, 28, 255),
  bgCard: packColor(28, 28, 38, 255),
  border: packColor(50, 50, 70, 255),
  text: packColor(220, 220, 235, 255),
  textMuted: packColor(100, 100, 130, 255),
  textBright: packColor(255, 255, 255, 255),

  // Mode colors
  work: packColor(220, 80, 80, 255),       // Red for work
  workBg: packColor(60, 30, 30, 255),
  break: packColor(80, 200, 120, 255),     // Green for break
  breakBg: packColor(30, 60, 40, 255),
  idle: packColor(100, 140, 220, 255),     // Blue for idle
  idleBg: packColor(30, 40, 60, 255),
}

// =============================================================================
// CONFIGURATION
// =============================================================================

const WORK_MINUTES = signal(25)
const BREAK_MINUTES = signal(5)
const LONG_BREAK_MINUTES = 15
const SESSIONS_UNTIL_LONG_BREAK = 4

// =============================================================================
// STATE
// =============================================================================

// Timer state
const timerState = signal<TimerState>('idle')
const isRunning = signal(false)

// Time remaining in seconds
const timeRemaining = signal(WORK_MINUTES.value * 60)

// Session tracking
const sessionsCompleted = signal(0)
const totalWorkMinutes = signal(0)

// Notification flag
const showNotification = signal(false)

// =============================================================================
// DERIVED STATE
// =============================================================================

// Format time as MM:SS
const timeDisplay = derived(() => {
  const total = timeRemaining.value
  const mins = Math.floor(total / 60)
  const secs = total % 60
  return `${String(mins).padStart(2, '0')}:${String(secs).padStart(2, '0')}`
})

// Large time digits for display
const timeDigits = derived(() => {
  const display = timeDisplay.value
  return display.split('')
})

// Progress percentage
const progress = derived(() => {
  const state = timerState.value
  if (state === 'idle') return 0

  const total = state === 'work'
    ? WORK_MINUTES.value * 60
    : isLongBreak()
      ? LONG_BREAK_MINUTES * 60
      : BREAK_MINUTES.value * 60

  return Math.round(((total - timeRemaining.value) / total) * 100)
})

// Mode label
const modeLabel = derived(() => {
  switch (timerState.value) {
    case 'work': return 'FOCUS TIME'
    case 'break': return isLongBreak() ? 'LONG BREAK' : 'SHORT BREAK'
    default: return 'READY'
  }
})

// Mode color
const modeColor = derived(() => {
  switch (timerState.value) {
    case 'work': return colors.work
    case 'break': return colors.break
    default: return colors.idle
  }
})

// Mode background
const modeBg = derived(() => {
  switch (timerState.value) {
    case 'work': return colors.workBg
    case 'break': return colors.breakBg
    default: return colors.idleBg
  }
})

// Stats display
const statsText = derived(() => {
  const sessions = sessionsCompleted.value
  const minutes = totalWorkMinutes.value
  return `${sessions} session${sessions === 1 ? '' : 's'} | ${minutes} min focused`
})

// =============================================================================
// HELPERS
// =============================================================================

function isLongBreak(): boolean {
  return sessionsCompleted.value > 0 &&
    sessionsCompleted.value % SESSIONS_UNTIL_LONG_BREAK === 0
}

function playBell() {
  // Terminal bell
  process.stdout.write('\x07')
  showNotification.value = true
  setTimeout(() => { showNotification.value = false }, 2000)
}

// =============================================================================
// ACTIONS
// =============================================================================

function startTimer() {
  if (timerState.value === 'idle') {
    timerState.value = 'work'
    timeRemaining.value = WORK_MINUTES.value * 60
  }
  isRunning.value = true
}

function pauseTimer() {
  isRunning.value = false
}

function toggleTimer() {
  if (isRunning.value) {
    pauseTimer()
  } else {
    startTimer()
  }
}

function resetTimer() {
  isRunning.value = false
  const state = timerState.value
  if (state === 'work') {
    timeRemaining.value = WORK_MINUTES.value * 60
  } else if (state === 'break') {
    timeRemaining.value = isLongBreak()
      ? LONG_BREAK_MINUTES * 60
      : BREAK_MINUTES.value * 60
  } else {
    timeRemaining.value = WORK_MINUTES.value * 60
  }
}

function skipToNext() {
  isRunning.value = false
  const state = timerState.value

  if (state === 'work' || state === 'idle') {
    // Skip to break
    timerState.value = 'break'
    if (state === 'work') {
      sessionsCompleted.value++
      totalWorkMinutes.value += WORK_MINUTES.value
    }
    timeRemaining.value = isLongBreak()
      ? LONG_BREAK_MINUTES * 60
      : BREAK_MINUTES.value * 60
  } else {
    // Skip to work
    timerState.value = 'work'
    timeRemaining.value = WORK_MINUTES.value * 60
  }
}

function completeSession() {
  playBell()
  const state = timerState.value

  if (state === 'work') {
    sessionsCompleted.value++
    totalWorkMinutes.value += WORK_MINUTES.value
    timerState.value = 'break'
    timeRemaining.value = isLongBreak()
      ? LONG_BREAK_MINUTES * 60
      : BREAK_MINUTES.value * 60
    isRunning.value = true
  } else if (state === 'break') {
    timerState.value = 'work'
    timeRemaining.value = WORK_MINUTES.value * 60
    isRunning.value = true
  }
}

function adjustWorkTime(delta: number) {
  const newTime = Math.max(5, Math.min(60, WORK_MINUTES.value + delta))
  WORK_MINUTES.value = newTime
  if (timerState.value === 'idle' || (timerState.value === 'work' && !isRunning.value)) {
    timeRemaining.value = newTime * 60
  }
}

// =============================================================================
// TIMER TICK
// =============================================================================

let tickInterval: ReturnType<typeof setInterval> | null = null

effect(() => {
  if (isRunning.value) {
    if (!tickInterval) {
      tickInterval = setInterval(() => {
        if (timeRemaining.value > 0) {
          timeRemaining.value--
        } else {
          completeSession()
        }
      }, 1000)
    }
  } else {
    if (tickInterval) {
      clearInterval(tickInterval)
      tickInterval = null
    }
  }
})

// =============================================================================
// APP
// =============================================================================

const cols = process.stdout.columns || 80
const rows = process.stdout.rows || 24

mount(() => {
  // Global keyboard handler
  on((event: KeyEvent) => {
    if (!isPress(event)) return false

    const char = getChar(event)

    // Quit
    if (char === 'q' || char === 'Q') {
      if (tickInterval) clearInterval(tickInterval)
      process.exit(0)
    }

    // Start/Pause with Space or Enter
    if (isSpace(event) || isEnter(event)) {
      toggleTimer()
      return true
    }

    // Reset
    if (char === 'r' || char === 'R') {
      resetTimer()
      return true
    }

    // Skip
    if (char === 's' || char === 'S') {
      skipToNext()
      return true
    }

    // Adjust work time
    if (char === '+' || char === '=') {
      adjustWorkTime(5)
      return true
    }
    if (char === '-' || char === '_') {
      adjustWorkTime(-5)
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
    alignItems: 'center',
    justifyContent: 'center',
    bg: colors.bg,
    children: () => {
      // Main timer card
      box({
        id: 'timer-card',
        width: 50,
        flexDirection: 'column',
        alignItems: 'center',
        border: 1,
        borderColor: modeColor,
        bg: modeBg,
        padding: 2,
        children: () => {
          // Title
          text({
            content: 'POMODORO',
            fg: colors.textMuted,
          })

          // Mode indicator
          box({
            marginTop: 1,
            marginBottom: 1,
            flexDirection: 'row',
            alignItems: 'center',
            gap: 1,
            children: () => {
              // Running indicator
              show(
                () => isRunning.value,
                () => {
                  return text({
                    content: cycle(Frames.pulse, { fps: 2 }),
                    fg: modeColor,
                  })
                }
              )

              text({
                content: modeLabel,
                fg: modeColor,
              })
            },
          })

          // Large time display
          box({
            flexDirection: 'row',
            alignItems: 'center',
            justifyContent: 'center',
            marginTop: 1,
            marginBottom: 1,
            children: () => {
              text({
                content: timeDisplay,
                fg: colors.textBright,
              })
            },
          })

          // Progress bar
          box({
            width: 40,
            height: 1,
            bg: colors.border,
            marginTop: 1,
            children: () => {
              box({
                width: () => Math.round(40 * progress.value / 100),
                height: 1,
                bg: modeColor,
              })
            },
          })

          // Session dots
          box({
            flexDirection: 'row',
            gap: 1,
            marginTop: 2,
            children: () => {
              for (let i = 0; i < SESSIONS_UNTIL_LONG_BREAK; i++) {
                const idx = i
                text({
                  content: () => sessionsCompleted.value % SESSIONS_UNTIL_LONG_BREAK > idx ? '\u25CF' : '\u25CB',
                  fg: () => sessionsCompleted.value % SESSIONS_UNTIL_LONG_BREAK > idx ? colors.work : colors.textMuted,
                })
              }
            },
          })

          // Stats
          box({
            marginTop: 2,
            children: () => {
              text({
                content: statsText,
                fg: colors.textMuted,
              })
            },
          })

          // Control buttons
          box({
            flexDirection: 'row',
            gap: 2,
            marginTop: 2,
            children: () => {
              // Start/Pause button
              box({
                width: 12,
                height: 3,
                justifyContent: 'center',
                alignItems: 'center',
                bg: modeColor,
                fg: colors.bg,
                border: 1,
                borderColor: modeColor,
                focusable: true,
                onClick: toggleTimer,
                children: () => {
                  text({
                    content: () => isRunning.value ? 'PAUSE' : 'START',
                    fg: colors.bg,
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
                borderColor: colors.border,
                focusable: true,
                onClick: resetTimer,
                children: () => {
                  text({ content: 'RESET', fg: colors.text })
                },
              })

              // Skip button
              box({
                width: 10,
                height: 3,
                justifyContent: 'center',
                alignItems: 'center',
                border: 1,
                borderColor: colors.border,
                focusable: true,
                onClick: skipToNext,
                children: () => {
                  text({ content: 'SKIP', fg: colors.text })
                },
              })
            },
          })
        },
      })

      // Notification toast
      show(
        () => showNotification.value,
        () => {
          return box({
            marginTop: 2,
            padding: 1,
            paddingLeft: 2,
            paddingRight: 2,
            bg: colors.break,
            children: () => {
              text({
                content: () => timerState.value === 'break' ? 'Time for a break!' : 'Break is over - back to work!',
                fg: colors.bg,
              })
            },
          })
        }
      )

      // Settings and help
      box({
        marginTop: 2,
        flexDirection: 'column',
        alignItems: 'center',
        children: () => {
          text({
            content: () => `Work: ${WORK_MINUTES.value}min (+/- to adjust) | Break: ${BREAK_MINUTES.value}min`,
            fg: colors.textMuted,
          })
          box({ height: 1 })
          text({
            content: 'Space:start/pause  r:reset  s:skip  q:quit',
            fg: colors.textMuted,
          })
        },
      })
    },
  })
}, { mode: 'fullscreen' })

console.log('[pomodoro] Started - Press Space to begin')
await new Promise(() => {})
