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
import { mount } from '../ts/engine'
import { loadEngine } from '../ts/bridge/ffi'
import { isEnter, isSpace, isChar, getChar } from '../ts/engine/events'
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

// Derived display string with padding for visual stability
const countDisplay = derived(() => {
  const n = count.value
  const sign = n >= 0 ? ' ' : ''
  return sign + n.toString().padStart(4, ' ')
})

// =============================================================================
// MOUNT APP
// =============================================================================

console.log('[counter] Mounting app...')

const { buffer, unmount } = mount(() => {
  // Root: Full terminal, centered
  box({
    width: '100%',
    height: '100%',
    flexDirection: 'column',
    justifyContent: 'center',
    alignItems: 'center',
    bg: { r: 30, g: 30, b: 46, a: 255 }, // Catppuccin base
    children: () => {
      // Card container
      box({
        // width: 40,
        flexDirection: 'column',
        alignItems: 'center',
        padding: 2,
        gap: 1,
        border: 1,
        borderColor: { r: 69, g: 71, b: 90, a: 255 }, // overlay0
        bg: { r: 49, g: 50, b: 68, a: 255 }, // surface0
        children: () => {
          // Title
          text({
            content: '✨ SparkTUI Counter ✨',
            fg: { r: 180, g: 190, b: 254, a: 255 }, // lavender
          })

          // Spacer
          box({ height: 1 })

          // Counter row: [ - ]  42  [ + ]
          box({
            flexDirection: 'row',
            alignItems: 'center',
            justifyContent: 'center',
            gap: 2,
            children: () => {
              // Minus button
              box({
                width: 7,
                height: 3,
                justifyContent: 'center',
                alignItems: 'center',
                border: 1,
                borderColor: { r: 243, g: 139, b: 168, a: 255 }, // red
                bg: { r: 243, g: 139, b: 168, a: 255 },
                fg: { r: 30, g: 30, b: 46, a: 255 },
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
                  text({ content: '  -  ', fg: { r: 30, g: 30, b: 46, a: 255 } })
                },
              })

              // Count display
              box({
                width: 8,
                height: 3,
                justifyContent: 'center',
                alignItems: 'center',
                children: () => {
                  text({
                    content: countDisplay,
                    fg: { r: 205, g: 214, b: 244, a: 255 }, // text
                  })
                },
              })

              // Plus button
              box({
                width: 7,
                height: 3,
                justifyContent: 'center',
                alignItems: 'center',
                border: 1,
                borderColor: { r: 166, g: 227, b: 161, a: 255 }, // green
                bg: { r: 166, g: 227, b: 161, a: 255 },
                fg: { r: 30, g: 30, b: 46, a: 255 },
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
                  text({ content: '  +  ', fg: { r: 30, g: 30, b: 46, a: 255 } })
                },
              })
            },
          })

          // Spacer
          box({ height: 1 })

          // Footer
          text({
            content: '+/- count  Tab focus  q quit',
            fg: { r: 166, g: 173, b: 200, a: 255 }, // subtext0
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
        unmount()
        process.exit(0)
      }
    },
  })
})

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
