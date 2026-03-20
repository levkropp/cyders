# Build Interactive Demo Automatically

## What's New: UV-Based Build System

`run-cyders.bat` now uses **UV** (a fast Python package manager) for completely isolated Python environments!

## Why UV?

### Old Approach (build-smart.bat)
- ❌ Required manual Python 3.11 installation
- ❌ Could conflict with system Python
- ❌ Needed to manage PATH and multiple Python versions
- ❌ ESP-IDF incompatible with Python 3.14+

### New Approach (build-uv.bat)
- ✅ **No manual Python installation needed**
- ✅ **Completely isolated** - downloads Python 3.11 locally
- ✅ **No system Python conflicts** - your Python 3.14 stays untouched
- ✅ **Automatic dependency management** - UV handles everything
- ✅ **Fast and reliable** - UV is 10-100x faster than pip
- ✅ **Portable** - everything in `.tools/` folder

## How It Works

### First Run

When you run `run-cyders.bat` for the first time:

```
================================================
Firmware not found!
================================================

Building firmware automatically...
This will take 3-5 minutes on first build.

Running UV build script...
This will use an isolated Python environment.

================================================
LVGL Demo Build - Using UV
================================================

ESP-IDF: C:\Users\26200.7462\esp\esp-idf

================================================
Downloading UV Package Manager
================================================

UV is a fast Python package manager that will:
  - Download Python 3.11 locally for this project
  - Create an isolated environment
  - Install ESP-IDF dependencies

Downloading uv...
UV downloaded successfully!

================================================
Installing Python 3.11
================================================

UV will download and install Python 3.11 locally.
This will NOT affect your system Python installation.

This may take 2-3 minutes...

Python 3.11 installed successfully!

================================================
Creating Virtual Environment
================================================

Creating isolated Python environment for ESP-IDF...

Virtual environment created!

================================================
Installing ESP-IDF Tools
================================================

Installing ESP-IDF Python dependencies...
This will take 1-2 minutes on first setup...

ESP-IDF tools installed successfully!

================================================
Building Firmware
================================================

Building LVGL demo firmware...
This will take 2-3 minutes on first build (downloads LVGL)

[... build output ...]

================================================
BUILD SUCCESSFUL!
================================================

Output files:
  Binary: build\lvgl-demo.bin
  ELF:    build\lvgl-demo.elf

Using isolated Python environment at:
  C:\Users\26200.7462\cyd-emulator\test-firmware\60-lvgl-demo\.tools\venv

[Emulator launches automatically]
```

### Subsequent Runs

```
================================================
Cyders - CYD Emulator
================================================
Loading: LVGL Comprehensive Demo

[Emulator launches in <1 second]
```

## What Gets Installed

All in `.tools/` folder (local to the project):

```
test-firmware/60-lvgl-demo/.tools/
├── uv.exe                    # UV package manager (~10MB)
├── python/                   # Python 3.11 installation (~50MB)
│   ├── python.exe
│   ├── python311.dll
│   └── Lib/
└── venv/                     # Virtual environment for ESP-IDF
    ├── Scripts/
    │   ├── python.exe
    │   ├── activate.bat
    │   └── idf.py
    └── Lib/
        └── site-packages/
            ├── esp_idf_kconfig/
            ├── windows_curses/
            └── ... (ESP-IDF dependencies)
```

**Total size: ~200MB** (one-time download)

## Just Run It!

```cmd
cd C:\Users\26200.7462\cyders
run-cyders.bat
```

That's it! No Python installation required.

## Advantages

### Complete Isolation
- **System Python untouched**: Keep using Python 3.14 for other projects
- **No PATH pollution**: Everything in local `.tools/` folder
- **No global packages**: Virtual environment per project
- **Reproducible builds**: Exact Python 3.11 version locked

### Automatic Setup
- **One command**: Just run `run-cyders.bat`
- **No manual downloads**: UV handles everything
- **Smart caching**: UV reuses downloads across projects
- **Fast installs**: 10-100x faster than pip

### Clean Uninstall
Want to remove everything? Just delete one folder:
```cmd
rmdir /s /q .tools
```

All Python tools, dependencies, and caches are gone!

## Timeline

### First Time Setup
```
[0:00] Download UV package manager
[0:30] UV installs Python 3.11
[1:30] Create virtual environment
[2:00] Install ESP-IDF dependencies
[3:00] Build firmware (downloads LVGL)
[5:00] Launch emulator
```
**Total: ~5 minutes (one time)**

### Subsequent Runs
```
[0:00] Check firmware exists
[0:01] Launch emulator
```
**Total: <1 second**

## Troubleshooting

### Download Failures

**Problem**: UV or Python download fails

**Solutions**:
1. Check internet connection
2. Retry - sometimes downloads timeout
3. Check firewall/antivirus settings
4. Delete `.tools/` and start fresh

### Build Failures

**Problem**: Firmware build fails

**Solutions**:
1. Check ESP-IDF is installed at `C:\Users\26200.7462\esp\esp-idf`
2. Delete `build/` folder and rebuild
3. Check console output for specific errors
4. Run manual build for detailed output:
   ```cmd
   cd ..\cyd-emulator\test-firmware\60-lvgl-demo
   build-uv.bat
   ```

### Clean Rebuild

Start completely fresh:
```cmd
cd ..\cyd-emulator\test-firmware\60-lvgl-demo
rmdir /s /q build
rmdir /s /q .tools
cd C:\Users\26200.7462\cyders
run-cyders.bat
```

This will re-download everything and rebuild.

### Disk Space

**Minimum required**: ~500MB free space
- UV: ~10MB
- Python 3.11: ~50MB
- Virtual environment: ~100MB
- ESP-IDF tools: ~200MB
- Build artifacts: ~50MB

### Internet Connection

**First build requires internet** for:
- Downloading UV (~10MB)
- Downloading Python 3.11 (~50MB)
- Installing ESP-IDF packages (~50MB)
- Downloading LVGL component (~5MB)

**Subsequent builds work offline** - everything is cached locally.

## Technical Details

### UV Documentation
- Website: https://docs.astral.sh/uv/
- GitHub: https://github.com/astral-sh/uv
- Written in Rust, 10-100x faster than pip

### Python Version
- **Installed**: Python 3.11.9 (latest 3.11.x)
- **Compatibility**: ESP-IDF v5.1.5 requires Python 3.8-3.11
- **Isolation**: Completely separate from system Python

### Virtual Environment
- **Type**: Standard Python venv (created by UV)
- **Location**: `.tools/venv/`
- **Activation**: Automatic via build script
- **Deactivation**: Not needed - script handles it

### ESP-IDF Integration
- **Version**: ESP-IDF v5.1.5
- **Requirements**: Installed from `esp-idf/requirements.txt`
- **Tools**: xtensa-esp32-elf toolchain, cmake, ninja
- **Environment**: Loaded via `export.bat`

## What You'll Get

**LVGL Comprehensive Demo with:**
- 🏠 Home screen with 4 navigation buttons
- 🎯 Button test - Interactive counter
- 🎛️ Widget test - Slider, switch, checkboxes, progress bar
- ✨ Animation test - Pulsing circle
- 🎨 Color test - RGB color buttons

**All fully interactive with touch input!**

## Comparison: Old vs New

| Feature | build-smart.bat | build-uv.bat |
|---------|----------------|--------------|
| **Python Install** | Manual | Automatic |
| **System Python** | Can conflict | Isolated |
| **Setup Time** | 5 min + manual install | 5 min fully automated |
| **Dependencies** | pip (slow) | uv (10-100x faster) |
| **Clean Uninstall** | Manual cleanup | Delete `.tools/` |
| **Portable** | No (uses system) | Yes (self-contained) |
| **Python 3.14** | Incompatible | No conflict |
| **Reproducible** | Depends on system | Always Python 3.11 |

## Summary

✅ **Zero manual setup** - Just run `run-cyders.bat`
✅ **No Python conflicts** - Completely isolated environment
✅ **Fast and reliable** - UV handles all dependencies
✅ **Clean and portable** - Everything in `.tools/` folder
✅ **Works with any system Python** - 3.14, 3.15, or none at all

**Just double-click run-cyders.bat and it works!** 🎉
