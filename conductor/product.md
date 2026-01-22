# Product Definition: spark-tui

## Initial Concept
D all options above and anyone who might want/need to build a cli or a tui with the speed of rust, complete set of reactive primitives and typescript-like ergonomics

## Vision
A high-performance, fully reactive terminal UI framework for Rust that prioritizes developer experience (DX) by providing TypeScript-like ergonomics on top of a memory-efficient ECS architecture.

## Target Audience
- Rust developers building complex, interactive CLI tools.
- System administrators needing declarative dashboards.
- Game developers exploring 2D terminal engines.
- Anyone moving from TypeScript to Rust who misses reactive primitives.

## Core Pillars (The "North Star")
1. **Total Reactivity:** A pipeline built on `spark-signals` where changing any prop in a primitive triggers a surgical re-render of only what changed. No fixed FPS, no manual dirty tracking.
2. **TS-Like Ergonomics:** A high-level macro API that mimics the pure functional style of the original TypeScript implementation, alongside a standard Rust builder pattern.
3. **Reactive Layout:** A full CSS Flexbox implementation via Taffy, tightly integrated into the reactive slot system.
4. **Vim-Grade Input:** A sophisticated input system that supports selection, clipboard integration, and multi-line editing with Vim-inspired navigation hooks.

## The Theme System
- **Semantic & Mathematical:** Combines standard semantic colors (primary, secondary, accent) with "Sacred Geometry" theme generation (Golden Ratio, Merkaba Fields).
- **Fully Reactive:** Theme changes propagate through the component tree automatically.
- **Terminal Respect:** Default presets honor the current terminal's ANSI colors while supporting modern OKLCH color spaces.

## Technical Foundation
- **Language:** Rust (Stable, 2024 Edition).
- **State Management:** `spark-signals` (ECS-style parallel arrays).
- **Layout:** Taffy (Flexbox).
- **Backend:** Crossterm.
