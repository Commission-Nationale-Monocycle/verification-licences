use std::process::Command;

fn main() {
    println!("cargo::rerun-if-changed=src/web/frontend/wasm/src");
    let profile = std::env::var("PROFILE").unwrap();
    let mut build_args = vec![
        "build",
        "--target=wasm32-unknown-unknown",
        "--target-dir=target-wasm",
        "--manifest-path=./src/web/frontend/wasm/Cargo.toml",
    ];
    if profile == "release" {
        build_args.push("--release");
    }
    Command::new("cargo")
        .args(build_args)
        .output()
        .expect("Failed to compile frontend.");

    let wasm_file_path =
        format!("target-wasm/wasm32-unknown-unknown/{profile}/wasm_verification_licences.wasm");
    let wasm_bindgen_args = [
        "--target=web",
        "--out-dir=public/static/pkg",
        wasm_file_path.as_str(),
    ];
    Command::new("wasm-bindgen")
        .args(wasm_bindgen_args)
        .output()
        .expect("Failed to generate WASM wrappers.");
}
