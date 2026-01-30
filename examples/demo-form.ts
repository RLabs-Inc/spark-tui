/**
 * SparkTUI - Interactive Form Demo
 *
 * Demonstrates:
 * - Input components with placeholders and validation
 * - Password masking
 * - Tab navigation between fields
 * - Live validation with reactive feedback
 * - Conditional rendering with show()
 * - Focus management and visual indicators
 * - Form submission handling
 *
 * A beautiful, polished signup form with real UX patterns.
 */

import { signal, derived, effect } from '@rlabs-inc/signals'
import { mount } from '../ts/engine/mount'
import { box, text, input, show } from '../ts/primitives'
import { t, theme, setTheme } from '../ts/state/theme'
import { focus, focusedIndex } from '../ts/state/focus'
import { on, matchesKey } from '../ts/state/keyboard'
import type { KeyEvent } from '../ts/state/keyboard'
import type { RGBA } from '../ts/types'

// =============================================================================
// FORM STATE
// =============================================================================

// Field values
const username = signal('')
const email = signal('')
const password = signal('')

// Form state
const submitted = signal(false)
const submitting = signal(false)

// Track which field is focused for visual feedback
const currentField = signal<'username' | 'email' | 'password' | 'submit' | null>(null)

// =============================================================================
// VALIDATION
// =============================================================================

// Username validation
const usernameValid = derived(() => {
  const val = username.value.trim()
  if (val.length === 0) return null // Not yet typed
  if (val.length < 3) return { valid: false, message: 'Must be at least 3 characters' }
  if (val.length > 20) return { valid: false, message: 'Must be 20 characters or less' }
  if (!/^[a-zA-Z0-9_]+$/.test(val)) return { valid: false, message: 'Only letters, numbers, and underscores' }
  return { valid: true, message: 'Username available' }
})

// Email validation
const emailValid = derived(() => {
  const val = email.value.trim()
  if (val.length === 0) return null
  if (!val.includes('@')) return { valid: false, message: 'Must contain @' }
  if (!val.includes('.')) return { valid: false, message: 'Must contain a domain' }
  if (val.length < 5) return { valid: false, message: 'Too short' }
  return { valid: true, message: 'Valid email' }
})

// Password validation
const passwordValid = derived(() => {
  const val = password.value
  if (val.length === 0) return null
  if (val.length < 8) return { valid: false, message: 'Must be at least 8 characters' }
  if (!/[A-Z]/.test(val)) return { valid: false, message: 'Needs an uppercase letter' }
  if (!/[a-z]/.test(val)) return { valid: false, message: 'Needs a lowercase letter' }
  if (!/[0-9]/.test(val)) return { valid: false, message: 'Needs a number' }
  return { valid: true, message: 'Strong password' }
})

// Overall form validity
const formValid = derived(() => {
  const u = usernameValid.value
  const e = emailValid.value
  const p = passwordValid.value
  return u?.valid && e?.valid && p?.valid
})

// =============================================================================
// COLORS
// =============================================================================

// Theme-aware colors
const successColor: RGBA = { r: 80, g: 200, b: 120, a: 255 }
const errorColor: RGBA = { r: 240, g: 80, b: 80, a: 255 }
const warningColor: RGBA = { r: 240, g: 180, b: 50, a: 255 }
const mutedColor: RGBA = { r: 100, g: 100, b: 100, a: 255 }
const primaryColor: RGBA = { r: 80, g: 160, b: 240, a: 255 }
const accentColor: RGBA = { r: 200, g: 120, b: 240, a: 255 }
const whiteColor: RGBA = { r: 255, g: 255, b: 255, a: 255 }

// =============================================================================
// TAB NAVIGATION
// =============================================================================

const fieldOrder = ['username', 'email', 'password', 'submit'] as const
type FieldName = typeof fieldOrder[number]

function focusNextField() {
  const current = currentField.value
  if (!current) {
    currentField.value = 'username'
    return
  }
  const idx = fieldOrder.indexOf(current)
  const nextIdx = (idx + 1) % fieldOrder.length
  currentField.value = fieldOrder[nextIdx]
}

function focusPrevField() {
  const current = currentField.value
  if (!current) {
    currentField.value = 'submit'
    return
  }
  const idx = fieldOrder.indexOf(current)
  const prevIdx = (idx - 1 + fieldOrder.length) % fieldOrder.length
  currentField.value = fieldOrder[prevIdx]
}

// =============================================================================
// FORM SUBMISSION
// =============================================================================

function handleSubmit() {
  if (!formValid.value) {
    console.log('Form is not valid')
    return
  }

  submitting.value = true

  // Simulate async submission
  setTimeout(() => {
    submitting.value = false
    submitted.value = true

    console.log('\n--- Form Submitted ---')
    console.log(`Username: ${username.value}`)
    console.log(`Email: ${email.value}`)
    console.log(`Password: ${'*'.repeat(password.value.length)}`)
    console.log('----------------------\n')

    // Exit after showing success
    setTimeout(() => process.exit(0), 2000)
  }, 1000)
}

// =============================================================================
// UI COMPONENTS
// =============================================================================

/**
 * Validation message component
 */
function validationMessage(
  validation: { readonly value: { valid: boolean; message: string } | null }
) {
  return () => {
    // Show nothing if not validated yet
    show(
      () => validation.value !== null,
      () => {
        // Success message
        show(
          () => validation.value?.valid === true,
          () => text({
            content: () => `  ✓ ${validation.value?.message ?? ''}`,
            fg: successColor,
          })
        )

        // Error message
        show(
          () => validation.value?.valid === false,
          () => text({
            content: () => `  ✗ ${validation.value?.message ?? ''}`,
            fg: errorColor,
          })
        )

        return () => {}
      }
    )
    return () => {}
  }
}

/**
 * Form field with label, input, and validation
 */
function formField(config: {
  label: string
  value: ReturnType<typeof signal<string>>
  placeholder: string
  validation: typeof usernameValid
  fieldName: FieldName
  password?: boolean
}) {
  const { label, value, placeholder, validation, fieldName, password } = config

  // Border color based on focus and validation state
  const borderColor = derived((): RGBA => {
    const focused = currentField.value === fieldName
    const v = validation.value

    if (focused) return primaryColor
    if (v?.valid === true) return successColor
    if (v?.valid === false) return errorColor
    return mutedColor
  })

  box({
    width: '100%',
    flexDirection: 'column',
    gap: 0,
    marginBottom: 1,
    children: () => {
      // Label
      text({
        content: `  ${label}:`,
        fg: () => currentField.value === fieldName ? whiteColor : mutedColor,
      })

      // Input container
      box({
        width: '100%',
        paddingLeft: 2,
        paddingRight: 2,
        children: () => {
          input({
            value,
            placeholder,
            password,
            width: 45,
            border: 1,
            borderColor,
            padding: 0,
            paddingLeft: 1,
            fg: whiteColor,
            autoFocus: fieldName === 'username',
            onFocus: () => { currentField.value = fieldName },
            onBlur: () => {
              if (currentField.value === fieldName) {
                currentField.value = null
              }
            },
            onChange: (newValue) => {
              // Value updates automatically via signal binding
              // This is for validation feedback
            },
            onSubmit: () => {
              if (fieldName === 'password') {
                handleSubmit()
              } else {
                focusNextField()
              }
            },
            cursor: {
              style: 'bar',
              blink: { fps: 2 },
            },
          })
        },
      })

      // Validation message
      validationMessage(validation)()
    },
  })
}

// =============================================================================
// MOUNT APP
// =============================================================================

const { unmount, setMode, getMode } = mount(() => {
  // Global key handler for tab navigation and escape
  on((event: KeyEvent) => {
    // Tab / Shift+Tab navigation
    if (event.keycode === 9) { // Tab
      if ((event.modifiers & 1) !== 0) { // Shift
        focusPrevField()
      } else {
        focusNextField()
      }
      return true
    }

    // Escape to cancel
    if (event.keycode === 27) {
      console.log('Form cancelled')
      unmount()
      process.exit(0)
    }

    return false
  })

  // Main form container
  box({
    width: 55,
    flexDirection: 'column',
    border: 1,
    borderColor: mutedColor,
    padding: 0,
    children: () => {
      // Header
      box({
        width: '100%',
        justifyContent: 'center',
        paddingTop: 1,
        paddingBottom: 1,
        borderBottom: 1,
        borderColor: mutedColor,
        children: () => {
          text({
            content: 'Create Account',
            fg: whiteColor,
          })
        },
      })

      // Form body
      box({
        width: '100%',
        flexDirection: 'column',
        padding: 1,
        paddingTop: 2,
        children: () => {
          // Username field
          formField({
            label: 'Username',
            value: username,
            placeholder: 'Enter username...',
            validation: usernameValid,
            fieldName: 'username',
          })

          // Email field
          formField({
            label: 'Email',
            value: email,
            placeholder: 'you@example.com',
            validation: emailValid,
            fieldName: 'email',
          })

          // Password field
          formField({
            label: 'Password',
            value: password,
            placeholder: 'Enter password...',
            validation: passwordValid,
            fieldName: 'password',
            password: true,
          })

          // Spacer
          box({ height: 1 })

          // Submit button
          box({
            width: '100%',
            justifyContent: 'center',
            children: () => {
              // Conditional: Show submit or success
              show(
                () => !submitted.value,
                () => {
                  box({
                    width: 20,
                    height: 3,
                    justifyContent: 'center',
                    alignItems: 'center',
                    border: 1,
                    borderColor: () => {
                      if (currentField.value === 'submit') return primaryColor
                      if (formValid.value) return successColor
                      return mutedColor
                    },
                    bg: () => {
                      if (submitting.value) return mutedColor
                      if (currentField.value === 'submit' && formValid.value) return primaryColor
                      return null
                    },
                    focusable: true,
                    onFocus: () => { currentField.value = 'submit' },
                    onBlur: () => {
                      if (currentField.value === 'submit') {
                        currentField.value = null
                      }
                    },
                    onKey: (event) => {
                      if (event.keycode === 13 && formValid.value) { // Enter
                        handleSubmit()
                        return true
                      }
                      return false
                    },
                    onClick: () => {
                      if (formValid.value) handleSubmit()
                    },
                    children: () => {
                      show(
                        () => submitting.value,
                        () => text({
                          content: 'Creating...',
                          fg: whiteColor,
                        }),
                        () => text({
                          content: 'Create Account',
                          fg: () => formValid.value ? whiteColor : mutedColor,
                        })
                      )
                    },
                  })
                  return () => {}
                },
                () => {
                  // Success message
                  box({
                    width: '100%',
                    justifyContent: 'center',
                    padding: 1,
                    children: () => {
                      text({
                        content: '✓ Account created successfully!',
                        fg: successColor,
                      })
                    },
                  })
                  return () => {}
                }
              )
            },
          })

          // Spacer
          box({ height: 1 })

          // Help text
          box({
            width: '100%',
            justifyContent: 'center',
            children: () => {
              text({
                content: '[Tab] Next  [Shift+Tab] Back  [Enter] Submit  [Esc] Cancel',
                fg: mutedColor,
              })
            },
          })
        },
      })
    },
  })
}, {
  mode: 'fullscreen',
  noopNotifier: true, // Testing without Rust for now
})

console.log('=== SparkTUI Interactive Form Demo ===\n')
console.log('Form UI created successfully!')
console.log('This demo shows the component structure.')
console.log('In production, Rust would render this to the terminal.\n')

// Log reactive state for debugging
effect(() => {
  console.log(`[State] Username: "${username.value}" | Email: "${email.value}" | Password: ${'*'.repeat(password.value.length)}`)
})

effect(() => {
  const u = usernameValid.value
  const e = emailValid.value
  const p = passwordValid.value
  console.log(`[Validation] Username: ${u?.valid ?? 'N/A'} | Email: ${e?.valid ?? 'N/A'} | Password: ${p?.valid ?? 'N/A'} | Form: ${formValid.value}`)
})

effect(() => {
  console.log(`[Focus] Current field: ${currentField.value ?? 'none'}`)
})

// Simulate some user input for demo
console.log('\n--- Simulating user input ---\n')

// Simulate typing
setTimeout(() => { username.value = 'john' }, 500)
setTimeout(() => { username.value = 'john_doe' }, 1000)
setTimeout(() => { email.value = 'john' }, 1500)
setTimeout(() => { email.value = 'john@example.com' }, 2000)
setTimeout(() => { password.value = 'pass' }, 2500)
setTimeout(() => { password.value = 'Password123' }, 3000)

// Simulate focus changes
setTimeout(() => { currentField.value = 'username' }, 100)
setTimeout(() => { currentField.value = 'email' }, 1500)
setTimeout(() => { currentField.value = 'password' }, 2500)
setTimeout(() => { currentField.value = 'submit' }, 3500)

// Simulate form submission
setTimeout(() => {
  console.log('\n--- Simulating form submission ---\n')
  handleSubmit()
}, 4000)
