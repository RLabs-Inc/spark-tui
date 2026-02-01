/**
 * Debug script to check render count
 */

import { signal } from '@rlabs-inc/signals'
import { box, text } from '../ts/primitives'
import { mountSync } from '../ts/engine'
import { getRenderCount, getNodeCount } from '../ts/bridge/shared-buffer'

const counter = signal(0)

const handle = mountSync(() => {
  box({
    width: 30,
    height: 5,
    children: () => {
      text({ content: counter })
    }
  })
}, { mode: 'inline' })

await Bun.sleep(300)  // Let engine fully initialize - NO console.log before this!

const nodeCount = getNodeCount(handle.buffer)
const initialCount = getRenderCount(handle.buffer)
console.log('Mounted, node count:', nodeCount)
console.log('Initial render count:', initialCount)

await Bun.sleep(200)
console.log('After 500ms, render count:', getRenderCount(handle.buffer))

counter.value = 1
console.log('After counter=1, render count:', getRenderCount(handle.buffer))

await Bun.sleep(100)
console.log('After 100ms more, render count:', getRenderCount(handle.buffer))

counter.value = 2
await Bun.sleep(100)
console.log('After counter=2 + 100ms, render count:', getRenderCount(handle.buffer))

handle.unmount()
process.exit(0)
