//! Main application structure with egui UI integration

use winit::event::WindowEvent;
use winit::window::Window;
use anyhow::Result;

use crate::display::Display;
use crate::input::InputHandler;
use crate::emulator::Emulator;
use crate::ui::{UiState, UiData};

/// Main application state
pub struct App {
    /// Display manager
    display: Display,
    /// Input handler
    input: InputHandler,
    /// Emulator instance (None if not loaded)
    emulator: Option<Emulator>,
    /// UI state
    ui_state: UiState,
    /// Board model string
    board_model: String,
    /// Exit requested flag
    exit_requested: bool,
}

impl App {
    /// Create a new application instance
    pub fn new(window: &Window, scale: u32, board_model: String) -> Result<Self> {
        let display = Display::new(window)?;
        let input = InputHandler::new(scale);
        let ui_state = UiState::new();

        Ok(Self {
            display,
            input,
            emulator: None,
            ui_state,
            board_model,
            exit_requested: false,
        })
    }

    /// Load firmware and start emulation
    pub fn load_firmware(&mut self, bin_path: String, elf_path: Option<String>) -> Result<()> {
        tracing::info!("Loading firmware: {}", bin_path);

        let config = crate::flexe_ffi::SessionConfig {
            bin_path,
            elf_path,
            sdcard_path: None,
            sdcard_size: 0,
            entry_override: 0,
            initial_sp: 0,
            single_core: false,
            window_trace: false,
            spill_verify: false,
            native_freertos: false,
        };

        let framebuffer = self.display.framebuffer();
        let touch_state = self.input.touch_state();
        let emulator = Emulator::new(config, framebuffer, Some(touch_state))?;

        self.emulator = Some(emulator);
        tracing::info!("Emulator started");

        Ok(())
    }

    /// Handle winit window events
    pub fn handle_event(&mut self, _window: &Window, event: &WindowEvent) -> bool {
        // Let egui handle the event first
        let egui_consumed = self.display.handle_event(event);

        // Always let input handler process mouse events for touch input
        // Even if egui consumed them, we still want touch state updated
        match event {
            WindowEvent::MouseInput { state, button, .. } => {
                self.input.handle_mouse_button(*state, *button);
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.input.handle_cursor_moved(*position);
            }
            _ => {}
        }

        // Handle other events
        if egui_consumed {
            return true;
        }

        match event {
            WindowEvent::Resized(size) => {
                self.display.resize(size.width, size.height).ok();
                true
            }
            WindowEvent::KeyboardInput { input, .. } => {
                // Handle keyboard shortcuts
                if let Some(key) = input.virtual_keycode {
                    use winit::event::VirtualKeyCode;
                    match key {
                        VirtualKeyCode::Escape => {
                            self.exit_requested = true;
                        }
                        VirtualKeyCode::P if input.state == winit::event::ElementState::Pressed => {
                            // Toggle pause
                            if let Some(emu) = &self.emulator {
                                if emu.is_running() {
                                    emu.pause();
                                    tracing::info!("Emulator paused");
                                } else {
                                    emu.resume();
                                    tracing::info!("Emulator resumed");
                                }
                            }
                        }
                        VirtualKeyCode::F10 if input.state == winit::event::ElementState::Pressed => {
                            // Toggle info panel
                            self.ui_state.show_info_panel = !self.ui_state.show_info_panel;
                        }
                        _ => {}
                    }
                }
                true
            }
            _ => false,
        }
    }

    /// Update application state (called every frame)
    pub fn update(&mut self) {
        // Mark display as dirty if emulator is running
        if let Some(emu) = &self.emulator {
            if emu.is_running() {
                self.display.mark_dirty();
            }
        }
    }

    /// Render everything (display + UI)
    pub fn render(&mut self, window: &Window) -> Result<()> {
        // Prepare UI data
        let ui_data = self.get_ui_data();

        // Render display framebuffer with egui overlay
        self.display.render_with_ui(window, |ctx| {
            self.ui_state.render(ctx, &ui_data);
        })?;

        Ok(())
    }

    /// Gather data for UI rendering
    fn get_ui_data(&self) -> UiData {
        let mut data = UiData::default();
        data.board_model = self.board_model.clone();
        data.chip_name = "ESP32 (Dual-Core)".to_string();

        // Get touch log from input handler
        data.touch_log = self.input.get_touch_log();

        if let Some(emu) = &self.emulator {
            data.cycles = emu.cycle_count();
            data.mips = emu.mips();
            data.batch_rate = emu.fps();
            data.pc = emu.pc();
            data.is_running = emu.is_running();
            data.is_halted = emu.is_halted();
            data.registers = emu.get_all_registers();
            data.windowbase = emu.get_windowbase();
            data.intlevel = emu.get_intlevel();

            // Look up symbol for PC
            if let Some(symbol) = emu.lookup_symbol(data.pc) {
                data.pc_symbol = Some(symbol.format());
            }

            // Set status string
            data.status = if data.is_halted {
                "Halted".to_string()
            } else if data.is_running {
                "Running".to_string()
            } else {
                "Paused".to_string()
            };
        }

        data
    }

    /// Check if exit was requested
    pub fn should_exit(&self) -> bool {
        self.exit_requested
    }

    /// Get performance stats string
    pub fn stats_string(&self) -> String {
        if let Some(emu) = &self.emulator {
            format!(
                "Cyders | Cycles: {} | PC: 0x{:08X} | {:.1} MIPS | {} | Press ESC to quit, P to pause/resume",
                emu.cycle_count(),
                emu.pc(),
                emu.mips(),
                if emu.is_running() { "Running" } else { "Paused" }
            )
        } else {
            "Cyders | No firmware loaded | Press ESC to quit".to_string()
        }
    }

    /// Update window title with stats
    pub fn update_title(&self, window: &Window) {
        window.set_title(&self.stats_string());
    }
}
