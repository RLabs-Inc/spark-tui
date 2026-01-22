//! Reactive Example - Signals and automatic updates
//!
//! This example demonstrates the reactive nature of spark-tui:
//! - Creating components with signal-bound props
//! - Updating signals and seeing values change
//! - Using getter functions for computed values
//!
//! Run with: cargo run -p spark-tui --example reactive

use std::rc::Rc;
use spark_signals::signal;
use spark_tui::{
    box_primitive, text, BoxProps, TextProps, PropValue,
    Dimension, BorderStyle,
    reset_registry, get_flex_node,
    engine::arrays::text as text_arrays,
};

fn main() {
    reset_registry();

    println!("=== spark-tui Reactive Example ===\n");

    // Create reactive signals
    let width = signal(Dimension::Cells(30));
    let counter = signal(0i32);
    let message = signal("Initial message".to_string());

    // Clone signals for use in components
    let width_for_box = width.clone();
    let counter_for_text = counter.clone();
    let message_for_text = message.clone();

    // Create a box with reactive width
    let _cleanup = box_primitive(BoxProps {
        id: Some("reactive_box".to_string()),
        width: Some(PropValue::Signal(width_for_box)),
        height: Some(PropValue::Static(Dimension::Cells(8))),
        border: Some(PropValue::Static(BorderStyle::Rounded)),
        padding: Some(PropValue::Static(1)),
        children: Some(Box::new(move || {
            // Text with reactive content from signal
            text(TextProps {
                id: Some("message_text".to_string()),
                content: PropValue::Signal(message_for_text),
                ..Default::default()
            });

            // Text with computed content using a getter
            let counter_clone = counter_for_text.clone();
            text(TextProps {
                id: Some("counter_text".to_string()),
                content: PropValue::Getter(Rc::new(move || {
                    format!("Counter: {}", counter_clone.get())
                })),
                ..Default::default()
            });
        })),
        ..Default::default()
    });

    // Get initial values
    let flex_node = get_flex_node(0).unwrap();
    println!("Initial state:");
    println!("  Box width: {:?}", flex_node.width.get());
    println!("  Message: \"{}\"", text_arrays::get_text_content(1));
    println!("  Counter text: \"{}\"", text_arrays::get_text_content(2));

    // Update signals - values should change!
    println!("\n--- Updating signals ---\n");

    width.set(Dimension::Cells(50));
    message.set("Updated message!".to_string());
    counter.set(42);

    println!("After updates:");
    println!("  Box width: {:?}", flex_node.width.get());
    println!("  Message: \"{}\"", text_arrays::get_text_content(1));
    println!("  Counter text: \"{}\"", text_arrays::get_text_content(2));

    // Update counter again
    println!("\n--- Incrementing counter ---\n");
    counter.set(counter.get() + 1);
    println!("  Counter text: \"{}\"", text_arrays::get_text_content(2));

    counter.set(counter.get() + 1);
    println!("  Counter text: \"{}\"", text_arrays::get_text_content(2));

    println!("\n=== Reactive updates work! ===");
}
