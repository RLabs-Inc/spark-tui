# Product Guidelines: spark-tui

## Developer Experience (DX)
- **Concise & Magical:** Prioritize a TypeScript-like developer experience. Use macros to minimize boilerplate and make reactive state management feel "invisible" and automatic.
- **Escape Hatches:** Always provide low-level access to Rust primitives for power users who need performance tuning or custom integration.
- **Helpful Errors:** Error messages should be conversational and actionable, guiding the developer toward a solution rather than just stating a fault.

## Visual & Design Philosophy
- **Modern Hyper-Style:** Aim for interfaces that rival modern desktop applications. Use full RGB/OKLCH color support, refined borders, and polished layouts.
- **Mathematical Harmony:** Incorporate "Sacred Geometry" (Golden Ratio, Merkaba fields) into the core layout and theme generation to ensure visual balance.
- **Respectful Defaults:** By default, respect the user's terminal settings and ANSI color palette while offering easy paths to "Level Up" to high-fidelity colors.

## Stability & Resilience
- **Fail Gracefully:** In production, prioritize UI availability. If a non-critical component fails to render, log the error and maintain the rest of the interface.
- **Developer Inspection:** Include built-in debugging tools that allow developers to inspect the reactive tree and layout nodes directly in the terminal during development.

## Documentation Strategy
- **Dual-Track Docs:** Maintain high-quality technical references (API docs) alongside narrative "How-To" guides and tutorials for common use cases.
- **Live Examples:** Prioritize runnable code examples that demonstrate the power of the reactive primitives in real-world scenarios.
