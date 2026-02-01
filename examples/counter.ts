/**
 * SparkTUI — Beautiful Counter
 *
 * The classic demo app showcasing:
 * - Reactive state with signals
 * - Flexbox layout (centered, column, row)
 * - Styled components with variants
 * - Focus & keyboard navigation
 * - Click handlers
 *
 * Controls:
 *   + or =    Increment
 *   - or _    Decrement
 *   Tab       Navigate focus
 *   Enter     Activate focused button
 *   t         Cycle theme
 *   q         Quit
 *   Ctrl+C    Quit
 */

import { signal, derived } from '@rlabs-inc/signals'
import { box, text } from '../ts/primitives'
import { t, themes, setTheme, getThemeNames } from '../ts/state/theme'
import { mount } from '../ts/engine'
import { isEnter, isSpace, getChar } from '../ts/engine/events'

// =============================================================================
// REACTIVE STATE
// =============================================================================

const count = signal(0)
const themeNames = getThemeNames()
const themeIndex = signal(0)
const currentThemeName = derived(() => themeNames[themeIndex.value])

// =============================================================================
// APP
// =============================================================================

await mount(() => {
  // Root: Full terminal, centered
  box({
    flexDirection: 'column',
    children: () => {
      // Card container
      box({
        width: '100%',
        flexDirection: 'column',
        alignItems: 'center',
        padding: 2,
        paddingBottom: 0,
        gap: 2,
        border: 1,
        borderColor: t.secondary,
        children: () => {
          // Title
          text({
            content: '✨ SparkTUI Counter ✨',
            fg: t.primary,
          })

          // Counter row: [ - ]  42  [ + ]
          box({
            width: '100%',
            flexDirection: 'row',
            alignItems: 'center',
            justifyContent: 'space-between',
            gap: 2,
            children: () => {
              // Minus button
              box({
                width: 7,
                height: 3,
                justifyContent: 'center',
                alignItems: 'center',
                bg: t.error,
                fg: t.text,
                focusable: true,
                onClick: () => {
                  count.value = count.value - 1
                },
                onKey: (key) => {
                  if (isEnter(key) || isSpace(key)) {
                    count.value = count.value - 1
                    return true
                  }
                },
                children: () => {
                  text({ content: '  -  ', fg: t.text })
                },
              })

              // Count display
              box({
                justifyContent: 'center',
                alignItems: 'center',
                children: () => {
                  text({
                    content: count,
                    fg: t.text,
                  })
                },
              })

              // Plus button
              box({
                width: 7,
                height: 3,
                justifyContent: 'center',
                alignItems: 'center',
                bg: t.success,
                fg: t.text,
                focusable: true,
                onClick: () => {
                  count.value = count.value + 1
                },
                onKey: (key) => {
                  if (isEnter(key) || isSpace(key)) {
                    count.value = count.value + 1
                    return true
                  }
                },
                children: () => {
                  text({ content: '  +  ', fg: t.text })
                },
              })
            },
          })

          // Footer
          box({
            flexDirection: 'column',
            justifyContent: 'center',
            children: () => {
              text({
                content: '+/- count  q quit',
                fg: t.textMuted,
              })
              text({
                content: () => `t theme: ${currentThemeName.value}`,
                fg: t.textMuted,
              })
            },
          })
        },
      })
    },

    // Global keyboard handler
    onKey: (key) => {
      const ch = getChar(key)
      if (ch === '+' || ch === '=') {
        count.value = count.value + 1
        return true
      }
      if (ch === '-' || ch === '_') {
        count.value = count.value - 1
        return true
      }
      if (ch === 'q' || ch === 'Q') {
        process.exit(0)
      }
      if (ch === 't' || ch === 'T') {
        themeIndex.value = (themeIndex.value + 1) % themeNames.length
        setTheme(themeNames[themeIndex.value] as keyof typeof themes)
        return true
      }
    },
  })
}, { mode: 'inline' })
