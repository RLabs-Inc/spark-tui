/**
 * SparkTUI Pong Game
 *
 * Classic two-player pong with:
 * - Two paddles (W/S and Up/Down)
 * - Bouncing ball with cycle animation
 * - Score tracking
 * - Win condition (first to 5)
 * - Ball speed increases over time
 *
 * Controls:
 *   W/S         Player 1 paddle (left)
 *   Up/Down     Player 2 paddle (right)
 *   Space       Start/Pause
 *   R           Restart
 *   Q           Quit
 */

import { signal, derived } from '@rlabs-inc/signals'
import { mount } from '../ts/engine/mount'
import { box, text, show } from '../ts/primitives'
import { packColor } from '../ts/bridge/shared-buffer'
import { on, isPress } from '../ts/state/keyboard'
import { KEY_UP, KEY_DOWN, getChar } from '../ts/engine/events'
import type { KeyEvent } from '../ts/engine/events'

// =============================================================================
// CONSTANTS
// =============================================================================

const BOARD_WIDTH = 60
const BOARD_HEIGHT = 20
const PADDLE_HEIGHT = 5
const PADDLE_MARGIN = 2 // Distance from edge
const WIN_SCORE = 5
const INITIAL_BALL_SPEED = 80 // ms between moves
const SPEED_INCREASE_RATE = 2 // ms faster per hit
const MIN_BALL_SPEED = 30

// =============================================================================
// COLORS
// =============================================================================

const colors = {
  bg: packColor(15, 15, 25, 255),
  board: packColor(20, 25, 35, 255),
  border: packColor(50, 60, 80, 255),
  borderActive: packColor(100, 140, 200, 255),
  paddle1: packColor(100, 180, 255, 255),
  paddle2: packColor(255, 140, 100, 255),
  ball: packColor(255, 255, 100, 255),
  net: packColor(50, 60, 70, 255),
  text: packColor(220, 220, 235, 255),
  textMuted: packColor(120, 120, 150, 255),
  textBright: packColor(255, 255, 255, 255),
  winner: packColor(100, 220, 140, 255),
}

// =============================================================================
// GAME STATE
// =============================================================================

const score1 = signal(0)
const score2 = signal(0)
const paddle1Y = signal(Math.floor(BOARD_HEIGHT / 2) - Math.floor(PADDLE_HEIGHT / 2))
const paddle2Y = signal(Math.floor(BOARD_HEIGHT / 2) - Math.floor(PADDLE_HEIGHT / 2))
const ballX = signal(Math.floor(BOARD_WIDTH / 2))
const ballY = signal(Math.floor(BOARD_HEIGHT / 2))
const ballDX = signal(1)
const ballDY = signal(0)
const ballSpeed = signal(INITIAL_BALL_SPEED)
const gameStarted = signal(false)
const paused = signal(false)
const winner = signal<1 | 2 | null>(null)
const tick = signal(0) // Forces re-render

// Track held keys for smooth movement
const keysHeld = signal<Set<string>>(new Set())

// =============================================================================
// GAME LOGIC
// =============================================================================

function resetBall(direction: 1 | -1): void {
  ballX.value = Math.floor(BOARD_WIDTH / 2)
  ballY.value = Math.floor(BOARD_HEIGHT / 2)
  ballDX.value = direction
  ballDY.value = (Math.random() > 0.5 ? 1 : -1) * (Math.random() * 0.5 + 0.3)
  ballSpeed.value = INITIAL_BALL_SPEED
}

function resetGame(): void {
  score1.value = 0
  score2.value = 0
  paddle1Y.value = Math.floor(BOARD_HEIGHT / 2) - Math.floor(PADDLE_HEIGHT / 2)
  paddle2Y.value = Math.floor(BOARD_HEIGHT / 2) - Math.floor(PADDLE_HEIGHT / 2)
  winner.value = null
  gameStarted.value = false
  paused.value = false
  resetBall(Math.random() > 0.5 ? 1 : -1)
  tick.value++
}

function movePaddles(): void {
  if (paused.value || winner.value) return

  const keys = keysHeld.value

  // Player 1 (W/S)
  if (keys.has('w') || keys.has('W')) {
    paddle1Y.value = Math.max(0, paddle1Y.value - 1)
    tick.value++
  }
  if (keys.has('s') || keys.has('S')) {
    paddle1Y.value = Math.min(BOARD_HEIGHT - PADDLE_HEIGHT, paddle1Y.value + 1)
    tick.value++
  }

  // Player 2 (Up/Down)
  if (keys.has('up')) {
    paddle2Y.value = Math.max(0, paddle2Y.value - 1)
    tick.value++
  }
  if (keys.has('down')) {
    paddle2Y.value = Math.min(BOARD_HEIGHT - PADDLE_HEIGHT, paddle2Y.value + 1)
    tick.value++
  }
}

function moveBall(): void {
  if (!gameStarted.value || paused.value || winner.value) return

  let newX = ballX.value + ballDX.value
  let newY = ballY.value + ballDY.value
  let newDX = ballDX.value
  let newDY = ballDY.value

  // Top/bottom wall collision
  if (newY <= 0 || newY >= BOARD_HEIGHT - 1) {
    newDY = -newDY
    newY = Math.max(0, Math.min(BOARD_HEIGHT - 1, newY))
  }

  // Left paddle collision
  if (newX <= PADDLE_MARGIN + 1) {
    const paddleTop = paddle1Y.value
    const paddleBottom = paddle1Y.value + PADDLE_HEIGHT
    if (newY >= paddleTop && newY < paddleBottom) {
      // Hit paddle - bounce back
      newDX = Math.abs(newDX)
      // Add spin based on where ball hit paddle
      const hitPos = (newY - paddleTop) / PADDLE_HEIGHT
      newDY = (hitPos - 0.5) * 2
      newX = PADDLE_MARGIN + 2
      // Speed up
      ballSpeed.value = Math.max(MIN_BALL_SPEED, ballSpeed.value - SPEED_INCREASE_RATE)
    } else if (newX < 0) {
      // Missed - Player 2 scores
      score2.value++
      if (score2.value >= WIN_SCORE) {
        winner.value = 2
        gameStarted.value = false
      } else {
        resetBall(-1)
      }
      tick.value++
      return
    }
  }

  // Right paddle collision
  if (newX >= BOARD_WIDTH - PADDLE_MARGIN - 2) {
    const paddleTop = paddle2Y.value
    const paddleBottom = paddle2Y.value + PADDLE_HEIGHT
    if (newY >= paddleTop && newY < paddleBottom) {
      // Hit paddle - bounce back
      newDX = -Math.abs(newDX)
      // Add spin based on where ball hit paddle
      const hitPos = (newY - paddleTop) / PADDLE_HEIGHT
      newDY = (hitPos - 0.5) * 2
      newX = BOARD_WIDTH - PADDLE_MARGIN - 3
      // Speed up
      ballSpeed.value = Math.max(MIN_BALL_SPEED, ballSpeed.value - SPEED_INCREASE_RATE)
    } else if (newX >= BOARD_WIDTH) {
      // Missed - Player 1 scores
      score1.value++
      if (score1.value >= WIN_SCORE) {
        winner.value = 1
        gameStarted.value = false
      } else {
        resetBall(1)
      }
      tick.value++
      return
    }
  }

  ballX.value = Math.round(newX)
  ballY.value = Math.round(Math.max(0, Math.min(BOARD_HEIGHT - 1, newY)))
  ballDX.value = newDX
  ballDY.value = newDY
  tick.value++
}

function handleKey(event: KeyEvent): boolean {
  const char = getChar(event)

  if (isPress(event)) {
    // Quit
    if (char === 'q' || char === 'Q') {
      process.exit(0)
    }

    // Restart
    if (char === 'r' || char === 'R') {
      resetGame()
      return true
    }

    // Start/Pause
    if (char === ' ') {
      if (winner.value) {
        resetGame()
      } else if (!gameStarted.value) {
        gameStarted.value = true
      } else {
        paused.value = !paused.value
      }
      return true
    }

    // Track held keys
    const keys = new Set(keysHeld.value)
    if (char === 'w' || char === 'W') keys.add('w')
    if (char === 's' || char === 'S') keys.add('s')
    if (event.keycode === KEY_UP) keys.add('up')
    if (event.keycode === KEY_DOWN) keys.add('down')
    keysHeld.value = keys
  } else {
    // Key release - stop movement
    const keys = new Set(keysHeld.value)
    if (char === 'w' || char === 'W') keys.delete('w')
    if (char === 's' || char === 'S') keys.delete('s')
    if (event.keycode === KEY_UP) keys.delete('up')
    if (event.keycode === KEY_DOWN) keys.delete('down')
    keysHeld.value = keys
  }

  return false
}

// =============================================================================
// RENDER HELPERS
// =============================================================================

function renderBoard(): string[] {
  const _ = tick.value // Subscribe to tick changes
  const lines: string[] = []

  for (let y = 0; y < BOARD_HEIGHT; y++) {
    let line = ''
    for (let x = 0; x < BOARD_WIDTH; x++) {
      // Center line (net)
      if (x === Math.floor(BOARD_WIDTH / 2) && y % 2 === 0) {
        line += '|'
        continue
      }

      // Left paddle
      if (x === PADDLE_MARGIN && y >= paddle1Y.value && y < paddle1Y.value + PADDLE_HEIGHT) {
        line += '#'
        continue
      }

      // Right paddle
      if (x === BOARD_WIDTH - PADDLE_MARGIN - 1 && y >= paddle2Y.value && y < paddle2Y.value + PADDLE_HEIGHT) {
        line += '#'
        continue
      }

      // Ball
      if (Math.round(ballX.value) === x && Math.round(ballY.value) === y) {
        line += 'O'
        continue
      }

      // Empty space
      line += ' '
    }
    lines.push(line)
  }

  return lines
}

// =============================================================================
// GAME LOOPS
// =============================================================================

// Ball movement (variable speed)
let ballInterval: ReturnType<typeof setInterval> | null = null

function startBallLoop(): void {
  if (ballInterval) clearInterval(ballInterval)
  ballInterval = setInterval(moveBall, ballSpeed.value)
}

// Speed change detector
setInterval(() => {
  startBallLoop()
}, 50)

// Paddle movement (fixed rate)
setInterval(movePaddles, 40)

// =============================================================================
// MOUNT APP
// =============================================================================

const cols = process.stdout.columns || 80
const rows = process.stdout.rows || 24

mount(() => {
  // Global keyboard handler
  on(handleKey)

  // Start game loop
  startBallLoop()

  // Root container
  box({
    width: cols,
    height: rows,
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    bg: colors.bg,
    children: () => {
      // Title and Score
      box({
        width: BOARD_WIDTH + 2,
        flexDirection: 'row',
        justifyContent: 'space-between',
        marginBottom: 1,
        children: () => {
          // Player 1 score
          box({
            flexDirection: 'row',
            gap: 1,
            children: () => {
              text({ content: 'P1 (W/S):', fg: colors.paddle1 })
              text({ content: () => String(score1.value), fg: colors.textBright })
            },
          })

          // Title
          text({ content: 'PONG', fg: colors.textBright })

          // Player 2 score
          box({
            flexDirection: 'row',
            gap: 1,
            children: () => {
              text({ content: () => String(score2.value), fg: colors.textBright })
              text({ content: ':P2 (Arrows)', fg: colors.paddle2 })
            },
          })
        },
      })

      // Game board
      box({
        width: BOARD_WIDTH + 2,
        height: BOARD_HEIGHT + 2,
        border: 1,
        borderColor: () => winner.value ? colors.winner : paused.value ? colors.border : colors.borderActive,
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

          // Start screen
          show(
            () => !gameStarted.value && !winner.value,
            () => box({
              width: BOARD_WIDTH,
              height: 5,
              flexDirection: 'column',
              alignItems: 'center',
              justifyContent: 'center',
              bg: packColor(0, 0, 0, 200),
              marginTop: -BOARD_HEIGHT / 2 - 2,
              children: () => {
                text({ content: 'PONG', fg: colors.textBright })
                text({ content: `First to ${WIN_SCORE} wins!`, fg: colors.text })
                text({ content: 'Press SPACE to start', fg: colors.textMuted })
              },
            })
          )

          // Paused overlay
          show(
            () => paused.value && !winner.value,
            () => box({
              width: BOARD_WIDTH,
              height: 3,
              flexDirection: 'column',
              alignItems: 'center',
              justifyContent: 'center',
              bg: packColor(0, 0, 0, 200),
              marginTop: -BOARD_HEIGHT / 2 - 1,
              children: () => {
                text({ content: 'PAUSED', fg: colors.textBright })
                text({ content: 'Press SPACE to resume', fg: colors.textMuted })
              },
            })
          )

          // Winner overlay
          show(
            () => winner.value !== null,
            () => box({
              width: BOARD_WIDTH,
              height: 5,
              flexDirection: 'column',
              alignItems: 'center',
              justifyContent: 'center',
              bg: packColor(0, 0, 0, 200),
              marginTop: -BOARD_HEIGHT / 2 - 2,
              children: () => {
                text({
                  content: () => `PLAYER ${winner.value} WINS!`,
                  fg: winner.value === 1 ? colors.paddle1 : colors.paddle2,
                })
                text({ content: () => `${score1.value} - ${score2.value}`, fg: colors.textBright })
                text({ content: 'Press SPACE to play again', fg: colors.textMuted })
              },
            })
          )
        },
      })

      // Controls
      box({
        flexDirection: 'column',
        alignItems: 'center',
        marginTop: 1,
        children: () => {
          text({
            content: () => `Ball Speed: ${Math.round(1000 / ballSpeed.value)} | Win at ${WIN_SCORE}`,
            fg: colors.textMuted,
          })
          text({ content: 'R to restart, Q to quit', fg: colors.textMuted })
        },
      })
    },
  })
}, { mode: 'fullscreen' })

console.log('[pong] Game ready - press SPACE to start!')
await new Promise(() => {})
