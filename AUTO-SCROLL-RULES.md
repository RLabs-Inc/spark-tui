# Auto-Scroll Detection Rules

## The Core Rule

**When children computed size > parent computed size → parent becomes scrollable AND focusable**

This is computed AFTER Taffy layout, using the final computed dimensions.

## Key Points

### 1. Only Containers Can Scroll

- Text nodes have no children → never scrollable
- Box nodes can have children → can become scrollable
- Any future component with children → same rule applies

### 2. Use Computed Sizes (Never Miss)

The check uses COMPUTED sizes from Taffy layout output:
- `layout.content_size` = total size needed by children
- `layout.size` = actual rendered size of container

If `content_size > size` → overflow exists → scrollable.

This catches ALL cases:
- Explicit height that's too small
- Percentage height that resolves to too small
- Auto height constrained by parent
- Any constraint that results in overflow

### 3. Scrollable = Focusable

When a node becomes scrollable, it ALSO becomes focusable automatically.
- Tab navigation includes scrollable nodes
- Keyboard arrows scroll the focused container
- Mouse scroll works on hovered containers

### 4. Overflow Prop Behavior

- `overflow: visible` (0, default) → auto-scroll when content overflows
- `overflow: hidden` (1) → OPT-OUT, content clipped, no scroll
- `overflow: scroll` (2) → always scrollable (even if no overflow yet)
- `overflow: auto` (3) → scrollable only when content overflows

### 5. Nested Scroll is Independent

Each scrollable container has its OWN scroll state:
- Parent scroll affects where child CONTAINER is positioned
- Child scroll affects where child CONTENT is positioned
- They are independent - scrolling parent doesn't reset child's scroll
- Scrolling child doesn't affect parent's scroll

### 6. Root Element

Same rules apply to root:
- If root's children computed size > root's computed size → root scrollable
- Root with `height: 100%` and lots of content → scrollable

## Implementation Location

The detection happens in `rust/src/layout/layout_tree_aos.rs` in `write_output()`:
1. After Taffy computes layout
2. For each node with children
3. Compare `content_size` vs `size`
4. Call `buf.set_output_scroll(idx, is_scrollable, max_scroll_x, max_scroll_y)`

## Focus Integration

In `rust/src/input/focus.rs`:
- `get_focusable_list()` already includes `output_scrollable()` nodes
- `focus()` must ALSO accept `output_scrollable()` nodes (not just explicit `focusable`)

## What NOT to Do

- Don't check declared dimensions (height prop) - use computed
- Don't make text nodes scrollable
- Don't accumulate scroll offsets incorrectly in nested scroll
- Don't forget to set focusable when setting scrollable
