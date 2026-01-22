//! Color modification functions for theme colors.
//!
//! Provides functions for manipulating colors in OKLCH space:
//! - lighten/darken: Adjust lightness
//! - saturate/desaturate: Adjust chroma (saturation)
//! - alpha/fade: Modify transparency
//! - mix: Blend two colors
//! - contrast: Ensure minimum contrast ratio
//!
//! These functions work on Rgba values. For ANSI or terminal default colors,
//! most functions return the original color unchanged since they cannot be
//! meaningfully modified in OKLCH space.
//!
//! # Example
//! ```rust
//! use spark_tui::types::Rgba;
//! use spark_tui::theme::modifiers::{lighten, darken, alpha};
//!
//! let color = Rgba::rgb(100, 50, 150);
//! let lighter = lighten(color, 0.2);  // 20% lighter
//! let darker = darken(color, 0.1);    // 10% darker
//! let semi = alpha(color, 0.5);       // 50% transparent
//! ```

use crate::types::Rgba;

// =============================================================================
// Lightness Modifiers
// =============================================================================

/// Lighten a color by adjusting OKLCH lightness.
///
/// Amount is 0.0-1.0 where 0.2 = 20% increase in lightness.
/// Returns the original color for ANSI or terminal default colors.
///
/// # Example
/// ```rust
/// use spark_tui::types::Rgba;
/// use spark_tui::theme::modifiers::lighten;
///
/// let gray = Rgba::rgb(100, 100, 100);
/// let lighter = lighten(gray, 0.2);
/// // lighter has higher RGB values
/// ```
#[inline]
pub fn lighten(color: Rgba, amount: f32) -> Rgba {
    color.adjust_lightness(amount).unwrap_or(color)
}

/// Darken a color by adjusting OKLCH lightness.
///
/// Amount is 0.0-1.0 where 0.2 = 20% decrease in lightness.
/// Returns the original color for ANSI or terminal default colors.
///
/// # Example
/// ```rust
/// use spark_tui::types::Rgba;
/// use spark_tui::theme::modifiers::darken;
///
/// let gray = Rgba::rgb(150, 150, 150);
/// let darker = darken(gray, 0.2);
/// // darker has lower RGB values
/// ```
#[inline]
pub fn darken(color: Rgba, amount: f32) -> Rgba {
    color.adjust_lightness(-amount).unwrap_or(color)
}

// =============================================================================
// Saturation Modifiers
// =============================================================================

/// Increase color saturation (chroma) by amount.
///
/// Amount is added to current chroma. Typical chroma range is 0.0-0.4,
/// where 0 is gray and ~0.4 is highly saturated.
/// Returns the original color for ANSI or terminal default colors.
///
/// # Example
/// ```rust
/// use spark_tui::types::Rgba;
/// use spark_tui::theme::modifiers::saturate;
///
/// let muted = Rgba::rgb(128, 100, 100);
/// let vivid = saturate(muted, 0.1);
/// // vivid has more color contrast
/// ```
#[inline]
pub fn saturate(color: Rgba, amount: f32) -> Rgba {
    if let Some((l, c, h)) = color.to_oklch() {
        let new_c = (c + amount).clamp(0.0, 0.5);
        Rgba::oklch(l, new_c, h, color.a as u8)
    } else {
        color
    }
}

/// Decrease color saturation (chroma) by amount.
///
/// Amount is subtracted from current chroma.
/// Returns the original color for ANSI or terminal default colors.
///
/// # Example
/// ```rust
/// use spark_tui::types::Rgba;
/// use spark_tui::theme::modifiers::desaturate;
///
/// let red = Rgba::RED;
/// let muted_red = desaturate(red, 0.1);
/// // muted_red is less saturated
/// ```
#[inline]
pub fn desaturate(color: Rgba, amount: f32) -> Rgba {
    if let Some((l, c, h)) = color.to_oklch() {
        let new_c = (c - amount).max(0.0);
        Rgba::oklch(l, new_c, h, color.a as u8)
    } else {
        color
    }
}

// =============================================================================
// Alpha Modifiers
// =============================================================================

/// Set alpha value.
///
/// If value <= 1.0, treat as a fraction (0.0-1.0 maps to 0-255).
/// If value > 1.0, treat as an absolute value (0-255).
///
/// # Example
/// ```rust
/// use spark_tui::types::Rgba;
/// use spark_tui::theme::modifiers::alpha;
///
/// let opaque = Rgba::RED;
/// let half = alpha(opaque, 0.5);    // a = 127
/// let fixed = alpha(opaque, 200.0); // a = 200
/// ```
#[inline]
pub fn alpha(color: Rgba, value: f32) -> Rgba {
    let a = if value <= 1.0 {
        (value * 255.0) as i16
    } else {
        value as i16
    };
    Rgba {
        a: a.clamp(0, 255),
        ..color
    }
}

/// Multiply alpha by a factor (0.0-1.0).
///
/// Useful for fading colors out gradually.
///
/// # Example
/// ```rust
/// use spark_tui::types::Rgba;
/// use spark_tui::theme::modifiers::fade;
///
/// let color = Rgba::new(100, 100, 100, 200);
/// let faded = fade(color, 0.5);  // a = 100
/// ```
#[inline]
pub fn fade(color: Rgba, factor: f32) -> Rgba {
    let new_a = (color.a as f32 * factor) as i16;
    Rgba {
        a: new_a.clamp(0, 255),
        ..color
    }
}

// =============================================================================
// Color Mixing
// =============================================================================

/// Mix two colors with linear interpolation.
///
/// Weight 0.0 = first color, 1.0 = second color, 0.5 = equal blend.
///
/// # Example
/// ```rust
/// use spark_tui::types::Rgba;
/// use spark_tui::theme::modifiers::mix;
///
/// let gray = mix(Rgba::BLACK, Rgba::WHITE, 0.5);
/// // gray is approximately (127, 127, 127)
/// ```
#[inline]
pub fn mix(a: Rgba, b: Rgba, weight: f32) -> Rgba {
    Rgba::lerp(a, b, weight)
}

// =============================================================================
// Contrast Calculation
// =============================================================================

/// Adjust foreground color to have minimum contrast against background.
///
/// Uses OKLCH to preserve hue while adjusting lightness to meet the
/// target contrast ratio. Returns the original color if it already
/// meets the requirement or cannot be adjusted (ANSI/terminal default).
///
/// # Arguments
/// * `fg` - Foreground color to adjust
/// * `bg` - Background color to contrast against
/// * `min_ratio` - Minimum contrast ratio (WCAG AA = 4.5, AAA = 7.0)
///
/// # Example
/// ```rust
/// use spark_tui::types::Rgba;
/// use spark_tui::theme::modifiers::contrast;
///
/// let dark_bg = Rgba::rgb(30, 30, 30);
/// let dark_fg = Rgba::rgb(50, 50, 50);
/// let readable = contrast(dark_fg, dark_bg, 4.5);
/// // readable is lighter to achieve 4.5:1 contrast
/// ```
#[inline]
pub fn contrast(fg: Rgba, bg: Rgba, min_ratio: f32) -> Rgba {
    Rgba::ensure_contrast(fg, bg, min_ratio).unwrap_or(fg)
}

/// Ensure WCAG AA contrast (4.5:1 ratio).
///
/// Convenience wrapper for `contrast(fg, bg, 4.5)`.
/// WCAG AA is the standard for normal text accessibility.
#[inline]
pub fn contrast_aa(fg: Rgba, bg: Rgba) -> Rgba {
    contrast(fg, bg, 4.5)
}

/// Ensure WCAG AAA contrast (7.0:1 ratio).
///
/// Convenience wrapper for `contrast(fg, bg, 7.0)`.
/// WCAG AAA is the enhanced level for better accessibility.
#[inline]
pub fn contrast_aaa(fg: Rgba, bg: Rgba) -> Rgba {
    contrast(fg, bg, 7.0)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lighten() {
        let color = Rgba::rgb(100, 100, 100);
        let lighter = lighten(color, 0.2);

        // Should be brighter (higher RGB values on average)
        let avg_before = (color.r + color.g + color.b) / 3;
        let avg_after = (lighter.r + lighter.g + lighter.b) / 3;
        assert!(
            avg_after > avg_before,
            "lighter {} should be > original {}",
            avg_after,
            avg_before
        );
    }

    #[test]
    fn test_darken() {
        let color = Rgba::rgb(150, 150, 150);
        let darker = darken(color, 0.2);

        let avg_before = (color.r + color.g + color.b) / 3;
        let avg_after = (darker.r + darker.g + darker.b) / 3;
        assert!(
            avg_after < avg_before,
            "darker {} should be < original {}",
            avg_after,
            avg_before
        );
    }

    #[test]
    fn test_lighten_ansi_returns_original() {
        let ansi = Rgba::ansi(4);
        let result = lighten(ansi, 0.2);
        assert_eq!(result, ansi);
    }

    #[test]
    fn test_darken_ansi_returns_original() {
        let ansi = Rgba::ansi(1);
        let result = darken(ansi, 0.2);
        assert_eq!(result, ansi);
    }

    #[test]
    fn test_lighten_terminal_default_returns_original() {
        let def = Rgba::TERMINAL_DEFAULT;
        let result = lighten(def, 0.2);
        assert_eq!(result, def);
    }

    #[test]
    fn test_alpha_fraction() {
        let color = Rgba::RED;
        let transparent = alpha(color, 0.5);
        assert_eq!(transparent.a, 127);
        assert_eq!(transparent.r, color.r);
        assert_eq!(transparent.g, color.g);
        assert_eq!(transparent.b, color.b);
    }

    #[test]
    fn test_alpha_absolute() {
        let color = Rgba::RED;
        let result = alpha(color, 200.0);
        assert_eq!(result.a, 200);
    }

    #[test]
    fn test_alpha_clamp() {
        let color = Rgba::RED;
        let over = alpha(color, 300.0);
        assert_eq!(over.a, 255);

        let under = alpha(color, -10.0);
        assert_eq!(under.a, 0);
    }

    #[test]
    fn test_fade() {
        let color = Rgba::new(100, 100, 100, 200);
        let faded = fade(color, 0.5);
        assert_eq!(faded.a, 100);
        assert_eq!(faded.r, color.r);
    }

    #[test]
    fn test_fade_to_zero() {
        let color = Rgba::RED;
        let invisible = fade(color, 0.0);
        assert_eq!(invisible.a, 0);
    }

    #[test]
    fn test_saturate() {
        let gray = Rgba::rgb(128, 128, 128);
        let saturated = saturate(gray, 0.1);

        // Gray has 0 chroma, adding some should create color
        let (_, c_before, _) = gray.to_oklch().unwrap();
        let (_, c_after, _) = saturated.to_oklch().unwrap();
        assert!(
            c_after > c_before,
            "saturated chroma {} should be > original {}",
            c_after,
            c_before
        );
    }

    #[test]
    fn test_desaturate() {
        let color = Rgba::rgb(255, 0, 0); // Pure red, high chroma
        let desat = desaturate(color, 0.1);

        let (_, c_before, _) = color.to_oklch().unwrap();
        let (_, c_after, _) = desat.to_oklch().unwrap();
        assert!(
            c_after < c_before,
            "desaturated chroma {} should be < original {}",
            c_after,
            c_before
        );
    }

    #[test]
    fn test_saturate_ansi_returns_original() {
        let ansi = Rgba::ansi(1);
        let result = saturate(ansi, 0.1);
        assert_eq!(result, ansi);
    }

    #[test]
    fn test_mix_equal() {
        let a = Rgba::BLACK;
        let b = Rgba::WHITE;
        let mixed = mix(a, b, 0.5);

        // Should be gray
        assert!(
            (mixed.r - 127).abs() <= 1,
            "r={} should be ~127",
            mixed.r
        );
        assert!(
            (mixed.g - 127).abs() <= 1,
            "g={} should be ~127",
            mixed.g
        );
        assert!(
            (mixed.b - 127).abs() <= 1,
            "b={} should be ~127",
            mixed.b
        );
    }

    #[test]
    fn test_mix_extremes() {
        let a = Rgba::RED;
        let b = Rgba::BLUE;

        // Weight 0 = first color
        let all_a = mix(a, b, 0.0);
        assert_eq!(all_a, a);

        // Weight 1 = second color
        let all_b = mix(a, b, 1.0);
        assert_eq!(all_b, b);
    }

    #[test]
    fn test_contrast_dark_on_dark() {
        let dark_bg = Rgba::rgb(30, 30, 30);
        // Use a colored foreground (not pure gray) for better OKLCH adjustment
        let dark_fg = Rgba::rgb(80, 60, 60);
        let adjusted = contrast(dark_fg, dark_bg, 4.5);

        let ratio = Rgba::contrast_ratio(adjusted, dark_bg);
        assert!(ratio >= 4.5, "Ratio {} should be >= 4.5", ratio);
    }

    #[test]
    fn test_contrast_light_on_light() {
        let light_bg = Rgba::rgb(230, 230, 230);
        let light_fg = Rgba::rgb(200, 200, 200);
        let adjusted = contrast(light_fg, light_bg, 4.5);

        let ratio = Rgba::contrast_ratio(adjusted, light_bg);
        assert!(ratio >= 4.5, "Ratio {} should be >= 4.5", ratio);
    }

    #[test]
    fn test_contrast_already_good() {
        let bg = Rgba::BLACK;
        let fg = Rgba::WHITE;
        let adjusted = contrast(fg, bg, 4.5);

        // Already has 21:1 contrast, should not change
        assert_eq!(adjusted.r, fg.r);
        assert_eq!(adjusted.g, fg.g);
        assert_eq!(adjusted.b, fg.b);
    }

    #[test]
    fn test_contrast_aa() {
        let bg = Rgba::rgb(50, 50, 50);
        let fg = Rgba::rgb(80, 80, 80);
        let adjusted = contrast_aa(fg, bg);

        let ratio = Rgba::contrast_ratio(adjusted, bg);
        assert!(ratio >= 4.5, "AA ratio {} should be >= 4.5", ratio);
    }

    #[test]
    fn test_contrast_aaa() {
        let bg = Rgba::rgb(50, 50, 50);
        let fg = Rgba::rgb(80, 80, 80);
        let adjusted = contrast_aaa(fg, bg);

        let ratio = Rgba::contrast_ratio(adjusted, bg);
        assert!(ratio >= 7.0, "AAA ratio {} should be >= 7.0", ratio);
    }

    #[test]
    fn test_contrast_ansi_returns_original() {
        let bg = Rgba::BLACK;
        let fg = Rgba::ansi(8);
        let result = contrast(fg, bg, 4.5);
        assert_eq!(result, fg);
    }
}
