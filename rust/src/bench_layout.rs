//! Layout Benchmark - Compare AoS (256 bytes) vs New (512 bytes) buffer layouts
//!
//! Run with: cargo test --lib bench_layout --release -- --nocapture

use std::ptr;
use std::time::Instant;

use crate::layout::{compute_layout, compute_layout_aos};
use crate::shared_buffer::{
    SharedBuffer, HEADER_SIZE, NODE_STRIDE, DEFAULT_TEXT_POOL_SIZE, EVENT_RING_SIZE,
    COMPONENT_BOX, COMPONENT_TEXT,
    F_WIDTH, F_HEIGHT, F_FLEX_GROW,
    U_FLEX_DIRECTION, U_COMPONENT_TYPE, U_VISIBLE,
    I_PARENT_INDEX,
    H_VERSION, H_NODE_COUNT, H_MAX_NODES, H_TEXT_POOL_SIZE, H_TERMINAL_WIDTH, H_TERMINAL_HEIGHT,
};
use crate::shared_buffer_aos::{
    AoSBuffer,
    HEADER_SIZE as AOS_HEADER_SIZE, STRIDE as AOS_STRIDE, TEXT_POOL_SIZE as AOS_TEXT_POOL_SIZE,
    EVENT_RING_SIZE as AOS_EVENT_RING_SIZE,
    COMPONENT_BOX as AOS_COMPONENT_BOX, COMPONENT_TEXT as AOS_COMPONENT_TEXT,
    F_WIDTH as AOS_F_WIDTH, F_HEIGHT as AOS_F_HEIGHT, F_FLEX_GROW as AOS_F_FLEX_GROW,
    U_FLEX_DIRECTION as AOS_U_FLEX_DIRECTION, U_COMPONENT_TYPE as AOS_U_COMPONENT_TYPE, U_VISIBLE as AOS_U_VISIBLE,
    I_PARENT_INDEX as AOS_I_PARENT_INDEX,
    H_VERSION as AOS_H_VERSION, H_NODE_COUNT as AOS_H_NODE_COUNT, H_MAX_NODES as AOS_H_MAX_NODES,
    H_TERMINAL_WIDTH as AOS_H_TERMINAL_WIDTH, H_TERMINAL_HEIGHT as AOS_H_TERMINAL_HEIGHT,
};

const BENCH_NODES: usize = 1000;
const BENCH_ITERATIONS: usize = 1000;
const TERMINAL_WIDTH: u32 = 120;
const TERMINAL_HEIGHT: u32 = 40;

/// Create and populate the OLD buffer (256 bytes/node)
fn create_aos_buffer(node_count: usize) -> (Vec<u8>, AoSBuffer) {
    let total_size = AOS_HEADER_SIZE + node_count * AOS_STRIDE + AOS_TEXT_POOL_SIZE + AOS_EVENT_RING_SIZE;
    let mut data = vec![0u8; total_size];
    let ptr = data.as_mut_ptr();

    unsafe {
        // Header
        ptr::write_unaligned(ptr.add(AOS_H_VERSION) as *mut u32, 1);
        ptr::write_unaligned(ptr.add(AOS_H_NODE_COUNT) as *mut u32, node_count as u32);
        ptr::write_unaligned(ptr.add(AOS_H_MAX_NODES) as *mut u32, node_count as u32);
        ptr::write_unaligned(ptr.add(AOS_H_TERMINAL_WIDTH) as *mut u32, TERMINAL_WIDTH);
        ptr::write_unaligned(ptr.add(AOS_H_TERMINAL_HEIGHT) as *mut u32, TERMINAL_HEIGHT);

        // Create a tree: root -> 10 containers -> 100 children each
        // Node 0: root
        let root = AOS_HEADER_SIZE;
        ptr::write_unaligned(ptr.add(root + AOS_F_WIDTH) as *mut f32, f32::NAN); // auto
        ptr::write_unaligned(ptr.add(root + AOS_F_HEIGHT) as *mut f32, f32::NAN); // auto
        *ptr.add(root + AOS_U_FLEX_DIRECTION) = 1; // Column
        *ptr.add(root + AOS_U_COMPONENT_TYPE) = AOS_COMPONENT_BOX;
        *ptr.add(root + AOS_U_VISIBLE) = 1;
        ptr::write_unaligned(ptr.add(root + AOS_I_PARENT_INDEX) as *mut i32, -1);

        // 10 containers at indices 1-10
        for i in 1..=10 {
            let node = AOS_HEADER_SIZE + i * AOS_STRIDE;
            ptr::write_unaligned(ptr.add(node + AOS_F_WIDTH) as *mut f32, f32::NAN);
            ptr::write_unaligned(ptr.add(node + AOS_F_HEIGHT) as *mut f32, f32::NAN);
            ptr::write_unaligned(ptr.add(node + AOS_F_FLEX_GROW) as *mut f32, 1.0);
            *ptr.add(node + AOS_U_FLEX_DIRECTION) = 0; // Row
            *ptr.add(node + AOS_U_COMPONENT_TYPE) = AOS_COMPONENT_BOX;
            *ptr.add(node + AOS_U_VISIBLE) = 1;
            ptr::write_unaligned(ptr.add(node + AOS_I_PARENT_INDEX) as *mut i32, 0);
        }

        // Remaining nodes as children of containers
        for i in 11..node_count {
            let parent = 1 + ((i - 11) % 10); // Distribute among containers
            let node = AOS_HEADER_SIZE + i * AOS_STRIDE;
            ptr::write_unaligned(ptr.add(node + AOS_F_WIDTH) as *mut f32, 10.0);
            ptr::write_unaligned(ptr.add(node + AOS_F_HEIGHT) as *mut f32, 1.0);
            *ptr.add(node + AOS_U_COMPONENT_TYPE) = AOS_COMPONENT_TEXT;
            *ptr.add(node + AOS_U_VISIBLE) = 1;
            ptr::write_unaligned(ptr.add(node + AOS_I_PARENT_INDEX) as *mut i32, parent as i32);
        }
    }

    let buf = unsafe { AoSBuffer::from_raw(ptr, total_size) };
    (data, buf)
}

/// Create and populate the NEW buffer (512 bytes/node)
fn create_new_buffer(node_count: usize) -> (Vec<u8>, SharedBuffer) {
    let text_pool_size = DEFAULT_TEXT_POOL_SIZE;
    let text_pool_offset = HEADER_SIZE + node_count * NODE_STRIDE;
    let event_ring_offset = text_pool_offset + text_pool_size;
    let total_size = event_ring_offset + EVENT_RING_SIZE;

    let mut data = vec![0u8; total_size];
    let ptr = data.as_mut_ptr();

    unsafe {
        // Header
        ptr::write_unaligned(ptr.add(H_VERSION) as *mut u32, 2);
        ptr::write_unaligned(ptr.add(H_NODE_COUNT) as *mut u32, node_count as u32);
        ptr::write_unaligned(ptr.add(H_MAX_NODES) as *mut u32, node_count as u32);
        ptr::write_unaligned(ptr.add(H_TEXT_POOL_SIZE) as *mut u32, text_pool_size as u32);
        ptr::write_unaligned(ptr.add(H_TERMINAL_WIDTH) as *mut u32, TERMINAL_WIDTH);
        ptr::write_unaligned(ptr.add(H_TERMINAL_HEIGHT) as *mut u32, TERMINAL_HEIGHT);

        // Create same tree structure
        // Node 0: root
        let root = HEADER_SIZE;
        ptr::write_unaligned(ptr.add(root + F_WIDTH) as *mut f32, f32::NAN);
        ptr::write_unaligned(ptr.add(root + F_HEIGHT) as *mut f32, f32::NAN);
        *ptr.add(root + U_FLEX_DIRECTION) = 1; // Column
        *ptr.add(root + U_COMPONENT_TYPE) = COMPONENT_BOX;
        *ptr.add(root + U_VISIBLE) = 1;
        ptr::write_unaligned(ptr.add(root + I_PARENT_INDEX) as *mut i32, -1);

        // 10 containers
        for i in 1..=10 {
            let node = HEADER_SIZE + i * NODE_STRIDE;
            ptr::write_unaligned(ptr.add(node + F_WIDTH) as *mut f32, f32::NAN);
            ptr::write_unaligned(ptr.add(node + F_HEIGHT) as *mut f32, f32::NAN);
            ptr::write_unaligned(ptr.add(node + F_FLEX_GROW) as *mut f32, 1.0);
            *ptr.add(node + U_FLEX_DIRECTION) = 0; // Row
            *ptr.add(node + U_COMPONENT_TYPE) = COMPONENT_BOX;
            *ptr.add(node + U_VISIBLE) = 1;
            ptr::write_unaligned(ptr.add(node + I_PARENT_INDEX) as *mut i32, 0);
        }

        // Remaining nodes
        for i in 11..node_count {
            let parent = 1 + ((i - 11) % 10);
            let node = HEADER_SIZE + i * NODE_STRIDE;
            ptr::write_unaligned(ptr.add(node + F_WIDTH) as *mut f32, 10.0);
            ptr::write_unaligned(ptr.add(node + F_HEIGHT) as *mut f32, 1.0);
            *ptr.add(node + U_COMPONENT_TYPE) = COMPONENT_TEXT;
            *ptr.add(node + U_VISIBLE) = 1;
            ptr::write_unaligned(ptr.add(node + I_PARENT_INDEX) as *mut i32, parent as i32);
        }
    }

    let buf = unsafe { SharedBuffer::from_raw(ptr, total_size) };
    (data, buf)
}

#[test]
fn bench_layout_comparison() {
    println!("\n========================================");
    println!("Layout Benchmark: {} nodes, {} iterations", BENCH_NODES, BENCH_ITERATIONS);
    println!("========================================\n");

    // Create buffers
    let (_aos_data, aos_buf) = create_aos_buffer(BENCH_NODES);
    let (_new_data, new_buf) = create_new_buffer(BENCH_NODES);

    // Warm up
    for _ in 0..10 {
        compute_layout_aos(&aos_buf);
        compute_layout(&new_buf);
    }

    // Benchmark OLD (256 bytes/node)
    let start = Instant::now();
    for _ in 0..BENCH_ITERATIONS {
        compute_layout_aos(&aos_buf);
    }
    let aos_elapsed = start.elapsed();
    let aos_per_iter = aos_elapsed.as_nanos() as f64 / BENCH_ITERATIONS as f64;

    // Benchmark NEW (512 bytes/node)
    let start = Instant::now();
    for _ in 0..BENCH_ITERATIONS {
        compute_layout(&new_buf);
    }
    let new_elapsed = start.elapsed();
    let new_per_iter = new_elapsed.as_nanos() as f64 / BENCH_ITERATIONS as f64;

    // Results
    println!("OLD (256 bytes/node):");
    println!("  Total: {:?}", aos_elapsed);
    println!("  Per iteration: {:.2} µs", aos_per_iter / 1000.0);
    println!("  Buffer size: {} KB", (_aos_data.len() as f64 / 1024.0) as u32);
    println!();
    println!("NEW (512 bytes/node):");
    println!("  Total: {:?}", new_elapsed);
    println!("  Per iteration: {:.2} µs", new_per_iter / 1000.0);
    println!("  Buffer size: {} KB", (_new_data.len() as f64 / 1024.0) as u32);
    println!();

    let ratio = new_per_iter / aos_per_iter;
    if ratio > 1.0 {
        println!("Result: NEW is {:.2}x SLOWER than OLD", ratio);
    } else {
        println!("Result: NEW is {:.2}x FASTER than OLD", 1.0 / ratio);
    }
    println!("========================================\n");
}
