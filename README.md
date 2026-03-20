# Cyders - CYD Emulator

A fast, modern ESP32 emulator for testing CYD (Cheap Yellow Display) firmware on your desktop.

## Quick Start

**Just run the launcher:**

```cmd
cd C:\Users\26200.7462\cyders
run-cyders.bat
```

That's it! The launcher will:
- Auto-build the LVGL demo firmware (first time only)
- Launch the emulator with full touch input
- Show a comprehensive interactive demo

## What You'll See

### LVGL Comprehensive Demo

The demo includes 5 interactive screens:

1. **Home Screen** - Navigation hub with 4 buttons
2. **Button Test** - Interactive counter that increments on click
3. **Widget Test** - Slider, switch, checkboxes, progress bar
4. **Animation Test** - Smooth pulsing circle animation
5. **Color Test** - RGB color selector buttons

### Controls

- **Mouse Click** - Simulates touch input
- **ESC** - Quit emulator
- **P** - Pause/Resume
- **F10** - Toggle info panel

### UI Panels

**Left Panel:**
- Registers - CPU register values
- Stack - Call stack and memory
- Locals - Local variables

**Right Panel:**
- Board Info - Hardware configuration
- Performance - MIPS, FPS, cycle count
- CPU State - Program counter, current instruction
- Touch Log - Touch events (Down/Up with coordinates)
- UART Output - Serial debug output from firmware

## Requirements

### Python Environment (Automatic)

**No manual Python installation required!**

The build script uses **UV** (a fast Python package manager) to:
- Download Python 3.11 locally for this project
- Create an isolated environment (no system Python conflicts)
- Install ESP-IDF dependencies automatically

Everything is downloaded and installed locally in `.tools/` folder on first build.

**First build will take 3-5 minutes** to set up the environment.
**Subsequent builds are instant** (just launches the emulator).

### Visual C++ Runtime

If you get a VCRUNTIME140.dll error:

```cmd
winget install Microsoft.VCRedist.2015+.x64
```

Or download from: https://aka.ms/vs/17/release/vc_redist.x64.exe

## How It Works

### First Run (Automatic Setup)

```
[0:00] Check if firmware exists
[0:01] Not found - auto-build starts
[0:05] Download UV package manager (~30 seconds)
[0:35] UV installs Python 3.11 locally (~1 minute)
[1:35] Create isolated virtual environment (~30 seconds)
[2:05] Install ESP-IDF dependencies (~1 minute)
[3:05] Build firmware (2-3 minutes, downloads LVGL)
[5:00] Launch emulator
```

**Total: ~5 minutes (one time only)**

All Python tools are installed locally in `.tools/` folder.
Your system Python is not touched!

### Subsequent Runs

```
[0:00] Check firmware exists - found!
[0:01] Launch emulator
```

**Total: <1 second**

## Testing Touch Input

1. Run `run-cyders.bat`
2. Click anywhere on the display
3. Check **Touch Log panel** (right side) for coordinates
4. Try the interactive demo:
   - Home screen: Click navigation buttons
   - Button test: Click to increment counter
   - Widget test: Drag slider, toggle switch
   - Animation test: Watch the pulsing circle
   - Color test: Click color buttons

All touch events are logged to both the Touch Log panel and UART output!

## Performance

- **40+ MIPS** - Real-time execution speed
- **60 FPS** - Smooth display rendering
- **Full touch input** - Mouse clicks mapped to touchscreen coordinates
- **Dual-core emulation** - Both ESP32 cores running

## Advanced Usage

### Running Custom Firmware

```cmd
run-cyders.bat path\to\firmware.bin path\to\firmware.elf
```

### Building from Source

```cmd
cargo build --release
```

The executable will be in `target\release\cyders.exe`.

### Manual Build of Demo Firmware

```cmd
cd ..\cyd-emulator\test-firmware\60-lvgl-demo
build-smart.bat
```

## Troubleshooting

### "Failed to download uv"

- Check your internet connection
- UV is downloaded from: https://github.com/astral-sh/uv/releases/latest
- The download is ~10MB

### "Failed to install Python 3.11"

- Check your internet connection
- Python is downloaded by UV automatically
- Requires ~50MB of disk space in `.tools/` folder

### Build fails with detailed errors

Run manual build to see full output:
```cmd
cd ..\cyd-emulator\test-firmware\60-lvgl-demo
build-uv.bat
```

### Clean rebuild

Delete the build folder and `.tools/` to start fresh:
```cmd
cd ..\cyd-emulator\test-firmware\60-lvgl-demo
rmdir /s /q build
rmdir /s /q .tools
build-uv.bat
```

### Black screen or no display

1. Check that firmware was built successfully
2. Look for errors in the console
3. Try rebuilding: delete `test-firmware\60-lvgl-demo\build` folder and run again

### Touch input not working

1. Check Touch Log panel (right side) - events should appear when clicking
2. Make sure you're clicking on the display area, not the UI panels
3. Try clicking the button on the demo - should see events in Touch Log and UART Output

## Project Structure

```
cyders/
├── src/               # Rust source code
├── target/            # Build artifacts
│   └── release/
│       └── cyders.exe # Compiled emulator
├── boards.json        # Board configurations
├── run-cyders.bat     # Main launcher
├── README.md          # This file
└── BUILD_DEMO_AUTOMATICALLY.md  # Python 3.11 detailed guide
```

Related folders:
```
cyd-emulator/
├── flexe/             # Xtensa emulator library
└── test-firmware/
    └── 60-lvgl-demo/  # Interactive LVGL demo
        ├── main/
        │   └── main.c         # 640 lines of demo code
        └── build-smart.bat    # Auto-build with Python detection
```

## Architecture

**Display Rendering:**
- `pixels` - Hardware-accelerated pixel buffer
- `winit` - Cross-platform windowing
- 60 FPS with GPU rendering

**Emulator:**
- `flexe` - Xtensa CPU emulator (C library via FFI)
- Dual-core ESP32 support
- Full peripheral emulation

**UI:**
- `egui` - Immediate-mode GUI
- Resizable panels
- Real-time performance monitoring

**Touch Input:**
- Mouse events → TouchState (Rust)
- TouchState → C callback (FFI)
- C callback → Firmware (XPT2046 touch driver)

## Supported Boards

- **2432S028R** - ESP32, 2.8" 320x240, XPT2046 resistive touch, Micro-USB
- **2432S024C** - ESP32, 2.4" 320x240, XPT2046 resistive touch, USB-C
- **3248S035C** - ESP32, 3.5" 480x320, GT911 capacitive touch, USB-C
- **4827S043C** - ESP32-S3, 4.3" 800x480, GT911 capacitive touch, USB-C

Default board: **2432S028R** (most common CYD variant)

## What's Built

The demo firmware tests all major LVGL features:

- **Screen Management** - 5 screens with smooth transitions
- **Buttons** - Click handling, event logging
- **Labels** - Text rendering, dynamic updates
- **Slider** - Touch dragging, value changes
- **Switch** - Toggle on/off
- **Checkbox** - Multi-select
- **Progress Bar** - Animated updates
- **Animations** - Smooth transitions and effects
- **Colors** - Full RGB color support
- **Touch Input** - XPT2046 driver integration
- **Event Handlers** - Full LVGL event system
- **UART Logging** - ESP-IDF logging to console

## Additional Documentation

- **BUILD_DEMO_AUTOMATICALLY.md** - Detailed Python 3.11 setup guide

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Acknowledgments

- **flexe** - Xtensa emulator core (MIT licensed)
- **LVGL** - Embedded graphics library
- **ESP-IDF** - Espressif IoT Development Framework

## Links

- [CYD Hardware](https://github.com/witnessmenow/ESP32-Cheap-Yellow-Display)
- [LVGL Documentation](https://docs.lvgl.io/)
- [ESP-IDF Documentation](https://docs.espressif.com/projects/esp-idf/)
