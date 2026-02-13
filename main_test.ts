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

// =========================================================================
// Tests for raw passthrough extension (`,` escape)
// =========================================================================

Deno.test("raw passthrough encode", () => {
  // "test" (4 ASCII chars) should be encoded with passthrough
  const input = new TextEncoder().encode("test");
  const encoded = encode(input);
  assertEquals(encoded, ",test", "4 safe chars should use passthrough");
});

Deno.test("raw passthrough decode", () => {
  // ",test" should decode to "test"
  const decoded = decode(",test");
  assertEquals(decoded, new TextEncoder().encode("test"));
});

Deno.test("raw passthrough roundtrip", () => {
  // Various safe character combinations
  const testCases = ["test", "abcd", "ABCD", "1234", ".-:+", ",;|~"];

  for (const str of testCases) {
    const input = new TextEncoder().encode(str);
    const encoded = encode(input);
    // Should use passthrough (starts with ,)
    assertEquals(encoded.startsWith(","), true, `Expected passthrough for ${str}`);
    const decoded = decode(encoded);
    assertEquals(decoded, input, `roundtrip failed for ${str}`);
  }
});

Deno.test("mixed passthrough and z85", () => {
  // Mix of safe and non-safe blocks
  // First 4 bytes: 0x00 0x00 0x00 0x00 (not safe - contains null bytes)
  // Next 4 bytes: "test" (safe)
  const input = new Uint8Array([0x00, 0x00, 0x00, 0x00, 0x74, 0x65, 0x73, 0x74]);
  const encoded = encode(input);
  // Should be "00000,test" - first block Z85, second passthrough
  assertEquals(encoded, "00000,test");

  const decoded = decode(encoded);
  assertEquals(decoded, input);
});

Deno.test("no passthrough for unsafe bytes", () => {
  // Bytes that are not in safe chars set
  const input = new Uint8Array([0x00, 0x01, 0x02, 0x03]);
  const encoded = encode(input);
  // Should NOT start with `,` - not safe for passthrough
  assertEquals(encoded.startsWith(","), false, "Non-safe bytes should use Z85");

  const decoded = decode(encoded);
  assertEquals(decoded, input);
});

Deno.test("passthrough decode does not validate", () => {
  // Decoder should accept ANY bytes after `,`, not just safe ones
  // This is important: decoder trusts the input
  const encoded = ",\x00\x01\x02\x03"; // Not actually safe chars
  const decoded = decode(encoded);
  assertEquals(decoded, new Uint8Array([0x00, 0x01, 0x02, 0x03]));
});

Deno.test("passthrough with trailing bytes", () => {
  // "test" + null byte (5 bytes)
  // First 4 bytes: "test" (safe, passthrough)
  // Trailing 1 byte: 0x00 (Z85 encoded)
  const input = new Uint8Array([0x74, 0x65, 0x73, 0x74, 0x00]);
  const encoded = encode(input);
  assertEquals(encoded, ",test00"); // passthrough + 1-byte Z85

  const decoded = decode(encoded);
  assertEquals(decoded, input);
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
//
// Test case structure:
// - X.input: raw bytes to encode/decode
// - X.encoded: standard Z85 encoding (MUST always be present)
// - X.encoded-Y: alternative valid encodings (optional, any number)
// - X.encoded-expected: if present, encoder MUST produce exactly this
//
// Test behavior:
// - Decode tests: ALL .encoded* files must decode to same .input
// - Encode tests: result must match .encoded-expected if present, otherwise any .encoded* file

/**
 * Find all encoded files for a given test case base name.
 * Returns { standard: string, alternatives: string[], expected?: string }
 */
async function findEncodedFiles(
  testCasesDir: string,
  baseName: string
): Promise<{ standard: string; alternatives: string[]; expected?: string }> {
  const result: { standard: string; alternatives: string[]; expected?: string } = {
    standard: "",
    alternatives: [],
  };

  // Read the standard .encoded file
  const encodedPath = `${testCasesDir}/${baseName}.encoded`;
  result.standard = await Deno.readTextFile(encodedPath);

  // Scan for alternative encoded files
  for await (const entry of Deno.readDir(testCasesDir)) {
    const name = entry.name;

    // Check for .encoded-expected
    if (name === `${baseName}.encoded-expected`) {
      result.expected = await Deno.readTextFile(`${testCasesDir}/${name}`);
    }
    // Check for .encoded-Y pattern (but not .encoded-expected)
    else if (name.startsWith(`${baseName}.encoded-`) && name !== `${baseName}.encoded-expected`) {
      const altEncoded = await Deno.readTextFile(`${testCasesDir}/${name}`);
      result.alternatives.push(altEncoded);
    }
  }

  return result;
}

Deno.test("test cases from shared directory", async () => {
  const testCasesDir = "/Users/jeb/cleanroom/test-cases";

  for await (const entry of Deno.readDir(testCasesDir)) {
    if (!entry.name.endsWith(".input")) continue;

    const baseName = entry.name.replace(".input", "");
    const inputPath = `${testCasesDir}/${baseName}.input`;

    const inputBytes = await Deno.readFile(inputPath);

    // Check if this is an error test case
    const inputStr = new TextDecoder().decode(inputBytes);

    if (inputStr === "<error />") {
      // This is a decode error test: all encoded files should fail to decode
      const encodedFiles = await findEncodedFiles(testCasesDir, baseName);

      // Test standard encoding fails
      assertThrows(
        () => decode(encodedFiles.standard),
        Z85DecodeError,
        undefined,
        `Expected decode error for ${baseName} (standard)`
      );

      // Test alternatives also fail
      for (const alt of encodedFiles.alternatives) {
        assertThrows(
          () => decode(alt),
          Z85DecodeError,
          undefined,
          `Expected decode error for ${baseName} (alternative)`
        );
      }

      // Test expected also fails if present
      if (encodedFiles.expected) {
        assertThrows(
          () => decode(encodedFiles.expected!),
          Z85DecodeError,
          undefined,
          `Expected decode error for ${baseName} (expected)`
        );
      }
    } else {
      // Normal test case
      const encodedFiles = await findEncodedFiles(testCasesDir, baseName);

      // DECODE TESTS: All encoded files must decode to same input
      const decodedStandard = decode(encodedFiles.standard);
      assertEquals(decodedStandard, inputBytes, `decode mismatch for ${baseName} (standard)`);

      for (let i = 0; i < encodedFiles.alternatives.length; i++) {
        const decodedAlt = decode(encodedFiles.alternatives[i]);
        assertEquals(decodedAlt, inputBytes, `decode mismatch for ${baseName} (alternative ${i})`);
      }

      if (encodedFiles.expected) {
        const decodedExpected = decode(encodedFiles.expected);
        assertEquals(decodedExpected, inputBytes, `decode mismatch for ${baseName} (expected)`);
      }

      // ENCODE TEST: result must match .encoded-expected if present, otherwise any .encoded* file
      const actualEncoded = encode(inputBytes);

      if (encodedFiles.expected) {
        // Must match expected exactly
        assertEquals(actualEncoded, encodedFiles.expected, `encode must match expected for ${baseName}`);
      } else {
        // Must match one of: standard, or any alternative
        const validEncodings = [encodedFiles.standard, ...encodedFiles.alternatives];
        const matches = validEncodings.some((valid) => actualEncoded === valid);
        assertEquals(
          matches,
          true,
          `encode mismatch for ${baseName}: got "${actualEncoded}", expected one of: ${validEncodings.map(s => `"${s}"`).join(", ")}`
        );
      }
    }
  }
});
