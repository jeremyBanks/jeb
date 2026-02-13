import { assertEquals, assertThrows } from "@std/assert";
import { encode, decode, Z85DecodeError } from "./z85.ts";

// Unit tests for Z85 encode/decode

Deno.test("encode empty", () => {
  assertEquals(encode(new Uint8Array(0)), "");
});

Deno.test("decode empty", () => {
  assertEquals(decode(""), new Uint8Array(0));
});

Deno.test("encode zeros 4 bytes", () => {
  assertEquals(encode(new Uint8Array([0, 0, 0, 0])), "00000");
});

Deno.test("decode zeros 4 bytes", () => {
  assertEquals(decode("00000"), new Uint8Array([0, 0, 0, 0]));
});

Deno.test("encode max 4 bytes", () => {
  assertEquals(encode(new Uint8Array([0xff, 0xff, 0xff, 0xff])), "%nSc0");
});

Deno.test("decode max 4 bytes", () => {
  assertEquals(decode("%nSc0"), new Uint8Array([0xff, 0xff, 0xff, 0xff]));
});

Deno.test("encode one byte zero", () => {
  assertEquals(encode(new Uint8Array([0x00])), "00");
});

Deno.test("decode one byte zero", () => {
  assertEquals(decode("00"), new Uint8Array([0x00]));
});

Deno.test("encode one byte max", () => {
  // 0xFF = 255 = 3*85 + 0, so encodes to "30"
  assertEquals(encode(new Uint8Array([0xff])), "30");
});

Deno.test("decode one byte max", () => {
  assertEquals(decode("30"), new Uint8Array([0xff]));
});

Deno.test("roundtrip various lengths", () => {
  const testCases: Uint8Array[] = [
    new Uint8Array([]),
    new Uint8Array([0]),
    new Uint8Array([0, 0]),
    new Uint8Array([0, 0, 0]),
    new Uint8Array([0, 0, 0, 0]),
    new Uint8Array([0, 0, 0, 0, 0]),
    new Uint8Array([0xff]),
    new Uint8Array([0xff, 0xff]),
    new Uint8Array([0xff, 0xff, 0xff]),
    new Uint8Array([0xff, 0xff, 0xff, 0xff]),
    new Uint8Array([1, 2, 3, 4]),
    new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]),
  ];

  for (const input of testCases) {
    const encoded = encode(input);
    const decoded = decode(encoded);
    assertEquals(decoded, input, `roundtrip failed for ${Array.from(input)}`);
  }
});

Deno.test("decode invalid character", () => {
  // Use valid length strings (5 chars) with invalid characters
  assertThrows(() => decode('hel"o'), Z85DecodeError);
  assertThrows(() => decode("hel o"), Z85DecodeError);
});

Deno.test("decode invalid length", () => {
  // Length 1 is invalid
  assertThrows(() => decode("0"), Z85DecodeError);
  // Length 6 is invalid (would be 1 mod 5)
  assertThrows(() => decode("000000"), Z85DecodeError);
});

Deno.test("decode overflow", () => {
  // "#####" = 84*85^4 + 84*85^3 + 84*85^2 + 84*85 + 84 = 4,437,053,124 > 0xFFFFFFFF
  assertThrows(() => decode("#####"), Z85DecodeError);
  // "##" for 1 byte: 84*85 + 84 = 7224 > 255
  assertThrows(() => decode("##"), Z85DecodeError);
});

// Cross-testing with Rust CLI
// These tests invoke the Rust implementation to verify both agree

async function runRustEncode(input: Uint8Array): Promise<string> {
  const command = new Deno.Command("cargo", {
    args: ["run", "--quiet", "--release", "--manifest-path", "/Users/jeb/cleanroom/Cargo.toml", "--", "encode"],
    stdin: "piped",
    stdout: "piped",
    stderr: "piped",
  });
  const process = command.spawn();

  const writer = process.stdin.getWriter();
  await writer.write(input);
  await writer.close();

  const output = await process.output();
  return new TextDecoder().decode(output.stdout);
}

async function runRustDecode(input: string): Promise<Uint8Array | null> {
  const command = new Deno.Command("cargo", {
    args: ["run", "--quiet", "--release", "--manifest-path", "/Users/jeb/cleanroom/Cargo.toml", "--", "decode"],
    stdin: "piped",
    stdout: "piped",
    stderr: "piped",
  });
  const process = command.spawn();

  const writer = process.stdin.getWriter();
  await writer.write(new TextEncoder().encode(input));
  await writer.close();

  const output = await process.output();
  if (!output.success) {
    return null; // Error case
  }
  return output.stdout;
}

Deno.test("cross-test encode with Rust", async () => {
  const testCases: Uint8Array[] = [
    new Uint8Array([]),
    new Uint8Array([0]),
    new Uint8Array([0, 0, 0, 0]),
    new Uint8Array([0xff, 0xff, 0xff, 0xff]),
    new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]),
  ];

  for (const input of testCases) {
    const tsResult = encode(input);
    const rustResult = await runRustEncode(input);
    assertEquals(tsResult, rustResult, `encode mismatch for ${Array.from(input)}`);
  }
});

Deno.test("cross-test decode with Rust", async () => {
  const testCases = ["", "00", "00000", "%nSc0", "0000000000"];

  for (const input of testCases) {
    const tsResult = decode(input);
    const rustResult = await runRustDecode(input);
    assertEquals(tsResult, rustResult, `decode mismatch for "${input}"`);
  }
});

// Test cases from test-cases/ directory
Deno.test("test cases from shared directory", async () => {
  const testCasesDir = "/Users/jeb/cleanroom/test-cases";

  for await (const entry of Deno.readDir(testCasesDir)) {
    if (!entry.name.endsWith(".input")) continue;

    const baseName = entry.name.replace(".input", "");
    const inputPath = `${testCasesDir}/${baseName}.input`;
    const encodedPath = `${testCasesDir}/${baseName}.encoded`;

    const inputBytes = await Deno.readFile(inputPath);
    const encodedStr = await Deno.readTextFile(encodedPath);

    // Check if this is an error test case
    const inputStr = new TextDecoder().decode(inputBytes);

    if (inputStr === "<error />") {
      // This is a decode error test: encoded should fail to decode
      assertThrows(
        () => decode(encodedStr),
        Z85DecodeError,
        undefined,
        `Expected decode error for ${baseName}`
      );
    } else {
      // Normal test: encode input should match encoded
      const actualEncoded = encode(inputBytes);
      assertEquals(actualEncoded, encodedStr, `encode mismatch for ${baseName}`);

      // And decode should roundtrip
      const decoded = decode(encodedStr);
      assertEquals(decoded, inputBytes, `decode mismatch for ${baseName}`);
    }
  }
});
