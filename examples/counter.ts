/**
 * SparkTUI - Simple Counter
 *
 * A minimal counter demonstrating:
 * - Reactive state with signals
 * - Flexbox layout
 * - Theme-aware styling
 * - Keyboard shortcuts (+/- keys)
 * - Mouse click handling
 * - Theme cycling
 *
 * Controls:
 *   +/=    Increment
 *   -/_    Decrement
 *   r      Reset to zero
 *   t      Cycle theme
 *   q      Quit
 *   Ctrl+C Quit
 *
 * Run: bun run examples/counter.ts
 */

import { signal, derived } from '@rlabs-inc/signals'
import { mount } from '../ts/engine'
import { box, text } from '../ts/primitives'
import { t, themes, setTheme, getThemeNames } from '../ts/state/theme'
import { isEnter, isSpace, getChar } from '../ts/engine/events'

// =============================================================================
// STATE
// =============================================================================

const count = signal(0)
const themeNames = getThemeNames()
const themeIndex = signal(0)
const currentThemeName = derived(() => themeNames[themeIndex.value])

// =============================================================================
// HELPERS
// =============================================================================

function cycleTheme() {
  themeIndex.value = (themeIndex.value + 1) % themeNames.length
  setTheme(themeNames[themeIndex.value] as keyof typeof themes)
}

// =============================================================================
// APP
// =============================================================================

await mount(() => {
  // Root container - centered
  box({
    width: '100%',
    height: '100%',
    flexDirection: 'column',
    justifyContent: 'center',
    alignItems: 'center',
    children: () => {
      // Card
      box({
        width: 40,
        flexDirection: 'column',
        alignItems: 'center',
        border: 3, // rounded
        borderColor: t.primary,
        padding: 2,
        gap: 2,
        children: () => {
          // Title
          text({
            content: 'SparkTUI Counter',
            fg: t.primary,
          })

          // Counter display row: [ - ] count [ + ]
          box({
            width: '100%',
            flexDirection: 'row',
            justifyContent: 'space-between',
            alignItems: 'center',
            children: () => {
              // Decrement button
              box({
                width: 9,
                height: 3,
                justifyContent: 'center',
                alignItems: 'center',
                border: 1,
                borderColor: t.error,
                focusable: true,
                onClick: () => { count.value-- },
                onKey: (key) => {
                  if (isEnter(key) || isSpace(key)) {
                    count.value--
                    return true
                  }
                },
                children: () => {
                  text({ content: '  -  ', fg: t.error })
                },
              })

              // Count display
              box({
                width: 10,
                height: 3,
                justifyContent: 'center',
                alignItems: 'center',
                border: 1,
                borderColor: t.textMuted,
                children: () => {
                  text({
                    content: derived(() => String(count.value).padStart(4)),
                    fg: t.textBright,
                  })
                },
              })

              // Increment button
              box({
                width: 9,
                height: 3,
                justifyContent: 'center',
                alignItems: 'center',
                border: 1,
                borderColor: t.success,
                focusable: true,
                onClick: () => { count.value++ },
                onKey: (key) => {
                  if (isEnter(key) || isSpace(key)) {
                    count.value++
                    return true
                  }
                },
                children: () => {
                  text({ content: '  +  ', fg: t.success })
                },
              })
            },
          })

          // Help text
          box({
            flexDirection: 'column',
            alignItems: 'center',
            gap: 0,
            children: () => {
              text({
                content: '+/- count | r reset | t theme',
                fg: t.textMuted,
              })
              text({
                content: derived(() => `Theme: ${currentThemeName.value}`),
                fg: t.textDim,
              })
            },
          })
        },
      })
    },

    // Global keyboard handler
    onKey: (key) => {
      const ch = getChar(key)

      // Increment
      if (ch === '+' || ch === '=') {
        count.value++
        return true
      }

      // Decrement
      if (ch === '-' || ch === '_') {
        count.value--
        return true
      }

      // Reset
      if (ch === 'r' || ch === 'R') {
        count.value = 0
        return true
      }

      // Theme cycle
      if (ch === 't' || ch === 'T') {
        cycleTheme()
        return true
      }

      // Quit
      if (ch === 'q' || ch === 'Q') {
        process.exit(0)
      }
    },
  })
})
