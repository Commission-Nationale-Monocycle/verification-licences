use std::fs;
use std::fs::exists;
use std::process::Command;

fn main() {
    println!("cargo::rerun-if-changed=src/web/frontend/wasm/src");
    let compilation_path = "target-wasm";
    let pkg_path = "public/static/pkg";
    delete_entity(compilation_path);
    delete_entity(pkg_path);
    let profile = std::env::var("PROFILE").unwrap();
    let profile = profile.as_str();
    let wasm_file_path = &format!(
        "{compilation_path}/wasm32-unknown-unknown/{profile}/wasm_verification_licences.wasm"
    );
    compile_wasm(compilation_path, profile, wasm_file_path);
    generate_bindings(wasm_file_path, pkg_path);
}

fn compile_wasm(compilation_path: &str, profile: &str, wasm_file_path: &str) {
    let target_dir = format!("--target-dir={compilation_path}");
    let mut build_args = vec![
        "build",
        "--target=wasm32-unknown-unknown",
        target_dir.as_str(),
        "--manifest-path=./src/web/frontend/wasm/Cargo.toml",
    ];
    if profile == "release" {
        build_args.push("--release");
    }
    Command::new("cargo")
        .args(build_args)
        .output()
        .expect("Failed to compile frontend.");

    assert!(exists(wasm_file_path).is_ok_and(|exists| exists));
}

/// Generate JS & TS bindings
fn generate_bindings(wasm_file_path: &str, pkg_path: &str) {
    let out_dir_param = format!("--out-dir={pkg_path}");
    let wasm_bindgen_args = ["--target=web", out_dir_param.as_str(), wasm_file_path];
    Command::new("wasm-bindgen")
        .args(wasm_bindgen_args)
        .output()
        .expect("Failed to generate WASM wrappers.");

    assert!(
        exists(format!("{pkg_path}/wasm_verification_licences.js")).is_ok_and(|exists| exists)
    );
}

fn delete_entity(compilation_path: &str) {
    match fs::metadata(compilation_path) {
        Ok(metadata) if metadata.is_dir() => fs::remove_dir_all(compilation_path)
            .expect(&format!("Couldn't delete {compilation_path}")),
        Ok(_) => {
            fs::remove_file(compilation_path).expect(&format!("Couldn't delete {compilation_path}"))
        }
        Err(_) => {}
    }
}
