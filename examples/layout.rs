//! Layout Example - Nested components and flexbox
//!
//! This example demonstrates layout capabilities:
//! - Nested box containers
//! - Row and column layouts
//! - Flex properties (grow, gap)
//! - Computing layout with Taffy
//!
//! Run with: cargo run -p spark-tui --example layout

use spark_tui::{
    box_primitive, text, BoxProps, TextProps, PropValue,
    Dimension, BorderStyle, Rgba, Attr,
    reset_registry, compute_layout, set_terminal_size,
    engine::arrays::core,
    types::ComponentType,
};

fn main() {
    reset_registry();

    println!("=== spark-tui Layout Example ===\n");

    // Set terminal size for layout computation
    set_terminal_size(80, 24);

    // Create a dashboard-like layout
    let _cleanup = box_primitive(BoxProps {
        id: Some("root".to_string()),
        width: Some(PropValue::Static(Dimension::Percent(100.0))),
        height: Some(PropValue::Static(Dimension::Cells(20))),
        flex_direction: Some(PropValue::Static(0)), // column
        gap: Some(PropValue::Static(1)),
        children: Some(Box::new(|| {
            // Header row
            box_primitive(BoxProps {
                id: Some("header".to_string()),
                width: Some(PropValue::Static(Dimension::Percent(100.0))),
                height: Some(PropValue::Static(Dimension::Cells(3))),
                border: Some(PropValue::Static(BorderStyle::Double)),
                border_color: Some(PropValue::Static(Rgba::CYAN)),
                justify_content: Some(PropValue::Static(1)), // center
                align_items: Some(PropValue::Static(2)), // center
                children: Some(Box::new(|| {
                    text(TextProps {
                        content: PropValue::Static("Dashboard".to_string()),
                        attrs: Some(PropValue::Static(Attr::BOLD)),
                        fg: Some(PropValue::Static(Rgba::YELLOW)),
                        ..Default::default()
                    });
                })),
                ..Default::default()
            });

            // Content row (horizontal layout)
            box_primitive(BoxProps {
                id: Some("content".to_string()),
                width: Some(PropValue::Static(Dimension::Percent(100.0))),
                grow: Some(PropValue::Static(1.0)), // Take remaining space
                flex_direction: Some(PropValue::Static(1)), // row
                gap: Some(PropValue::Static(1)),
                children: Some(Box::new(|| {
                    // Left panel
                    box_primitive(BoxProps {
                        id: Some("left_panel".to_string()),
                        width: Some(PropValue::Static(Dimension::Cells(20))),
                        border: Some(PropValue::Static(BorderStyle::Single)),
                        padding: Some(PropValue::Static(1)),
                        children: Some(Box::new(|| {
                            text(TextProps {
                                content: PropValue::Static("Menu".to_string()),
                                attrs: Some(PropValue::Static(Attr::BOLD)),
                                ..Default::default()
                            });
                            text(TextProps {
                                content: PropValue::Static("- Item 1".to_string()),
                                ..Default::default()
                            });
                            text(TextProps {
                                content: PropValue::Static("- Item 2".to_string()),
                                ..Default::default()
                            });
                            text(TextProps {
                                content: PropValue::Static("- Item 3".to_string()),
                                ..Default::default()
                            });
                        })),
                        ..Default::default()
                    });

                    // Main content area
                    box_primitive(BoxProps {
                        id: Some("main".to_string()),
                        grow: Some(PropValue::Static(1.0)), // Expand to fill
                        border: Some(PropValue::Static(BorderStyle::Single)),
                        border_color: Some(PropValue::Static(Rgba::GREEN)),
                        padding: Some(PropValue::Static(1)),
                        children: Some(Box::new(|| {
                            text(TextProps {
                                content: PropValue::Static("Main Content".to_string()),
                                attrs: Some(PropValue::Static(Attr::BOLD | Attr::UNDERLINE)),
                                ..Default::default()
                            });
                            text(TextProps {
                                content: PropValue::Static("This is the main content area.".to_string()),
                                ..Default::default()
                            });
                            text(TextProps {
                                content: PropValue::Static("It expands to fill available space.".to_string()),
                                ..Default::default()
                            });
                        })),
                        ..Default::default()
                    });

                    // Right panel
                    box_primitive(BoxProps {
                        id: Some("right_panel".to_string()),
                        width: Some(PropValue::Static(Dimension::Cells(15))),
                        border: Some(PropValue::Static(BorderStyle::Single)),
                        padding: Some(PropValue::Static(1)),
                        children: Some(Box::new(|| {
                            text(TextProps {
                                content: PropValue::Static("Info".to_string()),
                                attrs: Some(PropValue::Static(Attr::BOLD)),
                                ..Default::default()
                            });
                            text(TextProps {
                                content: PropValue::Static("Status: OK".to_string()),
                                fg: Some(PropValue::Static(Rgba::GREEN)),
                                ..Default::default()
                            });
                        })),
                        ..Default::default()
                    });
                })),
                ..Default::default()
            });

            // Footer
            box_primitive(BoxProps {
                id: Some("footer".to_string()),
                width: Some(PropValue::Static(Dimension::Percent(100.0))),
                height: Some(PropValue::Static(Dimension::Cells(1))),
                bg: Some(PropValue::Static(Rgba::rgb(40, 40, 50))),
                justify_content: Some(PropValue::Static(1)), // center
                children: Some(Box::new(|| {
                    text(TextProps {
                        content: PropValue::Static("Press 'q' to quit".to_string()),
                        fg: Some(PropValue::Static(Rgba::GRAY)),
                        ..Default::default()
                    });
                })),
                ..Default::default()
            });
        })),
        ..Default::default()
    });

    // Print component hierarchy
    println!("Component hierarchy:");
    print_hierarchy(0, 0);

    // Compute layout
    println!("\nComputing layout (80x24 terminal)...");
    let layout = compute_layout(80, 24, true);

    println!("\nLayout results:");
    for i in 0..18 {
        if core::get_component_type(i) != ComponentType::None {
            let w = layout.width.get(i).copied().unwrap_or(0);
            let h = layout.height.get(i).copied().unwrap_or(0);
            let x = layout.x.get(i).copied().unwrap_or(0);
            let y = layout.y.get(i).copied().unwrap_or(0);
            let component_type = core::get_component_type(i);
            println!("  [{:2}] {:?} at ({}, {}) size {}x{}", i, component_type, x, y, w, h);
        }
    }

    println!("\n=== Layout Example Complete ===");
}

fn print_hierarchy(index: usize, depth: usize) {
    let indent = "  ".repeat(depth);
    let component_type = core::get_component_type(index);

    if component_type == ComponentType::None {
        return;
    }

    println!("{}[{}] {:?}", indent, index, component_type);

    // Find children
    for child in 0..20 {
        if core::get_parent_index(child) == Some(index) {
            print_hierarchy(child, depth + 1);
        }
    }
}
