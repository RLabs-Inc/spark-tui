/**
 * SparkTUI - Kanban Board
 *
 * A Kanban-style task board demonstrating:
 * - Multi-column layout
 * - Drag-drop equivalent via keyboard
 * - Card management (add, move, delete)
 * - Scrollable columns
 * - Visual feedback for selection
 *
 * Controls:
 *   Tab         Switch between columns
 *   Up/Down     Navigate cards in column
 *   Left/Right  Move card between columns
 *   Enter       Edit card title
 *   n           Add new card to current column
 *   d           Delete selected card
 *   q           Quit
 */

import { signal, derived } from '@rlabs-inc/signals'
import { mount } from '../ts/engine/mount'
import { box, text, input, each, show } from '../ts/primitives'
import { packColor } from '../ts/bridge/shared-buffer'
import { on, isPress } from '../ts/state/keyboard'
import type { KeyEvent } from '../ts/state/keyboard'
import { getChar, isEnter, isEscape } from '../ts/engine/events'

// =============================================================================
// TYPES
// =============================================================================

interface Card {
  id: string
  title: string
  createdAt: number
}

type ColumnId = 'todo' | 'progress' | 'done'

interface Column {
  id: ColumnId
  title: string
  cards: Card[]
}

// =============================================================================
// COLORS
// =============================================================================

const colors = {
  bg: packColor(20, 22, 28, 255),
  bgColumn: packColor(28, 30, 38, 255),
  bgCard: packColor(38, 42, 52, 255),
  bgCardSelected: packColor(50, 55, 70, 255),
  bgInput: packColor(35, 38, 48, 255),
  border: packColor(50, 55, 70, 255),
  borderFocus: packColor(100, 140, 220, 255),
  text: packColor(220, 220, 235, 255),
  textMuted: packColor(110, 115, 140, 255),
  textBright: packColor(255, 255, 255, 255),

  // Column header colors
  todo: packColor(220, 160, 80, 255),       // Orange/amber
  progress: packColor(100, 140, 220, 255),  // Blue
  done: packColor(80, 200, 120, 255),       // Green

  accent: packColor(100, 140, 220, 255),
  danger: packColor(220, 80, 80, 255),
}

// =============================================================================
// STATE
// =============================================================================

// Board data
const columns = signal<Column[]>([
  {
    id: 'todo',
    title: 'To Do',
    cards: [
      { id: '1', title: 'Design user interface', createdAt: Date.now() - 100000 },
      { id: '2', title: 'Set up database', createdAt: Date.now() - 80000 },
      { id: '3', title: 'Write documentation', createdAt: Date.now() - 60000 },
    ],
  },
  {
    id: 'progress',
    title: 'In Progress',
    cards: [
      { id: '4', title: 'Implement auth', createdAt: Date.now() - 50000 },
      { id: '5', title: 'Build API endpoints', createdAt: Date.now() - 40000 },
    ],
  },
  {
    id: 'done',
    title: 'Done',
    cards: [
      { id: '6', title: 'Project setup', createdAt: Date.now() - 200000 },
      { id: '7', title: 'Requirements analysis', createdAt: Date.now() - 180000 },
    ],
  },
])

// UI state
const activeColumnId = signal<ColumnId>('todo')
const selectedCardId = signal<string | null>('1')
const isAddingCard = signal(false)
const newCardTitle = signal('')
const isEditingCard = signal(false)
const editCardTitle = signal('')

// =============================================================================
// DERIVED STATE
// =============================================================================

// Get column by ID
function getColumn(id: ColumnId): Column | undefined {
  return columns.value.find(c => c.id === id)
}

// Active column
const activeColumn = derived(() => getColumn(activeColumnId.value))

// Selected card
const selectedCard = derived(() => {
  const id = selectedCardId.value
  if (!id) return null

  for (const col of columns.value) {
    const card = col.cards.find(c => c.id === id)
    if (card) return card
  }
  return null
})

// Find which column a card is in
function findCardColumn(cardId: string): Column | undefined {
  return columns.value.find(col => col.cards.some(c => c.id === cardId))
}

// Column colors
function getColumnColor(id: ColumnId): number {
  switch (id) {
    case 'todo': return colors.todo
    case 'progress': return colors.progress
    case 'done': return colors.done
  }
}

// =============================================================================
// ACTIONS
// =============================================================================

function selectNextCard() {
  const col = activeColumn.value
  if (!col || col.cards.length === 0) return

  const currentIdx = selectedCardId.value
    ? col.cards.findIndex(c => c.id === selectedCardId.value)
    : -1

  const nextIdx = Math.min(currentIdx + 1, col.cards.length - 1)
  selectedCardId.value = col.cards[nextIdx]?.id ?? null
}

function selectPrevCard() {
  const col = activeColumn.value
  if (!col || col.cards.length === 0) return

  const currentIdx = selectedCardId.value
    ? col.cards.findIndex(c => c.id === selectedCardId.value)
    : col.cards.length

  const prevIdx = Math.max(currentIdx - 1, 0)
  selectedCardId.value = col.cards[prevIdx]?.id ?? null
}

function switchColumn(direction: 'left' | 'right') {
  const colIds: ColumnId[] = ['todo', 'progress', 'done']
  const currentIdx = colIds.indexOf(activeColumnId.value)

  let newIdx: number
  if (direction === 'left') {
    newIdx = (currentIdx - 1 + colIds.length) % colIds.length
  } else {
    newIdx = (currentIdx + 1) % colIds.length
  }

  activeColumnId.value = colIds[newIdx]!

  // Select first card in new column if any
  const newCol = getColumn(activeColumnId.value)
  if (newCol && newCol.cards.length > 0) {
    selectedCardId.value = newCol.cards[0]!.id
  } else {
    selectedCardId.value = null
  }
}

function moveCard(direction: 'left' | 'right') {
  const cardId = selectedCardId.value
  if (!cardId) return

  const sourceCol = findCardColumn(cardId)
  if (!sourceCol) return

  const colIds: ColumnId[] = ['todo', 'progress', 'done']
  const sourceIdx = colIds.indexOf(sourceCol.id)

  let targetIdx: number
  if (direction === 'left') {
    targetIdx = sourceIdx - 1
  } else {
    targetIdx = sourceIdx + 1
  }

  // Can't move past edges
  if (targetIdx < 0 || targetIdx >= colIds.length) return

  const targetColId = colIds[targetIdx]!
  const card = sourceCol.cards.find(c => c.id === cardId)
  if (!card) return

  // Move card between columns
  columns.value = columns.value.map(col => {
    if (col.id === sourceCol.id) {
      return { ...col, cards: col.cards.filter(c => c.id !== cardId) }
    }
    if (col.id === targetColId) {
      return { ...col, cards: [...col.cards, card] }
    }
    return col
  })

  // Update active column
  activeColumnId.value = targetColId
}

function addCard() {
  const title = newCardTitle.value.trim()
  if (!title) {
    isAddingCard.value = false
    return
  }

  const newCard: Card = {
    id: String(Date.now()),
    title,
    createdAt: Date.now(),
  }

  columns.value = columns.value.map(col => {
    if (col.id === activeColumnId.value) {
      return { ...col, cards: [...col.cards, newCard] }
    }
    return col
  })

  selectedCardId.value = newCard.id
  newCardTitle.value = ''
  isAddingCard.value = false
}

function deleteCard(cardId: string) {
  columns.value = columns.value.map(col => ({
    ...col,
    cards: col.cards.filter(c => c.id !== cardId),
  }))

  if (selectedCardId.value === cardId) {
    const col = activeColumn.value
    selectedCardId.value = col?.cards[0]?.id ?? null
  }
}

function startEditing() {
  if (!selectedCard.value) return
  editCardTitle.value = selectedCard.value.title
  isEditingCard.value = true
}

function saveEdit() {
  const cardId = selectedCardId.value
  const title = editCardTitle.value.trim()

  if (!cardId || !title) {
    isEditingCard.value = false
    return
  }

  columns.value = columns.value.map(col => ({
    ...col,
    cards: col.cards.map(c =>
      c.id === cardId ? { ...c, title } : c
    ),
  }))

  isEditingCard.value = false
}

function cancelEdit() {
  isEditingCard.value = false
  isAddingCard.value = false
  newCardTitle.value = ''
  editCardTitle.value = ''
}

// =============================================================================
// APP
// =============================================================================

const cols = process.stdout.columns || 100
const rows = process.stdout.rows || 30

const columnWidth = Math.floor((cols - 8) / 3)

mount(() => {
  // Global keyboard handler
  on((event: KeyEvent) => {
    if (!isPress(event)) return false

    const char = getChar(event)

    // Quit
    if (char === 'q' || char === 'Q') {
      process.exit(0)
    }

    // Escape to cancel
    if (isEscape(event)) {
      if (isAddingCard.value || isEditingCard.value) {
        cancelEdit()
        return true
      }
    }

    // Don't process other keys while in input mode
    if (isAddingCard.value || isEditingCard.value) return false

    // Tab to switch columns (cycle through)
    if (event.keycode === 9) {
      switchColumn('right')
      return true
    }

    // Arrow navigation
    if (event.keycode === 0x1b5b42) { // Down
      selectNextCard()
      return true
    }
    if (event.keycode === 0x1b5b41) { // Up
      selectPrevCard()
      return true
    }
    if (event.keycode === 0x1b5b44) { // Left - move card
      moveCard('left')
      return true
    }
    if (event.keycode === 0x1b5b43) { // Right - move card
      moveCard('right')
      return true
    }

    // New card
    if (char === 'n' || char === 'N') {
      isAddingCard.value = true
      return true
    }

    // Delete card
    if ((char === 'd' || char === 'D') && selectedCardId.value) {
      deleteCard(selectedCardId.value)
      return true
    }

    // Edit card
    if (isEnter(event) && selectedCardId.value) {
      startEditing()
      return true
    }

    return false
  })

  // Root container
  box({
    id: 'root',
    width: cols,
    height: rows,
    flexDirection: 'column',
    bg: colors.bg,
    children: () => {
      // Header
      box({
        width: '100%',
        height: 3,
        flexDirection: 'row',
        justifyContent: 'space-between',
        alignItems: 'center',
        paddingLeft: 2,
        paddingRight: 2,
        borderBottom: 1,
        borderColor: colors.border,
        children: () => {
          text({ content: 'Kanban Board', fg: colors.accent })

          // Card count summary
          box({
            flexDirection: 'row',
            gap: 3,
            children: () => {
              for (const col of columns.value) {
                text({
                  content: `${col.title}: ${col.cards.length}`,
                  fg: getColumnColor(col.id),
                })
              }
            },
          })
        },
      })

      // Board content
      box({
        width: '100%',
        grow: 1,
        flexDirection: 'row',
        padding: 1,
        gap: 1,
        children: () => {
          // Render each column
          for (const colId of ['todo', 'progress', 'done'] as ColumnId[]) {
            const id = colId

            box({
              id: `column-${id}`,
              width: columnWidth,
              flexDirection: 'column',
              border: 1,
              borderColor: () => activeColumnId.value === id ? colors.borderFocus : colors.border,
              bg: colors.bgColumn,
              children: () => {
                // Column header
                box({
                  width: '100%',
                  height: 3,
                  flexDirection: 'row',
                  justifyContent: 'space-between',
                  alignItems: 'center',
                  paddingLeft: 1,
                  paddingRight: 1,
                  borderBottom: 1,
                  borderColor: colors.border,
                  bg: () => {
                    const baseColor = getColumnColor(id)
                    // Darken for background
                    return packColor(
                      ((baseColor >> 16) & 0xff) / 3,
                      ((baseColor >> 8) & 0xff) / 3,
                      (baseColor & 0xff) / 3,
                      255
                    )
                  },
                  focusable: true,
                  onClick: () => { activeColumnId.value = id },
                  children: () => {
                    text({
                      content: () => getColumn(id)?.title ?? '',
                      fg: getColumnColor(id),
                    })

                    text({
                      content: () => String(getColumn(id)?.cards.length ?? 0),
                      fg: colors.textMuted,
                    })
                  },
                })

                // Cards list
                box({
                  id: `cards-${id}`,
                  width: '100%',
                  grow: 1,
                  flexDirection: 'column',
                  overflow: 'scroll',
                  padding: 1,
                  gap: 1,
                  children: () => {
                    each(
                      () => getColumn(id)?.cards ?? [],
                      (getItem, key) => {
                        const isSelected = derived(() => selectedCardId.value === key)
                        const isActiveCol = derived(() => activeColumnId.value === id)

                        return box({
                          id: `card-${key}`,
                          width: '100%',
                          minHeight: 3,
                          flexDirection: 'column',
                          padding: 1,
                          bg: () => isSelected.value && isActiveCol.value ? colors.bgCardSelected : colors.bgCard,
                          border: 1,
                          borderColor: () => isSelected.value && isActiveCol.value ? colors.borderFocus : colors.border,
                          focusable: true,
                          onClick: () => {
                            activeColumnId.value = id
                            selectedCardId.value = key
                          },
                          children: () => {
                            // Show edit input or card title
                            show(
                              () => isEditingCard.value && selectedCardId.value === key,
                              () => {
                                return input({
                                  id: `edit-${key}`,
                                  value: editCardTitle,
                                  width: '100%',
                                  bg: colors.bgInput,
                                  fg: colors.text,
                                  border: 1,
                                  borderColor: colors.accent,
                                  autoFocus: true,
                                  onSubmit: saveEdit,
                                  onCancel: cancelEdit,
                                })
                              },
                              () => {
                                return text({
                                  content: () => getItem().title,
                                  fg: () => isSelected.value && isActiveCol.value ? colors.textBright : colors.text,
                                  wrap: 'wrap',
                                })
                              }
                            )
                          },
                        })
                      },
                      { key: (item) => item.id }
                    )

                    // Add card input (shows at bottom when adding)
                    show(
                      () => isAddingCard.value && activeColumnId.value === id,
                      () => {
                        return box({
                          width: '100%',
                          minHeight: 3,
                          flexDirection: 'column',
                          padding: 1,
                          bg: colors.bgCard,
                          border: 1,
                          borderColor: colors.accent,
                          children: () => {
                            input({
                              id: `new-card-${id}`,
                              value: newCardTitle,
                              placeholder: 'Card title...',
                              width: '100%',
                              bg: colors.bgInput,
                              fg: colors.text,
                              autoFocus: true,
                              onSubmit: addCard,
                              onCancel: cancelEdit,
                            })
                          },
                        })
                      }
                    )

                    // Empty state
                    show(
                      () => (getColumn(id)?.cards.length ?? 0) === 0 && !(isAddingCard.value && activeColumnId.value === id),
                      () => {
                        return box({
                          width: '100%',
                          height: 4,
                          justifyContent: 'center',
                          alignItems: 'center',
                          children: () => {
                            text({
                              content: 'No cards',
                              fg: colors.textMuted,
                            })
                          },
                        })
                      }
                    )
                  },
                })
              },
            })
          }
        },
      })

      // Footer with shortcuts
      box({
        width: '100%',
        height: 1,
        flexDirection: 'row',
        justifyContent: 'center',
        borderTop: 1,
        borderColor: colors.border,
        children: () => {
          text({
            content: 'Tab:column  Arrows:select  Left/Right:move card  n:new  d:delete  Enter:edit  q:quit',
            fg: colors.textMuted,
          })
        },
      })
    },
  })
}, { mode: 'fullscreen' })

console.log('[kanban] Started - Press Tab to switch columns')
await new Promise(() => {})
