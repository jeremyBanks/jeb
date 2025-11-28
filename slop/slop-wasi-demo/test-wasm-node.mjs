#!/usr/bin/env node
/**
 * Test script to run the WASM binary using Node.js WASI support
 * This verifies the WASM works correctly before testing with Deno
 */

import { readFile } from 'node:fs/promises';
import { WASI } from 'node:wasi';
import { argv, env } from 'node:process';

const wasi = new WASI({
  version: 'preview1',
  args: ['slop-wasi-demo', ...argv.slice(2)],
  env,
  preopens: {
    '.': '.',
  },
});

const wasmPath = './slop-wasi-demo.wasm';
const wasmBinary = await readFile(wasmPath);
const { instance } = await WebAssembly.instantiate(wasmBinary, {
  wasi_snapshot_preview1: wasi.wasiImport,
});

wasi.start(instance);
