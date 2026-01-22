//! Benchmark Example - Stress test spark-tui
//!
//! Tests:
//! - Component creation throughput
//! - Reactive signal update speed
//! - Layout computation at scale
//! - Frame buffer generation
//! - Memory usage estimation
//!
//! Run with: cargo run -p spark-tui --example benchmark --release

use std::time::{Duration, Instant};
use spark_signals::signal;
use spark_tui::{
    box_primitive, text, BoxProps, TextProps, PropValue,
    Dimension, BorderStyle,
    reset_registry, set_terminal_size,
    create_layout_derived, create_frame_buffer_derived,
    get_allocated_indices,
};

fn main() {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║           spark-tui Benchmark Suite                      ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    // Set a reasonable terminal size
    set_terminal_size(200, 50);

    // Run benchmarks
    bench_component_creation();
    bench_reactive_updates();
    bench_layout_computation();
    bench_frame_buffer();
    bench_stress_test();
    bench_laggy_threshold();
    bench_break_it();

    println!("\n══════════════════════════════════════════════════════════");
    println!("Benchmark complete!");
}

/// Benchmark component creation throughput
fn bench_component_creation() {
    println!("┌──────────────────────────────────────────────────────────┐");
    println!("│ 1. Component Creation Throughput                         │");
    println!("└──────────────────────────────────────────────────────────┘");

    for count in [100, 1_000, 10_000, 50_000] {
        reset_registry();

        let start = Instant::now();

        // Create root
        let _root = box_primitive(BoxProps {
            width: Some(PropValue::Static(Dimension::Percent(100.0))),
            height: Some(PropValue::Static(Dimension::Percent(100.0))),
            children: Some(Box::new(move || {
                for i in 0..count {
                    if i % 2 == 0 {
                        box_primitive(BoxProps {
                            width: Some(PropValue::Static(Dimension::Cells(10))),
                            height: Some(PropValue::Static(Dimension::Cells(1))),
                            ..Default::default()
                        });
                    } else {
                        text(TextProps {
                            content: PropValue::Static(format!("Item {}", i)),
                            ..Default::default()
                        });
                    }
                }
            })),
            ..Default::default()
        });

        let elapsed = start.elapsed();
        let per_component = elapsed.as_nanos() as f64 / count as f64;
        let per_second = 1_000_000_000.0 / per_component;

        println!(
            "  {:>6} components: {:>8.2?}  ({:.0} ns/component, {:.0}k/sec)",
            count, elapsed, per_component, per_second / 1000.0
        );
    }
    println!();
}

/// Benchmark reactive signal updates
fn bench_reactive_updates() {
    println!("┌──────────────────────────────────────────────────────────┐");
    println!("│ 2. Reactive Signal Updates                               │");
    println!("└──────────────────────────────────────────────────────────┘");

    reset_registry();

    // Create signals
    let signals: Vec<_> = (0..1000).map(|i| signal(i)).collect();

    // Create components bound to signals
    let _root = box_primitive(BoxProps {
        width: Some(PropValue::Static(Dimension::Percent(100.0))),
        children: Some(Box::new({
            let signals = signals.clone();
            move || {
                for sig in signals.iter().take(100) {
                    let sig = sig.clone();
                    text(TextProps {
                        content: PropValue::Getter(std::rc::Rc::new(move || {
                            format!("Value: {}", sig.get())
                        })),
                        ..Default::default()
                    });
                }
            }
        })),
        ..Default::default()
    });

    // Benchmark signal updates
    for update_count in [1_000, 10_000, 100_000, 1_000_000] {
        let start = Instant::now();

        for i in 0..update_count {
            signals[i % signals.len()].set(i as i32);
        }

        let elapsed = start.elapsed();
        let per_update = elapsed.as_nanos() as f64 / update_count as f64;
        let per_second = 1_000_000_000.0 / per_update;

        println!(
            "  {:>9} updates: {:>8.2?}  ({:.0} ns/update, {:.1}M/sec)",
            update_count, elapsed, per_update, per_second / 1_000_000.0
        );
    }
    println!();
}

/// Benchmark layout computation
fn bench_layout_computation() {
    println!("┌──────────────────────────────────────────────────────────┐");
    println!("│ 3. Layout Computation (Taffy)                            │");
    println!("└──────────────────────────────────────────────────────────┘");

    for count in [100, 500, 1_000, 5_000] {
        reset_registry();

        // Create nested layout
        let _root = box_primitive(BoxProps {
            width: Some(PropValue::Static(Dimension::Percent(100.0))),
            height: Some(PropValue::Static(Dimension::Percent(100.0))),
            flex_direction: Some(PropValue::Static(0)), // column
            children: Some(Box::new(move || {
                for _ in 0..(count / 10) {
                    box_primitive(BoxProps {
                        width: Some(PropValue::Static(Dimension::Percent(100.0))),
                        flex_direction: Some(PropValue::Static(1)), // row
                        children: Some(Box::new(|| {
                            for _ in 0..10 {
                                box_primitive(BoxProps {
                                    width: Some(PropValue::Static(Dimension::Cells(10))),
                                    height: Some(PropValue::Static(Dimension::Cells(2))),
                                    border: Some(PropValue::Static(BorderStyle::Single)),
                                    ..Default::default()
                                });
                            }
                        })),
                        ..Default::default()
                    });
                }
            })),
            ..Default::default()
        });

        // Create and trigger layout derived
        let layout_derived = create_layout_derived();

        let start = Instant::now();
        let iterations = 100;
        for _ in 0..iterations {
            let _ = layout_derived.get();
        }
        let elapsed = start.elapsed();
        let per_layout = elapsed / iterations;

        println!(
            "  {:>5} nodes: {:>8.2?}/layout  ({:.2} ms)",
            count, per_layout, per_layout.as_secs_f64() * 1000.0
        );
    }
    println!();
}

/// Benchmark frame buffer generation
fn bench_frame_buffer() {
    println!("┌──────────────────────────────────────────────────────────┐");
    println!("│ 4. Frame Buffer Generation                               │");
    println!("└──────────────────────────────────────────────────────────┘");

    for (width, height) in [(80, 24), (200, 50), (400, 100)] {
        reset_registry();
        set_terminal_size(width, height);

        // Create a grid of boxes
        let cols = width / 12;
        let rows = height / 3;

        let _root = box_primitive(BoxProps {
            width: Some(PropValue::Static(Dimension::Percent(100.0))),
            height: Some(PropValue::Static(Dimension::Percent(100.0))),
            flex_direction: Some(PropValue::Static(0)), // column
            children: Some(Box::new(move || {
                for r in 0..rows {
                    box_primitive(BoxProps {
                        flex_direction: Some(PropValue::Static(1)), // row
                        children: Some(Box::new(move || {
                            for c in 0..cols {
                                box_primitive(BoxProps {
                                    width: Some(PropValue::Static(Dimension::Cells(10))),
                                    height: Some(PropValue::Static(Dimension::Cells(2))),
                                    border: Some(PropValue::Static(BorderStyle::Single)),
                                    children: Some(Box::new(move || {
                                        text(TextProps {
                                            content: PropValue::Static(format!("{},{}", r, c)),
                                            ..Default::default()
                                        });
                                    })),
                                    ..Default::default()
                                });
                            }
                        })),
                        ..Default::default()
                    });
                }
            })),
            ..Default::default()
        });

        let layout_derived = create_layout_derived();
        let fb_derived = create_frame_buffer_derived(layout_derived);

        let start = Instant::now();
        let iterations = 50;
        for _ in 0..iterations {
            let _ = fb_derived.get();
        }
        let elapsed = start.elapsed();
        let per_frame = elapsed / iterations;
        let fps = 1.0 / per_frame.as_secs_f64();

        let cells = width as u32 * height as u32;
        println!(
            "  {}x{} ({} cells): {:>6.2?}/frame  ({:.0} FPS)",
            width, height, cells, per_frame, fps
        );
    }
    println!();
}

/// Stress test - find the breaking point
fn bench_stress_test() {
    println!("┌──────────────────────────────────────────────────────────┐");
    println!("│ 5. Stress Test - Finding the Limits                      │");
    println!("└──────────────────────────────────────────────────────────┘");

    let mut last_success = 0;

    for count in [10_000, 25_000, 50_000, 100_000, 200_000, 500_000] {
        reset_registry();

        let start = Instant::now();

        // Create flat list of components
        let _root = box_primitive(BoxProps {
            children: Some(Box::new(move || {
                for i in 0..count {
                    text(TextProps {
                        content: PropValue::Static(format!("{}", i)),
                        ..Default::default()
                    });
                }
            })),
            ..Default::default()
        });

        let creation_time = start.elapsed();
        let allocated = get_allocated_indices().len();

        if creation_time > Duration::from_secs(10) {
            println!(
                "  {:>7} components: TIMEOUT (>{:.1}s) - stopping here",
                count, creation_time.as_secs_f64()
            );
            break;
        }

        // Estimate memory (very rough)
        // Each component has: FlexNode (~400 bytes), arrays (~100 bytes)
        let estimated_mb = (allocated as f64 * 500.0) / (1024.0 * 1024.0);

        println!(
            "  {:>7} components: {:>8.2?}  (~{:.1} MB estimated)",
            count, creation_time, estimated_mb
        );

        last_success = count;
    }

    println!();
    println!("  Maximum tested: {} components", last_success);
    println!();

    // Quick reactive stress test
    println!("  Reactive stress test (1000 signals, 1M updates)...");
    reset_registry();

    let signals: Vec<_> = (0..1000).map(|i| signal(i)).collect();

    let start = Instant::now();
    for i in 0..1_000_000 {
        signals[i % 1000].set(i as i32);
    }
    let elapsed = start.elapsed();

    println!(
        "  1M signal updates: {:?} ({:.1}M updates/sec)",
        elapsed,
        1.0 / elapsed.as_secs_f64()
    );
}

/// Find the "laggy threshold" - where does it become unpleasant?
fn bench_laggy_threshold() {
    println!("┌──────────────────────────────────────────────────────────┐");
    println!("│ 6. Laggy Threshold - Where Does It Feel Slow?            │");
    println!("└──────────────────────────────────────────────────────────┘");
    println!("  Target: <16.6ms for 60 FPS, <33ms for 30 FPS\n");

    // Test full pipeline (create + layout + render) for increasing sizes
    println!("  Full Pipeline (layout + frame buffer):");

    for count in [1_000, 2_500, 5_000, 10_000, 20_000, 50_000, 100_000] {
        reset_registry();
        set_terminal_size(200, 50);

        // Create nested grid
        let rows = (count as f64).sqrt() as u16;
        let cols = rows;

        let _root = box_primitive(BoxProps {
            width: Some(PropValue::Static(Dimension::Percent(100.0))),
            height: Some(PropValue::Static(Dimension::Percent(100.0))),
            flex_direction: Some(PropValue::Static(0)),
            children: Some(Box::new(move || {
                for _ in 0..rows {
                    box_primitive(BoxProps {
                        flex_direction: Some(PropValue::Static(1)),
                        children: Some(Box::new(move || {
                            for _ in 0..cols {
                                box_primitive(BoxProps {
                                    width: Some(PropValue::Static(Dimension::Cells(2))),
                                    height: Some(PropValue::Static(Dimension::Cells(1))),
                                    ..Default::default()
                                });
                            }
                        })),
                        ..Default::default()
                    });
                }
            })),
            ..Default::default()
        });

        let layout_derived = create_layout_derived();
        let fb_derived = create_frame_buffer_derived(layout_derived);

        // Measure full pipeline
        let start = Instant::now();
        let iterations = 10;
        for _ in 0..iterations {
            let _ = fb_derived.get();
        }
        let elapsed = start.elapsed();
        let per_frame = elapsed / iterations;
        let fps = 1.0 / per_frame.as_secs_f64();

        let status = if per_frame.as_millis() < 17 {
            "smooth (60fps+)"
        } else if per_frame.as_millis() < 33 {
            "acceptable (30fps+)"
        } else if per_frame.as_millis() < 100 {
            "LAGGY"
        } else {
            "UNUSABLE"
        };

        println!(
            "    {:>6} nodes: {:>8.2?}/frame ({:>5.0} FPS) - {}",
            count, per_frame, fps, status
        );

        if per_frame.as_millis() > 100 {
            println!("    ^ Found the laggy threshold!");
            break;
        }
    }

    println!();

    // Test reactive updates WITH effects (more realistic)
    println!("  Reactive Updates with Effects:");
    reset_registry();

    let counter = signal(0i32);
    let counter_clone = counter.clone();

    // Create component that reads the signal
    let _root = box_primitive(BoxProps {
        children: Some(Box::new(move || {
            let counter = counter_clone.clone();
            text(TextProps {
                content: PropValue::Getter(std::rc::Rc::new(move || {
                    format!("Count: {}", counter.get())
                })),
                ..Default::default()
            });
        })),
        ..Default::default()
    });

    let layout_derived = create_layout_derived();
    let fb_derived = create_frame_buffer_derived(layout_derived);

    // Warm up
    let _ = fb_derived.get();

    for updates_per_frame in [1, 10, 100, 1_000, 10_000] {
        let start = Instant::now();
        let frames = 100;

        for frame in 0..frames {
            for u in 0..updates_per_frame {
                counter.set((frame * updates_per_frame + u) as i32);
            }
            let _ = fb_derived.get();
        }

        let elapsed = start.elapsed();
        let per_frame = elapsed / frames;
        let fps = 1.0 / per_frame.as_secs_f64();

        println!(
            "    {:>5} updates/frame: {:>8.2?}/frame ({:.0} FPS)",
            updates_per_frame, per_frame, fps
        );
    }
}

/// TRY TO BREAK IT - aggressive stress tests
fn bench_break_it() {
    println!("┌──────────────────────────────────────────────────────────┐");
    println!("│ 7. TRY TO BREAK IT                                       │");
    println!("└──────────────────────────────────────────────────────────┘\n");

    // Test 1: MASSIVE component count
    println!("  Test 1: Maximum components until OOM or >10s...");
    for count in [500_000, 1_000_000, 2_000_000, 5_000_000] {
        reset_registry();

        let start = Instant::now();

        let result = std::panic::catch_unwind(|| {
            let _root = box_primitive(BoxProps {
                children: Some(Box::new(move || {
                    for i in 0..count {
                        text(TextProps {
                            content: PropValue::Static(format!("{}", i % 1000)),
                            ..Default::default()
                        });
                    }
                })),
                ..Default::default()
            });
        });

        let elapsed = start.elapsed();

        match result {
            Ok(_) => {
                if elapsed > Duration::from_secs(10) {
                    println!("    {:>10} components: TIMEOUT ({:.1}s) - stopping", count, elapsed.as_secs_f64());
                    break;
                }
                let mb = (count as f64 * 500.0) / (1024.0 * 1024.0);
                println!("    {:>10} components: {:>8.2?} (~{:.0} MB)", count, elapsed, mb);
            }
            Err(_) => {
                println!("    {:>10} components: CRASHED/OOM!", count);
                break;
            }
        }
    }

    println!();

    // Test 2: Deep nesting (recursive depth)
    println!("  Test 2: Maximum nesting depth...");
    for depth in [100, 500, 1_000, 2_000, 5_000, 10_000] {
        reset_registry();

        let start = Instant::now();

        fn create_nested(depth: usize, current: usize) {
            if current >= depth {
                text(TextProps {
                    content: PropValue::Static("leaf".to_string()),
                    ..Default::default()
                });
                return;
            }
            box_primitive(BoxProps {
                children: Some(Box::new(move || {
                    create_nested(depth, current + 1);
                })),
                ..Default::default()
            });
        }

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            create_nested(depth, 0);
        }));

        let elapsed = start.elapsed();

        match result {
            Ok(_) => {
                println!("    {:>5} levels deep: {:>8.2?}", depth, elapsed);
            }
            Err(_) => {
                println!("    {:>5} levels deep: STACK OVERFLOW!", depth);
                break;
            }
        }
    }

    println!();

    // Test 3: Rapid signal updates (simulating scroll/animation)
    println!("  Test 3: Rapid-fire signal updates (animation simulation)...");
    reset_registry();

    let signals: Vec<_> = (0..100).map(|i| signal(i as f32)).collect();

    // Create components bound to signals
    let signals_clone = signals.clone();
    let _root = box_primitive(BoxProps {
        children: Some(Box::new(move || {
            for sig in signals_clone.iter() {
                let sig = sig.clone();
                text(TextProps {
                    content: PropValue::Getter(std::rc::Rc::new(move || {
                        format!("{:.2}", sig.get())
                    })),
                    ..Default::default()
                });
            }
        })),
        ..Default::default()
    });

    let layout_derived = create_layout_derived();
    let fb_derived = create_frame_buffer_derived(layout_derived);

    // Simulate 60fps animation for 10 seconds worth of frames
    let frames = 600; // 10 seconds at 60fps
    let start = Instant::now();

    for frame in 0..frames {
        // Update all signals (simulating animation)
        let t = frame as f32 / 60.0;
        for (i, sig) in signals.iter().enumerate() {
            sig.set((t * (i + 1) as f32).sin() * 100.0);
        }
        let _ = fb_derived.get();
    }

    let elapsed = start.elapsed();
    let actual_fps = frames as f64 / elapsed.as_secs_f64();

    println!(
        "    600 frames (10s @ 60fps): {:?} (actual: {:.0} FPS)",
        elapsed, actual_fps
    );
    if actual_fps >= 60.0 {
        println!("    Result: SMOOTH - could render faster than 60fps!");
    } else if actual_fps >= 30.0 {
        println!("    Result: Acceptable - above 30fps");
    } else {
        println!("    Result: Would feel laggy");
    }

    println!();

    // Test 4: Large terminal size
    println!("  Test 4: Massive terminal sizes...");
    for (w, h) in [(400, 100), (800, 200), (1920, 1080), (3840, 2160)] {
        reset_registry();
        set_terminal_size(w, h);

        // Create enough boxes to fill it
        let cols = w / 4;
        let rows = h / 2;

        let _root = box_primitive(BoxProps {
            width: Some(PropValue::Static(Dimension::Percent(100.0))),
            height: Some(PropValue::Static(Dimension::Percent(100.0))),
            children: Some(Box::new(move || {
                for _ in 0..rows {
                    box_primitive(BoxProps {
                        flex_direction: Some(PropValue::Static(1)),
                        children: Some(Box::new(move || {
                            for _ in 0..cols {
                                text(TextProps {
                                    content: PropValue::Static("X".to_string()),
                                    ..Default::default()
                                });
                            }
                        })),
                        ..Default::default()
                    });
                }
            })),
            ..Default::default()
        });

        let layout_derived = create_layout_derived();
        let fb_derived = create_frame_buffer_derived(layout_derived);

        let start = Instant::now();
        let _ = fb_derived.get();
        let elapsed = start.elapsed();

        let cells = w as u32 * h as u32;
        let fps = 1.0 / elapsed.as_secs_f64();

        println!(
            "    {}x{} ({} cells): {:>8.2?} ({:.0} FPS)",
            w, h, cells, elapsed, fps
        );
    }
}
