#/bin/bash
VER=1.69.0
rustup toolchain install $VER
rustup default $VER
rustup target add wasm32-unknown-unknown
cargo build -p buyback --target wasm32-unknown-unknown --release