/**
 * Debug: Compare inline vs fullscreen mode
 */

import { signal } from '@rlabs-inc/signals'
import { box, text } from '../ts/primitives'
import { mountSync } from '../ts/engine'
import { getRenderCount, wakeRust } from '../ts/bridge/shared-buffer'

console.log('Testing INLINE mode...')

const counter1 = signal(0)

const handle1 = mountSync(() => {
  box({
    width: 30,
    height: 3,
    children: () => {
      text({ content: counter1 })
    }
  })
}, { mode: 'inline' })

await Bun.sleep(300)
console.log('Inline - Initial render count:', getRenderCount(handle1.buffer))

wakeRust(handle1.buffer)
await Bun.sleep(100)
console.log('Inline - After wake:', getRenderCount(handle1.buffer))

// Check wake flag
const wakeIdx = 64 / 4
const flag = Atomics.load(handle1.buffer.headerI32, wakeIdx)
console.log('Inline - Wake flag consumed?', flag === 0 ? 'YES' : 'NO (still ' + flag + ')')

handle1.unmount()
console.log('\nDone.')
process.exit(0)
