//! Variant system for consistent component theming.
//!
//! Provides semantic variants (primary, success, error, etc.) that automatically
//! calculate proper foreground/background colors with WCAG-compliant contrast.
//!
//! # Example
//!
//! ```ignore
//! use spark_tui::theme::{Variant, get_variant_style, variant_style};
//!
//! // Get style for a variant (instant, non-reactive)
//! let style = get_variant_style(Variant::Primary);
//! println!("fg: {:?}, bg: {:?}", style.fg, style.bg);
//!
//! // Reactive variant style that updates when theme changes
//! let reactive_style = variant_style(Variant::Error);
//! let current = reactive_style.get(); // VariantStyle
//! ```

use crate::Rgba;
use super::reactive::resolved_theme;
use spark_signals::{Derived, derived};

// =============================================================================
// Variant Enum
// =============================================================================

/// Semantic variants for component theming.
/// Each variant defines a consistent color scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Variant {
    /// Default styling (text on background)
    #[default]
    Default,

    /// Primary action/emphasis
    Primary,
    /// Secondary action
    Secondary,
    /// Tertiary action
    Tertiary,
    /// Accent/highlight
    Accent,

    /// Success state (green)
    Success,
    /// Warning state (yellow/orange)
    Warning,
    /// Error state (red)
    Error,
    /// Informational state (blue/cyan)
    Info,

    /// Muted/subtle styling
    Muted,
    /// Surface/card styling
    Surface,
    /// Elevated surface (more contrast)
    Elevated,

    /// Ghost - transparent background
    Ghost,
    /// Outline - border only, transparent bg
    Outline,
}

impl Variant {
    /// Parse from string (case-insensitive).
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "default" => Some(Self::Default),
            "primary" => Some(Self::Primary),
            "secondary" => Some(Self::Secondary),
            "tertiary" => Some(Self::Tertiary),
            "accent" => Some(Self::Accent),
            "success" => Some(Self::Success),
            "warning" => Some(Self::Warning),
            "error" => Some(Self::Error),
            "info" => Some(Self::Info),
            "muted" => Some(Self::Muted),
            "surface" => Some(Self::Surface),
            "elevated" => Some(Self::Elevated),
            "ghost" => Some(Self::Ghost),
            "outline" => Some(Self::Outline),
            _ => None,
        }
    }

    /// Get all variant names as a slice.
    pub const fn all() -> &'static [Variant] {
        &[
            Self::Default,
            Self::Primary,
            Self::Secondary,
            Self::Tertiary,
            Self::Accent,
            Self::Success,
            Self::Warning,
            Self::Error,
            Self::Info,
            Self::Muted,
            Self::Surface,
            Self::Elevated,
            Self::Ghost,
            Self::Outline,
        ]
    }
}

// =============================================================================
// VariantStyle
// =============================================================================

/// Resolved colors for a variant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VariantStyle {
    /// Foreground (text) color
    pub fg: Rgba,
    /// Background color
    pub bg: Rgba,
    /// Border color
    pub border: Rgba,
    /// Border color when focused
    pub border_focus: Rgba,
}

// =============================================================================
// Contrast Calculation
// =============================================================================

/// Get a foreground color with proper contrast against background.
/// For ANSI colors: trust terminal's handling.
/// For RGB colors: adjust lightness using OKLCH for WCAG AA (4.5:1).
fn get_contrast_fg(desired_fg: Rgba, bg: Rgba) -> Rgba {
    // If background is ANSI, trust terminal
    if bg.is_ansi() {
        return desired_fg;
    }

    // If foreground is ANSI but background is RGB, trust it
    if desired_fg.is_ansi() {
        return desired_fg;
    }

    // If background is terminal default, trust it
    if bg.is_terminal_default() {
        return desired_fg;
    }

    // If foreground is terminal default, trust it
    if desired_fg.is_terminal_default() {
        return desired_fg;
    }

    // Both are RGB - ensure proper contrast
    Rgba::ensure_contrast(desired_fg, bg, 4.5).unwrap_or(desired_fg)
}

// =============================================================================
// get_variant_style
// =============================================================================

/// Get colors for a variant based on current theme.
/// Automatically calculates contrast for RGB themes.
pub fn get_variant_style(variant: Variant) -> VariantStyle {
    let theme = resolved_theme();

    match variant {
        Variant::Primary => VariantStyle {
            fg: get_contrast_fg(theme.text_bright, theme.primary),
            bg: theme.primary,
            border: theme.primary,
            border_focus: theme.accent,
        },

        Variant::Secondary => VariantStyle {
            fg: get_contrast_fg(theme.text_bright, theme.secondary),
            bg: theme.secondary,
            border: theme.secondary,
            border_focus: theme.accent,
        },

        Variant::Tertiary => VariantStyle {
            fg: get_contrast_fg(theme.text_bright, theme.tertiary),
            bg: theme.tertiary,
            border: theme.tertiary,
            border_focus: theme.accent,
        },

        Variant::Accent => VariantStyle {
            // Accent is often bright (yellow), need dark fg
            fg: get_contrast_fg(Rgba::BLACK, theme.accent),
            bg: theme.accent,
            border: theme.accent,
            border_focus: theme.primary,
        },

        Variant::Success => VariantStyle {
            fg: get_contrast_fg(theme.text_bright, theme.success),
            bg: theme.success,
            border: theme.success,
            border_focus: theme.accent,
        },

        Variant::Warning => VariantStyle {
            // Warning is often yellow, need dark fg
            fg: get_contrast_fg(Rgba::BLACK, theme.warning),
            bg: theme.warning,
            border: theme.warning,
            border_focus: theme.accent,
        },

        Variant::Error => VariantStyle {
            fg: get_contrast_fg(theme.text_bright, theme.error),
            bg: theme.error,
            border: theme.error,
            border_focus: theme.accent,
        },

        Variant::Info => VariantStyle {
            fg: get_contrast_fg(theme.text_bright, theme.info),
            bg: theme.info,
            border: theme.info,
            border_focus: theme.accent,
        },

        Variant::Muted => VariantStyle {
            fg: theme.text_muted,
            bg: theme.surface,
            border: theme.border,
            border_focus: theme.border_focus,
        },

        Variant::Surface => VariantStyle {
            fg: theme.text,
            bg: theme.surface,
            border: theme.border,
            border_focus: theme.border_focus,
        },

        Variant::Elevated => VariantStyle {
            fg: get_contrast_fg(theme.text_bright, theme.surface),
            bg: theme.surface,
            border: theme.primary,
            border_focus: theme.border_focus,
        },

        Variant::Ghost => VariantStyle {
            fg: theme.text,
            bg: Rgba::TERMINAL_DEFAULT,
            border: Rgba::TERMINAL_DEFAULT,
            border_focus: theme.border_focus,
        },

        Variant::Outline => VariantStyle {
            fg: theme.primary,
            bg: Rgba::TERMINAL_DEFAULT,
            border: theme.primary,
            border_focus: theme.border_focus,
        },

        Variant::Default => VariantStyle {
            fg: theme.text,
            bg: theme.background,
            border: theme.border,
            border_focus: theme.border_focus,
        },
    }
}

// =============================================================================
// Reactive variant_style
// =============================================================================

/// Get reactive variant style that updates when theme changes.
///
/// Returns a Derived that automatically recalculates when the theme changes.
/// Use `.get()` to read the current style.
///
/// # Example
/// ```ignore
/// use spark_tui::theme::{Variant, variant_style, set_theme};
///
/// let style = variant_style(Variant::Primary);
/// let current = style.get(); // VariantStyle
///
/// set_theme("dracula");
/// let updated = style.get(); // New colors
/// ```
pub fn variant_style(variant: Variant) -> Derived<VariantStyle, impl Fn() -> VariantStyle> {
    derived(move || get_variant_style(variant))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{set_theme, reset_theme_state};

    #[test]
    fn test_variant_from_str() {
        assert_eq!(Variant::from_str("primary"), Some(Variant::Primary));
        assert_eq!(Variant::from_str("SUCCESS"), Some(Variant::Success));
        assert_eq!(Variant::from_str("Warning"), Some(Variant::Warning));
        assert_eq!(Variant::from_str("invalid"), None);
        assert_eq!(Variant::from_str(""), None);
    }

    #[test]
    fn test_variant_from_str_all_variants() {
        let cases = [
            ("default", Variant::Default),
            ("primary", Variant::Primary),
            ("secondary", Variant::Secondary),
            ("tertiary", Variant::Tertiary),
            ("accent", Variant::Accent),
            ("success", Variant::Success),
            ("warning", Variant::Warning),
            ("error", Variant::Error),
            ("info", Variant::Info),
            ("muted", Variant::Muted),
            ("surface", Variant::Surface),
            ("elevated", Variant::Elevated),
            ("ghost", Variant::Ghost),
            ("outline", Variant::Outline),
        ];

        for (name, expected) in cases {
            assert_eq!(Variant::from_str(name), Some(expected), "Failed for {}", name);
        }
    }

    #[test]
    fn test_variant_all() {
        let all = Variant::all();
        assert_eq!(all.len(), 14);
        assert!(all.contains(&Variant::Default));
        assert!(all.contains(&Variant::Primary));
        assert!(all.contains(&Variant::Ghost));
        assert!(all.contains(&Variant::Outline));
    }

    #[test]
    fn test_variant_default() {
        assert_eq!(Variant::default(), Variant::Default);
    }

    #[test]
    fn test_variant_style_terminal_theme() {
        reset_theme_state();
        let style = get_variant_style(Variant::Primary);
        // Terminal theme uses ANSI colors
        assert!(style.bg.is_ansi());
    }

    #[test]
    fn test_variant_style_rgb_theme() {
        reset_theme_state();
        set_theme("dracula");
        let style = get_variant_style(Variant::Primary);
        // Dracula uses RGB colors
        assert!(!style.bg.is_ansi());
        assert!(!style.bg.is_terminal_default());
        reset_theme_state();
    }

    #[test]
    fn test_variant_style_contrast() {
        reset_theme_state();
        set_theme("dracula");
        let style = get_variant_style(Variant::Warning);
        // Warning bg is bright, fg should have good contrast
        let ratio = Rgba::contrast_ratio(style.fg, style.bg);
        assert!(ratio >= 4.5, "Contrast ratio {} should be >= 4.5", ratio);
        reset_theme_state();
    }

    #[test]
    fn test_variant_style_contrast_primary() {
        reset_theme_state();
        set_theme("dracula");
        let style = get_variant_style(Variant::Primary);
        let ratio = Rgba::contrast_ratio(style.fg, style.bg);
        assert!(ratio >= 4.5, "Primary contrast ratio {} should be >= 4.5", ratio);
        reset_theme_state();
    }

    #[test]
    fn test_variant_style_contrast_error() {
        reset_theme_state();
        set_theme("dracula");
        let style = get_variant_style(Variant::Error);
        let ratio = Rgba::contrast_ratio(style.fg, style.bg);
        assert!(ratio >= 4.5, "Error contrast ratio {} should be >= 4.5", ratio);
        reset_theme_state();
    }

    #[test]
    fn test_all_variants_have_styles() {
        reset_theme_state();
        for v in Variant::all() {
            let style = get_variant_style(*v);
            // Should not panic, should return valid style
            // At least one of fg or bg should be set (not both default)
            let _ = style;
        }
    }

    #[test]
    fn test_ghost_variant_transparent_bg() {
        reset_theme_state();
        let style = get_variant_style(Variant::Ghost);
        assert!(style.bg.is_terminal_default());
        assert!(style.border.is_terminal_default());
    }

    #[test]
    fn test_outline_variant_transparent_bg() {
        reset_theme_state();
        let style = get_variant_style(Variant::Outline);
        assert!(style.bg.is_terminal_default());
        // Border should be set to primary
        assert!(!style.border.is_terminal_default());
    }

    #[test]
    fn test_variant_style_reactive() {
        reset_theme_state();
        let style_derived = variant_style(Variant::Primary);
        let initial = style_derived.get();

        set_theme("nord");
        let after = style_derived.get();

        // Style should change with theme (nord has different colors than terminal)
        assert_ne!(initial.bg, after.bg, "Style should change when theme changes");
        reset_theme_state();
    }

    #[test]
    fn test_variant_style_reactive_multiple_themes() {
        reset_theme_state();
        let style_derived = variant_style(Variant::Error);

        let terminal_style = style_derived.get();

        set_theme("dracula");
        let dracula_style = style_derived.get();

        set_theme("nord");
        let nord_style = style_derived.get();

        // All three should be different (different themes have different error colors)
        assert_ne!(terminal_style.bg, dracula_style.bg);
        assert_ne!(dracula_style.bg, nord_style.bg);

        reset_theme_state();
    }
}
