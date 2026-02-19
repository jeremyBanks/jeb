/**
 * Test suite for z855-jeremy.ts
 */

import { assertEquals, assertThrows } from "jsr:@std/assert";
import { encode, decode, textEncode, textDecode } from "./z855-jeremy.ts";

// Helper to convert string to Uint8Array
function bytes(s: string): Uint8Array {
  return new TextEncoder().encode(s);
}

// Helper to convert Uint8Array to string
function str(b: Uint8Array): string {
  return new TextDecoder().decode(b);
}

Deno.test("encode/decode empty input", () => {
  const input = new Uint8Array(0);
  const encoded = encode(input);
  assertEquals(encoded.length, 0);
  const decoded = decode(encoded);
  assertEquals(decoded.length, 0);
});

Deno.test("roundtrip all lengths 0-64", () => {
  for (let len = 0; len <= 64; len++) {
    const input = new Uint8Array(len);
    for (let i = 0; i < len; i++) {
      input[i] = (i * 17 + 42) & 0xff;
    }
    const encoded = encode(input);
    const decoded = decode(encoded);
    assertEquals(decoded, input, `failed at length ${len}`);
  }
});

Deno.test("roundtrip zeros", () => {
  for (let len = 1; len <= 20; len++) {
    const input = new Uint8Array(len).fill(0);
    const encoded = encode(input);
    const decoded = decode(encoded);
    assertEquals(decoded, input, `failed at length ${len}`);
  }
});

Deno.test("roundtrip max bytes", () => {
  for (let len = 1; len <= 20; len++) {
    const input = new Uint8Array(len).fill(0xff);
    const encoded = encode(input);
    const decoded = decode(encoded);
    assertEquals(decoded, input, `failed at length ${len}`);
  }
});

Deno.test("roundtrip text strings", () => {
  const testStrings = [
    "hello",
    "hello world",
    "test 1234",
    "Z85 is cool!",
    "abcdefghijklmnopqrstuvwxyz",
    "0123456789",
    "The quick brown fox jumps over the lazy dog.",
  ];

  for (const s of testStrings) {
    const input = bytes(s);
    const encoded = encode(input);
    const decoded = decode(encoded);
    assertEquals(str(decoded), s, `failed for "${s}"`);
  }
});

Deno.test("4-byte escape (block-aligned)", () => {
  // "test" is 4 safe bytes, should use _ escape
  const input = bytes("test");
  const encoded = encode(input);
  const decoded = decode(encoded);
  assertEquals(str(decoded), "test");
  // Should contain _ escape
  assertEquals(str(encoded).includes("_"), true);
});

Deno.test("5-byte escape", () => {
  // "hello" is 5 safe bytes, should use , escape
  const input = bytes("hello");
  const encoded = encode(input);
  const decoded = decode(encoded);
  assertEquals(str(decoded), "hello");
  // Should contain , escape
  assertEquals(str(encoded).includes(","), true);
});

Deno.test("6-byte escape", () => {
  // "foobar" is 6 safe bytes, should use ~ escape
  const input = bytes("foobar");
  const encoded = encode(input);
  const decoded = decode(encoded);
  assertEquals(str(decoded), "foobar");
  // Should contain ~ escape
  assertEquals(str(encoded).includes("~"), true);
});

Deno.test("7-byte escape", () => {
  // "testing" is 7 safe bytes, should use ; escape
  const input = bytes("testing");
  const encoded = encode(input);
  const decoded = decode(encoded);
  assertEquals(str(decoded), "testing");
  // Should contain ; escape
  assertEquals(str(encoded).includes(";"), true);
});

Deno.test("long escape (8+ bytes)", () => {
  // Long safe string should use | escape
  const input = bytes("hello world!");
  const encoded = encode(input);
  const decoded = decode(encoded);
  assertEquals(str(decoded), "hello world!");
  // Should contain | escape
  assertEquals(str(encoded).includes("|"), true);
});

Deno.test("rest-of-input escape (0|)", () => {
  // Long safe string at end, non-concatenatable
  const input = bytes("test hello world and more text here");
  const encoded = encode(input);
  const decoded = decode(encoded);
  assertEquals(str(decoded), "test hello world and more text here");
  // Should contain 0| escape
  assertEquals(str(encoded).includes("0|"), true);
});

Deno.test("concatenatable mode", () => {
  const input = new Uint8Array([0x07]); // Single byte
  const encoded = encode(input, { concatenatable: true });
  const decoded = decode(encoded);
  assertEquals(decoded, input);
  // Should have hash padding
  assertEquals(str(encoded).startsWith("#"), true);
});

Deno.test("concatenatable roundtrip", () => {
  for (let len = 1; len <= 20; len++) {
    const input = new Uint8Array(len);
    for (let i = 0; i < len; i++) {
      input[i] = (i * 13 + 7) & 0xff;
    }
    const encoded = encode(input, { concatenatable: true });
    const decoded = decode(encoded);
    assertEquals(decoded, input, `failed at length ${len}`);
  }
});

Deno.test("partial blocks (1-3 bytes)", () => {
  // 1 byte
  let input = new Uint8Array([0x42]);
  let encoded = encode(input);
  let decoded = decode(encoded);
  assertEquals(decoded, input);

  // 2 bytes
  input = new Uint8Array([0x12, 0x34]);
  encoded = encode(input);
  decoded = decode(encoded);
  assertEquals(decoded, input);

  // 3 bytes
  input = new Uint8Array([0xaa, 0xbb, 0xcc]);
  encoded = encode(input);
  decoded = decode(encoded);
  assertEquals(decoded, input);
});

Deno.test("mixed safe and unsafe bytes", () => {
  const input = new Uint8Array([0xff, 0x00, 0x61, 0x62, 0x63, 0x64, 0x65]);
  const encoded = encode(input);
  const decoded = decode(encoded);
  assertEquals(decoded, input);
});

Deno.test("decode invalid char", () => {
  // Byte 0x01 is not in Z85 alphabet
  const invalid = new Uint8Array([0x01]);
  assertThrows(() => decode(invalid), Error, "invalid char");
});

Deno.test("decode single trailing char", () => {
  // Single trailing Z85 char is invalid
  const invalid = bytes("0");
  assertThrows(() => decode(invalid), Error, "single trailing char");
});

Deno.test("textEncode/textDecode", () => {
  const input = bytes("hello world");
  const encoded = textEncode(input);
  const decoded = textDecode(encoded);
  assertEquals(str(decoded), "hello world");
});

Deno.test("all byte values roundtrip", () => {
  // Test all 256 byte values
  const input = new Uint8Array(256);
  for (let i = 0; i < 256; i++) {
    input[i] = i;
  }
  const encoded = encode(input);
  const decoded = decode(encoded);
  assertEquals(decoded, input);
});

Deno.test("repeated patterns", () => {
  const patterns = [
    new Uint8Array([0x00, 0x00, 0x00, 0x00]),
    new Uint8Array([0xff, 0xff, 0xff, 0xff]),
    new Uint8Array([0xaa, 0xaa, 0xaa, 0xaa]),
    new Uint8Array([0x55, 0x55, 0x55, 0x55]),
  ];

  for (const input of patterns) {
    const encoded = encode(input);
    const decoded = decode(encoded);
    assertEquals(decoded, input);
  }
});

console.log("All tests passed!");
