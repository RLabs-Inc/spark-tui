# Memory Leak Investigation Report

**Date**: 2026-02-01
**Issue**: Catastrophic memory growth during benchmarks (18GB → 86GB)

## Symptoms

1. Running `bun examples/bench.ts` causes memory to grow from ~20MB to 86GB
2. Memory continues growing even AFTER update loops stop (during `await sleep()`)
3. `process.exit(0)` doesn't terminate the process - requires Ctrl+C
4. Memory IS released when process is killed (not a permanent leak)

## What We Ruled Out

### JavaScript Heap
- `process.memoryUsage().heapUsed` shows only 22-23MB throughout
- The leak is NOT in JS heap

### SharedArrayBuffer
- Only ONE buffer created (20MB) with single-mount pattern
- Original benchmark had multiple mounts creating multiple buffers - FIXED
- But leak persists even with single mount

### Derived Value Storage
- `DerivedInner.value` uses `RefCell<Option<T>>`
- `compute()` replaces value with `*self.value.borrow_mut() = Some(new_value)`
- Old value should be dropped - looks correct

### Renderer Previous Buffer
- `DiffRenderer.previous = Some(buffer.clone())` replaces, not accumulates
- Only stores ONE previous buffer - looks correct

## Prime Suspects

### 1. spark-signals Rust Crate
Location: `/Users/rusty/Documents/Projects/TUI/tui-rust/crates/spark-signals/src/`

**Scheduling System** (`reactivity/scheduling.rs`):
- `pending_reactions: Vec<Weak<dyn AnyReaction>>` - queue of pending reactions
- `queued_root_effects: Vec<Weak<dyn AnyReaction>>` - queue of root effects
- These use Weak refs but vectors could grow if not properly drained

**Mark Reactions** (`reactivity/tracking.rs:123`):
- `mark_reactions()` creates temporary vectors each call
- Pushes to stack, collects reactions, schedules effects
- Each signal.set() triggers this

**Effect Scheduling** (`reactivity/tracking.rs:188`):
- `schedule_effect()` adds to pending queue
- `flush_pending_effects()` should drain it
- If flush doesn't complete, queue grows

### 2. Effect Tree Structure
`EffectInner` has parent/child/sibling structure:
```rust
first_child: RefCell<Option<Rc<EffectInner>>>
next_sibling: RefCell<Option<Rc<EffectInner>>>
```
If effects create children that aren't cleaned up, tree grows.

### 3. Reaction References
Each source keeps `reactions: RefCell<Vec<Weak<dyn AnyReaction>>>`
`cleanup_dead_reactions()` is called in `mark_reactions()` but may not be thorough.

## The Smoking Gun Clue

**Memory grows during `await sleep(2000)` AFTER all updates stopped.**

This means something is still running/allocating after our code stops updating signals.
Possible causes:
- Rust threads still running (stdin reader, wake watcher, resize watcher)
- Reactive graph still processing queued effects
- Some infinite loop triggered by our final state

## Files to Investigate

1. **spark-signals scheduling**:
   - `tui-rust/crates/spark-signals/src/reactivity/scheduling.rs`
   - Check `flush_pending_effects()` and `flush_sync()`

2. **spark-signals effect execution**:
   - `tui-rust/crates/spark-signals/src/primitives/effect.rs`
   - Check `execute()` method and cleanup

3. **Engine threads**:
   - `SparkTUI/rust/src/input/reader.rs` - stdin reader, resize watcher
   - `SparkTUI/rust/src/pipeline/wake.rs` - wake watcher
   - Check if threads are spinning or leaking

4. **FrameBuffer allocation**:
   - `SparkTUI/rust/src/framebuffer/render_tree.rs`
   - `compute_framebuffer()` creates FrameBuffer + Vec<HitRegion> each frame

## Reproduction

Minimal test case:
```bash
bun examples/mem-leak-test.ts
```

Watch Activity Monitor → bun process → Memory column.
Memory should stay ~stable but grows continuously.

## Next Steps

1. Add memory profiling to Rust side (track allocations)
2. Instrument spark-signals to log queue sizes
3. Check if `flush_pending_effects()` ever completes
4. Verify all Rust threads terminate on stop()
5. Consider using Instruments.app or heaptrack for Rust memory profiling

## Commits Made This Session

1. `3dab208` - feat: clean mount API + responsive terminal resize
2. `4bbeac6` - feat: add comprehensive benchmarks with single-mount pattern

## Test Files Created

- `examples/bench.ts` - comprehensive benchmarks (has the leak)
- `examples/mem-test.ts` - JS heap diagnostic (shows no JS leak)
- `examples/mem-leak-test.ts` - minimal reproduction case
