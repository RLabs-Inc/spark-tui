/**
 * SparkTUI Typing Test
 *
 * Typing speed test with:
 * - Random words/sentences to type
 * - Real-time comparison (correct=green, wrong=red)
 * - WPM (words per minute) calculation
 * - Accuracy percentage
 * - Timer countdown
 * - Results screen
 *
 * Controls:
 *   Type        Match the text
 *   Backspace   Delete last character
 *   Enter       Start/Restart
 *   Q           Quit (when not typing)
 */

import { signal, derived } from '@rlabs-inc/signals'
import { mount } from '../ts/engine/mount'
import { box, text, show } from '../ts/primitives'
import { packColor } from '../ts/bridge/shared-buffer'
import { on, isPress } from '../ts/state/keyboard'
import { KEY_ENTER, KEY_BACKSPACE, KEY_ESCAPE, getChar } from '../ts/engine/events'
import type { KeyEvent } from '../ts/engine/events'

// =============================================================================
// CONSTANTS
// =============================================================================

const TEST_DURATION = 60 // seconds
const WORDS_PER_LINE = 8

// Word lists for typing test
const COMMON_WORDS = [
  'the', 'be', 'to', 'of', 'and', 'a', 'in', 'that', 'have', 'I',
  'it', 'for', 'not', 'on', 'with', 'he', 'as', 'you', 'do', 'at',
  'this', 'but', 'his', 'by', 'from', 'they', 'we', 'say', 'her', 'she',
  'or', 'an', 'will', 'my', 'one', 'all', 'would', 'there', 'their', 'what',
  'so', 'up', 'out', 'if', 'about', 'who', 'get', 'which', 'go', 'me',
  'when', 'make', 'can', 'like', 'time', 'no', 'just', 'him', 'know', 'take',
  'people', 'into', 'year', 'your', 'good', 'some', 'could', 'them', 'see', 'other',
  'than', 'then', 'now', 'look', 'only', 'come', 'its', 'over', 'think', 'also',
  'back', 'after', 'use', 'two', 'how', 'our', 'work', 'first', 'well', 'way',
  'even', 'new', 'want', 'because', 'any', 'these', 'give', 'day', 'most', 'us',
  'code', 'type', 'fast', 'slow', 'quick', 'test', 'word', 'text', 'key', 'board',
  'speed', 'time', 'score', 'high', 'best', 'game', 'play', 'win', 'lose', 'try',
]

// =============================================================================
// COLORS
// =============================================================================

const colors = {
  bg: packColor(18, 18, 28, 255),
  surface: packColor(28, 30, 42, 255),
  border: packColor(60, 70, 90, 255),
  borderActive: packColor(100, 140, 220, 255),
  textPending: packColor(100, 100, 130, 255),
  textCurrent: packColor(255, 255, 255, 255),
  textCorrect: packColor(100, 220, 140, 255),
  textWrong: packColor(255, 100, 100, 255),
  textMuted: packColor(120, 120, 150, 255),
  textBright: packColor(255, 255, 255, 255),
  cursor: packColor(255, 200, 100, 255),
  statLabel: packColor(140, 160, 200, 255),
  statValue: packColor(255, 255, 255, 255),
  wpm: packColor(100, 200, 255, 255),
  accuracy: packColor(100, 220, 140, 255),
  timer: packColor(255, 180, 100, 255),
}

// =============================================================================
// GAME STATE
// =============================================================================

type GameState = 'waiting' | 'playing' | 'finished'

const state = signal<GameState>('waiting')
const targetText = signal('')
const typedText = signal('')
const timeRemaining = signal(TEST_DURATION)
const startTime = signal<number | null>(null)

// Stats
const correctChars = signal(0)
const wrongChars = signal(0)
const totalKeystrokes = signal(0)
const wordsCompleted = signal(0)

// Results
const finalWpm = signal(0)
const finalAccuracy = signal(0)
const bestWpm = signal<number | null>(null)

const tick = signal(0)

// Derived
const currentWpm = derived(() => {
  if (state.value !== 'playing' || !startTime.value) return 0
  const elapsed = (Date.now() - startTime.value) / 1000 / 60 // minutes
  if (elapsed < 0.01) return 0
  return Math.round(correctChars.value / 5 / elapsed) // 5 chars = 1 word
})

const currentAccuracy = derived(() => {
  if (totalKeystrokes.value === 0) return 100
  return Math.round((correctChars.value / totalKeystrokes.value) * 100)
})

// =============================================================================
// GAME LOGIC
// =============================================================================

function generateText(): string {
  const words: string[] = []
  for (let i = 0; i < 50; i++) {
    words.push(COMMON_WORDS[Math.floor(Math.random() * COMMON_WORDS.length)]!)
  }
  return words.join(' ')
}

function startGame(): void {
  targetText.value = generateText()
  typedText.value = ''
  timeRemaining.value = TEST_DURATION
  startTime.value = Date.now()
  correctChars.value = 0
  wrongChars.value = 0
  totalKeystrokes.value = 0
  wordsCompleted.value = 0
  state.value = 'playing'
  tick.value++
}

function endGame(): void {
  const elapsed = startTime.value ? (Date.now() - startTime.value) / 1000 / 60 : 1
  finalWpm.value = Math.round(correctChars.value / 5 / elapsed)
  finalAccuracy.value = totalKeystrokes.value > 0
    ? Math.round((correctChars.value / totalKeystrokes.value) * 100)
    : 0

  if (bestWpm.value === null || finalWpm.value > bestWpm.value) {
    bestWpm.value = finalWpm.value
  }

  state.value = 'finished'
  tick.value++
}

function handleTyping(char: string): void {
  if (state.value !== 'playing') return

  const currentPos = typedText.value.length
  const targetChar = targetText.value[currentPos]

  typedText.value += char
  totalKeystrokes.value++

  if (char === targetChar) {
    correctChars.value++
    // Count completed words (space after word)
    if (char === ' ') {
      wordsCompleted.value++
    }
  } else {
    wrongChars.value++
  }

  // Check if completed
  if (typedText.value.length >= targetText.value.length) {
    endGame()
  }

  tick.value++
}

function handleBackspace(): void {
  if (state.value !== 'playing') return
  if (typedText.value.length > 0) {
    typedText.value = typedText.value.slice(0, -1)
    tick.value++
  }
}

function handleKey(event: KeyEvent): boolean {
  if (!isPress(event)) return false

  const char = getChar(event)

  // Always allow quit when not playing
  if ((char === 'q' || char === 'Q') && state.value !== 'playing') {
    process.exit(0)
  }

  // Escape to quit during play
  if (event.keycode === KEY_ESCAPE) {
    if (state.value === 'playing') {
      endGame()
      return true
    }
    process.exit(0)
  }

  // Enter to start/restart
  if (event.keycode === KEY_ENTER) {
    if (state.value !== 'playing') {
      startGame()
      return true
    }
  }

  // Backspace
  if (event.keycode === KEY_BACKSPACE) {
    handleBackspace()
    return true
  }

  // Regular typing
  if (char && state.value === 'playing') {
    handleTyping(char)
    return true
  }

  return false
}

// =============================================================================
// TIMER
// =============================================================================

setInterval(() => {
  if (state.value === 'playing' && timeRemaining.value > 0) {
    timeRemaining.value--
    tick.value++
    if (timeRemaining.value === 0) {
      endGame()
    }
  }
}, 1000)

// =============================================================================
// RENDER HELPERS
// =============================================================================

function renderTextDisplay(): { lines: Array<{ chars: Array<{ char: string; color: number }> }> } {
  const _ = tick.value
  const target = targetText.value
  const typed = typedText.value
  const lines: Array<{ chars: Array<{ char: string; color: number }> }> = []
  let currentLine: Array<{ char: string; color: number }> = []
  let wordCount = 0

  for (let i = 0; i < target.length; i++) {
    const targetChar = target[i]!
    let color: number

    if (i < typed.length) {
      // Already typed
      color = typed[i] === targetChar ? colors.textCorrect : colors.textWrong
    } else if (i === typed.length) {
      // Current position (cursor)
      color = colors.cursor
    } else {
      // Not yet typed
      color = colors.textPending
    }

    currentLine.push({ char: targetChar, color })

    // Word boundary
    if (targetChar === ' ') {
      wordCount++
      if (wordCount >= WORDS_PER_LINE) {
        lines.push({ chars: currentLine })
        currentLine = []
        wordCount = 0
      }
    }
  }

  if (currentLine.length > 0) {
    lines.push({ chars: currentLine })
  }

  return { lines: lines.slice(0, 5) } // Show max 5 lines
}

// =============================================================================
// MOUNT APP
// =============================================================================

const cols = process.stdout.columns || 80
const rows = process.stdout.rows || 24

mount(() => {
  // Global keyboard handler
  on(handleKey)

  // Root container
  box({
    width: cols,
    height: rows,
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    bg: colors.bg,
    children: () => {
      // Title
      text({ content: 'TYPING TEST', fg: colors.textBright, marginBottom: 1 })

      // Stats bar
      box({
        flexDirection: 'row',
        gap: 4,
        marginBottom: 1,
        children: () => {
          // Timer
          box({
            flexDirection: 'row',
            gap: 1,
            children: () => {
              text({ content: 'Time:', fg: colors.statLabel })
              text({
                content: () => {
                  const secs = timeRemaining.value
                  const mins = Math.floor(secs / 60)
                  const s = secs % 60
                  return `${mins}:${s.toString().padStart(2, '0')}`
                },
                fg: colors.timer,
              })
            },
          })

          // WPM
          box({
            flexDirection: 'row',
            gap: 1,
            children: () => {
              text({ content: 'WPM:', fg: colors.statLabel })
              text({ content: () => String(currentWpm.value), fg: colors.wpm })
            },
          })

          // Accuracy
          box({
            flexDirection: 'row',
            gap: 1,
            children: () => {
              text({ content: 'Accuracy:', fg: colors.statLabel })
              text({ content: () => `${currentAccuracy.value}%`, fg: colors.accuracy })
            },
          })

          // Words
          box({
            flexDirection: 'row',
            gap: 1,
            children: () => {
              text({ content: 'Words:', fg: colors.statLabel })
              text({ content: () => String(wordsCompleted.value), fg: colors.statValue })
            },
          })
        },
      })

      // Text display area
      box({
        width: 70,
        minHeight: 8,
        padding: 1,
        border: 1,
        borderColor: () => state.value === 'playing' ? colors.borderActive : colors.border,
        bg: colors.surface,
        flexDirection: 'column',
        children: () => {
          // Waiting state
          show(
            () => state.value === 'waiting',
            () => box({
              flexDirection: 'column',
              alignItems: 'center',
              justifyContent: 'center',
              grow: 1,
              children: () => {
                text({ content: 'Test your typing speed!', fg: colors.textBright })
                text({ content: `${TEST_DURATION} seconds to type as many words as you can`, fg: colors.textMuted })
                text({ content: '', fg: colors.textMuted })
                text({ content: 'Press ENTER to start', fg: colors.cursor })
                show(
                  () => bestWpm.value !== null,
                  () => text({ content: () => `Best: ${bestWpm.value} WPM`, fg: colors.wpm, marginTop: 1 })
                )
              },
            })
          )

          // Playing state - render the text
          show(
            () => state.value === 'playing',
            () => box({
              flexDirection: 'column',
              children: () => {
                // Render each line
                for (let lineIdx = 0; lineIdx < 5; lineIdx++) {
                  const idx = lineIdx
                  text({
                    content: () => {
                      const display = renderTextDisplay()
                      const line = display.lines[idx]
                      if (!line) return ''
                      return line.chars.map(c => c.char).join('')
                    },
                    fg: () => {
                      // For now, use a single color per line based on progress
                      // Full coloring would require per-char rendering
                      const display = renderTextDisplay()
                      const line = display.lines[idx]
                      if (!line || line.chars.length === 0) return colors.textPending
                      // Use the first char's color as indicator
                      return line.chars[0]!.color
                    },
                  })
                }
              },
            })
          )

          // Finished state
          show(
            () => state.value === 'finished',
            () => box({
              flexDirection: 'column',
              alignItems: 'center',
              justifyContent: 'center',
              grow: 1,
              children: () => {
                text({ content: 'Test Complete!', fg: colors.textBright })
                text({ content: '', fg: colors.textMuted })

                // Results
                box({
                  flexDirection: 'row',
                  gap: 4,
                  marginTop: 1,
                  children: () => {
                    box({
                      flexDirection: 'column',
                      alignItems: 'center',
                      children: () => {
                        text({ content: () => String(finalWpm.value), fg: colors.wpm })
                        text({ content: 'WPM', fg: colors.statLabel })
                      },
                    })

                    box({
                      flexDirection: 'column',
                      alignItems: 'center',
                      children: () => {
                        text({ content: () => `${finalAccuracy.value}%`, fg: colors.accuracy })
                        text({ content: 'Accuracy', fg: colors.statLabel })
                      },
                    })

                    box({
                      flexDirection: 'column',
                      alignItems: 'center',
                      children: () => {
                        text({ content: () => String(wordsCompleted.value), fg: colors.statValue })
                        text({ content: 'Words', fg: colors.statLabel })
                      },
                    })

                    box({
                      flexDirection: 'column',
                      alignItems: 'center',
                      children: () => {
                        text({ content: () => String(correctChars.value), fg: colors.textCorrect })
                        text({ content: 'Correct', fg: colors.statLabel })
                      },
                    })

                    box({
                      flexDirection: 'column',
                      alignItems: 'center',
                      children: () => {
                        text({ content: () => String(wrongChars.value), fg: colors.textWrong })
                        text({ content: 'Errors', fg: colors.statLabel })
                      },
                    })
                  },
                })

                // Best score
                show(
                  () => bestWpm.value !== null,
                  () => box({
                    marginTop: 1,
                    children: () => {
                      show(
                        () => finalWpm.value === bestWpm.value,
                        () => text({ content: 'NEW BEST SCORE!', fg: colors.cursor })
                      )
                      show(
                        () => finalWpm.value !== bestWpm.value,
                        () => text({ content: () => `Best: ${bestWpm.value} WPM`, fg: colors.textMuted })
                      )
                    },
                  })
                )

                text({ content: 'Press ENTER to try again', fg: colors.textMuted, marginTop: 1 })
              },
            })
          )
        },
      })

      // Typed text indicator (during play)
      show(
        () => state.value === 'playing',
        () => box({
          marginTop: 1,
          flexDirection: 'row',
          gap: 1,
          children: () => {
            text({ content: 'Typed:', fg: colors.statLabel })
            text({
              content: () => {
                const typed = typedText.value
                // Show last 30 chars
                const display = typed.length > 30 ? '...' + typed.slice(-27) : typed
                return display + '_'
              },
              fg: colors.textMuted,
            })
          },
        })
      )

      // Instructions
      box({
        flexDirection: 'column',
        alignItems: 'center',
        marginTop: 1,
        children: () => {
          show(
            () => state.value !== 'playing',
            () => text({ content: 'Press Q to quit', fg: colors.textMuted })
          )
          show(
            () => state.value === 'playing',
            () => text({ content: 'Press ESC to end early', fg: colors.textMuted })
          )
        },
      })
    },
  })
}, { mode: 'fullscreen' })

console.log('[typing-test] Ready - press ENTER to start!')
await new Promise(() => {})
