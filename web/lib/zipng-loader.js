/**
 * WASM loader for zipng in the browser.
 */

let wasmModule = null;
let wasmExports = null;

/**
 * Initialize the WASM module.
 */
export async function initWasm() {
  if (wasmModule) {
    return wasmExports;
  }

  // Load WASM module
  const wasmPath = "../../crates/zipng-wasm/pkg/web/zipng_wasm_bg.wasm";
  const response = await fetch(wasmPath);
  const wasmBytes = await response.arrayBuffer();
  const wasmObj = await WebAssembly.instantiate(wasmBytes, {});

  wasmModule = wasmObj.module;
  wasmExports = wasmObj.instance.exports;

  // Import JS bindings
  const { default: init, encode, encode_simple, version } = await import(
    "../../crates/zipng-wasm/pkg/web/zipng_wasm.js"
  );

  // Initialize with our WASM instance
  await init(wasmModule);

  return { encode, encode_simple, version };
}
