//! Pre-generated FFI bindings for flexe
//!
//! These bindings are manually created to avoid bindgen dependency on Windows.
//! They correspond to the flexe C API as defined in flexe_session.h and xtensa.h.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::os::raw::{c_char, c_int, c_void};

// Opaque types (forward declarations from C)
#[repr(C)]
pub struct flexe_session_t {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct xtensa_cpu_t {
    // Physical AR registers (64 total, 16 visible via window)
    // offset 0, 256 bytes
    pub ar: [u32; 64],

    // offset 256: Hot section
    pub pc: u32,
    pub ccount: u32,
    pub next_timer_event: u32,
    pub windowbase: u32,
    pub ps: u32,
    pub sar: u32,
    pub lbeg: u32,
    pub lend: u32,
    pub lcount: u32,
    pub intenable: u32,
    pub interrupt: u32,
    pub br: u32,
    pub running: bool,
    pub halted: bool,
    pub exception: bool,
    pub _pc_written: bool,
    pub irq_check: bool,
    pub breakpoint_count: c_int,

    // offset ~312: Pointers and cycle count
    _padding: [u8; 4], // Alignment padding before u64
    pub cycle_count: u64,  // This is what we actually want!

    // Rest is opaque (memory pointer, hooks, etc.)
    _private: [u8; 0],
}

#[repr(C)]
pub struct xtensa_mem_t {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct elf_symbols_t {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct esp32_periph_t {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct esp32_rom_stubs_t {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct freertos_stubs_t {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct display_stubs_t {
    _unused: [u8; 0],
}

// pthread_mutex_t - Windows implementation detail:
// In pthreads-win32, this is "typedef struct pthread_mutex_t_ * pthread_mutex_t"
// But we declare it as an opaque struct here for FFI compatibility
// The actual Windows implementation uses a pointer, but our FFI layer treats it as a struct
#[repr(C)]
pub struct pthread_mutex_t {
    _opaque: [u8; 8],  // Size of a pointer on x64 Windows
}

// Callback function types
pub type uart_cb = Option<extern "C" fn(ctx: *mut c_void, byte: u8)>;
pub type touch_fn = Option<extern "C" fn(x: *mut c_int, y: *mut c_int, ctx: *mut c_void) -> c_int>;

// flexe_session_config_t
#[repr(C)]
pub struct flexe_session_config_t {
    // Required
    pub bin_path: *const c_char,

    // Optional - ELF / SD card
    pub elf_path: *const c_char,
    pub sdcard_path: *const c_char,
    pub sdcard_size: u64,

    // CPU configuration
    pub entry_override: u32,
    pub initial_sp: u32,
    pub single_core: c_int,
    pub window_trace: c_int,
    pub spill_verify: c_int,
    pub native_freertos: c_int,

    // UART output callback
    pub uart_cb: uart_cb,
    pub uart_ctx: *mut c_void,

    // Display framebuffer
    pub framebuf: *mut u16,
    pub framebuf_mutex: *mut pthread_mutex_t,
    pub framebuf_w: c_int,
    pub framebuf_h: c_int,

    // Touch input callback
    pub touch_fn: touch_fn,
    pub touch_ctx: *mut c_void,
}

// Extern C functions from flexe
extern "C" {
    // flexe_session.h functions
    pub fn flexe_session_create(cfg: *const flexe_session_config_t) -> *mut flexe_session_t;
    pub fn flexe_session_destroy(s: *mut flexe_session_t);
    pub fn flexe_session_cpu(s: *mut flexe_session_t, core: c_int) -> *mut xtensa_cpu_t;
    pub fn flexe_session_mem(s: *mut flexe_session_t) -> *mut xtensa_mem_t;
    pub fn flexe_session_syms(s: *const flexe_session_t) -> *const elf_symbols_t;
    pub fn flexe_session_periph(s: *mut flexe_session_t) -> *mut esp32_periph_t;
    pub fn flexe_session_rom(s: *mut flexe_session_t) -> *mut esp32_rom_stubs_t;
    pub fn flexe_session_frt(s: *mut flexe_session_t) -> *mut freertos_stubs_t;
    pub fn flexe_session_display(s: *mut flexe_session_t) -> *mut display_stubs_t;
    pub fn flexe_session_is_native_freertos(s: *const flexe_session_t) -> c_int;
    pub fn flexe_session_post_batch(s: *mut flexe_session_t, batch_size: c_int);

    // xtensa.h functions
    pub fn xtensa_run(cpu: *mut xtensa_cpu_t, cycles: c_int) -> c_int;
    pub fn xtensa_step(cpu: *mut xtensa_cpu_t) -> c_int;

    // pthread functions (from pthreadVC3.lib)
    pub fn pthread_mutex_init(mutex: *mut pthread_mutex_t, attr: *const c_void) -> c_int;
    pub fn pthread_mutex_destroy(mutex: *mut pthread_mutex_t) -> c_int;
    pub fn pthread_mutex_lock(mutex: *mut pthread_mutex_t) -> c_int;
    pub fn pthread_mutex_unlock(mutex: *mut pthread_mutex_t) -> c_int;
}
