//! Event ring buffer for Rust → TS communication.
//!
//! Events are written by Rust (input system) and read by TS (callback dispatch).
//! The ring buffer is designed for lock-free single-producer (Rust) single-consumer (TS).

// =============================================================================
// Event Types
// =============================================================================

/// Event types written to the ring buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EventType {
    None = 0,
    Key = 1,
    MouseDown = 2,
    MouseUp = 3,
    Click = 4,
    MouseEnter = 5,
    MouseLeave = 6,
    Scroll = 7,
    Focus = 8,
    Blur = 9,
    ValueChange = 10,
    Submit = 11,
    Cancel = 12,
    Exit = 13,
    Resize = 14,
}

/// An event to be written to the ring buffer.
#[derive(Debug, Clone)]
pub struct Event {
    pub event_type: EventType,
    pub component_index: u16,
    pub data: [u8; 16],
}

impl Event {
    pub fn new(event_type: EventType, component_index: u16) -> Self {
        Self {
            event_type,
            component_index,
            data: [0; 16],
        }
    }

    /// Create a key event with keycode and modifiers.
    pub fn key(component_index: u16, keycode: u32, modifiers: u8) -> Self {
        let mut ev = Self::new(EventType::Key, component_index);
        let bytes = keycode.to_le_bytes();
        ev.data[0..4].copy_from_slice(&bytes);
        ev.data[4] = modifiers;
        ev
    }

    /// Create a mouse event with position.
    pub fn mouse(event_type: EventType, component_index: u16, x: u16, y: u16, button: u8) -> Self {
        let mut ev = Self::new(event_type, component_index);
        ev.data[0..2].copy_from_slice(&x.to_le_bytes());
        ev.data[2..4].copy_from_slice(&y.to_le_bytes());
        ev.data[4] = button;
        ev
    }

    /// Create a scroll event.
    pub fn scroll(component_index: u16, dx: i32, dy: i32) -> Self {
        let mut ev = Self::new(EventType::Scroll, component_index);
        ev.data[0..4].copy_from_slice(&dx.to_le_bytes());
        ev.data[4..8].copy_from_slice(&dy.to_le_bytes());
        ev
    }

    /// Create a resize event.
    pub fn resize(width: u16, height: u16) -> Self {
        let mut ev = Self::new(EventType::Resize, 0);
        ev.data[0..2].copy_from_slice(&width.to_le_bytes());
        ev.data[2..4].copy_from_slice(&height.to_le_bytes());
        ev
    }

    /// Create an exit event (Ctrl+C).
    pub fn exit() -> Self {
        Self::new(EventType::Exit, 0)
    }

    /// Create a value change event (input text changed).
    pub fn value_change(component_index: u16) -> Self {
        Self::new(EventType::ValueChange, component_index)
    }

    /// Create a submit event (Enter in input).
    pub fn submit(component_index: u16) -> Self {
        Self::new(EventType::Submit, component_index)
    }

    /// Create a cancel event (Escape in input).
    pub fn cancel(component_index: u16) -> Self {
        Self::new(EventType::Cancel, component_index)
    }

    /// Create focus/blur events.
    pub fn focus(component_index: u16) -> Self {
        Self::new(EventType::Focus, component_index)
    }

    pub fn blur(component_index: u16) -> Self {
        Self::new(EventType::Blur, component_index)
    }
}

// =============================================================================
// Ring Buffer
// =============================================================================

/// Size of each event in bytes: type(1) + component(2) + data(16) = 19, padded to 20.
const EVENT_SIZE: usize = 20;
/// Maximum events in the ring buffer.
const MAX_EVENTS: usize = 256;
/// Total ring buffer size: header(12) + events(256 * 20) = 5132 bytes.
pub const RING_BUFFER_SIZE: usize = 12 + MAX_EVENTS * EVENT_SIZE;

/// In-memory ring buffer for events.
///
/// In the SharedBuffer version, this maps to a section of shared memory.
/// For now, this is an in-memory implementation that can be ported to SharedBuffer.
pub struct EventRingBuffer {
    events: Vec<Event>,
    write_idx: usize,
    read_idx: usize,
}

impl EventRingBuffer {
    pub fn new() -> Self {
        Self {
            events: Vec::with_capacity(MAX_EVENTS),
            write_idx: 0,
            read_idx: 0,
        }
    }

    /// Write an event to the ring buffer.
    pub fn push(&mut self, event: Event) {
        if self.events.len() < MAX_EVENTS {
            self.events.push(event);
        } else {
            self.events[self.write_idx % MAX_EVENTS] = event;
        }
        self.write_idx += 1;
    }

    /// Read the next event from the ring buffer.
    pub fn pop(&mut self) -> Option<Event> {
        if self.read_idx >= self.write_idx {
            return None;
        }
        let idx = self.read_idx % MAX_EVENTS;
        self.read_idx += 1;
        if idx < self.events.len() {
            Some(self.events[idx].clone())
        } else {
            None
        }
    }

    /// Number of pending events.
    pub fn pending_count(&self) -> usize {
        self.write_idx.saturating_sub(self.read_idx)
    }

    /// Check if there are pending events.
    pub fn has_pending(&self) -> bool {
        self.pending_count() > 0
    }

    /// Drain all pending events.
    pub fn drain(&mut self) -> Vec<Event> {
        let mut events = Vec::with_capacity(self.pending_count());
        while let Some(ev) = self.pop() {
            events.push(ev);
        }
        events
    }
}

impl Default for EventRingBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_push_pop() {
        let mut rb = EventRingBuffer::new();
        assert!(!rb.has_pending());

        rb.push(Event::exit());
        assert!(rb.has_pending());
        assert_eq!(rb.pending_count(), 1);

        let ev = rb.pop().unwrap();
        assert_eq!(ev.event_type, EventType::Exit);
        assert!(!rb.has_pending());
    }

    #[test]
    fn test_ring_buffer_drain() {
        let mut rb = EventRingBuffer::new();
        rb.push(Event::focus(1));
        rb.push(Event::blur(1));
        rb.push(Event::value_change(2));

        let events = rb.drain();
        assert_eq!(events.len(), 3);
        assert_eq!(events[0].event_type, EventType::Focus);
        assert_eq!(events[1].event_type, EventType::Blur);
        assert_eq!(events[2].event_type, EventType::ValueChange);
        assert!(!rb.has_pending());
    }

    #[test]
    fn test_key_event_data() {
        let ev = Event::key(5, 0x61, 0x04); // 'a' with CTRL
        assert_eq!(ev.event_type, EventType::Key);
        assert_eq!(ev.component_index, 5);
        assert_eq!(u32::from_le_bytes([ev.data[0], ev.data[1], ev.data[2], ev.data[3]]), 0x61);
        assert_eq!(ev.data[4], 0x04);
    }

    #[test]
    fn test_mouse_event_data() {
        let ev = Event::mouse(EventType::Click, 3, 10, 20, 0);
        assert_eq!(ev.component_index, 3);
        assert_eq!(u16::from_le_bytes([ev.data[0], ev.data[1]]), 10);
        assert_eq!(u16::from_le_bytes([ev.data[2], ev.data[3]]), 20);
    }
}
