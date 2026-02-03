/**
 * Layout test - debugging 100% width and flex-wrap behavior
 */
import { signal } from '@rlabs-inc/signals'
import { mount } from '../ts/engine/mount'
import { box, text } from '../ts/primitives'
import { packColor } from '../ts/bridge/shared-buffer'

const red = packColor(255, 100, 100, 255)
const green = packColor(100, 255, 100, 255)
const blue = packColor(100, 100, 255, 255)
const yellow = packColor(255, 255, 100, 255)
const bg = packColor(30, 30, 40, 255)
const border = packColor(100, 100, 100, 255)

mount(() => {
  box({
    id: 'root',
    width: '100%',
    height: '100%',
    flexDirection: 'column',
    bg: bg,
    gap: 1,
    padding: 1,
    children: () => {
      // Test 1: Parent with explicit width, child with 100%
      text({ content: 'Test 1: Parent width=50, child width=100%', fg: yellow })
      box({
        width: 50,
        border: 1,
        borderColor: border,
        children: () => {
          box({
            width: '100%',  // Should be 50-2 (border) = 48
            height: 2,
            bg: red,
            children: () => text({ content: 'width: 100%' })
          })
        }
      })

      // Test 2: Parent with grow, child with 100%
      text({ content: 'Test 2: Parent grow=1, child width=100%', fg: yellow })
      box({
        height: 4,
        flexDirection: 'row',
        gap: 1,
        children: () => {
          box({
            grow: 1,
            border: 1,
            borderColor: border,
            children: () => {
              box({
                width: '100%',  // Should fill the grown parent
                height: 2,
                bg: green,
                children: () => text({ content: 'width: 100%' })
              })
            }
          })
          box({ width: 20, height: 2, bg: blue, children: () => text({ content: 'fixed 20' }) })
        }
      })

      // Test 3: flex-wrap with many children
      text({ content: 'Test 3: flex-wrap with 20 items (should wrap)', fg: yellow })
      box({
        width: 60,  // Explicit width
        border: 1,
        borderColor: border,
        flexDirection: 'row',
        flexWrap: 'wrap',
        gap: 1,
        padding: 1,
        children: () => {
          for (let i = 0; i < 20; i++) {
            box({
              width: 5,
              height: 1,
              bg: i % 2 === 0 ? red : green,
              children: () => text({ content: String(i).padStart(2, '0') })
            })
          }
        }
      })

      // Test 4: flex-wrap WITHOUT explicit width (this is the problem case)
      text({ content: 'Test 4: flex-wrap NO width (does it wrap?)', fg: yellow })
      box({
        // NO width - grows to content
        border: 1,
        borderColor: border,
        flexDirection: 'row',
        flexWrap: 'wrap',
        gap: 1,
        padding: 1,
        children: () => {
          for (let i = 0; i < 20; i++) {
            box({
              width: 5,
              height: 1,
              bg: i % 2 === 0 ? blue : yellow,
              children: () => text({ content: String(i).padStart(2, '0') })
            })
          }
        }
      })

      text({ content: '[Ctrl+C to exit]', fg: packColor(100, 100, 100, 255) })
    }
  })
}, { mode: 'fullscreen' })

await new Promise(() => {})
