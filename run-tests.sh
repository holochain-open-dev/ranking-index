CARGO_TARGET_DIR=target cargo build --release --target wasm32-unknown-unknown
hc dna pack example/workdir
WASM_LOG=warn cargo test