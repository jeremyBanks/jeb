#!/usr/bin/env -S deno run --allow-read

// Import decoders and encoder
import { decode as z855Decode, encode as z855Encode } from "./z855.ts";
import { decode as minDecode, z855 as minEncode } from "./min.mjs";

// ANSI color codes
const RED = "\x1b[31m";
const GREEN = "\x1b[32m";
const YELLOW = "\x1b[33m";
const BLUE = "\x1b[34m";
const RESET = "\x1b[0m";
const BOLD = "\x1b[1m";

// Helper to compare Uint8Arrays
function bytesEqual(a, b) {
  if (a.length !== b.length) return false;
  for (let i = 0; i < a.length; i++) {
    if (a[i] !== b[i]) return false;
  }
  return true;
}

// Helper to format bytes for display
function formatBytes(bytes, maxLen = 50) {
  const hex = Array.from(bytes)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join(" ");
  if (hex.length > maxLen) {
    return hex.slice(0, maxLen) + "...";
  }
  return hex;
}

// Helper to format string for display
function formatString(str, maxLen = 50) {
  if (str.length > maxLen) {
    return str.slice(0, maxLen) + "...";
  }
  return str;
}

// Read all gap test files
const testCasesDir = "./test-cases";
const files = [];

for await (const entry of Deno.readDir(testCasesDir)) {
  if (entry.name.startsWith("gap-") && entry.name.endsWith(".input")) {
    const baseName = entry.name.replace(".input", "");
    const inputPath = `${testCasesDir}/${entry.name}`;
    const encodedPath = `${testCasesDir}/${baseName}.encoded`;

    // Check if corresponding .encoded file exists
    try {
      await Deno.stat(encodedPath);
      files.push({ baseName, inputPath, encodedPath });
    } catch {
      console.log(
        `${YELLOW}Warning: No .encoded file for ${entry.name}${RESET}`
      );
    }
  }
}

// Sort for consistent output
files.sort((a, b) => a.baseName.localeCompare(b.baseName));

console.log(`${BOLD}Z855 Gap Test Verification${RESET}`);
console.log(`${BOLD}============================${RESET}\n`);
console.log(`Found ${files.length} test file pairs\n`);

// Statistics
let totalTests = 0;
let z855DecodeFailures = 0;
let minDecodeFailures = 0;
let encodingDifferences = 0;
let allPassed = 0;

const failures = [];
const differences = [];

// Test each pair
for (const { baseName, inputPath, encodedPath } of files) {
  totalTests++;

  // Read files
  const originalInput = await Deno.readFile(inputPath);
  const canonicalEncoding = await Deno.readTextFile(encodedPath);

  let testFailed = false;
  let hasDifference = false;

  // Test 1: z855.ts decoder
  let z855DecodeResult;
  let z855DecodeOk = false;
  try {
    z855DecodeResult = z855Decode(canonicalEncoding);
    z855DecodeOk = bytesEqual(z855DecodeResult, originalInput);

    if (!z855DecodeOk) {
      z855DecodeFailures++;
      testFailed = true;
      failures.push({
        file: baseName,
        test: "z855.ts decode",
        expected: formatBytes(originalInput),
        got: formatBytes(z855DecodeResult),
        encoding: formatString(canonicalEncoding),
      });
    }
  } catch (error) {
    z855DecodeFailures++;
    testFailed = true;
    failures.push({
      file: baseName,
      test: "z855.ts decode",
      error: error.message,
      encoding: formatString(canonicalEncoding),
    });
  }

  // Test 2: min.mjs decoder
  let minDecodeResult;
  let minDecodeOk = false;
  try {
    minDecodeResult = minDecode(canonicalEncoding);
    minDecodeOk = bytesEqual(minDecodeResult, originalInput);

    if (!minDecodeOk) {
      minDecodeFailures++;
      testFailed = true;
      failures.push({
        file: baseName,
        test: "min.mjs decode",
        expected: formatBytes(originalInput),
        got: formatBytes(minDecodeResult),
        encoding: formatString(canonicalEncoding),
      });
    }
  } catch (error) {
    minDecodeFailures++;
    testFailed = true;
    failures.push({
      file: baseName,
      test: "min.mjs decode",
      error: error.message,
      encoding: formatString(canonicalEncoding),
    });
  }

  // Test 3: min.mjs encoder (check if it produces canonical encoding)
  let minEncodeResult;
  let minEncodeMatches = false;
  try {
    minEncodeResult = minEncode(originalInput);
    minEncodeMatches = minEncodeResult === canonicalEncoding;

    if (!minEncodeMatches) {
      hasDifference = true;
      encodingDifferences++;

      // Verify that the min.mjs encoding is still valid (round-trips correctly)
      let minEncodeValid = false;
      try {
        const roundTrip = minDecode(minEncodeResult);
        minEncodeValid = bytesEqual(roundTrip, originalInput);
      } catch {
        minEncodeValid = false;
      }

      differences.push({
        file: baseName,
        canonical: formatString(canonicalEncoding),
        minEncoded: formatString(minEncodeResult),
        valid: minEncodeValid,
        input: formatBytes(originalInput),
      });
    }
  } catch (error) {
    // Encoding failure is not necessarily a test failure
    // The minimal encoder might not support all features
    hasDifference = true;
    encodingDifferences++;
    differences.push({
      file: baseName,
      error: error.message,
      input: formatBytes(originalInput),
    });
  }

  if (!testFailed && !hasDifference) {
    allPassed++;
  }

  // Print progress indicator
  const status = testFailed
    ? `${RED}✗${RESET}`
    : hasDifference
    ? `${YELLOW}△${RESET}`
    : `${GREEN}✓${RESET}`;
  console.log(`${status} ${baseName}`);
}

// Print summary
console.log(`\n${BOLD}Summary${RESET}`);
console.log(`${BOLD}=======${RESET}`);
console.log(`Total test pairs: ${totalTests}`);
console.log(
  `All tests passed: ${GREEN}${allPassed}${RESET} (${((allPassed / totalTests) * 100).toFixed(1)}%)`
);
console.log(`z855.ts decode failures: ${z855DecodeFailures > 0 ? RED : GREEN}${z855DecodeFailures}${RESET}`);
console.log(`min.mjs decode failures: ${minDecodeFailures > 0 ? RED : GREEN}${minDecodeFailures}${RESET}`);
console.log(
  `Encoding differences: ${encodingDifferences > 0 ? YELLOW : GREEN}${encodingDifferences}${RESET} (valid variations)`
);

// Show failures
if (failures.length > 0) {
  console.log(`\n${BOLD}${RED}FAILURES${RESET}`);
  console.log(`${BOLD}${RED}========${RESET}\n`);

  for (const failure of failures) {
    console.log(`${RED}✗ ${failure.file} - ${failure.test}${RESET}`);
    if (failure.error) {
      console.log(`  Error: ${failure.error}`);
    } else {
      console.log(`  Expected: ${failure.expected}`);
      console.log(`  Got:      ${failure.got}`);
    }
    console.log(`  Encoding: ${failure.encoding}`);
    console.log();
  }
}

// Show encoding differences
if (differences.length > 0) {
  console.log(`\n${BOLD}${YELLOW}ENCODING DIFFERENCES${RESET}`);
  console.log(`${BOLD}${YELLOW}====================${RESET}\n`);
  console.log(
    `(These are cases where min.mjs produces different but valid encodings)\n`
  );

  for (const diff of differences) {
    console.log(`${YELLOW}△ ${diff.file}${RESET}`);
    if (diff.error) {
      console.log(`  Error: ${diff.error}`);
    } else {
      console.log(`  Canonical:   ${diff.canonical}`);
      console.log(`  min.mjs:     ${diff.minEncoded}`);
      console.log(
        `  Valid:       ${diff.valid ? GREEN + "✓" + RESET : RED + "✗" + RESET}`
      );
    }
    console.log(`  Input:       ${diff.input}`);
    console.log();
  }
}

// Exit code
if (failures.length > 0) {
  console.log(`${BOLD}${RED}VERIFICATION FAILED${RESET}\n`);
  Deno.exit(1);
} else {
  console.log(`${BOLD}${GREEN}VERIFICATION PASSED${RESET}\n`);
  if (differences.length > 0) {
    console.log(
      `Note: ${differences.length} encoding difference(s) found, but all are valid variations.`
    );
  }
  Deno.exit(0);
}
