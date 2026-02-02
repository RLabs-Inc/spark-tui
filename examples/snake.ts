/**
 * SparkTUI Snake Game
 *
 * Classic snake game with:
 * - Arrow key controls
 * - Growing snake on food collection
 * - Score display
 * - Game over detection (wall/self collision)
 * - Restart option
 * - Speed increases with score
 *
 * Controls:
 *   Arrow keys  Move the snake
 *   Space       Pause/Resume
 *   R           Restart
 *   Q           Quit
 */

import { signal, derived } from '@rlabs-inc/signals'
import { mount } from '../ts/engine/mount'
import { box, text, show } from '../ts/primitives'
import { packColor } from '../ts/bridge/shared-buffer'
import { on, isPress } from '../ts/state/keyboard'
import { KEY_UP, KEY_DOWN, KEY_LEFT, KEY_RIGHT, getChar } from '../ts/engine/events'
import type { KeyEvent } from '../ts/engine/events'

// =============================================================================
// CONSTANTS
// =============================================================================

const BOARD_WIDTH = 40
const BOARD_HEIGHT = 20
const INITIAL_SPEED = 150 // ms between moves
const SPEED_INCREASE = 5 // ms faster per food eaten
const MIN_SPEED = 50 // fastest speed

// =============================================================================
// COLORS
// =============================================================================

const colors = {
  bg: packColor(15, 15, 20, 255),
  board: packColor(25, 30, 35, 255),
  border: packColor(60, 70, 80, 255),
  borderActive: packColor(100, 140, 200, 255),
  snake: packColor(100, 220, 120, 255),
  snakeHead: packColor(150, 255, 160, 255),
  food: packColor(255, 100, 100, 255),
  text: packColor(220, 220, 235, 255),
  textMuted: packColor(120, 120, 150, 255),
  textBright: packColor(255, 255, 255, 255),
  gameOver: packColor(255, 80, 80, 255),
  paused: packColor(255, 200, 100, 255),
}

// =============================================================================
// TYPES
// =============================================================================

interface Point {
  x: number
  y: number
}

type Direction = 'up' | 'down' | 'left' | 'right'

// =============================================================================
// GAME STATE
// =============================================================================

const score = signal(0)
const highScore = signal(0)
const gameOver = signal(false)
const paused = signal(false)
const direction = signal<Direction>('right')
const nextDirection = signal<Direction>('right')
const snake = signal<Point[]>([
  { x: 10, y: 10 },
  { x: 9, y: 10 },
  { x: 8, y: 10 },
])
const food = signal<Point>({ x: 20, y: 10 })
const tick = signal(0) // Forces re-render of board

// Derived state
const speed = derived(() => Math.max(MIN_SPEED, INITIAL_SPEED - score.value * SPEED_INCREASE))

// =============================================================================
// GAME LOGIC
// =============================================================================

function spawnFood(): void {
  const occupied = new Set(snake.value.map(p => `${p.x},${p.y}`))
  let newFood: Point
  do {
    newFood = {
      x: Math.floor(Math.random() * BOARD_WIDTH),
      y: Math.floor(Math.random() * BOARD_HEIGHT),
    }
  } while (occupied.has(`${newFood.x},${newFood.y}`))
  food.value = newFood
}

function resetGame(): void {
  if (score.value > highScore.value) {
    highScore.value = score.value
  }
  score.value = 0
  gameOver.value = false
  paused.value = false
  direction.value = 'right'
  nextDirection.value = 'right'
  snake.value = [
    { x: 10, y: 10 },
    { x: 9, y: 10 },
    { x: 8, y: 10 },
  ]
  spawnFood()
  tick.value++
}

function moveSnake(): void {
  if (gameOver.value || paused.value) return

  // Apply buffered direction
  direction.value = nextDirection.value

  const head = snake.value[0]!
  let newHead: Point

  switch (direction.value) {
    case 'up':
      newHead = { x: head.x, y: head.y - 1 }
      break
    case 'down':
      newHead = { x: head.x, y: head.y + 1 }
      break
    case 'left':
      newHead = { x: head.x - 1, y: head.y }
      break
    case 'right':
      newHead = { x: head.x + 1, y: head.y }
      break
  }

  // Check wall collision
  if (newHead.x < 0 || newHead.x >= BOARD_WIDTH || newHead.y < 0 || newHead.y >= BOARD_HEIGHT) {
    gameOver.value = true
    return
  }

  // Check self collision
  for (const segment of snake.value) {
    if (segment.x === newHead.x && segment.y === newHead.y) {
      gameOver.value = true
      return
    }
  }

  // Check food collision
  const ateFood = newHead.x === food.value.x && newHead.y === food.value.y

  // Move snake
  const newSnake = [newHead, ...snake.value]
  if (!ateFood) {
    newSnake.pop() // Remove tail if didn't eat
  } else {
    score.value++
    spawnFood()
  }

  snake.value = newSnake
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
    resetGame()
    return true
  }

  // Pause
  if (char === ' ') {
    if (!gameOver.value) {
      paused.value = !paused.value
    }
    return true
  }

  // Direction (buffer to prevent 180-degree turns)
  if (!gameOver.value && !paused.value) {
    const current = direction.value
    switch (event.keycode) {
      case KEY_UP:
        if (current !== 'down') nextDirection.value = 'up'
        return true
      case KEY_DOWN:
        if (current !== 'up') nextDirection.value = 'down'
        return true
      case KEY_LEFT:
        if (current !== 'right') nextDirection.value = 'left'
        return true
      case KEY_RIGHT:
        if (current !== 'left') nextDirection.value = 'right'
        return true
    }
  }

  return false
}

// =============================================================================
// RENDER HELPERS
// =============================================================================

function renderBoard(): string[] {
  const _ = tick.value // Subscribe to tick changes
  const lines: string[] = []
  const snakeSet = new Set(snake.value.map(p => `${p.x},${p.y}`))
  const headPos = `${snake.value[0]!.x},${snake.value[0]!.y}`
  const foodPos = `${food.value.x},${food.value.y}`

  for (let y = 0; y < BOARD_HEIGHT; y++) {
    let line = ''
    for (let x = 0; x < BOARD_WIDTH; x++) {
      const pos = `${x},${y}`
      if (pos === headPos) {
        line += '@' // Snake head
      } else if (snakeSet.has(pos)) {
        line += 'O' // Snake body
      } else if (pos === foodPos) {
        line += '*' // Food
      } else {
        line += ' ' // Empty
      }
    }
    lines.push(line)
  }

  return lines
}

// =============================================================================
// GAME LOOP
// =============================================================================

let gameInterval: ReturnType<typeof setInterval> | null = null

function startGameLoop(): void {
  if (gameInterval) clearInterval(gameInterval)
  gameInterval = setInterval(() => {
    moveSnake()
  }, speed.value)
}

// Restart loop when speed changes
let lastSpeed = speed.value
setInterval(() => {
  if (speed.value !== lastSpeed) {
    lastSpeed = speed.value
    if (!gameOver.value && !paused.value) {
      startGameLoop()
    }
  }
}, 100)

// =============================================================================
// MOUNT APP
// =============================================================================

const cols = process.stdout.columns || 80
const rows = process.stdout.rows || 24

mount(() => {
  // Global keyboard handler
  on(handleKey)

  // Start game loop
  startGameLoop()

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
      box({
        flexDirection: 'row',
        gap: 2,
        marginBottom: 1,
        children: () => {
          text({ content: 'SNAKE', fg: colors.textBright })
          text({ content: '|', fg: colors.textMuted })
          text({ content: () => `Score: ${score.value}`, fg: colors.text })
          text({ content: '|', fg: colors.textMuted })
          text({ content: () => `High: ${highScore.value}`, fg: colors.textMuted })
          text({ content: '|', fg: colors.textMuted })
          text({ content: () => `Speed: ${Math.round(1000 / speed.value)}`, fg: colors.textMuted })
        },
      })

      // Game board
      box({
        width: BOARD_WIDTH + 2,
        height: BOARD_HEIGHT + 2,
        border: 1,
        borderColor: () => gameOver.value ? colors.gameOver : paused.value ? colors.paused : colors.border,
        bg: colors.board,
        flexDirection: 'column',
        children: () => {
          // Board content
          box({
            flexDirection: 'column',
            children: () => {
              for (let y = 0; y < BOARD_HEIGHT; y++) {
                const rowY = y
                text({
                  content: () => {
                    const lines = renderBoard()
                    return lines[rowY] || ''
                  },
                  fg: colors.text,
                })
              }
            },
          })

          // Game over overlay
          show(
            () => gameOver.value,
            () => box({
              width: BOARD_WIDTH,
              height: 5,
              flexDirection: 'column',
              alignItems: 'center',
              justifyContent: 'center',
              bg: packColor(0, 0, 0, 200),
              marginTop: -BOARD_HEIGHT / 2 - 2,
              children: () => {
                text({ content: 'GAME OVER', fg: colors.gameOver })
                text({ content: () => `Final Score: ${score.value}`, fg: colors.text })
                text({ content: 'Press R to restart', fg: colors.textMuted })
              },
            })
          )

          // Paused overlay
          show(
            () => paused.value && !gameOver.value,
            () => box({
              width: BOARD_WIDTH,
              height: 3,
              flexDirection: 'column',
              alignItems: 'center',
              justifyContent: 'center',
              bg: packColor(0, 0, 0, 200),
              marginTop: -BOARD_HEIGHT / 2 - 1,
              children: () => {
                text({ content: 'PAUSED', fg: colors.paused })
                text({ content: 'Press SPACE to resume', fg: colors.textMuted })
              },
            })
          )
        },
      })

      // Legend
      box({
        flexDirection: 'row',
        gap: 3,
        marginTop: 1,
        children: () => {
          text({ content: '@ Head', fg: colors.snakeHead })
          text({ content: 'O Body', fg: colors.snake })
          text({ content: '* Food', fg: colors.food })
        },
      })

      // Controls
      box({
        flexDirection: 'column',
        alignItems: 'center',
        marginTop: 1,
        children: () => {
          text({ content: 'Controls: Arrow keys to move, Space to pause', fg: colors.textMuted })
          text({ content: 'R to restart, Q to quit', fg: colors.textMuted })
        },
      })
    },
  })
}, { mode: 'fullscreen' })

console.log('[snake] Game started - use arrow keys to play!')
await new Promise(() => {})
