//! Emulator thread management
//!
//! Runs flexe in a background thread with command/control via channels.

use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use parking_lot::Mutex;
use crossbeam::channel::{self, Sender, Receiver};
use anyhow::Result;

use crate::flexe_ffi::{FlexeSession, SessionConfig};
use crate::input::TouchState;

/// Commands that can be sent to the emulator thread
#[derive(Debug, Clone)]
pub enum EmulatorCommand {
    /// Pause execution
    Pause,
    /// Resume execution
    Resume,
    /// Stop the emulator thread
    Stop,
}

/// Emulator state shared between threads
struct EmulatorState {
    running: bool,
    paused: bool,
    cycle_count: u64,
    pc: u32,
    mips: f64,
    fps: f64,
    halted: bool,
    registers: [u32; 16],
    windowbase: u32,
    intlevel: u32,
}

/// Emulator manager (runs in main thread, controls background emulator thread)
pub struct Emulator {
    framebuffer: Arc<Mutex<Vec<u16>>>,
    cmd_tx: Sender<EmulatorCommand>,
    state: Arc<Mutex<EmulatorState>>,
    thread_handle: Option<thread::JoinHandle<()>>,
    /// Shared reference to the flexe session (for symbol lookups, etc.)
    session: Arc<Mutex<Option<Arc<FlexeSession>>>>,
}

impl Emulator {
    /// Create and start a new emulator instance
    pub fn new(
        config: SessionConfig,
        framebuffer: Arc<Mutex<Vec<u16>>>,
        touch_state: Option<Arc<Mutex<TouchState>>>,
    ) -> Result<Self> {
        let (cmd_tx, cmd_rx) = channel::unbounded();

        let state = Arc::new(Mutex::new(EmulatorState {
            running: true,
            paused: false,
            cycle_count: 0,
            pc: 0,
            mips: 0.0,
            fps: 0.0,
            halted: false,
            registers: [0; 16],
            windowbase: 0,
            intlevel: 0,
        }));

        let state_clone = state.clone();
        let fb_clone = framebuffer.clone();
        let session_shared = Arc::new(Mutex::new(None));
        let session_clone = session_shared.clone();

        // Spawn emulator thread
        let thread_handle = thread::Builder::new()
            .name("emulator".to_string())
            .spawn(move || {
                emulator_thread(config, fb_clone, touch_state, cmd_rx, state_clone, session_clone);
            })?;

        Ok(Self {
            framebuffer,
            cmd_tx,
            state,
            thread_handle: Some(thread_handle),
            session: session_shared,
        })
    }

    /// Pause emulation
    pub fn pause(&self) {
        self.cmd_tx.send(EmulatorCommand::Pause).ok();
        self.state.lock().paused = true;
    }

    /// Resume emulation
    pub fn resume(&self) {
        self.cmd_tx.send(EmulatorCommand::Resume).ok();
        self.state.lock().paused = false;
    }

    /// Check if emulator is running (not paused)
    pub fn is_running(&self) -> bool {
        !self.state.lock().paused
    }

    /// Get current cycle count
    pub fn cycle_count(&self) -> u64 {
        self.state.lock().cycle_count
    }

    /// Get current PC
    pub fn pc(&self) -> u32 {
        self.state.lock().pc
    }

    /// Get current MIPS
    pub fn mips(&self) -> f64 {
        self.state.lock().mips
    }

    /// Get current FPS (frames per second)
    pub fn fps(&self) -> f64 {
        self.state.lock().fps
    }

    /// Check if CPU is halted
    pub fn is_halted(&self) -> bool {
        self.state.lock().halted
    }

    /// Get a register value (simplified - would need FFI access)
    pub fn get_register(&self, _reg: u32) -> u32 {
        // TODO: Add FFI to read register values
        0
    }

    /// Get all registers (a0-a15) from cached state
    pub fn get_all_registers(&self) -> [u32; 16] {
        self.state.lock().registers
    }

    /// Look up symbol for an address (requires direct session access)
    pub fn lookup_symbol(&self, addr: u32) -> Option<crate::flexe_ffi::SymbolInfo> {
        if let Some(session) = self.session.lock().as_ref() {
            session.lookup_symbol(addr)
        } else {
            None
        }
    }

    /// Get WINDOWBASE register from cached state
    pub fn get_windowbase(&self) -> u32 {
        self.state.lock().windowbase
    }

    /// Get INTLEVEL from cached state
    pub fn get_intlevel(&self) -> u32 {
        self.state.lock().intlevel
    }
}

impl Drop for Emulator {
    fn drop(&mut self) {
        // Send stop command
        self.cmd_tx.send(EmulatorCommand::Stop).ok();

        // Wait for thread to exit
        if let Some(handle) = self.thread_handle.take() {
            handle.join().ok();
        }
    }
}

/// Emulator thread - runs the flexe session
fn emulator_thread(
    config: SessionConfig,
    framebuffer: Arc<Mutex<Vec<u16>>>,
    touch_state: Option<Arc<Mutex<TouchState>>>,
    cmd_rx: Receiver<EmulatorCommand>,
    state: Arc<Mutex<EmulatorState>>,
    session_shared: Arc<Mutex<Option<Arc<FlexeSession>>>>,
) {
    tracing::info!("Emulator thread starting");

    // Create flexe session
    let framebuf_mutex = Arc::new(Mutex::new(()));
    let session = match FlexeSession::new(config, framebuffer, framebuf_mutex, touch_state) {
        Ok(s) => Arc::new(s),
        Err(e) => {
            tracing::error!("Failed to create flexe session: {}", e);
            state.lock().running = false;
            return;
        }
    };

    // Store session for main thread access
    *session_shared.lock() = Some(session.clone());

    tracing::info!("Flexe session created successfully");

    let mut paused = false;
    let batch_size = 10_000u32; // 10K instruction batches
    let mut last_perf_update = Instant::now();
    let mut cycles_since_perf = 0u64;
    let mut batches_since_perf = 0u64;

    while state.lock().running {
        // Check for commands (non-blocking)
        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                EmulatorCommand::Pause => {
                    tracing::info!("Emulator paused");
                    paused = true;
                }
                EmulatorCommand::Resume => {
                    tracing::info!("Emulator resumed");
                    paused = false;
                    last_perf_update = Instant::now();
                    cycles_since_perf = 0;
                    batches_since_perf = 0;
                }
                EmulatorCommand::Stop => {
                    tracing::info!("Emulator stopping");
                    state.lock().running = false;
                    break;
                }
            }
        }

        if paused {
            // Sleep while paused to reduce CPU usage
            thread::sleep(Duration::from_millis(10));
            continue;
        }

        // Run a batch of instructions
        let cycles_executed = session.run_batch(batch_size);

        if cycles_executed < 0 {
            tracing::error!("Emulator error: run_batch returned {}", cycles_executed);
            state.lock().halted = true;
            paused = true;
            continue;
        }

        // Post-batch sync (core 1, preemption, etc.)
        session.post_batch(batch_size as i32);

        // Update state
        cycles_since_perf += cycles_executed as u64;
        batches_since_perf += 1;

        let cycle_count = session.cycle_count();
        let pc = session.pc();
        let halted = session.is_halted();

        // Debug: Log first few updates to verify state is being read correctly
        static mut UPDATE_COUNT: u64 = 0;
        unsafe {
            UPDATE_COUNT += 1;
            if UPDATE_COUNT < 5 || UPDATE_COUNT % 1000 == 0 {
                tracing::info!("State update #{}: cycles={}, pc=0x{:08X}, halted={}, executed={}",
                    UPDATE_COUNT, cycle_count, pc, halted, cycles_executed);
            }
        }

        let mut s = state.lock();
        s.cycle_count = cycle_count;
        s.pc = pc;
        s.halted = halted;
        s.registers = session.get_all_registers();
        s.windowbase = session.get_windowbase();
        s.intlevel = session.get_intlevel();
        drop(s);

        // Update performance metrics every second
        let elapsed = last_perf_update.elapsed();
        if elapsed >= Duration::from_secs(1) {
            let secs = elapsed.as_secs_f64();
            let mips = (cycles_since_perf as f64) / secs / 1_000_000.0;
            let fps = (batches_since_perf as f64) / secs;

            let mut s = state.lock();
            s.mips = mips;
            s.fps = fps;
            drop(s);

            tracing::debug!(
                "Performance: {:.2} MIPS, {:.1} batches/sec, {} cycles",
                mips, fps, cycles_since_perf
            );

            last_perf_update = Instant::now();
            cycles_since_perf = 0;
            batches_since_perf = 0;
        }

        // Small yield to prevent thread starvation
        thread::yield_now();
    }

    tracing::info!("Emulator thread exiting");
}
