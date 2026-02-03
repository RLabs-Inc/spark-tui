/**
 * Test: Auto-scroll Detection Fix
 *
 * Tests AUTO-DETECTION of scrollable containers - NO explicit overflow settings.
 * The framework should automatically detect when children > parent and enable scroll.
 *
 * Test Cases:
 * 1. Simple overflow - 20 lines in 5-line container
 * 2. Nested scroll - outer has overflow, inner also has overflow (both should scroll independently)
 * 3. Mixed levels - scroll, no-scroll, scroll intercalated
 * 4. Sequential scrolls - two scrollable siblings
 * 5. Deep nesting - 4 levels, only deepest overflows
 *
 * NO overflow: 'auto' or overflow: 'scroll' anywhere!
 * The fix should auto-detect based on: children_computed_size > parent_computed_size
 *
 * Run: bun run examples/test-scroll-fix.ts
 */

import { mount } from '../ts/engine'
import { box, text, each } from '../ts/primitives'
import { packColor } from '../ts/bridge/shared-buffer'
import { getChar } from '../ts/engine/events'

const colors = {
  bg: packColor(18, 18, 24, 255),
  border: packColor(60, 60, 80, 255),
  text: packColor(200, 200, 220, 255),
  textMuted: packColor(120, 120, 140, 255),
  accent1: packColor(100, 150, 255, 255),
  accent2: packColor(150, 255, 150, 255),
  accent3: packColor(255, 150, 100, 255),
  accent4: packColor(255, 100, 255, 255),
}

// Generate content lines
function lines(count: number, prefix: string) {
  return Array.from({ length: count }, (_, i) => ({
    id: `${prefix}-${i}`,
    text: `${prefix} line ${i + 1}`,
  }))
}

function App() {
  box({
    width: '100%',
    height: '100%',
    bg: colors.bg,
    flexDirection: 'column',
    gap: 1,
    padding: 1,
    children: () => {
      // Title
      text({ content: 'Auto-Scroll Detection Test (NO explicit overflow settings)', fg: colors.accent1 })
      text({ content: 'All scroll should be AUTO-DETECTED from content overflow', fg: colors.textMuted })

      // Main content row
      box({
        width: '100%',
        grow: 1,
        flexDirection: 'row',
        gap: 2,
        children: () => {

          // ============================================
          // TEST 1: Simple overflow
          // Container height=6, content=15 lines
          // Expected: container becomes scrollable
          // ============================================
          box({
            width: 25,
            height: 8,
            border: 1,
            borderColor: colors.accent1,
            flexDirection: 'column',
            children: () => {
              text({ content: 'TEST 1: Simple', fg: colors.accent1 })
              text({ content: 'h=6, 15 lines', fg: colors.textMuted })

              // This box should auto-scroll (15 lines in 4-line space)
              box({
                width: '100%',
                height: 4,
                flexDirection: 'column',
                children: () => {
                  each(
                    () => lines(15, 'A'),
                    (getItem) => {
                      text({ content: () => getItem().text, fg: colors.text })
                      return () => { }
                    },
                    { key: item => item.id }
                  )
                },
              })
            },
          })

          // ============================================
          // TEST 2: Nested scroll
          // Outer: height=10, has 20 lines total
          // Inner: height=4, has 10 lines
          // Expected: BOTH should scroll independently
          // ============================================
          box({
            width: 28,
            height: 12,
            border: 1,
            borderColor: colors.accent2,
            flexDirection: 'column',
            children: () => {
              text({ content: 'TEST 2: Nested', fg: colors.accent2 })

              // Outer scroll area (height ~8 after header)
              box({
                width: '100%',
                height: 8,
                flexDirection: 'column',
                children: () => {
                  text({ content: 'Outer start', fg: colors.textMuted })

                  // Inner scroll area (height=3, 10 lines)
                  box({
                    width: '90%',
                    height: 3,
                    border: 1,
                    borderColor: colors.accent2,
                    flexDirection: 'column',
                    children: () => {
                      each(
                        () => lines(10, 'Inner'),
                        (getItem) => {
                          text({ content: () => getItem().text, fg: colors.text })
                          return () => { }
                        },
                        { key: item => item.id }
                      )
                    },
                  })

                  // More outer content to make outer scroll
                  each(
                    () => lines(12, 'Outer'),
                    (getItem) => {
                      text({ content: () => getItem().text, fg: colors.textMuted })
                      return () => { }
                    },
                    { key: item => item.id }
                  )
                },
              })
            },
          })

          // ============================================
          // TEST 3: Mixed levels (scroll-noscroll-scroll)
          // Level 1: scroll (lots of content)
          //   Level 2: NO scroll (content fits)
          //     Level 3: scroll (content overflows)
          // ============================================
          box({
            width: 28,
            height: 12,
            border: 1,
            borderColor: colors.accent3,
            flexDirection: 'column',
            children: () => {
              text({ content: 'TEST 3: Mixed', fg: colors.accent3 })

              // Level 1 - should scroll
              box({
                width: '100%',
                height: 9,
                flexDirection: 'column',
                children: () => {
                  text({ content: 'L1 (scroll)', fg: colors.textMuted })

                  // Level 2 - should NOT scroll (content fits)
                  box({
                    width: '95%',
                    height: 6,
                    border: 1,
                    borderColor: colors.textMuted,
                    flexDirection: 'column',
                    children: () => {
                      text({ content: 'L2 (no scroll)', fg: colors.textMuted })

                      // Level 3 - should scroll
                      box({
                        width: '90%',
                        height: 3,
                        border: 1,
                        borderColor: colors.accent3,
                        flexDirection: 'column',
                        children: () => {
                          each(
                            () => lines(8, 'L3'),
                            (getItem) => {
                              text({ content: () => getItem().text, fg: colors.text })
                              return () => { }
                            },
                            { key: item => item.id }
                          )
                        },
                      })
                    },
                  })

                  // More L1 content to trigger scroll
                  each(
                    () => lines(10, 'L1'),
                    (getItem) => {
                      text({ content: () => getItem().text, fg: colors.textMuted })
                      return () => { }
                    },
                    { key: item => item.id }
                  )
                },
              })
            },
          })

          // ============================================
          // TEST 4: Sequential scrolls (siblings)
          // Two scrollable boxes side by side
          // ============================================
          box({
            width: 20,
            height: 12,
            border: 1,
            borderColor: colors.accent4,
            flexDirection: 'column',
            children: () => {
              text({ content: 'TEST 4: Sibling', fg: colors.accent4 })

              // Two sibling scroll containers
              box({
                width: '100%',
                grow: 1,
                flexDirection: 'row',
                gap: 1,
                children: () => {
                  // Left sibling - scrollable
                  box({
                    width: '50%',
                    height: 8,
                    border: 1,
                    borderColor: colors.textMuted,
                    flexDirection: 'column',
                    children: () => {
                      each(
                        () => lines(12, 'L'),
                        (getItem) => {
                          text({ content: () => getItem().text, fg: colors.text })
                          return () => { }
                        },
                        { key: item => item.id }
                      )
                    },
                  })

                  // Right sibling - scrollable
                  box({
                    width: '50%',
                    height: 8,
                    border: 1,
                    borderColor: colors.textMuted,
                    flexDirection: 'column',
                    children: () => {
                      each(
                        () => lines(12, 'R'),
                        (getItem) => {
                          text({ content: () => getItem().text, fg: colors.text })
                          return () => { }
                        },
                        { key: item => item.id }
                      )
                    },
                  })
                },
              })
            },
          })

        },
      })

      // Footer
      text({ content: '[Tab] Focus  [Arrows] Scroll  [Ctrl+C] Exit', fg: colors.textMuted })
    },
  })
}

await mount(App)
