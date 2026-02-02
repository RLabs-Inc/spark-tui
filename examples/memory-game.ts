/**
 * SparkTUI Memory Game
 *
 * Card matching game with:
 * - 4x4 grid of cards
 * - Flip cards to reveal symbols
 * - Match pairs
 * - Move counter
 * - Win detection
 * - Restart button
 *
 * Controls:
 *   Arrow keys  Navigate cards
 *   Enter/Space Flip card
 *   R           Restart
 *   Q           Quit
 */

import { signal, derived } from '@rlabs-inc/signals'
import { mount } from '../ts/engine/mount'
import { box, text, show } from '../ts/primitives'
import { packColor } from '../ts/bridge/shared-buffer'
import { on, isPress } from '../ts/state/keyboard'
import { KEY_UP, KEY_DOWN, KEY_LEFT, KEY_RIGHT, KEY_ENTER, KEY_SPACE, getChar } from '../ts/engine/events'
import type { KeyEvent } from '../ts/engine/events'

// =============================================================================
// CONSTANTS
// =============================================================================

const GRID_SIZE = 4
const CARD_WIDTH = 8
const CARD_HEIGHT = 4
const FLIP_DELAY = 1000 // ms to show unmatched pair

// Card symbols (8 pairs for 4x4 grid)
const SYMBOLS = ['*', '#', '@', '%', '&', '+', '=', '~']

// =============================================================================
// COLORS
// =============================================================================

const colors = {
  bg: packColor(20, 20, 30, 255),
  cardBack: packColor(60, 70, 100, 255),
  cardFront: packColor(40, 50, 70, 255),
  cardMatched: packColor(50, 100, 70, 255),
  cardSelected: packColor(100, 120, 180, 255),
  border: packColor(80, 90, 120, 255),
  borderSelected: packColor(150, 180, 255, 255),
  borderMatched: packColor(100, 200, 140, 255),
  symbol: packColor(255, 220, 100, 255),
  text: packColor(220, 220, 235, 255),
  textMuted: packColor(120, 120, 150, 255),
  textBright: packColor(255, 255, 255, 255),
  success: packColor(100, 220, 140, 255),
}

// =============================================================================
// TYPES
// =============================================================================

interface Card {
  id: number
  symbol: string
  flipped: boolean
  matched: boolean
}

// =============================================================================
// GAME STATE
// =============================================================================

const cards = signal<Card[]>([])
const selectedIndex = signal(0)
const firstFlipped = signal<number | null>(null)
const secondFlipped = signal<number | null>(null)
const moves = signal(0)
const matches = signal(0)
const isChecking = signal(false)
const gameWon = signal(false)
const bestScore = signal<number | null>(null)
const tick = signal(0)

// =============================================================================
// GAME LOGIC
// =============================================================================

function shuffle<T>(array: T[]): T[] {
  const result = [...array]
  for (let i = result.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1))
    ;[result[i], result[j]] = [result[j]!, result[i]!]
  }
  return result
}

function initializeGame(): void {
  // Create pairs of cards
  const symbols = [...SYMBOLS, ...SYMBOLS] // 8 pairs
  const shuffled = shuffle(symbols)

  cards.value = shuffled.map((symbol, index) => ({
    id: index,
    symbol,
    flipped: false,
    matched: false,
  }))

  selectedIndex.value = 0
  firstFlipped.value = null
  secondFlipped.value = null
  moves.value = 0
  matches.value = 0
  isChecking.value = false
  gameWon.value = false
  tick.value++
}

function flipCard(index: number): void {
  if (isChecking.value) return

  const card = cards.value[index]
  if (!card || card.flipped || card.matched) return

  const newCards = [...cards.value]
  newCards[index] = { ...card, flipped: true }
  cards.value = newCards

  if (firstFlipped.value === null) {
    // First card of pair
    firstFlipped.value = index
  } else {
    // Second card of pair
    secondFlipped.value = index
    moves.value++
    isChecking.value = true

    // Check for match
    const first = cards.value[firstFlipped.value]!
    const second = newCards[index]!

    if (first.symbol === second.symbol) {
      // Match found!
      setTimeout(() => {
        const matchedCards = [...cards.value]
        matchedCards[firstFlipped.value!] = { ...matchedCards[firstFlipped.value!]!, matched: true }
        matchedCards[index] = { ...matchedCards[index]!, matched: true }
        cards.value = matchedCards

        matches.value++
        firstFlipped.value = null
        secondFlipped.value = null
        isChecking.value = false

        // Check for win
        if (matches.value === SYMBOLS.length) {
          gameWon.value = true
          if (bestScore.value === null || moves.value < bestScore.value) {
            bestScore.value = moves.value
          }
        }

        tick.value++
      }, 300)
    } else {
      // No match - flip back
      setTimeout(() => {
        const resetCards = [...cards.value]
        resetCards[firstFlipped.value!] = { ...resetCards[firstFlipped.value!]!, flipped: false }
        resetCards[index] = { ...resetCards[index]!, flipped: false }
        cards.value = resetCards

        firstFlipped.value = null
        secondFlipped.value = null
        isChecking.value = false
        tick.value++
      }, FLIP_DELAY)
    }
  }

  tick.value++
}

function handleKey(event: KeyEvent): boolean {
  if (!isPress(event)) return false

  const char = getChar(event)

  // Quit
  if (char === 'q' || char === 'Q') {
    process.exit(0)
  }

  // Restart
  if (char === 'r' || char === 'R') {
    initializeGame()
    return true
  }

  // Navigation
  const current = selectedIndex.value
  const col = current % GRID_SIZE
  const row = Math.floor(current / GRID_SIZE)

  switch (event.keycode) {
    case KEY_UP:
      if (row > 0) {
        selectedIndex.value = current - GRID_SIZE
        tick.value++
      }
      return true
    case KEY_DOWN:
      if (row < GRID_SIZE - 1) {
        selectedIndex.value = current + GRID_SIZE
        tick.value++
      }
      return true
    case KEY_LEFT:
      if (col > 0) {
        selectedIndex.value = current - 1
        tick.value++
      }
      return true
    case KEY_RIGHT:
      if (col < GRID_SIZE - 1) {
        selectedIndex.value = current + 1
        tick.value++
      }
      return true
    case KEY_ENTER:
    case KEY_SPACE:
      flipCard(current)
      return true
  }

  if (char === ' ') {
    flipCard(current)
    return true
  }

  return false
}

// =============================================================================
// RENDER HELPERS
// =============================================================================

function getCardDisplay(card: Card, isSelected: boolean): { bg: number; border: number; content: string } {
  const _ = tick.value // Subscribe to changes

  if (card.matched) {
    return {
      bg: colors.cardMatched,
      border: colors.borderMatched,
      content: card.symbol,
    }
  }

  if (card.flipped) {
    return {
      bg: colors.cardFront,
      border: isSelected ? colors.borderSelected : colors.border,
      content: card.symbol,
    }
  }

  return {
    bg: isSelected ? colors.cardSelected : colors.cardBack,
    border: isSelected ? colors.borderSelected : colors.border,
    content: '?',
  }
}

// =============================================================================
// MOUNT APP
// =============================================================================

const cols = process.stdout.columns || 80
const rows = process.stdout.rows || 24

// Initialize game
initializeGame()

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
      // Title and stats
      box({
        flexDirection: 'row',
        gap: 3,
        marginBottom: 1,
        children: () => {
          text({ content: 'MEMORY GAME', fg: colors.textBright })
          text({ content: '|', fg: colors.textMuted })
          text({ content: () => `Moves: ${moves.value}`, fg: colors.text })
          text({ content: '|', fg: colors.textMuted })
          text({ content: () => `Matches: ${matches.value}/${SYMBOLS.length}`, fg: colors.text })
          show(
            () => bestScore.value !== null,
            () => box({
              flexDirection: 'row',
              gap: 1,
              children: () => {
                text({ content: '|', fg: colors.textMuted })
                text({ content: () => `Best: ${bestScore.value}`, fg: colors.success })
              },
            })
          )
        },
      })

      // Game board
      box({
        flexDirection: 'column',
        gap: 1,
        padding: 1,
        border: 1,
        borderColor: colors.border,
        children: () => {
          for (let row = 0; row < GRID_SIZE; row++) {
            box({
              flexDirection: 'row',
              gap: 1,
              children: () => {
                for (let col = 0; col < GRID_SIZE; col++) {
                  const index = row * GRID_SIZE + col
                  const cardIndex = index

                  box({
                    width: CARD_WIDTH,
                    height: CARD_HEIGHT,
                    justifyContent: 'center',
                    alignItems: 'center',
                    border: 1,
                    borderColor: () => {
                      const card = cards.value[cardIndex]
                      if (!card) return colors.border
                      const display = getCardDisplay(card, selectedIndex.value === cardIndex)
                      return display.border
                    },
                    bg: () => {
                      const card = cards.value[cardIndex]
                      if (!card) return colors.cardBack
                      const display = getCardDisplay(card, selectedIndex.value === cardIndex)
                      return display.bg
                    },
                    children: () => {
                      text({
                        content: () => {
                          const card = cards.value[cardIndex]
                          if (!card) return '?'
                          const display = getCardDisplay(card, selectedIndex.value === cardIndex)
                          return display.content
                        },
                        fg: () => {
                          const card = cards.value[cardIndex]
                          if (!card) return colors.text
                          if (card.flipped || card.matched) return colors.symbol
                          return colors.text
                        },
                      })
                    },
                  })
                }
              },
            })
          }
        },
      })

      // Win overlay
      show(
        () => gameWon.value,
        () => box({
          flexDirection: 'column',
          alignItems: 'center',
          padding: 2,
          marginTop: 1,
          border: 1,
          borderColor: colors.success,
          bg: packColor(30, 60, 40, 255),
          children: () => {
            text({ content: 'CONGRATULATIONS!', fg: colors.success })
            text({ content: () => `You won in ${moves.value} moves!`, fg: colors.textBright })
            show(
              () => bestScore.value === moves.value,
              () => text({ content: 'NEW BEST SCORE!', fg: colors.symbol })
            )
            text({ content: 'Press R to play again', fg: colors.textMuted, marginTop: 1 })
          },
        })
      )

      // Instructions
      show(
        () => !gameWon.value,
        () => box({
          flexDirection: 'column',
          alignItems: 'center',
          marginTop: 1,
          children: () => {
            text({ content: 'Arrow keys to move, Enter/Space to flip', fg: colors.textMuted })
            text({ content: 'R to restart, Q to quit', fg: colors.textMuted })
          },
        })
      )
    },
  })
}, { mode: 'fullscreen' })

console.log('[memory-game] Game started - match all the pairs!')
await new Promise(() => {})
