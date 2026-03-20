/*
 * wrapper.h — FFI wrapper for flexe C API
 *
 * This header includes all the flexe headers needed for Rust bindings.
 * Used by build.rs with bindgen to generate Rust FFI bindings.
 */

#ifndef WRAPPER_H
#define WRAPPER_H

/* Core flexe headers */
#include "../cyd-emulator/flexe/src/flexe_session.h"
#include "../cyd-emulator/flexe/src/xtensa.h"
#include "../cyd-emulator/flexe/src/memory.h"
#include "../cyd-emulator/flexe/src/peripherals.h"
#include "../cyd-emulator/flexe/src/elf_symbols.h"
#include "../cyd-emulator/flexe/src/rom_stubs.h"
#include "../cyd-emulator/flexe/src/freertos_stubs.h"
#include "../cyd-emulator/flexe/src/display_stubs.h"

/* System headers needed for types */
#include <stdint.h>
#include <pthread.h>

#endif /* WRAPPER_H */
