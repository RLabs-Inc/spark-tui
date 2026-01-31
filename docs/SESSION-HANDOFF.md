# Session Handoff — SparkTUI Rust Rewrite

**Date:** January 31, 2026
**Phase:** Clean layout_tree.rs rewrite — 95% complete
**Blocker:** EmptyLineNames lifetime issue (Grid support)

---

## What We Did This Session

1. **Studied Taffy 0.9 API completely** via 5-agent reconnaissance squad:
   - Trait Scout: All 6 core traits documented
   - Style Scout: CoreStyle 15 methods (all have defaults)
   - Grid Scout: TemplateLineNames trait requirements
   - Compute Scout: 4-param compute_leaf_layout signature
   - Example Scout: Implementation patterns

2. **Wrote layout_tree.rs from scratch** (clean, no workarounds):
   - Zero-copy NodeStyle wrapper
   - All Taffy 0.9 traits implemented
   - 4-param compute_leaf_layout with resolve_calc_value
   - Dirty flag integration with cache
   - Grid support (EmptyLineNames needs lifetime fix)

3. **Added box_sizing() getter** to shared_buffer.rs

---

## The One Remaining Issue

**EmptyLineNames lifetime mismatch** in Grid support.

Taffy's TemplateLineNames trait has complex lifetime requirements:
```rust
pub trait TemplateLineNames<'a, S: CheapCloneStr> :
    Iterator<Item = Self::LineNameSet<'a>> + ExactSizeIterator + Clone
{
    type LineNameSet<'b>: Iterator<Item = &'b S> + ExactSizeIterator + Clone;
}
```

Taffy only implements this for a specific Map iterator type:
```rust
impl<'a, S: CheapCloneStr> TemplateLineNames<'a, S>
    for core::iter::Map<core::slice::Iter<'a, Vec<S>>, fn(&Vec<S>) -> core::slice::Iter<'_, S>>
```

**Options to fix:**
1. Match Taffy's exact type with a static empty slice
2. Create a custom wrapper that satisfies the GAT requirements
3. Skip grid support for now (return errors from grid methods)

**Note:** We return `None` from `grid_template_*_names()`, so the type never needs to be constructed at runtime. It just needs to satisfy the compiler.

---

## Files Changed

```
rust/src/layout/layout_tree.rs     — NEW clean rewrite (needs EmptyLineNames fix)
rust/src/layout/layout_tree.rs.old — Previous patched version (backup)
rust/src/layout/layout_tree.rs.bak — Original 0.7 version (backup)
rust/src/shared_buffer.rs          — Added box_sizing() getter at line 1533
```

---

## Key Insights From Reconnaissance

### Taffy 0.9 Trait Requirements

| Trait | Required Methods |
|-------|-----------------|
| TraversePartialTree | child_ids(), child_count(), get_child_id() |
| TraverseTree | (marker trait, empty) |
| LayoutPartialTree | get_core_container_style(), set_unrounded_layout(), compute_child_layout() |
| CacheTree | cache_get(), cache_store(), cache_clear() |
| RoundTree | get_unrounded_layout(), set_final_layout() |
| PrintTree | get_debug_label(), get_final_layout() |
| LayoutFlexboxContainer | get_flexbox_container_style(), get_flexbox_child_style() |
| LayoutGridContainer | get_grid_container_style(), get_grid_child_style() |

### compute_leaf_layout Signature (NEW in 0.9)
```rust
compute_leaf_layout(
    inputs: LayoutInput,
    style: &impl CoreStyle,
    resolve_calc_value: impl Fn(*const (), f32) -> f32,  // NEW! Pass |_, _| 0.0
    measure_function: MeasureFunction,
) -> LayoutOutput
```

### Dimension Convention
- `100.0` = 100 cells (length)
- `-50.0` = 50% (percent)
- `NaN` = auto

---

## Reactive Cache Integration

We discovered spark-signals has exactly what we need:

1. **SharedSlotBuffer** — has per-index dirty flags built in
2. **Repeater** — inline forwarding (zero scheduling)
3. **Notifier** — cross-side wake via futex

**Cache flow:**
```
Signal changes → dirty flag set → layout derived runs →
cache_clear() only for dirty nodes → Taffy runs with full caching →
unchanged nodes use cache, changed nodes recompute
```

---

## Next Steps

1. **Fix EmptyLineNames** — resolve the GAT lifetime issue
2. **Verify layout_tree.rs compiles** — ignore AoSBuffer errors (those files are next)
3. **Continue reactive flow rewrite:** lib.rs → pipeline/setup.rs → framebuffer → renderer

---

## Architecture Reminders

- **AoSBuffer** = OLD (256 bytes) — being replaced
- **SharedBuffer** = NEW v3.0 (1024 bytes) — target for all files
- **Rewrite order:** Follow reactive data flow, file by file
- **No workarounds:** Clean, production-grade code only

---

*Sherlock & Watson — 95% there, one lifetime puzzle remaining*
