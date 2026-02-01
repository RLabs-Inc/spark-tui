/**
 * Memory leak isolation test
 *
 * Run: /usr/bin/time -l bun examples/mem-leak-test.ts
 */

import { signal } from '@rlabs-inc/signals'
import { box, text } from '../ts/primitives'
import { mountSync } from '../ts/engine'

function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms))
}

console.log('Memory Leak Isolation Test')
console.log('==========================\n')

const counter = signal(0)

const app = mountSync(() => {
  box({
    children: () => {
      text({ content: counter })
    },
  })
}, { mode: 'inline' })

// Wait for initial render
await sleep(500)

console.log('Starting 5 seconds of sustained updates...')
console.log('Watch Activity Monitor - memory should stay ~stable')
console.log('')

const duration = 5000 // 5 seconds
let updates = 0
const start = Date.now()

while (Date.now() - start < duration) {
  counter.value = updates++
  await sleep(0)
}

console.log(`\nCompleted ${updates} updates`)
console.log('Waiting 2 seconds before exit...')
await sleep(2000)

app.unmount()
console.log('Done')
process.exit(0)
