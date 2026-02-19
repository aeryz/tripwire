use std::{env, fs, path::PathBuf, process::Command};

fn main() {
    println!("cargo::rerun-if-changed=wasm-binary");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let wasm_dir = manifest_dir.join("../wasm-binary");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let wasm_out = out_dir.join("wasm_binary.wasm");

    let target_dir = out_dir.join("wasm-target");

    let status = Command::new(env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .current_dir(&wasm_dir)
        .env("CARGO_TARGET_DIR", &target_dir)
        .args(["build", "--release", "--target", "wasm32-unknown-unknown"])
        .status()
        .expect("failed to run cargo build for the wasm binary");

    assert!(status.success(), "guest wasm build failed");

    let built_wasm = target_dir
        .join("wasm32-unknown-unknown")
        .join("release")
        .join("wasm_binary.wasm");

    // Copy into OUT_DIR with a stable name
    fs::copy(&built_wasm, &wasm_out).unwrap_or_else(|e| {
        panic!(
            "copy {} -> {} failed: {e}",
            built_wasm.display(),
            wasm_out.display()
        )
    });

    println!("cargo:rustc-env=WASM_BINARY_PATH={}", wasm_out.display());
}
