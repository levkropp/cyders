//! Input handling - Mouse to touch input mapping
//!
//! Maps mouse events to touch coordinates for firmware interaction.

use winit::event::{ElementState, MouseButton};
use winit::dpi::PhysicalPosition;
use parking_lot::Mutex;
use std::sync::Arc;

/// Touch state with edge detection
#[derive(Debug, Clone)]
pub struct TouchState {
    /// Is touch currently down?
    pub down: bool,
    /// Current X coordinate (display pixels, 0-319)
    pub x: i32,
    /// Current Y coordinate (display pixels, 0-239)
    pub y: i32,
    /// Edge detection: touch just went down
    pending_down: bool,
}

impl TouchState {
    pub fn new() -> Self {
        Self {
            down: false,
            x: 0,
            y: 0,
            pending_down: false,
        }
    }

    /// Update touch state with new position
    pub fn update(&mut self, down: bool, x: i32, y: i32) {
        // Detect rising edge (touch down event)
        if down && !self.down {
            self.pending_down = true;
        }

        self.down = down;
        self.x = x.clamp(0, 319);
        self.y = y.clamp(0, 239);
    }

    /// Read current touch position (returns None if not touching)
    pub fn read(&self) -> Option<(i32, i32)> {
        if self.down {
            Some((self.x, self.y))
        } else {
            None
        }
    }

    /// Check if there's a pending tap event and consume it
    pub fn consume_tap(&mut self) -> bool {
        let result = self.pending_down;
        self.pending_down = false;
        result
    }
}

impl Default for TouchState {
    fn default() -> Self {
        Self::new()
    }
}

/// Input handler with window scaling support
pub struct InputHandler {
    touch_state: Arc<Mutex<TouchState>>,
    scale: f64,
    /// Touch event log (recent touch down/up events)
    touch_log: Arc<Mutex<Vec<String>>>,
    /// Maximum number of touch events to keep
    max_touch_events: usize,
}

impl InputHandler {
    /// Create a new input handler with the given window scale
    pub fn new(scale: u32) -> Self {
        Self {
            touch_state: Arc::new(Mutex::new(TouchState::new())),
            scale: scale as f64,
            touch_log: Arc::new(Mutex::new(Vec::new())),
            max_touch_events: 20,
        }
    }

    /// Handle mouse button press/release
    pub fn handle_mouse_button(&self, state: ElementState, button: MouseButton) {
        if button == MouseButton::Left {
            let is_down = state == ElementState::Pressed;
            let mut touch = self.touch_state.lock();

            // Update only the down state, preserve position
            let x = touch.x;
            let y = touch.y;
            let prev_down = touch.down;
            touch.update(is_down, x, y);

            // Log touch event (only on state change)
            if is_down != prev_down {
                let event = if is_down {
                    format!("Down ({}, {})", x, y)
                } else {
                    format!("Up   ({}, {})", x, y)
                };

                let mut log = self.touch_log.lock();
                log.push(event);

                // Keep only the most recent events
                let len = log.len();
                if len > self.max_touch_events {
                    log.drain(0..len - self.max_touch_events);
                }
            }
        }
    }

    /// Handle cursor movement
    pub fn handle_cursor_moved(&self, position: PhysicalPosition<f64>) {
        // Convert window coordinates to display coordinates
        let x = (position.x / self.scale) as i32;
        let y = (position.y / self.scale) as i32;

        let mut touch = self.touch_state.lock();
        let is_down = touch.down;
        touch.update(is_down, x, y);
    }

    /// Get a clone of the touch state Arc (for sharing with emulator)
    pub fn touch_state(&self) -> Arc<Mutex<TouchState>> {
        self.touch_state.clone()
    }

    /// Get the touch event log
    pub fn get_touch_log(&self) -> Vec<String> {
        self.touch_log.lock().clone()
    }

    /// Update the window scale factor
    pub fn set_scale(&mut self, scale: u32) {
        self.scale = scale as f64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_touch_edge_detection() {
        let mut touch = TouchState::new();
        assert!(!touch.consume_tap());

        // Simulate touch down
        touch.update(true, 100, 100);
        assert!(touch.consume_tap());
        assert!(!touch.consume_tap()); // Should only fire once

        // Touch still down
        touch.update(true, 150, 150);
        assert!(!touch.consume_tap());

        // Touch up
        touch.update(false, 150, 150);
        assert!(!touch.consume_tap());

        // Touch down again
        touch.update(true, 200, 200);
        assert!(touch.consume_tap());
    }

    #[test]
    fn test_touch_clamping() {
        let mut touch = TouchState::new();

        // Test out-of-bounds coordinates
        touch.update(true, -50, -50);
        assert_eq!(touch.x, 0);
        assert_eq!(touch.y, 0);

        touch.update(true, 500, 500);
        assert_eq!(touch.x, 319);
        assert_eq!(touch.y, 239);
    }

    #[test]
    fn test_input_handler_scaling() {
        let handler = InputHandler::new(2);

        // Simulate click at window coordinates (640, 480)
        // Should map to display coordinates (320, 240) with scale=2
        // But clamped to (319, 239) since display is 320x240
        handler.handle_cursor_moved(PhysicalPosition::new(640.0, 480.0));
        handler.handle_mouse_button(ElementState::Pressed, MouseButton::Left);

        let touch = handler.touch_state.lock();
        assert_eq!(touch.x, 319); // Clamped
        assert_eq!(touch.y, 239); // Clamped
        assert!(touch.down);
    }
}
