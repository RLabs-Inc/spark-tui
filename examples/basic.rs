//! Basic Example - Box and Text primitives
//!
//! This example demonstrates basic usage of the spark-tui primitives:
//! - Creating a Box container
//! - Adding Text children
//! - Setting dimensions and borders
//!
//! Run with: cargo run -p spark-tui --example basic

use spark_tui::{
    box_primitive, text, BoxProps, TextProps, PropValue,
    Dimension, BorderStyle, Attr, Rgba,
    reset_registry,
    engine::arrays::{core, text as text_arrays, visual},
    types::ComponentType,
};

fn main() {
    // Reset to ensure clean state
    reset_registry();

    println!("=== spark-tui Basic Example ===\n");

    // Create a simple box with text
    let _cleanup = box_primitive(BoxProps {
        id: Some("main_box".to_string()),
        width: Some(PropValue::Static(Dimension::Cells(40))),
        height: Some(PropValue::Static(Dimension::Cells(10))),
        border: Some(PropValue::Static(BorderStyle::Single)),
        border_color: Some(PropValue::Static(Rgba::CYAN)),
        bg: Some(PropValue::Static(Rgba::rgb(30, 30, 40))),
        padding: Some(PropValue::Static(1)),
        children: Some(Box::new(|| {
            // Title text
            text(TextProps {
                id: Some("title".to_string()),
                content: PropValue::Static("Hello, spark-tui!".to_string()),
                attrs: Some(PropValue::Static(Attr::BOLD)),
                fg: Some(PropValue::Static(Rgba::YELLOW)),
                ..Default::default()
            });

            // Description text
            text(TextProps {
                id: Some("description".to_string()),
                content: PropValue::Static("A reactive TUI framework for Rust".to_string()),
                fg: Some(PropValue::Static(Rgba::WHITE)),
                ..Default::default()
            });
        })),
        ..Default::default()
    });

    // Verify the components were created correctly
    println!("Components created:");
    println!("  Index 0: {:?} (main_box)", core::get_component_type(0));
    println!("  Index 1: {:?} (title)", core::get_component_type(1));
    println!("  Index 2: {:?} (description)", core::get_component_type(2));

    println!("\nParent relationships:");
    println!("  title parent: {:?}", core::get_parent_index(1));
    println!("  description parent: {:?}", core::get_parent_index(2));

    println!("\nText content:");
    println!("  title: \"{}\"", text_arrays::get_text_content(1));
    println!("  description: \"{}\"", text_arrays::get_text_content(2));

    println!("\nVisual properties:");
    println!("  main_box border: {:?}", visual::get_border_style(0));
    println!("  main_box border_color: {:?}", visual::get_border_color(0));
    println!("  main_box bg: {:?}", visual::get_bg_color(0));
    println!("  title fg: {:?}", visual::get_fg_color(1));
    println!("  title attrs: {:?}", text_arrays::get_text_attrs(1));

    println!("\n=== Example Complete ===");
}
