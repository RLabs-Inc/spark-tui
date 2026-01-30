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
 *   q         Quit
 *   Ctrl+C    Quit
 */

import { signal, derived } from '@rlabs-inc/signals'
import { box, text } from '../ts/primitives'
import { t, themes, setTheme, getThemeNames } from '../ts/state/theme'
import { mount } from '../ts/engine'
import { loadEngine } from '../ts/bridge/ffi'
import { isEnter, isSpace, isChar, getChar, KeyEvent } from '../ts/engine/events'
import { ptr } from 'bun:ffi'
import { join } from 'path'

// =============================================================================
// BUILD RUST ENGINE
// =============================================================================

console.log('[counter] Building Rust engine...')
const buildResult = Bun.spawnSync({
  cmd: ['cargo', 'build', '--release'],
  cwd: join(import.meta.dir, '../rust'),
  stdout: 'inherit',
  stderr: 'inherit',
})

if (buildResult.exitCode !== 0) {
  console.error('[counter] Failed to build Rust engine')
  process.exit(1)
}

// =============================================================================
// REACTIVE STATE
// =============================================================================

const count = signal(0)
const themeNames = getThemeNames()
const themeIndex = signal(0)
const currentThemeName = derived(() => themeNames[themeIndex.value])

// Derived display string with padding for visual stability
// const countDisplay = derived(() => {
//   const n = count.value
//   const sign = n >= -9 ? '' : ' '
//   return sign + n.toString().padStart(4, '')
// })

// =============================================================================
// MOUNT APP
// =============================================================================

console.log('[counter] Mounting app...')

// Use 'inline' mode to test if positioning bug is diff-related
// Change to 'fullscreen' for normal behavior
const { buffer, unmount } = mount(() => {
  // Root: Full terminal, centered
  box({
    // width: '100%',
    // height: '100%',
    // overflow: 'scroll',
    flexDirection: 'column',
    // justifyContent: 'space-around',
    // alignItems: 'center',
    children: () => {
      // Card container
      box({
        width: '100%',
        // height: 50,
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
            // gap: 2,
            children: () => {
              // Minus button
              box({
                width: 7,
                height: 3,
                justifyContent: 'center',
                alignItems: 'center',
                // border: 1,
                // borderColor: t.error,
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
                // border: 1,
                // borderColor: t.success,
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
          box({
            // Footer
            flexDirection: 'column',
            justifyContent: 'center',
            // visible: false,
            children: () => {
              text({
                content: '+/- count  q quit',
                fg: t.textMuted
              })
              text({
                content: () => `t theme: ${currentThemeName.value}`,
                fg: t.textMuted
              })
            }
          })
        }
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
        unmount()
        process.exit(0)
      }
      if (ch === 't' || ch === 'T') {
        themeIndex.value = (themeIndex.value + 1) % themeNames.length
        setTheme(themeNames[themeIndex.value] as keyof typeof themes)
        return true
      }
    }

  })

}, { mode: 'inline' }) // Test inline mode - change to 'fullscreen' for normal

// =============================================================================
// START RUST ENGINE
// =============================================================================

console.log('[counter] Starting Rust engine...')
const engine = loadEngine()
const result = engine.init(ptr(buffer.buffer), buffer.buffer.byteLength)

if (result !== 0) {
  console.error(`[counter] Engine init failed: ${result}`)
  process.exit(1)
}

console.log('[counter] Running! Press +/- to count, q to quit')

// Keep process alive - engine handles stdin
await new Promise(() => { })
