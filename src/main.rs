//! Cyders - Modern Rust CYD Emulator
//!
//! A cross-platform emulator for CYD ESP32 development boards using
//! pixels + winit for rendering and flexe for Xtensa emulation.

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use clap::Parser;
use anyhow::Result;
use std::time::Instant;

mod app;
mod display;
mod input;
mod ui;
mod emulator;
mod flexe_ffi;
mod board;
mod peripherals;

use app::App;

// Crash handler FFI (Windows only)
#[cfg(target_os = "windows")]
extern "C" {
    fn cyders_crash_handler_install();
}

/// Cyders - CYD Emulator (Rust Edition)
#[derive(Parser, Debug)]
#[command(name = "cyders")]
#[command(about = "Modern Rust-based emulator for CYD ESP32 development boards", long_about = None)]
#[command(version)]
struct Cli {
    /// Firmware binary file (.bin)
    #[arg(short, long)]
    firmware: Option<String>,

    /// ELF file for symbols (optional)
    #[arg(short, long)]
    elf: Option<String>,

    /// Board model (default: 2432S028R)
    #[arg(short, long, default_value = "2432S028R")]
    board: String,

    /// Window scale factor (default: 2)
    #[arg(short, long, default_value = "2")]
    scale: u32,

    /// SD card image file (optional)
    #[arg(long)]
    sdcard: Option<String>,

    /// Single-core mode (disable Core 1)
    #[arg(long)]
    single_core: bool,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    // Install crash handler (Windows only)
    #[cfg(target_os = "windows")]
    unsafe {
        cyders_crash_handler_install();
    }

    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose {
        "cyders=debug"
    } else {
        "cyders=info"
    };

    tracing_subscriber::fmt()
        .with_env_filter(log_level)
        .with_target(false)
        .init();

    tracing::info!("Cyders - CYD Emulator v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!("Board: {}, Scale: {}x", cli.board, cli.scale);

    // Validate board
    if let Some(board) = board::get_board(&cli.board) {
        tracing::info!("Board config: {} - {} ({})",
            board.model, board.chip, board.display_size);
    } else {
        tracing::warn!("Unknown board model: {} (using default config)", cli.board);
    }

    // Create event loop and window
    // Window size = Display (320) + flexible UI panels on sides
    // Start at 800x240 logical (1600x480 at 2x scale) for comfortable default
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title(format!("Cyders - {} Emulator", cli.board))
        .with_inner_size(winit::dpi::LogicalSize::new(
            800,  // Display + flexible panels (logical pixels)
            240,
        ))
        .with_resizable(true)
        .build(&event_loop)
        .expect("Failed to create window");

    // Create application
    let mut app = App::new(&window, cli.scale, cli.board.clone())?;

    // Load firmware if provided
    if let Some(firmware) = cli.firmware {
        match app.load_firmware(firmware, cli.elf) {
            Ok(()) => tracing::info!("Firmware loaded successfully"),
            Err(e) => {
                tracing::error!("Failed to load firmware: {}", e);
                return Err(e);
            }
        }
    } else {
        tracing::info!("No firmware specified - load via CLI args");
        println!("\nUsage: cyders --firmware <path.bin> [--elf <path.elf>]");
        println!("Example: cyders --firmware firmware.bin --elf firmware.elf\n");
    }

    // Title update timer
    let mut last_title_update = Instant::now();

    // Main event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, window_id } if window_id == window.id() => {
                // Let app handle the event
                app.handle_event(&window, &event);

                // Handle window-level events
                match event {
                    WindowEvent::CloseRequested => {
                        tracing::info!("Window close requested");
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {}
                }
            }

            Event::RedrawRequested(_) => {
                if let Err(e) = app.render(&window) {
                    tracing::error!("Render error: {}", e);
                }
            }

            Event::MainEventsCleared => {
                // Update app state
                app.update();

                // Update window title with stats (every 200ms)
                if last_title_update.elapsed().as_millis() > 200 {
                    app.update_title(&window);
                    last_title_update = Instant::now();
                }

                // Check if exit was requested
                if app.should_exit() {
                    tracing::info!("Exit requested");
                    *control_flow = ControlFlow::Exit;
                }

                // Request redraw
                window.request_redraw();
            }

            _ => {}
        }
    });
}
