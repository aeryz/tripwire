use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    println!("cargo::rerun-if-changed=wasm-runtime");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let wasm_dir = manifest_dir.join("./wasm-runtime");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bin_out = out_dir.join("wasm-runtime");

    let target_dir = out_dir.join("wasm-target");

    let status = Command::new(env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .current_dir(&wasm_dir)
        .env("CARGO_TARGET_DIR", &target_dir)
        .args(["build", "--release"])
        .status()
        .expect("failed to run cargo build for the wasm binary");

    assert!(status.success(), "build failed");

    let built_bin = target_dir.join("release").join("wasm-runtime");

    // Copy into OUT_DIR with a stable name
    fs::copy(&built_bin, &bin_out).unwrap_or_else(|e| {
        panic!(
            "copy {} -> {} failed: {e}",
            built_bin.display(),
            bin_out.display()
        )
    });

    println!("cargo:rustc-env=WASM_RUNTIME_PATH={}", bin_out.display());
}
