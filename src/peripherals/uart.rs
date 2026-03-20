//! UART emulation with circular buffer
//!
//! Captures UART output from firmware for display/logging.

use std::collections::VecDeque;
use parking_lot::Mutex;
use std::sync::Arc;

const MAX_BUFFER_SIZE: usize = 16384; // 16 KB

/// UART output buffer
pub struct UartBuffer {
    buffer: Arc<Mutex<VecDeque<u8>>>,
}

impl UartBuffer {
    /// Create a new UART buffer
    pub fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(VecDeque::with_capacity(MAX_BUFFER_SIZE))),
        }
    }

    /// Write a byte to the buffer
    pub fn write_byte(&self, byte: u8) {
        let mut buf = self.buffer.lock();

        // If buffer is full, remove oldest byte
        if buf.len() >= MAX_BUFFER_SIZE {
            buf.pop_front();
        }

        buf.push_back(byte);
    }

    /// Write multiple bytes to the buffer
    pub fn write_bytes(&self, bytes: &[u8]) {
        for &byte in bytes {
            self.write_byte(byte);
        }
    }

    /// Read all available bytes (consumes them)
    pub fn read_all(&self) -> Vec<u8> {
        let mut buf = self.buffer.lock();
        buf.drain(..).collect()
    }

    /// Read up to `count` bytes (consumes them)
    pub fn read(&self, count: usize) -> Vec<u8> {
        let mut buf = self.buffer.lock();
        let to_read = count.min(buf.len());
        buf.drain(..to_read).collect()
    }

    /// Peek at all bytes without consuming
    pub fn peek_all(&self) -> Vec<u8> {
        let buf = self.buffer.lock();
        buf.iter().copied().collect()
    }

    /// Get the number of bytes in the buffer
    pub fn len(&self) -> usize {
        self.buffer.lock().len()
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.lock().is_empty()
    }

    /// Clear the buffer
    pub fn clear(&self) {
        self.buffer.lock().clear();
    }

    /// Get the buffer as a UTF-8 string (invalid UTF-8 replaced with �)
    pub fn as_string(&self) -> String {
        let bytes = self.peek_all();
        String::from_utf8_lossy(&bytes).into_owned()
    }

    /// Get a clone of the buffer Arc (for sharing)
    pub fn clone_arc(&self) -> Arc<Mutex<VecDeque<u8>>> {
        self.buffer.clone()
    }
}

impl Default for UartBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for UartBuffer {
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uart_write_read() {
        let uart = UartBuffer::new();

        uart.write_bytes(b"Hello");
        assert_eq!(uart.len(), 5);

        let data = uart.read_all();
        assert_eq!(data, b"Hello");
        assert_eq!(uart.len(), 0);
    }

    #[test]
    fn test_uart_overflow() {
        let uart = UartBuffer::new();

        // Fill buffer beyond capacity
        let test_data = vec![0xAA; MAX_BUFFER_SIZE + 100];
        uart.write_bytes(&test_data);

        // Should be capped at MAX_BUFFER_SIZE
        assert_eq!(uart.len(), MAX_BUFFER_SIZE);

        // Oldest bytes should have been dropped
        let data = uart.read_all();
        assert_eq!(data.len(), MAX_BUFFER_SIZE);
        assert!(data.iter().all(|&b| b == 0xAA));
    }

    #[test]
    fn test_uart_as_string() {
        let uart = UartBuffer::new();

        uart.write_bytes(b"Hello, World!\n");
        assert_eq!(uart.as_string(), "Hello, World!\n");

        // Clear and verify
        uart.clear();
        assert!(uart.is_empty());
        assert_eq!(uart.as_string(), "");
    }

    #[test]
    fn test_uart_peek() {
        let uart = UartBuffer::new();

        uart.write_bytes(b"Test");
        assert_eq!(uart.peek_all(), b"Test");
        assert_eq!(uart.len(), 4); // Peek doesn't consume

        let data = uart.read(2);
        assert_eq!(data, b"Te");
        assert_eq!(uart.len(), 2);

        assert_eq!(uart.peek_all(), b"st");
    }
}
