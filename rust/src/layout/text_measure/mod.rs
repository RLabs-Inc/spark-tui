//! Professional text measurement for terminal rendering.
//!
//! Provides Unicode-aware text measurement, ANSI escape sequence handling,
//! word-break text wrapping, and grapheme-safe text truncation.
//!
//! # Capabilities
//!
//! - **Width calculation**: Correct terminal cell width for any Unicode text
//! - **ANSI stripping**: Properly skips CSI, OSC, and ESC escape sequences
//! - **Grapheme awareness**: Never breaks in the middle of a grapheme cluster
//! - **Emoji sequences**: ZWJ families, skin tones, flags measured as width 2
//! - **Text wrapping**: Character-break and word-break modes
//! - **Text truncation**: Grapheme-safe truncation with configurable suffix
//!
//! # Implementation
//!
//! Uses `unicode-width` (Unicode 16.0 East Asian Width tables) and
//! `unicode-segmentation` (UAX #29 grapheme cluster boundaries) as the
//! foundation, with custom handling for ANSI escapes and emoji sequences.

mod ansi;
mod truncate;
mod width;
mod wrap;

pub use ansi::strip_ansi;
pub use truncate::truncate_text;
pub use width::{char_width, grapheme_width, string_width};
pub use wrap::{measure_text_height, wrap_text, wrap_text_word};
