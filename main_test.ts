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
