/**
 * SparkTUI - Notes App
 *
 * A simple note-taking application demonstrating:
 * - Two-pane layout (list + editor)
 * - Reactive lists with each()
 * - Text input and editing
 * - Search/filter functionality
 * - Auto-save indicator
 * - Timestamps
 *
 * Controls:
 *   Tab        Switch between list and editor
 *   Up/Down    Navigate notes list
 *   Enter      Select note / Save note (in editor)
 *   n          Create new note
 *   d          Delete selected note
 *   /          Focus search
 *   Esc        Clear search / Cancel edit
 *   q          Quit
 */

import { signal, derived, effect } from '@rlabs-inc/signals'
import { mount } from '../ts/engine/mount'
import { box, text, input, each, show } from '../ts/primitives'
import { packColor } from '../ts/bridge/shared-buffer'
import { on, isPress } from '../ts/state/keyboard'
import type { KeyEvent } from '../ts/state/keyboard'
import { getChar, isEnter, isEscape } from '../ts/engine/events'

// =============================================================================
// TYPES
// =============================================================================

interface Note {
  id: string
  title: string
  content: string
  createdAt: number
  updatedAt: number
}

type Panel = 'list' | 'editor' | 'search'

// =============================================================================
// COLORS
// =============================================================================

const colors = {
  bg: packColor(22, 22, 30, 255),
  bgPanel: packColor(28, 28, 38, 255),
  bgSelected: packColor(45, 45, 65, 255),
  bgInput: packColor(35, 35, 50, 255),
  border: packColor(55, 55, 75, 255),
  borderFocus: packColor(100, 140, 220, 255),
  text: packColor(220, 220, 235, 255),
  textMuted: packColor(110, 110, 140, 255),
  textBright: packColor(255, 255, 255, 255),
  accent: packColor(100, 140, 220, 255),
  success: packColor(80, 200, 120, 255),
  warning: packColor(220, 180, 80, 255),
  danger: packColor(220, 80, 80, 255),
}

// =============================================================================
// STATE
// =============================================================================

// Notes data
const notes = signal<Note[]>([
  {
    id: '1',
    title: 'Welcome to Notes',
    content: 'This is a simple note-taking app built with SparkTUI.\n\nUse the keyboard to navigate:\n- Tab to switch panels\n- Arrow keys to select notes\n- n to create new\n- d to delete',
    createdAt: Date.now() - 3600000,
    updatedAt: Date.now() - 1800000,
  },
  {
    id: '2',
    title: 'SparkTUI Features',
    content: 'SparkTUI offers:\n- Reactive signals\n- Flexbox layout\n- Keyboard & mouse input\n- Theme system\n- Animation primitives',
    createdAt: Date.now() - 7200000,
    updatedAt: Date.now() - 7200000,
  },
  {
    id: '3',
    title: 'Ideas',
    content: 'Future improvements:\n- Rich text editing\n- Export to markdown\n- Cloud sync\n- Tags and categories',
    createdAt: Date.now() - 86400000,
    updatedAt: Date.now() - 43200000,
  },
])

// UI state
const activePanel = signal<Panel>('list')
const selectedNoteId = signal<string | null>('1')
const searchQuery = signal('')
const editingContent = signal('')

// Auto-save state
const isSaving = signal(false)
const lastSaved = signal<number | null>(null)

// =============================================================================
// DERIVED STATE
// =============================================================================

// Filtered notes based on search
const filteredNotes = derived(() => {
  const query = searchQuery.value.toLowerCase().trim()
  if (!query) return notes.value

  return notes.value.filter(note =>
    note.title.toLowerCase().includes(query) ||
    note.content.toLowerCase().includes(query)
  )
})

// Selected note
const selectedNote = derived(() => {
  const id = selectedNoteId.value
  if (!id) return null
  return notes.value.find(n => n.id === id) ?? null
})

// Note count text
const noteCountText = derived(() => {
  const total = notes.value.length
  const filtered = filteredNotes.value.length
  if (searchQuery.value) {
    return `${filtered}/${total} notes`
  }
  return `${total} note${total === 1 ? '' : 's'}`
})

// Format relative time
function formatRelativeTime(timestamp: number): string {
  const diff = Date.now() - timestamp
  const mins = Math.floor(diff / 60000)
  const hours = Math.floor(diff / 3600000)
  const days = Math.floor(diff / 86400000)

  if (mins < 1) return 'just now'
  if (mins < 60) return `${mins}m ago`
  if (hours < 24) return `${hours}h ago`
  return `${days}d ago`
}

// Auto-save status text
const saveStatusText = derived(() => {
  if (isSaving.value) return 'Saving...'
  if (lastSaved.value) return `Saved ${formatRelativeTime(lastSaved.value)}`
  return ''
})

// =============================================================================
// ACTIONS
// =============================================================================

function createNote() {
  const newNote: Note = {
    id: String(Date.now()),
    title: 'Untitled Note',
    content: '',
    createdAt: Date.now(),
    updatedAt: Date.now(),
  }

  notes.value = [newNote, ...notes.value]
  selectedNoteId.value = newNote.id
  editingContent.value = ''
  activePanel.value = 'editor'
}

function deleteNote(id: string) {
  notes.value = notes.value.filter(n => n.id !== id)
  if (selectedNoteId.value === id) {
    selectedNoteId.value = notes.value[0]?.id ?? null
    if (selectedNoteId.value) {
      const note = notes.value.find(n => n.id === selectedNoteId.value)
      editingContent.value = note?.content ?? ''
    }
  }
}

function selectNote(id: string) {
  selectedNoteId.value = id
  const note = notes.value.find(n => n.id === id)
  editingContent.value = note?.content ?? ''
}

function selectNextNote() {
  const list = filteredNotes.value
  if (list.length === 0) return

  const currentIdx = selectedNoteId.value
    ? list.findIndex(n => n.id === selectedNoteId.value)
    : -1

  const nextIdx = Math.min(currentIdx + 1, list.length - 1)
  selectNote(list[nextIdx]!.id)
}

function selectPrevNote() {
  const list = filteredNotes.value
  if (list.length === 0) return

  const currentIdx = selectedNoteId.value
    ? list.findIndex(n => n.id === selectedNoteId.value)
    : 1

  const prevIdx = Math.max(currentIdx - 1, 0)
  selectNote(list[prevIdx]!.id)
}

function saveNote() {
  const id = selectedNoteId.value
  if (!id) return

  isSaving.value = true

  // Simulate async save
  setTimeout(() => {
    notes.value = notes.value.map(n => {
      if (n.id !== id) return n

      const content = editingContent.value
      const lines = content.split('\n')
      const title = lines[0]?.trim() || 'Untitled Note'

      return {
        ...n,
        title,
        content,
        updatedAt: Date.now(),
      }
    })

    isSaving.value = false
    lastSaved.value = Date.now()
  }, 300)
}

// Auto-save when content changes
let saveTimeout: ReturnType<typeof setTimeout> | null = null

effect(() => {
  const _ = editingContent.value // Track changes
  if (saveTimeout) clearTimeout(saveTimeout)
  saveTimeout = setTimeout(() => {
    if (selectedNoteId.value && editingContent.value !== selectedNote.value?.content) {
      saveNote()
    }
  }, 1000)
})

// =============================================================================
// APP
// =============================================================================

const cols = process.stdout.columns || 100
const rows = process.stdout.rows || 30

const listWidth = 30
const editorWidth = cols - listWidth - 6

mount(() => {
  // Global keyboard handler
  on((event: KeyEvent) => {
    if (!isPress(event)) return false

    const char = getChar(event)

    // Quit
    if (char === 'q' || char === 'Q') {
      process.exit(0)
    }

    // Tab to switch panels
    if (event.keycode === 9) {
      if (activePanel.value === 'list') {
        activePanel.value = 'editor'
      } else if (activePanel.value === 'editor') {
        activePanel.value = 'list'
      } else if (activePanel.value === 'search') {
        activePanel.value = 'list'
      }
      return true
    }

    // Search shortcut
    if (char === '/') {
      activePanel.value = 'search'
      return true
    }

    // Escape to clear/cancel
    if (isEscape(event)) {
      if (searchQuery.value) {
        searchQuery.value = ''
      }
      activePanel.value = 'list'
      return true
    }

    // Panel-specific shortcuts
    if (activePanel.value === 'list') {
      // Arrow navigation
      if (event.keycode === 0x1b5b42) { // Down
        selectNextNote()
        return true
      }
      if (event.keycode === 0x1b5b41) { // Up
        selectPrevNote()
        return true
      }

      // New note
      if (char === 'n' || char === 'N') {
        createNote()
        return true
      }

      // Delete note
      if ((char === 'd' || char === 'D') && selectedNoteId.value) {
        deleteNote(selectedNoteId.value)
        return true
      }

      // Enter to edit
      if (isEnter(event) && selectedNoteId.value) {
        activePanel.value = 'editor'
        return true
      }
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
        bg: colors.bgPanel,
        children: () => {
          text({ content: 'Notes', fg: colors.accent })

          // Save status
          text({
            content: saveStatusText,
            fg: () => isSaving.value ? colors.warning : colors.success,
          })
        },
      })

      // Main content area
      box({
        width: '100%',
        grow: 1,
        flexDirection: 'row',
        children: () => {
          // Left panel - Notes list
          box({
            id: 'list-panel',
            width: listWidth,
            flexDirection: 'column',
            borderRight: 1,
            borderColor: () => activePanel.value === 'list' ? colors.borderFocus : colors.border,
            bg: colors.bgPanel,
            children: () => {
              // Search box
              box({
                width: '100%',
                padding: 1,
                borderBottom: 1,
                borderColor: colors.border,
                children: () => {
                  input({
                    id: 'search-input',
                    value: searchQuery,
                    placeholder: '/ Search notes...',
                    width: '100%',
                    border: 1,
                    borderColor: () => activePanel.value === 'search' ? colors.borderFocus : colors.border,
                    bg: colors.bgInput,
                    fg: colors.text,
                    paddingLeft: 1,
                    onFocus: () => { activePanel.value = 'search' },
                  })
                },
              })

              // Note count
              box({
                width: '100%',
                paddingLeft: 1,
                paddingRight: 1,
                paddingTop: 1,
                children: () => {
                  text({
                    content: noteCountText,
                    fg: colors.textMuted,
                  })
                },
              })

              // Notes list
              box({
                id: 'notes-list',
                width: '100%',
                grow: 1,
                flexDirection: 'column',
                overflow: 'scroll',
                children: () => {
                  // Empty state
                  show(
                    () => filteredNotes.value.length === 0,
                    () => {
                      return box({
                        width: '100%',
                        padding: 2,
                        justifyContent: 'center',
                        alignItems: 'center',
                        children: () => {
                          text({
                            content: searchQuery.value ? 'No matches' : 'No notes yet',
                            fg: colors.textMuted,
                          })
                        },
                      })
                    }
                  )

                  // Note items
                  each(
                    () => filteredNotes.value,
                    (getItem, key) => {
                      return box({
                        id: `note-${key}`,
                        width: '100%',
                        flexDirection: 'column',
                        padding: 1,
                        bg: () => selectedNoteId.value === key ? colors.bgSelected : undefined,
                        borderBottom: 1,
                        borderColor: colors.border,
                        focusable: true,
                        onClick: () => selectNote(key),
                        children: () => {
                          // Title
                          text({
                            content: () => getItem().title,
                            fg: () => selectedNoteId.value === key ? colors.textBright : colors.text,
                            wrap: 'truncate',
                          })

                          // Preview and timestamp
                          box({
                            flexDirection: 'row',
                            justifyContent: 'space-between',
                            marginTop: 0,
                            children: () => {
                              // Content preview
                              text({
                                content: () => {
                                  const content = getItem().content
                                  const preview = content.split('\n').slice(1).join(' ').trim()
                                  return preview.substring(0, 20) || '(empty)'
                                },
                                fg: colors.textMuted,
                                wrap: 'truncate',
                              })

                              // Timestamp
                              text({
                                content: () => formatRelativeTime(getItem().updatedAt),
                                fg: colors.textMuted,
                              })
                            },
                          })
                        },
                      })
                    },
                    { key: (item) => item.id }
                  )
                },
              })

              // List footer with shortcuts
              box({
                width: '100%',
                padding: 1,
                borderTop: 1,
                borderColor: colors.border,
                children: () => {
                  text({
                    content: 'n:new  d:delete',
                    fg: colors.textMuted,
                  })
                },
              })
            },
          })

          // Right panel - Editor
          box({
            id: 'editor-panel',
            grow: 1,
            flexDirection: 'column',
            bg: colors.bgPanel,
            children: () => {
              // Editor header
              box({
                width: '100%',
                padding: 1,
                borderBottom: 1,
                borderColor: () => activePanel.value === 'editor' ? colors.borderFocus : colors.border,
                children: () => {
                  show(
                    () => selectedNote.value !== null,
                    () => {
                      return box({
                        flexDirection: 'row',
                        justifyContent: 'space-between',
                        width: '100%',
                        children: () => {
                          text({
                            content: () => selectedNote.value?.title ?? '',
                            fg: colors.textBright,
                          })
                          text({
                            content: () => `Created ${formatRelativeTime(selectedNote.value?.createdAt ?? 0)}`,
                            fg: colors.textMuted,
                          })
                        },
                      })
                    },
                    () => {
                      return text({
                        content: 'Select a note to edit',
                        fg: colors.textMuted,
                      })
                    }
                  )
                },
              })

              // Editor content
              box({
                width: '100%',
                grow: 1,
                padding: 1,
                children: () => {
                  show(
                    () => selectedNote.value !== null,
                    () => {
                      return input({
                        id: 'editor-input',
                        value: editingContent,
                        placeholder: 'Start typing...',
                        width: '100%',
                        height: rows - 8,
                        bg: colors.bgInput,
                        fg: colors.text,
                        border: 1,
                        borderColor: () => activePanel.value === 'editor' ? colors.borderFocus : colors.border,
                        padding: 1,
                        onFocus: () => { activePanel.value = 'editor' },
                      })
                    },
                    () => {
                      return box({
                        width: '100%',
                        height: '100%',
                        justifyContent: 'center',
                        alignItems: 'center',
                        flexDirection: 'column',
                        children: () => {
                          text({ content: 'No note selected', fg: colors.textMuted })
                          text({ content: 'Press n to create a new note', fg: colors.textMuted })
                        },
                      })
                    }
                  )
                },
              })
            },
          })
        },
      })

      // Footer
      box({
        width: '100%',
        height: 1,
        flexDirection: 'row',
        justifyContent: 'center',
        borderTop: 1,
        borderColor: colors.border,
        bg: colors.bgPanel,
        children: () => {
          text({
            content: 'Tab:switch panel  /:search  Esc:cancel  q:quit',
            fg: colors.textMuted,
          })
        },
      })
    },
  })
}, { mode: 'fullscreen' })

// Initialize editor content
if (selectedNoteId.value) {
  const note = notes.value.find(n => n.id === selectedNoteId.value)
  editingContent.value = note?.content ?? ''
}

console.log('[notes] Started - Press Tab to switch panels')
await new Promise(() => {})
