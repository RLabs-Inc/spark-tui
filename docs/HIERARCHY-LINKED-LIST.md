# SparkTUI O(1) Hierarchy Implementation

**Status**: Ready for implementation
**Priority**: Core infrastructure
**Date**: January 31, 2026
**Authors**: Rusty & Claude

---

## Problem Statement

Current child lookup is O(N) - we scan ALL allocated nodes to find children of ONE parent:

```typescript
// registry.ts releaseIndex() - SLOW
for (const childIndex of allocatedIndices) {
  const parentIdx = arrays.parentIndex.get(childIndex)
  if (parentIdx === index) {
    children.push(childIndex)  // O(N) scan!
  }
}
```

```rust
// render_tree.rs compute_framebuffer() - SLOW
for i in 0..node_count {
  if let Some(parent) = buf.parent_index(i) {
    child_map[parent].push(i);  // O(N) scan!
  }
}
```

For 50,000 nodes, this is brutal. We need O(1) child discovery.

---

## Solution: Doubly-Linked Sibling List

Add three fields per node:

| Field | Type | Description |
|-------|------|-------------|
| `first_child` | i32 | Index of first child (-1 = no children) |
| `prev_sibling` | i32 | Previous sibling (-1 = first child) |
| `next_sibling` | i32 | Next sibling (-1 = last child) |

### Operations

**Add child (prepend, O(1)):**
```
new_child.next_sibling = parent.first_child
new_child.prev_sibling = -1
if parent.first_child >= 0:
    old_first.prev_sibling = new_child
parent.first_child = new_child
```

**Remove child (O(1)):**
```
if child.prev_sibling >= 0:
    prev.next_sibling = child.next_sibling
else:
    parent.first_child = child.next_sibling

if child.next_sibling >= 0:
    next.prev_sibling = child.prev_sibling
```

**Iterate children (O(children)):**
```
child = parent.first_child
while child >= 0:
    yield child
    child = child.next_sibling
```

### Why Order Doesn't Matter

Z-index determines render order, not insertion order. The framebuffer already sorts children by z-index before rendering.

---

## Buffer Layout Change

### Spec Update (SHARED-BUFFER-SPEC.md)

In **Line 4 (192-255): Grid Container Properties**, use reserved bytes:

| Offset | Size | Type | Name | Default | Description |
|--------|------|------|------|---------|-------------|
| 220 | 4 | i32 | `first_child` | -1 | First child index (-1 = none) |
| 224 | 4 | i32 | `prev_sibling` | -1 | Previous sibling (-1 = first) |
| 228 | 4 | i32 | `next_sibling` | -1 | Next sibling (-1 = last) |

These use 12 of the 39 reserved bytes (217-255).

---

## Implementation Tasks

### Phase 1: Spec & Constants ✅ DONE

- [x] **1.1** Update `docs/SHARED-BUFFER-SPEC.md`
  - Add hierarchy fields table to Line 4 section
  - Update reserved bytes count (39 → 27)
  - Add "Hierarchy Management" section explaining linked list

- [x] **1.2** Update `rust/src/shared_buffer.rs`
  - Add constants:
    ```rust
    pub const N_FIRST_CHILD: usize = 220;
    pub const N_PREV_SIBLING: usize = 224;
    pub const N_NEXT_SIBLING: usize = 228;
    ```
  - Add accessors:
    ```rust
    pub fn first_child(&self, i: usize) -> i32
    pub fn prev_sibling(&self, i: usize) -> i32
    pub fn next_sibling(&self, i: usize) -> i32
    pub fn set_first_child(&self, i: usize, v: i32)
    pub fn set_prev_sibling(&self, i: usize, v: i32)
    pub fn set_next_sibling(&self, i: usize, v: i32)
    ```
  - Add helper:
    ```rust
    pub fn iter_children(&self, parent: usize) -> impl Iterator<Item = usize>
    ```

- [x] **1.3** Update `ts/bridge/shared-buffer.ts`
  - Add constants:
    ```typescript
    export const N_FIRST_CHILD = 220;
    export const N_PREV_SIBLING = 224;
    export const N_NEXT_SIBLING = 228;
    ```
  - Add accessors (getFirstChild, setFirstChild, etc.)

- [x] **1.4** Update `ts/bridge/reactive-arrays.ts`
  - Add slot buffers:
    ```typescript
    firstChild: createSlotBuffer('i32', N_FIRST_CHILD),
    prevSibling: createSlotBuffer('i32', N_PREV_SIBLING),
    nextSibling: createSlotBuffer('i32', N_NEXT_SIBLING),
    ```

### Phase 2: TypeScript Registry ✅ DONE

- [x] **2.1** Update `ts/engine/registry.ts`
  - Add `registerParent(childIndex: number, parentIndex: number)`:
    ```typescript
    export function registerParent(childIndex: number, parentIndex: number): void {
      const arrays = getArrays()

      // Remove from old parent if reparenting
      const oldParent = arrays.parentIndex.get(childIndex)
      if (oldParent >= 0 && oldParent !== parentIndex) {
        unlinkChild(childIndex, oldParent)
      }

      // Link to new parent
      if (parentIndex >= 0) {
        linkChild(childIndex, parentIndex)
      }
    }
    ```

  - Add `linkChild(child, parent)` helper:
    ```typescript
    function linkChild(child: number, parent: number): void {
      const arrays = getArrays()
      const oldFirst = arrays.firstChild.get(parent)

      // Prepend: new child becomes first
      arrays.nextSibling.set(child, oldFirst)
      arrays.prevSibling.set(child, -1)

      if (oldFirst >= 0) {
        arrays.prevSibling.set(oldFirst, child)
      }

      arrays.firstChild.set(parent, child)
    }
    ```

  - Add `unlinkChild(child, parent)` helper:
    ```typescript
    function unlinkChild(child: number, parent: number): void {
      const arrays = getArrays()
      const prev = arrays.prevSibling.get(child)
      const next = arrays.nextSibling.get(child)

      // Update previous sibling or parent's first_child
      if (prev >= 0) {
        arrays.nextSibling.set(prev, next)
      } else {
        arrays.firstChild.set(parent, next)
      }

      // Update next sibling
      if (next >= 0) {
        arrays.prevSibling.set(next, prev)
      }

      // Clear own links
      arrays.prevSibling.set(child, -1)
      arrays.nextSibling.set(child, -1)
    }
    ```

  - Add `getChildren(parent: number): number[]` helper:
    ```typescript
    export function getChildren(parentIndex: number): number[] {
      const arrays = getArrays()
      const children: number[] = []
      let child = arrays.firstChild.get(parentIndex)
      while (child >= 0) {
        children.push(child)
        child = arrays.nextSibling.get(child)
      }
      return children
    }
    ```

  - Update `releaseIndex()` to use O(1) linked list:
    ```typescript
    // OLD (O(N)):
    for (const childIndex of allocatedIndices) { ... }

    // NEW (O(children)):
    const children = getChildren(index)
    for (const child of children) {
      releaseIndex(child)
    }
    ```

  - Update `resetRegistry()` to clear hierarchy:
    ```typescript
    // Clear all linked list state (done automatically since nodes reset to -1)
    ```

### Phase 3: TypeScript Primitives ⏳ BLOCKED

**BLOCKED**: Primitives still use old 256-byte `shared-buffer-aos.ts`. Need full rewrite to new 1024-byte `shared-buffer.ts` with Grid props. Do in next session.

- [ ] **3.1** Update `ts/primitives/box.ts`
  - After writing parentIndex, call registerParent:
    ```typescript
    const parentIdx = getCurrentParentIndex()
    arrays.parentIndex.set(index, parentIdx)
    registerParent(index, parentIdx)
    ```

- [ ] **3.2** Update `ts/primitives/text.ts`
  - Same pattern as box.ts

- [ ] **3.3** Update `ts/primitives/input.ts`
  - Same pattern as box.ts

### Phase 4: Rust Layout

- [ ] **4.1** Update `rust/src/layout/layout_tree.rs`
  - Replace `rebuild_hierarchy()` with linked list traversal:
    ```rust
    // OLD: Scan all nodes
    fn rebuild_hierarchy(&mut self, buf: &SharedBuffer) {
      for i in 0..node_count {
        if let Some(parent) = buf.parent_index(i) {
          self.children[parent].push(i);
        }
      }
    }

    // NEW: Just use buf.iter_children() in traversal
    fn children(&self, parent: usize) -> impl Iterator<Item = NodeId> {
      buf.iter_children(parent).map(NodeId::from)
    }
    ```

### Phase 5: Rust Framebuffer

- [ ] **5.1** Update `rust/src/framebuffer/render_tree.rs`
  - Replace child_map building with linked list:
    ```rust
    // OLD: O(N) scan
    for i in 0..node_count {
      if let Some(parent) = buf.parent_index(i) {
        child_map[parent].push(i);
      }
    }

    // NEW: O(children) per parent, called during DFS
    fn get_sorted_children(buf: &SharedBuffer, parent: usize) -> Vec<usize> {
      let mut children: Vec<usize> = buf.iter_children(parent).collect();
      children.sort_by_key(|&i| buf.z_index(i));
      children
    }
    ```

### Phase 6: Testing & Verification

- [ ] **6.1** Add Rust unit tests for linked list operations
  - Test add child, remove child, iterate children
  - Test reparenting
  - Test cleanup on release

- [ ] **6.2** Add TypeScript tests for registry
  - Test registerParent adds to parent's children
  - Test releaseIndex removes from siblings
  - Test getChildren returns correct order

- [ ] **6.3** Integration test
  - Create 1000 nodes with hierarchy
  - Verify releaseIndex is fast (< 1ms)
  - Verify render still works with z-index sorting

---

## Files Modified

| File | Changes |
|------|---------|
| `docs/SHARED-BUFFER-SPEC.md` | Add hierarchy fields |
| `rust/src/shared_buffer.rs` | Add offsets + accessors |
| `ts/bridge/shared-buffer.ts` | Add offsets + helpers |
| `ts/bridge/reactive-arrays.ts` | Add 3 slot buffers |
| `ts/engine/registry.ts` | Add registerParent, linkChild, unlinkChild, getChildren |
| `ts/primitives/box.ts` | Call registerParent after setting parentIndex |
| `ts/primitives/text.ts` | Call registerParent after setting parentIndex |
| `ts/primitives/input.ts` | Call registerParent after setting parentIndex |
| `rust/src/layout/layout_tree.rs` | Use iter_children instead of rebuild_hierarchy |
| `rust/src/framebuffer/render_tree.rs` | Use iter_children instead of child_map scan |

---

## Performance Impact

| Operation | Before | After |
|-----------|--------|-------|
| Find children of node | O(N) | O(children) |
| Release subtree | O(N × depth) | O(subtree) |
| Framebuffer child discovery | O(N) | O(children) |
| Layout hierarchy rebuild | O(N) | O(1) - just follow links |

For 50,000 nodes with average 5 children per parent:
- Before: 50,000 iterations per parent lookup
- After: 5 iterations per parent lookup
- **10,000x improvement**

---

## Execution Order

This implementation should be done in order:

1. **Spec first** - Single source of truth
2. **Rust shared_buffer.rs** - Low-level accessors
3. **TypeScript bridge** - Match Rust exactly
4. **TypeScript registry.ts** - Core logic
5. **TypeScript primitives** - Wire up registerParent calls
6. **Rust layout_tree.rs** - Use new accessors
7. **Rust render_tree.rs** - Use new accessors
8. **Tests** - Verify everything works

---

## Notes

- **Z-index sorting is still needed** - Linked list gives us children fast, but render order needs z-index sort. This is O(children × log(children)) per parent, which is fine.

- **Prepend vs append** - We prepend (O(1)) because order doesn't matter for rendering (z-index determines order).

- **Doubly-linked** - `prev_sibling` enables O(1) removal from middle of list.

- **Thread safety** - Both TS and Rust read/write the same memory. The reactive system ensures updates propagate correctly.

---

*This document is the implementation plan. Execute tasks in order, checking them off as complete.*
