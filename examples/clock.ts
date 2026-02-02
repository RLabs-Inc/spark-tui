/**
 * SparkTUI - Digital Clock
 *
 * A beautiful real-time digital clock featuring:
 * - Large ASCII art digits
 * - Date display with day of week
 * - Timezone information
 * - Smooth second ticking with cycle()
 * - Theme cycling
 *
 * Controls:
 *   t      Cycle theme
 *   q      Quit
 *   Ctrl+C Quit
 *
 * Run: bun run examples/clock.ts
 */

import { signal, derived } from '@rlabs-inc/signals'
import { mount } from '../ts/engine'
import { box, text, cycle } from '../ts/primitives'
import { t, themes, setTheme, getThemeNames } from '../ts/state/theme'
import { getChar } from '../ts/engine/events'
import { onCleanup } from '../ts/primitives/scope'

// =============================================================================
// ASCII DIGIT FONT (5 rows, 6 chars wide each)
// =============================================================================

const DIGITS: Record<string, string[]> = {
  '0': [
    ' ####  ',
    '#    # ',
    '#    # ',
    '#    # ',
    ' ####  ',
  ],
  '1': [
    '   #   ',
    '  ##   ',
    '   #   ',
    '   #   ',
    ' ##### ',
  ],
  '2': [
    ' ####  ',
    '     # ',
    ' ####  ',
    '#      ',
    '###### ',
  ],
  '3': [
    ' ####  ',
    '     # ',
    '  ###  ',
    '     # ',
    ' ####  ',
  ],
  '4': [
    '#    # ',
    '#    # ',
    '###### ',
    '     # ',
    '     # ',
  ],
  '5': [
    '###### ',
    '#      ',
    '#####  ',
    '     # ',
    '#####  ',
  ],
  '6': [
    ' ####  ',
    '#      ',
    '#####  ',
    '#    # ',
    ' ####  ',
  ],
  '7': [
    '###### ',
    '     # ',
    '    #  ',
    '   #   ',
    '  #    ',
  ],
  '8': [
    ' ####  ',
    '#    # ',
    ' ####  ',
    '#    # ',
    ' ####  ',
  ],
  '9': [
    ' ####  ',
    '#    # ',
    ' ##### ',
    '     # ',
    ' ####  ',
  ],
  ':': [
    '       ',
    '   #   ',
    '       ',
    '   #   ',
    '       ',
  ],
}

// =============================================================================
// STATE
// =============================================================================

const now = signal(new Date())
const themeNames = getThemeNames()
const themeIndex = signal(0)
const currentThemeName = derived(() => themeNames[themeIndex.value])

// Colon blink for clock separator
const colonVisible = cycle([true, false], { fps: 1 })

// =============================================================================
// DERIVED VALUES
// =============================================================================

const hours = derived(() => String(now.value.getHours()).padStart(2, '0'))
const minutes = derived(() => String(now.value.getMinutes()).padStart(2, '0'))
const seconds = derived(() => String(now.value.getSeconds()).padStart(2, '0'))

const dateStr = derived(() => {
  const d = now.value
  const days = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday']
  const months = ['January', 'February', 'March', 'April', 'May', 'June',
                  'July', 'August', 'September', 'October', 'November', 'December']
  return `${days[d.getDay()]}, ${months[d.getMonth()]} ${d.getDate()}, ${d.getFullYear()}`
})

const timezone = derived(() => {
  const tz = Intl.DateTimeFormat().resolvedOptions().timeZone
  const offset = -now.value.getTimezoneOffset()
  const sign = offset >= 0 ? '+' : '-'
  const h = Math.floor(Math.abs(offset) / 60)
  const m = Math.abs(offset) % 60
  return `${tz} (UTC${sign}${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')})`
})

// =============================================================================
// HELPERS
// =============================================================================

function cycleTheme() {
  themeIndex.value = (themeIndex.value + 1) % themeNames.length
  setTheme(themeNames[themeIndex.value] as keyof typeof themes)
}

/** Render a digit string as ASCII art (returns 5 rows) */
function renderDigitRow(digitStr: string, row: number, blinkColon: boolean): string {
  let result = ''
  for (const ch of digitStr) {
    if (ch === ':') {
      const colonRow = blinkColon ? DIGITS[':']![row] : '       '
      result += colonRow
    } else if (DIGITS[ch]) {
      result += DIGITS[ch]![row]
    }
  }
  return result
}

// =============================================================================
// APP
// =============================================================================

await mount(() => {
  // Update time every second
  const interval = setInterval(() => {
    now.value = new Date()
  }, 1000)

  onCleanup(() => clearInterval(interval))

  // Root container - centered
  box({
    width: '100%',
    height: '100%',
    flexDirection: 'column',
    justifyContent: 'center',
    alignItems: 'center',
    children: () => {
      // Clock card
      box({
        flexDirection: 'column',
        alignItems: 'center',
        border: 3, // rounded
        borderColor: t.primary,
        padding: 2,
        gap: 1,
        children: () => {
          // Title
          text({
            content: 'SparkTUI Clock',
            fg: t.textMuted,
          })

          // Large ASCII time display (HH:MM:SS)
          box({
            flexDirection: 'column',
            alignItems: 'center',
            paddingTop: 1,
            paddingBottom: 1,
            children: () => {
              // Render each row of the ASCII digits
              for (let row = 0; row < 5; row++) {
                text({
                  content: derived(() => {
                    const timeStr = `${hours.value}:${minutes.value}:${seconds.value}`
                    return renderDigitRow(timeStr, row, colonVisible.value)
                  }),
                  fg: t.primary,
                })
              }
            },
          })

          // Date display
          text({
            content: dateStr,
            fg: t.textBright,
          })

          // Timezone
          text({
            content: timezone,
            fg: t.textMuted,
          })

          // Help text
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
