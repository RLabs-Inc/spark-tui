/**
 * SparkTUI - Settings Panel Example
 *
 * A settings UI demonstrating:
 * - Grouped settings sections
 * - Toggle switches (on/off)
 * - Selection lists (single choice)
 * - Text inputs for custom values
 * - Save/Reset buttons
 * - Live preview of changes
 *
 * Run: bun run examples/settings-panel.ts
 */

import { signal, derived } from '@rlabs-inc/signals'
import { mount } from '../ts/engine/mount'
import { box, text, input, show, each } from '../ts/primitives'
import { on, isPress } from '../ts/state/keyboard'
import type { KeyEvent } from '../ts/state/keyboard'
import { KEY_TAB, KEY_ENTER, KEY_ESCAPE, KEY_UP, KEY_DOWN, hasShift } from '../ts/engine/events'
import { packColor } from '../ts/bridge/shared-buffer'

// =============================================================================
// COLORS (theme-aware preview)
// =============================================================================

const baseColors = {
  bg: packColor(12, 12, 18, 255),
  surface: packColor(22, 22, 32, 255),
  surfaceAlt: packColor(30, 30, 42, 255),
  surfaceHover: packColor(40, 40, 55, 255),
  border: packColor(50, 50, 70, 255),
  borderFocus: packColor(100, 160, 255, 255),
  text: packColor(215, 215, 230, 255),
  textMuted: packColor(105, 105, 135, 255),
  textBright: packColor(255, 255, 255, 255),
  primary: packColor(100, 160, 255, 255),
  success: packColor(75, 195, 115, 255),
  warning: packColor(255, 175, 65, 255),
  error: packColor(255, 85, 85, 255),
  toggleOn: packColor(75, 195, 115, 255),
  toggleOff: packColor(70, 70, 90, 255),
}

// =============================================================================
// SETTINGS STATE
// =============================================================================

// Appearance settings
const theme = signal<'dark' | 'light' | 'auto'>('dark')
const accentColor = signal<'blue' | 'purple' | 'green' | 'orange'>('blue')
const fontSize = signal<'small' | 'medium' | 'large'>('medium')

// Editor settings
const tabSize = signal('4')
const wordWrap = signal(true)
const lineNumbers = signal(true)
const minimap = signal(false)

// Notifications
const soundEnabled = signal(true)
const desktopNotifs = signal(true)
const emailDigest = signal<'none' | 'daily' | 'weekly'>('daily')

// Privacy
const telemetry = signal(false)
const crashReports = signal(true)

// Track unsaved changes
const hasChanges = signal(false)
const savedMessage = signal('')

// Focus management
type SettingId = string
const focusedSetting = signal<SettingId>('theme-dark')

// All focusable items
const allSettings: SettingId[] = [
  // Appearance
  'theme-dark', 'theme-light', 'theme-auto',
  'accent-blue', 'accent-purple', 'accent-green', 'accent-orange',
  'fontSize-small', 'fontSize-medium', 'fontSize-large',
  // Editor
  'tabSize', 'wordWrap', 'lineNumbers', 'minimap',
  // Notifications
  'soundEnabled', 'desktopNotifs', 'emailDigest-none', 'emailDigest-daily', 'emailDigest-weekly',
  // Privacy
  'telemetry', 'crashReports',
  // Actions
  'save', 'reset',
]

// =============================================================================
// NAVIGATION
// =============================================================================

function focusNext() {
  const idx = allSettings.indexOf(focusedSetting.value)
  const nextIdx = (idx + 1) % allSettings.length
  focusedSetting.value = allSettings[nextIdx]!
}

function focusPrev() {
  const idx = allSettings.indexOf(focusedSetting.value)
  const prevIdx = (idx - 1 + allSettings.length) % allSettings.length
  focusedSetting.value = allSettings[prevIdx]!
}

function markChanged() {
  hasChanges.value = true
  savedMessage.value = ''
}

// =============================================================================
// ACTIONS
// =============================================================================

function handleSave() {
  console.log('\n=== Settings Saved ===')
  console.log(`Theme: ${theme.value}`)
  console.log(`Accent: ${accentColor.value}`)
  console.log(`Font Size: ${fontSize.value}`)
  console.log(`Tab Size: ${tabSize.value}`)
  console.log(`Word Wrap: ${wordWrap.value}`)
  console.log(`Line Numbers: ${lineNumbers.value}`)
  console.log(`Minimap: ${minimap.value}`)
  console.log(`Sound: ${soundEnabled.value}`)
  console.log(`Desktop Notifs: ${desktopNotifs.value}`)
  console.log(`Email Digest: ${emailDigest.value}`)
  console.log(`Telemetry: ${telemetry.value}`)
  console.log(`Crash Reports: ${crashReports.value}`)
  console.log('======================\n')

  hasChanges.value = false
  savedMessage.value = 'Settings saved successfully!'

  setTimeout(() => {
    savedMessage.value = ''
  }, 3000)
}

function handleReset() {
  theme.value = 'dark'
  accentColor.value = 'blue'
  fontSize.value = 'medium'
  tabSize.value = '4'
  wordWrap.value = true
  lineNumbers.value = true
  minimap.value = false
  soundEnabled.value = true
  desktopNotifs.value = true
  emailDigest.value = 'daily'
  telemetry.value = false
  crashReports.value = true
  hasChanges.value = false
  savedMessage.value = 'Settings reset to defaults'

  setTimeout(() => {
    savedMessage.value = ''
  }, 3000)
}

// =============================================================================
// UI COMPONENTS
// =============================================================================

function SectionHeader(title: string) {
  box({
    width: '100%',
    marginTop: 1,
    marginBottom: 1,
    borderBottom: 1,
    borderColor: baseColors.border,
    paddingBottom: 0,
    children: () => {
      text({ content: title, fg: baseColors.primary })
    },
  })
}

function Toggle(config: {
  id: SettingId
  label: string
  description?: string
  value: typeof wordWrap
}) {
  const { id, label, description, value } = config
  const isFocused = () => focusedSetting.value === id

  box({
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    width: '100%',
    height: 2,
    paddingLeft: 1,
    paddingRight: 1,
    bg: () => isFocused() ? baseColors.surfaceHover : 0,
    children: () => {
      // Label side
      box({
        flexDirection: 'column',
        children: () => {
          text({
            content: label,
            fg: () => isFocused() ? baseColors.textBright : baseColors.text,
          })
          if (description) {
            text({ content: description, fg: baseColors.textMuted })
          }
        },
      })

      // Toggle switch
      box({
        width: 8,
        height: 1,
        border: 1,
        borderColor: () => isFocused() ? baseColors.borderFocus : baseColors.border,
        bg: () => value.value ? baseColors.toggleOn : baseColors.toggleOff,
        justifyContent: 'center',
        alignItems: 'center',
        focusable: true,
        onFocus: () => { focusedSetting.value = id },
        onClick: () => { value.value = !value.value; markChanged() },
        onKey: (event: KeyEvent) => {
          if (event.keycode === KEY_ENTER || event.keycode === 32) {
            value.value = !value.value
            markChanged()
            return true
          }
          return false
        },
        children: () => {
          text({
            content: () => value.value ? 'ON' : 'OFF',
            fg: baseColors.textBright,
          })
        },
      })
    },
  })
}

function RadioGroup<T extends string>(config: {
  label: string
  options: { id: SettingId; value: T; label: string }[]
  selected: { value: T }
  setSelected: (v: T) => void
}) {
  const { label, options, selected, setSelected } = config

  box({
    flexDirection: 'column',
    width: '100%',
    marginBottom: 1,
    children: () => {
      text({ content: label, fg: baseColors.textMuted, marginBottom: 0 })

      box({
        flexDirection: 'row',
        gap: 1,
        flexWrap: 'wrap',
        children: () => {
          for (const opt of options) {
            const isFocused = () => focusedSetting.value === opt.id
            const isSelected = () => selected.value === opt.value

            box({
              width: 12,
              height: 1,
              border: 1,
              borderColor: () => {
                if (isFocused()) return baseColors.borderFocus
                if (isSelected()) return baseColors.primary
                return baseColors.border
              },
              bg: () => isSelected() ? baseColors.surfaceAlt : baseColors.surface,
              justifyContent: 'center',
              alignItems: 'center',
              focusable: true,
              onFocus: () => { focusedSetting.value = opt.id },
              onClick: () => { setSelected(opt.value); markChanged() },
              onKey: (event: KeyEvent) => {
                if (event.keycode === KEY_ENTER || event.keycode === 32) {
                  setSelected(opt.value)
                  markChanged()
                  return true
                }
                return false
              },
              children: () => {
                text({
                  content: opt.label,
                  fg: () => isSelected() || isFocused() ? baseColors.textBright : baseColors.text,
                })
              },
            })
          }
        },
      })
    },
  })
}

function TextSetting(config: {
  id: SettingId
  label: string
  value: typeof tabSize
  width?: number
}) {
  const { id, label, value, width = 10 } = config
  const isFocused = () => focusedSetting.value === id

  box({
    flexDirection: 'row',
    alignItems: 'center',
    gap: 2,
    marginBottom: 1,
    paddingLeft: 1,
    children: () => {
      text({
        content: label,
        fg: () => isFocused() ? baseColors.textBright : baseColors.text,
      })

      input({
        id,
        value,
        width,
        border: 1,
        borderColor: () => isFocused() ? baseColors.borderFocus : baseColors.border,
        bg: baseColors.surface,
        fg: baseColors.text,
        paddingLeft: 1,
        onFocus: () => { focusedSetting.value = id },
        onChange: () => markChanged(),
      })
    },
  })
}

function ActionButton(config: {
  id: SettingId
  label: string
  primary?: boolean
  disabled?: () => boolean
  onClick: () => void
}) {
  const { id, label, primary, disabled, onClick } = config
  const isFocused = () => focusedSetting.value === id
  const isDisabled = disabled ?? (() => false)

  box({
    width: 14,
    height: 3,
    border: 1,
    borderColor: () => {
      if (isDisabled()) return baseColors.border
      if (isFocused()) return primary ? baseColors.primary : baseColors.borderFocus
      return baseColors.border
    },
    bg: () => {
      if (isDisabled()) return baseColors.surface
      if (primary && isFocused()) return baseColors.primary
      if (isFocused()) return baseColors.surfaceHover
      return baseColors.surface
    },
    justifyContent: 'center',
    alignItems: 'center',
    focusable: true,
    onFocus: () => { focusedSetting.value = id },
    onClick: () => { if (!isDisabled()) onClick() },
    onKey: (event: KeyEvent) => {
      if (event.keycode === KEY_ENTER && !isDisabled()) {
        onClick()
        return true
      }
      return false
    },
    children: () => {
      text({
        content: label,
        fg: () => {
          if (isDisabled()) return baseColors.textMuted
          if (primary && isFocused()) return baseColors.textBright
          return baseColors.text
        },
      })
    },
  })
}

// =============================================================================
// LIVE PREVIEW
// =============================================================================

function LivePreview() {
  // Compute preview colors based on current settings
  const previewBg = derived(() => {
    switch (theme.value) {
      case 'light': return packColor(245, 245, 250, 255)
      case 'dark': return packColor(22, 22, 32, 255)
      default: return packColor(22, 22, 32, 255)
    }
  })

  const previewText = derived(() => {
    switch (theme.value) {
      case 'light': return packColor(30, 30, 40, 255)
      default: return packColor(215, 215, 230, 255)
    }
  })

  const previewAccent = derived(() => {
    switch (accentColor.value) {
      case 'blue': return packColor(100, 160, 255, 255)
      case 'purple': return packColor(160, 100, 255, 255)
      case 'green': return packColor(100, 200, 130, 255)
      case 'orange': return packColor(255, 160, 80, 255)
    }
  })

  const previewFontSize = derived(() => {
    switch (fontSize.value) {
      case 'small': return 'Small Text'
      case 'large': return 'Large Text'
      default: return 'Medium Text'
    }
  })

  box({
    width: '100%',
    flexDirection: 'column',
    border: 1,
    borderColor: baseColors.border,
    children: () => {
      // Preview header
      box({
        width: '100%',
        height: 1,
        bg: previewAccent,
        justifyContent: 'center',
        children: () => {
          text({ content: 'Live Preview', fg: baseColors.textBright })
        },
      })

      // Preview content
      box({
        width: '100%',
        height: 8,
        bg: previewBg,
        padding: 1,
        flexDirection: 'column',
        gap: 1,
        children: () => {
          text({ content: previewFontSize, fg: previewText })

          box({
            flexDirection: 'row',
            gap: 2,
            children: () => {
              text({ content: 'Accent color:', fg: previewText })
              box({
                width: 4,
                height: 1,
                bg: previewAccent,
              })
            },
          })

          box({
            flexDirection: 'row',
            gap: 1,
            children: () => {
              show(
                () => lineNumbers.value,
                () => text({ content: '1', fg: baseColors.textMuted })
              )
              text({
                content: () => `${'  '.repeat(parseInt(tabSize.value) || 4)}// Tab: ${tabSize.value} spaces`,
                fg: previewText,
              })
            },
          })

          show(
            () => minimap.value,
            () => box({
              width: 6,
              height: 3,
              bg: baseColors.surfaceAlt,
              border: 1,
              borderColor: baseColors.border,
              justifyContent: 'center',
              children: () => text({ content: 'map', fg: baseColors.textMuted }),
            })
          )
        },
      })
    },
  })
}

// =============================================================================
// MOUNT APP
// =============================================================================

const cols = process.stdout.columns || 100
const rows = process.stdout.rows || 30

mount(() => {
  // Global keyboard handler
  on((event: KeyEvent) => {
    if (!isPress(event)) return false

    if (event.keycode === KEY_TAB) {
      if (hasShift(event)) {
        focusPrev()
      } else {
        focusNext()
      }
      return true
    }

    if (event.keycode === KEY_ESCAPE) {
      if (hasChanges.value) {
        console.log('Unsaved changes discarded')
      }
      process.exit(0)
    }

    // Ctrl+S to save
    if (event.keycode === 115 && (event.modifiers & 1)) { // 's' with ctrl
      handleSave()
      return true
    }

    return false
  })

  // Root container
  box({
    id: 'root',
    width: cols,
    height: rows,
    bg: baseColors.bg,
    flexDirection: 'row',
    children: () => {
      // Settings panel (left)
      box({
        width: 55,
        height: '100%',
        flexDirection: 'column',
        border: 1,
        borderColor: baseColors.border,
        children: () => {
          // Header
          box({
            width: '100%',
            height: 3,
            justifyContent: 'space-between',
            alignItems: 'center',
            paddingLeft: 2,
            paddingRight: 2,
            borderBottom: 1,
            borderColor: baseColors.border,
            bg: baseColors.surfaceAlt,
            children: () => {
              text({ content: 'Settings', fg: baseColors.textBright })

              show(
                () => hasChanges.value,
                () => text({ content: '(unsaved)', fg: baseColors.warning })
              )
            },
          })

          // Scrollable settings area
          box({
            width: '100%',
            grow: 1,
            overflow: 'scroll',
            flexDirection: 'column',
            padding: 1,
            children: () => {
              // APPEARANCE SECTION
              SectionHeader('Appearance')

              RadioGroup({
                label: 'Theme',
                options: [
                  { id: 'theme-dark', value: 'dark', label: 'Dark' },
                  { id: 'theme-light', value: 'light', label: 'Light' },
                  { id: 'theme-auto', value: 'auto', label: 'Auto' },
                ],
                selected: theme,
                setSelected: (v) => { theme.value = v },
              })

              RadioGroup({
                label: 'Accent Color',
                options: [
                  { id: 'accent-blue', value: 'blue', label: 'Blue' },
                  { id: 'accent-purple', value: 'purple', label: 'Purple' },
                  { id: 'accent-green', value: 'green', label: 'Green' },
                  { id: 'accent-orange', value: 'orange', label: 'Orange' },
                ],
                selected: accentColor,
                setSelected: (v) => { accentColor.value = v },
              })

              RadioGroup({
                label: 'Font Size',
                options: [
                  { id: 'fontSize-small', value: 'small', label: 'Small' },
                  { id: 'fontSize-medium', value: 'medium', label: 'Medium' },
                  { id: 'fontSize-large', value: 'large', label: 'Large' },
                ],
                selected: fontSize,
                setSelected: (v) => { fontSize.value = v },
              })

              // EDITOR SECTION
              SectionHeader('Editor')

              TextSetting({
                id: 'tabSize',
                label: 'Tab Size:',
                value: tabSize,
                width: 6,
              })

              Toggle({ id: 'wordWrap', label: 'Word Wrap', value: wordWrap })
              Toggle({ id: 'lineNumbers', label: 'Line Numbers', value: lineNumbers })
              Toggle({ id: 'minimap', label: 'Show Minimap', value: minimap })

              // NOTIFICATIONS SECTION
              SectionHeader('Notifications')

              Toggle({ id: 'soundEnabled', label: 'Sound Effects', value: soundEnabled })
              Toggle({ id: 'desktopNotifs', label: 'Desktop Notifications', value: desktopNotifs })

              RadioGroup({
                label: 'Email Digest',
                options: [
                  { id: 'emailDigest-none', value: 'none', label: 'None' },
                  { id: 'emailDigest-daily', value: 'daily', label: 'Daily' },
                  { id: 'emailDigest-weekly', value: 'weekly', label: 'Weekly' },
                ],
                selected: emailDigest,
                setSelected: (v) => { emailDigest.value = v },
              })

              // PRIVACY SECTION
              SectionHeader('Privacy')

              Toggle({
                id: 'telemetry',
                label: 'Usage Analytics',
                description: 'Help improve the app',
                value: telemetry,
              })

              Toggle({
                id: 'crashReports',
                label: 'Crash Reports',
                description: 'Automatically send crash reports',
                value: crashReports,
              })
            },
          })

          // Action buttons
          box({
            width: '100%',
            height: 5,
            flexDirection: 'column',
            borderTop: 1,
            borderColor: baseColors.border,
            padding: 1,
            gap: 1,
            children: () => {
              // Status message
              show(
                () => savedMessage.value !== '',
                () => text({
                  content: () => savedMessage.value,
                  fg: baseColors.success,
                })
              )

              box({
                flexDirection: 'row',
                justifyContent: 'flex-end',
                gap: 2,
                children: () => {
                  ActionButton({
                    id: 'reset',
                    label: 'Reset',
                    onClick: handleReset,
                  })

                  ActionButton({
                    id: 'save',
                    label: 'Save',
                    primary: true,
                    disabled: () => !hasChanges.value,
                    onClick: handleSave,
                  })
                },
              })
            },
          })
        },
      })

      // Preview panel (right)
      box({
        grow: 1,
        height: '100%',
        flexDirection: 'column',
        padding: 2,
        children: () => {
          text({ content: 'Preview', fg: baseColors.textMuted, marginBottom: 1 })
          LivePreview()

          // Help text
          box({
            marginTop: 2,
            flexDirection: 'column',
            gap: 0,
            children: () => {
              text({ content: 'Keyboard Shortcuts', fg: baseColors.textMuted })
              text({ content: '[Tab] Next setting', fg: baseColors.textMuted })
              text({ content: '[Shift+Tab] Previous', fg: baseColors.textMuted })
              text({ content: '[Enter/Space] Toggle/Select', fg: baseColors.textMuted })
              text({ content: '[Ctrl+S] Save', fg: baseColors.textMuted })
              text({ content: '[Esc] Exit', fg: baseColors.textMuted })
            },
          })
        },
      })
    },
  })
}, { mode: 'fullscreen' })

console.log('[settings-panel] App mounted - Press Ctrl+C to exit')
await new Promise(() => {})
