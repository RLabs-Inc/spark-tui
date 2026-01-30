/**
 * SparkTUI — Scroll System Test
 *
 * Tests the auto-scroll detection system:
 * 1. Normal scroll - explicit overflow: 'scroll' with content > container
 * 2. Auto-detected scroll - no explicit overflow, but children computed height > container
 * 3. Nested scroll - scroll containers inside scroll containers
 * 4. Root scroll - when total content exceeds terminal height
 *
 * Controls:
 *   Tab       Navigate focus between scrollable areas
 *   ↑/↓       Scroll focused container
 *   q         Quit
 */

import { signal, derived } from '@rlabs-inc/signals'
import { box, text } from '../ts/primitives'
import { t, setTheme } from '../ts/state/theme'
import { mount } from '../ts/engine'
import { loadEngine } from '../ts/bridge/ffi'
import { isChar, getChar, KeyEvent } from '../ts/engine/events'
import { ptr } from 'bun:ffi'
import { join } from 'path'

// =============================================================================
// BUILD RUST ENGINE
// =============================================================================

console.log('[scroll-test] Building Rust engine...')
const buildResult = Bun.spawnSync({
  cmd: ['cargo', 'build', '--release'],
  cwd: join(import.meta.dir, '../rust'),
  stdout: 'inherit',
  stderr: 'inherit',
})

if (buildResult.exitCode !== 0) {
  console.error('[scroll-test] Failed to build Rust engine')
  process.exit(1)
}

// Set a nice theme
setTheme('tokyoNight')

// =============================================================================
// HELPER: Generate lines of content
// =============================================================================

function generateLines(prefix: string, count: number): string[] {
  return Array.from({ length: count }, (_, i) => `${prefix} line ${i + 1}`)
}

// =============================================================================
// MOUNT APP
// =============================================================================

console.log('[scroll-test] Mounting app...')

const { buffer, unmount } = mount(() => {
  // Root container - will it become scrollable when content > terminal?
  box({
    width: '100%',
    height: '100%',
    flexDirection: 'column',
    padding: 1,
    gap: 1,
    bg: t.bg,
    fg: t.text,
    // Note: NO overflow specified - testing if auto-scroll kicks in
    children: () => {
      // Title
      text({
        content: '🔄 Scroll System Test',
        fg: t.primary,
      })
      text({
        content: 'Tab to navigate, ↑/↓ to scroll, q to quit',
        fg: t.textMuted,
      })

      // =======================================================================
      // TEST 1: Explicit scroll container
      // =======================================================================
      box({
        flexDirection: 'column',
        gap: 0,
        children: () => {
          text({ content: '━━━ TEST 1: Explicit overflow: scroll ━━━', fg: t.secondary })
          box({
            width: 40,
            height: 6,
            overflow: 'scroll',
            border: 1,
            borderColor: t.success,
            flexDirection: 'column',
            padding: 1,
            children: () => {
              for (const line of generateLines('Explicit', 20)) {
                text({ content: line, fg: t.text })
              }
            },
          })
        },
      })

      // =======================================================================
      // TEST 2: Auto-scroll (NO overflow prop - should auto-detect)
      // =======================================================================
      box({
        flexDirection: 'column',
        gap: 0,
        children: () => {
          text({ content: '━━━ TEST 2: Auto-scroll (computed sizes) ━━━', fg: t.secondary })
          // This box has fixed height but NO overflow prop
          // Rule: children computed size > parent computed size → parent scrollable
          // Should auto-detect that children > container and become scrollable
          box({
            width: 40,
            height: 6,
            // NO overflow specified! Auto-scroll kicks in automatically
            border: 1,
            borderColor: t.warning,
            flexDirection: 'column',
            padding: 1,
            children: () => {
              for (const line of generateLines('Auto', 20)) {
                text({ content: line, fg: t.text })
              }
            },
          })
        },
      })

      // =======================================================================
      // TEST 3: Nested scroll containers
      // =======================================================================
      box({
        flexDirection: 'column',
        gap: 0,
        children: () => {
          text({ content: '━━━ TEST 3: Nested scroll containers ━━━', fg: t.secondary })
          // Outer scroll container
          box({
            width: 50,
            height: 10,
            overflow: 'scroll',
            border: 1,
            borderColor: t.info,
            flexDirection: 'column',
            padding: 1,
            gap: 1,
            children: () => {
              text({ content: '↓ OUTER container (scroll me)', fg: t.info })

              // Inner scroll container
              box({
                width: '100%',
                height: 4,
                overflow: 'scroll',
                border: 1,
                borderColor: t.error,
                flexDirection: 'column',
                children: () => {
                  text({ content: '↓ INNER container', fg: t.error })
                  for (const line of generateLines('  Inner', 15)) {
                    text({ content: line, fg: t.textMuted })
                  }
                },
              })

              // More content in outer
              for (const line of generateLines('Outer', 15)) {
                text({ content: line, fg: t.text })
              }
            },
          })
        },
      })

      // =======================================================================
      // TEST 4: Content to push root into scroll territory
      // =======================================================================
      box({
        flexDirection: 'column',
        gap: 0,
        children: () => {
          text({ content: '━━━ TEST 4: Root overflow test ━━━', fg: t.secondary })
          text({ content: 'Below is lots of content to make root scrollable:', fg: t.textMuted })

          // Generate enough content to overflow typical terminal
          for (let i = 1; i <= 30; i++) {
            text({
              content: `Root content line ${i} - if you can see this, root scroll works!`,
              fg: i % 5 === 0 ? t.primary : t.text,
            })
          }
        },
      })

      // Footer at the very bottom
      text({
        content: '🏁 END OF CONTENT - You scrolled to the bottom!',
        fg: t.success,
      })
    },

    // Global keyboard handler
    onKey: (key) => {
      const ch = getChar(key)
      if (ch === 'q' || ch === 'Q') {
        unmount()
        process.exit(0)
      }
    },
  })
}, { mode: 'fullscreen' })

// =============================================================================
// START RUST ENGINE
// =============================================================================

console.log('[scroll-test] Starting Rust engine...')
const engine = loadEngine()
const result = engine.init(ptr(buffer.buffer), buffer.buffer.byteLength)

if (result !== 0) {
  console.error(`[scroll-test] Engine init failed: ${result}`)
  process.exit(1)
}

console.log('[scroll-test] Running! Tab between areas, ↑/↓ to scroll, q to quit')

// Keep process alive
await new Promise(() => {})
