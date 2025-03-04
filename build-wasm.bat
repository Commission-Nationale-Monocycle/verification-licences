cargo build --target=wasm32-unknown-unknown --target-dir=target-wasm --manifest-path=./src/web/frontend/wasm/Cargo.toml
wasm-bindgen --target=web --out-dir=public/static/pkg target-wasm/wasm32-unknown-unknown/debug/wasm_verification_licences.wasm
