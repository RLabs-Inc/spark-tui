# Tech Stack: spark-tui

## Language & Tools
- **Rust:** Stable (Edition 2024).
- **Cargo:** Dependency management and build tool.

## Core Reactive Engine
- **spark-signals:** Production-ready signals library providing the foundation for total reactivity.
- **ECS (Parallel Arrays):** Memory-efficient entity-component-system pattern for storing and processing UI components.

## UI & Layout
- **Taffy:** A robust CSS Flexbox layout engine used to compute component positions and sizes.
- **Titan:** Our internal layout bridge that connects the reactive Slot system to Taffy's flexbox algorithm.

## Terminal Backend
- **Crossterm:** The cross-platform terminal library used for handling events, cursor movement, and raw output.

## Rendering Pipeline
- **Derived-based Pipeline:** A system where visual and layout changes are calculated through a series of pure derived signals.
- **Multi-Mode Rendering:** Support for Fullscreen (diff-based), Inline (incremental), and Append modes.
