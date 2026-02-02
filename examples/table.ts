/**
 * SparkTUI - Data Table
 *
 * A beautiful data table showcasing:
 * - Column headers with sorting indicators
 * - Alternating row colors
 * - Scrollable content
 * - Selection highlight
 * - Keyboard navigation
 *
 * Controls:
 *   ↑/↓     Navigate rows
 *   Enter   Select row
 *   s       Sort by current column
 *   Tab     Switch column
 *   q       Quit
 *
 * Run: bun run examples/table.ts
 */

import { signal, derived } from '@rlabs-inc/signals'
import { mount } from '../ts/engine/mount'
import { box } from '../ts/primitives/box'
import { text } from '../ts/primitives/text'
import { each } from '../ts/primitives/each'
import { isArrowUp, isArrowDown, isEnter, getChar } from '../ts/engine/events'
import type { RGBA } from '../ts/types'

// =============================================================================
// COLORS
// =============================================================================

function rgba(r: number, g: number, b: number, a: number = 255): RGBA {
  return { r, g, b, a }
}

const colors = {
  bgDark: rgba(18, 18, 24),
  bgCard: rgba(26, 26, 36),
  bgRow: rgba(30, 30, 42),
  bgRowAlt: rgba(36, 36, 50),
  bgRowHover: rgba(45, 45, 65),
  bgRowSelected: rgba(60, 100, 140),
  bgHeader: rgba(40, 60, 90),

  textPrimary: rgba(235, 235, 245),
  textSecondary: rgba(160, 160, 180),
  textMuted: rgba(100, 100, 120),

  cyan: rgba(80, 200, 255),
  green: rgba(80, 220, 140),
  yellow: rgba(255, 200, 80),
  red: rgba(255, 100, 100),
  purple: rgba(180, 130, 255),

  borderDim: rgba(50, 50, 65),
  borderAccent: rgba(80, 140, 200),
}

// =============================================================================
// DATA
// =============================================================================

interface User {
  id: number
  name: string
  email: string
  role: 'admin' | 'user' | 'guest'
  status: 'active' | 'inactive' | 'pending'
  lastLogin: string
  usage: number  // percentage
}

const initialUsers: User[] = [
  { id: 1, name: 'Alice Johnson', email: 'alice@example.com', role: 'admin', status: 'active', lastLogin: '2024-01-15', usage: 85 },
  { id: 2, name: 'Bob Smith', email: 'bob@example.com', role: 'user', status: 'active', lastLogin: '2024-01-14', usage: 42 },
  { id: 3, name: 'Carol Williams', email: 'carol@example.com', role: 'user', status: 'inactive', lastLogin: '2024-01-10', usage: 12 },
  { id: 4, name: 'David Brown', email: 'david@example.com', role: 'guest', status: 'pending', lastLogin: '2024-01-13', usage: 5 },
  { id: 5, name: 'Eve Davis', email: 'eve@example.com', role: 'admin', status: 'active', lastLogin: '2024-01-15', usage: 92 },
  { id: 6, name: 'Frank Miller', email: 'frank@example.com', role: 'user', status: 'active', lastLogin: '2024-01-12', usage: 67 },
  { id: 7, name: 'Grace Wilson', email: 'grace@example.com', role: 'user', status: 'inactive', lastLogin: '2024-01-08', usage: 23 },
  { id: 8, name: 'Henry Taylor', email: 'henry@example.com', role: 'guest', status: 'active', lastLogin: '2024-01-11', usage: 31 },
  { id: 9, name: 'Ivy Anderson', email: 'ivy@example.com', role: 'user', status: 'pending', lastLogin: '2024-01-09', usage: 8 },
  { id: 10, name: 'Jack Thomas', email: 'jack@example.com', role: 'admin', status: 'active', lastLogin: '2024-01-15', usage: 78 },
  { id: 11, name: 'Karen Jackson', email: 'karen@example.com', role: 'user', status: 'active', lastLogin: '2024-01-14', usage: 55 },
  { id: 12, name: 'Leo White', email: 'leo@example.com', role: 'user', status: 'inactive', lastLogin: '2024-01-06', usage: 15 },
  { id: 13, name: 'Mia Harris', email: 'mia@example.com', role: 'guest', status: 'active', lastLogin: '2024-01-13', usage: 28 },
  { id: 14, name: 'Noah Martin', email: 'noah@example.com', role: 'user', status: 'active', lastLogin: '2024-01-15', usage: 61 },
  { id: 15, name: 'Olivia Garcia', email: 'olivia@example.com', role: 'admin', status: 'active', lastLogin: '2024-01-15', usage: 88 },
]

// =============================================================================
// STATE
// =============================================================================

const users = signal<User[]>(initialUsers)
const selectedIndex = signal(0)
const sortColumn = signal<keyof User>('id')
const sortDirection = signal<'asc' | 'desc'>('asc')
const currentColumnIndex = signal(0)

const columns: Array<{ key: keyof User; label: string; width: number }> = [
  { key: 'id', label: 'ID', width: 4 },
  { key: 'name', label: 'Name', width: 16 },
  { key: 'email', label: 'Email', width: 22 },
  { key: 'role', label: 'Role', width: 8 },
  { key: 'status', label: 'Status', width: 10 },
  { key: 'lastLogin', label: 'Last Login', width: 12 },
  { key: 'usage', label: 'Usage', width: 10 },
]

// Derived sorted users
const sortedUsers = derived(() => {
  const col = sortColumn.value
  const dir = sortDirection.value
  const sorted = [...users.value]

  sorted.sort((a, b) => {
    const aVal = a[col]
    const bVal = b[col]

    if (typeof aVal === 'string' && typeof bVal === 'string') {
      return dir === 'asc'
        ? aVal.localeCompare(bVal)
        : bVal.localeCompare(aVal)
    }
    if (typeof aVal === 'number' && typeof bVal === 'number') {
      return dir === 'asc' ? aVal - bVal : bVal - aVal
    }
    return 0
  })

  return sorted
})

// =============================================================================
// HELPERS
// =============================================================================

function statusColor(status: User['status']): RGBA {
  switch (status) {
    case 'active': return colors.green
    case 'inactive': return colors.red
    case 'pending': return colors.yellow
  }
}

function roleColor(role: User['role']): RGBA {
  switch (role) {
    case 'admin': return colors.purple
    case 'user': return colors.cyan
    case 'guest': return colors.textMuted
  }
}

function usageBar(pct: number): string {
  const width = 6
  const filled = Math.floor((pct / 100) * width)
  return '█'.repeat(filled) + '░'.repeat(width - filled)
}

function usageColor(pct: number): RGBA {
  if (pct >= 80) return colors.green
  if (pct >= 40) return colors.yellow
  return colors.red
}

// =============================================================================
// ACTIONS
// =============================================================================

function moveSelection(delta: number) {
  const maxIndex = sortedUsers.value.length - 1
  selectedIndex.value = Math.max(0, Math.min(maxIndex, selectedIndex.value + delta))
}

function toggleSort(col: keyof User) {
  if (sortColumn.value === col) {
    sortDirection.value = sortDirection.value === 'asc' ? 'desc' : 'asc'
  } else {
    sortColumn.value = col
    sortDirection.value = 'asc'
  }
}

function nextColumn() {
  currentColumnIndex.value = (currentColumnIndex.value + 1) % columns.length
}

// =============================================================================
// MAIN TABLE
// =============================================================================

const cols = process.stdout.columns || 80
const rows = process.stdout.rows || 24

await mount(() => {
  box({
    width: cols,
    height: rows,
    flexDirection: 'column',
    bg: colors.bgDark,
    padding: 1,
    onKey: (key) => {
      if (isArrowUp(key)) {
        moveSelection(-1)
        return true
      }
      if (isArrowDown(key)) {
        moveSelection(1)
        return true
      }
      const ch = getChar(key)
      if (ch === 's' || ch === 'S') {
        toggleSort(columns[currentColumnIndex.value]!.key)
        return true
      }
      if (ch === '\t') {
        nextColumn()
        return true
      }
      if (ch === 'q' || ch === 'Q') {
        process.exit(0)
      }
      return false
    },
    children: () => {
      // Title
      box({
        width: '100%',
        height: 3,
        flexDirection: 'row',
        justifyContent: 'space-between',
        alignItems: 'center',
        border: 1,
        borderColor: colors.borderAccent,
        bg: colors.bgCard,
        paddingLeft: 2,
        paddingRight: 2,
        children: () => {
          text({ content: '  User Management', fg: colors.cyan })

          box({
            flexDirection: 'row',
            gap: 2,
            children: () => {
              text({
                content: derived(() => `${users.value.length} users`),
                fg: colors.textSecondary,
              })
              text({
                content: derived(() => `Sort: ${sortColumn.value} ${sortDirection.value === 'asc' ? '↑' : '↓'}`),
                fg: colors.textMuted,
              })
            },
          })
        },
      })

      // Table container
      box({
        width: '100%',
        grow: 1,
        flexDirection: 'column',
        border: 1,
        borderColor: colors.borderDim,
        bg: colors.bgCard,
        marginTop: 1,
        overflow: 'scroll',
        children: () => {
          // Header row
          box({
            width: '100%',
            height: 1,
            flexDirection: 'row',
            bg: colors.bgHeader,
            paddingLeft: 1,
            paddingRight: 1,
            children: () => {
              for (let i = 0; i < columns.length; i++) {
                const col = columns[i]!
                const isCurrentCol = i === currentColumnIndex.value
                const isSortCol = sortColumn.value === col.key

                box({
                  width: col.width,
                  flexDirection: 'row',
                  children: () => {
                    text({
                      content: derived(() => {
                        let label = col.label
                        if (isSortCol) {
                          label += sortDirection.value === 'asc' ? ' ↑' : ' ↓'
                        }
                        return label.padEnd(col.width)
                      }),
                      fg: isCurrentCol ? colors.cyan : colors.textPrimary,
                    })
                  },
                })
              }
            },
          })

          // Separator
          box({
            width: '100%',
            height: 1,
            children: () => {
              text({
                content: '─'.repeat(cols - 4),
                fg: colors.borderDim,
              })
            },
          })

          // Data rows
          box({
            width: '100%',
            flexDirection: 'column',
            paddingLeft: 1,
            paddingRight: 1,
            children: () => {
              each(
                () => sortedUsers.value,
                (getUser, key) => {
                  const user = getUser()
                  const index = sortedUsers.value.findIndex(u => u.id === user.id)
                  const isSelected = index === selectedIndex.value
                  const isAlt = index % 2 === 1

                  return box({
                    width: '100%',
                    height: 1,
                    flexDirection: 'row',
                    bg: derived(() => {
                      if (index === selectedIndex.value) return colors.bgRowSelected
                      return isAlt ? colors.bgRowAlt : colors.bgRow
                    }),
                    children: () => {
                      // ID
                      text({
                        content: String(user.id).padEnd(columns[0]!.width),
                        fg: colors.textMuted,
                      })

                      // Name
                      text({
                        content: user.name.slice(0, columns[1]!.width - 1).padEnd(columns[1]!.width),
                        fg: colors.textPrimary,
                      })

                      // Email
                      text({
                        content: user.email.slice(0, columns[2]!.width - 1).padEnd(columns[2]!.width),
                        fg: colors.textSecondary,
                      })

                      // Role (with color)
                      text({
                        content: user.role.padEnd(columns[3]!.width),
                        fg: roleColor(user.role),
                      })

                      // Status (with color + indicator)
                      box({
                        width: columns[4]!.width,
                        flexDirection: 'row',
                        gap: 1,
                        children: () => {
                          text({
                            content: user.status === 'active' ? '●' : user.status === 'pending' ? '◐' : '○',
                            fg: statusColor(user.status),
                          })
                          text({
                            content: user.status.padEnd(8),
                            fg: statusColor(user.status),
                          })
                        },
                      })

                      // Last Login
                      text({
                        content: user.lastLogin.padEnd(columns[5]!.width),
                        fg: colors.textMuted,
                      })

                      // Usage (bar + percentage)
                      box({
                        width: columns[6]!.width,
                        flexDirection: 'row',
                        gap: 1,
                        children: () => {
                          text({
                            content: usageBar(user.usage),
                            fg: usageColor(user.usage),
                          })
                          text({
                            content: `${user.usage}%`,
                            fg: colors.textMuted,
                          })
                        },
                      })
                    },
                  })
                },
                { key: (user) => String(user.id) }
              )
            },
          })
        },
      })

      // Footer
      box({
        width: '100%',
        height: 2,
        flexDirection: 'column',
        justifyContent: 'center',
        alignItems: 'center',
        marginTop: 1,
        children: () => {
          text({
            content: '↑/↓ Navigate  Tab Switch Column  s Sort  Enter Select  q Quit',
            fg: colors.textMuted,
          })
          text({
            content: derived(() => {
              const user = sortedUsers.value[selectedIndex.value]
              return user ? `Selected: ${user.name} (${user.email})` : ''
            }),
            fg: colors.cyan,
          })
        },
      })
    },
  })
}, {
  mode: 'fullscreen',
})
