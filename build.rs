use std::path::PathBuf;

fn main() {
    let openjph_root = PathBuf::from("vendor/openjph/src/core");

    // Collect all OpenJPH source files
    let mut cpp_files: Vec<PathBuf> = Vec::new();
    let mut c_files: Vec<PathBuf> = Vec::new();

    for subdir in &["codestream", "coding", "others", "transform"] {
        let dir = openjph_root.join(subdir);
        for entry in std::fs::read_dir(&dir).expect(&format!("read {}", dir.display())) {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_str().unwrap();

            // Skip WASM-specific files
            if name.contains("_wasm") {
                continue;
            }

            // On ARM (Apple Silicon), skip x86 SIMD files
            #[cfg(target_arch = "aarch64")]
            {
                if name.contains("_sse") || name.contains("_avx") {
                    continue;
                }
            }

            // On x86, skip ARM NEON files (there aren't any named _neon, but just in case)
            #[cfg(target_arch = "x86_64")]
            {
                if name.contains("_neon") {
                    continue;
                }
            }

            if path.extension().map(|e| e == "cpp").unwrap_or(false) {
                cpp_files.push(path);
            } else if path.extension().map(|e| e == "c").unwrap_or(false) {
                c_files.push(path);
            }
        }
    }

    // Add our shim
    cpp_files.push(PathBuf::from("vendor/openjph_shim/ojph_shim.cpp"));

    // Include directories
    let include_dirs = [
        openjph_root.join("openjph"),
        PathBuf::from("vendor/openjph_shim"),
    ];

    // Build C++ sources
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .std("c++14")
        .warnings(false);

    for dir in &include_dirs {
        build.include(dir);
    }

    // SIMD flags for x86_64
    #[cfg(target_arch = "x86_64")]
    {
        // The _sse, _sse2, _avx, _avx2, _avx512 files need their respective flags.
        // Since cc::Build applies flags to all files, we compile SIMD files separately.
        // For now, compile only generic files + shim with the main build.
    }

    for file in &cpp_files {
        build.file(file);
    }

    build.compile("openjph");

    // Build C sources separately
    if !c_files.is_empty() {
        let mut c_build = cc::Build::new();
        c_build.warnings(false);
        for dir in &include_dirs {
            c_build.include(dir);
        }
        for file in &c_files {
            c_build.file(file);
        }
        c_build.compile("openjph_c");
    }

    // Link C++ standard library
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-lib=c++");

    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-lib=stdc++");

    println!("cargo:rerun-if-changed=vendor/openjph_shim/ojph_shim.cpp");
    println!("cargo:rerun-if-changed=vendor/openjph_shim/ojph_shim.h");
}
