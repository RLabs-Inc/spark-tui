/**
 * SparkTUI Rapid Updates Test
 *
 * Performance showcase demonstrating efficient selective updates.
 * Shows a grid of cells updating at different rates, proving that
 * SparkTUI's reactive system only updates what actually changes.
 *
 * Key concepts demonstrated:
 * - Different update frequencies (frame-level, 100ms, 500ms, 1s)
 * - Selective reactivity - only changed cells re-render
 * - FPS counter showing smooth performance
 * - Update counters per-zone
 *
 * Controls:
 * - 1/2/3/4: Toggle zone updates
 * - a: Toggle all zones
 * - Ctrl+C: Exit
 */

import { signal, derived } from '@rlabs-inc/signals'
import { mount } from '../ts/engine/mount'
import { box, text, cycle, Frames, show } from '../ts/primitives'
import { packColor } from '../ts/bridge/shared-buffer'
import { on, getChar, isPress } from '../ts/state/keyboard'
import type { KeyEvent } from '../ts/state/keyboard'

// =============================================================================
// COLORS
// =============================================================================

const colors = {
  bg: packColor(15, 15, 22, 255),
  bgCard: packColor(25, 25, 35, 255),
  bgZone: packColor(30, 30, 45, 255),
  border: packColor(70, 70, 100, 255),
  text: packColor(220, 220, 235, 255),
  textMuted: packColor(120, 120, 150, 255),
  textAccent: packColor(140, 170, 255, 255),
  textSuccess: packColor(100, 220, 140, 255),
  textWarning: packColor(255, 200, 100, 255),
  textError: packColor(255, 120, 120, 255),

  // Zone colors (subtle backgrounds)
  zone1: packColor(60, 30, 30, 255),  // Red - fast
  zone2: packColor(60, 50, 30, 255),  // Orange - 100ms
  zone3: packColor(30, 60, 30, 255),  // Green - 500ms
  zone4: packColor(30, 30, 60, 255),  // Blue - 1s

  // Cell colors when active
  cellActive1: packColor(255, 100, 100, 255),
  cellActive2: packColor(255, 180, 100, 255),
  cellActive3: packColor(100, 255, 140, 255),
  cellActive4: packColor(100, 180, 255, 255),
}

// =============================================================================
// STATE
// =============================================================================

// Zone toggles
const zone1Active = signal(true)  // Frame-level updates
const zone2Active = signal(true)  // 100ms updates
const zone3Active = signal(true)  // 500ms updates
const zone4Active = signal(true)  // 1s updates

// Update counters
const zone1Updates = signal(0)
const zone2Updates = signal(0)
const zone3Updates = signal(0)
const zone4Updates = signal(0)

// Cell states (each zone has 16 cells in a 4x4 grid)
const zone1Cells = Array.from({ length: 16 }, () => signal(false))
const zone2Cells = Array.from({ length: 16 }, () => signal(false))
const zone3Cells = Array.from({ length: 16 }, () => signal(false))
const zone4Cells = Array.from({ length: 16 }, () => signal(false))

// Current cell being updated in each zone
const zone1Index = signal(0)
const zone2Index = signal(0)
const zone3Index = signal(0)
const zone4Index = signal(0)

// FPS tracking
const fps = signal(0)
let fpsFrames = 0
let lastFpsTime = performance.now()

function updateFps() {
  fpsFrames++
  const now = performance.now()
  if (now - lastFpsTime >= 1000) {
    fps.value = fpsFrames
    fpsFrames = 0
    lastFpsTime = now
  }
}

// Total updates per second
const totalUpdatesPerSec = derived(() => {
  const z1 = zone1Active.value ? 60 : 0  // ~60 updates/sec
  const z2 = zone2Active.value ? 10 : 0  // 10 updates/sec
  const z3 = zone3Active.value ? 2 : 0   // 2 updates/sec
  const z4 = zone4Active.value ? 1 : 0   // 1 update/sec
  return z1 + z2 + z3 + z4
})

// =============================================================================
// UPDATE LOGIC
// =============================================================================

// Zone 1: Frame-level updates (~60fps)
let zone1Interval: ReturnType<typeof setInterval>
function startZone1() {
  zone1Interval = setInterval(() => {
    if (!zone1Active.value) return
    updateFps()

    // Turn off previous cell
    const prevIdx = zone1Index.value
    zone1Cells[prevIdx]!.value = false

    // Turn on next cell
    const nextIdx = (prevIdx + 1) % 16
    zone1Index.value = nextIdx
    zone1Cells[nextIdx]!.value = true

    zone1Updates.value++
  }, 16) // ~60fps
}

// Zone 2: 100ms updates (10fps)
let zone2Interval: ReturnType<typeof setInterval>
function startZone2() {
  zone2Interval = setInterval(() => {
    if (!zone2Active.value) return

    const prevIdx = zone2Index.value
    zone2Cells[prevIdx]!.value = false

    const nextIdx = (prevIdx + 1) % 16
    zone2Index.value = nextIdx
    zone2Cells[nextIdx]!.value = true

    zone2Updates.value++
  }, 100)
}

// Zone 3: 500ms updates (2fps)
let zone3Interval: ReturnType<typeof setInterval>
function startZone3() {
  zone3Interval = setInterval(() => {
    if (!zone3Active.value) return

    const prevIdx = zone3Index.value
    zone3Cells[prevIdx]!.value = false

    const nextIdx = (prevIdx + 1) % 16
    zone3Index.value = nextIdx
    zone3Cells[nextIdx]!.value = true

    zone3Updates.value++
  }, 500)
}

// Zone 4: 1s updates (1fps)
let zone4Interval: ReturnType<typeof setInterval>
function startZone4() {
  zone4Interval = setInterval(() => {
    if (!zone4Active.value) return

    const prevIdx = zone4Index.value
    zone4Cells[prevIdx]!.value = false

    const nextIdx = (prevIdx + 1) % 16
    zone4Index.value = nextIdx
    zone4Cells[nextIdx]!.value = true

    zone4Updates.value++
  }, 1000)
}

// Start all zones
startZone1()
startZone2()
startZone3()
startZone4()

// =============================================================================
// COMPONENTS
// =============================================================================

function ZoneGrid(props: {
  title: string
  rate: string
  zoneBg: number
  cellActiveColor: number
  cells: ReturnType<typeof signal<boolean>>[]
  updates: ReturnType<typeof signal<number>>
  active: ReturnType<typeof signal<boolean>>
  hotkey: string
}) {
  return box({
    flexDirection: 'column',
    width: 24,
    border: 1,
    borderColor: () => props.active.value ? colors.border : packColor(50, 50, 60, 255),
    bg: props.zoneBg,
    padding: 1,
    children: () => {
      // Header
      box({
        flexDirection: 'row',
        justifyContent: 'space-between',
        marginBottom: 1,
        children: () => {
          text({
            content: `[${props.hotkey}] ${props.title}`,
            fg: () => props.active.value ? colors.text : colors.textMuted,
          })
          text({
            content: props.rate,
            fg: () => props.active.value ? colors.textAccent : colors.textMuted,
          })
        },
      })

      // 4x4 Grid
      box({
        flexDirection: 'column',
        gap: 0,
        children: () => {
          for (let row = 0; row < 4; row++) {
            box({
              flexDirection: 'row',
              gap: 0,
              children: () => {
                for (let col = 0; col < 4; col++) {
                  const idx = row * 4 + col
                  const cell = props.cells[idx]!
                  box({
                    width: 4,
                    height: 2,
                    bg: () => cell.value ? props.cellActiveColor : colors.bgCard,
                    justifyContent: 'center',
                    alignItems: 'center',
                    children: () => {
                      text({
                        content: () => cell.value ? '  ' : '  ',
                        fg: colors.text,
                      })
                    },
                  })
                }
              },
            })
          }
        },
      })

      // Footer - update count
      box({
        marginTop: 1,
        children: () => {
          text({
            content: () => `Updates: ${props.updates.value.toLocaleString()}`,
            fg: colors.textMuted,
          })
        },
      })

      // Status
      box({
        children: () => {
          text({
            content: () => props.active.value ? 'ACTIVE' : 'PAUSED',
            fg: () => props.active.value ? colors.textSuccess : colors.textError,
          })
        },
      })
    },
  })
}

// =============================================================================
// MOUNT APP
// =============================================================================

const cols = process.stdout.columns || 120
const rows = process.stdout.rows || 35

mount(() => {
  // ─────────────────────────────────────────────────────────────────────────────
  // KEYBOARD HANDLER
  // ─────────────────────────────────────────────────────────────────────────────
  on((event: KeyEvent) => {
    if (!isPress(event)) return false

    const char = getChar(event)
    if (char === '1') {
      zone1Active.value = !zone1Active.value
      return true
    }
    if (char === '2') {
      zone2Active.value = !zone2Active.value
      return true
    }
    if (char === '3') {
      zone3Active.value = !zone3Active.value
      return true
    }
    if (char === '4') {
      zone4Active.value = !zone4Active.value
      return true
    }
    if (char === 'a' || char === 'A') {
      const allActive = zone1Active.value && zone2Active.value && zone3Active.value && zone4Active.value
      zone1Active.value = !allActive
      zone2Active.value = !allActive
      zone3Active.value = !allActive
      zone4Active.value = !allActive
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
              box({
                flexDirection: 'row',
                gap: 2,
                children: () => {
                  text({
                    content: cycle(Frames.spinner, { fps: 12 }),
                    fg: colors.textSuccess,
                  })
                  text({ content: 'SparkTUI Rapid Updates Test', fg: colors.textAccent })
                },
              })
              text({
                content: () => `FPS: ${fps.value}`,
                fg: () => fps.value >= 30 ? colors.textSuccess : fps.value >= 15 ? colors.textWarning : colors.textError,
              })
            },
          })

          box({
            flexDirection: 'row',
            gap: 4,
            marginTop: 1,
            children: () => {
              text({
                content: () => `Active Zones: ${[zone1Active, zone2Active, zone3Active, zone4Active].filter(z => z.value).length}/4`,
                fg: colors.text,
              })
              text({
                content: () => `Updates/sec: ~${totalUpdatesPerSec.value}`,
                fg: colors.textAccent,
              })
              text({
                content: () => `Total Updates: ${(zone1Updates.value + zone2Updates.value + zone3Updates.value + zone4Updates.value).toLocaleString()}`,
                fg: colors.textMuted,
              })
            },
          })
        },
      })

      // ─────────────────────────────────────────────────────────────────────────
      // MAIN CONTENT
      // ─────────────────────────────────────────────────────────────────────────
      box({
        grow: 1,
        flexDirection: 'column',
        padding: 2,
        children: () => {
          // Description
          box({
            marginBottom: 2,
            children: () => {
              text({
                content: 'Each zone updates at a different rate. Only changed cells trigger re-renders.',
                fg: colors.textMuted,
              })
            },
          })

          // Four zones in a row
          box({
            flexDirection: 'row',
            gap: 2,
            justifyContent: 'center',
            children: () => {
              ZoneGrid({
                title: 'Zone 1',
                rate: '~60fps',
                zoneBg: colors.zone1,
                cellActiveColor: colors.cellActive1,
                cells: zone1Cells,
                updates: zone1Updates,
                active: zone1Active,
                hotkey: '1',
              })

              ZoneGrid({
                title: 'Zone 2',
                rate: '100ms',
                zoneBg: colors.zone2,
                cellActiveColor: colors.cellActive2,
                cells: zone2Cells,
                updates: zone2Updates,
                active: zone2Active,
                hotkey: '2',
              })

              ZoneGrid({
                title: 'Zone 3',
                rate: '500ms',
                zoneBg: colors.zone3,
                cellActiveColor: colors.cellActive3,
                cells: zone3Cells,
                updates: zone3Updates,
                active: zone3Active,
                hotkey: '3',
              })

              ZoneGrid({
                title: 'Zone 4',
                rate: '1s',
                zoneBg: colors.zone4,
                cellActiveColor: colors.cellActive4,
                cells: zone4Cells,
                updates: zone4Updates,
                active: zone4Active,
                hotkey: '4',
              })
            },
          })

          // Explanation
          box({
            marginTop: 2,
            flexDirection: 'column',
            children: () => {
              text({
                content: 'Selective Updates Demo:',
                fg: colors.textAccent,
              })
              text({
                content: '- Zone 1 updates ~60 cells/sec but only ONE cell changes per frame',
                fg: colors.textMuted,
              })
              text({
                content: '- Zones 2-4 update progressively slower',
                fg: colors.textMuted,
              })
              text({
                content: '- SparkTUI renders ONLY the changed cells - no full redraws',
                fg: colors.textMuted,
              })
            },
          })
        },
      })

      // ─────────────────────────────────────────────────────────────────────────
      // CONTROLS
      // ─────────────────────────────────────────────────────────────────────────
      box({
        id: 'controls',
        width: '100%',
        height: 1,
        bg: colors.bgCard,
        flexDirection: 'row',
        justifyContent: 'center',
        gap: 3,
        children: () => {
          text({ content: '[1-4] Toggle zones', fg: colors.textMuted })
          text({ content: '[A] Toggle all', fg: colors.textMuted })
          text({ content: '[Ctrl+C] Exit', fg: colors.textMuted })
        },
      })
    },
  })
}, { mode: 'fullscreen' })

console.log('[rapid-updates] App mounted - Press 1-4 to toggle zones')
await new Promise(() => {})
