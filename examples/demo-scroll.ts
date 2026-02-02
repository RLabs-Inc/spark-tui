/**
 * SparkTUI - Scroll Demo
 *
 * Comprehensive scroll testing demonstrating:
 * 1. Simple vertical scroll - Long content in a fixed-height container
 * 2. Horizontal scroll - Wide content in a fixed-width container
 * 3. Both directions - Content larger than container in both axes
 * 4. Nested scrolls - Scrollable container inside another scrollable
 * 5. Scroll chaining - Inner scroll chains to outer when at limit
 * 6. Keyboard scroll - Arrow keys, Page Up/Down, Home/End
 * 7. Scroll position display - Show current scroll offset
 *
 * This is PURELY REACTIVE. No render loops, no polling.
 * Signals update UI automatically through the reactive graph.
 *
 * Run: bun run examples/demo-scroll.ts
 */

import { ptr } from 'bun:ffi'
import { signal, derived, effect, effectScope } from '@rlabs-inc/signals'
import type { WritableSignal } from '@rlabs-inc/signals'
import { loadEngine } from '../ts/bridge/ffi'
import { box } from '../ts/primitives/box'
import { text } from '../ts/primitives/text'
import { each } from '../ts/primitives/each'
import { show } from '../ts/primitives/show'
import { scoped, onCleanup } from '../ts/primitives/scope'
import { pulse, Frames } from '../ts/primitives/animation'
import {
  setTerminalSize,
  setNodeCount,
  packColor,
} from '../ts/bridge/shared-buffer'
import type { RGBA } from '../ts/types'

// =============================================================================
// COLORS - Semantic palette
// =============================================================================

function rgba(r: number, g: number, b: number, a: number = 255): RGBA {
  return { r, g, b, a }
}

const colors = {
  // Backgrounds
  bgDark: rgba(18, 18, 24),
  bgCard: rgba(28, 28, 38),
  bgCardHover: rgba(35, 35, 48),
  bgScrollbar: rgba(45, 45, 60),
  bgScrollThumb: rgba(80, 80, 100),

  // Text
  textPrimary: rgba(240, 240, 250),
  textSecondary: rgba(160, 160, 180),
  textMuted: rgba(100, 100, 120),
  textDim: rgba(70, 70, 90),

  // Accent colors
  cyan: rgba(80, 200, 255),
  green: rgba(80, 220, 140),
  yellow: rgba(255, 200, 80),
  red: rgba(255, 100, 100),
  orange: rgba(255, 150, 80),
  purple: rgba(180, 130, 255),
  blue: rgba(100, 150, 255),

  // Border
  borderDim: rgba(50, 50, 70),
  borderAccent: rgba(80, 140, 200),
  borderFocus: rgba(100, 180, 255),
}

// =============================================================================
// SCROLL STATE
// =============================================================================

// Track scroll positions for different containers
// Note: In the final implementation, these would be read reactively from SharedBuffer
// via computed output arrays. For now, we track via onScroll callbacks.
const verticalScrollY = signal(0)
const horizontalScrollX = signal(0)
const bothScrollX = signal(0)
const bothScrollY = signal(0)
const outerScrollY = signal(0)
const inner1ScrollY = signal(0)
const inner2ScrollY = signal(0)

// Accumulated scroll deltas (since scroll events give deltas, not absolute positions)
let verticalScrollAccum = 0
let horizontalScrollAccum = 0
let bothScrollXAccum = 0
let bothScrollYAccum = 0
let outerScrollAccum = 0
let inner1ScrollAccum = 0
let inner2ScrollAccum = 0

// Which container is focused for keyboard control
type FocusedContainer =
  | 'vertical'
  | 'horizontal'
  | 'both'
  | 'outer'
  | 'inner1'
  | 'inner2'
  | null
const focusedContainer = signal<FocusedContainer>('vertical')

// Render mode toggle
const renderMode = signal<'fullscreen' | 'inline'>('fullscreen')

// =============================================================================
// DATA GENERATION
// =============================================================================

/** Generate lines of content for vertical scroll */
function generateLines(count: number): { id: string; content: string }[] {
  return Array.from({ length: count }, (_, i) => ({
    id: `line-${i}`,
    content: `Line ${i + 1}: ${getLineContent(i)}`,
  }))
}

/** Get varied content for each line */
function getLineContent(index: number): string {
  const contents = [
    'The quick brown fox jumps over the lazy dog.',
    'Lorem ipsum dolor sit amet, consectetur adipiscing.',
    'Reactive programming without the complexity.',
    'Zero-copy shared memory between TS and Rust.',
    'Purely reactive - no loops, no polling.',
    'SparkTUI: All Rust benefits without borrowing a single thing.',
    'Flexbox layout powered by Taffy.',
    'Unicode-aware text measurement.',
    'Signals and deriveds form a dependency graph.',
    'Changes propagate through it instantly.',
  ]
  return contents[index % contents.length]!
}

/** Generate wide content for horizontal scroll */
function generateWideContent(): string {
  return Array.from(
    { length: 20 },
    (_, i) => `[Block ${i + 1}]`
  ).join(' --- ')
}

// Content data
const verticalContent = generateLines(50)
const horizontalContent = generateWideContent()
const bothContent = generateLines(30).map(line => ({
  ...line,
  content: line.content + ' '.repeat(20) + '[Extended horizontal content here]',
}))
const outerContent = generateLines(40)
const inner1Content = generateLines(20)
const inner2Content = generateLines(20)

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/** Format scroll position display */
function formatScroll(x: number, y: number): string {
  return `(${Math.round(x)}, ${Math.round(y)})`
}

/** Get border color based on focus state */
function borderColor(container: FocusedContainer): RGBA {
  return focusedContainer.value === container
    ? colors.borderFocus
    : colors.borderDim
}

// =============================================================================
// SCROLL CONTAINER COMPONENTS
// =============================================================================

/**
 * Vertical Scroll Demo
 * Fixed height container with many lines of content
 */
function VerticalScrollDemo() {
  box({
    width: 35,
    height: 10,
    flexDirection: 'column',
    children: () => {
      // Header
      text({ content: 'Vertical Scroll', fg: colors.cyan })

      // Scrollable container
      box({
        width: '100%',
        grow: 1,
        overflow: 'scroll',
        border: 1,
        borderColor: derived(() => borderColor('vertical')),
        focusable: true,
        onFocus: () => {
          focusedContainer.value = 'vertical'
        },
        onScroll: (e: any) => {
          // ScrollEvent has deltaX/deltaY - accumulate to get position
          verticalScrollAccum = Math.max(0, verticalScrollAccum + (e.deltaY ?? 0))
          verticalScrollY.value = verticalScrollAccum
        },
        children: () => {
          each(
            () => verticalContent,
            (getItem) => {
              text({
                content: () => getItem().content,
                fg: colors.textSecondary,
              })
              return () => { }
            },
            { key: item => item.id }
          )
        },
      })

      // Scroll position indicator
      text({
        content: derived(() => `Scroll Y: ${Math.round(verticalScrollY.value)}`),
        fg: colors.textMuted,
      })
    },
  })
}

/**
 * Horizontal Scroll Demo
 * Fixed width container with wide content
 */
function HorizontalScrollDemo() {
  box({
    width: 35,
    height: 6,
    flexDirection: 'column',
    children: () => {
      // Header
      text({ content: 'Horizontal Scroll', fg: colors.purple })

      // Scrollable container
      box({
        width: '100%',
        height: 3,
        overflow: 'scroll',
        border: 1,
        borderColor: derived(() => borderColor('horizontal')),
        focusable: true,
        onFocus: () => {
          focusedContainer.value = 'horizontal'
        },
        onScroll: (e: any) => {
          // ScrollEvent has deltaX/deltaY - accumulate to get position
          horizontalScrollAccum = Math.max(0, horizontalScrollAccum + (e.deltaX ?? 0))
          horizontalScrollX.value = horizontalScrollAccum
        },
        children: () => {
          // Single wide line - no wrap
          text({
            content: horizontalContent,
            fg: colors.textSecondary,
          })
        },
      })

      // Scroll position indicator
      text({
        content: derived(() => `Scroll X: ${Math.round(horizontalScrollX.value)}`),
        fg: colors.textMuted,
      })
    },
  })
}

/**
 * Both Directions Scroll Demo
 * Content larger than container in both axes
 */
function BothDirectionsDemo() {
  box({
    width: 35,
    height: 10,
    flexDirection: 'column',
    children: () => {
      // Header
      text({ content: 'Both Directions', fg: colors.green })

      // Scrollable container
      box({
        width: '100%',
        grow: 1,
        overflow: 'scroll',
        border: 1,
        borderColor: derived(() => borderColor('both')),
        focusable: true,
        onFocus: () => {
          focusedContainer.value = 'both'
        },
        onScroll: (e: any) => {
          // ScrollEvent has deltaX/deltaY - accumulate to get position
          bothScrollXAccum = Math.max(0, bothScrollXAccum + (e.deltaX ?? 0))
          bothScrollYAccum = Math.max(0, bothScrollYAccum + (e.deltaY ?? 0))
          bothScrollX.value = bothScrollXAccum
          bothScrollY.value = bothScrollYAccum
        },
        children: () => {
          each(
            () => bothContent,
            (getItem) => {
              text({
                content: () => getItem().content,
                fg: colors.textSecondary,
              })
              return () => { }
            },
            { key: item => item.id }
          )
        },
      })

      // Scroll position indicator
      text({
        content: derived(() =>
          `Scroll: ${formatScroll(bothScrollX.value, bothScrollY.value)}`
        ),
        fg: colors.textMuted,
      })
    },
  })
}

/**
 * Nested Scrolls Demo
 * Demonstrates scroll chaining and nested containers
 */
function NestedScrollsDemo() {
  const cols = process.stdout.columns || 80

  box({
    width: cols - 4,
    height: 18,
    flexDirection: 'column',
    children: () => {
      // Header
      box({
        flexDirection: 'row',
        justifyContent: 'space-between',
        children: () => {
          text({ content: 'Nested Scrolls (with chaining)', fg: colors.orange })
          text({
            content: derived(() => `Outer Y: ${Math.round(outerScrollY.value)}`),
            fg: colors.textMuted,
          })
        },
      })

      // Outer scrollable container
      box({
        width: '100%',
        grow: 1,
        overflow: 'scroll',
        border: 1,
        borderColor: derived(() => borderColor('outer')),
        focusable: true,
        onFocus: () => {
          focusedContainer.value = 'outer'
        },
        onScroll: (e: any) => {
          // ScrollEvent has deltaX/deltaY - accumulate to get position
          outerScrollAccum = Math.max(0, outerScrollAccum + (e.deltaY ?? 0))
          outerScrollY.value = outerScrollAccum
        },
        padding: 1,
        flexDirection: 'column',
        gap: 1,
        children: () => {
          // Intro text
          text({
            content: 'Outer container - scroll me! Inner containers scroll first.',
            fg: colors.textPrimary,
          })
          text({
            content: 'When inner hits limit, scroll chains to outer.',
            fg: colors.textSecondary,
          })

          // Inner scroll 1
          box({
            width: '95%',
            height: 5,
            overflow: 'scroll',
            border: 1,
            borderColor: derived(() => borderColor('inner1')),
            focusable: true,
            marginTop: 1,
            onFocus: () => {
              focusedContainer.value = 'inner1'
            },
            onScroll: (e: any) => {
              // ScrollEvent has deltaX/deltaY - accumulate to get position
              inner1ScrollAccum = Math.max(0, inner1ScrollAccum + (e.deltaY ?? 0))
              inner1ScrollY.value = inner1ScrollAccum
            },
            children: () => {
              text({
                content: 'Inner Scroll 1',
                fg: colors.cyan,
              })
              each(
                () => inner1Content,
                (getItem) => {
                  text({
                    content: () => `  ${getItem().content}`,
                    fg: colors.textSecondary,
                  })
                  return () => { }
                },
                { key: item => item.id }
              )
            },
          })

          // Inner scroll 1 position
          text({
            content: derived(() =>
              `  Inner 1 scroll Y: ${Math.round(inner1ScrollY.value)}`
            ),
            fg: colors.textDim,
          })

          // Inner scroll 2
          box({
            width: '95%',
            height: 5,
            overflow: 'scroll',
            border: 1,
            borderColor: derived(() => borderColor('inner2')),
            focusable: true,
            marginTop: 1,
            onFocus: () => {
              focusedContainer.value = 'inner2'
            },
            onScroll: (e: any) => {
              // ScrollEvent has deltaX/deltaY - accumulate to get position
              inner2ScrollAccum = Math.max(0, inner2ScrollAccum + (e.deltaY ?? 0))
              inner2ScrollY.value = inner2ScrollAccum
            },
            children: () => {
              text({
                content: 'Inner Scroll 2',
                fg: colors.purple,
              })
              each(
                () => inner2Content,
                (getItem) => {
                  text({
                    content: () => `  ${getItem().content}`,
                    fg: colors.textSecondary,
                  })
                  return () => { }
                },
                { key: item => item.id }
              )
            },
          })

          // Inner scroll 2 position
          text({
            content: derived(() =>
              `  Inner 2 scroll Y: ${Math.round(inner2ScrollY.value)}`
            ),
            fg: colors.textDim,
          })

          // More outer content that forces outer scroll
          text({ content: '', fg: colors.textMuted })
          text({
            content: '--- More outer content below ---',
            fg: colors.textSecondary,
          })
          each(
            () => outerContent,
            (getItem) => {
              text({
                content: () => getItem().content,
                fg: colors.textMuted,
              })
              return () => { }
            },
            { key: item => item.id }
          )
        },
      })
    },
  })
}

// =============================================================================
// MAIN DASHBOARD
// =============================================================================

function ScrollDemoApp() {
  return scoped(() => {
    const cols = process.stdout.columns || 80
    const rows = process.stdout.rows || 24

    // Pulsing indicator for focused container
    const focusPulse = pulse({ fps: 2 })

    // Root container
    box({
      width: cols,
      height: rows,
      flexDirection: 'column',
      bg: colors.bgDark,
      children: () => {
        // ===== HEADER =====
        box({
          width: '100%',
          height: 3,
          flexDirection: 'row',
          justifyContent: 'space-between',
          alignItems: 'center',
          border: 1,
          borderColor: colors.borderAccent,
          bg: colors.bgCard,
          paddingLeft: 2,
          paddingRight: 2,
          children: () => {
            // Title
            text({ content: 'SparkTUI Scroll Demo', fg: colors.cyan })

            // Mode and focused container
            box({
              flexDirection: 'row',
              gap: 3,
              alignItems: 'center',
              children: () => {
                // Focused container indicator
                text({
                  content: derived(() => {
                    const f = focusedContainer.value
                    return f ? `Focus: ${f}` : 'No focus'
                  }),
                  fg: derived(() =>
                    focusPulse.value ? colors.green : colors.textMuted
                  ),
                })

                // Mode toggle (clickable)
                box({
                  border: 1,
                  borderColor: colors.borderDim,
                  paddingLeft: 1,
                  paddingRight: 1,
                  focusable: true,
                  onClick: () => {
                    renderMode.value =
                      renderMode.value === 'fullscreen' ? 'inline' : 'fullscreen'
                  },
                  children: () => {
                    text({
                      content: derived(() => `Mode: [${renderMode.value}]`),
                      fg: colors.textSecondary,
                    })
                  },
                })
              },
            })
          },
        })

        // ===== TOP ROW: Vertical + Horizontal + Both =====
        box({
          width: '100%',
          flexDirection: 'row',
          gap: 2,
          padding: 1,
          children: () => {
            // Vertical scroll demo
            VerticalScrollDemo()

            // Horizontal scroll demo (shorter)
            box({
              flexDirection: 'column',
              gap: 1,
              children: () => {
                HorizontalScrollDemo()

                // Stats box
                box({
                  width: 35,
                  height: 3,
                  border: 1,
                  borderColor: colors.borderDim,
                  bg: colors.bgCard,
                  padding: 1,
                  flexDirection: 'column',
                  children: () => {
                    text({
                      content: 'Scroll Statistics',
                      fg: colors.yellow,
                    })
                    text({
                      content: derived(() => {
                        const total =
                          verticalScrollY.value +
                          horizontalScrollX.value +
                          bothScrollX.value +
                          bothScrollY.value +
                          outerScrollY.value +
                          inner1ScrollY.value +
                          inner2ScrollY.value
                        return `Total scroll: ${Math.round(total)}px`
                      }),
                      fg: colors.textMuted,
                    })
                  },
                })
              },
            })

            // Both directions demo
            BothDirectionsDemo()
          },
        })

        // ===== NESTED SCROLLS ROW =====
        box({
          width: '100%',
          paddingLeft: 1,
          paddingRight: 1,
          children: () => {
            NestedScrollsDemo()
          },
        })

        // ===== FOOTER: Keyboard shortcuts =====
        box({
          width: '100%',
          height: 2,
          justifyContent: 'center',
          alignItems: 'center',
          borderTop: 1,
          borderColor: colors.borderDim,
          marginTop: 1,
          children: () => {
            text({
              content:
                '[Arrow Keys] Scroll  [PgUp/PgDn] Page  [Home/End] Jump  [Tab] Next Container  [Ctrl+C] Exit',
              fg: colors.textDim,
            })
          },
        })
      },
    })

    // Log state changes for debugging
    effect(() => {
      const focus = focusedContainer.value
      console.log(`[Focus] Container: ${focus ?? 'none'}`)
    })
  })
}

// =============================================================================
// MAIN
// =============================================================================

console.log('Initializing SparkTUI Scroll Demo...\n')

// Initialize the AoS bridge
const { buffer } = initBridgeAoS()

// Set terminal size
const cols = process.stdout.columns || 80
const rows = process.stdout.rows || 24
setTerminalSize(buffer, cols, rows)

// Build the UI tree
const cleanup = ScrollDemoApp()

// Set node count for Rust (upper bound estimate)
setNodeCount(buffer, 500)

// Load and initialize the Rust engine
const engine = loadEngine()
const result = engine.init(ptr(buffer.buffer), buffer.buffer.byteLength)

if (result !== 0) {
  console.error(`Engine init failed: ${result}`)
  cleanup()
  process.exit(1)
}

// Handle terminal resize
process.stdout.on('resize', () => {
  const newCols = process.stdout.columns || 80
  const newRows = process.stdout.rows || 24
  setTerminalSize(buffer, newCols, newRows)
})

console.log('Scroll demo initialized!')
console.log(`Terminal size: ${cols}x${rows}`)
console.log('')
console.log('Scroll Behaviors to Test:')
console.log('  - overflow: "scroll" makes containers scrollable')
console.log('  - focusable: true allows keyboard control when focused')
console.log('  - onScroll callback receives scroll position updates')
console.log('  - Arrow keys: scroll by 1 line')
console.log('  - Page Up/Down: scroll by page')
console.log('  - Home/End: jump to start/end')
console.log('  - Mouse wheel: scroll (if mouse enabled)')
console.log('  - Nested scrolling: inner containers scroll first, then chain to outer')
console.log('')

// Keep process alive - Rust engine handles stdin/Ctrl+C
await new Promise(() => { })
