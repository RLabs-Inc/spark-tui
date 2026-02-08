#!/usr/bin/env bun
/**
 * create-spark-tui
 *
 * Scaffold a new SparkTUI project with a single command.
 *
 * Usage:
 *   bunx create-spark-tui my-app
 *   bunx create-spark-tui my-app --template minimal
 */

import { existsSync, mkdirSync, writeFileSync, cpSync } from 'fs'
import { join, resolve, basename, dirname } from 'path'
import { fileURLToPath } from 'url'

// =============================================================================
// ANSI COLORS
// =============================================================================

const RESET = '\x1b[0m'
const BOLD = '\x1b[1m'
const DIM = '\x1b[2m'
const CYAN = '\x1b[36m'
const GREEN = '\x1b[32m'
const YELLOW = '\x1b[33m'
const RED = '\x1b[31m'
const MAGENTA = '\x1b[35m'

// =============================================================================
// CLI HELPERS
// =============================================================================

function log(msg: string) {
  console.log(msg)
}

function success(msg: string) {
  console.log(`${GREEN}✓${RESET} ${msg}`)
}

function error(msg: string) {
  console.error(`${RED}✗${RESET} ${msg}`)
}

function info(msg: string) {
  console.log(`${CYAN}→${RESET} ${msg}`)
}

function printBanner() {
  log('')
  log(`${BOLD}${MAGENTA}  ⚡ SparkTUI${RESET}`)
  log(`${DIM}     Hybrid TUI Framework${RESET}`)
  log('')
}

function printUsage() {
  log(`${BOLD}Usage:${RESET}`)
  log(`  bunx create-spark-tui ${CYAN}<project-name>${RESET}`)
  log('')
  log(`${BOLD}Options:${RESET}`)
  log(`  --template <name>  Template to use (default: "default")`)
  log(`  --help, -h         Show this help message`)
  log('')
  log(`${BOLD}Examples:${RESET}`)
  log(`  bunx create-spark-tui my-app`)
  log(`  bunx create-spark-tui my-dashboard --template minimal`)
  log('')
}

// =============================================================================
// TEMPLATE FILES
// =============================================================================

function getPackageJson(name: string): string {
  return JSON.stringify({
    name,
    version: '0.1.0',
    type: 'module',
    scripts: {
      dev: 'bun run src/main.ts',
      build: 'bun build src/main.ts --outdir dist --target bun',
      typecheck: 'tsc --noEmit',
    },
    dependencies: {
      '@spark-tui/core': '^0.1.0',
    },
    devDependencies: {
      typescript: '^5.7.0',
      '@types/bun': 'latest',
    },
  }, null, 2)
}

function getTsConfig(): string {
  return JSON.stringify({
    compilerOptions: {
      target: 'ESNext',
      module: 'ESNext',
      moduleResolution: 'bundler',
      strict: true,
      skipLibCheck: true,
      noEmit: true,
      esModuleInterop: true,
      allowSyntheticDefaultImports: true,
      resolveJsonModule: true,
      types: ['bun'],
    },
    include: ['src/**/*'],
  }, null, 2)
}

function getMainTs(): string {
  return `/**
 * SparkTUI Application
 *
 * Run: bun run src/main.ts
 */

import { mount } from '@spark-tui/core'
import { Counter } from './components/counter'

// App
await mount(() => {
  Counter()
}, { mode: 'inline', disableMouse: true })
`
}

function getCounterTs(): string {
  return `/**
 * SparkTUI Counter example
 *
 * A minimal counter demonstrating:
 * - Reactive state with signals
 * - Flexbox layout
 * - Theme-aware styling
 * - Keyboard shortcuts (+/- keys)
 * - Mouse click handling
 * - Theme cycling
 *
 * Controls:
 *   +/=    Increment
 *   -/_    Decrement
 *   r      Reset to zero
 *   t      Cycle theme
 *   q      Quit
 *   Ctrl+C Quit
 */

import {
    signal,
    derived,
    box,
    text,
    t,
    themes,
    setTheme,
    getThemeNames,
    isEnter,
    isSpace,
    getChar,
    bold,
    underline,
} from '@spark-tui/core'

// =============================================================================
// STATE
// =============================================================================

const count = signal(0)
const themeNames = getThemeNames()
const themeIndex = signal(0)
const currentThemeName = derived(() => themeNames[themeIndex.value])

// =============================================================================
// HELPERS
// =============================================================================

function cycleTheme() {
    themeIndex.value = (themeIndex.value + 1) % themeNames.length
    setTheme(themeNames[themeIndex.value] as keyof typeof themes)
}

// =============================================================================
// Component
// =============================================================================

export function Counter() {
    // Root container - centered
    box({
        width: '100%',
        height: '100%',
        flexDirection: 'column',
        justifyContent: 'center',
        alignItems: 'center',
        children: () => {
            // Card
            box({
                flexDirection: 'column',
                alignItems: 'center',
                border: 3, // rounded
                borderColor: t.primary,
                paddingTop: 2,
                paddingBottom: 0,
                paddingLeft: 2,
                paddingRight: 2,
                gap: 2,
                children: () => {
                    // Title
                    text({ content: '⚡ SparkTUI Counter', fg: t.primary, bold })

                    // Counter display row: [ - ] count [ + ]
                    box({
                        width: '100%',
                        flexDirection: 'row',
                        justifyContent: 'space-between',
                        alignItems: 'center',
                        gap: 2,
                        children: () => {
                            // Decrement button
                            box({
                                width: 9,
                                justifyContent: 'center',
                                alignItems: 'center',
                                border: 1,
                                borderColor: t.error,
                                focusable: true,
                                onClick: () => { count.value-- },
                                onKey: (key) => {
                                    if (isEnter(key) || isSpace(key)) {
                                        count.value--
                                        return true
                                    }
                                },
                                children: () => {
                                    text({ content: '  -  ', fg: t.error })
                                },
                            })

                            // Count display
                            box({
                                minWidth: 10,
                                justifyContent: 'center',
                                alignItems: 'center',
                                border: 1,
                                borderColor: t.textMuted,
                                children: () => {
                                    text({
                                        content: derived(() => String(count.value).padStart(4)),
                                        fg: t.textBright,
                                    })
                                },
                            })

                            // Increment button
                            box({
                                width: 9,
                                justifyContent: 'center',
                                alignItems: 'center',
                                border: 1,
                                borderColor: t.success,
                                focusable: true,
                                onClick: () => { count.value++ },
                                onKey: (key) => {
                                    if (isEnter(key) || isSpace(key)) {
                                        count.value++
                                        return true
                                    }
                                },
                                children: () => {
                                    text({ content: '  +  ', fg: t.success })
                                },
                            })
                        },
                    })

                    // Help text
                    box({
                        flexDirection: 'column',
                        alignItems: 'center',
                        children: () => {
                            text({
                                content: '+/- count | r reset | q quit',
                                fg: t.textMuted,
                            })
                            box({
                                flexDirection: 'row',
                                gap: 1,
                                children: () => {
                                    text({ content: 't theme | Theme:', fg: t.textMuted })
                                    text({ content: derived(() => currentThemeName.value), fg: t.textBright, underline })
                                }
                            })
                        },
                    })
                }
            })
        },
        // Global keyboard handler
        onKey: (key) => {
            const ch = getChar(key)

            // Increment
            if (ch === '+' || ch === '=') {
                count.value++
                return true
            }

            // Decrement
            if (ch === '-' || ch === '_') {
                count.value--
                return true
            }

            // Reset
            if (ch === 'r' || ch === 'R') {
                count.value = 0
                return true
            }

            // Theme cycle
            if (ch === 't' || ch === 'T') {
                cycleTheme()
                return true
            }

            // Quit
            if (ch === 'q' || ch === 'Q') {
                process.exit(0)
            }
        },
    })
}
`
}

function getGitIgnore(): string {
  return `# Dependencies
node_modules/

# Build output
dist/

# Bun
bun.lockb

# OS
.DS_Store
Thumbs.db

# IDE
.vscode/
.idea/
*.swp
*.swo

# Logs
*.log
`
}

function getReadme(name: string): string {
  return `# ${name}

A terminal UI application built with [SparkTUI](https://github.com/rlabs-inc/spark-tui).

## Getting Started

\`\`\`bash
# Install dependencies
bun install

# Run the app
bun dev
\`\`\`

## Project Structure

\`\`\`
${name}/
├── src/
│   ├── main.ts              # Application entry point
│   └── components/
│       └── counter.ts       # Counter component
├── package.json
├── tsconfig.json
└── README.md
\`\`\`

## Learn More

- [SparkTUI Documentation](https://spark-tui.dev)
- [API Reference](https://spark-tui.dev/api)
- [Examples](https://github.com/rlabs-inc/spark-tui/tree/main/examples)
`
}

// =============================================================================
// MAIN
// =============================================================================

async function main() {
  const args = process.argv.slice(2)

  // Handle help
  if (args.includes('--help') || args.includes('-h') || args.length === 0) {
    printBanner()
    printUsage()
    process.exit(args.length === 0 ? 1 : 0)
  }

  // Parse arguments
  let projectName = ''
  let template = 'default'

  for (let i = 0; i < args.length; i++) {
    const arg = args[i]
    if (arg === '--template' && args[i + 1]) {
      template = args[++i]
    } else if (!arg.startsWith('-')) {
      projectName = arg
    }
  }

  if (!projectName) {
    printBanner()
    error('Please specify a project name')
    log('')
    printUsage()
    process.exit(1)
  }

  printBanner()

  // Validate project name
  const targetDir = resolve(process.cwd(), projectName)
  const projectBaseName = basename(projectName)

  if (existsSync(targetDir)) {
    error(`Directory "${projectName}" already exists`)
    process.exit(1)
  }

  info(`Creating project in ${CYAN}${targetDir}${RESET}`)
  log('')

  // Create directory structure
  mkdirSync(join(targetDir, 'src', 'components'), { recursive: true })

  // Write files
  writeFileSync(join(targetDir, 'package.json'), getPackageJson(projectBaseName))
  success('Created package.json')

  writeFileSync(join(targetDir, 'tsconfig.json'), getTsConfig())
  success('Created tsconfig.json')

  writeFileSync(join(targetDir, 'src', 'main.ts'), getMainTs())
  success('Created src/main.ts')

  writeFileSync(join(targetDir, 'src', 'components', 'counter.ts'), getCounterTs())
  success('Created src/components/counter.ts')

  writeFileSync(join(targetDir, '.gitignore'), getGitIgnore())
  success('Created .gitignore')

  writeFileSync(join(targetDir, 'README.md'), getReadme(projectBaseName))
  success('Created README.md')

  log('')
  log(`${GREEN}${BOLD}Done!${RESET} Your SparkTUI project is ready.`)
  log('')
  log(`${BOLD}Next steps:${RESET}`)
  log(`  ${DIM}$${RESET} cd ${projectName}`)
  log(`  ${DIM}$${RESET} bun install`)
  log(`  ${DIM}$${RESET} bun dev`)
  log('')
}

main().catch((err) => {
  error(String(err))
  process.exit(1)
})
