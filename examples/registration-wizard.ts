/**
 * SparkTUI - Registration Wizard Example
 *
 * A multi-step registration form demonstrating:
 * - Step 1: Personal info (name, email)
 * - Step 2: Account (username, password, confirm password)
 * - Step 3: Preferences (theme selection, notifications toggle)
 * - Step indicator showing progress
 * - Next/Back/Submit buttons
 * - Validation per step
 *
 * Run: bun run examples/registration-wizard.ts
 */

import { signal, derived } from '@rlabs-inc/signals'
import { mount } from '../ts/engine/mount'
import { box, text, input, show, each } from '../ts/primitives'
import { on, isPress } from '../ts/state/keyboard'
import type { KeyEvent } from '../ts/state/keyboard'
import { KEY_TAB, KEY_ENTER, KEY_ESCAPE, hasShift } from '../ts/engine/events'
import { packColor } from '../ts/bridge/shared-buffer'

// =============================================================================
// COLORS
// =============================================================================

const colors = {
  bg: packColor(15, 15, 22, 255),
  surface: packColor(25, 25, 35, 255),
  surfaceHover: packColor(35, 35, 50, 255),
  border: packColor(55, 55, 75, 255),
  borderFocus: packColor(130, 100, 255, 255),
  text: packColor(220, 220, 235, 255),
  textMuted: packColor(110, 110, 140, 255),
  textBright: packColor(255, 255, 255, 255),
  primary: packColor(130, 100, 255, 255),
  primaryDim: packColor(80, 60, 160, 255),
  success: packColor(80, 200, 120, 255),
  error: packColor(255, 90, 90, 255),
  warning: packColor(255, 180, 70, 255),
  stepActive: packColor(130, 100, 255, 255),
  stepComplete: packColor(80, 200, 120, 255),
  stepPending: packColor(55, 55, 75, 255),
}

// =============================================================================
// FORM STATE
// =============================================================================

// Step 1: Personal Info
const fullName = signal('')
const email = signal('')

// Step 2: Account
const username = signal('')
const password = signal('')
const confirmPassword = signal('')

// Step 3: Preferences
const selectedTheme = signal<'light' | 'dark' | 'system'>('system')
const emailNotifications = signal(true)
const pushNotifications = signal(false)

// Wizard state
const currentStep = signal(1)
const isSubmitting = signal(false)
const isComplete = signal(false)

// Focus tracking per step
type Step1Field = 'fullName' | 'email' | 'next'
type Step2Field = 'username' | 'password' | 'confirmPassword' | 'back' | 'next'
type Step3Field = 'theme-light' | 'theme-dark' | 'theme-system' | 'emailNotif' | 'pushNotif' | 'back' | 'submit'

const step1Fields: Step1Field[] = ['fullName', 'email', 'next']
const step2Fields: Step2Field[] = ['username', 'password', 'confirmPassword', 'back', 'next']
const step3Fields: Step3Field[] = ['theme-light', 'theme-dark', 'theme-system', 'emailNotif', 'pushNotif', 'back', 'submit']

const focusedStep1 = signal<Step1Field>('fullName')
const focusedStep2 = signal<Step2Field>('username')
const focusedStep3 = signal<Step3Field>('theme-light')

// =============================================================================
// VALIDATION
// =============================================================================

// Step 1 validation
const step1Valid = derived(() => {
  const name = fullName.value.trim()
  const mail = email.value.trim()
  return name.length >= 2 && mail.includes('@') && mail.includes('.')
})

const nameError = derived(() => {
  const name = fullName.value.trim()
  if (name.length === 0) return null
  if (name.length < 2) return 'Name must be at least 2 characters'
  return null
})

const emailError = derived(() => {
  const mail = email.value.trim()
  if (mail.length === 0) return null
  if (!mail.includes('@') || !mail.includes('.')) return 'Please enter a valid email'
  return null
})

// Step 2 validation
const step2Valid = derived(() => {
  const user = username.value.trim()
  const pass = password.value
  const confirm = confirmPassword.value
  return user.length >= 3 && pass.length >= 8 && pass === confirm
})

const usernameError = derived(() => {
  const user = username.value.trim()
  if (user.length === 0) return null
  if (user.length < 3) return 'Username must be at least 3 characters'
  if (!/^[a-zA-Z0-9_]+$/.test(user)) return 'Only letters, numbers, underscores'
  return null
})

const passwordError = derived(() => {
  const pass = password.value
  if (pass.length === 0) return null
  if (pass.length < 8) return 'Password must be at least 8 characters'
  if (!/[A-Z]/.test(pass)) return 'Needs an uppercase letter'
  if (!/[a-z]/.test(pass)) return 'Needs a lowercase letter'
  if (!/[0-9]/.test(pass)) return 'Needs a number'
  return null
})

const confirmError = derived(() => {
  const pass = password.value
  const confirm = confirmPassword.value
  if (confirm.length === 0) return null
  if (pass !== confirm) return 'Passwords do not match'
  return null
})

// Step 3 is always valid (preferences have defaults)
const step3Valid = derived(() => true)

// =============================================================================
// NAVIGATION
// =============================================================================

function getCurrentFields(): string[] {
  switch (currentStep.value) {
    case 1: return step1Fields
    case 2: return step2Fields
    case 3: return step3Fields
    default: return []
  }
}

function getCurrentFocus(): string {
  switch (currentStep.value) {
    case 1: return focusedStep1.value
    case 2: return focusedStep2.value
    case 3: return focusedStep3.value
    default: return ''
  }
}

function setCurrentFocus(field: string): void {
  switch (currentStep.value) {
    case 1: focusedStep1.value = field as Step1Field; break
    case 2: focusedStep2.value = field as Step2Field; break
    case 3: focusedStep3.value = field as Step3Field; break
  }
}

function focusNext() {
  const fields = getCurrentFields()
  const current = getCurrentFocus()
  const idx = fields.indexOf(current)
  const nextIdx = (idx + 1) % fields.length
  setCurrentFocus(fields[nextIdx]!)
}

function focusPrev() {
  const fields = getCurrentFields()
  const current = getCurrentFocus()
  const idx = fields.indexOf(current)
  const prevIdx = (idx - 1 + fields.length) % fields.length
  setCurrentFocus(fields[prevIdx]!)
}

function goNext() {
  if (currentStep.value === 1 && step1Valid.value) {
    currentStep.value = 2
    focusedStep2.value = 'username'
  } else if (currentStep.value === 2 && step2Valid.value) {
    currentStep.value = 3
    focusedStep3.value = 'theme-light'
  }
}

function goBack() {
  if (currentStep.value === 2) {
    currentStep.value = 1
    focusedStep1.value = 'fullName'
  } else if (currentStep.value === 3) {
    currentStep.value = 2
    focusedStep2.value = 'username'
  }
}

function handleSubmit() {
  if (!step1Valid.value || !step2Valid.value) return

  isSubmitting.value = true

  setTimeout(() => {
    isSubmitting.value = false
    isComplete.value = true

    console.log('\n=== Registration Complete ===')
    console.log(`Name: ${fullName.value}`)
    console.log(`Email: ${email.value}`)
    console.log(`Username: ${username.value}`)
    console.log(`Theme: ${selectedTheme.value}`)
    console.log(`Email Notifications: ${emailNotifications.value}`)
    console.log(`Push Notifications: ${pushNotifications.value}`)
    console.log('=============================\n')

    setTimeout(() => process.exit(0), 2500)
  }, 1500)
}

// =============================================================================
// UI COMPONENTS
// =============================================================================

function StepIndicator() {
  const steps = [
    { num: 1, label: 'Personal' },
    { num: 2, label: 'Account' },
    { num: 3, label: 'Preferences' },
  ]

  box({
    width: '100%',
    flexDirection: 'row',
    justifyContent: 'center',
    alignItems: 'center',
    gap: 1,
    marginBottom: 2,
    children: () => {
      for (let i = 0; i < steps.length; i++) {
        const step = steps[i]!
        const isActive = () => currentStep.value === step.num
        const isCompleted = () => currentStep.value > step.num

        // Step circle
        box({
          width: 3,
          height: 1,
          justifyContent: 'center',
          alignItems: 'center',
          border: 1,
          borderColor: () => {
            if (isCompleted()) return colors.stepComplete
            if (isActive()) return colors.stepActive
            return colors.stepPending
          },
          bg: () => {
            if (isCompleted()) return colors.stepComplete
            if (isActive()) return colors.stepActive
            return colors.surface
          },
          children: () => {
            text({
              content: () => isCompleted() ? 'v' : String(step.num),
              fg: () => isCompleted() || isActive() ? colors.textBright : colors.textMuted,
            })
          },
        })

        // Step label
        text({
          content: step.label,
          fg: () => {
            if (isCompleted()) return colors.stepComplete
            if (isActive()) return colors.textBright
            return colors.textMuted
          },
        })

        // Connector line (except for last)
        if (i < steps.length - 1) {
          box({
            width: 6,
            height: 1,
            justifyContent: 'center',
            children: () => {
              text({
                content: '----',
                fg: () => currentStep.value > step.num ? colors.stepComplete : colors.stepPending,
              })
            },
          })
        }
      }
    },
  })
}

function InputField(config: {
  label: string
  value: typeof fullName
  placeholder: string
  fieldName: string
  getFocus: () => string
  setFocus: (f: string) => void
  password?: boolean
  error: { readonly value: string | null }
  onSubmit?: () => void
}) {
  const { label, value, placeholder, fieldName, getFocus, setFocus, password, error, onSubmit } = config
  const isFocused = () => getFocus() === fieldName

  box({
    width: '100%',
    flexDirection: 'column',
    marginBottom: 1,
    children: () => {
      text({
        content: label,
        fg: () => isFocused() ? colors.textBright : colors.textMuted,
      })

      input({
        id: fieldName,
        value,
        placeholder,
        password,
        width: 44,
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
        onFocus: () => setFocus(fieldName),
        onSubmit: onSubmit ?? focusNext,
        cursor: { style: 'bar', blink: { fps: 2 } },
      })

      show(
        () => error.value !== null,
        () => text({ content: () => error.value ?? '', fg: colors.error })
      )
    },
  })
}

function Toggle(config: {
  label: string
  value: typeof emailNotifications
  fieldName: string
  getFocus: () => string
  setFocus: (f: string) => void
}) {
  const { label, value, fieldName, getFocus, setFocus } = config
  const isFocused = () => getFocus() === fieldName

  box({
    flexDirection: 'row',
    alignItems: 'center',
    gap: 2,
    marginBottom: 1,
    children: () => {
      box({
        width: 6,
        height: 1,
        border: 1,
        borderColor: () => isFocused() ? colors.borderFocus : colors.border,
        bg: () => value.value ? colors.success : colors.surfaceHover,
        justifyContent: 'center',
        focusable: true,
        onFocus: () => setFocus(fieldName),
        onClick: () => { value.value = !value.value },
        onKey: (event: KeyEvent) => {
          if (event.keycode === KEY_ENTER || event.keycode === 32) {
            value.value = !value.value
            return true
          }
          return false
        },
        children: () => {
          text({
            content: () => value.value ? 'ON' : 'OFF',
            fg: colors.textBright,
          })
        },
      })

      text({
        content: label,
        fg: () => isFocused() ? colors.textBright : colors.text,
      })
    },
  })
}

function ThemeOption(config: {
  label: string
  value: 'light' | 'dark' | 'system'
  fieldName: string
  getFocus: () => string
  setFocus: (f: string) => void
}) {
  const { label, value, fieldName, getFocus, setFocus } = config
  const isFocused = () => getFocus() === fieldName
  const isSelected = () => selectedTheme.value === value

  box({
    width: 14,
    height: 3,
    border: 1,
    borderColor: () => {
      if (isFocused()) return colors.borderFocus
      if (isSelected()) return colors.primary
      return colors.border
    },
    bg: () => isSelected() ? colors.primaryDim : colors.surface,
    justifyContent: 'center',
    alignItems: 'center',
    focusable: true,
    onFocus: () => setFocus(fieldName),
    onClick: () => { selectedTheme.value = value },
    onKey: (event: KeyEvent) => {
      if (event.keycode === KEY_ENTER || event.keycode === 32) {
        selectedTheme.value = value
        return true
      }
      return false
    },
    children: () => {
      text({
        content: label,
        fg: () => isSelected() || isFocused() ? colors.textBright : colors.text,
      })
    },
  })
}

function Button(config: {
  label: string
  fieldName: string
  getFocus: () => string
  setFocus: (f: string) => void
  primary?: boolean
  disabled?: () => boolean
  onClick: () => void
}) {
  const { label, fieldName, getFocus, setFocus, primary, disabled, onClick } = config
  const isFocused = () => getFocus() === fieldName
  const isDisabled = disabled ?? (() => false)

  box({
    width: 12,
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
    onFocus: () => setFocus(fieldName),
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
          if (isDisabled()) return colors.textMuted
          if (primary && isFocused()) return colors.textBright
          return colors.text
        },
      })
    },
  })
}

// =============================================================================
// STEP CONTENT
// =============================================================================

function Step1Content() {
  return box({
    flexDirection: 'column',
    children: () => {
      text({ content: 'Personal Information', fg: colors.textBright, marginBottom: 1 })
      text({ content: 'Please enter your name and email address.', fg: colors.textMuted, marginBottom: 2 })

      InputField({
        label: 'Full Name',
        value: fullName,
        placeholder: 'John Doe',
        fieldName: 'fullName',
        getFocus: () => focusedStep1.value,
        setFocus: (f) => { focusedStep1.value = f as Step1Field },
        error: nameError,
      })

      InputField({
        label: 'Email Address',
        value: email,
        placeholder: 'john@example.com',
        fieldName: 'email',
        getFocus: () => focusedStep1.value,
        setFocus: (f) => { focusedStep1.value = f as Step1Field },
        error: emailError,
        onSubmit: () => { if (step1Valid.value) goNext(); else focusNext() },
      })

      box({
        flexDirection: 'row',
        justifyContent: 'flex-end',
        marginTop: 2,
        children: () => {
          Button({
            label: 'Next',
            fieldName: 'next',
            getFocus: () => focusedStep1.value,
            setFocus: (f) => { focusedStep1.value = f as Step1Field },
            primary: true,
            disabled: () => !step1Valid.value,
            onClick: goNext,
          })
        },
      })
    },
  })
}

function Step2Content() {
  return box({
    flexDirection: 'column',
    children: () => {
      text({ content: 'Account Setup', fg: colors.textBright, marginBottom: 1 })
      text({ content: 'Choose a username and password for your account.', fg: colors.textMuted, marginBottom: 2 })

      InputField({
        label: 'Username',
        value: username,
        placeholder: 'johndoe',
        fieldName: 'username',
        getFocus: () => focusedStep2.value,
        setFocus: (f) => { focusedStep2.value = f as Step2Field },
        error: usernameError,
      })

      InputField({
        label: 'Password',
        value: password,
        placeholder: 'Enter password...',
        fieldName: 'password',
        getFocus: () => focusedStep2.value,
        setFocus: (f) => { focusedStep2.value = f as Step2Field },
        password: true,
        error: passwordError,
      })

      InputField({
        label: 'Confirm Password',
        value: confirmPassword,
        placeholder: 'Confirm password...',
        fieldName: 'confirmPassword',
        getFocus: () => focusedStep2.value,
        setFocus: (f) => { focusedStep2.value = f as Step2Field },
        password: true,
        error: confirmError,
        onSubmit: () => { if (step2Valid.value) goNext(); else focusNext() },
      })

      box({
        flexDirection: 'row',
        justifyContent: 'space-between',
        marginTop: 2,
        children: () => {
          Button({
            label: 'Back',
            fieldName: 'back',
            getFocus: () => focusedStep2.value,
            setFocus: (f) => { focusedStep2.value = f as Step2Field },
            onClick: goBack,
          })

          Button({
            label: 'Next',
            fieldName: 'next',
            getFocus: () => focusedStep2.value,
            setFocus: (f) => { focusedStep2.value = f as Step2Field },
            primary: true,
            disabled: () => !step2Valid.value,
            onClick: goNext,
          })
        },
      })
    },
  })
}

function Step3Content() {
  return box({
    flexDirection: 'column',
    children: () => {
      text({ content: 'Preferences', fg: colors.textBright, marginBottom: 1 })
      text({ content: 'Customize your experience.', fg: colors.textMuted, marginBottom: 2 })

      // Theme selection
      text({ content: 'Theme', fg: colors.textMuted, marginBottom: 1 })
      box({
        flexDirection: 'row',
        gap: 2,
        marginBottom: 2,
        children: () => {
          ThemeOption({
            label: 'Light',
            value: 'light',
            fieldName: 'theme-light',
            getFocus: () => focusedStep3.value,
            setFocus: (f) => { focusedStep3.value = f as Step3Field },
          })

          ThemeOption({
            label: 'Dark',
            value: 'dark',
            fieldName: 'theme-dark',
            getFocus: () => focusedStep3.value,
            setFocus: (f) => { focusedStep3.value = f as Step3Field },
          })

          ThemeOption({
            label: 'System',
            value: 'system',
            fieldName: 'theme-system',
            getFocus: () => focusedStep3.value,
            setFocus: (f) => { focusedStep3.value = f as Step3Field },
          })
        },
      })

      // Notification toggles
      text({ content: 'Notifications', fg: colors.textMuted, marginBottom: 1 })

      Toggle({
        label: 'Email notifications',
        value: emailNotifications,
        fieldName: 'emailNotif',
        getFocus: () => focusedStep3.value,
        setFocus: (f) => { focusedStep3.value = f as Step3Field },
      })

      Toggle({
        label: 'Push notifications',
        value: pushNotifications,
        fieldName: 'pushNotif',
        getFocus: () => focusedStep3.value,
        setFocus: (f) => { focusedStep3.value = f as Step3Field },
      })

      box({
        flexDirection: 'row',
        justifyContent: 'space-between',
        marginTop: 2,
        children: () => {
          Button({
            label: 'Back',
            fieldName: 'back',
            getFocus: () => focusedStep3.value,
            setFocus: (f) => { focusedStep3.value = f as Step3Field },
            onClick: goBack,
          })

          Button({
            label: () => isSubmitting.value ? 'Creating...' : 'Create Account',
            fieldName: 'submit',
            getFocus: () => focusedStep3.value,
            setFocus: (f) => { focusedStep3.value = f as Step3Field },
            primary: true,
            disabled: () => isSubmitting.value,
            onClick: handleSubmit,
          })
        },
      })
    },
  })
}

// =============================================================================
// MOUNT APP
// =============================================================================

const cols = process.stdout.columns || 80
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
      console.log('Registration cancelled')
      process.exit(0)
    }

    return false
  })

  // Root container
  box({
    id: 'root',
    width: cols,
    height: rows,
    bg: colors.bg,
    justifyContent: 'center',
    alignItems: 'center',
    children: () => {
      // Wizard card
      box({
        id: 'wizard-card',
        width: 56,
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
              text({ content: 'Create Your Account', fg: colors.textBright })
            },
          })

          // Content area
          box({
            width: '100%',
            flexDirection: 'column',
            padding: 2,
            children: () => {
              show(
                () => isComplete.value,
                () => {
                  return box({
                    width: '100%',
                    height: 15,
                    flexDirection: 'column',
                    justifyContent: 'center',
                    alignItems: 'center',
                    gap: 1,
                    children: () => {
                      text({ content: 'Registration Complete!', fg: colors.success })
                      text({ content: `Welcome, ${fullName.value}!`, fg: colors.text })
                      text({ content: `Your username: ${username.value}`, fg: colors.textMuted })
                    },
                  })
                },
                () => {
                  return box({
                    flexDirection: 'column',
                    children: () => {
                      // Step indicator
                      StepIndicator()

                      // Step content
                      show(() => currentStep.value === 1, Step1Content)
                      show(() => currentStep.value === 2, Step2Content)
                      show(() => currentStep.value === 3, Step3Content)
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
                content: '[Tab] Next Field  [Shift+Tab] Previous  [Enter] Select  [Esc] Cancel',
                fg: colors.textMuted,
              })
            },
          })
        },
      })
    },
  })
}, { mode: 'fullscreen' })

console.log('[registration-wizard] App mounted - Press Ctrl+C to exit')
await new Promise(() => {})
