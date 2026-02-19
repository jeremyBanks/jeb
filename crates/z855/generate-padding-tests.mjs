#!/usr/bin/env -S deno run --allow-read --allow-write

/**
 * Generate 112 randomized padding tests to verify decoder ignores padding content.
 *
 * CRITICAL BUG: Current decoder checks padding content (looks for `.` and `|`).
 * These tests expose this bug by using different padding patterns.
 *
 * Test matrix:
 * - Raw lengths: 8, 16, 24, 32 bytes
 * - Offsets: 0, 1, 2, 4
 * - Padding patterns: dots, pipes, zeros, safe, unsafe, mixed, escapes
 * - Total: 4 × 4 × 7 = 112 test cases
 *
 * Each test:
 * 1. Generates valid raw bytes (random safe characters)
 * 2. Calculates padding positions based on offset and length
 * 3. Creates long escape format: <length>|<padding_before><raw_bytes><padding_after>
 * 4. Replaces padding bytes with the specified pattern
 * 5. Writes .input (raw bytes) and .encoded (Z855 with custom padding)
 *
 * Expected: Tests with non-dot padding SHOULD FAIL with current decoder.
 */

const RAW_LENGTHS = [8, 16, 24, 32];
const OFFSETS = [0, 1, 2, 4];
const PATTERNS = ["dots", "pipes", "zeros", "safe", "unsafe", "mixed", "escapes"];
const OUTPUT_DIR = "test-cases";

// Z855 safe character set (from z855.ts)
const SAFE_CHARS = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#,;|~_";
const SAFE_BYTES = new Uint8Array(SAFE_CHARS.split('').map(c => c.charCodeAt(0)));

// Escape characters
const ESCAPE_CHARS = ",;|~_";
const ESCAPE_BYTES = new Uint8Array(ESCAPE_CHARS.split('').map(c => c.charCodeAt(0)));

// Unsafe bytes (not in safe set)
const UNSAFE_RANGES = [
  { start: 0x00, end: 0x1F }, // Control characters
  { start: 0x20, end: 0x20 }, // Space
  { start: 0x22, end: 0x22 }, // "
  { start: 0x27, end: 0x27 }, // '
  { start: 0x5C, end: 0x5C }, // \
  { start: 0x60, end: 0x60 }, // `
  { start: 0x7F, end: 0x7F }, // Delete
  { start: 0x80, end: 0xFF }, // High bytes
];

/**
 * Generate base-42 self-terminating prefix for a number.
 * Used for length/offset encoding in long escapes.
 */
function generateLongEscapePrefix(n) {
  if (n < 42) {
    // Single digit: terminal value (0-41)
    return SAFE_CHARS[n];
  }

  // Multi-digit: convert to base-42, reverse, add continuation bit (+42) to non-terminal
  const digits = [];
  while (n > 0) {
    digits.push(n % 42);
    n = Math.floor(n / 42);
  }
  digits.reverse();

  // First digit is terminal, rest are continuations (+42)
  let result = "";
  for (let i = 0; i < digits.length; i++) {
    const offset = i > 0 ? 42 : 0;
    result += SAFE_CHARS[digits[i] + offset];
  }

  return result;
}

/**
 * Calculate padding needed for a given raw length.
 * Formula from z855.ts: ceil(rawLen * 5/4) - ceil((rawLen - 8) * 5/4)
 */
function calculatePaddingNeeded(rawLen) {
  if (rawLen < 8) return 0;
  const ceilFrac = (n, d) => Math.floor((n + d - 1) / d);
  return ceilFrac(rawLen * 5, 4) - ceilFrac((rawLen - 8) * 5, 4);
}

/**
 * Generate random safe bytes.
 */
function generateSafeBytes(count) {
  const result = new Uint8Array(count);
  for (let i = 0; i < count; i++) {
    result[i] = SAFE_BYTES[Math.floor(Math.random() * SAFE_BYTES.length)];
  }
  return result;
}

/**
 * Generate random unsafe bytes.
 */
function generateUnsafeBytes(count) {
  const result = new Uint8Array(count);
  for (let i = 0; i < count; i++) {
    const range = UNSAFE_RANGES[Math.floor(Math.random() * UNSAFE_RANGES.length)];
    result[i] = range.start + Math.floor(Math.random() * (range.end - range.start + 1));
  }
  return result;
}

/**
 * Generate random mixed bytes (any byte value).
 */
function generateMixedBytes(count) {
  const result = new Uint8Array(count);
  for (let i = 0; i < count; i++) {
    result[i] = Math.floor(Math.random() * 256);
  }
  return result;
}

/**
 * Generate padding bytes based on pattern.
 */
function generatePaddingBytes(pattern, count) {
  switch (pattern) {
    case "dots":
      // All dots (0x2E) - canonical padding
      return new Uint8Array(count).fill(0x2E);

    case "pipes":
      // All pipes (0x7C) - will expose the bug!
      return new Uint8Array(count).fill(0x7C);

    case "zeros":
      // All null bytes (0x00)
      return new Uint8Array(count).fill(0x00);

    case "safe":
      // Random safe characters
      return generateSafeBytes(count);

    case "unsafe":
      // Random unsafe characters
      return generateUnsafeBytes(count);

    case "mixed":
      // Random bytes (any value)
      return generateMixedBytes(count);

    case "escapes":
      // Random escape characters: ,;|~_
      const result = new Uint8Array(count);
      for (let i = 0; i < count; i++) {
        result[i] = ESCAPE_BYTES[Math.floor(Math.random() * ESCAPE_BYTES.length)];
      }
      return result;

    default:
      throw new Error(`Unknown pattern: ${pattern}`);
  }
}

/**
 * Create a long escape encoding with custom padding.
 */
function createLongEscapeWithPadding(rawBytes, offset, pattern) {
  const rawLen = rawBytes.length;

  // Calculate padding needed
  const paddingNeeded = calculatePaddingNeeded(rawLen);

  // Generate length prefix
  const lengthPrefix = generateLongEscapePrefix(rawLen);

  // Generate offset prefix (only if offset > 0)
  const offsetPrefix = offset > 0 ? generateLongEscapePrefix(offset) : "";

  // Calculate padding distribution
  const paddingBefore = offset;
  const paddingAfter = paddingNeeded - offsetPrefix.length - offset;

  if (paddingAfter < 0) {
    throw new Error(`Invalid offset ${offset}: not enough padding space`);
  }

  // Build the encoded string
  let encoded = "";

  // Add offset prefix (if any)
  encoded += offsetPrefix;

  // Add length prefix
  encoded += lengthPrefix;

  // Add pipe separator
  encoded += "|";

  // Add padding before (custom pattern)
  const paddingBeforeBytes = generatePaddingBytes(pattern, paddingBefore);
  for (const byte of paddingBeforeBytes) {
    encoded += String.fromCharCode(byte);
  }

  // Add raw bytes
  for (const byte of rawBytes) {
    encoded += String.fromCharCode(byte);
  }

  // Add padding after (custom pattern)
  const paddingAfterBytes = generatePaddingBytes(pattern, paddingAfter);
  for (const byte of paddingAfterBytes) {
    encoded += String.fromCharCode(byte);
  }

  return encoded;
}

/**
 * Generate a single test case.
 */
function generateTestCase(rawLen, offset, pattern) {
  // Generate random safe raw bytes
  const rawBytes = generateSafeBytes(rawLen);

  // Create encoded string with custom padding
  const encoded = createLongEscapeWithPadding(rawBytes, offset, pattern);

  // Generate filenames
  const baseName = `padding-${rawLen}-${offset}-${pattern}`;
  const inputPath = `${OUTPUT_DIR}/${baseName}.input`;
  const encodedPath = `${OUTPUT_DIR}/${baseName}.encoded`;

  // Write files
  Deno.writeFileSync(inputPath, rawBytes);
  Deno.writeTextFileSync(encodedPath, encoded);

  return {
    baseName,
    rawLen,
    offset,
    pattern,
    paddingNeeded: calculatePaddingNeeded(rawLen),
    inputPath,
    encodedPath,
    encodedLength: encoded.length,
  };
}

/**
 * Main function.
 */
function main() {
  console.log("Generating 112 randomized padding tests...\n");
  console.log("Test matrix:");
  console.log(`  Raw lengths: ${RAW_LENGTHS.join(", ")} bytes`);
  console.log(`  Offsets: ${OFFSETS.join(", ")}`);
  console.log(`  Patterns: ${PATTERNS.join(", ")}`);
  console.log(`  Total: ${RAW_LENGTHS.length} × ${OFFSETS.length} × ${PATTERNS.length} = ${RAW_LENGTHS.length * OFFSETS.length * PATTERNS.length} tests\n`);

  const results = [];
  let filesCreated = 0;

  // Generate all combinations
  for (const rawLen of RAW_LENGTHS) {
    for (const offset of OFFSETS) {
      for (const pattern of PATTERNS) {
        const result = generateTestCase(rawLen, offset, pattern);
        results.push(result);
        filesCreated += 2; // .input and .encoded
      }
    }
  }

  console.log("=".repeat(80));
  console.log("GENERATION COMPLETE");
  console.log("=".repeat(80));
  console.log(`Total files created: ${filesCreated} (${results.length} test cases × 2 files)`);
  console.log(`Files location: ${OUTPUT_DIR}/padding-*.{input,encoded}\n`);

  // Show breakdown by pattern
  console.log("BREAKDOWN BY PATTERN:");
  console.log("-".repeat(80));
  for (const pattern of PATTERNS) {
    const tests = results.filter(r => r.pattern === pattern);
    console.log(`  ${pattern.padEnd(10)}: ${tests.length} tests`);
  }
  console.log("");

  // Show some sample test cases
  console.log("SAMPLE TEST CASES:");
  console.log("-".repeat(80));

  const samples = [
    results.find(r => r.pattern === "dots" && r.rawLen === 8 && r.offset === 0),
    results.find(r => r.pattern === "pipes" && r.rawLen === 8 && r.offset === 0),
    results.find(r => r.pattern === "safe" && r.rawLen === 16 && r.offset === 2),
    results.find(r => r.pattern === "escapes" && r.rawLen === 32 && r.offset === 4),
  ];

  for (const sample of samples) {
    if (!sample) continue;

    const rawBytes = Deno.readFileSync(sample.inputPath);
    const encoded = Deno.readTextFileSync(sample.encodedPath);

    console.log(`\n${sample.baseName}`);
    console.log(`  Raw length: ${sample.rawLen} bytes`);
    console.log(`  Offset: ${sample.offset}`);
    console.log(`  Pattern: ${sample.pattern}`);
    console.log(`  Padding needed: ${sample.paddingNeeded} bytes`);
    console.log(`  Encoded length: ${sample.encodedLength} chars`);
    console.log(`  Raw bytes (hex): ${Array.from(rawBytes.slice(0, 16)).map(b => b.toString(16).padStart(2, '0')).join(' ')}${rawBytes.length > 16 ? '...' : ''}`);
    console.log(`  Encoded (first 40 chars): ${encoded.slice(0, 40)}${encoded.length > 40 ? '...' : ''}`);
  }

  console.log("\n" + "=".repeat(80));
  console.log("EXPECTED BEHAVIOR:");
  console.log("-".repeat(80));
  console.log("✅ Tests with pattern 'dots' SHOULD PASS (canonical padding)");
  console.log("❌ Tests with other patterns SHOULD FAIL (exposes decoder bug)");
  console.log("");
  console.log("The decoder currently checks padding content (looks for '.' and '|').");
  console.log("It SHOULD skip padding based on POSITION only, not content.");
  console.log("");
  console.log("NEXT STEPS:");
  console.log("-".repeat(80));
  console.log("1. Run verification to confirm tests fail:");
  console.log("   deno run --allow-read verify-roundtrip.mjs test-cases/padding-*.input");
  console.log("");
  console.log("2. Commit failing tests (document the bug)");
  console.log("");
  console.log("3. Fix decoder (position-based padding skip)");
  console.log("");
  console.log("4. Verify all tests pass");
  console.log("");
}

main();
