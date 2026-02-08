/**
 * SparkTUI - Shift Bug Test
 *
 * Minimal test to diagnose the first-render position shift bug.
 * Tests both 100% root (shifts) and fixed-size root (doesn't shift).
 *
 * Run: bun run examples/test-shift.ts [--fixed]
 *
 * After running, check /tmp/spark-debug.log for diagnostic data.
 * Press + to increment counter. If the card moves, bug confirmed.
 */

import {
  signal,
  derived,
  mount,
  box,
  text,
  getChar,
} from '@spark-tui/core'

const count = signal(0)

const useFixed = process.argv.includes('--fixed')
const cols = process.stdout.columns || 80
const rows = process.stdout.rows || 24

// Log TS-side terminal size for comparison with Rust ioctl
const fs = await import('fs')
fs.appendFileSync('/tmp/spark-debug.log',
  `[TS] process.stdout: ${cols}x${rows} useFixed=${useFixed}\n`)

mount(() => {
  // Root: either 100% (bug) or fixed-size (no bug)
  box({
    width: useFixed ? cols : '100%',
    height: useFixed ? rows : '100%',
    justifyContent: 'center',
    alignItems: 'center',
    border: 1,
    borderColor: [80, 80, 100],
    bg: [20, 20, 30],
    onKey: (e) => {
      const ch = getChar(e)
      if (ch === '+' || ch === '=') { count.value++; return true }
      if (ch === '-' || ch === '_') { count.value--; return true }
    },
    focusable: true,
    children: () => {
      box({
        width: 40,
        height: 8,
        border: 1,
        borderColor: [120, 120, 150],
        justifyContent: 'center',
        alignItems: 'center',
        flexDirection: 'column',
        gap: 1,
        children: () => {
          text({ content: 'SHIFT TEST', fg: [200, 200, 220] })
          text({ content: 'Press + to increment', fg: [150, 150, 170] })
          text({
            content: derived(() => `Count: ${count.value}`),
            fg: [180, 180, 200],
          })
          text({
            content: 'If this card moves, bug confirmed',
            fg: [100, 100, 120],
          })
        },
      })
    },
  })
}, {
  mode: 'fullscreen',
})

await new Promise(() => {})
