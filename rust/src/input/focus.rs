//! Focus management system.
//!
//! Manages which component is focused, focus navigation (Tab/Shift+Tab),
//! focus traps, focus history, and implicit focusable detection.
//!
//! All state is stored in SharedBuffer interaction arrays.

use crate::shared_buffer_aos::AoSBuffer;
use super::events::Event;

// =============================================================================
// Focus State
// =============================================================================

/// Focus management state.
pub struct FocusManager {
    /// Currently focused component index. -1 = no focus.
    focused_index: i32,
    /// Focus trap stack: each entry is a container index whose children
    /// are the only valid focus targets.
    trap_stack: Vec<usize>,
    /// Focus history for save/restore (max 10).
    history: Vec<i32>,
}

impl FocusManager {
    pub fn new() -> Self {
        Self {
            focused_index: -1,
            trap_stack: Vec::new(),
            history: Vec::new(),
        }
    }

    /// Get the currently focused component index.
    pub fn focused(&self) -> Option<usize> {
        if self.focused_index >= 0 {
            Some(self.focused_index as usize)
        } else {
            None
        }
    }

    /// Focus a specific component.
    pub fn focus(
        &mut self,
        buf: &AoSBuffer,
        index: usize,
    ) {
        let node_count = buf.node_count();
        if index >= node_count {
            return;
        }

        // Must be focusable and visible
        if !buf.focusable(index) || !buf.visible(index) {
            return;
        }

        // Blur previous
        if let Some(prev) = self.focused() {
            buf.push_event(&Event::blur(prev as u16));
        }

        self.focused_index = index as i32;
        buf.set_focused_index(index as i32); // Sync to SharedBuffer for rendering!
        buf.push_event(&Event::focus(index as u16));
    }

    /// Clear focus.
    pub fn blur(&mut self, buf: &AoSBuffer) {
        if let Some(prev) = self.focused() {
            buf.push_event(&Event::blur(prev as u16));
        }
        self.focused_index = -1;
        buf.set_focused_index(-1); // Sync to SharedBuffer!
    }

    /// Focus next focusable component (Tab navigation).
    pub fn focus_next(&mut self, buf: &AoSBuffer) {
        let focusables = self.get_focusable_list(buf);
        if focusables.is_empty() {
            return;
        }

        let next = match self.focused() {
            Some(current) => {
                let pos = focusables.iter().position(|&f| f == current);
                match pos {
                    Some(i) => focusables[(i + 1) % focusables.len()],
                    None => focusables[0],
                }
            }
            None => focusables[0],
        };

        self.focus(buf, next);
    }

    /// Focus previous focusable component (Shift+Tab navigation).
    pub fn focus_previous(&mut self, buf: &AoSBuffer) {
        let focusables = self.get_focusable_list(buf);
        if focusables.is_empty() {
            return;
        }

        let prev = match self.focused() {
            Some(current) => {
                let pos = focusables.iter().position(|&f| f == current);
                match pos {
                    Some(0) => focusables[focusables.len() - 1],
                    Some(i) => focusables[i - 1],
                    None => focusables[focusables.len() - 1],
                }
            }
            None => focusables[focusables.len() - 1],
        };

        self.focus(buf, prev);
    }

    /// Get sorted list of focusable component indices.
    fn get_focusable_list(&self, buf: &AoSBuffer) -> Vec<usize> {
        let node_count = buf.node_count();
        let mut focusables: Vec<(i32, usize)> = Vec::new();

        for i in 0..node_count {
            if buf.component_type(i) == 0 || !buf.visible(i) {
                continue;
            }
            if !buf.focusable(i) {
                // Check implicit focusable: scrollable boxes
                if buf.output_scrollable(i) {
                    // Implicit focusable (scroll boxes)
                } else {
                    continue;
                }
            }

            // Check focus trap
            if !self.is_in_focus_trap(buf, i) {
                continue;
            }

            focusables.push((buf.tab_index(i), i));
        }

        // Sort by tab index (stable sort preserves DOM order for equal tab indices)
        focusables.sort_by_key(|&(tab, _)| tab);
        focusables.into_iter().map(|(_, idx)| idx).collect()
    }

    /// Check if a component is within the current focus trap.
    fn is_in_focus_trap(&self, buf: &AoSBuffer, index: usize) -> bool {
        if self.trap_stack.is_empty() {
            return true; // No trap active
        }

        let trap = *self.trap_stack.last().unwrap();
        // Walk up parent chain to see if index is a descendant of trap container
        let mut current = Some(index);
        while let Some(idx) = current {
            if idx == trap {
                return true;
            }
            current = buf.parent_index(idx);
        }
        false
    }

    /// Push a focus trap (restrict focus to children of container).
    pub fn push_trap(&mut self, container_index: usize) {
        self.trap_stack.push(container_index);
    }

    /// Pop the current focus trap.
    pub fn pop_trap(&mut self) {
        self.trap_stack.pop();
    }

    /// Save current focus to history.
    pub fn save_focus(&mut self) {
        if self.history.len() >= 10 {
            self.history.remove(0);
        }
        self.history.push(self.focused_index);
    }

    /// Restore focus from history.
    pub fn restore_focus(&mut self, buf: &AoSBuffer) {
        if let Some(idx) = self.history.pop() {
            if idx >= 0 {
                self.focus(buf, idx as usize);
            } else {
                self.blur(buf);
            }
        }
    }

    /// Focus a component by click (focus-on-click).
    pub fn focus_by_click(
        &mut self,
        buf: &AoSBuffer,
        component_index: usize,
    ) {
        // Walk up from clicked component to find a focusable ancestor
        let mut current = Some(component_index);
        while let Some(idx) = current {
            if buf.focusable(idx) && buf.visible(idx) {
                self.focus(buf, idx);
                return;
            }
            current = buf.parent_index(idx);
        }
    }
}

impl Default for FocusManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_manager_new() {
        let fm = FocusManager::new();
        assert_eq!(fm.focused(), None);
        assert!(fm.trap_stack.is_empty());
    }

    #[test]
    fn test_focus_trap_stack() {
        let mut fm = FocusManager::new();
        fm.push_trap(5);
        fm.push_trap(10);
        assert_eq!(fm.trap_stack.len(), 2);
        fm.pop_trap();
        assert_eq!(fm.trap_stack.len(), 1);
        assert_eq!(fm.trap_stack[0], 5);
    }

    #[test]
    fn test_focus_history() {
        let mut fm = FocusManager::new();
        fm.focused_index = 5;
        fm.save_focus();
        fm.focused_index = 10;
        fm.save_focus();
        assert_eq!(fm.history.len(), 2);
        assert_eq!(fm.history[0], 5);
        assert_eq!(fm.history[1], 10);
    }
}
