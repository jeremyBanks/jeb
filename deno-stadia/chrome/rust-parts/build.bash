#!/bin/bash
set -veuo pipefail

# apt install gcc-mingw-w64-x86-64
# rustup toolchain install stable-x86_64-pc-windows-gnu
# rustup target add x86_64-pc-windows-gnu

cargo build --bin windows --target=x86_64-pc-windows-gnu
wasm-pack build --out-dir target/wasm_pkg
