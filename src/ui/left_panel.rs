//! Left panel widget - tabbed debugging tools (Registers / Stack / Locals)

use super::UiData;

/// Active tab in the left panel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeftPanelTab {
    Registers,
    Stack,
    Locals,
}

/// Left panel configuration
pub struct LeftPanel {
    /// Panel width (pixels)
    pub width: f32,
    /// Currently active tab
    pub active_tab: LeftPanelTab,
}

impl LeftPanel {
    /// Create a new left panel with default settings
    pub fn new() -> Self {
        Self {
            width: 320.0,
            active_tab: LeftPanelTab::Registers,
        }
    }

    /// Render the left panel
    pub fn render(&mut self, ctx: &egui::Context, state: &UiData) {
        // Create resizable side panel on the left
        egui::SidePanel::left("left_panel")
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
                // Add padding to prevent text cutoff
                ui.add_space(4.0);

                // Tab bar at the top
                ui.horizontal(|ui| {
                    ui.add_space(4.0);
                    ui.selectable_value(&mut self.active_tab, LeftPanelTab::Registers, "📋 Registers");
                    ui.selectable_value(&mut self.active_tab, LeftPanelTab::Stack, "📚 Stack");
                    ui.selectable_value(&mut self.active_tab, LeftPanelTab::Locals, "🔍 Locals");
                });

                ui.separator();

                // Content area with scrolling
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        match self.active_tab {
                            LeftPanelTab::Registers => self.render_registers(ui, state),
                            LeftPanelTab::Stack => self.render_stack(ui, state),
                            LeftPanelTab::Locals => self.render_locals(ui, state),
                        }
                    });
            });
    }

    /// Render registers tab
    fn render_registers(&self, ui: &mut egui::Ui, state: &UiData) {
        ui.heading("REGISTERS");

        // Show current window
        ui.monospace(format!("Window: {}", state.windowbase));
        ui.add_space(4.0);

        // General purpose registers (a0-a15)
        ui.label(egui::RichText::new("General Purpose").strong());
        egui::Grid::new("registers_grid")
            .striped(true)
            .spacing([10.0, 4.0])
            .show(ui, |ui| {
                for i in 0..16 {
                    let reg_name = format!("a{:<2}", i);
                    let reg_value = state.registers[i];

                    ui.monospace(&reg_name);
                    ui.monospace(format!("0x{:08X}", reg_value));

                    // Special annotations
                    if i == 0 {
                        ui.label(egui::RichText::new("(RA)").color(egui::Color32::GRAY));
                    } else if i == 1 {
                        ui.label(egui::RichText::new("(SP)").color(egui::Color32::GRAY));
                    } else {
                        ui.label("");
                    }

                    ui.end_row();
                }
            });

        ui.add_space(8.0);

        // Special registers
        ui.label(egui::RichText::new("Special Registers").strong());
        egui::Grid::new("special_regs_grid")
            .striped(true)
            .spacing([10.0, 4.0])
            .show(ui, |ui| {
                ui.monospace("PC");
                ui.monospace(format!("0x{:08X}", state.pc));
                ui.label("");
                ui.end_row();

                ui.monospace("WINDOWBASE");
                ui.monospace(format!("{}", state.windowbase));
                ui.label("");
                ui.end_row();

                ui.monospace("INTLEVEL");
                ui.monospace(format!("{}", state.intlevel));
                ui.label("");
                ui.end_row();
            });
    }

    /// Render stack tab (call stack)
    fn render_stack(&self, ui: &mut egui::Ui, state: &UiData) {
        ui.heading("CALL STACK");

        // For now, show current PC as top of stack
        // TODO: Implement stack unwinding from return addresses
        egui::Grid::new("stack_grid")
            .striped(true)
            .spacing([10.0, 4.0])
            .show(ui, |ui| {
                ui.monospace("#0");
                ui.monospace(format!("0x{:08X}", state.pc));
                if let Some(ref sym) = state.pc_symbol {
                    ui.label(sym);
                } else {
                    ui.label("<unknown>");
                }
                ui.end_row();

                // Show return address from a0
                if state.registers.len() > 0 {
                    let ra = state.registers[0];
                    if ra != 0 {
                        ui.monospace("#1");
                        ui.monospace(format!("0x{:08X}", ra));
                        ui.label("<caller>");
                        ui.end_row();
                    }
                }
            });

        ui.add_space(8.0);
        ui.label(egui::RichText::new("Stack unwinding coming soon...").color(egui::Color32::GRAY).italics());
    }

    /// Render locals tab (local variables)
    fn render_locals(&self, ui: &mut egui::Ui, state: &UiData) {
        ui.heading("LOCAL VARIABLES");

        // Show current function context
        if let Some(ref sym) = state.pc_symbol {
            ui.label(format!("Function: {}", sym));
        } else {
            ui.label("Function: <unknown>");
        }

        ui.add_space(8.0);

        // Show stack pointer and potential locals
        ui.label(egui::RichText::new("Stack Frame").strong());
        egui::Grid::new("locals_grid")
            .striped(true)
            .spacing([10.0, 4.0])
            .show(ui, |ui| {
                ui.monospace("SP (a1)");
                if state.registers.len() > 1 {
                    ui.monospace(format!("0x{:08X}", state.registers[1]));
                } else {
                    ui.monospace("0x00000000");
                }
                ui.end_row();

                // Show a2-a7 as potential function arguments/locals
                for i in 2..8 {
                    ui.monospace(format!("a{} (arg{})", i, i - 2));
                    if i < state.registers.len() {
                        ui.monospace(format!("0x{:08X}", state.registers[i]));
                    } else {
                        ui.monospace("0x00000000");
                    }
                    ui.end_row();
                }
            });

        ui.add_space(8.0);
        ui.label(egui::RichText::new("DWARF debug info required for full locals view").color(egui::Color32::GRAY).italics());
    }
}
