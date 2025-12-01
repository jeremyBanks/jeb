import * as generated from "./generated/jeb.js";

function fromUtf8(bytes: Uint8Array): string {
  return new TextDecoder().decode(bytes);
}

export function encodeJeb85(input: Uint8Array): string {
  return fromUtf8(generated.encode_jeb85(input));
}
