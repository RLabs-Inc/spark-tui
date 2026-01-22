//! FlexNode - Persistent flexbox layout object with reactive Slot properties.
//!
//! Each component gets one FlexNode that lives for the component's entire lifetime.
//! Properties are Slots that bind to user props (signals/getters/static values).
//! The layout algorithm reads `.get()` on properties, creating reactive dependencies.
//!
//! This eliminates the object allocation overhead while maintaining W3C CSS Flexbox
//! spec correctness and automatic reactivity.

use spark_signals::{slot, Slot};
use crate::types::Dimension;

/// FlexNode - Persistent layout object with 33 reactive Slot properties.
///
/// Each component gets one FlexNode for its entire lifetime. The FlexNode holds
/// all layout-related properties as Slots, allowing them to be bound to signals,
/// getters, or static values.
///
/// # Property Categories
///
/// - **Container (5)**: flexDirection, flexWrap, justifyContent, alignItems, alignContent
/// - **Item (5)**: flexGrow, flexShrink, flexBasis, alignSelf, order
/// - **Dimensions (6)**: width, height, minWidth, maxWidth, minHeight, maxHeight
/// - **Spacing (11)**: margin (4), padding (4), gap, rowGap, columnGap
/// - **Border (4)**: borderTop, borderRight, borderBottom, borderLeft
/// - **Other (2)**: overflow, position
///
/// # Example
///
/// ```ignore
/// use spark_tui::engine::{create_flex_node, FlexNode};
/// use spark_signals::signal;
///
/// let flex_node = create_flex_node(0);
///
/// // Bind width to a signal
/// let width_signal = signal(50u16);
/// flex_node.width.set_signal_readonly(move || Dimension::Cells(width_signal.get()));
///
/// // Set static padding
/// flex_node.padding_top.set_value(1);
/// flex_node.padding_bottom.set_value(1);
/// ```
pub struct FlexNode {
    /// Component index in parallel arrays.
    pub index: usize,

    // =========================================================================
    // CONTAINER PROPERTIES (5)
    // =========================================================================

    /// Flex direction: 0=column, 1=row, 2=column-reverse, 3=row-reverse
    pub flex_direction: Slot<u8>,

    /// Flex wrap: 0=nowrap, 1=wrap, 2=wrap-reverse
    pub flex_wrap: Slot<u8>,

    /// Justify content: 0=flex-start, 1=center, 2=flex-end, 3=space-between, 4=space-around, 5=space-evenly
    pub justify_content: Slot<u8>,

    /// Align items: 0=stretch, 1=flex-start, 2=center, 3=flex-end, 4=baseline
    pub align_items: Slot<u8>,

    /// Align content: 0=stretch, 1=flex-start, 2=center, 3=flex-end, 4=space-between, 5=space-around
    pub align_content: Slot<u8>,

    // =========================================================================
    // ITEM PROPERTIES (5)
    // =========================================================================

    /// Flex grow factor (default 0).
    pub flex_grow: Slot<f32>,

    /// Flex shrink factor (default 1).
    pub flex_shrink: Slot<f32>,

    /// Flex basis: initial size before grow/shrink.
    pub flex_basis: Slot<Dimension>,

    /// Align self: 0=auto (inherit from parent alignItems).
    pub align_self: Slot<u8>,

    /// Order for reordering flex items (default 0).
    pub order: Slot<i32>,

    // =========================================================================
    // DIMENSIONS (6)
    // =========================================================================

    /// Width: Auto, Cells(n), or Percent(n).
    pub width: Slot<Dimension>,

    /// Height: Auto, Cells(n), or Percent(n).
    pub height: Slot<Dimension>,

    /// Minimum width constraint.
    pub min_width: Slot<Dimension>,

    /// Maximum width constraint (Auto = no max).
    pub max_width: Slot<Dimension>,

    /// Minimum height constraint.
    pub min_height: Slot<Dimension>,

    /// Maximum height constraint (Auto = no max).
    pub max_height: Slot<Dimension>,

    // =========================================================================
    // SPACING (11)
    // =========================================================================

    /// Margin top.
    pub margin_top: Slot<u16>,

    /// Margin right.
    pub margin_right: Slot<u16>,

    /// Margin bottom.
    pub margin_bottom: Slot<u16>,

    /// Margin left.
    pub margin_left: Slot<u16>,

    /// Padding top.
    pub padding_top: Slot<u16>,

    /// Padding right.
    pub padding_right: Slot<u16>,

    /// Padding bottom.
    pub padding_bottom: Slot<u16>,

    /// Padding left.
    pub padding_left: Slot<u16>,

    /// Gap: applies to both row and column gaps if rowGap/columnGap not set.
    pub gap: Slot<u16>,

    /// Row gap (overrides gap for row spacing).
    pub row_gap: Slot<u16>,

    /// Column gap (overrides gap for column spacing).
    pub column_gap: Slot<u16>,

    // =========================================================================
    // BORDER (4) - layout needs for spacing calculation
    // =========================================================================

    /// Border top width (0=none, >0=width in cells).
    pub border_top: Slot<u16>,

    /// Border right width.
    pub border_right: Slot<u16>,

    /// Border bottom width.
    pub border_bottom: Slot<u16>,

    /// Border left width.
    pub border_left: Slot<u16>,

    // =========================================================================
    // OTHER (2)
    // =========================================================================

    /// Overflow: 0=visible, 1=hidden, 2=scroll.
    pub overflow: Slot<u8>,

    /// Position: 0=relative, 1=absolute.
    pub position: Slot<u8>,
}

impl FlexNode {
    /// Create a new FlexNode with default values.
    pub fn new(index: usize) -> Self {
        Self {
            index,

            // Container properties (5)
            flex_direction: slot(Some(0)),  // column
            flex_wrap: slot(Some(0)),       // nowrap
            justify_content: slot(Some(0)), // flex-start
            align_items: slot(Some(0)),     // stretch
            align_content: slot(Some(0)),   // stretch

            // Item properties (5)
            flex_grow: slot(Some(0.0)),
            flex_shrink: slot(Some(1.0)),   // Default shrink!
            flex_basis: slot(Some(Dimension::Auto)),
            align_self: slot(Some(0)),      // auto
            order: slot(Some(0)),

            // Dimensions (6)
            width: slot(Some(Dimension::Auto)),
            height: slot(Some(Dimension::Auto)),
            min_width: slot(Some(Dimension::Auto)),
            max_width: slot(Some(Dimension::Auto)),
            min_height: slot(Some(Dimension::Auto)),
            max_height: slot(Some(Dimension::Auto)),

            // Spacing (11)
            margin_top: slot(Some(0)),
            margin_right: slot(Some(0)),
            margin_bottom: slot(Some(0)),
            margin_left: slot(Some(0)),
            padding_top: slot(Some(0)),
            padding_right: slot(Some(0)),
            padding_bottom: slot(Some(0)),
            padding_left: slot(Some(0)),
            gap: slot(Some(0)),
            row_gap: slot(Some(0)),
            column_gap: slot(Some(0)),

            // Border (4)
            border_top: slot(Some(0)),
            border_right: slot(Some(0)),
            border_bottom: slot(Some(0)),
            border_left: slot(Some(0)),

            // Other (2)
            overflow: slot(Some(0)),  // visible
            position: slot(Some(0)),  // relative
        }
    }

    /// Disconnect all slot sources for cleanup.
    ///
    /// Called when component is destroyed via `release_index()`.
    /// This breaks reactive connections and resets to default values.
    pub fn disconnect(&self) {
        // Container properties
        self.flex_direction.set_value(0);
        self.flex_wrap.set_value(0);
        self.justify_content.set_value(0);
        self.align_items.set_value(0);
        self.align_content.set_value(0);

        // Item properties
        self.flex_grow.set_value(0.0);
        self.flex_shrink.set_value(1.0);
        self.flex_basis.set_value(Dimension::Auto);
        self.align_self.set_value(0);
        self.order.set_value(0);

        // Dimensions
        self.width.set_value(Dimension::Auto);
        self.height.set_value(Dimension::Auto);
        self.min_width.set_value(Dimension::Auto);
        self.max_width.set_value(Dimension::Auto);
        self.min_height.set_value(Dimension::Auto);
        self.max_height.set_value(Dimension::Auto);

        // Spacing
        self.margin_top.set_value(0);
        self.margin_right.set_value(0);
        self.margin_bottom.set_value(0);
        self.margin_left.set_value(0);
        self.padding_top.set_value(0);
        self.padding_right.set_value(0);
        self.padding_bottom.set_value(0);
        self.padding_left.set_value(0);
        self.gap.set_value(0);
        self.row_gap.set_value(0);
        self.column_gap.set_value(0);

        // Border
        self.border_top.set_value(0);
        self.border_right.set_value(0);
        self.border_bottom.set_value(0);
        self.border_left.set_value(0);

        // Other
        self.overflow.set_value(0);
        self.position.set_value(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spark_signals::signal;

    #[test]
    fn test_flex_node_creation() {
        let node = FlexNode::new(0);
        assert_eq!(node.index, 0);
        assert_eq!(node.flex_direction.get(), 0);
        assert_eq!(node.flex_shrink.get(), 1.0);
        assert_eq!(node.width.get(), Dimension::Auto);
    }

    #[test]
    fn test_flex_node_static_values() {
        let node = FlexNode::new(0);

        node.width.set_value(Dimension::Cells(50));
        node.padding_top.set_value(2);

        assert_eq!(node.width.get(), Dimension::Cells(50));
        assert_eq!(node.padding_top.get(), 2);
    }

    #[test]
    fn test_flex_node_reactive_binding() {
        let node = FlexNode::new(0);
        let width_signal = signal(Dimension::Cells(30));
        let width_for_slot = width_signal.clone();

        node.width.set_signal(width_for_slot);

        assert_eq!(node.width.get(), Dimension::Cells(30));

        width_signal.set(Dimension::Cells(60));
        assert_eq!(node.width.get(), Dimension::Cells(60));
    }

    #[test]
    fn test_flex_node_disconnect() {
        let node = FlexNode::new(0);

        node.width.set_value(Dimension::Cells(100));
        node.padding_top.set_value(5);
        node.flex_grow.set_value(2.0);

        node.disconnect();

        assert_eq!(node.width.get(), Dimension::Auto);
        assert_eq!(node.padding_top.get(), 0);
        assert_eq!(node.flex_grow.get(), 0.0);
    }
}
