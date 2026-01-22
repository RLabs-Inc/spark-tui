//! Interactive Example - Full working demo with colors
//!
//! This example demonstrates everything working together:
//! - Box and Text with theme colors
//! - Reactive updates
//! - Mouse clicks
//! - Keyboard handling
//! - Focus system
//!
//! Run with: cargo run -p spark-tui --example interactive

use std::rc::Rc;
use spark_signals::signal;
use spark_tui::{
    box_primitive, text, BoxProps, TextProps, PropValue,
    Dimension, BorderStyle, Attr,
    reset_registry, mount, unmount,
    theme::{t, set_theme},
};

fn main() {
    reset_registry();

    // Reactive counter for demo
    let counter = signal(0i32);
    let counter_for_text = counter.clone();
    let counter_for_click = counter.clone();

    // Get theme accessor
    let theme = t();

    // Create UI
    let _cleanup = box_primitive(BoxProps {
        id: Some("root".to_string()),
        width: Some(PropValue::Static(Dimension::Cells(50))),
        height: Some(PropValue::Static(Dimension::Cells(12))),
        border: Some(PropValue::Static(BorderStyle::Rounded)),
        border_color: Some(PropValue::Getter(Rc::new({
            let theme = theme.clone();
            move || theme.primary()
        }))),
        bg: Some(PropValue::Getter(Rc::new({
            let theme = theme.clone();
            move || theme.bg()
        }))),
        padding: Some(PropValue::Static(1)),
        gap: Some(PropValue::Static(1)),
        children: Some(Box::new(move || {
            // Title
            let theme = t();
            text(TextProps {
                content: PropValue::Static("spark-tui Demo".to_string()),
                attrs: Some(PropValue::Static(Attr::BOLD)),
                fg: Some(PropValue::Getter(Rc::new({
                    let theme = theme.clone();
                    move || theme.primary()
                }))),
                ..Default::default()
            });

            // Counter text
            let theme = t();
            text(TextProps {
                content: PropValue::Getter(Rc::new({
                    let counter = counter_for_text.clone();
                    move || format!("Counter: {}", counter.get())
                })),
                fg: Some(PropValue::Getter(Rc::new({
                    let theme = theme.clone();
                    move || theme.text()
                }))),
                ..Default::default()
            });

            // Clickable box
            let theme = t();
            box_primitive(BoxProps {
                id: Some("click_box".to_string()),
                width: Some(PropValue::Static(Dimension::Cells(20))),
                height: Some(PropValue::Static(Dimension::Cells(3))),
                border: Some(PropValue::Static(BorderStyle::Single)),
                border_color: Some(PropValue::Getter(Rc::new({
                    let theme = theme.clone();
                    move || theme.accent()
                }))),
                justify_content: Some(PropValue::Static(1)), // center
                align_items: Some(PropValue::Static(2)), // center
                focusable: Some(true),
                on_click: Some(Rc::new({
                    let counter = counter_for_click.clone();
                    move |_event| {
                        counter.set(counter.get() + 1);
                    }
                })),
                children: Some(Box::new({
                    let theme = t();
                    move || {
                        text(TextProps {
                            content: PropValue::Static("Click me!".to_string()),
                            fg: Some(PropValue::Getter(Rc::new({
                                let theme = theme.clone();
                                move || theme.accent()
                            }))),
                            ..Default::default()
                        });
                    }
                })),
                ..Default::default()
            });

            // Instructions
            let theme = t();
            text(TextProps {
                content: PropValue::Static("Press 'd' for Dracula, 't' for Terminal".to_string()),
                fg: Some(PropValue::Getter(Rc::new({
                    let theme = theme.clone();
                    move || theme.text_muted()
                }))),
                ..Default::default()
            });

            text(TextProps {
                content: PropValue::Static("Press Ctrl+C to exit".to_string()),
                fg: Some(PropValue::Getter(Rc::new({
                    let theme = t();
                    move || theme.text_muted()
                }))),
                ..Default::default()
            });
        })),
        ..Default::default()
    });

    // Set up keyboard handlers for theme switching
    let _cleanup_d = spark_tui::on_key("d", || {
        set_theme("dracula");
        true
    });

    let _cleanup_t = spark_tui::on_key("t", || {
        set_theme("terminal");
        true
    });

    // Mount and run!
    match mount() {
        Ok(handle) => {
            // Run blocking event loop
            let _ = spark_tui::pipeline::mount::run(&handle);
            unmount(handle);
        }
        Err(e) => {
            eprintln!("Failed to mount: {}", e);
        }
    }
}
