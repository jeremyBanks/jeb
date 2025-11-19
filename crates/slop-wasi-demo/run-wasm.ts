#!/usr/bin/env -S deno run --allow-read --allow-env
/**
 * Deno wrapper for running slop-wasi-demo as WASM
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

import Context from "https://deno.land/std@0.204.0/wasi/snapshot_preview1.ts";

// Path to the WASM binary (relative to this script)
const WASM_PATH = new URL("./slop-wasi-demo.wasm", import.meta.url).pathname;

async function main() {
  // Read the WASM binary
  let wasmBinary: Uint8Array;
  try {
    wasmBinary = await Deno.readFile(WASM_PATH);
  } catch (error) {
    console.error(`Error: Could not find WASM binary at ${WASM_PATH}`);
    console.error("Please build it first with:");
    console.error("  cargo build --target wasm32-wasip1 --release");
    console.error("  cp ../../target/wasm32-wasip1/release/slop-wasi-demo.wasm .");
    Deno.exit(1);
  }

  // Create WASI context with environment and arguments
  const context = new Context({
    args: ["slop-wasi-demo", ...Deno.args],
    env: Deno.env.toObject(),
    stdin: Deno.stdin.rid,
    stdout: Deno.stdout.rid,
    stderr: Deno.stderr.rid,
  });

  // Compile and instantiate the WASM module
  const module = await WebAssembly.compile(wasmBinary);
  const instance = await WebAssembly.instantiate(module, {
    wasi_snapshot_preview1: context.exports,
  });

  // Start the WASI program
  try {
    context.start(instance);
  } catch (error) {
    // WASI programs exit by throwing, check if it's a normal exit
    if (error instanceof Error && error.message.includes("unreachable")) {
      // Normal exit - WASI uses unreachable for exit(0)
      Deno.exit(0);
    }
    throw error;
  }
}

// Run the program
if (import.meta.main) {
  await main();
}
