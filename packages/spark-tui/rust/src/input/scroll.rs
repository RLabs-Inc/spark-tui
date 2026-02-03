//! Scroll management.
//!
//! Handles keyboard scroll (arrows, page, home/end), mouse wheel,
//! scroll-into-view, and scroll chaining (bubble to parent at boundary).

use crate::shared_buffer::SharedBuffer;

/// Scroll manager.
pub struct ScrollManager;

impl ScrollManager {
    pub fn new() -> Self {
        Self
    }

    /// Set absolute scroll offset, clamped to valid range.
    pub fn scroll_to(&self, buf: &SharedBuffer, index: usize, x: i32, y: i32) {
        if !buf.is_scrollable(index) {
            return;
        }

        let max_x = buf.max_scroll_x(index) as i32;
        let max_y = buf.max_scroll_y(index) as i32;

        let clamped_x = x.clamp(0, max_x.max(0));
        let clamped_y = y.clamp(0, max_y.max(0));

        buf.set_scroll(index, clamped_x, clamped_y);
    }

    /// Scroll by a delta, clamped to valid range.
    /// Returns true if scroll actually changed (false if at boundary).
    ///
    /// `allow_chain`: if true, scroll chains to parent when at boundary (mouse behavior).
    ///                if false, scroll stops at boundary (keyboard behavior).
    pub fn scroll_by(&self, buf: &SharedBuffer, index: usize, dx: i32, dy: i32, allow_chain: bool) -> bool {
        if !buf.is_scrollable(index) {
            // Try scroll chaining: walk up to find scrollable parent (only if allowed)
            if allow_chain {
                return self.try_chain_scroll(buf, index, dx, dy);
            }
            return false;
        }

        let current_x = buf.scroll_x(index);
        let current_y = buf.scroll_y(index);
        let max_x = buf.max_scroll_x(index) as i32;
        let max_y = buf.max_scroll_y(index) as i32;

        let new_x = (current_x + dx).clamp(0, max_x.max(0));
        let new_y = (current_y + dy).clamp(0, max_y.max(0));

        let changed = new_x != current_x || new_y != current_y;

        if changed {
            buf.set_scroll(index, new_x, new_y);
        } else if allow_chain {
            // At boundary: try chaining to parent (only for mouse scroll)
            return self.try_chain_scroll(buf, index, dx, dy);
        }

        changed
    }

    /// Walk up parent chain to find a scrollable parent and scroll it.
    fn try_chain_scroll(&self, buf: &SharedBuffer, index: usize, dx: i32, dy: i32) -> bool {
        let mut current = buf.parent_index(index);
        while let Some(parent_idx) = current {
            if buf.is_scrollable(parent_idx) {
                let curr_x = buf.scroll_x(parent_idx);
                let curr_y = buf.scroll_y(parent_idx);
                let max_x = buf.max_scroll_x(parent_idx) as i32;
                let max_y = buf.max_scroll_y(parent_idx) as i32;

                let new_x = (curr_x + dx).clamp(0, max_x.max(0));
                let new_y = (curr_y + dy).clamp(0, max_y.max(0));

                if new_x != curr_x || new_y != curr_y {
                    buf.set_scroll(parent_idx, new_x, new_y);
                    return true;
                }
            }
            current = buf.parent_index(parent_idx);
        }
        false
    }

    /// Scroll to make a component visible within its scrollable parent.
    pub fn scroll_into_view(&self, buf: &SharedBuffer, index: usize) {
        let mut current = buf.parent_index(index);
        while let Some(parent_idx) = current {
            if buf.is_scrollable(parent_idx) {
                let child_y = buf.computed_y(index) as i32;
                let child_h = buf.computed_height(index) as i32;
                let parent_y = buf.computed_y(parent_idx) as i32;
                let parent_h = buf.computed_height(parent_idx) as i32;
                let scroll_y = buf.scroll_y(parent_idx);

                let child_top = child_y - parent_y + scroll_y;
                let child_bottom = child_top + child_h;

                if child_top < scroll_y {
                    // Child is above viewport
                    let max_y = buf.max_scroll_y(parent_idx) as i32;
                    buf.set_scroll(parent_idx, buf.scroll_x(parent_idx), child_top.clamp(0, max_y));
                } else if child_bottom > scroll_y + parent_h {
                    // Child is below viewport
                    let new_y = child_bottom - parent_h;
                    let max_y = buf.max_scroll_y(parent_idx) as i32;
                    buf.set_scroll(parent_idx, buf.scroll_x(parent_idx), new_y.clamp(0, max_y));
                }
                break;
            }
            current = buf.parent_index(parent_idx);
        }
    }
}

impl Default for ScrollManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_manager_new() {
        let _sm = ScrollManager::new();
    }
}
