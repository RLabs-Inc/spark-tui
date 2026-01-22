//! Theme accessor for reactive color access.
//!
//! The t.* pattern provides ergonomic access to theme colors.
//! Each color accessor reads from the specific color's Signal and resolves to Rgba,
//! enabling fine-grained reactivity - only reading from ONE signal per color.

use std::cell::RefCell;
use spark_signals::Signal;
use crate::types::Rgba;
use super::{ThemeColor, reactive::get_reactive_theme};

/// Accessor for reactive theme colors.
///
/// Each field is a Signal<ThemeColor> from the ReactiveTheme.
/// To get the resolved Rgba, call `.get().resolve()` on any field.
///
/// The key property is fine-grained reactivity: reading `accessor.primary.get()`
/// only tracks the primary signal, not the entire theme. Effects that read
/// primary won't re-run when secondary changes.
///
/// # Example
/// ```ignore
/// use spark_tui::theme::t;
///
/// // Get accessor
/// let theme = t();
///
/// // Get a color as Rgba
/// let primary_color = theme.primary(); // Rgba
///
/// // Or use the signal directly for reactive tracking
/// let primary_signal = theme.primary_signal();
/// let color = primary_signal.get().resolve();
/// ```
#[derive(Clone)]
pub struct ThemeAccessor {
    // Main palette
    primary_sig: Signal<ThemeColor>,
    secondary_sig: Signal<ThemeColor>,
    tertiary_sig: Signal<ThemeColor>,
    accent_sig: Signal<ThemeColor>,

    // Semantic
    success_sig: Signal<ThemeColor>,
    warning_sig: Signal<ThemeColor>,
    error_sig: Signal<ThemeColor>,
    info_sig: Signal<ThemeColor>,

    // Text
    text_sig: Signal<ThemeColor>,
    text_muted_sig: Signal<ThemeColor>,
    text_dim_sig: Signal<ThemeColor>,
    text_disabled_sig: Signal<ThemeColor>,
    text_bright_sig: Signal<ThemeColor>,

    // Backgrounds
    background_sig: Signal<ThemeColor>,
    background_muted_sig: Signal<ThemeColor>,
    surface_sig: Signal<ThemeColor>,
    overlay_sig: Signal<ThemeColor>,

    // Borders
    border_sig: Signal<ThemeColor>,
    border_focus_sig: Signal<ThemeColor>,
}

impl ThemeAccessor {
    /// Create a new accessor from the reactive theme.
    pub fn new() -> Self {
        let rt = get_reactive_theme();

        Self {
            primary_sig: rt.primary,
            secondary_sig: rt.secondary,
            tertiary_sig: rt.tertiary,
            accent_sig: rt.accent,
            success_sig: rt.success,
            warning_sig: rt.warning,
            error_sig: rt.error,
            info_sig: rt.info,
            text_sig: rt.text,
            text_muted_sig: rt.text_muted,
            text_dim_sig: rt.text_dim,
            text_disabled_sig: rt.text_disabled,
            text_bright_sig: rt.text_bright,
            background_sig: rt.background,
            background_muted_sig: rt.background_muted,
            surface_sig: rt.surface,
            overlay_sig: rt.overlay,
            border_sig: rt.border,
            border_focus_sig: rt.border_focus,
        }
    }

    // =========================================================================
    // Color accessor methods - read and resolve in one call
    // =========================================================================

    /// Get primary color as Rgba. Tracks only the primary signal.
    #[inline]
    pub fn primary(&self) -> Rgba {
        self.primary_sig.get().resolve()
    }

    /// Get secondary color as Rgba. Tracks only the secondary signal.
    #[inline]
    pub fn secondary(&self) -> Rgba {
        self.secondary_sig.get().resolve()
    }

    /// Get tertiary color as Rgba. Tracks only the tertiary signal.
    #[inline]
    pub fn tertiary(&self) -> Rgba {
        self.tertiary_sig.get().resolve()
    }

    /// Get accent color as Rgba. Tracks only the accent signal.
    #[inline]
    pub fn accent(&self) -> Rgba {
        self.accent_sig.get().resolve()
    }

    /// Get success color as Rgba. Tracks only the success signal.
    #[inline]
    pub fn success(&self) -> Rgba {
        self.success_sig.get().resolve()
    }

    /// Get warning color as Rgba. Tracks only the warning signal.
    #[inline]
    pub fn warning(&self) -> Rgba {
        self.warning_sig.get().resolve()
    }

    /// Get error color as Rgba. Tracks only the error signal.
    #[inline]
    pub fn error(&self) -> Rgba {
        self.error_sig.get().resolve()
    }

    /// Get info color as Rgba. Tracks only the info signal.
    #[inline]
    pub fn info(&self) -> Rgba {
        self.info_sig.get().resolve()
    }

    /// Get text color as Rgba. Tracks only the text signal.
    #[inline]
    pub fn text(&self) -> Rgba {
        self.text_sig.get().resolve()
    }

    /// Get text_muted color as Rgba. Tracks only the text_muted signal.
    #[inline]
    pub fn text_muted(&self) -> Rgba {
        self.text_muted_sig.get().resolve()
    }

    /// Get text_dim color as Rgba. Tracks only the text_dim signal.
    #[inline]
    pub fn text_dim(&self) -> Rgba {
        self.text_dim_sig.get().resolve()
    }

    /// Get text_disabled color as Rgba. Tracks only the text_disabled signal.
    #[inline]
    pub fn text_disabled(&self) -> Rgba {
        self.text_disabled_sig.get().resolve()
    }

    /// Get text_bright color as Rgba. Tracks only the text_bright signal.
    #[inline]
    pub fn text_bright(&self) -> Rgba {
        self.text_bright_sig.get().resolve()
    }

    /// Get background color as Rgba. Tracks only the background signal.
    #[inline]
    pub fn bg(&self) -> Rgba {
        self.background_sig.get().resolve()
    }

    /// Get background_muted color as Rgba. Tracks only the background_muted signal.
    #[inline]
    pub fn bg_muted(&self) -> Rgba {
        self.background_muted_sig.get().resolve()
    }

    /// Get surface color as Rgba. Tracks only the surface signal.
    #[inline]
    pub fn surface(&self) -> Rgba {
        self.surface_sig.get().resolve()
    }

    /// Get overlay color as Rgba. Tracks only the overlay signal.
    #[inline]
    pub fn overlay(&self) -> Rgba {
        self.overlay_sig.get().resolve()
    }

    /// Get border color as Rgba. Tracks only the border signal.
    #[inline]
    pub fn border(&self) -> Rgba {
        self.border_sig.get().resolve()
    }

    /// Get border_focus color as Rgba. Tracks only the border_focus signal.
    #[inline]
    pub fn border_focus(&self) -> Rgba {
        self.border_focus_sig.get().resolve()
    }

    // =========================================================================
    // Signal accessor methods - for creating deriveds
    // =========================================================================

    /// Get the primary color signal for creating deriveds.
    #[inline]
    pub fn primary_signal(&self) -> Signal<ThemeColor> {
        self.primary_sig.clone()
    }

    /// Get the secondary color signal for creating deriveds.
    #[inline]
    pub fn secondary_signal(&self) -> Signal<ThemeColor> {
        self.secondary_sig.clone()
    }

    /// Get the tertiary color signal for creating deriveds.
    #[inline]
    pub fn tertiary_signal(&self) -> Signal<ThemeColor> {
        self.tertiary_sig.clone()
    }

    /// Get the accent color signal for creating deriveds.
    #[inline]
    pub fn accent_signal(&self) -> Signal<ThemeColor> {
        self.accent_sig.clone()
    }

    /// Get the success color signal for creating deriveds.
    #[inline]
    pub fn success_signal(&self) -> Signal<ThemeColor> {
        self.success_sig.clone()
    }

    /// Get the warning color signal for creating deriveds.
    #[inline]
    pub fn warning_signal(&self) -> Signal<ThemeColor> {
        self.warning_sig.clone()
    }

    /// Get the error color signal for creating deriveds.
    #[inline]
    pub fn error_signal(&self) -> Signal<ThemeColor> {
        self.error_sig.clone()
    }

    /// Get the info color signal for creating deriveds.
    #[inline]
    pub fn info_signal(&self) -> Signal<ThemeColor> {
        self.info_sig.clone()
    }

    /// Get the text color signal for creating deriveds.
    #[inline]
    pub fn text_signal(&self) -> Signal<ThemeColor> {
        self.text_sig.clone()
    }

    /// Get the text_muted color signal for creating deriveds.
    #[inline]
    pub fn text_muted_signal(&self) -> Signal<ThemeColor> {
        self.text_muted_sig.clone()
    }

    /// Get the text_dim color signal for creating deriveds.
    #[inline]
    pub fn text_dim_signal(&self) -> Signal<ThemeColor> {
        self.text_dim_sig.clone()
    }

    /// Get the text_disabled color signal for creating deriveds.
    #[inline]
    pub fn text_disabled_signal(&self) -> Signal<ThemeColor> {
        self.text_disabled_sig.clone()
    }

    /// Get the text_bright color signal for creating deriveds.
    #[inline]
    pub fn text_bright_signal(&self) -> Signal<ThemeColor> {
        self.text_bright_sig.clone()
    }

    /// Get the background color signal for creating deriveds.
    #[inline]
    pub fn bg_signal(&self) -> Signal<ThemeColor> {
        self.background_sig.clone()
    }

    /// Get the background_muted color signal for creating deriveds.
    #[inline]
    pub fn bg_muted_signal(&self) -> Signal<ThemeColor> {
        self.background_muted_sig.clone()
    }

    /// Get the surface color signal for creating deriveds.
    #[inline]
    pub fn surface_signal(&self) -> Signal<ThemeColor> {
        self.surface_sig.clone()
    }

    /// Get the overlay color signal for creating deriveds.
    #[inline]
    pub fn overlay_signal(&self) -> Signal<ThemeColor> {
        self.overlay_sig.clone()
    }

    /// Get the border color signal for creating deriveds.
    #[inline]
    pub fn border_signal(&self) -> Signal<ThemeColor> {
        self.border_sig.clone()
    }

    /// Get the border_focus color signal for creating deriveds.
    #[inline]
    pub fn border_focus_signal(&self) -> Signal<ThemeColor> {
        self.border_focus_sig.clone()
    }
}

impl Default for ThemeAccessor {
    fn default() -> Self {
        Self::new()
    }
}

thread_local! {
    /// Cached accessor - created once per thread, reused.
    static ACCESSOR: RefCell<Option<ThemeAccessor>> = const { RefCell::new(None) };
}

/// Get the theme accessor.
///
/// Returns a ThemeAccessor with methods to get colors as Rgba or as Signals.
/// The accessor is cached per thread for efficiency.
///
/// # Example
/// ```ignore
/// use spark_tui::theme::{t, set_theme};
///
/// let theme = t();
/// let primary = theme.primary(); // Rgba
///
/// set_theme("dracula");
/// let new_primary = theme.primary(); // Updated!
/// ```
pub fn t() -> ThemeAccessor {
    ACCESSOR.with(|a| {
        let mut opt = a.borrow_mut();
        if opt.is_none() {
            *opt = Some(ThemeAccessor::new());
        }
        opt.clone().unwrap()
    })
}

/// Reset accessor cache (for testing).
pub fn reset_accessor() {
    ACCESSOR.with(|a| *a.borrow_mut() = None);
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{reset_theme_state, set_theme, get_reactive_theme, ThemeColor};
    use spark_signals::effect;
    use std::cell::Cell;
    use std::rc::Rc;

    #[test]
    fn test_t_accessor_returns_colors() {
        reset_theme_state();
        reset_accessor();
        let accessor = t();
        // Terminal theme uses ANSI 12 for primary
        let primary = accessor.primary();
        assert!(primary.is_ansi());
    }

    #[test]
    fn test_t_accessor_is_reactive() {
        reset_theme_state();
        reset_accessor();
        let accessor = t();

        let initial = accessor.primary();
        set_theme("dracula");
        let after = accessor.primary();

        assert_ne!(initial, after);
        reset_theme_state();
    }

    #[test]
    fn test_t_accessor_fine_grained() {
        reset_theme_state();
        reset_accessor();
        let accessor = t();

        // Track primary
        let primary_count = Rc::new(Cell::new(0));
        let count = primary_count.clone();
        let primary_sig = accessor.primary_signal();
        let _e1 = effect(move || {
            let _ = primary_sig.get();
            count.set(count.get() + 1);
        });

        // Track secondary
        let secondary_count = Rc::new(Cell::new(0));
        let count2 = secondary_count.clone();
        let secondary_sig = accessor.secondary_signal();
        let _e2 = effect(move || {
            let _ = secondary_sig.get();
            count2.set(count2.get() + 1);
        });

        assert_eq!(primary_count.get(), 1);
        assert_eq!(secondary_count.get(), 1);

        // Modify only primary via ReactiveTheme
        let rt = get_reactive_theme();
        rt.primary.set(ThemeColor::Rgb(Rgba::BLUE));

        // Only primary effect re-ran!
        assert_eq!(primary_count.get(), 2);
        assert_eq!(secondary_count.get(), 1);

        reset_theme_state();
    }

    #[test]
    fn test_set_theme_updates_all_colors() {
        reset_theme_state();
        reset_accessor();

        let accessor = t();
        let initial_primary = accessor.primary();
        let initial_secondary = accessor.secondary();

        set_theme("nord");

        let after_primary = accessor.primary();
        let after_secondary = accessor.secondary();

        // Both should change (nord has different colors)
        assert_ne!(initial_primary, after_primary);
        assert_ne!(initial_secondary, after_secondary);
        reset_theme_state();
    }

    #[test]
    fn test_t_accessor_bg_aliases() {
        reset_theme_state();
        reset_accessor();
        let accessor = t();

        // bg and bg_muted should both resolve to ANSI for terminal theme
        let bg = accessor.bg();
        let bg_muted = accessor.bg_muted();

        // Both are valid colors (either ANSI or terminal default)
        assert!(bg.is_ansi() || bg.is_terminal_default());
        assert!(bg_muted.is_ansi() || bg_muted.is_terminal_default());
    }
}
