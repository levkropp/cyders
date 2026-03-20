/*
 * emu_stubs.c - Stubs for symbols expected by flexe
 *
 * These are callbacks/globals that flexe expects from the host application.
 */

#include <stdint.h>

/* Global flag indicating if the emulator app is running.
 * Used by touch_stubs.c in flexe to determine if touch input should be processed.
 */
int emu_app_running = 1;
