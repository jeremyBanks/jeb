export * as _wasm from "./generated/jeb.js";
import * as _wasm from "./generated/jeb.js";

function fromUtf8(bytes: Uint8Array): string {
  return new TextDecoder().decode(bytes);
}

export function encodeJeb85(input: Uint8Array): string {
  return fromUtf8(_wasm.encode_jeb85(input));
}
