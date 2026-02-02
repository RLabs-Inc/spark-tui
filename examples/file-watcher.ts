/**
 * SparkTUI - File Watcher UI
 *
 * A professional file system watcher demonstrating:
 * - Directory tree view (collapsible)
 * - File list with icons
 * - File size and modification date
 * - Simulated file change events
 * - Event log panel
 * - Filter by extension
 *
 * This is PURELY REACTIVE. No render loops, no polling.
 * Signals update UI automatically through the reactive graph.
 *
 * Run: bun run examples/file-watcher.ts
 */

import { signal, derived } from '@rlabs-inc/signals'
import { mount } from '../ts/engine/mount'
import { box, text, each, show, cycle, pulse, Frames } from '../ts/primitives'
import { onCleanup } from '../ts/primitives/scope'
import { packColor } from '../ts/bridge/shared-buffer'

// =============================================================================
// TYPES
// =============================================================================

type FileType = 'folder' | 'file'
type EventType = 'created' | 'modified' | 'deleted' | 'renamed'

interface FileEntry {
  id: string
  name: string
  type: FileType
  extension?: string
  size: number // bytes
  modified: Date
  expanded?: boolean
  depth: number
  children?: FileEntry[]
}

interface FileEvent {
  id: string
  timestamp: string
  type: EventType
  path: string
  oldPath?: string // for renames
}

// =============================================================================
// COLORS
// =============================================================================

const colors = {
  // Backgrounds
  bgDark: packColor(18, 18, 24, 255),
  bgCard: packColor(28, 28, 38, 255),
  bgHeader: packColor(35, 35, 50, 255),
  bgSelected: packColor(45, 60, 90, 255),
  bgHover: packColor(40, 40, 55, 255),

  // Text
  textPrimary: packColor(240, 240, 250, 255),
  textSecondary: packColor(160, 160, 180, 255),
  textMuted: packColor(100, 100, 120, 255),
  textDim: packColor(70, 70, 90, 255),

  // File types
  folder: packColor(255, 200, 100, 255),
  file: packColor(160, 160, 180, 255),
  typescript: packColor(80, 160, 255, 255),
  javascript: packColor(255, 220, 100, 255),
  json: packColor(180, 130, 255, 255),
  markdown: packColor(100, 200, 150, 255),
  rust: packColor(255, 150, 100, 255),
  config: packColor(200, 200, 200, 255),

  // Events
  created: packColor(80, 220, 140, 255),
  modified: packColor(100, 180, 255, 255),
  deleted: packColor(255, 100, 100, 255),
  renamed: packColor(255, 200, 80, 255),

  // Borders
  borderDim: packColor(50, 50, 70, 255),
  borderAccent: packColor(80, 140, 200, 255),
  borderActive: packColor(100, 180, 255, 255),
}

// =============================================================================
// STATE
// =============================================================================

// Selected file
const selectedFileId = signal<string | null>(null)

// Extension filter
const extensionFilter = signal<string>('all')

// Expanded folders
const expandedFolders = signal<Set<string>>(new Set(['root', 'src', 'examples']))

// File events
let eventIdCounter = 0
const fileEvents = signal<FileEvent[]>([])

// Watch status
const isWatching = signal(true)
const eventsPerMinute = signal(0)

// File tree data
const fileTree = signal<FileEntry>({
  id: 'root',
  name: 'project',
  type: 'folder',
  size: 0,
  modified: new Date(),
  depth: 0,
  expanded: true,
  children: [
    {
      id: 'src',
      name: 'src',
      type: 'folder',
      size: 0,
      modified: new Date(),
      depth: 1,
      expanded: true,
      children: [
        { id: 'src-index', name: 'index.ts', type: 'file', extension: 'ts', size: 2456, modified: new Date(), depth: 2 },
        { id: 'src-app', name: 'app.ts', type: 'file', extension: 'ts', size: 8920, modified: new Date(), depth: 2 },
        { id: 'src-utils', name: 'utils.ts', type: 'file', extension: 'ts', size: 3421, modified: new Date(), depth: 2 },
        {
          id: 'src-components',
          name: 'components',
          type: 'folder',
          size: 0,
          modified: new Date(),
          depth: 2,
          children: [
            { id: 'src-comp-button', name: 'Button.ts', type: 'file', extension: 'ts', size: 1234, modified: new Date(), depth: 3 },
            { id: 'src-comp-input', name: 'Input.ts', type: 'file', extension: 'ts', size: 2345, modified: new Date(), depth: 3 },
            { id: 'src-comp-modal', name: 'Modal.ts', type: 'file', extension: 'ts', size: 3456, modified: new Date(), depth: 3 },
          ],
        },
      ],
    },
    {
      id: 'examples',
      name: 'examples',
      type: 'folder',
      size: 0,
      modified: new Date(),
      depth: 1,
      expanded: true,
      children: [
        { id: 'ex-demo', name: 'demo.ts', type: 'file', extension: 'ts', size: 5678, modified: new Date(), depth: 2 },
        { id: 'ex-test', name: 'test.ts', type: 'file', extension: 'ts', size: 4321, modified: new Date(), depth: 2 },
      ],
    },
    {
      id: 'rust',
      name: 'rust',
      type: 'folder',
      size: 0,
      modified: new Date(),
      depth: 1,
      children: [
        { id: 'rust-lib', name: 'lib.rs', type: 'file', extension: 'rs', size: 12345, modified: new Date(), depth: 2 },
        { id: 'rust-main', name: 'main.rs', type: 'file', extension: 'rs', size: 6789, modified: new Date(), depth: 2 },
        { id: 'rust-cargo', name: 'Cargo.toml', type: 'file', extension: 'toml', size: 890, modified: new Date(), depth: 2 },
      ],
    },
    { id: 'package', name: 'package.json', type: 'file', extension: 'json', size: 1234, modified: new Date(), depth: 1 },
    { id: 'tsconfig', name: 'tsconfig.json', type: 'file', extension: 'json', size: 567, modified: new Date(), depth: 1 },
    { id: 'readme', name: 'README.md', type: 'file', extension: 'md', size: 4567, modified: new Date(), depth: 1 },
    { id: 'gitignore', name: '.gitignore', type: 'file', extension: 'config', size: 234, modified: new Date(), depth: 1 },
  ],
})

// Flattened file list for display
const flattenedFiles = derived(() => {
  const result: FileEntry[] = []
  const expanded = expandedFolders.value

  function flatten(entry: FileEntry) {
    result.push(entry)
    if (entry.type === 'folder' && entry.children && expanded.has(entry.id)) {
      for (const child of entry.children) {
        flatten(child)
      }
    }
  }

  flatten(fileTree.value)
  return result
})

// Filtered files
const filteredFiles = derived(() => {
  const filter = extensionFilter.value
  if (filter === 'all') return flattenedFiles.value
  return flattenedFiles.value.filter(f =>
    f.type === 'folder' || f.extension === filter
  )
})

// =============================================================================
// HELPERS
// =============================================================================

function getTimestamp(): string {
  return new Date().toLocaleTimeString('en-US', { hour12: false })
}

function formatSize(bytes: number): string {
  if (bytes === 0) return '-'
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

function formatDate(date: Date): string {
  return date.toLocaleDateString() + ' ' + date.toLocaleTimeString('en-US', { hour12: false })
}

function getFileIcon(entry: FileEntry): string {
  if (entry.type === 'folder') {
    return expandedFolders.value.has(entry.id) ? '\u{1F4C2}' : '\u{1F4C1}'
  }
  switch (entry.extension) {
    case 'ts': return '\u{1F4DC}'
    case 'js': return '\u{1F4DC}'
    case 'rs': return '\u{2699}'
    case 'json': return '\u{1F4CB}'
    case 'md': return '\u{1F4DD}'
    case 'toml': return '\u{2699}'
    case 'config': return '\u{2699}'
    default: return '\u{1F4C4}'
  }
}

function getFileColor(entry: FileEntry): number {
  if (entry.type === 'folder') return colors.folder
  switch (entry.extension) {
    case 'ts': return colors.typescript
    case 'js': return colors.javascript
    case 'rs': return colors.rust
    case 'json': return colors.json
    case 'md': return colors.markdown
    case 'toml': return colors.config
    default: return colors.file
  }
}

function getEventColor(type: EventType): number {
  switch (type) {
    case 'created': return colors.created
    case 'modified': return colors.modified
    case 'deleted': return colors.deleted
    case 'renamed': return colors.renamed
  }
}

function getEventIcon(type: EventType): string {
  switch (type) {
    case 'created': return '+'
    case 'modified': return '\u2022'
    case 'deleted': return '\u2718'
    case 'renamed': return '\u2192'
  }
}

function addEvent(type: EventType, path: string, oldPath?: string): void {
  if (!isWatching.value) return

  const event: FileEvent = {
    id: `event-${eventIdCounter++}`,
    timestamp: getTimestamp(),
    type,
    path,
    oldPath,
  }
  fileEvents.value = [event, ...fileEvents.value.slice(0, 49)]
}

function toggleFolder(id: string): void {
  const expanded = new Set(expandedFolders.value)
  if (expanded.has(id)) {
    expanded.delete(id)
  } else {
    expanded.add(id)
  }
  expandedFolders.value = expanded
}

// =============================================================================
// SIMULATION
// =============================================================================

function startSimulation(): () => void {
  let eventCount = 0

  // Simulate file changes
  const eventInterval = setInterval(() => {
    if (!isWatching.value) return

    const files = flattenedFiles.value.filter(f => f.type === 'file')
    if (files.length === 0) return

    const randomFile = files[Math.floor(Math.random() * files.length)]!
    const eventTypes: EventType[] = ['modified', 'modified', 'modified', 'created', 'deleted', 'renamed']
    const eventType = eventTypes[Math.floor(Math.random() * eventTypes.length)]!

    const path = getFilePath(randomFile)

    switch (eventType) {
      case 'modified':
        addEvent('modified', path)
        // Update modified time
        randomFile.modified = new Date()
        randomFile.size = Math.max(100, randomFile.size + Math.floor((Math.random() - 0.5) * 500))
        fileTree.value = { ...fileTree.value }
        break
      case 'created':
        addEvent('created', path.replace(randomFile.name, `new_file_${Date.now()}.ts`))
        break
      case 'deleted':
        addEvent('deleted', path)
        break
      case 'renamed':
        const newName = `renamed_${Date.now()}.ts`
        addEvent('renamed', path.replace(randomFile.name, newName), path)
        break
    }

    eventCount++
  }, 2000 + Math.random() * 3000)

  // Track events per minute
  const statsInterval = setInterval(() => {
    eventsPerMinute.value = Math.floor(eventCount * (60 / 5))
    eventCount = 0
  }, 5000)

  return () => {
    clearInterval(eventInterval)
    clearInterval(statsInterval)
  }
}

function getFilePath(entry: FileEntry): string {
  // Simple path reconstruction
  return `/project/${entry.name}`
}

// =============================================================================
// COMPONENTS
// =============================================================================

function FilterBar() {
  const extensions = ['all', 'ts', 'js', 'rs', 'json', 'md']

  box({
    width: '100%',
    height: 3,
    flexDirection: 'row',
    alignItems: 'center',
    gap: 2,
    padding: 1,
    bg: colors.bgCard,
    children: () => {
      text({ content: 'Filter:', fg: colors.textSecondary })

      for (const ext of extensions) {
        box({
          border: 1,
          borderColor: derived(() =>
            extensionFilter.value === ext ? colors.borderActive : colors.borderDim
          ),
          bg: derived(() =>
            extensionFilter.value === ext ? colors.bgSelected : colors.bgCard
          ),
          paddingLeft: 1,
          paddingRight: 1,
          focusable: true,
          onClick: () => {
            extensionFilter.value = ext
          },
          children: () => {
            text({
              content: ext === 'all' ? 'All' : `.${ext}`,
              fg: derived(() =>
                extensionFilter.value === ext ? colors.textPrimary : colors.textSecondary
              ),
            })
          },
        })
      }

      // Spacer
      box({ grow: 1, children: () => {} })

      // File count
      text({
        content: derived(() => {
          const files = filteredFiles.value.filter(f => f.type === 'file')
          return `${files.length} files`
        }),
        fg: colors.textMuted,
      })
    },
  })
}

function FileTreeView() {
  box({
    width: '50%',
    grow: 1,
    overflow: 'scroll',
    border: 1,
    borderColor: colors.borderDim,
    bg: colors.bgDark,
    padding: 1,
    flexDirection: 'column',
    focusable: true,
    children: () => {
      each(
        () => filteredFiles.value,
        (getEntry, key) => {
          const entry = getEntry()
          const isSelected = derived(() => selectedFileId.value === entry.id)
          const indent = '  '.repeat(entry.depth)

          box({
            flexDirection: 'row',
            bg: derived(() => isSelected.value ? colors.bgSelected : colors.bgDark),
            paddingLeft: 1,
            paddingRight: 1,
            focusable: true,
            onClick: () => {
              if (entry.type === 'folder') {
                toggleFolder(entry.id)
              } else {
                selectedFileId.value = entry.id
              }
            },
            children: () => {
              // Indent
              text({ content: indent, fg: colors.textDim })

              // Expand indicator for folders
              show(
                () => entry.type === 'folder',
                () => {
                  text({
                    content: derived(() =>
                      expandedFolders.value.has(entry.id) ? '\u25BC ' : '\u25B6 '
                    ),
                    fg: colors.folder,
                  })
                  return () => {}
                }
              )

              // Icon
              text({
                content: getFileIcon(entry) + ' ',
                fg: getFileColor(entry),
              })

              // Name
              text({
                content: entry.name,
                fg: derived(() => isSelected.value ? colors.textPrimary : getFileColor(entry)),
              })

              // Size (for files)
              show(
                () => entry.type === 'file',
                () => {
                  box({ grow: 1, children: () => {} })
                  text({
                    content: formatSize(entry.size),
                    fg: colors.textDim,
                  })
                  return () => {}
                }
              )
            },
          })
          return () => {}
        },
        { key: entry => entry.id }
      )
    },
  })
}

function EventLog() {
  box({
    width: '50%',
    grow: 1,
    overflow: 'scroll',
    border: 1,
    borderColor: colors.borderDim,
    bg: colors.bgDark,
    padding: 1,
    flexDirection: 'column',
    focusable: true,
    children: () => {
      // Header
      box({
        flexDirection: 'row',
        justifyContent: 'space-between',
        marginBottom: 1,
        children: () => {
          text({ content: 'File Events', fg: colors.textSecondary })
          text({
            content: derived(() => `${eventsPerMinute.value}/min`),
            fg: colors.textMuted,
          })
        },
      })

      each(
        () => fileEvents.value,
        (getEvent, key) => {
          box({
            flexDirection: 'row',
            gap: 2,
            children: () => {
              // Timestamp
              text({
                content: () => getEvent().timestamp,
                fg: colors.textMuted,
              })
              // Type indicator
              text({
                content: () => `[${getEventIcon(getEvent().type)}]`,
                fg: () => getEventColor(getEvent().type),
              })
              // Path
              text({
                content: () => {
                  const ev = getEvent()
                  if (ev.type === 'renamed' && ev.oldPath) {
                    return `${ev.oldPath} -> ${ev.path}`
                  }
                  return ev.path
                },
                fg: colors.textPrimary,
              })
            },
          })
          return () => {}
        },
        { key: ev => ev.id }
      )

      show(
        () => fileEvents.value.length === 0,
        () => {
          text({
            content: 'Waiting for file changes...',
            fg: colors.textMuted,
            marginTop: 1,
          })
          return () => {}
        }
      )
    },
  })
}

function FileDetails() {
  const selectedFile = derived(() => {
    const id = selectedFileId.value
    if (!id) return null
    return flattenedFiles.value.find(f => f.id === id) ?? null
  })

  box({
    width: '100%',
    height: 5,
    border: 1,
    borderColor: colors.borderDim,
    bg: colors.bgCard,
    padding: 1,
    flexDirection: 'column',
    children: () => {
      show(
        () => selectedFile.value !== null,
        () => {
          const file = selectedFile.value!
          box({
            flexDirection: 'row',
            gap: 4,
            children: () => {
              text({
                content: derived(() => `${getFileIcon(file)} ${file.name}`),
                fg: getFileColor(file),
              })
              text({
                content: derived(() => `Size: ${formatSize(file.size)}`),
                fg: colors.textSecondary,
              })
              text({
                content: derived(() => `Modified: ${formatDate(file.modified)}`),
                fg: colors.textSecondary,
              })
              show(
                () => file.extension !== undefined,
                () => {
                  text({
                    content: derived(() => `Type: .${file.extension}`),
                    fg: colors.textMuted,
                  })
                  return () => {}
                }
              )
            },
          })
          return () => {}
        },
        () => {
          text({
            content: 'Select a file to view details',
            fg: colors.textMuted,
          })
          return () => {}
        }
      )
    },
  })
}

// =============================================================================
// MAIN APP
// =============================================================================

const cols = process.stdout.columns || 80
const rows = process.stdout.rows || 24

mount(() => {
  // Start simulation
  const stopSimulation = startSimulation()
  onCleanup(stopSimulation)

  // Root container
  box({
    width: cols,
    height: rows,
    flexDirection: 'column',
    bg: colors.bgDark,
    children: () => {
      // ===== HEADER =====
      box({
        width: '100%',
        height: 3,
        flexDirection: 'row',
        justifyContent: 'space-between',
        alignItems: 'center',
        border: 1,
        borderColor: colors.borderAccent,
        bg: colors.bgHeader,
        paddingLeft: 2,
        paddingRight: 2,
        children: () => {
          box({
            flexDirection: 'row',
            gap: 2,
            alignItems: 'center',
            children: () => {
              text({
                content: cycle(Frames.dots, { fps: 8 }),
                fg: colors.folder,
              })
              text({ content: 'File Watcher', fg: colors.folder })
              text({
                content: derived(() => `(${flattenedFiles.value.length} items)`),
                fg: colors.textMuted,
              })
            },
          })

          // Watch toggle
          box({
            flexDirection: 'row',
            gap: 2,
            alignItems: 'center',
            children: () => {
              const watchPulse = pulse({ fps: 2 })
              show(
                () => isWatching.value,
                () => {
                  text({
                    content: derived(() => watchPulse.value ? '\u25CF' : '\u25CB'),
                    fg: colors.created,
                  })
                  text({ content: 'Watching', fg: colors.created })
                  return () => {}
                },
                () => {
                  text({ content: '\u25CB', fg: colors.textMuted })
                  text({ content: 'Paused', fg: colors.textMuted })
                  return () => {}
                }
              )

              box({
                border: 1,
                borderColor: colors.borderDim,
                paddingLeft: 1,
                paddingRight: 1,
                focusable: true,
                onClick: () => {
                  isWatching.value = !isWatching.value
                },
                children: () => {
                  text({
                    content: derived(() => isWatching.value ? 'Pause' : 'Resume'),
                    fg: colors.textSecondary,
                  })
                },
              })
            },
          })
        },
      })

      // ===== FILTER BAR =====
      FilterBar()

      // ===== MAIN CONTENT =====
      box({
        width: '100%',
        grow: 1,
        flexDirection: 'row',
        padding: 1,
        gap: 1,
        children: () => {
          FileTreeView()
          EventLog()
        },
      })

      // ===== FILE DETAILS =====
      box({
        width: '100%',
        paddingLeft: 1,
        paddingRight: 1,
        children: () => {
          FileDetails()
        },
      })

      // ===== FOOTER =====
      box({
        width: '100%',
        height: 2,
        flexDirection: 'row',
        justifyContent: 'space-between',
        alignItems: 'center',
        paddingLeft: 2,
        paddingRight: 2,
        borderTop: 1,
        borderColor: colors.borderDim,
        children: () => {
          // Legend
          box({
            flexDirection: 'row',
            gap: 3,
            children: () => {
              text({ content: '+ Created', fg: colors.created })
              text({ content: '\u2022 Modified', fg: colors.modified })
              text({ content: '\u2718 Deleted', fg: colors.deleted })
              text({ content: '\u2192 Renamed', fg: colors.renamed })
            },
          })

          text({
            content: 'Press Ctrl+C to exit',
            fg: colors.textDim,
          })
        },
      })
    },
  })
}, {
  mode: 'fullscreen',
})

console.log('[file-watcher] App mounted')

// Keep process alive
await new Promise(() => {})
