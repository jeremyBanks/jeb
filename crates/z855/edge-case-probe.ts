import { encode, decode } from "./z855-readable.ts";

const te = new TextEncoder();
const td = new TextDecoder();

function hex(arr: Uint8Array) { return [...arr].map(b => b.toString(16).padStart(2, '0')).join(' '); }

// Edge case 1: empty input
console.log("=== Empty input ===");
let enc = encode(new Uint8Array(0));
console.log("encode([]) =", JSON.stringify(enc));
let dec = decode(enc);
console.log("decode('') =", hex(dec));
console.log();

// Edge case 2: 1-5 bytes
console.log("=== 1-5 bytes of 0x42 ===");
for (let n = 1; n <= 5; n++) {
  const input = new Uint8Array(n).fill(0x42);
  const encoded = encode(input);
  const decoded = decode(encoded);
  const match = input.every((v, i) => v === decoded[i]) && input.length === decoded.length;
  console.log(`${n} byte(s): encode=${JSON.stringify(encoded)} match=${match}`);
}
console.log();

// Edge case 3: all zeros
console.log("=== All zeros ===");
for (let n = 1; n <= 8; n++) {
  const input = new Uint8Array(n);
  const encoded = encode(input);
  const decoded = decode(encoded);
  const match = input.every((v, i) => v === decoded[i]) && input.length === decoded.length;
  console.log(`${n} zero(s): encode=${JSON.stringify(encoded)} match=${match}`);
}
console.log();

// Edge case 4: all 0xFF
console.log("=== All 0xFF ===");
for (let n = 1; n <= 8; n++) {
  const input = new Uint8Array(n).fill(0xFF);
  const encoded = encode(input);
  const decoded = decode(encoded);
  const match = input.every((v, i) => v === decoded[i]) && input.length === decoded.length;
  console.log(`${n} x FF: encode=${JSON.stringify(encoded)} match=${match}`);
}
console.log();

// Edge case 5: pure ASCII text
console.log("=== Pure ASCII text ===");
const texts = ["hello", "Hello, World!", "test1234", "abcdefghijklmnopqrstuvwxyz", "ABCDEFGHIJ"];
for (const t of texts) {
  const input = te.encode(t);
  const encoded = encode(input);
  const decoded = decode(encoded);
  const out = td.decode(decoded);
  console.log(`"${t}" -> "${encoded}" -> match=${out === t}`);
}
console.log();

// Edge case 6: roundtrip with random data 0-32 bytes
console.log("=== Roundtrip 0-32 random bytes ===");
let allMatch = true;
for (let len = 0; len <= 32; len++) {
  const input = new Uint8Array(len);
  crypto.getRandomValues(input);
  const encoded = encode(input);
  const decoded = decode(encoded);
  if (input.length !== decoded.length || !input.every((v, i) => v === decoded[i])) {
    console.log(`ROUNDTRIP FAIL at len=${len}: ${hex(input)} -> ${JSON.stringify(encoded)} -> ${hex(decoded)}`);
    allMatch = false;
  }
}
console.log(`Result: ${allMatch ? "ALL PASS" : "SOME FAILED"}`);
console.log();

// Edge case 7: mixed text + binary
console.log("=== Mixed text+binary ===");
const mixed = new Uint8Array([0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x00, 0x01, 0xff, 0x57, 0x6f, 0x72, 0x6c, 0x64, 0x21, 0x00, 0x00]);
const mixedEnc = encode(mixed);
const mixedDec = decode(mixedEnc);
const mixedMatch = mixed.every((v, i) => v === mixedDec[i]) && mixed.length === mixedDec.length;
console.log(`Input: ${hex(mixed)}`);
console.log(`Encoded: "${mixedEnc}"`);
console.log(`Roundtrip: ${mixedMatch}`);
console.log();

// Edge case 8: length comparison with base64
console.log("=== Length comparison: z855 vs base64 ===");
for (const len of [1, 4, 8, 16, 32, 64, 100, 256]) {
  const input = new Uint8Array(len);
  crypto.getRandomValues(input);
  const z855Len = encode(input).length;
  const b64Len = btoa(String.fromCharCode(...input)).length;
  const z85Theoretical = Math.ceil(len * 5 / 4);
  console.log(`${len} bytes: z855=${z855Len} z85_theoretical=${z85Theoretical} base64=${b64Len}`);
}
console.log();

// Edge case 9: output aesthetics for various content types
console.log("=== Aesthetics ===");
const samples = [
  { name: "URL", data: te.encode("https://example.com/path?q=1") },
  { name: "JSON", data: te.encode('{"key":"value","n":42}') },
  { name: "Code", data: te.encode("fn main() { println!(\"hello\"); }") },
  { name: "Email", data: te.encode("user@example.com") },
  { name: "Hex string", data: te.encode("deadbeef01234567") },
  { name: "PNG header", data: new Uint8Array([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) },
  { name: "Null bytes", data: new Uint8Array([0, 0, 0, 0, 0, 0, 0, 0]) },
];
for (const s of samples) {
  const encoded = encode(s.data);
  const decoded = decode(encoded);
  const match = s.data.every((v, i) => v === decoded[i]) && s.data.length === decoded.length;
  console.log(`${s.name}: "${encoded}" (match=${match})`);
}

// Edge case 10: consistency between readable and production
console.log();
console.log("=== Cross-implementation consistency ===");
import { encode as encProd, decode as decProd } from "./z855.ts";
let crossMatch = true;
for (let len = 0; len <= 64; len++) {
  const input = new Uint8Array(len);
  crypto.getRandomValues(input);

  const encReadable = encode(input);
  const encProduction = encProd(input);
  const decReadable = decode(encReadable);
  const decProduction = decProd(encProduction);

  // Both should decode correctly
  const readableRT = decReadable.length === input.length && decReadable.every((v, i) => v === input[i]);
  const prodRT = decProduction.length === input.length && decProduction.every((v, i) => v === input[i]);

  // Cross decode: production decode of readable encode, and vice versa
  const crossDec1 = decProd(new TextEncoder().encode(encReadable));
  const crossDec2 = decode(new TextDecoder().decode(encProduction));
  const crossRT1 = crossDec1.length === input.length && crossDec1.every((v, i) => v === input[i]);
  const crossRT2 = crossDec2.length === input.length && crossDec2.every((v, i) => v === input[i]);

  if (!readableRT || !prodRT || !crossRT1 || !crossRT2) {
    console.log(`FAIL at len=${len}: readable_rt=${readableRT} prod_rt=${prodRT} cross1=${crossRT1} cross2=${crossRT2}`);
    crossMatch = false;
  }
}
console.log(`Cross-impl 0-64 bytes: ${crossMatch ? "ALL PASS" : "SOME FAILED"}`);
