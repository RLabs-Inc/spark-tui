//! Primitive types - Props and cleanup.
//!
//! These types define the interface for component props.
//! Props support static values, signals, and getters for reactivity.

use std::rc::Rc;
use std::cell::RefCell;
use spark_signals::Signal;

use crate::types::{Rgba, Dimension, Attr, BorderStyle, TextAlign, TextWrap, CursorStyle};
use crate::state::mouse::MouseEvent;
use crate::state::keyboard::KeyboardEvent;

// =============================================================================
// Cleanup Function
// =============================================================================

/// Cleanup function returned by components.
///
/// Call this to unmount the component and release resources.
pub type Cleanup = Box<dyn FnOnce()>;

// =============================================================================
// Callback Types
// =============================================================================

/// Mouse event callback type (Rc for shared ownership in closures).
///
/// Using Rc<dyn Fn> instead of Box<dyn Fn> allows cloning callbacks
/// into closures without ownership issues. This is the standard pattern
/// for event callbacks in Rust when callbacks need to be captured in closures.
pub type MouseCallback = Rc<dyn Fn(&MouseEvent)>;

/// Mouse event callback that can consume the event.
///
/// Return true to indicate the event was consumed and should not
/// propagate to other handlers.
pub type MouseCallbackConsuming = Rc<dyn Fn(&MouseEvent) -> bool>;

/// Keyboard event callback.
///
/// Return true to indicate the event was consumed and should not
/// propagate to other handlers.
pub type KeyCallback = Rc<dyn Fn(&KeyboardEvent) -> bool>;

/// Input value change callback.
pub type InputChangeCallback = Rc<dyn Fn(&str)>;

/// Input submit callback (Enter key).
pub type InputSubmitCallback = Rc<dyn Fn(&str)>;

/// Input cancel callback (Escape key).
pub type InputCancelCallback = Rc<dyn Fn()>;

/// Focus callback (called when component gains focus).
pub type FocusCallback = Rc<dyn Fn()>;

/// Blur callback (called when component loses focus).
pub type BlurCallback = Rc<dyn Fn()>;

// =============================================================================
// Prop Value - Reactive property wrapper
// =============================================================================

/// A property value that can be static, a signal, or a getter.
///
/// This enables reactive props while maintaining type safety.
/// When binding to FlexNode slots or arrays, the reactive connection is preserved.
#[derive(Clone)]
pub enum PropValue<T: Clone + PartialEq + 'static> {
    /// Static value (not reactive).
    Static(T),
    /// Reactive signal (changes propagate automatically).
    Signal(Signal<T>),
    /// Getter function (called each time value is needed).
    Getter(Rc<dyn Fn() -> T>),
}

impl<T: Clone + PartialEq + 'static> PropValue<T> {
    /// Get the current value (for immediate reads).
    pub fn get(&self) -> T {
        match self {
            PropValue::Static(v) => v.clone(),
            PropValue::Signal(s) => s.get(),
            PropValue::Getter(f) => f(),
        }
    }
}

impl<T: Clone + PartialEq + Default + 'static> Default for PropValue<T> {
    fn default() -> Self {
        PropValue::Static(T::default())
    }
}

impl<T: Clone + PartialEq + 'static> From<T> for PropValue<T> {
    fn from(value: T) -> Self {
        PropValue::Static(value)
    }
}

impl<T: Clone + PartialEq + 'static> From<Signal<T>> for PropValue<T> {
    fn from(signal: Signal<T>) -> Self {
        PropValue::Signal(signal)
    }
}

// Dimension is PartialEq so these work
impl From<u16> for PropValue<Dimension> {
    fn from(value: u16) -> Self {
        PropValue::Static(Dimension::from(value))
    }
}

impl From<i32> for PropValue<Dimension> {
    fn from(value: i32) -> Self {
        PropValue::Static(Dimension::from(value))
    }
}

// =============================================================================
// Box Props
// =============================================================================

/// Properties for the Box component.
///
/// Box is the fundamental container - it can have children, borders,
/// backgrounds, and handles events.
///
/// # Example
///
/// ```ignore
/// use spark_tui::primitives::{box_primitive, BoxProps};
/// use spark_signals::signal;
///
/// let width_signal = signal(Dimension::Cells(50));
///
/// let cleanup = box_primitive(BoxProps {
///     width: Some(width_signal.into()),
///     height: Some(10.into()),
///     border: Some(BorderStyle::Single.into()),
///     children: Some(Box::new(|| {
///         // Child components here
///     })),
///     ..Default::default()
/// });
///
/// // Later: update width reactively
/// width_signal.set(Dimension::Cells(80));
/// ```
#[derive(Default)]
pub struct BoxProps {
    // =========================================================================
    // Identity
    // =========================================================================

    /// Optional component ID for lookup.
    pub id: Option<String>,

    // =========================================================================
    // Visibility
    // =========================================================================

    /// Whether the component is visible (default: true).
    pub visible: Option<PropValue<bool>>,

    // =========================================================================
    // Layout - Container
    // =========================================================================

    /// Flex direction: column (default), row, column-reverse, row-reverse.
    pub flex_direction: Option<PropValue<u8>>,

    /// Flex wrap: nowrap (default), wrap, wrap-reverse.
    pub flex_wrap: Option<PropValue<u8>>,

    /// Justify content: flex-start (default), center, flex-end, space-between, space-around, space-evenly.
    pub justify_content: Option<PropValue<u8>>,

    /// Align items: stretch (default), flex-start, center, flex-end, baseline.
    pub align_items: Option<PropValue<u8>>,

    /// Align content (multi-line): stretch (default), flex-start, center, flex-end, space-between, space-around.
    pub align_content: Option<PropValue<u8>>,

    // =========================================================================
    // Layout - Item
    // =========================================================================

    /// Flex grow factor (default: 0).
    pub grow: Option<PropValue<f32>>,

    /// Flex shrink factor (default: 1).
    pub shrink: Option<PropValue<f32>>,

    /// Flex basis (default: auto).
    pub flex_basis: Option<PropValue<Dimension>>,

    /// Align self override (default: auto).
    pub align_self: Option<PropValue<u8>>,

    /// Order for reordering flex items (default: 0).
    pub order: Option<PropValue<i32>>,

    // =========================================================================
    // Dimensions
    // =========================================================================

    /// Width.
    pub width: Option<PropValue<Dimension>>,

    /// Height.
    pub height: Option<PropValue<Dimension>>,

    /// Minimum width.
    pub min_width: Option<PropValue<Dimension>>,

    /// Maximum width.
    pub max_width: Option<PropValue<Dimension>>,

    /// Minimum height.
    pub min_height: Option<PropValue<Dimension>>,

    /// Maximum height.
    pub max_height: Option<PropValue<Dimension>>,

    // =========================================================================
    // Spacing
    // =========================================================================

    /// Margin (all sides).
    pub margin: Option<PropValue<u16>>,

    /// Margin top.
    pub margin_top: Option<PropValue<u16>>,

    /// Margin right.
    pub margin_right: Option<PropValue<u16>>,

    /// Margin bottom.
    pub margin_bottom: Option<PropValue<u16>>,

    /// Margin left.
    pub margin_left: Option<PropValue<u16>>,

    /// Padding (all sides).
    pub padding: Option<PropValue<u16>>,

    /// Padding top.
    pub padding_top: Option<PropValue<u16>>,

    /// Padding right.
    pub padding_right: Option<PropValue<u16>>,

    /// Padding bottom.
    pub padding_bottom: Option<PropValue<u16>>,

    /// Padding left.
    pub padding_left: Option<PropValue<u16>>,

    /// Gap between children (both row and column).
    pub gap: Option<PropValue<u16>>,

    /// Row gap (overrides gap for row spacing).
    pub row_gap: Option<PropValue<u16>>,

    /// Column gap (overrides gap for column spacing).
    pub column_gap: Option<PropValue<u16>>,

    // =========================================================================
    // Position
    // =========================================================================

    /// Position: relative (default) or absolute.
    pub position: Option<PropValue<u8>>,

    // =========================================================================
    // Border
    // =========================================================================

    /// Border style (all sides).
    pub border: Option<PropValue<BorderStyle>>,

    /// Border top style.
    pub border_top: Option<PropValue<BorderStyle>>,

    /// Border right style.
    pub border_right: Option<PropValue<BorderStyle>>,

    /// Border bottom style.
    pub border_bottom: Option<PropValue<BorderStyle>>,

    /// Border left style.
    pub border_left: Option<PropValue<BorderStyle>>,

    /// Border color.
    pub border_color: Option<PropValue<Rgba>>,

    // =========================================================================
    // Visual
    // =========================================================================

    /// Foreground color (for text content).
    pub fg: Option<PropValue<Rgba>>,

    /// Background color.
    pub bg: Option<PropValue<Rgba>>,

    /// Opacity (0-255, 255 = fully opaque).
    pub opacity: Option<PropValue<u8>>,

    // =========================================================================
    // Interaction
    // =========================================================================

    /// Whether the component can receive focus.
    pub focusable: Option<bool>,

    /// Tab index for focus navigation.
    pub tab_index: Option<i32>,

    /// Z-index for stacking order.
    pub z_index: Option<PropValue<i32>>,

    // =========================================================================
    // Overflow
    // =========================================================================

    /// Overflow behavior: visible (default), hidden, scroll, auto.
    pub overflow: Option<PropValue<u8>>,

    /// Auto-scroll to bottom when new content is added.
    ///
    /// When enabled, if the scroll position is at or near the bottom,
    /// the component will automatically scroll to show new content
    /// (e.g., for chat or log views).
    ///
    /// User scrolling up disables auto-follow until they scroll back to bottom.
    pub stick_to_bottom: bool,

    // =========================================================================
    // Event Callbacks
    // =========================================================================

    /// Click callback (fires on mouse up if down was on same component).
    pub on_click: Option<MouseCallback>,

    /// Mouse down callback.
    pub on_mouse_down: Option<MouseCallback>,

    /// Mouse up callback.
    pub on_mouse_up: Option<MouseCallback>,

    /// Mouse enter callback (hover starts).
    pub on_mouse_enter: Option<MouseCallback>,

    /// Mouse leave callback (hover ends).
    pub on_mouse_leave: Option<MouseCallback>,

    /// Scroll callback (mouse wheel).
    pub on_scroll: Option<MouseCallbackConsuming>,

    /// Keyboard callback (when focused).
    pub on_key: Option<KeyCallback>,

    /// Focus callback (fires when component gains focus).
    pub on_focus: Option<FocusCallback>,

    /// Blur callback (fires when component loses focus).
    pub on_blur: Option<BlurCallback>,

    // =========================================================================
    // Children
    // =========================================================================

    /// Child render function.
    pub children: Option<Box<dyn FnOnce()>>,
}

// =============================================================================
// Text Props
// =============================================================================

/// Properties for the Text component.
///
/// Text is a pure display component for text content. Cannot have children.
///
/// # Example
///
/// ```ignore
/// use spark_tui::primitives::{text, TextProps};
/// use spark_signals::signal;
///
/// let message = signal("Hello!".to_string());
///
/// let cleanup = text(TextProps {
///     content: message.into(),
///     attrs: Some(Attr::BOLD.into()),
///     ..Default::default()
/// });
///
/// // Later: update text reactively
/// message.set("Updated!".to_string());
/// ```
pub struct TextProps {
    // =========================================================================
    // Identity
    // =========================================================================

    /// Optional component ID for lookup.
    pub id: Option<String>,

    // =========================================================================
    // Content - REQUIRED
    // =========================================================================

    /// The text content to display.
    pub content: PropValue<String>,

    // =========================================================================
    // Visibility
    // =========================================================================

    /// Whether the component is visible (default: true).
    pub visible: Option<PropValue<bool>>,

    // =========================================================================
    // Text Styling
    // =========================================================================

    /// Text attributes (bold, italic, etc.).
    pub attrs: Option<PropValue<Attr>>,

    /// Text alignment.
    pub align: Option<PropValue<TextAlign>>,

    /// Text wrap mode.
    pub wrap: Option<PropValue<TextWrap>>,

    // =========================================================================
    // Layout - Item
    // =========================================================================

    /// Flex grow factor (default: 0).
    pub grow: Option<PropValue<f32>>,

    /// Flex shrink factor (default: 1).
    pub shrink: Option<PropValue<f32>>,

    /// Flex basis (default: auto).
    pub flex_basis: Option<PropValue<Dimension>>,

    /// Align self override (default: auto).
    pub align_self: Option<PropValue<u8>>,

    // =========================================================================
    // Dimensions
    // =========================================================================

    /// Width.
    pub width: Option<PropValue<Dimension>>,

    /// Height.
    pub height: Option<PropValue<Dimension>>,

    /// Minimum width.
    pub min_width: Option<PropValue<Dimension>>,

    /// Maximum width.
    pub max_width: Option<PropValue<Dimension>>,

    /// Minimum height.
    pub min_height: Option<PropValue<Dimension>>,

    /// Maximum height.
    pub max_height: Option<PropValue<Dimension>>,

    // =========================================================================
    // Spacing
    // =========================================================================

    /// Padding (all sides).
    pub padding: Option<PropValue<u16>>,

    /// Padding top.
    pub padding_top: Option<PropValue<u16>>,

    /// Padding right.
    pub padding_right: Option<PropValue<u16>>,

    /// Padding bottom.
    pub padding_bottom: Option<PropValue<u16>>,

    /// Padding left.
    pub padding_left: Option<PropValue<u16>>,

    // =========================================================================
    // Visual
    // =========================================================================

    /// Foreground color.
    pub fg: Option<PropValue<Rgba>>,

    /// Background color.
    pub bg: Option<PropValue<Rgba>>,

    /// Opacity (0-255, 255 = fully opaque).
    pub opacity: Option<PropValue<u8>>,

    // =========================================================================
    // Interaction
    // =========================================================================

    /// Whether the component can receive focus (for selectable/clickable text).
    pub focusable: Option<bool>,

    /// Tab index for focus navigation.
    pub tab_index: Option<i32>,

    // =========================================================================
    // Event Callbacks
    // =========================================================================

    /// Click callback for clickable text.
    pub on_click: Option<MouseCallback>,

    /// Keyboard callback (when focused).
    pub on_key: Option<KeyCallback>,

    /// Focus callback (fires when component gains focus).
    pub on_focus: Option<FocusCallback>,

    /// Blur callback (fires when component loses focus).
    pub on_blur: Option<BlurCallback>,
}

impl Default for TextProps {
    fn default() -> Self {
        Self {
            id: None,
            content: PropValue::Static(String::new()),
            visible: None,
            attrs: None,
            align: None,
            wrap: None,
            grow: None,
            shrink: None,
            flex_basis: None,
            align_self: None,
            width: None,
            height: None,
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            padding: None,
            padding_top: None,
            padding_right: None,
            padding_bottom: None,
            padding_left: None,
            fg: None,
            bg: None,
            opacity: None,
            focusable: None,
            tab_index: None,
            on_click: None,
            on_key: None,
            on_focus: None,
            on_blur: None,
        }
    }
}

// =============================================================================
// Input History
// =============================================================================

/// Input history state for Up/Down arrow navigation.
///
/// Provides command-line style history where Up recalls previous entries
/// and Down moves forward through history.
///
/// # Example
///
/// ```ignore
/// use spark_tui::primitives::InputHistory;
/// use std::rc::Rc;
/// use std::cell::RefCell;
///
/// // Create shared history
/// let history = Rc::new(RefCell::new(InputHistory::default()));
///
/// // Use with input
/// let props = InputProps {
///     value: signal("".to_string()),
///     history: Some(history.clone()),
///     ..InputProps::new(signal("".to_string()))
/// };
/// ```
#[derive(Debug, Clone)]
pub struct InputHistory {
    /// History entries (oldest first).
    pub entries: Vec<String>,
    /// Current position in history (-1 = not in history, editing new).
    pub position: i32,
    /// Maximum history entries to keep.
    pub max_entries: usize,
    /// Value being edited before entering history.
    pub editing_value: Option<String>,
}

impl Default for InputHistory {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            position: -1,
            max_entries: 100,
            editing_value: None,
        }
    }
}

impl InputHistory {
    /// Create a new history with the given entries.
    pub fn new(entries: Vec<String>) -> Self {
        Self {
            entries,
            position: -1,
            max_entries: 100,
            editing_value: None,
        }
    }

    /// Create a new empty history for auto-tracking.
    pub fn auto() -> Self {
        Self::default()
    }

    /// Add an entry to history.
    ///
    /// - Skips duplicates of the most recent entry
    /// - Skips empty entries
    /// - Trims to max_entries if needed
    pub fn push(&mut self, entry: String) {
        // Don't add duplicates of the most recent entry
        if self.entries.last().map(|s| s.as_str()) == Some(&entry) {
            return;
        }
        // Don't add empty entries
        if entry.is_empty() {
            return;
        }
        self.entries.push(entry);
        // Trim to max size
        while self.entries.len() > self.max_entries {
            self.entries.remove(0);
        }
        // Reset position
        self.position = -1;
        self.editing_value = None;
    }

    /// Move up in history (older).
    ///
    /// Returns the entry to display, or None if at boundary.
    /// On first call, saves the current editing value.
    pub fn up(&mut self, current_value: &str) -> Option<&str> {
        if self.entries.is_empty() {
            return None;
        }

        if self.position == -1 {
            // Save current editing value
            self.editing_value = Some(current_value.to_string());
            self.position = self.entries.len() as i32 - 1;
            Some(&self.entries[self.position as usize])
        } else if self.position > 0 {
            self.position -= 1;
            Some(&self.entries[self.position as usize])
        } else {
            None // At oldest entry
        }
    }

    /// Move down in history (newer).
    ///
    /// Returns the entry to display, or None if not in history.
    /// When reaching the end, returns the original editing value.
    pub fn down(&mut self) -> Option<String> {
        if self.position == -1 {
            return None; // Not in history
        }

        if self.position < self.entries.len() as i32 - 1 {
            self.position += 1;
            Some(self.entries[self.position as usize].clone())
        } else {
            // Return to editing
            self.position = -1;
            self.editing_value.take()
        }
    }

    /// Reset history position (called when value changes during editing).
    pub fn reset_position(&mut self) {
        self.position = -1;
        self.editing_value = None;
    }

    /// Check if currently browsing history.
    pub fn is_browsing(&self) -> bool {
        self.position >= 0
    }
}

// =============================================================================
// Input Props
// =============================================================================

/// Cursor blink configuration.
#[derive(Debug, Clone)]
pub struct BlinkConfig {
    /// Enable blink (default: true).
    pub enabled: bool,
    /// Blink rate in FPS (default: 2 = 500ms cycle).
    pub fps: u8,
    /// Character to show on "off" phase (default: space).
    pub alt_char: Option<char>,
}

impl Default for BlinkConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            fps: 2,
            alt_char: None,
        }
    }
}

/// Cursor configuration for Input component.
#[derive(Clone, Default)]
pub struct CursorConfig {
    /// Cursor shape: Block, Bar, or Underline.
    pub style: Option<CursorStyle>,
    /// Custom cursor character (overrides style preset).
    pub char: Option<char>,
    /// Blink configuration.
    pub blink: Option<BlinkConfig>,
    /// Custom cursor foreground color.
    pub fg: Option<PropValue<Rgba>>,
    /// Custom cursor background color.
    pub bg: Option<PropValue<Rgba>>,
}

/// Properties for the Input component.
///
/// Single-line text input with two-way value binding.
///
/// # Example
///
/// ```ignore
/// use spark_tui::primitives::{input, InputProps};
/// use spark_signals::signal;
///
/// let name = signal("".to_string());
/// let name_clone = name.clone();
///
/// let cleanup = input(InputProps {
///     value: name_clone,
///     placeholder: Some("Enter your name...".to_string()),
///     on_submit: Some(Rc::new(|val| println!("Submitted: {}", val))),
///     ..Default::default()
/// });
///
/// // Later: read or update the value
/// name.set("Alice".to_string());
/// ```
pub struct InputProps {
    // =========================================================================
    // Identity
    // =========================================================================

    /// Optional component ID for lookup.
    pub id: Option<String>,

    // =========================================================================
    // Value (Required)
    // =========================================================================

    /// Current value (two-way bound signal).
    pub value: Signal<String>,

    // =========================================================================
    // Text Display
    // =========================================================================

    /// Placeholder text shown when value is empty.
    pub placeholder: Option<String>,

    /// Placeholder color (default: dimmed fg).
    pub placeholder_color: Option<PropValue<Rgba>>,

    /// Text attributes (bold, italic, etc.) - for the input value.
    pub attrs: Option<PropValue<Attr>>,

    // =========================================================================
    // Input Behavior
    // =========================================================================

    /// Maximum input length (0 = unlimited).
    pub max_length: Option<usize>,

    /// Password mode - mask characters with mask_char.
    pub password: bool,

    /// Password mask character (default: '•').
    pub mask_char: Option<char>,

    /// Auto-focus on mount.
    pub auto_focus: bool,

    /// Input history for Up/Down arrow navigation.
    ///
    /// When provided, Up arrow recalls previous history entries
    /// and Down arrow moves forward through history.
    /// Submit (Enter) automatically adds to history.
    pub history: Option<Rc<RefCell<InputHistory>>>,

    // =========================================================================
    // Cursor
    // =========================================================================

    /// Cursor configuration (style, blink, colors).
    pub cursor: Option<CursorConfig>,

    // =========================================================================
    // Visibility
    // =========================================================================

    /// Whether the component is visible (default: true).
    pub visible: Option<PropValue<bool>>,

    // =========================================================================
    // Dimensions
    // =========================================================================

    /// Width.
    pub width: Option<PropValue<Dimension>>,

    /// Height.
    pub height: Option<PropValue<Dimension>>,

    /// Minimum width.
    pub min_width: Option<PropValue<Dimension>>,

    /// Maximum width.
    pub max_width: Option<PropValue<Dimension>>,

    /// Minimum height.
    pub min_height: Option<PropValue<Dimension>>,

    /// Maximum height.
    pub max_height: Option<PropValue<Dimension>>,

    // =========================================================================
    // Spacing
    // =========================================================================

    /// Padding (all sides).
    pub padding: Option<PropValue<u16>>,

    /// Padding top.
    pub padding_top: Option<PropValue<u16>>,

    /// Padding right.
    pub padding_right: Option<PropValue<u16>>,

    /// Padding bottom.
    pub padding_bottom: Option<PropValue<u16>>,

    /// Padding left.
    pub padding_left: Option<PropValue<u16>>,

    /// Margin (all sides).
    pub margin: Option<PropValue<u16>>,

    /// Margin top.
    pub margin_top: Option<PropValue<u16>>,

    /// Margin right.
    pub margin_right: Option<PropValue<u16>>,

    /// Margin bottom.
    pub margin_bottom: Option<PropValue<u16>>,

    /// Margin left.
    pub margin_left: Option<PropValue<u16>>,

    // =========================================================================
    // Border
    // =========================================================================

    /// Border style (all sides).
    pub border: Option<PropValue<BorderStyle>>,

    /// Border top style.
    pub border_top: Option<PropValue<BorderStyle>>,

    /// Border right style.
    pub border_right: Option<PropValue<BorderStyle>>,

    /// Border bottom style.
    pub border_bottom: Option<PropValue<BorderStyle>>,

    /// Border left style.
    pub border_left: Option<PropValue<BorderStyle>>,

    /// Border color.
    pub border_color: Option<PropValue<Rgba>>,

    // =========================================================================
    // Visual
    // =========================================================================

    /// Foreground color (for text).
    pub fg: Option<PropValue<Rgba>>,

    /// Background color.
    pub bg: Option<PropValue<Rgba>>,

    /// Opacity (0-255, 255 = fully opaque).
    pub opacity: Option<PropValue<u8>>,

    // =========================================================================
    // Interaction
    // =========================================================================

    /// Tab index for focus navigation (default: 0).
    pub tab_index: Option<i32>,

    // =========================================================================
    // Event Callbacks
    // =========================================================================

    /// Called when value changes (on every keystroke).
    pub on_change: Option<InputChangeCallback>,

    /// Called on Enter key.
    pub on_submit: Option<InputSubmitCallback>,

    /// Called on Escape key.
    pub on_cancel: Option<InputCancelCallback>,

    /// Called when component gains focus.
    pub on_focus: Option<FocusCallback>,

    /// Called when component loses focus.
    pub on_blur: Option<BlurCallback>,

    // =========================================================================
    // Mouse Callbacks
    // =========================================================================

    /// Click callback (fires on mouse up if down was on same component).
    pub on_click: Option<MouseCallback>,

    /// Mouse down callback.
    pub on_mouse_down: Option<MouseCallback>,

    /// Mouse up callback.
    pub on_mouse_up: Option<MouseCallback>,

    /// Mouse enter callback (hover starts).
    pub on_mouse_enter: Option<MouseCallback>,

    /// Mouse leave callback (hover ends).
    pub on_mouse_leave: Option<MouseCallback>,

    /// Scroll callback (mouse wheel).
    pub on_scroll: Option<MouseCallbackConsuming>,
}

impl InputProps {
    /// Create new InputProps with the given value signal.
    ///
    /// This is the recommended way to create InputProps since value is required.
    pub fn new(value: Signal<String>) -> Self {
        Self {
            id: None,
            value,
            placeholder: None,
            placeholder_color: None,
            attrs: None,
            max_length: None,
            password: false,
            mask_char: None,
            auto_focus: false,
            history: None,
            cursor: None,
            visible: None,
            width: None,
            height: None,
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            padding: None,
            padding_top: None,
            padding_right: None,
            padding_bottom: None,
            padding_left: None,
            margin: None,
            margin_top: None,
            margin_right: None,
            margin_bottom: None,
            margin_left: None,
            border: None,
            border_top: None,
            border_right: None,
            border_bottom: None,
            border_left: None,
            border_color: None,
            fg: None,
            bg: None,
            opacity: None,
            tab_index: None,
            on_change: None,
            on_submit: None,
            on_cancel: None,
            on_focus: None,
            on_blur: None,
            on_click: None,
            on_mouse_down: None,
            on_mouse_up: None,
            on_mouse_enter: None,
            on_mouse_leave: None,
            on_scroll: None,
        }
    }
}
