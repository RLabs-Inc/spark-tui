/**
 * SparkTUI Node Stress Test
 *
 * Performance showcase demonstrating SparkTUI's ability to handle
 * large numbers of nodes efficiently. Tests the reactive system
 * with 100, 500, 1000, and 5000 nodes.
 *
 * Key concepts demonstrated:
 * - Massive node creation/destruction
 * - Memory usage tracking
 * - FPS/render time measurement
 * - Reactive updates at scale
 *
 * Controls:
 * - 1/2/3/4: Switch node counts (100/500/1000/5000)
 * - Space: Toggle animation (updating nodes)
 * - r: Reset stats
 * - Ctrl+C: Exit
 */

import { signal, derived } from '@rlabs-inc/signals'
import { mount } from '../ts/engine/mount'
import { box, text, each, cycle, Frames } from '../ts/primitives'
import { packColor } from '../ts/bridge/shared-buffer'
import { on, getChar, isPress } from '../ts/state/keyboard'
import type { KeyEvent } from '../ts/state/keyboard'

// =============================================================================
// COLORS
// =============================================================================

const colors = {
  bg: packColor(15, 15, 22, 255),
  bgCard: packColor(25, 25, 35, 255),
  bgNode: packColor(40, 40, 55, 255),
  bgNodeHot: packColor(60, 40, 40, 255),
  border: packColor(70, 70, 100, 255),
  borderActive: packColor(100, 140, 255, 255),
  text: packColor(220, 220, 235, 255),
  textMuted: packColor(120, 120, 150, 255),
  textAccent: packColor(140, 170, 255, 255),
  textSuccess: packColor(100, 220, 140, 255),
  textWarning: packColor(255, 200, 100, 255),
  textError: packColor(255, 120, 120, 255),
}

// Rainbow colors for animated nodes
const rainbowColors = [
  packColor(255, 100, 100, 255),
  packColor(255, 180, 100, 255),
  packColor(255, 255, 100, 255),
  packColor(100, 255, 140, 255),
  packColor(100, 200, 255, 255),
  packColor(180, 140, 255, 255),
]

// =============================================================================
// STATE
// =============================================================================

const nodeCount = signal(100)
const animationActive = signal(true)
const frameCount = signal(0)
const lastFrameTime = signal(performance.now())
const fps = signal(0)
const avgRenderTime = signal(0)
const renderTimes: number[] = []
const maxRenderSamples = 60

// Memory tracking
const memoryUsage = signal('--')

// Generate items for node rendering
const items = derived(() => {
  const count = nodeCount.value
  const result: Array<{ id: string; index: number }> = []
  for (let i = 0; i < count; i++) {
    result.push({ id: `node-${i}`, index: i })
  }
  return result
})

// FPS calculation
let lastFpsUpdate = performance.now()
let framesSinceLastUpdate = 0

function updateFps() {
  const now = performance.now()
  const delta = now - lastFpsUpdate

  if (delta >= 1000) {
    fps.value = Math.round((framesSinceLastUpdate / delta) * 1000)
    framesSinceLastUpdate = 0
    lastFpsUpdate = now

    // Update memory estimate
    const nodeBytes = nodeCount.value * 1024 // ~1KB per node in buffer
    const mb = (nodeBytes / 1024 / 1024).toFixed(2)
    memoryUsage.value = `~${mb} MB`
  }
}

function recordRenderTime(time: number) {
  renderTimes.push(time)
  if (renderTimes.length > maxRenderSamples) {
    renderTimes.shift()
  }
  avgRenderTime.value = renderTimes.reduce((a, b) => a + b, 0) / renderTimes.length
}

function resetStats() {
  renderTimes.length = 0
  avgRenderTime.value = 0
  fps.value = 0
  framesSinceLastUpdate = 0
  lastFpsUpdate = performance.now()
}

// =============================================================================
// MOUNT APP
// =============================================================================

const cols = process.stdout.columns || 120
const rows = process.stdout.rows || 40

mount(() => {
  // ─────────────────────────────────────────────────────────────────────────────
  // KEYBOARD HANDLER
  // ─────────────────────────────────────────────────────────────────────────────
  on((event: KeyEvent) => {
    if (!isPress(event)) return false

    const char = getChar(event)
    if (char === '1') {
      nodeCount.value = 100
      resetStats()
      return true
    }
    if (char === '2') {
      nodeCount.value = 500
      resetStats()
      return true
    }
    if (char === '3') {
      nodeCount.value = 1000
      resetStats()
      return true
    }
    if (char === '4') {
      nodeCount.value = 5000
      resetStats()
      return true
    }
    if (char === ' ') {
      animationActive.value = !animationActive.value
      return true
    }
    if (char === 'r' || char === 'R') {
      resetStats()
      return true
    }

    return false
  })

  // ─────────────────────────────────────────────────────────────────────────────
  // ROOT CONTAINER
  // ─────────────────────────────────────────────────────────────────────────────
  box({
    id: 'root',
    width: cols,
    height: rows,
    flexDirection: 'column',
    bg: colors.bg,
    children: () => {
      // ─────────────────────────────────────────────────────────────────────────
      // HEADER
      // ─────────────────────────────────────────────────────────────────────────
      box({
        id: 'header',
        width: '100%',
        height: 5,
        flexDirection: 'column',
        bg: colors.bgCard,
        border: 1,
        borderColor: colors.border,
        padding: 1,
        children: () => {
          box({
            flexDirection: 'row',
            justifyContent: 'space-between',
            children: () => {
              text({
                content: cycle(['  ', '  ', '  ', '  '], { fps: 4, active: animationActive }).value + 'SparkTUI Node Stress Test',
                fg: colors.textAccent,
              })
              text({
                content: cycle(Frames.dots, { fps: 10, active: animationActive }),
                fg: colors.textSuccess,
              })
            },
          })

          box({
            flexDirection: 'row',
            gap: 4,
            marginTop: 1,
            children: () => {
              // Stats display
              text({
                content: () => `Nodes: ${nodeCount.value.toLocaleString()}`,
                fg: colors.text,
              })
              text({
                content: () => `FPS: ${fps.value}`,
                fg: () => fps.value >= 30 ? colors.textSuccess : fps.value >= 15 ? colors.textWarning : colors.textError,
              })
              text({
                content: () => `Avg Render: ${avgRenderTime.value.toFixed(2)}ms`,
                fg: colors.text,
              })
              text({
                content: () => `Memory: ${memoryUsage.value}`,
                fg: colors.textMuted,
              })
              text({
                content: () => `Animation: ${animationActive.value ? 'ON' : 'OFF'}`,
                fg: () => animationActive.value ? colors.textSuccess : colors.textMuted,
              })
            },
          })
        },
      })

      // ─────────────────────────────────────────────────────────────────────────
      // CONTROLS BAR
      // ─────────────────────────────────────────────────────────────────────────
      box({
        id: 'controls',
        width: '100%',
        height: 1,
        flexDirection: 'row',
        gap: 2,
        paddingLeft: 2,
        bg: colors.bgCard,
        children: () => {
          const nodeCounts = [100, 500, 1000, 5000]
          for (let i = 0; i < nodeCounts.length; i++) {
            const count = nodeCounts[i]!
            const key = i + 1
            box({
              children: () => {
                text({
                  content: `[${key}] ${count.toLocaleString()}`,
                  fg: () => nodeCount.value === count ? colors.textAccent : colors.textMuted,
                })
              },
            })
          }
          text({ content: '[Space] Toggle', fg: colors.textMuted })
          text({ content: '[R] Reset', fg: colors.textMuted })
          text({ content: '[Ctrl+C] Exit', fg: colors.textMuted })
        },
      })

      // ─────────────────────────────────────────────────────────────────────────
      // NODE GRID
      // ─────────────────────────────────────────────────────────────────────────
      box({
        id: 'grid-container',
        grow: 1,
        overflow: 'scroll',
        padding: 1,
        children: () => {
          box({
            id: 'grid',
            flexDirection: 'row',
            flexWrap: 'wrap',
            gap: 0,
            children: () => {
              // Track render time
              const startTime = performance.now()

              each(
                () => items.value,
                (getItem, key) => {
                  const item = getItem()
                  const colorIndex = item.index % rainbowColors.length

                  return box({
                    id: `box-${key}`,
                    width: 3,
                    height: 1,
                    bg: () => {
                      if (!animationActive.value) return colors.bgNode
                      // Animate background based on frame count and index
                      const phase = (frameCount.value + item.index) % rainbowColors.length
                      return rainbowColors[phase]!
                    },
                    children: () => {
                      text({
                        content: () => {
                          // Update frame count for animation
                          if (animationActive.value && item.index === 0) {
                            framesSinceLastUpdate++
                            updateFps()
                          }
                          return '   '
                        },
                        fg: colors.text,
                      })
                    },
                  })
                },
                { key: (item) => item.id }
              )

              // Record render time after each() completes
              const elapsed = performance.now() - startTime
              recordRenderTime(elapsed)
            },
          })
        },
      })

      // ─────────────────────────────────────────────────────────────────────────
      // FOOTER
      // ─────────────────────────────────────────────────────────────────────────
      box({
        id: 'footer',
        width: '100%',
        height: 1,
        bg: colors.bgCard,
        justifyContent: 'center',
        children: () => {
          text({
            content: 'Pure reactive rendering - NO loops, NO polling, NO fixed FPS',
            fg: colors.textMuted,
          })
        },
      })
    },
  })
}, { mode: 'fullscreen' })

// =============================================================================
// ANIMATION DRIVER
// =============================================================================

// Simple animation driver - updates frame count periodically
const animationInterval = setInterval(() => {
  if (animationActive.value) {
    frameCount.value = (frameCount.value + 1) % rainbowColors.length
  }
}, 100) // 10 fps animation

console.log('[stress-test] App mounted - Press 1-4 to change node count')
await new Promise(() => {})
