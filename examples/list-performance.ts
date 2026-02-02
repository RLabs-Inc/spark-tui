/**
 * SparkTUI Large List Performance Test
 *
 * Performance showcase demonstrating efficient handling of 10,000+ items
 * with smooth scrolling, instant search/filter, and add/remove operations.
 *
 * Key concepts demonstrated:
 * - Virtual scrolling via overflow: scroll
 * - Instant search filtering
 * - Add/remove operations on large lists
 * - each() handling massive item counts
 * - Reactive filtering with derived signals
 *
 * Controls:
 * - Type to search
 * - Up/Down: Navigate selection
 * - Enter: Select item
 * - a: Add random items
 * - d: Delete selected
 * - r: Reset list
 * - Tab: Focus search
 * - Ctrl+C: Exit
 */

import { signal, derived } from '@rlabs-inc/signals'
import { mount } from '../ts/engine/mount'
import { box, text, input, each, cycle, Frames, show } from '../ts/primitives'
import { packColor } from '../ts/bridge/shared-buffer'
import { on, getChar, isPress } from '../ts/state/keyboard'
import type { KeyEvent } from '../ts/state/keyboard'

// =============================================================================
// COLORS
// =============================================================================

const colors = {
  bg: packColor(15, 15, 22, 255),
  bgCard: packColor(25, 25, 35, 255),
  bgItem: packColor(30, 30, 45, 255),
  bgItemHover: packColor(40, 40, 60, 255),
  bgItemSelected: packColor(50, 70, 120, 255),
  border: packColor(60, 60, 90, 255),
  borderActive: packColor(100, 140, 255, 255),
  text: packColor(220, 220, 235, 255),
  textMuted: packColor(100, 100, 130, 255),
  textAccent: packColor(140, 170, 255, 255),
  textSuccess: packColor(100, 220, 140, 255),
  textWarning: packColor(255, 200, 100, 255),
  textError: packColor(255, 120, 120, 255),
}

// =============================================================================
// DATA GENERATION
// =============================================================================

interface ListItem {
  id: string
  name: string
  category: string
  value: number
  active: boolean
}

const categories = ['Electronics', 'Clothing', 'Food', 'Books', 'Sports', 'Home', 'Auto', 'Health']
const adjectives = ['Amazing', 'Premium', 'Basic', 'Deluxe', 'Ultra', 'Super', 'Mega', 'Pro', 'Elite', 'Classic']
const nouns = ['Widget', 'Gadget', 'Device', 'Item', 'Thing', 'Product', 'Tool', 'Gear', 'Kit', 'Set']

function generateItem(index: number): ListItem {
  const adj = adjectives[index % adjectives.length]!
  const noun = nouns[Math.floor(index / adjectives.length) % nouns.length]!
  const cat = categories[index % categories.length]!
  return {
    id: `item-${index}-${Date.now()}`,
    name: `${adj} ${noun} #${index + 1}`,
    category: cat,
    value: Math.floor(Math.random() * 1000) + 10,
    active: Math.random() > 0.3,
  }
}

function generateItems(count: number): ListItem[] {
  const items: ListItem[] = []
  for (let i = 0; i < count; i++) {
    items.push(generateItem(i))
  }
  return items
}

// =============================================================================
// STATE
// =============================================================================

const allItems = signal<ListItem[]>(generateItems(10000))
const searchQuery = signal('')
const selectedIndex = signal(0)
const selectedItem = signal<ListItem | null>(null)

// Stats
const filterTime = signal(0)
const visibleCount = signal(0)

// Filtered items (reactive)
const filteredItems = derived(() => {
  const start = performance.now()
  const query = searchQuery.value.toLowerCase().trim()
  let result: ListItem[]

  if (!query) {
    result = allItems.value
  } else {
    result = allItems.value.filter(item =>
      item.name.toLowerCase().includes(query) ||
      item.category.toLowerCase().includes(query)
    )
  }

  filterTime.value = performance.now() - start
  visibleCount.value = result.length
  return result
})

// Pagination for performance - show only visible items
const pageSize = 100
const currentPage = signal(0)

const paginatedItems = derived(() => {
  const items = filteredItems.value
  const start = currentPage.value * pageSize
  return items.slice(start, start + pageSize)
})

const totalPages = derived(() => Math.ceil(filteredItems.value.length / pageSize))

// =============================================================================
// ACTIONS
// =============================================================================

function addItems(count: number) {
  const newItems = [...allItems.value]
  const baseIndex = newItems.length
  for (let i = 0; i < count; i++) {
    newItems.push(generateItem(baseIndex + i))
  }
  allItems.value = newItems
}

function deleteSelected() {
  if (selectedItem.value) {
    allItems.value = allItems.value.filter(item => item.id !== selectedItem.value!.id)
    selectedItem.value = null
  }
}

function resetList() {
  allItems.value = generateItems(10000)
  searchQuery.value = ''
  selectedIndex.value = 0
  currentPage.value = 0
  selectedItem.value = null
}

function navigateSelection(delta: number) {
  const items = filteredItems.value
  if (items.length === 0) return

  let newIndex = selectedIndex.value + delta
  if (newIndex < 0) newIndex = 0
  if (newIndex >= items.length) newIndex = items.length - 1

  selectedIndex.value = newIndex
  selectedItem.value = items[newIndex] ?? null

  // Update page if needed
  const newPage = Math.floor(newIndex / pageSize)
  if (newPage !== currentPage.value) {
    currentPage.value = newPage
  }
}

function selectCurrentItem() {
  const items = filteredItems.value
  if (items.length > 0 && selectedIndex.value < items.length) {
    selectedItem.value = items[selectedIndex.value] ?? null
  }
}

// =============================================================================
// MOUNT APP
// =============================================================================

const cols = process.stdout.columns || 120
const rows = process.stdout.rows || 40

mount(() => {
  // ─────────────────────────────────────────────────────────────────────────────
  // KEYBOARD HANDLER
  // ─────────────────────────────────────────────────────────────────────────────
  on((event: KeyEvent) => {
    if (!isPress(event)) return false

    const char = getChar(event)

    // Navigation
    if (event.keycode === 0x1b5b41) { // Up
      navigateSelection(-1)
      return true
    }
    if (event.keycode === 0x1b5b42) { // Down
      navigateSelection(1)
      return true
    }
    if (event.keycode === 0x1b5b35) { // Page Up
      navigateSelection(-pageSize)
      return true
    }
    if (event.keycode === 0x1b5b36) { // Page Down
      navigateSelection(pageSize)
      return true
    }
    if (event.keycode === 13) { // Enter
      selectCurrentItem()
      return true
    }

    // Actions
    if (char === 'a' || char === 'A') {
      addItems(100)
      return true
    }
    if (char === 'd' || char === 'D') {
      deleteSelected()
      return true
    }
    if (char === 'r' || char === 'R') {
      resetList()
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
                    content: cycle(Frames.dots, { fps: 10 }),
                    fg: colors.textSuccess,
                  })
                  text({ content: 'SparkTUI Large List Performance', fg: colors.textAccent })
                },
              })
              text({
                content: () => `${allItems.value.length.toLocaleString()} total items`,
                fg: colors.text,
              })
            },
          })

          box({
            flexDirection: 'row',
            gap: 4,
            marginTop: 1,
            children: () => {
              text({
                content: () => `Showing: ${visibleCount.value.toLocaleString()}`,
                fg: colors.text,
              })
              text({
                content: () => `Filter: ${filterTime.value.toFixed(2)}ms`,
                fg: () => filterTime.value < 10 ? colors.textSuccess : filterTime.value < 50 ? colors.textWarning : colors.textError,
              })
              text({
                content: () => `Page: ${currentPage.value + 1}/${totalPages.value}`,
                fg: colors.textMuted,
              })
              show(
                () => selectedItem.value !== null,
                () => text({
                  content: () => `Selected: ${selectedItem.value?.name ?? 'None'}`,
                  fg: colors.textAccent,
                })
              )
            },
          })
        },
      })

      // ─────────────────────────────────────────────────────────────────────────
      // SEARCH BAR
      // ─────────────────────────────────────────────────────────────────────────
      box({
        id: 'search-bar',
        width: '100%',
        height: 3,
        flexDirection: 'row',
        alignItems: 'center',
        gap: 2,
        paddingLeft: 2,
        paddingRight: 2,
        bg: colors.bgCard,
        children: () => {
          text({ content: 'Search:', fg: colors.text })
          input({
            id: 'search-input',
            value: searchQuery,
            placeholder: 'Type to filter...',
            width: 40,
            border: 1,
            borderColor: colors.border,
            bg: colors.bgItem,
            fg: colors.text,
            paddingLeft: 1,
            paddingRight: 1,
            autoFocus: true,
            onChange: () => {
              // Reset selection on search
              selectedIndex.value = 0
              currentPage.value = 0
            },
          })
          text({
            content: () => searchQuery.value ? `Filtering "${searchQuery.value}"...` : 'No filter active',
            fg: colors.textMuted,
          })
        },
      })

      // ─────────────────────────────────────────────────────────────────────────
      // MAIN CONTENT
      // ─────────────────────────────────────────────────────────────────────────
      box({
        grow: 1,
        flexDirection: 'row',
        gap: 1,
        padding: 1,
        children: () => {
          // ─────────────────────────────────────────────────────────────────────
          // LIST PANEL
          // ─────────────────────────────────────────────────────────────────────
          box({
            grow: 2,
            flexDirection: 'column',
            border: 1,
            borderColor: colors.border,
            bg: colors.bgCard,
            children: () => {
              // List header
              box({
                height: 1,
                flexDirection: 'row',
                bg: colors.bgItem,
                paddingLeft: 1,
                gap: 2,
                children: () => {
                  text({ content: 'Name'.padEnd(30), fg: colors.textAccent })
                  text({ content: 'Category'.padEnd(12), fg: colors.textAccent })
                  text({ content: 'Value'.padEnd(8), fg: colors.textAccent })
                  text({ content: 'Status', fg: colors.textAccent })
                },
              })

              // List items (scrollable)
              box({
                grow: 1,
                overflow: 'scroll',
                flexDirection: 'column',
                children: () => {
                  each(
                    () => paginatedItems.value,
                    (getItem, key) => {
                      const item = getItem()
                      const globalIdx = currentPage.value * pageSize + paginatedItems.value.indexOf(item)
                      const isSelected = () => selectedIndex.value === globalIdx

                      return box({
                        id: `list-item-${key}`,
                        height: 1,
                        flexDirection: 'row',
                        bg: () => isSelected() ? colors.bgItemSelected : colors.bgItem,
                        paddingLeft: 1,
                        gap: 2,
                        onClick: () => {
                          selectedIndex.value = globalIdx
                          selectedItem.value = item
                        },
                        children: () => {
                          text({
                            content: () => getItem().name.padEnd(30).slice(0, 30),
                            fg: () => isSelected() ? colors.textAccent : colors.text,
                          })
                          text({
                            content: () => getItem().category.padEnd(12).slice(0, 12),
                            fg: colors.textMuted,
                          })
                          text({
                            content: () => `$${getItem().value}`.padEnd(8),
                            fg: colors.textSuccess,
                          })
                          text({
                            content: () => getItem().active ? 'Active' : 'Inactive',
                            fg: () => getItem().active ? colors.textSuccess : colors.textError,
                          })
                        },
                      })
                    },
                    { key: (item) => item.id }
                  )

                  // Empty state
                  show(
                    () => filteredItems.value.length === 0,
                    () => box({
                      height: 3,
                      justifyContent: 'center',
                      alignItems: 'center',
                      children: () => {
                        text({
                          content: 'No items match your search',
                          fg: colors.textMuted,
                        })
                      },
                    })
                  )
                },
              })
            },
          })

          // ─────────────────────────────────────────────────────────────────────
          // DETAILS PANEL
          // ─────────────────────────────────────────────────────────────────────
          box({
            width: 35,
            flexDirection: 'column',
            border: 1,
            borderColor: colors.border,
            bg: colors.bgCard,
            children: () => {
              // Panel header
              box({
                height: 1,
                bg: colors.bgItem,
                paddingLeft: 1,
                children: () => {
                  text({ content: 'Item Details', fg: colors.textAccent })
                },
              })

              // Details content
              box({
                grow: 1,
                padding: 1,
                flexDirection: 'column',
                gap: 1,
                children: () => {
                  show(
                    () => selectedItem.value !== null,
                    () => box({
                      flexDirection: 'column',
                      gap: 1,
                      children: () => {
                        text({
                          content: () => `ID: ${selectedItem.value?.id ?? ''}`,
                          fg: colors.textMuted,
                        })
                        text({
                          content: () => `Name: ${selectedItem.value?.name ?? ''}`,
                          fg: colors.text,
                        })
                        text({
                          content: () => `Category: ${selectedItem.value?.category ?? ''}`,
                          fg: colors.text,
                        })
                        text({
                          content: () => `Value: $${selectedItem.value?.value ?? 0}`,
                          fg: colors.textSuccess,
                        })
                        text({
                          content: () => `Status: ${selectedItem.value?.active ? 'Active' : 'Inactive'}`,
                          fg: () => selectedItem.value?.active ? colors.textSuccess : colors.textError,
                        })

                        box({
                          marginTop: 2,
                          children: () => {
                            text({
                              content: '[D] Delete this item',
                              fg: colors.textWarning,
                            })
                          },
                        })
                      },
                    })
                  )

                  show(
                    () => selectedItem.value === null,
                    () => box({
                      justifyContent: 'center',
                      alignItems: 'center',
                      grow: 1,
                      children: () => {
                        text({
                          content: 'Select an item',
                          fg: colors.textMuted,
                        })
                      },
                    })
                  )
                },
              })

              // Stats
              box({
                flexDirection: 'column',
                padding: 1,
                borderTop: 1,
                borderColor: colors.border,
                children: () => {
                  text({ content: 'Performance:', fg: colors.textAccent })
                  text({
                    content: () => `Total items: ${allItems.value.length.toLocaleString()}`,
                    fg: colors.textMuted,
                  })
                  text({
                    content: () => `Filtered: ${visibleCount.value.toLocaleString()}`,
                    fg: colors.textMuted,
                  })
                  text({
                    content: () => `Filter time: ${filterTime.value.toFixed(2)}ms`,
                    fg: colors.textMuted,
                  })
                  text({
                    content: () => `Rendered: ${paginatedItems.value.length}`,
                    fg: colors.textMuted,
                  })
                },
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
          text({ content: '[Up/Down] Navigate', fg: colors.textMuted })
          text({ content: '[PgUp/PgDn] Page', fg: colors.textMuted })
          text({ content: '[A] Add 100', fg: colors.textMuted })
          text({ content: '[R] Reset', fg: colors.textMuted })
          text({ content: '[Ctrl+C] Exit', fg: colors.textMuted })
        },
      })
    },
  })
}, { mode: 'fullscreen' })

console.log('[list-performance] App mounted - Type to search, arrows to navigate')
await new Promise(() => {})
