cargo build --target=wasm32-unknown-unknown --target-dir=./wasm/target --manifest-path=./wasm/Cargo.toml
wasm-bindgen --target=web --out-dir=public/static/pkg ./wasm/target/wasm32-unknown-unknown/debug/wasm.wasm
