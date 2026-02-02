/**
 * SparkTUI - Search Box with Autocomplete Example
 *
 * A search UI demonstrating:
 * - Input with search icon prefix
 * - Dropdown suggestions list (using each())
 * - Arrow key navigation through suggestions
 * - Enter to select
 * - Escape to close dropdown
 * - Debounced search (simulated)
 *
 * Run: bun run examples/search-box.ts
 */

import { signal, derived, effect } from '@rlabs-inc/signals'
import { mount } from '../ts/engine/mount'
import { box, text, input, show, each } from '../ts/primitives'
import { on, isPress } from '../ts/state/keyboard'
import type { KeyEvent } from '../ts/state/keyboard'
import { KEY_TAB, KEY_ENTER, KEY_ESCAPE, KEY_UP, KEY_DOWN, hasShift } from '../ts/engine/events'
import { packColor } from '../ts/bridge/shared-buffer'

// =============================================================================
// COLORS
// =============================================================================

const colors = {
  bg: packColor(14, 14, 20, 255),
  surface: packColor(24, 24, 34, 255),
  surfaceHover: packColor(36, 36, 50, 255),
  surfaceSelected: packColor(45, 45, 65, 255),
  border: packColor(55, 55, 75, 255),
  borderFocus: packColor(90, 150, 255, 255),
  text: packColor(220, 220, 235, 255),
  textMuted: packColor(110, 110, 140, 255),
  textBright: packColor(255, 255, 255, 255),
  primary: packColor(90, 150, 255, 255),
  highlight: packColor(255, 220, 100, 255),
  icon: packColor(140, 140, 170, 255),
  badge: packColor(100, 80, 180, 255),
}

// =============================================================================
// SAMPLE DATA
// =============================================================================

interface SearchItem {
  id: string
  title: string
  description: string
  category: string
  icon: string
}

const allItems: SearchItem[] = [
  { id: '1', title: 'Dashboard', description: 'View your dashboard', category: 'Navigation', icon: '[D]' },
  { id: '2', title: 'Settings', description: 'Configure your preferences', category: 'Navigation', icon: '[S]' },
  { id: '3', title: 'Profile', description: 'Edit your profile', category: 'Navigation', icon: '[P]' },
  { id: '4', title: 'Users', description: 'Manage users', category: 'Admin', icon: '[U]' },
  { id: '5', title: 'Analytics', description: 'View analytics data', category: 'Admin', icon: '[A]' },
  { id: '6', title: 'Reports', description: 'Generate reports', category: 'Admin', icon: '[R]' },
  { id: '7', title: 'New Project', description: 'Create a new project', category: 'Actions', icon: '[+]' },
  { id: '8', title: 'Import Data', description: 'Import from file', category: 'Actions', icon: '[I]' },
  { id: '9', title: 'Export Data', description: 'Export to file', category: 'Actions', icon: '[E]' },
  { id: '10', title: 'Documentation', description: 'Read the docs', category: 'Help', icon: '[?]' },
  { id: '11', title: 'Support', description: 'Get help', category: 'Help', icon: '[H]' },
  { id: '12', title: 'Keyboard Shortcuts', description: 'View all shortcuts', category: 'Help', icon: '[K]' },
  { id: '13', title: 'Dark Mode', description: 'Toggle dark mode', category: 'Settings', icon: '[M]' },
  { id: '14', title: 'Notifications', description: 'Notification settings', category: 'Settings', icon: '[N]' },
  { id: '15', title: 'Privacy', description: 'Privacy settings', category: 'Settings', icon: '[V]' },
]

// =============================================================================
// SEARCH STATE
// =============================================================================

const searchQuery = signal('')
const isDropdownOpen = signal(false)
const selectedIndex = signal(0)
const isLoading = signal(false)
const recentSearches = signal<string[]>(['Dashboard', 'Settings', 'Users'])
const selectedItem = signal<SearchItem | null>(null)

// Debounce timer
let debounceTimer: ReturnType<typeof setTimeout> | null = null

// Search results with debounced filtering
const searchResults = derived(() => {
  const query = searchQuery.value.toLowerCase().trim()

  if (query.length === 0) {
    return [] as SearchItem[]
  }

  // Filter items by title, description, or category
  return allItems.filter(item =>
    item.title.toLowerCase().includes(query) ||
    item.description.toLowerCase().includes(query) ||
    item.category.toLowerCase().includes(query)
  )
})

// Group results by category
const groupedResults = derived(() => {
  const results = searchResults.value
  const groups = new Map<string, SearchItem[]>()

  for (const item of results) {
    const existing = groups.get(item.category) ?? []
    existing.push(item)
    groups.set(item.category, existing)
  }

  return groups
})

// Flat list for keyboard navigation
const flatResults = derived(() => searchResults.value)

// =============================================================================
// HANDLERS
// =============================================================================

function handleQueryChange(value: string) {
  searchQuery.value = value

  // Show loading indicator
  if (value.trim().length > 0) {
    isLoading.value = true
    isDropdownOpen.value = true

    // Debounce the search
    if (debounceTimer) clearTimeout(debounceTimer)
    debounceTimer = setTimeout(() => {
      isLoading.value = false
    }, 300)
  } else {
    isDropdownOpen.value = false
    isLoading.value = false
  }

  selectedIndex.value = 0
}

function handleSelect(item: SearchItem) {
  selectedItem.value = item
  isDropdownOpen.value = false

  // Add to recent searches
  const recent = recentSearches.value.filter(s => s !== item.title)
  recentSearches.value = [item.title, ...recent].slice(0, 5)

  console.log(`\nSelected: ${item.title}`)
  console.log(`Description: ${item.description}`)
  console.log(`Category: ${item.category}\n`)

  // Clear search
  searchQuery.value = ''
}

function handleSelectCurrent() {
  const results = flatResults.value
  if (results.length > 0 && selectedIndex.value < results.length) {
    handleSelect(results[selectedIndex.value]!)
  }
}

function navigateUp() {
  const results = flatResults.value
  if (results.length > 0) {
    selectedIndex.value = (selectedIndex.value - 1 + results.length) % results.length
  }
}

function navigateDown() {
  const results = flatResults.value
  if (results.length > 0) {
    selectedIndex.value = (selectedIndex.value + 1) % results.length
  }
}

function clearSearch() {
  searchQuery.value = ''
  isDropdownOpen.value = false
  selectedIndex.value = 0
  selectedItem.value = null
}

// =============================================================================
// UI COMPONENTS
// =============================================================================

function SearchIcon() {
  box({
    width: 3,
    height: 1,
    justifyContent: 'center',
    alignItems: 'center',
    children: () => {
      text({ content: '@', fg: colors.icon })
    },
  })
}

function LoadingSpinner() {
  const frames = ['|', '/', '-', '\\']
  const frameIndex = signal(0)

  // Animate spinner
  const interval = setInterval(() => {
    frameIndex.value = (frameIndex.value + 1) % frames.length
  }, 100)

  // Note: In a real app, we'd clean this up properly

  return box({
    width: 3,
    height: 1,
    justifyContent: 'center',
    alignItems: 'center',
    children: () => {
      text({ content: () => frames[frameIndex.value]!, fg: colors.primary })
    },
  })
}

function RecentSearches() {
  box({
    flexDirection: 'column',
    padding: 1,
    children: () => {
      text({ content: 'Recent Searches', fg: colors.textMuted, marginBottom: 1 })

      each(
        () => recentSearches.value.map((s, i) => ({ id: String(i), text: s })),
        (getItem, key) => {
          return box({
            id: `recent-${key}`,
            flexDirection: 'row',
            alignItems: 'center',
            gap: 1,
            height: 2,
            paddingLeft: 1,
            bg: colors.surface,
            children: () => {
              text({ content: '[>]', fg: colors.icon })
              text({
                content: () => getItem().text,
                fg: colors.text,
              })
            },
            onClick: () => {
              searchQuery.value = getItem().text
              handleQueryChange(getItem().text)
            },
          })
        },
        { key: (item) => item.id }
      )
    },
  })
}

function SearchResultItem(config: {
  item: SearchItem
  index: number
  isSelected: () => boolean
}) {
  const { item, index, isSelected } = config

  box({
    id: `result-${item.id}`,
    flexDirection: 'row',
    alignItems: 'center',
    width: '100%',
    height: 3,
    paddingLeft: 1,
    paddingRight: 1,
    bg: () => isSelected() ? colors.surfaceSelected : colors.surface,
    borderLeft: () => isSelected() ? 2 : 0,
    borderColor: colors.primary,
    children: () => {
      // Icon
      box({
        width: 4,
        height: 1,
        justifyContent: 'center',
        children: () => {
          text({ content: item.icon, fg: colors.icon })
        },
      })

      // Title and description
      box({
        grow: 1,
        flexDirection: 'column',
        children: () => {
          text({
            content: item.title,
            fg: () => isSelected() ? colors.textBright : colors.text,
          })
          text({ content: item.description, fg: colors.textMuted })
        },
      })

      // Category badge
      box({
        paddingLeft: 1,
        paddingRight: 1,
        height: 1,
        bg: colors.badge,
        justifyContent: 'center',
        alignItems: 'center',
        children: () => {
          text({ content: item.category, fg: colors.textBright })
        },
      })
    },
    onClick: () => handleSelect(item),
  })
}

function SearchResults() {
  box({
    width: '100%',
    maxHeight: 20,
    overflow: 'scroll',
    flexDirection: 'column',
    bg: colors.surface,
    border: 1,
    borderColor: colors.border,
    children: () => {
      // Loading state
      show(
        () => isLoading.value,
        () => {
          return box({
            width: '100%',
            height: 3,
            justifyContent: 'center',
            alignItems: 'center',
            children: () => {
              text({ content: 'Searching...', fg: colors.textMuted })
            },
          })
        }
      )

      // No results
      show(
        () => !isLoading.value && searchQuery.value.length > 0 && searchResults.value.length === 0,
        () => {
          return box({
            width: '100%',
            height: 3,
            justifyContent: 'center',
            alignItems: 'center',
            flexDirection: 'column',
            children: () => {
              text({ content: 'No results found', fg: colors.textMuted })
              text({ content: () => `for "${searchQuery.value}"`, fg: colors.textMuted })
            },
          })
        }
      )

      // Results list
      show(
        () => !isLoading.value && searchResults.value.length > 0,
        () => {
          return box({
            flexDirection: 'column',
            children: () => {
              each(
                () => searchResults.value,
                (getItem, key) => {
                  const item = getItem()
                  const idx = searchResults.value.findIndex(i => i.id === item.id)

                  return box({
                    children: () => {
                      SearchResultItem({
                        item,
                        index: idx,
                        isSelected: () => selectedIndex.value === idx,
                      })
                    },
                  })
                },
                { key: (item) => item.id }
              )
            },
          })
        }
      )
    },
  })
}

function SelectedItemDisplay() {
  show(
    () => selectedItem.value !== null,
    () => {
      return box({
        width: '100%',
        flexDirection: 'column',
        border: 1,
        borderColor: colors.primary,
        bg: colors.surfaceHover,
        padding: 1,
        marginTop: 2,
        children: () => {
          text({ content: 'Selected:', fg: colors.textMuted })

          box({
            flexDirection: 'row',
            alignItems: 'center',
            gap: 2,
            marginTop: 1,
            children: () => {
              text({
                content: () => selectedItem.value?.icon ?? '',
                fg: colors.primary,
              })
              box({
                flexDirection: 'column',
                children: () => {
                  text({
                    content: () => selectedItem.value?.title ?? '',
                    fg: colors.textBright,
                  })
                  text({
                    content: () => selectedItem.value?.description ?? '',
                    fg: colors.textMuted,
                  })
                },
              })
            },
          })
        },
      })
    }
  )
}

// =============================================================================
// MOUNT APP
// =============================================================================

const cols = process.stdout.columns || 80
const rows = process.stdout.rows || 24

mount(() => {
  // Global keyboard handler for dropdown navigation
  on((event: KeyEvent) => {
    if (!isPress(event)) return false

    // Arrow navigation when dropdown is open
    if (isDropdownOpen.value) {
      if (event.keycode === KEY_UP) {
        navigateUp()
        return true
      }
      if (event.keycode === KEY_DOWN) {
        navigateDown()
        return true
      }
      if (event.keycode === KEY_ENTER) {
        handleSelectCurrent()
        return true
      }
      if (event.keycode === KEY_ESCAPE) {
        isDropdownOpen.value = false
        return true
      }
    }

    // Global escape to quit
    if (event.keycode === KEY_ESCAPE && !isDropdownOpen.value) {
      process.exit(0)
    }

    // Ctrl+K to focus search (common pattern)
    if (event.keycode === 107 && (event.modifiers & 1)) { // 'k' with ctrl
      // Focus is handled by autoFocus
      return true
    }

    return false
  })

  // Root container
  box({
    id: 'root',
    width: cols,
    height: rows,
    bg: colors.bg,
    flexDirection: 'column',
    alignItems: 'center',
    paddingTop: 3,
    children: () => {
      // Title
      text({
        content: 'Command Palette',
        fg: colors.textBright,
        marginBottom: 1,
      })

      text({
        content: 'Type to search commands, actions, and settings',
        fg: colors.textMuted,
        marginBottom: 2,
      })

      // Search container
      box({
        width: 60,
        flexDirection: 'column',
        children: () => {
          // Search input box
          box({
            width: '100%',
            flexDirection: 'row',
            alignItems: 'center',
            border: 1,
            borderColor: () => isDropdownOpen.value ? colors.borderFocus : colors.border,
            bg: colors.surface,
            children: () => {
              // Search icon
              SearchIcon()

              // Input
              input({
                id: 'search-input',
                value: searchQuery,
                placeholder: 'Search...',
                width: 50,
                bg: 0, // Transparent
                fg: colors.text,
                paddingLeft: 0,
                paddingRight: 1,
                autoFocus: true,
                onChange: handleQueryChange,
                onSubmit: handleSelectCurrent,
                cursor: { style: 'bar', blink: { fps: 2 } },
              })

              // Loading indicator or clear button
              show(
                () => isLoading.value,
                () => LoadingSpinner()
              )

              show(
                () => !isLoading.value && searchQuery.value.length > 0,
                () => {
                  return box({
                    width: 3,
                    height: 1,
                    justifyContent: 'center',
                    alignItems: 'center',
                    children: () => {
                      text({ content: 'x', fg: colors.textMuted })
                    },
                    onClick: clearSearch,
                  })
                }
              )
            },
          })

          // Dropdown
          show(
            () => isDropdownOpen.value,
            () => SearchResults()
          )

          // Recent searches (when dropdown is closed and no query)
          show(
            () => !isDropdownOpen.value && searchQuery.value.length === 0 && recentSearches.value.length > 0,
            () => {
              return box({
                width: '100%',
                marginTop: 1,
                children: () => RecentSearches(),
              })
            }
          )

          // Selected item display
          SelectedItemDisplay()
        },
      })

      // Help text at bottom
      box({
        position: 'absolute',
        bottom: 2,
        width: '100%',
        justifyContent: 'center',
        children: () => {
          text({
            content: '[Up/Down] Navigate  [Enter] Select  [Esc] Close/Exit  [Ctrl+K] Focus',
            fg: colors.textMuted,
          })
        },
      })
    },
  })
}, { mode: 'fullscreen' })

console.log('[search-box] App mounted - Type to search, Press Ctrl+C to exit')
await new Promise(() => {})
