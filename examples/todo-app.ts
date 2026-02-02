/**
 * SparkTUI - Todo App
 *
 * A complete todo list application demonstrating:
 * - Reactive state with signals
 * - List rendering with each()
 * - Conditional rendering with show()
 * - Text input with validation
 * - Keyboard navigation and shortcuts
 * - Filter/tab functionality
 * - Counter displays
 *
 * Controls:
 *   Tab        Navigate between input and todos
 *   Enter      Add new todo (when in input) / Toggle todo (when focused)
 *   d          Delete focused todo
 *   Space      Toggle focused todo
 *   1/2/3      Switch filter: All/Active/Completed
 *   c          Clear completed
 *   q          Quit
 */

import { signal, derived } from '@rlabs-inc/signals'
import { mount } from '../ts/engine/mount'
import { box, text, input, each, show } from '../ts/primitives'
import { packColor } from '../ts/bridge/shared-buffer'
import { on, isPress } from '../ts/state/keyboard'
import type { KeyEvent } from '../ts/state/keyboard'
import { getChar } from '../ts/engine/events'

// =============================================================================
// TYPES
// =============================================================================

interface Todo {
  id: string
  text: string
  completed: boolean
  createdAt: number
}

type Filter = 'all' | 'active' | 'completed'

// =============================================================================
// COLORS
// =============================================================================

const colors = {
  bg: packColor(24, 24, 32, 255),
  bgCard: packColor(32, 32, 44, 255),
  bgInput: packColor(40, 40, 56, 255),
  bgHover: packColor(50, 50, 70, 255),
  border: packColor(60, 60, 80, 255),
  borderFocus: packColor(100, 140, 220, 255),
  text: packColor(220, 220, 235, 255),
  textMuted: packColor(120, 120, 150, 255),
  textCompleted: packColor(80, 80, 100, 255),
  accent: packColor(100, 140, 220, 255),
  success: packColor(80, 200, 120, 255),
  danger: packColor(220, 80, 80, 255),
  warning: packColor(220, 180, 80, 255),
}

// =============================================================================
// STATE
// =============================================================================

// Todo list
const todos = signal<Todo[]>([
  { id: '1', text: 'Learn SparkTUI', completed: true, createdAt: Date.now() - 100000 },
  { id: '2', text: 'Build awesome apps', completed: false, createdAt: Date.now() - 50000 },
  { id: '3', text: 'Share with the world', completed: false, createdAt: Date.now() },
])

// Input state
const newTodoText = signal('')

// Filter state
const currentFilter = signal<Filter>('all')

// Focus state
const focusedTodoId = signal<string | null>(null)

// =============================================================================
// DERIVED STATE
// =============================================================================

// Filtered todos
const filteredTodos = derived(() => {
  const filter = currentFilter.value
  const list = todos.value
  switch (filter) {
    case 'active':
      return list.filter(t => !t.completed)
    case 'completed':
      return list.filter(t => t.completed)
    default:
      return list
  }
})

// Counts
const totalCount = derived(() => todos.value.length)
const activeCount = derived(() => todos.value.filter(t => !t.completed).length)
const completedCount = derived(() => todos.value.filter(t => t.completed).length)

// Items left text
const itemsLeftText = derived(() => {
  const count = activeCount.value
  return `${count} item${count === 1 ? '' : 's'} left`
})

// =============================================================================
// ACTIONS
// =============================================================================

function addTodo() {
  const text = newTodoText.value.trim()
  if (!text) return

  const newTodo: Todo = {
    id: String(Date.now()),
    text,
    completed: false,
    createdAt: Date.now(),
  }

  todos.value = [...todos.value, newTodo]
  newTodoText.value = ''
}

function toggleTodo(id: string) {
  todos.value = todos.value.map(t =>
    t.id === id ? { ...t, completed: !t.completed } : t
  )
}

function deleteTodo(id: string) {
  todos.value = todos.value.filter(t => t.id !== id)
  if (focusedTodoId.value === id) {
    focusedTodoId.value = null
  }
}

function clearCompleted() {
  todos.value = todos.value.filter(t => !t.completed)
}

function focusNextTodo() {
  const list = filteredTodos.value
  if (list.length === 0) return

  const currentIdx = focusedTodoId.value
    ? list.findIndex(t => t.id === focusedTodoId.value)
    : -1

  const nextIdx = (currentIdx + 1) % list.length
  focusedTodoId.value = list[nextIdx]!.id
}

function focusPrevTodo() {
  const list = filteredTodos.value
  if (list.length === 0) return

  const currentIdx = focusedTodoId.value
    ? list.findIndex(t => t.id === focusedTodoId.value)
    : 0

  const prevIdx = (currentIdx - 1 + list.length) % list.length
  focusedTodoId.value = list[prevIdx]!.id
}

// =============================================================================
// APP
// =============================================================================

const cols = process.stdout.columns || 80
const rows = process.stdout.rows || 24

mount(() => {
  // Global keyboard handler
  on((event: KeyEvent) => {
    if (!isPress(event)) return false

    const char = getChar(event)

    // Quit
    if (char === 'q' || char === 'Q') {
      process.exit(0)
    }

    // Filter shortcuts
    if (char === '1') {
      currentFilter.value = 'all'
      return true
    }
    if (char === '2') {
      currentFilter.value = 'active'
      return true
    }
    if (char === '3') {
      currentFilter.value = 'completed'
      return true
    }

    // Clear completed
    if (char === 'c' || char === 'C') {
      clearCompleted()
      return true
    }

    // Arrow navigation in todo list
    if (event.keycode === 0x1b5b42) { // Down
      focusNextTodo()
      return true
    }
    if (event.keycode === 0x1b5b41) { // Up
      focusPrevTodo()
      return true
    }

    // Delete focused todo
    if ((char === 'd' || char === 'D') && focusedTodoId.value) {
      deleteTodo(focusedTodoId.value)
      return true
    }

    // Toggle focused todo with space
    if (event.keycode === 32 && focusedTodoId.value) {
      toggleTodo(focusedTodoId.value)
      return true
    }

    // Toggle with Enter when focused
    if (event.keycode === 13 && focusedTodoId.value) {
      toggleTodo(focusedTodoId.value)
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
    alignItems: 'center',
    padding: 2,
    bg: colors.bg,
    children: () => {
      // Main card
      box({
        id: 'card',
        width: Math.min(60, cols - 4),
        flexDirection: 'column',
        border: 1,
        borderColor: colors.border,
        bg: colors.bgCard,
        children: () => {
          // Header
          box({
            width: '100%',
            flexDirection: 'column',
            alignItems: 'center',
            padding: 1,
            borderBottom: 1,
            borderColor: colors.border,
            children: () => {
              text({
                content: 'Todo App',
                fg: colors.accent,
              })
              text({
                content: 'Keyboard-driven task management',
                fg: colors.textMuted,
              })
            },
          })

          // Input section
          box({
            width: '100%',
            padding: 1,
            borderBottom: 1,
            borderColor: colors.border,
            children: () => {
              box({
                width: '100%',
                flexDirection: 'row',
                gap: 1,
                children: () => {
                  input({
                    id: 'new-todo-input',
                    value: newTodoText,
                    placeholder: 'What needs to be done?',
                    width: '100%',
                    border: 1,
                    borderColor: colors.border,
                    bg: colors.bgInput,
                    fg: colors.text,
                    paddingLeft: 1,
                    paddingRight: 1,
                    autoFocus: true,
                    onSubmit: () => {
                      addTodo()
                    },
                  })
                },
              })
            },
          })

          // Filter tabs
          box({
            width: '100%',
            flexDirection: 'row',
            justifyContent: 'center',
            gap: 2,
            padding: 1,
            borderBottom: 1,
            borderColor: colors.border,
            children: () => {
              const filters: { key: Filter; label: string; shortcut: string }[] = [
                { key: 'all', label: 'All', shortcut: '1' },
                { key: 'active', label: 'Active', shortcut: '2' },
                { key: 'completed', label: 'Done', shortcut: '3' },
              ]

              for (const filter of filters) {
                const f = filter
                box({
                  width: 12,
                  height: 1,
                  justifyContent: 'center',
                  alignItems: 'center',
                  bg: () => currentFilter.value === f.key ? colors.accent : colors.bgInput,
                  fg: () => currentFilter.value === f.key ? colors.bgCard : colors.text,
                  border: 1,
                  borderColor: () => currentFilter.value === f.key ? colors.accent : colors.border,
                  focusable: true,
                  onClick: () => { currentFilter.value = f.key },
                  children: () => {
                    text({ content: `${f.shortcut}:${f.label}` })
                  },
                })
              }
            },
          })

          // Todo list
          box({
            id: 'todo-list',
            width: '100%',
            flexDirection: 'column',
            minHeight: 8,
            maxHeight: 12,
            overflow: 'scroll',
            children: () => {
              // Empty state
              show(
                () => filteredTodos.value.length === 0,
                () => {
                  return box({
                    width: '100%',
                    height: 6,
                    justifyContent: 'center',
                    alignItems: 'center',
                    flexDirection: 'column',
                    children: () => {
                      text({ content: 'No todos yet', fg: colors.textMuted })
                      text({ content: 'Type above and press Enter', fg: colors.textMuted })
                    },
                  })
                }
              )

              // Todo items
              each(
                () => filteredTodos.value,
                (getItem, key) => {
                  return box({
                    id: `todo-${key}`,
                    width: '100%',
                    flexDirection: 'row',
                    alignItems: 'center',
                    padding: 1,
                    gap: 1,
                    bg: () => focusedTodoId.value === key ? colors.bgHover : undefined,
                    borderBottom: 1,
                    borderColor: colors.border,
                    focusable: true,
                    onClick: () => {
                      focusedTodoId.value = key
                      toggleTodo(key)
                    },
                    onFocus: () => { focusedTodoId.value = key },
                    children: () => {
                      // Checkbox
                      text({
                        content: () => getItem().completed ? '\u2611' : '\u2610',
                        fg: () => getItem().completed ? colors.success : colors.textMuted,
                      })

                      // Todo text
                      text({
                        content: () => getItem().text,
                        fg: () => getItem().completed ? colors.textCompleted : colors.text,
                        grow: 1,
                      })

                      // Delete hint (shows when focused)
                      show(
                        () => focusedTodoId.value === key,
                        () => {
                          return text({
                            content: '[d]',
                            fg: colors.danger,
                          })
                        }
                      )
                    },
                  })
                },
                { key: (item) => item.id }
              )
            },
          })

          // Footer
          box({
            width: '100%',
            flexDirection: 'row',
            justifyContent: 'space-between',
            alignItems: 'center',
            padding: 1,
            borderTop: 1,
            borderColor: colors.border,
            children: () => {
              // Items left count
              text({
                content: itemsLeftText,
                fg: colors.textMuted,
              })

              // Clear completed button
              show(
                () => completedCount.value > 0,
                () => {
                  return box({
                    width: 16,
                    height: 1,
                    justifyContent: 'center',
                    bg: colors.bgInput,
                    border: 1,
                    borderColor: colors.border,
                    focusable: true,
                    onClick: clearCompleted,
                    children: () => {
                      text({ content: 'c:Clear done', fg: colors.textMuted })
                    },
                  })
                }
              )
            },
          })
        },
      })

      // Help text
      box({
        marginTop: 1,
        children: () => {
          text({
            content: 'Enter:add  Space:toggle  d:delete  1/2/3:filter  c:clear  q:quit',
            fg: colors.textMuted,
          })
        },
      })
    },
  })
}, { mode: 'fullscreen' })

console.log('[todo-app] Started - Press q to quit')
await new Promise(() => {})
