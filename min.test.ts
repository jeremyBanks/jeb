import { assertEquals, assertThrows } from "jsr:@std/assert";
import * as z855 from "./z855.ts";
import * as min from "./min.mjs";
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

const TEST_CASES_DIR = "./test-cases";

// Helper to compare Uint8Arrays
function arraysEqual(a: Uint8Array, b: Uint8Array): boolean {
  if (a.length !== b.length) return false;
  for (let i = 0; i < a.length; i++) {
    if (a[i] !== b[i]) return false;
  }
  return true;
}

// Get all test case files
function getTestCases(): { name: string; input: Uint8Array; encoded?: string; isError?: boolean }[] {
  const files = readdirSync(TEST_CASES_DIR);
  const inputFiles = files.filter(f => f.endsWith(".input"));

  return inputFiles.map(inputFile => {
    const name = inputFile.replace(".input", "");
    const inputPath = join(TEST_CASES_DIR, inputFile);
    const encodedPath = join(TEST_CASES_DIR, name + ".encoded");

    const input = readFileSync(inputPath);
    let encoded: string | undefined;
    // Check if it's an error case (filename suggests error)
    const isError = name.includes("invalid") || name.includes("overflow") || name.includes("incomplete") || name.includes("error");

    try {
      const encodedData = readFileSync(encodedPath);
      encoded = new TextDecoder().decode(encodedData);
    } catch {
      // No encoded file
    }

    return { name, input: new Uint8Array(input), encoded, isError };
  });
}

// Test: min.decode matches z855.decode for all encoded files
Deno.test("min.decode matches z855.decode for valid encoded inputs", async (t) => {
  const testCases = getTestCases();

  for (const tc of testCases) {
    if (!tc.encoded || tc.isError) continue;

    await t.step(tc.name, () => {
      const z855Result = z855.decode(tc.encoded!);
      const minResult = min.decode(tc.encoded!);

      assertEquals(
        Array.from(minResult),
        Array.from(z855Result),
        `Decode mismatch for ${tc.name}`
      );
    });
  }
});

// Test: min.decode(z855.encode(input)) === input (round-trip deno->min)
Deno.test("Round-trip: z855.encode -> min.decode", async (t) => {
  const testCases = getTestCases();

  for (const tc of testCases) {
    if (tc.isError) continue;

    await t.step(tc.name, () => {
      const encoded = z855.encode(tc.input);
      const decoded = min.decode(encoded);

      assertEquals(
        Array.from(decoded),
        Array.from(tc.input),
        `Round-trip failed for ${tc.name}: encoded=${encoded}`
      );
    });
  }
});

// Test: z855.decode(min.encode(input)) === input (round-trip min->deno)
Deno.test("Round-trip: min.encode -> z855.decode", async (t) => {
  const testCases = getTestCases();

  for (const tc of testCases) {
    if (tc.isError) continue;

    await t.step(tc.name, () => {
      const encoded = min.encode(tc.input);
      const decoded = z855.decode(encoded);

      assertEquals(
        Array.from(decoded),
        Array.from(tc.input),
        `Round-trip failed for ${tc.name}: encoded=${encoded}`
      );
    });
  }
});

// Test: min.decode(min.encode(input)) === input (round-trip min->min)
Deno.test("Round-trip: min.encode -> min.decode", async (t) => {
  const testCases = getTestCases();

  for (const tc of testCases) {
    if (tc.isError) continue;

    await t.step(tc.name, () => {
      const encoded = min.encode(tc.input);
      const decoded = min.decode(encoded);

      assertEquals(
        Array.from(decoded),
        Array.from(tc.input),
        `Round-trip failed for ${tc.name}: encoded=${encoded}`
      );
    });
  }
});

// Test: Error cases throw
Deno.test("Error cases throw correctly", async (t) => {
  const errorInputs = [
    { name: "invalid-char", encoded: "hello world" }, // space is invalid
    { name: "single-char", encoded: "0" }, // 1 char is invalid
    { name: "overflow-5char", encoded: "#####" }, // max Z85 value exceeds u32
    { name: "overflow-2char", encoded: "31" }, // 2 chars: 3*85+1=256 > 255
    { name: "incomplete-comma", encoded: ",abc" }, // comma needs 4 bytes after
  ];

  for (const tc of errorInputs) {
    await t.step(tc.name, () => {
      assertThrows(
        () => min.decode(tc.encoded),
        Error,
        undefined,
        `Expected ${tc.name} to throw`
      );
    });
  }
});

// Test: Empty input
Deno.test("Empty input", () => {
  const emptyArr = new Uint8Array(0);
  assertEquals(min.encode(emptyArr), "");
  assertEquals(Array.from(min.decode("")), []);
});

// Test: Specific known values
Deno.test("Known values", async (t) => {
  await t.step("4 zero bytes -> 00000", () => {
    const input = new Uint8Array([0, 0, 0, 0]);
    const encoded = "00000";
    assertEquals(Array.from(min.decode(encoded)), Array.from(input));
  });

  await t.step("1 zero byte -> 00", () => {
    const input = new Uint8Array([0]);
    const encoded = "00";
    assertEquals(Array.from(min.decode(encoded)), Array.from(input));
  });

  await t.step("HelloWorld", () => {
    // "HelloWorld" in bytes
    const input = new Uint8Array([0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x57, 0x6f, 0x72, 0x6c, 0x64]);
    const encoded = z855.encode(input);
    assertEquals(Array.from(min.decode(encoded)), Array.from(input));
  });

  await t.step("4 commas passthrough", () => {
    const input = new Uint8Array([44, 44, 44, 44]); // ,,,,
    const encoded = ",,,,,"; // comma + 4 commas
    assertEquals(Array.from(min.decode(encoded)), Array.from(input));
  });
});

// Test: min.encode uses passthrough for safe bytes
Deno.test("Encoder uses passthrough for safe bytes", async (t) => {
  await t.step("4 safe bytes use comma passthrough", () => {
    const input = new Uint8Array([65, 66, 67, 68]); // ABCD
    const encoded = min.encode(input);
    // Should use comma passthrough: ,ABCD
    assertEquals(encoded, ",ABCD");
  });

  await t.step("8 safe bytes at end use 0| passthrough", () => {
    const input = new Uint8Array([97, 98, 99, 100, 101, 102, 103, 104]); // abcdefgh
    const encoded = min.encode(input);
    // Should use 0| passthrough
    assertEquals(encoded, "0|abcdefgh");
  });

  await t.step("8 safe bytes not at end use 8| passthrough", () => {
    const input = new Uint8Array([97, 98, 99, 100, 101, 102, 103, 104, 0]); // abcdefgh + null
    const encoded = min.encode(input);
    // Should use 8| passthrough for first 8, then standard Z85 for trailing byte
    assertEquals(encoded.startsWith("8|abcdefgh|"), true);
  });
});

// Test: Large random inputs round-trip
Deno.test("Large random inputs round-trip", () => {
  const sizes = [100, 1000, 10000];

  for (const size of sizes) {
    const input = new Uint8Array(size);
    for (let i = 0; i < size; i++) {
      input[i] = Math.floor(Math.random() * 256);
    }

    // min.encode -> min.decode
    const encoded = min.encode(input);
    const decoded = min.decode(encoded);
    assertEquals(
      Array.from(decoded),
      Array.from(input),
      `Round-trip failed for size ${size}`
    );

    // min.encode -> z855.decode
    const decoded2 = z855.decode(encoded);
    assertEquals(
      Array.from(decoded2),
      Array.from(input),
      `Cross-decode failed for size ${size}`
    );
  }
});
