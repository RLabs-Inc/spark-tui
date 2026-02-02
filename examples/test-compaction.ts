/**
 * Test text pool compaction.
 *
 * Simulates an unbounded growth pattern that would overflow without compaction,
 * but succeeds because compaction reclaims dead space.
 */

import {
  createSharedBuffer,
  setText,
  getTextPoolWritePtr,
  compactTextPool,
  getNodeCount,
  setNodeCount,
} from '../ts/bridge/shared-buffer'

// Create a small buffer for testing (100KB pool instead of 10MB)
const buf = createSharedBuffer({ maxNodes: 100, textPoolSize: 100 * 1024 })

// Simulate 100 nodes, each with text
setNodeCount(buf, 50)

console.log('=== Text Pool Compaction Test ===\n')
console.log(`Pool size: ${(buf.textPoolSize / 1024).toFixed(1)}KB`)
console.log(`Nodes: ${getNodeCount(buf)}`)

// Phase 1: Alternating between short and long text to create garbage
console.log('\n--- Phase 1: Creating garbage with alternating lengths ---')

const shortText = 'Hi'
const longText = 'This is a much longer piece of text that takes more space'

for (let round = 0; round < 500; round++) {
  for (let node = 0; node < 50; node++) {
    // Alternate between short and long
    const text = (round + node) % 2 === 0 ? shortText : longText
    const result = setText(buf, node, text)

    if (!result.success) {
      console.log(`Round ${round}, node ${node}: Pool full, compaction should have triggered`)
      console.log(`  Live: ${result.liveBytes} bytes, Pool: ${result.poolSize} bytes`)
      process.exit(1)
    }
  }

  if (round % 100 === 0) {
    const ptr = getTextPoolWritePtr(buf)
    console.log(`  Round ${round}: writePtr = ${ptr} bytes (${(ptr / 1024).toFixed(1)}KB)`)
  }
}

console.log('\n--- Phase 2: Monotonic growth (would fail without compaction) ---')

// Now grow each node's text monotonically
// This creates O(n) garbage per round
for (let round = 0; round < 100; round++) {
  const baseText = 'x'.repeat(50 + round) // Grows each round

  for (let node = 0; node < 50; node++) {
    const result = setText(buf, node, baseText + node)

    if (!result.success) {
      console.log(`\nGrowth test failed at round ${round}, node ${node}`)
      console.log(`  Live: ${result.liveBytes} bytes, Needed: ${result.needed} bytes`)
      console.log(`  This is expected if live data exceeds pool size`)

      // Calculate what live data should be
      const textLen = baseText.length + String(node).length
      const totalLive = textLen * 50
      console.log(`  Expected live: ~${totalLive} bytes`)

      if (result.liveBytes > buf.textPoolSize * 0.9) {
        console.log('\n✓ Pool genuinely full (live data exceeds capacity)')
        console.log('  Compaction is working - this is expected behavior')
        process.exit(0)
      }
      process.exit(1)
    }
  }

  if (round % 20 === 0) {
    const ptr = getTextPoolWritePtr(buf)
    console.log(`  Round ${round}: writePtr = ${ptr} bytes, text len = ${baseText.length}`)
  }
}

const finalPtr = getTextPoolWritePtr(buf)
console.log(`\n=== SUCCESS ===`)
console.log(`Final writePtr: ${finalPtr} bytes (${(finalPtr / 1024).toFixed(1)}KB)`)
console.log(`Compaction kept pool usage bounded despite unbounded updates!`)
