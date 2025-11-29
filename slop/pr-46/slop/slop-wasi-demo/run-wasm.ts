#!/usr/bin/env -S deno run --allow-read --allow-env
/**
 * Deno wrapper for running slop-wasi-demo as WASM using Node.js WASI compatibility
 *
 * This demonstrates how to distribute a Rust CLI as a Deno program using WASI.
 *
 * Usage:
 *   deno run --allow-read --allow-env run-wasm.ts
 *   deno run --allow-read --allow-env run-wasm.ts input.json
 *   deno run --allow-read --allow-env run-wasm.ts input.json --compact
 *
 * Or install it:
 *   deno install -A -n slop-wasi run-wasm.ts
 *   slop-wasi input.json
 */

import { WASI } from "node:wasi";
import { readFile } from "node:fs/promises";

// Path to the WASM binary (relative to this script)
const WASM_PATH = new URL("./slop-wasi-demo.wasm", import.meta.url).pathname;

async function main() {
  // Read the WASM binary
  let wasmBinary: Uint8Array;
  try {
    wasmBinary = await readFile(WASM_PATH);
  } catch (error) {
    console.error(`Error: Could not find WASM binary at ${WASM_PATH}`);
    console.error("Please build it first with:");
    console.error("  cargo build --target wasm32-wasip1 --release");
    console.error("  cp ../../target/wasm32-wasip1/release/slop-wasi-demo.wasm .");
    Deno.exit(1);
  }

  // Create WASI instance using Node.js compatibility
  const wasi = new WASI({
    args: ["slop-wasi-demo", ...Deno.args],
    env: Deno.env.toObject(),
    preopens: {
      ".": ".",
    },
  });

  // Compile and instantiate the WASM module
  const module = await WebAssembly.compile(wasmBinary);
  const instance = await WebAssembly.instantiate(module, wasi.wasiImport);

  // Start the WASI program
  wasi.start(instance);
}

// Run the program
if (import.meta.main) {
  await main();
}
