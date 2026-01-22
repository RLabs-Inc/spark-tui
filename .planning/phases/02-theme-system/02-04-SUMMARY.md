# Phase 02 Plan 04: Color Modifiers and Contrast Summary

## One-liner

Color manipulation functions (lighten/darken/alpha/mix) using OKLCH with t.contrast() method and ModifiableColor chaining.

---

## Deliverables

| Artifact | Status | Lines | Notes |
|----------|--------|-------|-------|
| src/theme/modifiers.rs | Complete | 481 | All modifier functions with tests |
| src/theme/accessor.rs updates | Complete | +90 | contrast(), contrast_with(), ModifiableColor |
| src/theme/reactive.rs updates | Complete | +73 | ResolvedTheme, resolved_theme() |
| src/types.rs updates | Complete | +20 | Improved ensure_contrast algorithm |

---

## Changes Made

### Task 1: Color Modifier Functions

**File: src/theme/modifiers.rs** (already created in 02-03, verified complete)

Implemented color manipulation functions:
- `lighten(color, amount)` - Increase OKLCH lightness
- `darken(color, amount)` - Decrease OKLCH lightness
- `saturate(color, amount)` - Increase OKLCH chroma
- `desaturate(color, amount)` - Decrease OKLCH chroma
- `alpha(color, value)` - Set alpha (0.0-1.0 as fraction, >1.0 as absolute)
- `fade(color, factor)` - Multiply alpha by factor
- `mix(a, b, weight)` - Linear interpolation between colors
- `contrast(fg, bg, min_ratio)` - Ensure minimum contrast ratio
- `contrast_aa(fg, bg)` - WCAG AA (4.5:1) convenience
- `contrast_aaa(fg, bg)` - WCAG AAA (7.0:1) convenience

All functions preserve ANSI/terminal default colors unchanged.

### Task 2: ThemeAccessor Contrast Methods

**File: src/theme/accessor.rs**

Added methods to ThemeAccessor:
```rust
impl ThemeAccessor {
    /// Get contrasting color using theme text color as base
    pub fn contrast(&self, bg: Rgba) -> Rgba

    /// Get specific color adjusted for contrast
    pub fn contrast_with(&self, fg: Rgba, bg: Rgba) -> Rgba
}
```

Added ModifiableColor for chaining:
```rust
pub struct ModifiableColor(pub Rgba);

impl ModifiableColor {
    pub fn lighten(self, amount: f32) -> Self
    pub fn darken(self, amount: f32) -> Self
    pub fn alpha(self, value: f32) -> Self
    pub fn fade(self, factor: f32) -> Self
    pub fn saturate(self, amount: f32) -> Self
    pub fn desaturate(self, amount: f32) -> Self
    pub fn contrast(self, bg: Rgba) -> Self
    pub fn mix(self, other: Rgba, weight: f32) -> Self
    pub fn get(self) -> Rgba
}
```

### Task 3: Tests

Added 9 new tests in accessor.rs:
- `test_t_accessor_contrast` - contrast() returns proper ratio
- `test_t_accessor_contrast_with` - contrast_with() works
- `test_modifiable_color_lighten` - lighten produces brighter
- `test_modifiable_color_darken` - darken produces darker
- `test_modifiable_color_alpha` - alpha sets correctly
- `test_modifiable_color_chaining` - multiple modifiers chain
- `test_modifiable_color_into_rgba` - From<ModifiableColor> works
- `test_modifiable_color_contrast` - contrast ensures 4.5:1
- `test_modifiable_color_mix` - mix produces midpoint

### Bug Fix: Contrast Algorithm

**File: src/types.rs**

Fixed `ensure_contrast()` to handle edge cases:
- **Problem**: White text on medium-light backgrounds (like Dracula purple) couldn't achieve 4.5:1 contrast because the algorithm only tried to make white lighter (impossible).
- **Solution**: Now tries preferred direction first (based on OKLCH lightness), then falls back to opposite direction if needed.

Also changed direction detection from relative luminance to OKLCH lightness for better consistency with the OKLCH-based adjustments.

---

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed contrast algorithm for edge cases**
- **Found during:** Task 2 testing
- **Issue:** White on Dracula purple (0.38 luminance) failed to achieve 4.5:1 contrast
- **Fix:** Added fallback to try opposite direction when preferred fails
- **Files modified:** src/types.rs
- **Commit:** 9367902

---

## Verification Results

```
cargo check -p spark-tui     # Compiles with warnings only
cargo test -p spark-tui --lib -- --test-threads=1
# test result: ok. 268 passed; 0 failed
```

All modifier functions tested:
- lighten/darken with RGB colors
- ANSI colors return unchanged
- alpha fraction and absolute
- fade multiplication
- saturate/desaturate chroma
- mix blending
- contrast ensures minimum ratio

---

## Test Count

| Category | Count |
|----------|-------|
| Previous tests | 259 |
| New accessor tests | 9 |
| **Total** | **268** |

---

## Key Files

| File | Purpose |
|------|---------|
| src/theme/modifiers.rs | Color modification functions |
| src/theme/accessor.rs | ThemeAccessor, ModifiableColor |
| src/theme/reactive.rs | ResolvedTheme, resolved_theme() |
| src/theme/mod.rs | Module exports |
| src/types.rs | Rgba::ensure_contrast() |

---

## Commits

| Hash | Message |
|------|---------|
| 4156054 | feat(02-04): add color modifier functions |
| 691b41e | feat(02-04): add ResolvedTheme struct and resolved_theme() function |

Note: Some commits were prefixed with 02-03 due to execution overlap, but the work is complete.

---

## API Usage Examples

```rust
use spark_tui::theme::{t, ModifiableColor};
use spark_tui::types::Rgba;
use spark_tui::theme::modifiers::{lighten, darken, alpha, contrast_aa};

// Direct modifier use
let lighter = lighten(Rgba::rgb(100, 100, 100), 0.2);
let semi_transparent = alpha(Rgba::RED, 0.5);
let readable = contrast_aa(Rgba::rgb(60, 60, 60), Rgba::BLACK);

// ThemeAccessor contrast methods
let theme = t();
let text_color = theme.contrast(dark_background);
let adjusted = theme.contrast_with(theme.primary(), custom_bg);

// ModifiableColor chaining
let result = ModifiableColor::from(theme.primary())
    .lighten(0.1)
    .alpha(0.8)
    .contrast(surface_color)
    .get();
```

---

## Next Phase Readiness

The variant system (02-03) uses these modifiers internally. Both are complete:
- Modifiers work independently on any Rgba
- ThemeAccessor provides convenient theme-aware contrast
- ModifiableColor enables fluent API
- Variant system uses ResolvedTheme for styling

Phase 2 is complete! Ready for Phase 3 (Input Component).

---

*Completed: 2026-01-22*
