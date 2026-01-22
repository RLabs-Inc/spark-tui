# Phase 2: Theme System - Context

**Gathered:** 2026-01-22
**Status:** Ready for planning

<domain>
## Phase Boundary

Enable visual customization with reactive theme colors. Includes:
- Rgba color type with sentinels
- Theme struct with signals
- All TypeScript presets (13 total)
- t.* accessor pattern
- Color inheritance
- Color manipulation (lighten, darken, saturate, contrast)

</domain>

<decisions>
## Implementation Decisions

### Color API design
- Unified Color enum: `Color::Ansi(u8)`, `Color::Rgb(r,g,b)`, `Color::Oklch(l,c,h)`
- Accept hex strings, tuples, AND OKLCH — maximum flexibility, TypeScript-like ergonomics
- Both associated functions AND standalone re-exports: `Color::hex("#ff5500")` and `hex("#ff5500")`

### t.* accessor pattern
- Match TypeScript's exact accessor pattern
- Fully reactive — theme change propagates to all components automatically
- Match TypeScript's variant access pattern (primary, secondary, success, etc.)
- Built-in contrast calculation using OKLCH — adjust lightness within same hue until sufficient contrast
- Opt-in contrast: developer calls `t.contrast(bg)` explicitly
- Chainable color modifiers: `t.primary.alpha(0.5)`, `t.primary.lighten(0.2)`
- Full color manipulation suite: lighten(), darken(), saturate(), desaturate(), mix(), alpha()
- All modifiers are fully reactive — modified colors update when theme changes

### Preset structure
- Match TypeScript preset structure for color definitions
- Default theme is terminal preset using ANSI colors (respects terminal's color scheme)
- Extendable themes: `Theme::new().extend(dracula).with_primary(custom_color)`
- All 13 TypeScript presets + documentation for custom themes
- Separate files: `src/theme/presets/dracula.rs`, etc.
- Just colors, no component-specific tokens — components use variants
- Separate presets for light/dark (no built-in mode toggle)
- Both enum AND string lookup: `Theme::Dracula` for code, `Theme::get("dracula")` for config

### Color inheritance
- Automatic fallback — no color set means use parent's color
- Match TypeScript for which properties inherit
- No special "reset" value — setting any explicit color breaks inheritance

### Claude's Discretion
- Sentinel representation for inherit/transparent (enum variants vs Option wrapper)
- Lazy vs eager preset color initialization
- Inheritance resolution timing (render time vs props time)

</decisions>

<specifics>
## Specific Ideas

- "the theme colors are fully reactive, once set to the components through variants or fg/bg color definitions and a different theme is set, all colors changes reactively without any other intervention"
- "t.contrast(color) must convert the color to oklch if not yet, and in the same hue adjust the color till it contrasts to the defined color enough"
- Terminal theme as default respects terminal's ANSI color definitions — apps can look native in any terminal

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 02-theme-system*
*Context gathered: 2026-01-22*
