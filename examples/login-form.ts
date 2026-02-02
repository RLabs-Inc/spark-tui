/**
 * SparkTUI - Login Form Example
 *
 * A beautiful login screen demonstrating:
 * - Username input
 * - Password input (masked with dots)
 * - "Remember me" toggle (focusable box)
 * - Submit/Cancel buttons
 * - Validation messages
 * - Tab navigation between fields
 * - Enter to submit
 *
 * Run: bun run examples/login-form.ts
 */

import { signal, derived } from '@rlabs-inc/signals'
import { mount } from '../ts/engine/mount'
import { box, text, input, show } from '../ts/primitives'
import { on, isPress } from '../ts/state/keyboard'
import type { KeyEvent } from '../ts/state/keyboard'
import { KEY_TAB, KEY_ENTER, KEY_ESCAPE, hasShift } from '../ts/engine/events'
import { packColor } from '../ts/bridge/shared-buffer'

// =============================================================================
// COLORS
// =============================================================================

const colors = {
  bg: packColor(18, 18, 24, 255),
  surface: packColor(28, 28, 38, 255),
  surfaceHover: packColor(38, 38, 52, 255),
  border: packColor(60, 60, 80, 255),
  borderFocus: packColor(100, 140, 255, 255),
  text: packColor(220, 220, 235, 255),
  textMuted: packColor(120, 120, 150, 255),
  textBright: packColor(255, 255, 255, 255),
  primary: packColor(100, 140, 255, 255),
  primaryHover: packColor(120, 160, 255, 255),
  success: packColor(80, 200, 120, 255),
  error: packColor(240, 80, 80, 255),
  toggle: packColor(80, 200, 120, 255),
  toggleOff: packColor(80, 80, 100, 255),
}

// =============================================================================
// FORM STATE
// =============================================================================

const username = signal('')
const password = signal('')
const rememberMe = signal(false)

// Form state
const isSubmitting = signal(false)
const isSubmitted = signal(false)
const errorMessage = signal('')

// Focus tracking
type FieldName = 'username' | 'password' | 'remember' | 'submit' | 'cancel'
const focusedField = signal<FieldName>('username')

const fields: FieldName[] = ['username', 'password', 'remember', 'submit', 'cancel']

// =============================================================================
// VALIDATION
// =============================================================================

const usernameError = derived(() => {
  const val = username.value.trim()
  if (val.length === 0) return null // Not yet typed
  if (val.length < 3) return 'Username must be at least 3 characters'
  return null
})

const passwordError = derived(() => {
  const val = password.value
  if (val.length === 0) return null
  if (val.length < 4) return 'Password must be at least 4 characters'
  return null
})

const isValid = derived(() => {
  return username.value.trim().length >= 3 && password.value.length >= 4
})

// =============================================================================
// NAVIGATION
// =============================================================================

function focusNext() {
  const currentIdx = fields.indexOf(focusedField.value)
  const nextIdx = (currentIdx + 1) % fields.length
  focusedField.value = fields[nextIdx]!
}

function focusPrev() {
  const currentIdx = fields.indexOf(focusedField.value)
  const prevIdx = (currentIdx - 1 + fields.length) % fields.length
  focusedField.value = fields[prevIdx]!
}

// =============================================================================
// FORM ACTIONS
// =============================================================================

function handleSubmit() {
  if (!isValid.value) {
    errorMessage.value = 'Please fill in all fields correctly'
    return
  }

  isSubmitting.value = true
  errorMessage.value = ''

  // Simulate login
  setTimeout(() => {
    isSubmitting.value = false
    isSubmitted.value = true

    console.log('\n=== Login Successful ===')
    console.log(`Username: ${username.value}`)
    console.log(`Remember Me: ${rememberMe.value}`)
    console.log('========================\n')

    // Exit after showing success
    setTimeout(() => process.exit(0), 2000)
  }, 1500)
}

function handleCancel() {
  console.log('Login cancelled')
  process.exit(0)
}

// =============================================================================
// UI COMPONENTS
// =============================================================================

function InputField(config: {
  label: string
  value: typeof username
  placeholder: string
  fieldName: FieldName
  password?: boolean
  error: { readonly value: string | null }
}) {
  const { label, value, placeholder, fieldName, password, error } = config
  const isFocused = () => focusedField.value === fieldName

  box({
    width: '100%',
    flexDirection: 'column',
    marginBottom: 1,
    children: () => {
      // Label
      text({
        content: label,
        fg: () => isFocused() ? colors.textBright : colors.textMuted,
      })

      // Input
      box({
        marginTop: 0,
        children: () => {
          input({
            id: fieldName,
            value,
            placeholder,
            password,
            width: 40,
            border: 1,
            borderColor: () => {
              if (isFocused()) return colors.borderFocus
              if (error.value) return colors.error
              return colors.border
            },
            bg: colors.surface,
            fg: colors.text,
            paddingLeft: 1,
            paddingRight: 1,
            autoFocus: fieldName === 'username',
            onFocus: () => { focusedField.value = fieldName },
            onSubmit: () => {
              if (fieldName === 'password') {
                handleSubmit()
              } else {
                focusNext()
              }
            },
            cursor: {
              style: 'bar',
              blink: { fps: 2 },
            },
          })
        },
      })

      // Error message
      show(
        () => error.value !== null,
        () => {
          return text({
            content: () => error.value ?? '',
            fg: colors.error,
            marginTop: 0,
          })
        }
      )
    },
  })
}

function RememberMeToggle() {
  const isFocused = () => focusedField.value === 'remember'

  box({
    flexDirection: 'row',
    alignItems: 'center',
    gap: 1,
    marginTop: 1,
    marginBottom: 1,
    children: () => {
      // Toggle box
      box({
        width: 4,
        height: 1,
        border: 1,
        borderColor: () => isFocused() ? colors.borderFocus : colors.border,
        bg: () => rememberMe.value ? colors.toggle : colors.toggleOff,
        justifyContent: 'center',
        alignItems: 'center',
        focusable: true,
        onFocus: () => { focusedField.value = 'remember' },
        onClick: () => { rememberMe.value = !rememberMe.value },
        onKey: (event: KeyEvent) => {
          if (event.keycode === KEY_ENTER || event.keycode === 32) { // Enter or Space
            rememberMe.value = !rememberMe.value
            return true
          }
          return false
        },
        children: () => {
          text({
            content: () => rememberMe.value ? 'ON' : '',
            fg: colors.textBright,
          })
        },
      })

      // Label
      text({
        content: 'Remember me',
        fg: () => isFocused() ? colors.textBright : colors.text,
      })
    },
  })
}

function Button(config: {
  label: string
  fieldName: FieldName
  primary?: boolean
  disabled?: () => boolean
  onClick: () => void
}) {
  const { label, fieldName, primary, disabled, onClick } = config
  const isFocused = () => focusedField.value === fieldName
  const isDisabled = disabled ?? (() => false)

  box({
    width: 14,
    height: 3,
    border: 1,
    borderColor: () => {
      if (isDisabled()) return colors.border
      if (isFocused()) return primary ? colors.primary : colors.borderFocus
      return colors.border
    },
    bg: () => {
      if (isDisabled()) return colors.surface
      if (primary && isFocused()) return colors.primary
      if (isFocused()) return colors.surfaceHover
      return colors.surface
    },
    justifyContent: 'center',
    alignItems: 'center',
    focusable: true,
    onFocus: () => { focusedField.value = fieldName },
    onClick: () => {
      if (!isDisabled()) onClick()
    },
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
          if (isDisabled()) return colors.textMuted
          if (primary && isFocused()) return colors.textBright
          return colors.text
        },
      })
    },
  })
}

// =============================================================================
// MOUNT APP
// =============================================================================

const cols = process.stdout.columns || 80
const rows = process.stdout.rows || 24

mount(() => {
  // Global keyboard handler
  on((event: KeyEvent) => {
    if (!isPress(event)) return false

    // Tab navigation
    if (event.keycode === KEY_TAB) {
      if (hasShift(event)) {
        focusPrev()
      } else {
        focusNext()
      }
      return true
    }

    // Escape to cancel
    if (event.keycode === KEY_ESCAPE) {
      handleCancel()
      return true
    }

    return false
  })

  // Root container - center the form
  box({
    id: 'root',
    width: cols,
    height: rows,
    bg: colors.bg,
    justifyContent: 'center',
    alignItems: 'center',
    children: () => {
      // Login card
      box({
        id: 'login-card',
        width: 50,
        flexDirection: 'column',
        border: 1,
        borderColor: colors.border,
        bg: colors.surface,
        children: () => {
          // Header
          box({
            width: '100%',
            height: 3,
            justifyContent: 'center',
            alignItems: 'center',
            borderBottom: 1,
            borderColor: colors.border,
            children: () => {
              text({
                content: 'Welcome Back',
                fg: colors.textBright,
              })
            },
          })

          // Form body
          box({
            width: '100%',
            flexDirection: 'column',
            padding: 2,
            children: () => {
              // Show success or form
              show(
                () => isSubmitted.value,
                () => {
                  return box({
                    width: '100%',
                    height: 10,
                    flexDirection: 'column',
                    justifyContent: 'center',
                    alignItems: 'center',
                    gap: 1,
                    children: () => {
                      text({ content: 'Login Successful!', fg: colors.success })
                      text({ content: `Welcome, ${username.value}!`, fg: colors.text })
                    },
                  })
                },
                () => {
                  return box({
                    flexDirection: 'column',
                    children: () => {
                      // Username field
                      InputField({
                        label: 'Username',
                        value: username,
                        placeholder: 'Enter your username...',
                        fieldName: 'username',
                        error: usernameError,
                      })

                      // Password field
                      InputField({
                        label: 'Password',
                        value: password,
                        placeholder: 'Enter your password...',
                        fieldName: 'password',
                        password: true,
                        error: passwordError,
                      })

                      // Remember me toggle
                      RememberMeToggle()

                      // Error message
                      show(
                        () => errorMessage.value !== '',
                        () => {
                          return text({
                            content: () => errorMessage.value,
                            fg: colors.error,
                            marginBottom: 1,
                          })
                        }
                      )

                      // Buttons
                      box({
                        flexDirection: 'row',
                        justifyContent: 'center',
                        gap: 2,
                        marginTop: 1,
                        children: () => {
                          Button({
                            label: () => isSubmitting.value ? 'Signing in...' : 'Sign In',
                            fieldName: 'submit',
                            primary: true,
                            disabled: () => !isValid.value || isSubmitting.value,
                            onClick: handleSubmit,
                          })

                          Button({
                            label: 'Cancel',
                            fieldName: 'cancel',
                            onClick: handleCancel,
                          })
                        },
                      })
                    },
                  })
                }
              )
            },
          })

          // Footer
          box({
            width: '100%',
            height: 2,
            justifyContent: 'center',
            alignItems: 'center',
            borderTop: 1,
            borderColor: colors.border,
            children: () => {
              text({
                content: '[Tab] Next  [Shift+Tab] Back  [Enter] Submit  [Esc] Cancel',
                fg: colors.textMuted,
              })
            },
          })
        },
      })
    },
  })
}, { mode: 'fullscreen' })

console.log('[login-form] App mounted - Press Ctrl+C to exit')
await new Promise(() => {})
