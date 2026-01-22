//! Theme presets for spark-tui.
//!
//! Contains 13 built-in themes matching the TypeScript implementation:
//! - terminal (default - uses ANSI colors)
//! - dracula
//! - nord
//! - monokai
//! - solarized
//! - catppuccin
//! - gruvbox
//! - tokyoNight
//! - oneDark
//! - rosePine
//! - kanagawa
//! - everforest
//! - nightOwl

use super::{Theme, ThemeColor};
use crate::types::Rgba;

// =============================================================================
// Terminal Theme (Default)
// =============================================================================

/// Terminal theme - uses ANSI colors to respect user's terminal theme.
/// This is the default and should be used for most applications.
pub fn terminal() -> Theme {
    Theme {
        name: "terminal".to_string(),
        description: "Uses terminal default colors".to_string(),
        // Main palette - ANSI bright colors
        primary: ThemeColor::Ansi(12),   // bright blue
        secondary: ThemeColor::Ansi(13), // bright magenta
        tertiary: ThemeColor::Ansi(14),  // bright cyan
        accent: ThemeColor::Ansi(11),    // bright yellow
        // Semantic
        success: ThemeColor::Ansi(2), // green
        warning: ThemeColor::Ansi(3), // yellow
        error: ThemeColor::Ansi(1),   // red
        info: ThemeColor::Ansi(6),    // cyan
        // Text
        text: ThemeColor::Default,
        text_muted: ThemeColor::Ansi(8),
        text_dim: ThemeColor::Ansi(8),
        text_disabled: ThemeColor::Ansi(8),
        text_bright: ThemeColor::Ansi(15),
        // Background
        background: ThemeColor::Default,
        background_muted: ThemeColor::Default,
        surface: ThemeColor::Default,
        overlay: ThemeColor::Default,
        // Border
        border: ThemeColor::Ansi(7),
        border_focus: ThemeColor::Ansi(12),
    }
}

// =============================================================================
// Dracula Theme
// =============================================================================

/// Dracula - dark theme with vivid colors.
/// Uses OKLCH for perceptually uniform colors.
pub fn dracula() -> Theme {
    Theme {
        name: "dracula".to_string(),
        description: "Dracula dark theme".to_string(),
        // Main palette - OKLCH strings
        primary: ThemeColor::Str("oklch(0.75 0.15 300)".to_string()),   // purple
        secondary: ThemeColor::Str("oklch(0.75 0.2 340)".to_string()),  // pink
        tertiary: ThemeColor::Str("oklch(0.85 0.12 200)".to_string()),  // cyan
        accent: ThemeColor::Str("oklch(0.9 0.15 100)".to_string()),     // yellow
        // Semantic
        success: ThemeColor::Str("oklch(0.8 0.2 140)".to_string()),  // green
        warning: ThemeColor::Str("oklch(0.9 0.15 100)".to_string()), // yellow
        error: ThemeColor::Str("oklch(0.7 0.25 25)".to_string()),    // red
        info: ThemeColor::Str("oklch(0.85 0.12 200)".to_string()),   // cyan
        // Text - RGB integers
        text: ThemeColor::Rgb(Rgba::from_rgb_int(0xf8f8f2)),
        text_muted: ThemeColor::Rgb(Rgba::from_rgb_int(0x6272a4)),
        text_dim: ThemeColor::Rgb(Rgba::from_rgb_int(0x6272a4)),
        text_disabled: ThemeColor::Rgb(Rgba::from_rgb_int(0x44475a)),
        text_bright: ThemeColor::Rgb(Rgba::from_rgb_int(0xffffff)),
        // Background
        background: ThemeColor::Rgb(Rgba::from_rgb_int(0x282a36)),
        background_muted: ThemeColor::Rgb(Rgba::from_rgb_int(0x343746)),
        surface: ThemeColor::Rgb(Rgba::from_rgb_int(0x44475a)),
        overlay: ThemeColor::Rgb(Rgba::from_rgb_int(0x21222c)),
        // Border
        border: ThemeColor::Rgb(Rgba::from_rgb_int(0x6272a4)),
        border_focus: ThemeColor::Rgb(Rgba::from_rgb_int(0xbd93f9)),
    }
}

// =============================================================================
// Nord Theme
// =============================================================================

/// Nord - arctic, bluish colors.
pub fn nord() -> Theme {
    Theme {
        name: "nord".to_string(),
        description: "Nord arctic theme".to_string(),
        // Main palette - OKLCH strings
        primary: ThemeColor::Str("oklch(0.8 0.08 210)".to_string()),  // frost cyan
        secondary: ThemeColor::Str("oklch(0.7 0.08 230)".to_string()), // frost blue
        tertiary: ThemeColor::Str("oklch(0.6 0.1 250)".to_string()),  // frost dark blue
        accent: ThemeColor::Str("oklch(0.7 0.12 50)".to_string()),    // aurora orange
        // Semantic
        success: ThemeColor::Str("oklch(0.75 0.1 130)".to_string()),  // aurora green
        warning: ThemeColor::Str("oklch(0.85 0.1 90)".to_string()),   // aurora yellow
        error: ThemeColor::Str("oklch(0.65 0.15 20)".to_string()),    // aurora red
        info: ThemeColor::Str("oklch(0.8 0.08 210)".to_string()),     // frost cyan
        // Text
        text: ThemeColor::Rgb(Rgba::from_rgb_int(0xd8dee9)),
        text_muted: ThemeColor::Rgb(Rgba::from_rgb_int(0x4c566a)),
        text_dim: ThemeColor::Rgb(Rgba::from_rgb_int(0x4c566a)),
        text_disabled: ThemeColor::Rgb(Rgba::from_rgb_int(0x3b4252)),
        text_bright: ThemeColor::Rgb(Rgba::from_rgb_int(0xeceff4)),
        // Background
        background: ThemeColor::Rgb(Rgba::from_rgb_int(0x2e3440)),
        background_muted: ThemeColor::Rgb(Rgba::from_rgb_int(0x3b4252)),
        surface: ThemeColor::Rgb(Rgba::from_rgb_int(0x434c5e)),
        overlay: ThemeColor::Rgb(Rgba::from_rgb_int(0x2e3440)),
        // Border
        border: ThemeColor::Rgb(Rgba::from_rgb_int(0x4c566a)),
        border_focus: ThemeColor::Rgb(Rgba::from_rgb_int(0x88c0d0)),
    }
}

// =============================================================================
// Monokai Theme
// =============================================================================

/// Monokai - vibrant syntax-highlighting inspired theme.
pub fn monokai() -> Theme {
    Theme {
        name: "monokai".to_string(),
        description: "Monokai vibrant theme".to_string(),
        // Main palette - OKLCH strings
        primary: ThemeColor::Str("oklch(0.65 0.25 350)".to_string()),   // pink
        secondary: ThemeColor::Str("oklch(0.85 0.25 125)".to_string()), // green
        tertiary: ThemeColor::Str("oklch(0.7 0.2 300)".to_string()),    // purple
        accent: ThemeColor::Str("oklch(0.75 0.18 60)".to_string()),     // orange
        // Semantic
        success: ThemeColor::Str("oklch(0.85 0.25 125)".to_string()), // green
        warning: ThemeColor::Str("oklch(0.75 0.18 60)".to_string()),  // orange
        error: ThemeColor::Str("oklch(0.65 0.25 350)".to_string()),   // pink
        info: ThemeColor::Str("oklch(0.8 0.12 220)".to_string()),     // blue
        // Text
        text: ThemeColor::Rgb(Rgba::from_rgb_int(0xf8f8f2)),
        text_muted: ThemeColor::Rgb(Rgba::from_rgb_int(0x75715e)),
        text_dim: ThemeColor::Rgb(Rgba::from_rgb_int(0x75715e)),
        text_disabled: ThemeColor::Rgb(Rgba::from_rgb_int(0x49483e)),
        text_bright: ThemeColor::Rgb(Rgba::from_rgb_int(0xffffff)),
        // Background
        background: ThemeColor::Rgb(Rgba::from_rgb_int(0x272822)),
        background_muted: ThemeColor::Rgb(Rgba::from_rgb_int(0x3e3d32)),
        surface: ThemeColor::Rgb(Rgba::from_rgb_int(0x49483e)),
        overlay: ThemeColor::Rgb(Rgba::from_rgb_int(0x1e1f1c)),
        // Border
        border: ThemeColor::Rgb(Rgba::from_rgb_int(0x75715e)),
        border_focus: ThemeColor::Rgb(Rgba::from_rgb_int(0xf92672)),
    }
}

// =============================================================================
// Solarized Dark Theme
// =============================================================================

/// Solarized Dark - precision color scheme.
pub fn solarized() -> Theme {
    Theme {
        name: "solarized".to_string(),
        description: "Solarized Dark theme".to_string(),
        // Main palette - RGB integers
        primary: ThemeColor::Rgb(Rgba::from_rgb_int(0x268bd2)),   // blue
        secondary: ThemeColor::Rgb(Rgba::from_rgb_int(0x2aa198)), // cyan
        tertiary: ThemeColor::Rgb(Rgba::from_rgb_int(0x859900)),  // green
        accent: ThemeColor::Rgb(Rgba::from_rgb_int(0xcb4b16)),    // orange
        // Semantic
        success: ThemeColor::Rgb(Rgba::from_rgb_int(0x859900)), // green
        warning: ThemeColor::Rgb(Rgba::from_rgb_int(0xb58900)), // yellow
        error: ThemeColor::Rgb(Rgba::from_rgb_int(0xdc322f)),   // red
        info: ThemeColor::Rgb(Rgba::from_rgb_int(0x268bd2)),    // blue
        // Text
        text: ThemeColor::Rgb(Rgba::from_rgb_int(0x839496)),        // base0
        text_muted: ThemeColor::Rgb(Rgba::from_rgb_int(0x586e75)),  // base01
        text_dim: ThemeColor::Rgb(Rgba::from_rgb_int(0x586e75)),
        text_disabled: ThemeColor::Rgb(Rgba::from_rgb_int(0x073642)), // base02
        text_bright: ThemeColor::Rgb(Rgba::from_rgb_int(0x93a1a1)),   // base1
        // Background
        background: ThemeColor::Rgb(Rgba::from_rgb_int(0x002b36)),       // base03
        background_muted: ThemeColor::Rgb(Rgba::from_rgb_int(0x073642)), // base02
        surface: ThemeColor::Rgb(Rgba::from_rgb_int(0x073642)),
        overlay: ThemeColor::Rgb(Rgba::from_rgb_int(0x002b36)),
        // Border
        border: ThemeColor::Rgb(Rgba::from_rgb_int(0x586e75)),
        border_focus: ThemeColor::Rgb(Rgba::from_rgb_int(0x268bd2)),
    }
}

// =============================================================================
// Catppuccin Mocha Theme
// =============================================================================

/// Catppuccin Mocha - soothing pastel theme (most popular variant).
pub fn catppuccin() -> Theme {
    Theme {
        name: "catppuccin".to_string(),
        description: "Catppuccin Mocha theme".to_string(),
        // Main palette
        primary: ThemeColor::Rgb(Rgba::from_rgb_int(0x89b4fa)),   // blue
        secondary: ThemeColor::Rgb(Rgba::from_rgb_int(0xcba6f7)), // mauve
        tertiary: ThemeColor::Rgb(Rgba::from_rgb_int(0x94e2d5)),  // teal
        accent: ThemeColor::Rgb(Rgba::from_rgb_int(0xf9e2af)),    // yellow
        // Semantic
        success: ThemeColor::Rgb(Rgba::from_rgb_int(0xa6e3a1)), // green
        warning: ThemeColor::Rgb(Rgba::from_rgb_int(0xf9e2af)), // yellow
        error: ThemeColor::Rgb(Rgba::from_rgb_int(0xf38ba8)),   // red
        info: ThemeColor::Rgb(Rgba::from_rgb_int(0x89dceb)),    // sky
        // Text
        text: ThemeColor::Rgb(Rgba::from_rgb_int(0xcdd6f4)),       // text
        text_muted: ThemeColor::Rgb(Rgba::from_rgb_int(0x6c7086)), // overlay0
        text_dim: ThemeColor::Rgb(Rgba::from_rgb_int(0x585b70)),   // surface2
        text_disabled: ThemeColor::Rgb(Rgba::from_rgb_int(0x45475a)), // surface0
        text_bright: ThemeColor::Rgb(Rgba::from_rgb_int(0xffffff)),
        // Background
        background: ThemeColor::Rgb(Rgba::from_rgb_int(0x1e1e2e)),       // base
        background_muted: ThemeColor::Rgb(Rgba::from_rgb_int(0x313244)), // surface0
        surface: ThemeColor::Rgb(Rgba::from_rgb_int(0x45475a)),          // surface1
        overlay: ThemeColor::Rgb(Rgba::from_rgb_int(0x181825)),          // mantle
        // Border
        border: ThemeColor::Rgb(Rgba::from_rgb_int(0x6c7086)),      // overlay0
        border_focus: ThemeColor::Rgb(Rgba::from_rgb_int(0x89b4fa)), // blue
    }
}

// =============================================================================
// Gruvbox Dark Theme
// =============================================================================

/// Gruvbox Dark - retro groove color scheme.
pub fn gruvbox() -> Theme {
    Theme {
        name: "gruvbox".to_string(),
        description: "Gruvbox Dark theme".to_string(),
        // Main palette
        primary: ThemeColor::Rgb(Rgba::from_rgb_int(0x458588)),   // blue
        secondary: ThemeColor::Rgb(Rgba::from_rgb_int(0xb16286)), // purple
        tertiary: ThemeColor::Rgb(Rgba::from_rgb_int(0x689d6a)),  // aqua
        accent: ThemeColor::Rgb(Rgba::from_rgb_int(0xd79921)),    // yellow
        // Semantic
        success: ThemeColor::Rgb(Rgba::from_rgb_int(0x98971a)), // green
        warning: ThemeColor::Rgb(Rgba::from_rgb_int(0xd79921)), // yellow
        error: ThemeColor::Rgb(Rgba::from_rgb_int(0xcc241d)),   // red
        info: ThemeColor::Rgb(Rgba::from_rgb_int(0x458588)),    // blue
        // Text
        text: ThemeColor::Rgb(Rgba::from_rgb_int(0xebdbb2)),       // fg
        text_muted: ThemeColor::Rgb(Rgba::from_rgb_int(0xa89984)), // gray
        text_dim: ThemeColor::Rgb(Rgba::from_rgb_int(0x928374)),   // gray
        text_disabled: ThemeColor::Rgb(Rgba::from_rgb_int(0x665c54)), // bg3
        text_bright: ThemeColor::Rgb(Rgba::from_rgb_int(0xfbf1c7)),   // fg0
        // Background
        background: ThemeColor::Rgb(Rgba::from_rgb_int(0x282828)),       // bg
        background_muted: ThemeColor::Rgb(Rgba::from_rgb_int(0x3c3836)), // bg1
        surface: ThemeColor::Rgb(Rgba::from_rgb_int(0x504945)),          // bg2
        overlay: ThemeColor::Rgb(Rgba::from_rgb_int(0x1d2021)),          // bg0_h
        // Border
        border: ThemeColor::Rgb(Rgba::from_rgb_int(0x665c54)),      // bg3
        border_focus: ThemeColor::Rgb(Rgba::from_rgb_int(0xfe8019)), // orange
    }
}

// =============================================================================
// Tokyo Night Theme
// =============================================================================

/// Tokyo Night - clean, dark theme inspired by Tokyo city lights.
pub fn tokyo_night() -> Theme {
    Theme {
        name: "tokyoNight".to_string(),
        description: "Tokyo Night theme".to_string(),
        // Main palette
        primary: ThemeColor::Rgb(Rgba::from_rgb_int(0x7aa2f7)),   // blue
        secondary: ThemeColor::Rgb(Rgba::from_rgb_int(0xbb9af7)), // purple
        tertiary: ThemeColor::Rgb(Rgba::from_rgb_int(0x7dcfff)),  // cyan
        accent: ThemeColor::Rgb(Rgba::from_rgb_int(0xe0af68)),    // yellow
        // Semantic
        success: ThemeColor::Rgb(Rgba::from_rgb_int(0x9ece6a)), // green
        warning: ThemeColor::Rgb(Rgba::from_rgb_int(0xe0af68)), // yellow
        error: ThemeColor::Rgb(Rgba::from_rgb_int(0xf7768e)),   // red
        info: ThemeColor::Rgb(Rgba::from_rgb_int(0x7dcfff)),    // cyan
        // Text
        text: ThemeColor::Rgb(Rgba::from_rgb_int(0xa9b1d6)),       // fg
        text_muted: ThemeColor::Rgb(Rgba::from_rgb_int(0x565f89)), // comment
        text_dim: ThemeColor::Rgb(Rgba::from_rgb_int(0x414868)),   // dark3
        text_disabled: ThemeColor::Rgb(Rgba::from_rgb_int(0x3b4261)), // dark2
        text_bright: ThemeColor::Rgb(Rgba::from_rgb_int(0xc0caf5)),   // fg_bright
        // Background
        background: ThemeColor::Rgb(Rgba::from_rgb_int(0x1a1b26)),       // bg
        background_muted: ThemeColor::Rgb(Rgba::from_rgb_int(0x24283b)), // bg_highlight
        surface: ThemeColor::Rgb(Rgba::from_rgb_int(0x414868)),          // dark3
        overlay: ThemeColor::Rgb(Rgba::from_rgb_int(0x16161e)),          // bg_dark
        // Border
        border: ThemeColor::Rgb(Rgba::from_rgb_int(0x414868)),      // dark3
        border_focus: ThemeColor::Rgb(Rgba::from_rgb_int(0x7aa2f7)), // blue
    }
}

// =============================================================================
// One Dark Theme
// =============================================================================

/// One Dark - Atom's iconic dark theme.
pub fn one_dark() -> Theme {
    Theme {
        name: "oneDark".to_string(),
        description: "One Dark (Atom) theme".to_string(),
        // Main palette
        primary: ThemeColor::Rgb(Rgba::from_rgb_int(0x61afef)),   // blue
        secondary: ThemeColor::Rgb(Rgba::from_rgb_int(0xc678dd)), // purple
        tertiary: ThemeColor::Rgb(Rgba::from_rgb_int(0x56b6c2)),  // cyan
        accent: ThemeColor::Rgb(Rgba::from_rgb_int(0xe5c07b)),    // yellow
        // Semantic
        success: ThemeColor::Rgb(Rgba::from_rgb_int(0x98c379)), // green
        warning: ThemeColor::Rgb(Rgba::from_rgb_int(0xe5c07b)), // yellow
        error: ThemeColor::Rgb(Rgba::from_rgb_int(0xe06c75)),   // red
        info: ThemeColor::Rgb(Rgba::from_rgb_int(0x56b6c2)),    // cyan
        // Text
        text: ThemeColor::Rgb(Rgba::from_rgb_int(0xabb2bf)),       // fg
        text_muted: ThemeColor::Rgb(Rgba::from_rgb_int(0x5c6370)), // comment
        text_dim: ThemeColor::Rgb(Rgba::from_rgb_int(0x4b5263)),   // gutter
        text_disabled: ThemeColor::Rgb(Rgba::from_rgb_int(0x3e4451)), // guide
        text_bright: ThemeColor::Rgb(Rgba::from_rgb_int(0xffffff)),
        // Background
        background: ThemeColor::Rgb(Rgba::from_rgb_int(0x282c34)),       // bg
        background_muted: ThemeColor::Rgb(Rgba::from_rgb_int(0x21252b)), // bg_dark
        surface: ThemeColor::Rgb(Rgba::from_rgb_int(0x3e4451)),          // guide
        overlay: ThemeColor::Rgb(Rgba::from_rgb_int(0x1e2127)),          // bg_darker
        // Border
        border: ThemeColor::Rgb(Rgba::from_rgb_int(0x3e4451)),      // guide
        border_focus: ThemeColor::Rgb(Rgba::from_rgb_int(0x61afef)), // blue
    }
}

// =============================================================================
// Rose Pine Theme
// =============================================================================

/// Rose Pine - all natural pine, faux fur and a bit of soho vibes.
pub fn rose_pine() -> Theme {
    Theme {
        name: "rosePine".to_string(),
        description: "Rose Pine theme".to_string(),
        // Main palette
        primary: ThemeColor::Rgb(Rgba::from_rgb_int(0x9ccfd8)),   // foam
        secondary: ThemeColor::Rgb(Rgba::from_rgb_int(0xc4a7e7)), // iris
        tertiary: ThemeColor::Rgb(Rgba::from_rgb_int(0x31748f)),  // pine
        accent: ThemeColor::Rgb(Rgba::from_rgb_int(0xf6c177)),    // gold
        // Semantic
        success: ThemeColor::Rgb(Rgba::from_rgb_int(0x31748f)), // pine
        warning: ThemeColor::Rgb(Rgba::from_rgb_int(0xf6c177)), // gold
        error: ThemeColor::Rgb(Rgba::from_rgb_int(0xeb6f92)),   // love
        info: ThemeColor::Rgb(Rgba::from_rgb_int(0x9ccfd8)),    // foam
        // Text
        text: ThemeColor::Rgb(Rgba::from_rgb_int(0xe0def4)),       // text
        text_muted: ThemeColor::Rgb(Rgba::from_rgb_int(0x908caa)), // subtle
        text_dim: ThemeColor::Rgb(Rgba::from_rgb_int(0x6e6a86)),   // muted
        text_disabled: ThemeColor::Rgb(Rgba::from_rgb_int(0x524f67)), // highlight_med
        text_bright: ThemeColor::Rgb(Rgba::from_rgb_int(0xffffff)),
        // Background
        background: ThemeColor::Rgb(Rgba::from_rgb_int(0x191724)),       // base
        background_muted: ThemeColor::Rgb(Rgba::from_rgb_int(0x1f1d2e)), // surface
        surface: ThemeColor::Rgb(Rgba::from_rgb_int(0x26233a)),          // overlay
        overlay: ThemeColor::Rgb(Rgba::from_rgb_int(0x16141f)),          // nc
        // Border
        border: ThemeColor::Rgb(Rgba::from_rgb_int(0x524f67)),      // highlight_med
        border_focus: ThemeColor::Rgb(Rgba::from_rgb_int(0xebbcba)), // rose
    }
}

// =============================================================================
// Kanagawa Theme
// =============================================================================

/// Kanagawa - theme inspired by Katsushika Hokusai's famous wave painting.
pub fn kanagawa() -> Theme {
    Theme {
        name: "kanagawa".to_string(),
        description: "Kanagawa wave theme".to_string(),
        // Main palette
        primary: ThemeColor::Rgb(Rgba::from_rgb_int(0x7e9cd8)),   // crystalBlue
        secondary: ThemeColor::Rgb(Rgba::from_rgb_int(0x957fb8)), // oniViolet
        tertiary: ThemeColor::Rgb(Rgba::from_rgb_int(0x7aa89f)),  // waveAqua2
        accent: ThemeColor::Rgb(Rgba::from_rgb_int(0xdca561)),    // carpYellow
        // Semantic
        success: ThemeColor::Rgb(Rgba::from_rgb_int(0x98bb6c)), // springGreen
        warning: ThemeColor::Rgb(Rgba::from_rgb_int(0xdca561)), // carpYellow
        error: ThemeColor::Rgb(Rgba::from_rgb_int(0xc34043)),   // autumnRed
        info: ThemeColor::Rgb(Rgba::from_rgb_int(0x7fb4ca)),    // springBlue
        // Text
        text: ThemeColor::Rgb(Rgba::from_rgb_int(0xdcd7ba)),       // fujiWhite
        text_muted: ThemeColor::Rgb(Rgba::from_rgb_int(0x727169)), // fujiGray
        text_dim: ThemeColor::Rgb(Rgba::from_rgb_int(0x54546d)),   // sumiInk4
        text_disabled: ThemeColor::Rgb(Rgba::from_rgb_int(0x363646)), // sumiInk3
        text_bright: ThemeColor::Rgb(Rgba::from_rgb_int(0xffffff)),
        // Background
        background: ThemeColor::Rgb(Rgba::from_rgb_int(0x1f1f28)),       // sumiInk1
        background_muted: ThemeColor::Rgb(Rgba::from_rgb_int(0x2a2a37)), // sumiInk2
        surface: ThemeColor::Rgb(Rgba::from_rgb_int(0x363646)),          // sumiInk3
        overlay: ThemeColor::Rgb(Rgba::from_rgb_int(0x16161d)),          // sumiInk0
        // Border
        border: ThemeColor::Rgb(Rgba::from_rgb_int(0x54546d)),      // sumiInk4
        border_focus: ThemeColor::Rgb(Rgba::from_rgb_int(0x7e9cd8)), // crystalBlue
    }
}

// =============================================================================
// Everforest Theme
// =============================================================================

/// Everforest - comfortable green-tinted theme.
pub fn everforest() -> Theme {
    Theme {
        name: "everforest".to_string(),
        description: "Everforest theme".to_string(),
        // Main palette
        primary: ThemeColor::Rgb(Rgba::from_rgb_int(0x7fbbb3)),   // aqua
        secondary: ThemeColor::Rgb(Rgba::from_rgb_int(0xd699b6)), // purple
        tertiary: ThemeColor::Rgb(Rgba::from_rgb_int(0x83c092)),  // green
        accent: ThemeColor::Rgb(Rgba::from_rgb_int(0xdbbc7f)),    // yellow
        // Semantic
        success: ThemeColor::Rgb(Rgba::from_rgb_int(0xa7c080)), // green
        warning: ThemeColor::Rgb(Rgba::from_rgb_int(0xdbbc7f)), // yellow
        error: ThemeColor::Rgb(Rgba::from_rgb_int(0xe67e80)),   // red
        info: ThemeColor::Rgb(Rgba::from_rgb_int(0x7fbbb3)),    // aqua
        // Text
        text: ThemeColor::Rgb(Rgba::from_rgb_int(0xd3c6aa)),       // fg
        text_muted: ThemeColor::Rgb(Rgba::from_rgb_int(0x9da9a0)), // grey1
        text_dim: ThemeColor::Rgb(Rgba::from_rgb_int(0x859289)),   // grey0
        text_disabled: ThemeColor::Rgb(Rgba::from_rgb_int(0x5c6a72)), // bg5
        text_bright: ThemeColor::Rgb(Rgba::from_rgb_int(0xfdf6e3)),
        // Background
        background: ThemeColor::Rgb(Rgba::from_rgb_int(0x2d353b)),       // bg_dim
        background_muted: ThemeColor::Rgb(Rgba::from_rgb_int(0x343f44)), // bg1
        surface: ThemeColor::Rgb(Rgba::from_rgb_int(0x3d484d)),          // bg3
        overlay: ThemeColor::Rgb(Rgba::from_rgb_int(0x272e33)),          // bg0
        // Border
        border: ThemeColor::Rgb(Rgba::from_rgb_int(0x5c6a72)),      // bg5
        border_focus: ThemeColor::Rgb(Rgba::from_rgb_int(0xa7c080)), // green
    }
}

// =============================================================================
// Night Owl Theme
// =============================================================================

/// Night Owl - designed with accessibility in mind.
pub fn night_owl() -> Theme {
    Theme {
        name: "nightOwl".to_string(),
        description: "Night Owl theme".to_string(),
        // Main palette
        primary: ThemeColor::Rgb(Rgba::from_rgb_int(0x82aaff)),   // blue
        secondary: ThemeColor::Rgb(Rgba::from_rgb_int(0xc792ea)), // purple
        tertiary: ThemeColor::Rgb(Rgba::from_rgb_int(0x7fdbca)),  // cyan
        accent: ThemeColor::Rgb(Rgba::from_rgb_int(0xffcb6b)),    // yellow
        // Semantic
        success: ThemeColor::Rgb(Rgba::from_rgb_int(0xaddb67)), // green
        warning: ThemeColor::Rgb(Rgba::from_rgb_int(0xffcb6b)), // yellow
        error: ThemeColor::Rgb(Rgba::from_rgb_int(0xef5350)),   // red
        info: ThemeColor::Rgb(Rgba::from_rgb_int(0x7fdbca)),    // cyan
        // Text
        text: ThemeColor::Rgb(Rgba::from_rgb_int(0xd6deeb)),       // fg
        text_muted: ThemeColor::Rgb(Rgba::from_rgb_int(0x637777)), // comment
        text_dim: ThemeColor::Rgb(Rgba::from_rgb_int(0x5f7e97)),   // lineHighlight
        text_disabled: ThemeColor::Rgb(Rgba::from_rgb_int(0x3b4252)), // guide
        text_bright: ThemeColor::Rgb(Rgba::from_rgb_int(0xffffff)),
        // Background
        background: ThemeColor::Rgb(Rgba::from_rgb_int(0x011627)),       // bg
        background_muted: ThemeColor::Rgb(Rgba::from_rgb_int(0x0b2942)), // bg_light
        surface: ThemeColor::Rgb(Rgba::from_rgb_int(0x1d3b53)),          // selection
        overlay: ThemeColor::Rgb(Rgba::from_rgb_int(0x010e1a)),          // bg_dark
        // Border
        border: ThemeColor::Rgb(Rgba::from_rgb_int(0x5f7e97)),      // lineHighlight
        border_focus: ThemeColor::Rgb(Rgba::from_rgb_int(0x82aaff)), // blue
    }
}

// =============================================================================
// Preset Lookup Functions
// =============================================================================

/// Get a preset theme by name.
///
/// Accepts various naming styles:
/// - lowercase: "dracula", "tokyonight"
/// - camelCase: "tokyoNight"
/// - snake_case: "tokyo_night"
///
/// # Example
///
/// ```rust
/// use spark_tui::theme::get_preset;
///
/// let dracula = get_preset("dracula").unwrap();
/// assert_eq!(dracula.name, "dracula");
///
/// let tokyo = get_preset("tokyoNight").unwrap();
/// let tokyo2 = get_preset("tokyo_night").unwrap();
/// assert_eq!(tokyo.name, tokyo2.name);
/// ```
pub fn get_preset(name: &str) -> Option<Theme> {
    match name.to_lowercase().replace('_', "").as_str() {
        "terminal" => Some(terminal()),
        "dracula" => Some(dracula()),
        "nord" => Some(nord()),
        "monokai" => Some(monokai()),
        "solarized" => Some(solarized()),
        "catppuccin" => Some(catppuccin()),
        "gruvbox" => Some(gruvbox()),
        "tokyonight" => Some(tokyo_night()),
        "onedark" => Some(one_dark()),
        "rosepine" => Some(rose_pine()),
        "kanagawa" => Some(kanagawa()),
        "everforest" => Some(everforest()),
        "nightowl" => Some(night_owl()),
        _ => None,
    }
}

/// List all available preset names.
///
/// # Example
///
/// ```rust
/// use spark_tui::theme::preset_names;
///
/// let names = preset_names();
/// assert!(names.contains(&"terminal"));
/// assert!(names.contains(&"dracula"));
/// assert_eq!(names.len(), 13);
/// ```
pub fn preset_names() -> &'static [&'static str] {
    &[
        "terminal",
        "dracula",
        "nord",
        "monokai",
        "solarized",
        "catppuccin",
        "gruvbox",
        "tokyoNight",
        "oneDark",
        "rosePine",
        "kanagawa",
        "everforest",
        "nightOwl",
    ]
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_preset() {
        let t = terminal();
        assert_eq!(t.name, "terminal");
        assert!(t.primary.is_ansi());
        assert!(t.text.is_default());
        assert!(t.background.is_default());
    }

    #[test]
    fn test_dracula_preset() {
        let t = dracula();
        assert_eq!(t.name, "dracula");
        // Primary is OKLCH string
        assert!(matches!(t.primary, ThemeColor::Str(_)));
        // Text is RGB
        assert!(t.text.is_rgb());
        assert_eq!(t.text.resolve(), Rgba::from_rgb_int(0xf8f8f2));
    }

    #[test]
    fn test_all_presets_exist() {
        let names = preset_names();
        assert_eq!(names.len(), 13);

        for name in names {
            let theme = get_preset(name);
            assert!(theme.is_some(), "Preset '{}' should exist", name);
        }
    }

    #[test]
    fn test_get_preset_case_insensitive() {
        assert!(get_preset("DRACULA").is_some());
        assert!(get_preset("Dracula").is_some());
        assert!(get_preset("dracula").is_some());
    }

    #[test]
    fn test_get_preset_snake_case() {
        let t1 = get_preset("tokyoNight").unwrap();
        let t2 = get_preset("tokyo_night").unwrap();
        let t3 = get_preset("tokyonight").unwrap();
        assert_eq!(t1.name, t2.name);
        assert_eq!(t2.name, t3.name);
    }

    #[test]
    fn test_get_preset_invalid() {
        assert!(get_preset("nonexistent").is_none());
        assert!(get_preset("").is_none());
    }

    #[test]
    fn test_nord_preset_colors() {
        let t = nord();
        assert_eq!(t.name, "nord");
        // Check background matches TypeScript
        assert_eq!(t.background.resolve(), Rgba::from_rgb_int(0x2e3440));
        // Check text matches TypeScript
        assert_eq!(t.text.resolve(), Rgba::from_rgb_int(0xd8dee9));
    }

    #[test]
    fn test_catppuccin_preset_colors() {
        let t = catppuccin();
        assert_eq!(t.name, "catppuccin");
        // Check background matches TypeScript (Mocha base)
        assert_eq!(t.background.resolve(), Rgba::from_rgb_int(0x1e1e2e));
        // Check primary is blue
        assert_eq!(t.primary.resolve(), Rgba::from_rgb_int(0x89b4fa));
    }

    #[test]
    fn test_gruvbox_preset_colors() {
        let t = gruvbox();
        assert_eq!(t.name, "gruvbox");
        assert_eq!(t.background.resolve(), Rgba::from_rgb_int(0x282828));
        assert_eq!(t.text.resolve(), Rgba::from_rgb_int(0xebdbb2));
    }

    #[test]
    fn test_tokyo_night_preset_colors() {
        let t = tokyo_night();
        assert_eq!(t.name, "tokyoNight");
        assert_eq!(t.background.resolve(), Rgba::from_rgb_int(0x1a1b26));
        assert_eq!(t.primary.resolve(), Rgba::from_rgb_int(0x7aa2f7));
    }

    #[test]
    fn test_one_dark_preset_colors() {
        let t = one_dark();
        assert_eq!(t.name, "oneDark");
        assert_eq!(t.background.resolve(), Rgba::from_rgb_int(0x282c34));
        assert_eq!(t.primary.resolve(), Rgba::from_rgb_int(0x61afef));
    }

    #[test]
    fn test_rose_pine_preset_colors() {
        let t = rose_pine();
        assert_eq!(t.name, "rosePine");
        assert_eq!(t.background.resolve(), Rgba::from_rgb_int(0x191724));
    }

    #[test]
    fn test_kanagawa_preset_colors() {
        let t = kanagawa();
        assert_eq!(t.name, "kanagawa");
        assert_eq!(t.background.resolve(), Rgba::from_rgb_int(0x1f1f28));
    }

    #[test]
    fn test_everforest_preset_colors() {
        let t = everforest();
        assert_eq!(t.name, "everforest");
        assert_eq!(t.background.resolve(), Rgba::from_rgb_int(0x2d353b));
    }

    #[test]
    fn test_night_owl_preset_colors() {
        let t = night_owl();
        assert_eq!(t.name, "nightOwl");
        assert_eq!(t.background.resolve(), Rgba::from_rgb_int(0x011627));
    }

    #[test]
    fn test_solarized_preset_colors() {
        let t = solarized();
        assert_eq!(t.name, "solarized");
        assert_eq!(t.background.resolve(), Rgba::from_rgb_int(0x002b36));
        assert_eq!(t.primary.resolve(), Rgba::from_rgb_int(0x268bd2));
    }

    #[test]
    fn test_monokai_preset_colors() {
        let t = monokai();
        assert_eq!(t.name, "monokai");
        assert_eq!(t.background.resolve(), Rgba::from_rgb_int(0x272822));
    }
}
