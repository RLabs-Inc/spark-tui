//! Core types for spark-tui.
//!
//! These types define the foundation that everything builds on.
//! They flow through the reactive pipeline and define what the renderer understands.

// =============================================================================
// Color
// =============================================================================

/// RGBA color with 8-bit channels (0-255).
///
/// Using integers for exact comparison - no floating point epsilon needed.
/// Alpha 255 = fully opaque, 0 = fully transparent.
/// Special value: r=-1 means "terminal default" (let terminal pick).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rgba {
    pub r: i16,
    pub g: i16,
    pub b: i16,
    pub a: i16,
}

impl Rgba {
    /// Create a new RGBA color.
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as i16,
            g: g as i16,
            b: b as i16,
            a: a as i16,
        }
    }

    /// Create an opaque RGB color.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(r, g, b, 255)
    }

    /// Terminal default color (let terminal decide).
    pub const TERMINAL_DEFAULT: Self = Self {
        r: -1,
        g: -1,
        b: -1,
        a: -1,
    };

    /// Transparent color.
    pub const TRANSPARENT: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };

    // Standard colors
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    pub const RED: Self = Self::rgb(255, 0, 0);
    pub const GREEN: Self = Self::rgb(0, 255, 0);
    pub const BLUE: Self = Self::rgb(0, 0, 255);
    pub const YELLOW: Self = Self::rgb(255, 255, 0);
    pub const CYAN: Self = Self::rgb(0, 255, 255);
    pub const MAGENTA: Self = Self::rgb(255, 0, 255);
    pub const GRAY: Self = Self::rgb(128, 128, 128);

    /// Create an ANSI palette color (0-255).
    ///
    /// Uses special marker: r=-2, g=palette_index.
    /// - 0-7: Standard colors
    /// - 8-15: Bright colors
    /// - 16-231: 6x6x6 RGB cube
    /// - 232-255: Grayscale
    pub const fn ansi(index: u8) -> Self {
        Self {
            r: -2,
            g: index as i16,
            b: 0,
            a: 255,
        }
    }

    /// Check if this is the terminal default color.
    #[inline]
    pub const fn is_terminal_default(&self) -> bool {
        self.r == -1
    }

    /// Check if this is an ANSI palette color.
    #[inline]
    pub const fn is_ansi(&self) -> bool {
        self.r == -2
    }

    /// Get ANSI palette index (only valid if is_ansi() returns true).
    #[inline]
    pub const fn ansi_index(&self) -> u8 {
        self.g as u8
    }

    /// Check if color is fully opaque.
    #[inline]
    pub const fn is_opaque(&self) -> bool {
        self.a == 255
    }

    /// Check if color is fully transparent.
    #[inline]
    pub const fn is_transparent(&self) -> bool {
        self.a == 0
    }

    /// Alpha blend src over dst (Porter-Duff "over" operation).
    ///
    /// Returns the blended color. Handles terminal default and ANSI colors
    /// by treating them as opaque.
    #[inline]
    pub fn blend(src: Self, dst: Self) -> Self {
        // Fast path: fully opaque source
        if src.is_opaque() || src.is_terminal_default() || src.is_ansi() {
            return src;
        }

        // Fast path: fully transparent source
        if src.is_transparent() {
            return dst;
        }

        // Special colors as dst are treated as opaque black
        let (dr, dg, db, da) = if dst.is_terminal_default() || dst.is_ansi() {
            (0i16, 0i16, 0i16, 255i16)
        } else {
            (dst.r, dst.g, dst.b, dst.a)
        };

        let sa = src.a as i32;
        let inv_sa = 255 - sa;

        // out_a = src_a + dst_a * (1 - src_a)
        let out_a = sa + (da as i32 * inv_sa) / 255;

        if out_a == 0 {
            return Self::TRANSPARENT;
        }

        // out_rgb = (src_rgb * src_a + dst_rgb * dst_a * (1 - src_a)) / out_a
        let out_r = ((src.r as i32 * sa) + (dr as i32 * da as i32 * inv_sa / 255)) / out_a;
        let out_g = ((src.g as i32 * sa) + (dg as i32 * da as i32 * inv_sa / 255)) / out_a;
        let out_b = ((src.b as i32 * sa) + (db as i32 * da as i32 * inv_sa / 255)) / out_a;

        Self {
            r: out_r.clamp(0, 255) as i16,
            g: out_g.clamp(0, 255) as i16,
            b: out_b.clamp(0, 255) as i16,
            a: out_a.clamp(0, 255) as i16,
        }
    }

    /// Linear interpolation between two colors.
    #[inline]
    pub fn lerp(a: Self, b: Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        let inv_t = 1.0 - t;

        Self {
            r: ((a.r as f32 * inv_t) + (b.r as f32 * t)) as i16,
            g: ((a.g as f32 * inv_t) + (b.g as f32 * t)) as i16,
            b: ((a.b as f32 * inv_t) + (b.b as f32 * t)) as i16,
            a: ((a.a as f32 * inv_t) + (b.a as f32 * t)) as i16,
        }
    }

    /// Dim the color by a factor (0.0 = black, 1.0 = unchanged).
    #[inline]
    pub fn dim(self, factor: f32) -> Self {
        if self.is_terminal_default() {
            return Self::GRAY;
        }
        if self.is_ansi() {
            return self; // Can't dim ANSI colors
        }
        Self {
            r: (self.r as f32 * factor).clamp(0.0, 255.0) as i16,
            g: (self.g as f32 * factor).clamp(0.0, 255.0) as i16,
            b: (self.b as f32 * factor).clamp(0.0, 255.0) as i16,
            a: self.a,
        }
    }

    // =========================================================================
    // OKLCH Color Space Support
    // =========================================================================

    /// Create color from OKLCH (perceptually uniform color space).
    ///
    /// - `l`: Lightness (0.0 = black, 1.0 = white)
    /// - `c`: Chroma (0.0 = gray, ~0.4 = vivid, typical max ~0.37)
    /// - `h`: Hue in degrees (0-360)
    /// - `a`: Alpha (0-255)
    ///
    /// OKLCH is ideal for:
    /// - Generating harmonious color schemes
    /// - Adjusting lightness without changing perceived hue
    /// - Smooth hue gradients that maintain perceived brightness
    pub fn oklch(l: f32, c: f32, h: f32, a: u8) -> Self {
        // Convert OKLCH to OKLab
        let h_rad = h.to_radians();
        let lab_a = c * h_rad.cos();
        let lab_b = c * h_rad.sin();

        // Convert OKLab to linear sRGB via LMS
        let l_ = l + 0.3963377774 * lab_a + 0.2158037573 * lab_b;
        let m_ = l - 0.1055613458 * lab_a - 0.0638541728 * lab_b;
        let s_ = l - 0.0894841775 * lab_a - 1.2914855480 * lab_b;

        let l_cubed = l_ * l_ * l_;
        let m_cubed = m_ * m_ * m_;
        let s_cubed = s_ * s_ * s_;

        let r_linear = 4.0767416621 * l_cubed - 3.3077115913 * m_cubed + 0.2309699292 * s_cubed;
        let g_linear = -1.2684380046 * l_cubed + 2.6097574011 * m_cubed - 0.3413193965 * s_cubed;
        let b_linear = -0.0041960863 * l_cubed - 0.7034186147 * m_cubed + 1.7076147010 * s_cubed;

        // Linear to sRGB gamma
        fn linear_to_srgb(x: f32) -> f32 {
            if x <= 0.0031308 {
                x * 12.92
            } else {
                1.055 * x.powf(1.0 / 2.4) - 0.055
            }
        }

        let r = (linear_to_srgb(r_linear) * 255.0).clamp(0.0, 255.0) as u8;
        let g = (linear_to_srgb(g_linear) * 255.0).clamp(0.0, 255.0) as u8;
        let b = (linear_to_srgb(b_linear) * 255.0).clamp(0.0, 255.0) as u8;

        Self::new(r, g, b, a)
    }

    /// Convert to OKLCH color space.
    ///
    /// Returns (lightness, chroma, hue_degrees).
    /// Returns None for terminal default or ANSI colors.
    pub fn to_oklch(&self) -> Option<(f32, f32, f32)> {
        if self.is_terminal_default() || self.is_ansi() {
            return None;
        }

        // sRGB to linear
        fn srgb_to_linear(x: f32) -> f32 {
            if x <= 0.04045 {
                x / 12.92
            } else {
                ((x + 0.055) / 1.055).powf(2.4)
            }
        }

        let r_linear = srgb_to_linear(self.r as f32 / 255.0);
        let g_linear = srgb_to_linear(self.g as f32 / 255.0);
        let b_linear = srgb_to_linear(self.b as f32 / 255.0);

        // Linear RGB to LMS
        let l = 0.4122214708 * r_linear + 0.5363325363 * g_linear + 0.0514459929 * b_linear;
        let m = 0.2119034982 * r_linear + 0.6806995451 * g_linear + 0.1073969566 * b_linear;
        let s = 0.0883024619 * r_linear + 0.2817188376 * g_linear + 0.6299787005 * b_linear;

        // LMS to OKLab
        let l_ = l.cbrt();
        let m_ = m.cbrt();
        let s_ = s.cbrt();

        let lab_l = 0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_;
        let lab_a = 1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_;
        let lab_b = 0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_;

        // OKLab to OKLCH
        let c = (lab_a * lab_a + lab_b * lab_b).sqrt();
        let h = lab_b.atan2(lab_a).to_degrees();
        let h = if h < 0.0 { h + 360.0 } else { h };

        Some((lab_l, c, h))
    }

    /// Adjust lightness while preserving hue and chroma.
    ///
    /// `delta` is added to the current lightness (can be negative).
    /// Returns None for terminal default or ANSI colors.
    pub fn adjust_lightness(&self, delta: f32) -> Option<Self> {
        let (l, c, h) = self.to_oklch()?;
        let new_l = (l + delta).clamp(0.0, 1.0);
        Some(Self::oklch(new_l, c, h, self.a as u8))
    }

    /// Calculate relative luminance for WCAG contrast calculations.
    pub fn relative_luminance(&self) -> f32 {
        if self.is_terminal_default() || self.is_ansi() {
            return 0.0; // Assume dark for special colors
        }

        fn channel_luminance(c: i16) -> f32 {
            let c = c as f32 / 255.0;
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        }

        0.2126 * channel_luminance(self.r)
            + 0.7152 * channel_luminance(self.g)
            + 0.0722 * channel_luminance(self.b)
    }

    /// Calculate WCAG 2.1 contrast ratio between two colors.
    ///
    /// Returns a value between 1.0 and 21.0.
    /// WCAG AA requires 4.5:1 for normal text, 3:1 for large text.
    /// WCAG AAA requires 7:1 for normal text, 4.5:1 for large text.
    pub fn contrast_ratio(c1: Self, c2: Self) -> f32 {
        let l1 = c1.relative_luminance();
        let l2 = c2.relative_luminance();
        let lighter = l1.max(l2);
        let darker = l1.min(l2);
        (lighter + 0.05) / (darker + 0.05)
    }

    /// Adjust foreground lightness to meet minimum contrast ratio against background.
    ///
    /// Uses binary search to find optimal lightness adjustment.
    /// Returns None if target contrast cannot be achieved.
    pub fn ensure_contrast(fg: Self, bg: Self, min_ratio: f32) -> Option<Self> {
        if Self::contrast_ratio(fg, bg) >= min_ratio {
            return Some(fg);
        }

        let (l, c, h) = fg.to_oklch()?;
        let (bg_l, _, _) = bg.to_oklch()?;

        // Determine initial direction using OKLCH lightness (matches TypeScript):
        // Dark bg (L <= 0.5) needs lighter fg; bright bg (L > 0.5) needs darker fg
        let make_lighter = bg_l <= 0.5;

        // Helper to do binary search in a direction
        fn search(l: f32, c: f32, h: f32, alpha: u8, bg: Rgba, min_ratio: f32, go_lighter: bool) -> Option<Rgba> {
            let (mut lo, mut hi) = if go_lighter {
                (l, 1.0)
            } else {
                (0.0, l)
            };

            // If no search range, return None
            if (hi - lo).abs() < 0.001 {
                return None;
            }

            let mut best: Option<Rgba> = None;

            for _ in 0..20 {
                let mid = (lo + hi) / 2.0;
                let candidate = Rgba::oklch(mid, c, h, alpha);
                let ratio = Rgba::contrast_ratio(candidate, bg);

                if ratio >= min_ratio {
                    best = Some(candidate);
                    if go_lighter {
                        hi = mid;
                    } else {
                        lo = mid;
                    }
                } else {
                    if go_lighter {
                        lo = mid;
                    } else {
                        hi = mid;
                    }
                }
            }

            best
        }

        // Try preferred direction first
        if let Some(result) = search(l, c, h, fg.a as u8, bg, min_ratio, make_lighter) {
            return Some(result);
        }

        // If that didn't work (e.g., white on medium bg), try the other direction
        search(l, c, h, fg.a as u8, bg, min_ratio, !make_lighter)
    }

    // =========================================================================
    // Color Parsing
    // =========================================================================

    /// Create from 0xRRGGBB integer format.
    ///
    /// # Examples
    ///
    /// ```
    /// use spark_tui::types::Rgba;
    ///
    /// let red = Rgba::from_rgb_int(0xff0000);
    /// assert_eq!(red, Rgba::rgb(255, 0, 0));
    ///
    /// let dracula_bg = Rgba::from_rgb_int(0x282a36);
    /// assert_eq!(dracula_bg, Rgba::rgb(40, 42, 54));
    /// ```
    pub const fn from_rgb_int(rgb: u32) -> Self {
        Self::rgb(
            ((rgb >> 16) & 0xFF) as u8,
            ((rgb >> 8) & 0xFF) as u8,
            (rgb & 0xFF) as u8,
        )
    }

    /// Parse hex color string (#RGB, #RRGGBB, #RRGGBBAA).
    ///
    /// Returns None for invalid format.
    ///
    /// # Examples
    ///
    /// ```
    /// use spark_tui::types::Rgba;
    ///
    /// // #RRGGBB format
    /// let red = Rgba::from_hex("#ff0000").unwrap();
    /// assert_eq!(red, Rgba::rgb(255, 0, 0));
    ///
    /// // #RGB shorthand (expands each digit)
    /// let white = Rgba::from_hex("#fff").unwrap();
    /// assert_eq!(white, Rgba::rgb(255, 255, 255));
    ///
    /// // #RRGGBBAA format (with alpha)
    /// let semi = Rgba::from_hex("#ff000080").unwrap();
    /// assert_eq!(semi, Rgba::new(255, 0, 0, 128));
    ///
    /// // Without # prefix also works
    /// let blue = Rgba::from_hex("0000ff").unwrap();
    /// assert_eq!(blue, Rgba::rgb(0, 0, 255));
    ///
    /// // Invalid returns None
    /// assert!(Rgba::from_hex("invalid").is_none());
    /// assert!(Rgba::from_hex("#gg0000").is_none());
    /// ```
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim().trim_start_matches('#');

        // Helper to parse a single hex digit
        fn hex_digit(c: u8) -> Option<u8> {
            match c {
                b'0'..=b'9' => Some(c - b'0'),
                b'a'..=b'f' => Some(c - b'a' + 10),
                b'A'..=b'F' => Some(c - b'A' + 10),
                _ => None,
            }
        }

        // Helper to parse two hex digits
        fn hex_byte(s: &[u8], i: usize) -> Option<u8> {
            let high = hex_digit(s[i])?;
            let low = hex_digit(s[i + 1])?;
            Some((high << 4) | low)
        }

        let bytes = hex.as_bytes();
        match bytes.len() {
            // #RGB -> expand to #RRGGBB
            3 => {
                let r = hex_digit(bytes[0])?;
                let g = hex_digit(bytes[1])?;
                let b = hex_digit(bytes[2])?;
                Some(Self::rgb((r << 4) | r, (g << 4) | g, (b << 4) | b))
            }
            // #RRGGBB
            6 => {
                let r = hex_byte(bytes, 0)?;
                let g = hex_byte(bytes, 2)?;
                let b = hex_byte(bytes, 4)?;
                Some(Self::rgb(r, g, b))
            }
            // #RRGGBBAA
            8 => {
                let r = hex_byte(bytes, 0)?;
                let g = hex_byte(bytes, 2)?;
                let b = hex_byte(bytes, 4)?;
                let a = hex_byte(bytes, 6)?;
                Some(Self::new(r, g, b, a))
            }
            _ => None,
        }
    }

    /// Parse OKLCH color string: "oklch(L C H)" or "oklch(L C H / A)".
    ///
    /// - L: Lightness 0-1 (or 0%-100%)
    /// - C: Chroma 0-0.4 roughly (0.15 is good for vivid colors)
    /// - H: Hue 0-360 degrees (or rad/turn units)
    /// - A: Alpha 0-1 (or 0%-100%), optional
    ///
    /// # Examples
    ///
    /// ```
    /// use spark_tui::types::Rgba;
    ///
    /// // Basic OKLCH
    /// let purple = Rgba::from_oklch_str("oklch(0.75 0.15 300)").unwrap();
    ///
    /// // With percentage lightness
    /// let bright = Rgba::from_oklch_str("oklch(80% 0.2 180)").unwrap();
    ///
    /// // With alpha (0.5 * 255 = 127.5, rounds to 127 or 128)
    /// let semi = Rgba::from_oklch_str("oklch(0.7 0.15 200 / 0.5)").unwrap();
    /// assert!(semi.a >= 127 && semi.a <= 128);
    ///
    /// // With percentage alpha
    /// let semi2 = Rgba::from_oklch_str("oklch(0.7 0.15 200 / 50%)").unwrap();
    /// assert!(semi2.a >= 127 && semi2.a <= 128);
    ///
    /// // Invalid returns None
    /// assert!(Rgba::from_oklch_str("not-oklch").is_none());
    /// assert!(Rgba::from_oklch_str("oklch(invalid)").is_none());
    /// ```
    pub fn from_oklch_str(s: &str) -> Option<Self> {
        let s = s.trim().to_lowercase();

        // Must start with "oklch(" and end with ")"
        if !s.starts_with("oklch(") || !s.ends_with(')') {
            return None;
        }

        // Extract content between parentheses
        let content = &s[6..s.len() - 1];

        // Split by whitespace and "/" (for alpha separator)
        let parts: Vec<&str> = content
            .split(|c: char| c.is_whitespace() || c == '/')
            .filter(|s| !s.is_empty())
            .collect();

        if parts.len() < 3 {
            return None;
        }

        // Parse L (lightness)
        let l_str = parts[0];
        let l = if l_str.ends_with('%') {
            l_str[..l_str.len() - 1].parse::<f32>().ok()? / 100.0
        } else {
            l_str.parse::<f32>().ok()?
        };

        // Parse C (chroma)
        let c = parts[1].parse::<f32>().ok()?;

        // Parse H (hue)
        let h_str = parts[2];
        let h = if h_str.ends_with("rad") {
            h_str[..h_str.len() - 3].parse::<f32>().ok()? * (180.0 / std::f32::consts::PI)
        } else if h_str.ends_with("turn") {
            h_str[..h_str.len() - 4].parse::<f32>().ok()? * 360.0
        } else if h_str.ends_with("deg") {
            h_str[..h_str.len() - 3].parse::<f32>().ok()?
        } else {
            h_str.parse::<f32>().ok()?
        };

        // Parse A (alpha) if present
        let a = if parts.len() > 3 {
            let a_str = parts[3];
            if a_str.ends_with('%') {
                ((a_str[..a_str.len() - 1].parse::<f32>().ok()? / 100.0) * 255.0).round() as u8
            } else {
                (a_str.parse::<f32>().ok()? * 255.0).round() as u8
            }
        } else {
            255
        };

        // Validate ranges
        if l.is_nan() || c.is_nan() || h.is_nan() {
            return None;
        }

        Some(Self::oklch(l.clamp(0.0, 1.0), c.max(0.0), h, a))
    }

    /// Parse any supported color format.
    ///
    /// Supports:
    /// - hex (#RGB, #RRGGBB, #RRGGBBAA)
    /// - oklch() function
    /// - "transparent" keyword
    /// - "default" or "inherit" for terminal default
    ///
    /// # Examples
    ///
    /// ```
    /// use spark_tui::types::Rgba;
    ///
    /// // Hex colors
    /// let red = Rgba::parse("#ff0000").unwrap();
    /// assert_eq!(red, Rgba::rgb(255, 0, 0));
    ///
    /// // OKLCH colors
    /// let purple = Rgba::parse("oklch(0.75 0.15 300)").unwrap();
    ///
    /// // Special keywords
    /// let trans = Rgba::parse("transparent").unwrap();
    /// assert_eq!(trans, Rgba::TRANSPARENT);
    ///
    /// let def = Rgba::parse("default").unwrap();
    /// assert!(def.is_terminal_default());
    ///
    /// // Invalid returns None
    /// assert!(Rgba::parse("invalid-color").is_none());
    /// ```
    pub fn parse(input: &str) -> Option<Self> {
        let input = input.trim();

        // Handle empty string
        if input.is_empty() {
            return None;
        }

        let lower = input.to_lowercase();

        // Special keywords
        match lower.as_str() {
            "transparent" => return Some(Self::TRANSPARENT),
            "default" | "inherit" | "initial" | "currentcolor" => {
                return Some(Self::TERMINAL_DEFAULT)
            }
            _ => {}
        }

        // Hex colors
        if input.starts_with('#') || input.chars().all(|c| c.is_ascii_hexdigit()) {
            return Self::from_hex(input);
        }

        // OKLCH colors
        if lower.starts_with("oklch(") {
            return Self::from_oklch_str(input);
        }

        None
    }
}

// =============================================================================
// Dimension - Supports absolute and percentage values
// =============================================================================

/// A dimension value that can be absolute (pixels/cells) or percentage.
///
/// - `Auto` (0): Auto-size based on content
/// - `Cells(n)`: Absolute value in terminal cells
/// - `Percent(n)`: Percentage of parent (0-100)
///
/// # Examples
///
/// ```
/// use spark_tui::types::Dimension;
///
/// let width = Dimension::Cells(50);       // 50 characters
/// let height = Dimension::Percent(100.0); // Full parent height
/// let auto = Dimension::Auto;             // Content-determined
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dimension {
    /// Auto-size based on content (equivalent to 0 in TypeScript).
    Auto,
    /// Absolute size in terminal cells.
    Cells(u16),
    /// Percentage of parent size (0-100).
    Percent(f32),
}

impl Default for Dimension {
    fn default() -> Self {
        Self::Auto
    }
}

impl From<u16> for Dimension {
    fn from(value: u16) -> Self {
        if value == 0 {
            Self::Auto
        } else {
            Self::Cells(value)
        }
    }
}

impl From<i32> for Dimension {
    fn from(value: i32) -> Self {
        if value <= 0 {
            Self::Auto
        } else {
            Self::Cells(value as u16)
        }
    }
}

// =============================================================================
// Cell Attributes (bitflags)
// =============================================================================

bitflags::bitflags! {
    /// Text attributes as a bitfield for efficient storage and comparison.
    ///
    /// Combine with bitwise OR: `Attr::BOLD | Attr::ITALIC`
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Attr: u8 {
        const NONE = 0;
        const BOLD = 1 << 0;
        const DIM = 1 << 1;
        const ITALIC = 1 << 2;
        const UNDERLINE = 1 << 3;
        const BLINK = 1 << 4;
        const INVERSE = 1 << 5;
        const HIDDEN = 1 << 6;
        const STRIKETHROUGH = 1 << 7;
    }
}

// =============================================================================
// Cell - The atomic unit of terminal rendering
// =============================================================================

/// A single terminal cell.
///
/// This is what the renderer deals with. Nothing more complex.
/// The entire pipeline computes these, the renderer outputs them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    /// Unicode codepoint (32 for space).
    pub char: u32,
    /// Foreground color.
    pub fg: Rgba,
    /// Background color.
    pub bg: Rgba,
    /// Attribute flags (bold, italic, etc.).
    pub attrs: Attr,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            char: b' ' as u32,
            fg: Rgba::TERMINAL_DEFAULT,
            bg: Rgba::TERMINAL_DEFAULT,
            attrs: Attr::NONE,
        }
    }
}

// =============================================================================
// ClipRect - For overflow handling
// =============================================================================

/// A clipping rectangle for overflow handling.
///
/// Used by frameBufferDerived to handle overflow:hidden and scrolling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClipRect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl ClipRect {
    /// Create a new clip rect.
    pub const fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self { x, y, width, height }
    }

    /// Check if a point is inside this rect.
    #[inline]
    pub fn contains(&self, x: u16, y: u16) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }

    /// Compute intersection of two rects.
    pub fn intersect(&self, other: &ClipRect) -> Option<ClipRect> {
        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = (self.x + self.width).min(other.x + other.width);
        let y2 = (self.y + self.height).min(other.y + other.height);

        if x2 > x1 && y2 > y1 {
            Some(ClipRect {
                x: x1,
                y: y1,
                width: x2 - x1,
                height: y2 - y1,
            })
        } else {
            None
        }
    }
}

// =============================================================================
// Component Types - For parallel arrays
// =============================================================================

/// Component types for the parallel arrays pattern.
///
/// Each component at index i has componentType[i] set to one of these.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum ComponentType {
    #[default]
    None = 0,
    Box = 1,
    Text = 2,
    Input = 3,
    Select = 4,
    Progress = 5,
    Canvas = 6,
}

// =============================================================================
// Border Styles
// =============================================================================

/// Border style constants.
///
/// All 10 standard terminal border styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum BorderStyle {
    #[default]
    None = 0,
    /// ─ │ ┌ ┐ └ ┘
    Single = 1,
    /// ═ ║ ╔ ╗ ╚ ╝
    Double = 2,
    /// ─ │ ╭ ╮ ╰ ╯
    Rounded = 3,
    /// ━ ┃ ┏ ┓ ┗ ┛
    Bold = 4,
    /// ┄ ┆ ┌ ┐ └ ┘
    Dashed = 5,
    /// · · · · · ·
    Dotted = 6,
    /// - | + + + +
    Ascii = 7,
    /// █ █ █ █ █ █
    Block = 8,
    /// ═ │ ╒ ╕ ╘ ╛ (double horizontal, single vertical)
    DoubleHorz = 9,
    /// ─ ║ ╓ ╖ ╙ ╜ (single horizontal, double vertical)
    DoubleVert = 10,
}

impl BorderStyle {
    /// Get the border characters for this style.
    ///
    /// Returns: (horizontal, vertical, top_left, top_right, bottom_right, bottom_left)
    pub const fn chars(&self) -> (&'static str, &'static str, &'static str, &'static str, &'static str, &'static str) {
        match self {
            Self::None => (" ", " ", " ", " ", " ", " "),
            Self::Single => ("─", "│", "┌", "┐", "┘", "└"),
            Self::Double => ("═", "║", "╔", "╗", "╝", "╚"),
            Self::Rounded => ("─", "│", "╭", "╮", "╯", "╰"),
            Self::Bold => ("━", "┃", "┏", "┓", "┛", "┗"),
            Self::Dashed => ("┄", "┆", "┌", "┐", "┘", "└"),
            Self::Dotted => ("·", "·", "·", "·", "·", "·"),
            Self::Ascii => ("-", "|", "+", "+", "+", "+"),
            Self::Block => ("█", "█", "█", "█", "█", "█"),
            Self::DoubleHorz => ("═", "│", "╒", "╕", "╛", "╘"),
            Self::DoubleVert => ("─", "║", "╓", "╖", "╜", "╙"),
        }
    }
}

// =============================================================================
// Flex Enums - For layout
// =============================================================================

/// Flex direction for container layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum FlexDirection {
    #[default]
    Column = 0,
    Row = 1,
    ColumnReverse = 2,
    RowReverse = 3,
}

impl FlexDirection {
    /// Check if this is a row direction (Row or RowReverse).
    pub const fn is_row(&self) -> bool {
        matches!(self, Self::Row | Self::RowReverse)
    }

    /// Check if this is a reverse direction (ColumnReverse or RowReverse).
    pub const fn is_reverse(&self) -> bool {
        matches!(self, Self::ColumnReverse | Self::RowReverse)
    }
}

impl From<u8> for FlexDirection {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Column,
            1 => Self::Row,
            2 => Self::ColumnReverse,
            3 => Self::RowReverse,
            _ => Self::Column,
        }
    }
}

/// Flex wrap behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum FlexWrap {
    #[default]
    NoWrap = 0,
    Wrap = 1,
    WrapReverse = 2,
}

impl From<u8> for FlexWrap {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::NoWrap,
            1 => Self::Wrap,
            2 => Self::WrapReverse,
            _ => Self::NoWrap,
        }
    }
}

/// Justify content (main axis alignment).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum JustifyContent {
    #[default]
    FlexStart = 0,
    Center = 1,
    FlexEnd = 2,
    SpaceBetween = 3,
    SpaceAround = 4,
    SpaceEvenly = 5,
}

impl From<u8> for JustifyContent {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::FlexStart,
            1 => Self::Center,
            2 => Self::FlexEnd,
            3 => Self::SpaceBetween,
            4 => Self::SpaceAround,
            5 => Self::SpaceEvenly,
            _ => Self::FlexStart,
        }
    }
}

/// Align items (cross axis alignment).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum AlignItems {
    #[default]
    Stretch = 0,
    FlexStart = 1,
    Center = 2,
    FlexEnd = 3,
    Baseline = 4,
}

impl From<u8> for AlignItems {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Stretch,
            1 => Self::FlexStart,
            2 => Self::Center,
            3 => Self::FlexEnd,
            4 => Self::Baseline,
            _ => Self::Stretch,
        }
    }
}

/// Align self (item override for align items).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum AlignSelf {
    #[default]
    Auto = 0,
    Stretch = 1,
    FlexStart = 2,
    Center = 3,
    FlexEnd = 4,
    Baseline = 5,
}

impl AlignSelf {
    /// Convert to AlignItems, returning None if Auto.
    pub const fn to_align_items(&self) -> Option<AlignItems> {
        match self {
            Self::Auto => None,
            Self::Stretch => Some(AlignItems::Stretch),
            Self::FlexStart => Some(AlignItems::FlexStart),
            Self::Center => Some(AlignItems::Center),
            Self::FlexEnd => Some(AlignItems::FlexEnd),
            Self::Baseline => Some(AlignItems::Baseline),
        }
    }
}

impl From<u8> for AlignSelf {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Auto,
            1 => Self::Stretch,
            2 => Self::FlexStart,
            3 => Self::Center,
            4 => Self::FlexEnd,
            5 => Self::Baseline,
            _ => Self::Auto,
        }
    }
}

/// Align content (multi-line cross axis).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum AlignContent {
    #[default]
    Stretch = 0,
    FlexStart = 1,
    Center = 2,
    FlexEnd = 3,
    SpaceBetween = 4,
    SpaceAround = 5,
}

impl From<u8> for AlignContent {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Stretch,
            1 => Self::FlexStart,
            2 => Self::Center,
            3 => Self::FlexEnd,
            4 => Self::SpaceBetween,
            5 => Self::SpaceAround,
            _ => Self::Stretch,
        }
    }
}

/// Overflow behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Overflow {
    #[default]
    Visible = 0,
    Hidden = 1,
    Scroll = 2,
    Auto = 3,
}

impl From<u8> for Overflow {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Visible,
            1 => Self::Hidden,
            2 => Self::Scroll,
            3 => Self::Auto,
            _ => Self::Visible,
        }
    }
}

/// Position type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Position {
    #[default]
    Relative = 0,
    Absolute = 1,
}

/// Text alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum TextAlign {
    #[default]
    Left = 0,
    Center = 1,
    Right = 2,
}

/// Text wrap mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum TextWrap {
    NoWrap = 0,
    #[default]
    Wrap = 1,
    Truncate = 2,
}

// =============================================================================
// Render Mode
// =============================================================================

/// Rendering mode for the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderMode {
    /// Alternate screen buffer, full terminal control.
    #[default]
    Fullscreen,
    /// Renders inline, updates in place.
    Inline,
    /// Active content at bottom, history via renderToHistory().
    Append,
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // Rgba::from_rgb_int tests
    // =========================================================================

    #[test]
    fn test_rgba_from_rgb_int_basic() {
        let red = Rgba::from_rgb_int(0xff0000);
        assert_eq!(red, Rgba::rgb(255, 0, 0));

        let green = Rgba::from_rgb_int(0x00ff00);
        assert_eq!(green, Rgba::rgb(0, 255, 0));

        let blue = Rgba::from_rgb_int(0x0000ff);
        assert_eq!(blue, Rgba::rgb(0, 0, 255));

        let black = Rgba::from_rgb_int(0x000000);
        assert_eq!(black, Rgba::rgb(0, 0, 0));

        let white = Rgba::from_rgb_int(0xffffff);
        assert_eq!(white, Rgba::rgb(255, 255, 255));
    }

    #[test]
    fn test_rgba_from_rgb_int_theme_colors() {
        // Dracula theme colors
        let dracula_bg = Rgba::from_rgb_int(0x282a36);
        assert_eq!(dracula_bg, Rgba::rgb(40, 42, 54));

        let dracula_fg = Rgba::from_rgb_int(0xf8f8f2);
        assert_eq!(dracula_fg, Rgba::rgb(248, 248, 242));

        let dracula_comment = Rgba::from_rgb_int(0x6272a4);
        assert_eq!(dracula_comment, Rgba::rgb(98, 114, 164));
    }

    // =========================================================================
    // Rgba::from_hex tests
    // =========================================================================

    #[test]
    fn test_rgba_from_hex_rrggbb() {
        let red = Rgba::from_hex("#ff0000").unwrap();
        assert_eq!(red, Rgba::rgb(255, 0, 0));

        let green = Rgba::from_hex("#00ff00").unwrap();
        assert_eq!(green, Rgba::rgb(0, 255, 0));

        let blue = Rgba::from_hex("#0000ff").unwrap();
        assert_eq!(blue, Rgba::rgb(0, 0, 255));
    }

    #[test]
    fn test_rgba_from_hex_rgb_shorthand() {
        let white = Rgba::from_hex("#fff").unwrap();
        assert_eq!(white, Rgba::rgb(255, 255, 255));

        let black = Rgba::from_hex("#000").unwrap();
        assert_eq!(black, Rgba::rgb(0, 0, 0));

        let red = Rgba::from_hex("#f00").unwrap();
        assert_eq!(red, Rgba::rgb(255, 0, 0));

        // #abc expands to #aabbcc
        let abc = Rgba::from_hex("#abc").unwrap();
        assert_eq!(abc, Rgba::rgb(0xaa, 0xbb, 0xcc));
    }

    #[test]
    fn test_rgba_from_hex_with_alpha() {
        let semi_transparent_red = Rgba::from_hex("#ff000080").unwrap();
        assert_eq!(semi_transparent_red, Rgba::new(255, 0, 0, 128));

        let fully_transparent = Rgba::from_hex("#00000000").unwrap();
        assert_eq!(fully_transparent, Rgba::new(0, 0, 0, 0));

        let fully_opaque = Rgba::from_hex("#ff0000ff").unwrap();
        assert_eq!(fully_opaque, Rgba::new(255, 0, 0, 255));
    }

    #[test]
    fn test_rgba_from_hex_without_hash() {
        let red = Rgba::from_hex("ff0000").unwrap();
        assert_eq!(red, Rgba::rgb(255, 0, 0));

        let short = Rgba::from_hex("abc").unwrap();
        assert_eq!(short, Rgba::rgb(0xaa, 0xbb, 0xcc));
    }

    #[test]
    fn test_rgba_from_hex_case_insensitive() {
        let upper = Rgba::from_hex("#AABBCC").unwrap();
        let lower = Rgba::from_hex("#aabbcc").unwrap();
        let mixed = Rgba::from_hex("#AaBbCc").unwrap();
        assert_eq!(upper, lower);
        assert_eq!(lower, mixed);
    }

    #[test]
    fn test_rgba_from_hex_whitespace() {
        let trimmed = Rgba::from_hex("  #ff0000  ").unwrap();
        assert_eq!(trimmed, Rgba::rgb(255, 0, 0));
    }

    #[test]
    fn test_rgba_from_hex_invalid() {
        // Invalid characters
        assert!(Rgba::from_hex("#gg0000").is_none());
        assert!(Rgba::from_hex("#xyz").is_none());

        // Wrong length
        assert!(Rgba::from_hex("#f").is_none());
        assert!(Rgba::from_hex("#ff").is_none());
        assert!(Rgba::from_hex("#ffff").is_none());
        assert!(Rgba::from_hex("#fffff").is_none());
        assert!(Rgba::from_hex("#fffffff").is_none());
        assert!(Rgba::from_hex("#fffffffff").is_none());

        // Empty string
        assert!(Rgba::from_hex("").is_none());
        assert!(Rgba::from_hex("#").is_none());
    }

    // =========================================================================
    // Rgba::from_oklch_str tests
    // =========================================================================

    #[test]
    fn test_rgba_from_oklch_str_basic() {
        // Purple from Dracula theme
        let purple = Rgba::from_oklch_str("oklch(0.75 0.15 300)").unwrap();
        // Check it's in the purple range (hue 300 = magenta/purple)
        assert!(purple.r > 100); // Has some red
        assert!(purple.g < 200); // Less green
        assert!(purple.b > 200); // High blue

        // Bright yellow
        let yellow = Rgba::from_oklch_str("oklch(0.9 0.15 100)").unwrap();
        assert!(yellow.r > 200); // High red
        assert!(yellow.g > 200); // High green
        assert!(yellow.b < 150); // Low blue
    }

    #[test]
    fn test_rgba_from_oklch_str_with_percentage_lightness() {
        let bright = Rgba::from_oklch_str("oklch(80% 0.2 180)").unwrap();
        // 80% = 0.8 lightness, should be fairly bright cyan (hue 180)
        assert!(bright.r < 150);
        assert!(bright.g > 150);
        assert!(bright.b > 150);
    }

    #[test]
    fn test_rgba_from_oklch_str_with_alpha() {
        let semi = Rgba::from_oklch_str("oklch(0.7 0.15 200 / 0.5)").unwrap();
        // 0.5 * 255 = 127.5, rounds to 127 or 128 depending on rounding
        assert!(semi.a >= 127 && semi.a <= 128);

        let percent_alpha = Rgba::from_oklch_str("oklch(0.7 0.15 200 / 50%)").unwrap();
        assert!(percent_alpha.a >= 127 && percent_alpha.a <= 128);

        let full_alpha = Rgba::from_oklch_str("oklch(0.7 0.15 200 / 1)").unwrap();
        assert_eq!(full_alpha.a, 255);
    }

    #[test]
    fn test_rgba_from_oklch_str_with_units() {
        // With deg unit (should work the same as without)
        let deg = Rgba::from_oklch_str("oklch(0.75 0.15 300deg)").unwrap();
        let no_unit = Rgba::from_oklch_str("oklch(0.75 0.15 300)").unwrap();
        assert_eq!(deg, no_unit);

        // With rad unit (180 degrees = PI radians)
        let rad = Rgba::from_oklch_str("oklch(0.75 0.1 3.14159rad)").unwrap();
        // ~180 degrees = cyan/teal
        assert!(rad.g > rad.r);
        assert!(rad.b > rad.r);

        // With turn unit (0.5 turn = 180 degrees)
        let turn = Rgba::from_oklch_str("oklch(0.75 0.1 0.5turn)").unwrap();
        // Should be similar to 180 degrees
        assert!(turn.g > turn.r);
    }

    #[test]
    fn test_rgba_from_oklch_str_edge_cases() {
        // Black (L=0)
        let black = Rgba::from_oklch_str("oklch(0 0 0)").unwrap();
        assert_eq!(black.r, 0);
        assert_eq!(black.g, 0);
        assert_eq!(black.b, 0);

        // White (L=1, C=0) - may be 254 or 255 due to gamma correction precision
        let white = Rgba::from_oklch_str("oklch(1 0 0)").unwrap();
        assert!(white.r >= 254);
        assert!(white.g >= 254);
        assert!(white.b >= 254);

        // Gray (L=0.5, C=0) - OKLCH mid-gray in perceptual space
        let gray = Rgba::from_oklch_str("oklch(0.5 0 0)").unwrap();
        // L=0.5 in OKLCH is perceptual mid-gray (roughly sRGB ~99)
        assert!(gray.r > 80 && gray.r < 120, "gray.r = {}", gray.r);
        // Should be neutral gray (R=G=B within floating point tolerance)
        assert!((gray.r - gray.g).abs() <= 1);
        assert!((gray.g - gray.b).abs() <= 1);
    }

    #[test]
    fn test_rgba_from_oklch_str_case_insensitive() {
        let lower = Rgba::from_oklch_str("oklch(0.75 0.15 300)").unwrap();
        let upper = Rgba::from_oklch_str("OKLCH(0.75 0.15 300)").unwrap();
        let mixed = Rgba::from_oklch_str("OkLcH(0.75 0.15 300)").unwrap();
        assert_eq!(lower, upper);
        assert_eq!(upper, mixed);
    }

    #[test]
    fn test_rgba_from_oklch_str_invalid() {
        // Not oklch
        assert!(Rgba::from_oklch_str("rgb(255, 0, 0)").is_none());
        assert!(Rgba::from_oklch_str("#ff0000").is_none());
        assert!(Rgba::from_oklch_str("oklch").is_none());

        // Missing parentheses
        assert!(Rgba::from_oklch_str("oklch 0.5 0.1 200").is_none());
        assert!(Rgba::from_oklch_str("oklch(0.5 0.1 200").is_none());

        // Too few values
        assert!(Rgba::from_oklch_str("oklch(0.5 0.1)").is_none());
        assert!(Rgba::from_oklch_str("oklch(0.5)").is_none());
        assert!(Rgba::from_oklch_str("oklch()").is_none());

        // Invalid numbers
        assert!(Rgba::from_oklch_str("oklch(abc 0.1 200)").is_none());
    }

    // =========================================================================
    // Rgba::parse tests
    // =========================================================================

    #[test]
    fn test_rgba_parse_hex() {
        let hex6 = Rgba::parse("#ff0000").unwrap();
        assert_eq!(hex6, Rgba::rgb(255, 0, 0));

        let hex3 = Rgba::parse("#f00").unwrap();
        assert_eq!(hex3, Rgba::rgb(255, 0, 0));

        let hex8 = Rgba::parse("#ff000080").unwrap();
        assert_eq!(hex8, Rgba::new(255, 0, 0, 128));
    }

    #[test]
    fn test_rgba_parse_oklch() {
        let oklch = Rgba::parse("oklch(0.75 0.15 300)").unwrap();
        // Should be purple
        assert!(oklch.b > 200);
    }

    #[test]
    fn test_rgba_parse_special_keywords() {
        let trans = Rgba::parse("transparent").unwrap();
        assert_eq!(trans, Rgba::TRANSPARENT);

        let def = Rgba::parse("default").unwrap();
        assert!(def.is_terminal_default());

        let inherit = Rgba::parse("inherit").unwrap();
        assert!(inherit.is_terminal_default());

        let initial = Rgba::parse("initial").unwrap();
        assert!(initial.is_terminal_default());

        let current = Rgba::parse("currentColor").unwrap();
        assert!(current.is_terminal_default());
    }

    #[test]
    fn test_rgba_parse_case_insensitive() {
        let upper = Rgba::parse("TRANSPARENT").unwrap();
        let lower = Rgba::parse("transparent").unwrap();
        let mixed = Rgba::parse("Transparent").unwrap();
        assert_eq!(upper, lower);
        assert_eq!(lower, mixed);

        let default_upper = Rgba::parse("DEFAULT").unwrap();
        assert!(default_upper.is_terminal_default());
    }

    #[test]
    fn test_rgba_parse_whitespace() {
        let trimmed = Rgba::parse("  #ff0000  ").unwrap();
        assert_eq!(trimmed, Rgba::rgb(255, 0, 0));

        let oklch_trimmed = Rgba::parse("  oklch(0.75 0.15 300)  ").unwrap();
        assert!(oklch_trimmed.b > 200);
    }

    #[test]
    fn test_rgba_parse_invalid() {
        assert!(Rgba::parse("").is_none());
        assert!(Rgba::parse("invalid").is_none());
        assert!(Rgba::parse("rgb(255, 0, 0)").is_none()); // rgb() not supported
        assert!(Rgba::parse("hsl(0, 100%, 50%)").is_none()); // hsl() not supported
    }

    // =========================================================================
    // Rgba OKLCH round-trip tests
    // =========================================================================

    #[test]
    fn test_rgba_oklch_round_trip() {
        // Create a color with OKLCH
        let original = Rgba::oklch(0.75, 0.15, 300.0, 255);

        // Convert to OKLCH
        let (l, c, h) = original.to_oklch().unwrap();

        // Create new color from those values
        let recreated = Rgba::oklch(l, c, h, 255);

        // Should be very close (within 1 due to float precision)
        assert!((original.r - recreated.r).abs() <= 1);
        assert!((original.g - recreated.g).abs() <= 1);
        assert!((original.b - recreated.b).abs() <= 1);
    }

    #[test]
    fn test_rgba_to_oklch_special_colors() {
        // Terminal default can't be converted
        assert!(Rgba::TERMINAL_DEFAULT.to_oklch().is_none());

        // ANSI colors can't be converted
        assert!(Rgba::ansi(12).to_oklch().is_none());
    }
}
