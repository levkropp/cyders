use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=../cyd-emulator/flexe");
    println!("cargo:rerun-if-changed=src/emu_stubs.c");
    println!("cargo:rerun-if-changed=src/crash_handler_wrapper.c");

    // Compile C stubs
    let mut builder = cc::Build::new();
    builder.file("src/emu_stubs.c");

    // Add crash handler wrapper on Windows
    #[cfg(target_os = "windows")]
    {
        builder.file("src/crash_handler_wrapper.c");
        builder.include("../cyd-emulator/flexe/src");
    }

    builder.compile("emu_stubs");

    // Determine the flexe library path based on the platform and build type
    let flexe_lib_path = if cfg!(target_os = "windows") {
        "../cyd-emulator/build/flexe/Release"
    } else {
        "../cyd-emulator/build/flexe"
    };

    // Tell cargo to link against flexe
    println!("cargo:rustc-link-search=native={}", flexe_lib_path);
    println!("cargo:rustc-link-lib=static=xtensa-emu-lib");

    // DYNAMIC LINKING with /MD CRT - Consistent with flexe build
    #[cfg(target_os = "windows")]
    {
        // Use dynamic vcpkg libraries (x64-windows triplet, /MD CRT)
        // This matches flexe's CMAKE_MSVC_RUNTIME_LIBRARY = /MD
        println!("cargo:rustc-link-search=native=C:/vcpkg/installed/x64-windows/lib");
        println!("cargo:rustc-link-lib=dylib=zlib");
        println!("cargo:rustc-link-lib=dylib=pthreadVC3");

        // OpenSSL from vcpkg x64-windows (dynamic libs)
        println!("cargo:rustc-link-lib=dylib=libcrypto");
        println!("cargo:rustc-link-lib=dylib=libssl");

        // Windows system libraries needed by OpenSSL and other deps
        println!("cargo:rustc-link-lib=dylib=crypt32");
        println!("cargo:rustc-link-lib=dylib=ws2_32");
        println!("cargo:rustc-link-lib=dylib=advapi32");
        println!("cargo:rustc-link-lib=dylib=user32");
        println!("cargo:rustc-link-lib=dylib=bcrypt");
    }

    #[cfg(not(target_os = "windows"))]
    {
        // Unix-like systems - static linking
        println!("cargo:rustc-link-lib=static=pthread");
        println!("cargo:rustc-link-lib=static=crypto");
        println!("cargo:rustc-link-lib=static=ssl");
    }

    println!("cargo:warning=Dynamic linking with /MD CRT - ensures consistency with flexe");

    // Copy required DLLs to output directory on Windows
    #[cfg(target_os = "windows")]
    {
        let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
        let out_dir = Path::new("target").join(&profile);

        // Ensure output directory exists
        fs::create_dir_all(&out_dir).ok();

        let vcpkg_bin = Path::new("C:/vcpkg/installed/x64-windows/bin");
        let dlls = ["pthreadVC3.dll", "zlib1.dll", "libcrypto-3-x64.dll", "libssl-3-x64.dll"];

        for dll in &dlls {
            let src = vcpkg_bin.join(dll);
            let dst = out_dir.join(dll);
            if src.exists() {
                match fs::copy(&src, &dst) {
                    Ok(_) => println!("cargo:warning=Copied {} to output directory", dll),
                    Err(e) => println!("cargo:warning=Failed to copy {}: {}", dll, e),
                }
            } else {
                println!("cargo:warning=DLL not found: {}", src.display());
            }
        }
    }
}
