#!/usr/bin/env -S deno run --allow-read --allow-env
/**
 * Deno wrapper for running slop-wasi-demo using Node.js WASI
 *
 * NOTE: This uses Deno's node:wasi compatibility, which currently
 * has better support than the deprecated std/wasi module.
 *
 * Usage:
 *   deno run --allow-read --allow-env run-wasm-node.ts example.json
 *
 * Or with Node.js directly:
 *   node run-wasm-node.mjs example.json
 */

import { WASI } from "node:wasi";
import { readFile } from "node:fs/promises";
import { argv, env } from "node:process";

const WASM_PATH = new URL("./slop-wasi-demo.wasm", import.meta.url).pathname;

async function main() {
  const wasmBinary = await readFile(WASM_PATH);

  const wasi = new WASI({
    args: ["slop-wasi-demo", ...Deno.args],
    env: Deno.env.toObject(),
    preopens: {
      ".": ".",
    },
  });

  const { instance } = await WebAssembly.instantiate(
    wasmBinary,
    wasi.wasiImport,
  );
  wasi.start(instance);
}

if (import.meta.main) {
  await main();
}
