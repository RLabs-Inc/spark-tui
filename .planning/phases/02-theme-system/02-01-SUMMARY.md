---
phase: 02-theme-system
plan: 01
subsystem: theme
tags: [theme, colors, presets, oklch, ansi]
requires: [01]
provides: [theme-types, theme-presets, color-parsing]
affects: [02-02, 02-03, 02-04]
tech-stack:
  added: []
  patterns: [enum-variants, from-impls, lazy-string-parsing]
key-files:
  created:
    - src/theme/mod.rs
    - src/theme/presets.rs
  modified:
    - src/types.rs
    - src/lib.rs
decisions:
  - id: theme-color-enum
    choice: ThemeColor enum with Default/Ansi/Rgb/Str variants
    rationale: Matches TypeScript semantics exactly - terminal default, ANSI palette, RGB values, and lazy-parsed strings
  - id: preset-functions
    choice: Functions returning Theme instead of const values
    rationale: Theme struct has String fields which require allocation; functions allow idiomatic Rust without lazy_static
  - id: case-insensitive-lookup
    choice: get_preset() normalizes to lowercase and strips underscores
    rationale: User-friendly API that accepts tokyoNight, tokyo_night, or TOKYONIGHT
metrics:
  duration: 6m15s
  completed: 2026-01-22
---

# Phase 02 Plan 01: Theme Foundation Summary

**One-liner:** Theme types with ThemeColor enum, 20-slot Theme struct, and all 13 TypeScript presets (terminal, dracula, nord, etc.)

## What Was Built

### ThemeColor Enum

A flexible color type matching TypeScript's `ThemeColor = null | number | string`:

```rust
pub enum ThemeColor {
    Default,        // Terminal default (null in TS)
    Ansi(u8),       // ANSI palette 0-255 (number 0-255 in TS)
    Rgb(Rgba),      // Explicit RGB (number > 255 in TS)
    Str(String),    // Parse on use (string in TS)
}
```

With ergonomic `From` implementations:
- `()` -> Default
- `u8` -> Ansi
- `Rgba` -> Rgb
- `&str` / `String` -> Str
- `u32` -> Rgb (0xRRGGBB)
- `Option<u32>` -> Default or Rgb

### Theme Struct

20 semantic color slots organized by purpose:

| Category | Fields |
|----------|--------|
| Main Palette | primary, secondary, tertiary, accent |
| Semantic | success, warning, error, info |
| Text | text, text_muted, text_dim, text_disabled, text_bright |
| Background | background, background_muted, surface, overlay |
| Border | border, border_focus |

### 13 Preset Themes

All TypeScript presets ported with exact color values:

1. **terminal** - ANSI colors (default)
2. **dracula** - Dark with vivid OKLCH colors
3. **nord** - Arctic bluish
4. **monokai** - Vibrant syntax-highlighting inspired
5. **solarized** - Precision color scheme
6. **catppuccin** - Soothing pastel (Mocha variant)
7. **gruvbox** - Retro groove
8. **tokyoNight** - Tokyo city lights
9. **oneDark** - Atom's iconic theme
10. **rosePine** - Natural pine vibes
11. **kanagawa** - Hokusai wave inspired
12. **everforest** - Green-tinted comfort
13. **nightOwl** - Accessibility focused

### Color Parsing (Already Existed)

Confirmed Rgba parsing methods work correctly:
- `Rgba::from_hex()` - #RGB, #RRGGBB, #RRGGBBAA
- `Rgba::from_oklch_str()` - oklch(L C H) with units
- `Rgba::parse()` - Any format detection
- `Rgba::from_rgb_int()` - 0xRRGGBB integers

## Commits

| Hash | Type | Description |
|------|------|-------------|
| 643ea4e | test | Add 24 Rgba color parsing tests |
| 528cd88 | feat | Theme module with ThemeColor, Theme, 13 presets |
| 725f255 | fix | OKLCH doc test alpha rounding tolerance |

## Test Coverage

- **24 new Rgba parsing tests** - hex, oklch, edge cases
- **16 ThemeColor tests** - enum variants, From impls, resolve()
- **17 preset tests** - all presets exist, colors match TypeScript

**Total tests:** 215 unit + 9 doc = 224 passing

## Deviations from Plan

None - plan executed exactly as written. Color parsing methods already existed in types.rs, so Task 1 only needed tests.

## API Usage

```rust
use spark_tui::theme::{Theme, ThemeColor, get_preset, dracula};

// Get preset by name (case-insensitive)
let theme = get_preset("dracula").unwrap();

// Or use the function directly
let theme = dracula();

// Access colors
let primary_color = theme.primary.resolve(); // -> Rgba

// Create custom theme color
let custom: ThemeColor = "#bd93f9".into();
let ansi: ThemeColor = 12u8.into();
let rgb: ThemeColor = 0xff5500u32.into();
```

## Next Phase Readiness

Ready for Plan 02 (Reactive Theme State):
- ThemeColor and Theme types are defined
- Presets are accessible via `get_preset()` and direct functions
- Color resolution via `ThemeColor::resolve()` works correctly

Dependencies satisfied:
- `Rgba` type with full color parsing
- All 13 presets with exact TypeScript color values
- Clean module structure for adding reactivity

## Files Changed

| File | Lines | Change |
|------|-------|--------|
| src/theme/mod.rs | 276 | Created - ThemeColor, Theme, tests |
| src/theme/presets.rs | 627 | Created - 13 presets, lookup functions |
| src/types.rs | +605 | Added comprehensive Rgba parsing tests |
| src/lib.rs | +6 | Added theme module and re-exports |

**Total:** 1,514 lines added
