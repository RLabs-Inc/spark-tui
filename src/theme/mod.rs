//! Theme System for spark-tui.
//!
//! Provides semantic color definitions with support for ANSI, RGB, and OKLCH colors.
//! The theme system respects terminal color schemes when using ANSI colors and
//! provides 13 built-in presets (terminal, dracula, nord, etc.).
//!
//! # Color Types
//!
//! - `ThemeColor::Default` - Uses terminal's default color
//! - `ThemeColor::Ansi(n)` - ANSI palette index (0-255)
//! - `ThemeColor::Rgb(rgba)` - Explicit RGB color
//! - `ThemeColor::Str(s)` - String to be parsed (hex, oklch, etc.)
//!
//! # Example
//!
//! ```rust
//! use spark_tui::theme::{Theme, ThemeColor, get_preset};
//!
//! // Get a preset theme
//! let dracula = get_preset("dracula").unwrap();
//!
//! // Use theme colors
//! let primary = dracula.primary.resolve();
//! ```

use spark_signals::Reactive;
use crate::types::Rgba;

pub mod accessor;
pub mod modifiers;
pub mod presets;
pub mod reactive;
pub mod variant;

pub use accessor::{t, reset_accessor, ThemeAccessor, ModifiableColor};
pub use modifiers::*;
pub use presets::*;
pub use reactive::*;
pub use variant::{Variant, VariantStyle, get_variant_style, variant_style};

// =============================================================================
// ThemeColor - A color that can be ANSI, RGB, or string
// =============================================================================

/// Theme color can be:
/// - `Default`: Terminal's default color
/// - `Ansi(n)`: ANSI palette index (0-255)
/// - `Rgb(rgba)`: Explicit RGB color
/// - `Str(s)`: String to be parsed (hex, oklch, etc.)
#[derive(Debug, Clone, PartialEq)]
pub enum ThemeColor {
    /// Use terminal's default color.
    Default,
    /// ANSI palette index (0-255).
    /// - 0-7: Standard colors
    /// - 8-15: Bright colors
    /// - 16-231: 6x6x6 RGB cube
    /// - 232-255: Grayscale
    Ansi(u8),
    /// Explicit RGB color.
    Rgb(Rgba),
    /// String to be parsed (hex, oklch, etc.).
    Str(String),
}

impl ThemeColor {
    /// Resolve to Rgba. Parses string if needed.
    ///
    /// - `Default` returns `Rgba::TERMINAL_DEFAULT`
    /// - `Ansi(n)` returns `Rgba::ansi(n)`
    /// - `Rgb(c)` returns the color directly
    /// - `Str(s)` parses the string, returning magenta on parse failure
    pub fn resolve(&self) -> Rgba {
        match self {
            Self::Default => Rgba::TERMINAL_DEFAULT,
            Self::Ansi(i) => Rgba::ansi(*i),
            Self::Rgb(c) => *c,
            Self::Str(s) => Rgba::parse(s).unwrap_or(Rgba::MAGENTA),
        }
    }

    /// Check if this is the terminal default.
    pub fn is_default(&self) -> bool {
        matches!(self, Self::Default)
    }

    /// Check if this is an ANSI color.
    pub fn is_ansi(&self) -> bool {
        matches!(self, Self::Ansi(_))
    }

    /// Check if this is an RGB color.
    pub fn is_rgb(&self) -> bool {
        matches!(self, Self::Rgb(_))
    }
}

// =============================================================================
// From implementations for ergonomic construction
// =============================================================================

impl Default for ThemeColor {
    fn default() -> Self {
        Self::Default
    }
}

/// `()` means terminal default.
impl From<()> for ThemeColor {
    fn from(_: ()) -> Self {
        Self::Default
    }
}

/// `u8` is an ANSI index.
impl From<u8> for ThemeColor {
    fn from(index: u8) -> Self {
        Self::Ansi(index)
    }
}

/// `Rgba` is an RGB color.
impl From<Rgba> for ThemeColor {
    fn from(color: Rgba) -> Self {
        Self::Rgb(color)
    }
}

/// `&str` is a string to parse.
impl From<&str> for ThemeColor {
    fn from(s: &str) -> Self {
        Self::Str(s.to_string())
    }
}

/// `String` is a string to parse.
impl From<String> for ThemeColor {
    fn from(s: String) -> Self {
        Self::Str(s)
    }
}

/// `u32` is an RGB integer (0xRRGGBB).
impl From<u32> for ThemeColor {
    fn from(rgb: u32) -> Self {
        Self::Rgb(Rgba::from_rgb_int(rgb))
    }
}

/// `Option<u32>` - None is default, Some is RGB.
impl From<Option<u32>> for ThemeColor {
    fn from(opt: Option<u32>) -> Self {
        match opt {
            None => Self::Default,
            Some(rgb) => Self::Rgb(Rgba::from_rgb_int(rgb)),
        }
    }
}

// =============================================================================
// Theme - All semantic colors
// =============================================================================

/// Theme definition with all semantic colors.
///
/// Contains 20 color slots organized into categories:
/// - Main palette: primary, secondary, tertiary, accent
/// - Semantic: success, warning, error, info
/// - Text: text, text_muted, text_dim, text_disabled, text_bright
/// - Background: background, background_muted, surface, overlay
/// - Border: border, border_focus
///
/// The `#[derive(Reactive)]` macro generates `ReactiveTheme` where each field
/// becomes a `Signal<T>`. This enables fine-grained reactivity - changing one
/// color only notifies deriveds reading that specific color.
#[derive(Debug, Clone, Reactive)]
pub struct Theme {
    /// Theme name (e.g., "dracula", "nord").
    pub name: String,
    /// Theme description.
    pub description: String,

    // =========================================================================
    // Main Palette
    // =========================================================================

    /// Primary brand color.
    pub primary: ThemeColor,
    /// Secondary accent color.
    pub secondary: ThemeColor,
    /// Tertiary color for variety.
    pub tertiary: ThemeColor,
    /// Accent for highlights.
    pub accent: ThemeColor,

    // =========================================================================
    // Semantic Colors
    // =========================================================================

    /// Success/positive feedback.
    pub success: ThemeColor,
    /// Warning/caution.
    pub warning: ThemeColor,
    /// Error/danger.
    pub error: ThemeColor,
    /// Informational.
    pub info: ThemeColor,

    // =========================================================================
    // Text Colors
    // =========================================================================

    /// Primary text color.
    pub text: ThemeColor,
    /// Muted/secondary text.
    pub text_muted: ThemeColor,
    /// Dimmed text (less prominent than muted).
    pub text_dim: ThemeColor,
    /// Disabled/inactive text.
    pub text_disabled: ThemeColor,
    /// Bright/emphasized text.
    pub text_bright: ThemeColor,

    // =========================================================================
    // Background Colors
    // =========================================================================

    /// Primary background.
    pub background: ThemeColor,
    /// Muted/alternate background.
    pub background_muted: ThemeColor,
    /// Surface (cards, panels).
    pub surface: ThemeColor,
    /// Overlay (modals, dropdowns).
    pub overlay: ThemeColor,

    // =========================================================================
    // Border Colors
    // =========================================================================

    /// Default border color.
    pub border: ThemeColor,
    /// Focused border color.
    pub border_focus: ThemeColor,
}

impl Default for Theme {
    fn default() -> Self {
        terminal()
    }
}

impl Theme {
    /// Create a new theme with all default colors.
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            primary: ThemeColor::Default,
            secondary: ThemeColor::Default,
            tertiary: ThemeColor::Default,
            accent: ThemeColor::Default,
            success: ThemeColor::Default,
            warning: ThemeColor::Default,
            error: ThemeColor::Default,
            info: ThemeColor::Default,
            text: ThemeColor::Default,
            text_muted: ThemeColor::Default,
            text_dim: ThemeColor::Default,
            text_disabled: ThemeColor::Default,
            text_bright: ThemeColor::Default,
            background: ThemeColor::Default,
            background_muted: ThemeColor::Default,
            surface: ThemeColor::Default,
            overlay: ThemeColor::Default,
            border: ThemeColor::Default,
            border_focus: ThemeColor::Default,
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_color_default() {
        let color = ThemeColor::Default;
        assert!(color.is_default());
        assert!(!color.is_ansi());
        assert!(!color.is_rgb());
        assert!(color.resolve().is_terminal_default());
    }

    #[test]
    fn test_theme_color_ansi() {
        let color = ThemeColor::Ansi(12);
        assert!(!color.is_default());
        assert!(color.is_ansi());
        assert!(!color.is_rgb());

        let resolved = color.resolve();
        assert!(resolved.is_ansi());
        assert_eq!(resolved.ansi_index(), 12);
    }

    #[test]
    fn test_theme_color_rgb() {
        let color = ThemeColor::Rgb(Rgba::rgb(255, 0, 0));
        assert!(!color.is_default());
        assert!(!color.is_ansi());
        assert!(color.is_rgb());

        let resolved = color.resolve();
        assert_eq!(resolved, Rgba::rgb(255, 0, 0));
    }

    #[test]
    fn test_theme_color_str_hex() {
        let color = ThemeColor::Str("#ff0000".to_string());
        let resolved = color.resolve();
        assert_eq!(resolved, Rgba::rgb(255, 0, 0));
    }

    #[test]
    fn test_theme_color_str_oklch() {
        let color = ThemeColor::Str("oklch(0.75 0.15 300)".to_string());
        let resolved = color.resolve();
        // Should be purple-ish (high blue)
        assert!(resolved.b > 200);
    }

    #[test]
    fn test_theme_color_str_invalid() {
        let color = ThemeColor::Str("invalid".to_string());
        let resolved = color.resolve();
        // Falls back to magenta
        assert_eq!(resolved, Rgba::MAGENTA);
    }

    #[test]
    fn test_theme_color_from_unit() {
        let color: ThemeColor = ().into();
        assert!(color.is_default());
    }

    #[test]
    fn test_theme_color_from_u8() {
        let color: ThemeColor = 12u8.into();
        assert_eq!(color, ThemeColor::Ansi(12));
    }

    #[test]
    fn test_theme_color_from_rgba() {
        let color: ThemeColor = Rgba::RED.into();
        assert_eq!(color, ThemeColor::Rgb(Rgba::RED));
    }

    #[test]
    fn test_theme_color_from_str() {
        let color: ThemeColor = "#ff0000".into();
        assert_eq!(color, ThemeColor::Str("#ff0000".to_string()));
    }

    #[test]
    fn test_theme_color_from_u32() {
        let color: ThemeColor = 0xff0000u32.into();
        assert_eq!(color, ThemeColor::Rgb(Rgba::rgb(255, 0, 0)));
    }

    #[test]
    fn test_theme_color_from_option_none() {
        let color: ThemeColor = None.into();
        assert!(color.is_default());
    }

    #[test]
    fn test_theme_color_from_option_some() {
        let color: ThemeColor = Some(0xff0000u32).into();
        assert_eq!(color, ThemeColor::Rgb(Rgba::rgb(255, 0, 0)));
    }

    #[test]
    fn test_theme_default() {
        let theme = Theme::default();
        assert_eq!(theme.name, "terminal");
    }

    #[test]
    fn test_theme_new() {
        let theme = Theme::new("custom", "My custom theme");
        assert_eq!(theme.name, "custom");
        assert_eq!(theme.description, "My custom theme");
        assert!(theme.primary.is_default());
    }
}
