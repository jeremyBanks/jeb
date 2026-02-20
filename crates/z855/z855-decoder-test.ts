#!/usr/bin/env -S deno run --allow-read
/**
 * z855-decoder-test.ts — verify z855-decoder.min.mjs passes all decode tests.
 *
 * Tests that the decoder-only build can correctly decode everything the full
 * encoder produces. Uses the full reference impl for encoding, decoder.min for decoding.
 */
import { assertEquals } from "jsr:@std/assert";
import { encode, decode, textEncode, textDecode } from "./z855-reference.ts";

// Import decoder-only build
const decoderUrl = new URL("./z855-decoder.min.mjs", import.meta.url);
const { decode: decodeMin, textDecode: textDecodeMin } = await import(decoderUrl.href);

const str = (b: Uint8Array) => new TextDecoder().decode(b);
const bytes = (s: string) => new TextEncoder().encode(s);

// ── Test helpers ──────────────────────────────────────────────────────────────
let pass = 0, fail = 0;
function test(name: string, fn: () => void) {
  try { fn(); console.log(`  ✓ ${name}`); pass++; }
  catch (e) { console.error(`  ✗ ${name}: ${e}`); fail++; }
}

console.log("\nz855-decoder.min.mjs verification\n");

// ── Round-trip: encode with full, decode with min ─────────────────────────────
console.log("Round-trip (encode full → decode min):");

test("empty input", () => {
  const enc = encode(new Uint8Array(0));
  assertEquals(str(decodeMin(enc)), "");
});

test("all zeros", () => {
  const input = new Uint8Array(100);
  assertEquals(decodeMin(encode(input)).join(","), input.join(","));
});

test("all 0xFF", () => {
  const input = new Uint8Array(100).fill(0xFF);
  assertEquals(decodeMin(encode(input)).join(","), input.join(","));
});

test("safe ASCII passthrough", () => {
  const input = bytes("Hello, World! This is a safe string.");
  assertEquals(str(decodeMin(encode(input))), str(input));
});

test("mixed binary + ASCII", () => {
  const input = new Uint8Array([0, 1, 2, 0xFF, 0xFE, ...bytes("hello"), 0x00]);
  assertEquals(decodeMin(encode(input)).join(","), input.join(","));
});

test("4-byte boundary passthrough (P=0)", () => {
  const input = new Uint8Array([0x41, 0x42, 0x43, 0x44, 0x00, 0x01]);
  assertEquals(decodeMin(encode(input)).join(","), input.join(","));
});

test("short escape 4 bytes mid-block (P=1)", () => {
  const input = new Uint8Array([0xFF, 0x41, 0x42, 0x43, 0x44, 0x00]);
  assertEquals(decodeMin(encode(input)).join(","), input.join(","));
});

test("long passthrough (>= 8 bytes)", () => {
  const input = new Uint8Array([...bytes("ABCDEFGHIJKLMNOP"), 0xFF, 0x00]);
  assertEquals(decodeMin(encode(input)).join(","), input.join(","));
});

test("concatenatable mode", () => {
  const a = encode(bytes("hello"), { concatenatable: true });
  const b = encode(bytes(" world"), { concatenatable: true });
  const combined = new Uint8Array([...a, ...b]);
  assertEquals(str(decodeMin(combined)), "hello world");
});

test("random 1000 bytes", () => {
  const input = new Uint8Array(1000);
  for (let i = 0; i < 1000; i++) input[i] = (i * 137 + 73) & 0xFF;
  assertEquals(decodeMin(encode(input)).join(","), input.join(","));
});

test("random 4999 bytes", () => {
  const input = new Uint8Array(4999);
  for (let i = 0; i < 4999; i++) input[i] = (i * 251 + 17) & 0xFF;
  assertEquals(decodeMin(encode(input)).join(","), input.join(","));
});

// ── textDecode tests ──────────────────────────────────────────────────────────
console.log("\ntextDecode (min):");

test("textDecode basic", () => {
  const encoded = textEncode(bytes("hello world"));
  assertEquals(str(textDecodeMin(encoded)), "hello world");
});

test("textDecode binary round-trip", () => {
  const input = new Uint8Array(256);
  for (let i = 0; i < 256; i++) input[i] = i;
  const encoded = textEncode(input);
  assertEquals(textDecodeMin(encoded).join(","), input.join(","));
});

// ── CLI decode mode ───────────────────────────────────────────────────────────
console.log("\nCLI decode mode:");

async function cliDecode(input: Uint8Array): Promise<Uint8Array> {
  const cmd = new Deno.Command("deno", {
    args: ["run", "--allow-read", "./z855-decoder.min.mjs", "decode"],
    stdin: "piped", stdout: "piped", stderr: "null",
  });
  const proc = cmd.spawn();
  const w = proc.stdin.getWriter();
  await w.write(input); await w.close();
  const { stdout } = await proc.output();
  return stdout;
}

async function cliDecodeLines(input: string): Promise<Uint8Array> {
  const cmd = new Deno.Command("deno", {
    args: ["run", "--allow-read", "./z855-decoder.min.mjs", "decode-lines"],
    stdin: "piped", stdout: "piped", stderr: "null",
  });
  const proc = cmd.spawn();
  const w = proc.stdin.getWriter();
  await w.write(bytes(input)); await w.close();
  const { stdout } = await proc.output();
  return stdout;
}

const cliTestInput = bytes("CLI test: binary \x00\x01\xFF data");
const cliEncoded = encode(cliTestInput);

test("cli decode", async () => {
  const result = await cliDecode(cliEncoded);
  assertEquals(result.join(","), cliTestInput.join(","));
});

const cliLinesInput = new Uint8Array(512);
for (let i = 0; i < 512; i++) cliLinesInput[i] = i & 0xFF;
const cliLinesEncoded = textEncode(cliLinesInput);
// Split into 80-char lines as encode-lines would
const cliLinesText = cliLinesEncoded.match(/.{1,80}/g)!.join("\n") + "\n";

test("cli decode-lines", async () => {
  const result = await cliDecodeLines(cliLinesText);
  assertEquals(result.join(","), cliLinesInput.join(","));
});

// ── Summary ───────────────────────────────────────────────────────────────────
console.log(`\n${pass + fail} tests: ${pass} passed, ${fail} failed`);
if (fail > 0) Deno.exit(1);
console.log("\n✅ z855-decoder.min.mjs passes all decode tests");
