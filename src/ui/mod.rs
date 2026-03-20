//! UI module - egui-based info panel and debug UI

pub mod info_panel;
pub mod left_panel;

pub use info_panel::InfoPanel;
pub use left_panel::LeftPanel;

/// UI state container
pub struct UiState {
    /// Left panel widget (Registers / Stack / Locals)
    pub left_panel: LeftPanel,
    /// Info panel widget (Board info, Performance, etc.)
    pub info_panel: InfoPanel,
    /// Whether to show the left panel
    pub show_left_panel: bool,
    /// Whether to show the info panel
    pub show_info_panel: bool,
}

impl UiState {
    /// Create a new UI state
    pub fn new() -> Self {
        Self {
            left_panel: LeftPanel::new(),
            info_panel: InfoPanel::new(),
            show_left_panel: true,
            show_info_panel: true,
        }
    }

    /// Render all UI components
    pub fn render(&mut self, ctx: &egui::Context, state: &UiData) {
        // Remove all default spacing to eliminate black gaps
        let mut style = (*ctx.style()).clone();
        style.spacing.window_margin = egui::Margin::same(0.0);
        style.spacing.item_spacing = egui::Vec2::new(0.0, 4.0);
        ctx.set_style(style);

        // Render left panel first (so it's on the left)
        if self.show_left_panel {
            self.left_panel.render(ctx, state);
        }

        // Then render right info panel
        if self.show_info_panel {
            self.info_panel.render(ctx, state);
        }

        // No CentralPanel needed - the space between the side panels
        // is automatically left for the pixels display
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self::new()
    }
}

/// Data to display in UI (passed from app to UI)
#[derive(Clone)]
pub struct UiData {
    /// Board model (e.g., "CYD-2432S028R")
    pub board_model: String,
    /// Chip name (e.g., "ESP32 (Dual-Core)")
    pub chip_name: String,
    /// Cycle count
    pub cycles: u64,
    /// MIPS (millions of instructions per second)
    pub mips: f64,
    /// Batch execution rate (batches/sec)
    pub batch_rate: f64,
    /// Program counter (PC)
    pub pc: u32,
    /// PC symbol name (e.g., "vTaskDelay+0x12")
    pub pc_symbol: Option<String>,
    /// CPU status ("Running", "Paused", "Halted")
    pub status: String,
    /// Is running (not paused)
    pub is_running: bool,
    /// Is halted (exception or halt instruction)
    pub is_halted: bool,
    /// Window registers (a0-a15)
    pub registers: [u32; 16],
    /// Special registers
    pub windowbase: u32,
    pub intlevel: u32,
    /// UART output (last N lines)
    pub uart_output: Vec<String>,
    /// Touch events log (last N events)
    pub touch_log: Vec<String>,
}

impl Default for UiData {
    fn default() -> Self {
        Self {
            board_model: "CYD-2432S028R".to_string(),
            chip_name: "ESP32 (Dual-Core)".to_string(),
            cycles: 0,
            mips: 0.0,
            batch_rate: 0.0,
            pc: 0,
            pc_symbol: None,
            status: "Paused".to_string(),
            is_running: false,
            is_halted: false,
            registers: [0; 16],
            windowbase: 0,
            intlevel: 0,
            uart_output: Vec::new(),
            touch_log: Vec::new(),
        }
    }
}
