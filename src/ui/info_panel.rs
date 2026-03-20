//! Info panel widget - displays emulator state and performance metrics

use super::UiData;

/// Info panel configuration
pub struct InfoPanel {
    /// Panel width (pixels)
    pub width: f32,
    /// Maximum UART lines to show
    pub max_uart_lines: usize,
    /// Maximum touch events to show
    pub max_touch_events: usize,
}

impl InfoPanel {
    /// Create a new info panel with default settings
    pub fn new() -> Self {
        Self {
            width: 320.0,
            max_uart_lines: 16,
            max_touch_events: 8,
        }
    }

    /// Render the info panel
    pub fn render(&mut self, ctx: &egui::Context, state: &UiData) {
        // Create resizable side panel on the right
        egui::SidePanel::right("info_panel")
            .resizable(true)
            .default_width(self.width)
            .min_width(200.0)
            .max_width(400.0)
            .frame(egui::Frame {
                fill: egui::Color32::from_rgb(30, 30, 30),  // Dark gray background
                inner_margin: egui::Margin::same(8.0),  // Minimal internal padding
                ..Default::default()
            })
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.render_board_info(ui, state);
                        ui.separator();
                        self.render_performance(ui, state);
                        ui.separator();
                        self.render_cpu_state(ui, state);
                        ui.separator();
                        self.render_touch_log(ui, state);
                        ui.separator();
                        self.render_uart_output(ui, state);
                        ui.separator();
                        self.render_statistics(ui, state);
                    });
            });
    }

    /// Render board information section
    fn render_board_info(&self, ui: &mut egui::Ui, state: &UiData) {
        ui.heading("BOARD INFO");
        ui.monospace(format!("Model: {}", state.board_model));
        ui.monospace(format!("Chip:  {}", state.chip_name));
    }

    /// Render performance metrics section
    fn render_performance(&self, ui: &mut egui::Ui, state: &UiData) {
        ui.heading("PERFORMANCE");
        ui.monospace(format!("Cycles:   {:>12}", format_number(state.cycles)));
        ui.monospace(format!("MIPS:     {:>12.1}", state.mips));
        ui.monospace(format!("Batch:    {:>12.0}/sec", state.batch_rate));
    }

    /// Render CPU state section
    fn render_cpu_state(&self, ui: &mut egui::Ui, state: &UiData) {
        ui.heading("CPU STATE");

        // PC address
        ui.monospace(format!("PC: 0x{:08X}", state.pc));

        // PC symbol name (if available)
        if let Some(ref symbol) = state.pc_symbol {
            ui.monospace(format!("    {}", symbol));
        }

        // Status with color coding
        let status_color = if state.is_halted {
            egui::Color32::from_rgb(255, 100, 100) // Red for halted
        } else if state.is_running {
            egui::Color32::from_rgb(100, 255, 100) // Green for running
        } else {
            egui::Color32::from_rgb(200, 200, 100) // Yellow for paused
        };

        ui.colored_label(status_color, format!("Status: {}", state.status));

        // Special registers
        ui.monospace(format!("WINDOWBASE: {}  INTLEVEL: {}", state.windowbase, state.intlevel));
    }

    /// Render register window section
    fn render_registers(&self, ui: &mut egui::Ui, state: &UiData) {
        ui.heading(format!("REGISTERS (Window {})", state.windowbase));

        egui::Grid::new("registers_grid")
            .striped(true)
            .spacing([10.0, 4.0])
            .show(ui, |ui| {
                for i in 0..16 {
                    // Register name
                    let reg_name = match i {
                        1 => "a1 (SP)",
                        _ => &format!("a{}", i),
                    };

                    ui.monospace(format!("{:8}", reg_name));
                    ui.monospace(format!("0x{:08X}", state.registers[i]));
                    ui.end_row();
                }
            });
    }

    /// Render touch events log
    fn render_touch_log(&self, ui: &mut egui::Ui, state: &UiData) {
        ui.heading("TOUCH LOG");

        if state.touch_log.is_empty() {
            ui.monospace("(no touch events)");
        } else {
            // Show last N touch events
            let start_idx = state.touch_log.len().saturating_sub(self.max_touch_events);
            for event in &state.touch_log[start_idx..] {
                ui.monospace(event);
            }
        }
    }

    /// Render UART output section
    fn render_uart_output(&self, ui: &mut egui::Ui, state: &UiData) {
        ui.heading("UART OUTPUT");

        egui::ScrollArea::vertical()
            .max_height(200.0)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                if state.uart_output.is_empty() {
                    ui.monospace("(no output)");
                } else {
                    // Show last N lines
                    let start_idx = state.uart_output.len().saturating_sub(self.max_uart_lines);
                    for line in &state.uart_output[start_idx..] {
                        ui.monospace(line);
                    }
                }
            });
    }

    /// Render statistics section
    fn render_statistics(&self, ui: &mut egui::Ui, _state: &UiData) {
        ui.heading("STATISTICS");

        ui.label(egui::RichText::new("Coming soon:").strong());
        ui.monospace("• ROM call counts");
        ui.monospace("• Memory access stats");
        ui.monospace("• Interrupt frequency");
        ui.monospace("• Exception history");
    }
}

impl Default for InfoPanel {
    fn default() -> Self {
        Self::new()
    }
}

/// Format large numbers with commas
fn format_number(n: u64) -> String {
    let s = n.to_string();
    let chars: Vec<char> = s.chars().collect();
    let mut result = String::new();

    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*c);
    }

    result
}
