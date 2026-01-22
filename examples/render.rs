//! Render Example - Full pipeline with terminal output
//!
//! This example demonstrates the complete rendering pipeline:
//! - Creating components
//! - Computing layout
//! - Generating frame buffer
//! - Rendering to terminal
//!
//! Run with: cargo run -p spark-tui --example render

use spark_tui::{
    box_primitive, text, BoxProps, TextProps, PropValue,
    Dimension, BorderStyle, Rgba, Attr,
    reset_registry, set_terminal_size,
    create_layout_derived, create_frame_buffer_derived,
    FrameBuffer,
};

fn main() {
    reset_registry();

    println!("=== spark-tui Render Example ===\n");

    // Set a fixed terminal size for this example
    set_terminal_size(60, 15);

    // Create a simple UI
    let _cleanup = box_primitive(BoxProps {
        width: Some(PropValue::Static(Dimension::Cells(40))),
        height: Some(PropValue::Static(Dimension::Cells(8))),
        border: Some(PropValue::Static(BorderStyle::Rounded)),
        border_color: Some(PropValue::Static(Rgba::CYAN)),
        bg: Some(PropValue::Static(Rgba::rgb(20, 20, 30))),
        padding: Some(PropValue::Static(1)),
        children: Some(Box::new(|| {
            text(TextProps {
                content: PropValue::Static("spark-tui".to_string()),
                attrs: Some(PropValue::Static(Attr::BOLD)),
                fg: Some(PropValue::Static(Rgba::YELLOW)),
                ..Default::default()
            });

            text(TextProps {
                content: PropValue::Static("".to_string()),
                ..Default::default()
            });

            text(TextProps {
                content: PropValue::Static("Reactive TUI Framework".to_string()),
                fg: Some(PropValue::Static(Rgba::WHITE)),
                ..Default::default()
            });

            text(TextProps {
                content: PropValue::Static("for Rust".to_string()),
                fg: Some(PropValue::Static(Rgba::GRAY)),
                ..Default::default()
            });
        })),
        ..Default::default()
    });

    // Create the reactive pipeline
    let layout_derived = create_layout_derived();
    let fb_derived = create_frame_buffer_derived(layout_derived);

    // Get the computed frame buffer
    let result = fb_derived.get();

    println!("Frame buffer size: {}x{}", result.buffer.width(), result.buffer.height());
    println!("Terminal size: {:?}", result.terminal_size);
    println!("Hit regions: {}", result.hit_regions.len());

    // Print the frame buffer as ASCII art
    println!("\nRendered output (ASCII preview):\n");
    print_buffer_preview(&result.buffer);

    println!("\n=== Render Example Complete ===");
}

fn print_buffer_preview(buffer: &FrameBuffer) {
    let max_y = buffer.height().min(20);
    let max_x = buffer.width().min(70);

    for y in 0..max_y {
        for x in 0..max_x {
            if let Some(cell) = buffer.get(x, y) {
                let ch = if cell.char == 0 || cell.char == b' ' as u32 {
                    ' '
                } else {
                    char::from_u32(cell.char).unwrap_or('?')
                };
                print!("{}", ch);
            } else {
                print!(" ");
            }
        }
        println!();
    }
}
