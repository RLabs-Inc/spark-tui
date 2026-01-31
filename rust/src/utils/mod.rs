//! Utility types for the renderer and framebuffer.
//!
//! These are rendering infrastructure types - Rgba, Cell, Attr, ClipRect.
//! The SharedBuffer is the source of truth for colors - these exist for
//! unpacking and outputting to the terminal.

// =============================================================================
// Rgba - Color representation for rendering
// =============================================================================

/// RGBA color for rendering.
///
/// TS resolves all colors (themes, OKLCH, etc.) before writing to SharedBuffer.
/// Rust just reads the packed u32, unpacks, detects markers, and outputs ANSI.
///
/// Special markers:
/// - r=-1 (or 255 after unpack): Terminal default
/// - r=-2 (or 254 after unpack): ANSI palette, index in g
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

    /// Create an ANSI palette color (0-255).
    /// Uses marker r=-2 to signal ANSI mode, index stored in g.
    pub const fn ansi(index: u8) -> Self {
        Self {
            r: -2,
            g: index as i16,
            b: 0,
            a: 255,
        }
    }

    // Standard colors (for convenience)
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    pub const WHITE: Self = Self::rgb(255, 255, 255);
    pub const GRAY: Self = Self::rgb(128, 128, 128);
    pub const RED: Self = Self::rgb(255, 0, 0);
    pub const GREEN: Self = Self::rgb(0, 255, 0);
    pub const BLUE: Self = Self::rgb(0, 0, 255);
    pub const YELLOW: Self = Self::rgb(255, 255, 0);
    pub const CYAN: Self = Self::rgb(0, 255, 255);
    pub const MAGENTA: Self = Self::rgb(255, 0, 255);

    /// Unpack from u32 (ARGB format from SharedBuffer).
    #[inline]
    pub const fn from_u32(packed: u32) -> Self {
        Self {
            a: ((packed >> 24) & 0xFF) as i16,
            r: ((packed >> 16) & 0xFF) as i16,
            g: ((packed >> 8) & 0xFF) as i16,
            b: (packed & 0xFF) as i16,
        }
    }

    /// Check if this is the terminal default color.
    /// Handles both semantic (-1) and packed (255) values.
    #[inline]
    pub const fn is_terminal_default(&self) -> bool {
        self.r == -1 || (self.r == 255 && self.g == 255 && self.b == 255 && self.a == 255)
    }

    /// Check if this is an ANSI palette color.
    /// Handles both semantic (-2) and packed (254) values.
    #[inline]
    pub const fn is_ansi(&self) -> bool {
        self.r == -2 || self.r == 254
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
    /// Used by framebuffer when compositing layers.
    #[inline]
    pub fn blend(src: Self, dst: Self) -> Self {
        // Fast path: fully opaque source or special colors
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

        let out_a = sa + (da as i32 * inv_sa) / 255;

        if out_a == 0 {
            return Self::TRANSPARENT;
        }

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

    /// Dim the color by a factor (0.0 = black, 1.0 = unchanged).
    /// Used for disabled states.
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
/// TODO: When rewriting render_tree.rs/buffer.rs, change x/y to i32
/// to support negative scroll positions properly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClipRect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl ClipRect {
    pub const fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self { x, y, width, height }
    }

    /// Check if a point is inside this rect.
    #[inline]
    pub fn contains(&self, px: u16, py: u16) -> bool {
        px >= self.x && px < self.x + self.width && py >= self.y && py < self.y + self.height
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
// Dimension - For layout values
// =============================================================================

/// A dimension value that can be absolute (cells) or percentage.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dimension {
    Auto,
    Cells(u16),
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
