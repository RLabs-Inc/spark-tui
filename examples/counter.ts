/**
 * SparkTUI — Beautiful Counter
 *
 * The classic demo app showcasing:
 * - Reactive state with signals
 * - Flexbox layout (centered, column, row)
 * - Styled components with variants
 * - Focus & keyboard navigation
 * - Click handlers
 *
 * Controls:
 *   + or =    Increment
 *   - or _    Decrement
 *   Tab       Navigate focus
 *   Enter     Activate focused button
 *   t         Cycle theme
 *   d         Toggle debug stats
 *   q         Quit
 *   Ctrl+C    Quit
 */

import { signal, derived } from '@rlabs-inc/signals'
import { box, text } from '../ts/primitives'
import { t, themes, setTheme, getThemeNames } from '../ts/state/theme'
import { mount } from '../ts/engine'
import { getBuffer } from '../ts/bridge'
import {
  getRenderCount,
  getLayoutCount,
  getLayoutTimeUs,
  getFramebufferTimeUs,
  getRenderTimeUs,
  getTotalFrameTimeUs,
  getTsNotifyCount,
  getWakeCount,
  getWakeLatencyUs,
  getEventWriteCount,
} from '../ts/bridge/shared-buffer'
import { isEnter, isSpace, getChar } from '../ts/engine/events'

// =============================================================================
// REACTIVE STATE
// =============================================================================

const count = signal(0)
const themeNames = getThemeNames()
const themeIndex = signal(0)
const currentThemeName = derived(() => themeNames[themeIndex.value])

// Debug panel state
const showDebug = signal(false)
const debugTick = signal(0) // Force refresh of debug values

// =============================================================================
// APP
// =============================================================================

await mount(() => {
  // Root: Full terminal, centered
  box({
    flexDirection: 'column',
    children: () => {
      // Card container
      box({
        width: '100%',
        flexDirection: 'column',
        alignItems: 'center',
        padding: 2,
        paddingBottom: 0,
        gap: 2,
        border: 1,
        borderColor: t.secondary,
        children: () => {
          // Title
          text({
            content: '✨ SparkTUI Counter ✨',
            fg: t.primary,
          })

          // Counter row: [ - ]  42  [ + ]
          box({
            width: '100%',
            flexDirection: 'row',
            alignItems: 'center',
            justifyContent: 'space-between',
            gap: 2,
            children: () => {
              // Minus button
              box({
                width: 7,
                height: 3,
                justifyContent: 'center',
                alignItems: 'center',
                bg: t.error,
                fg: t.text,
                focusable: true,
                onClick: () => {
                  count.value = count.value - 1
                },
                onKey: (key) => {
                  if (isEnter(key) || isSpace(key)) {
                    count.value = count.value - 1
                    return true
                  }
                },
                children: () => {
                  text({ content: '  -  ', fg: t.text })
                },
              })

              // Count display
              box({
                justifyContent: 'center',
                alignItems: 'center',
                children: () => {
                  text({
                    content: count,
                    fg: t.text,
                  })
                },
              })

              // Plus button
              box({
                width: 7,
                height: 3,
                justifyContent: 'center',
                alignItems: 'center',
                bg: t.success,
                fg: t.text,
                focusable: true,
                onClick: () => {
                  count.value = count.value + 1
                },
                onKey: (key) => {
                  if (isEnter(key) || isSpace(key)) {
                    count.value = count.value + 1
                    return true
                  }
                },
                children: () => {
                  text({ content: '  +  ', fg: t.text })
                },
              })
            },
          })

          // Footer
          box({
            flexDirection: 'column',
            justifyContent: 'center',
            children: () => {
              text({
                content: '+/- count  q quit  d debug',
                fg: t.textMuted,
              })
              text({
                content: () => `t theme: ${currentThemeName.value}`,
                fg: t.textMuted,
              })
            },
          })

          // Debug panel
          box({
            flexDirection: 'column',
            padding: 1,
            border: 1,
            borderColor: t.warning,
            children: () => {
              text({ content: '📊 Timing Stats (d to refresh)', fg: t.warning })
              text({
                content: () => {
                  const _ = debugTick.value // React to refresh
                  const buf = getBuffer()
                  const r = String(getRenderCount(buf)).padStart(6)
                  const l = String(getLayoutCount(buf)).padStart(6)
                  return `Renders:${r}  Layouts:${l}`
                },
                fg: t.text,
              })
              text({
                content: () => {
                  const _ = debugTick.value
                  const buf = getBuffer()
                  const lt = String(getLayoutTimeUs(buf)).padStart(6)
                  const fb = String(getFramebufferTimeUs(buf)).padStart(6)
                  return `Layout:${lt}μs  FB:${fb}μs`
                },
                fg: t.text,
              })
              text({
                content: () => {
                  const _ = debugTick.value
                  const buf = getBuffer()
                  const rt = String(getRenderTimeUs(buf)).padStart(6)
                  const tot = String(getTotalFrameTimeUs(buf)).padStart(6)
                  return `Render:${rt}μs  Total:${tot}μs`
                },
                fg: t.text,
              })
              // New instrumentation metrics
              text({
                content: () => {
                  const _ = debugTick.value
                  const buf = getBuffer()
                  const notify = String(getTsNotifyCount(buf)).padStart(6)
                  const wake = String(getWakeCount(buf)).padStart(6)
                  return `Notify:${notify}  Wakes:${wake}`
                },
                fg: t.text,
              })
              text({
                content: () => {
                  const _ = debugTick.value
                  const buf = getBuffer()
                  const latency = String(getWakeLatencyUs(buf)).padStart(6)
                  const events = String(getEventWriteCount(buf)).padStart(6)
                  return `WakeLat:${latency}μs  Events:${events}`
                },
                fg: t.text,
              })
            },
          })
        },
      })
    },

    // Global keyboard handler
    onKey: (key) => {
      const ch = getChar(key)
      if (ch === '+' || ch === '=') {
        count.value = count.value + 1
        return true
      }
      if (ch === '-' || ch === '_') {
        count.value = count.value - 1
        return true
      }
      if (ch === 'q' || ch === 'Q') {
        process.exit(0)
      }
      if (ch === 't' || ch === 'T') {
        themeIndex.value = (themeIndex.value + 1) % themeNames.length
        setTheme(themeNames[themeIndex.value] as keyof typeof themes)
        return true
      }
      if (ch === 'd' || ch === 'D') {
        showDebug.value = !showDebug.value
        debugTick.value++ // Force refresh timing values
        return true
      }
    },
  })
}, { mode: 'inline' })
