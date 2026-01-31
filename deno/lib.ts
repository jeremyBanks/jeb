/**
 * WASM loader and TypeScript-friendly API for zipng.
 */

// Load WASM module
const wasmPath = new URL(
  "../crates/zipng-wasm/pkg/web/zipng_wasm_bg.wasm",
  import.meta.url
);
const wasmModule = await WebAssembly.compileStreaming(fetch(wasmPath));
const wasmInstance = await WebAssembly.instantiate(wasmModule, {});

// Import generated JS bindings
const bindingsPath = new URL(
  "../crates/zipng-wasm/pkg/web/zipng_wasm.js",
  import.meta.url
);
const { encode: wasmEncode, encode_simple: wasmEncodeSimple, version } = await import(
  bindingsPath.href
);

// Initialize WASM
const imports = {
  wbg: wasmInstance.exports,
};

// TypeScript types
export interface FileInput {
  path: string;
  content: number[] | Uint8Array;
}

export interface EncodeOptions {
  mode?: "auto" | "indexed" | "rgba";
  font?: "swiss" | "sixth" | "sky" | "monte" | "sugimori" | "mini" | "micro";
  sort_mode?:
    | "lexicographic"
    | "reverse"
    | "by_size"
    | "by_extension"
    | "none";
}

/**
 * Normalize content to number array for JSON serialization.
 */
function normalizeContent(content: number[] | Uint8Array): number[] {
  return content instanceof Uint8Array ? Array.from(content) : content;
}

/**
 * Encode files into a polyglot PNG+ZIP with full options.
 */
export function encode(
  files: FileInput[],
  options?: EncodeOptions
): Uint8Array {
  const input = {
    files: files.map((f) => ({
      path: f.path,
      content: normalizeContent(f.content),
    })),
    options: options || {
      mode: "auto",
      sort_mode: "lexicographic",
    },
  };

  const inputJson = JSON.stringify(input);
  return wasmEncode(inputJson);
}

/**
 * Encode files with default options (auto mode, lexicographic sort).
 */
export function encodeSimple(files: FileInput[]): Uint8Array {
  const normalized = files.map((f) => ({
    path: f.path,
    content: normalizeContent(f.content),
  }));

  const filesJson = JSON.stringify(normalized);
  return wasmEncodeSimple(filesJson);
}

/**
 * Get the version of zipng-wasm.
 */
export function getVersion(): string {
  return version();
}
