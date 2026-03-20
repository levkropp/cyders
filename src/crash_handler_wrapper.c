/*
 * Crash handler wrapper for Rust FFI
 * Exposes crash handler functions to Rust code
 */

#include <stdio.h>

#ifdef _WIN32

/* Forward declarations from flexe crash_handler.h */
extern void crash_handler_install(void);
extern void crash_handler_set_cpu(void *cpu);

/* Wrapper functions callable from Rust */
void cyders_crash_handler_install(void) {
    printf("[Cyders] Installing Windows crash handler...\n");
    fflush(stdout);
    crash_handler_install();
}

void cyders_crash_handler_set_cpu(void *cpu) {
    crash_handler_set_cpu(cpu);
}

#else

/* Stub for non-Windows platforms */
void cyders_crash_handler_install(void) {}
void cyders_crash_handler_set_cpu(void *cpu) { (void)cpu; }

#endif
