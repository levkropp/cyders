//! Board profile management
//!
//! Loads and manages CYD board configurations from JSON.

use serde::{Deserialize, Serialize};

/// CYD board profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardProfile {
    /// Board model (e.g., "2432S028R")
    pub model: String,
    /// Chip type (e.g., "ESP32", "ESP32-S3")
    pub chip: String,
    /// Number of CPU cores
    pub cores: u32,
    /// Display width in pixels
    pub display_width: u32,
    /// Display height in pixels
    pub display_height: u32,
    /// Display size description (e.g., "2.8\"")
    pub display_size: String,
    /// Touch controller type (e.g., "XPT2046 (resistive)")
    pub touch_type: String,
    /// Number of SD card slots
    pub sd_slots: u32,
    /// USB connector type (e.g., "Micro-USB (UART)")
    pub usb_type: String,
}

/// Load all board profiles from embedded JSON
pub fn load_boards() -> Vec<BoardProfile> {
    let json = include_str!("../boards.json");
    serde_json::from_str(json).unwrap_or_else(|e| {
        eprintln!("Failed to parse boards.json: {}", e);
        vec![]
    })
}

/// Get a specific board profile by model name
pub fn get_board(model: &str) -> Option<BoardProfile> {
    load_boards().into_iter().find(|b| b.model == model)
}

/// Get all available board models
pub fn get_board_models() -> Vec<String> {
    load_boards().into_iter().map(|b| b.model).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_boards() {
        let boards = load_boards();
        assert!(!boards.is_empty(), "Should load at least one board");
    }

    #[test]
    fn test_get_board() {
        let board = get_board("2432S028R");
        assert!(board.is_some(), "Should find default board");

        if let Some(b) = board {
            assert_eq!(b.model, "2432S028R");
            assert_eq!(b.chip, "ESP32");
            assert_eq!(b.cores, 2);
            assert_eq!(b.display_width, 320);
            assert_eq!(b.display_height, 240);
        }
    }

    #[test]
    fn test_get_unknown_board() {
        let board = get_board("UNKNOWN_MODEL");
        assert!(board.is_none(), "Should not find unknown board");
    }
}
