#!/usr/bin/env -S deno run --allow-read
import { decode } from "./z855.ts";
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

const TEST_CASES_DIR = "./test-cases";
const allFiles = readdirSync(TEST_CASES_DIR);
const paddingInputFiles = allFiles
  .filter((f) => f.startsWith("padding-") && f.endsWith(".input"))
  .sort();

console.log("VERIFYING PADDING TESTS");
console.log("=".repeat(80));
console.log(`Testing ${paddingInputFiles.length} padding test cases...`);
console.log();

let successCount = 0;
let failureCount = 0;
const failures = [];
const successes = [];

// Group results by pattern
const resultsByPattern = {};

for (const filename of paddingInputFiles) {
  const inputPath = join(TEST_CASES_DIR, filename);
  const encodedPath = inputPath.replace(/\.input$/, ".encoded");

  const inputBytes = new Uint8Array(readFileSync(inputPath));
  const encodedText = readFileSync(encodedPath, "utf8");

  // Extract pattern from filename: padding-<rawlen>-<offset>-<pattern>.input
  const match = filename.match(/padding-\d+-\d+-([^.]+)\.input/);
  const pattern = match ? match[1] : "unknown";

  if (!resultsByPattern[pattern]) {
    resultsByPattern[pattern] = { success: 0, failures: [] };
  }

  // Decode the encoded text
  let decodedBytes;
  let error = null;
  try {
    decodedBytes = decode(encodedText);
  } catch (e) {
    error = `Decode failed: ${e.message}`;
    failures.push({ filename, pattern, error });
    resultsByPattern[pattern].failures.push({ filename, error });
    failureCount++;
    continue;
  }

  // Compare decoded bytes with original input
  if (decodedBytes.length !== inputBytes.length) {
    error = `Length mismatch: expected ${inputBytes.length}, got ${decodedBytes.length}`;
    failures.push({ filename, pattern, error });
    resultsByPattern[pattern].failures.push({ filename, error });
    failureCount++;
    continue;
  }

  let mismatch = false;
  for (let i = 0; i < inputBytes.length; i++) {
    if (decodedBytes[i] !== inputBytes[i]) {
      error = `Byte mismatch at position ${i}: expected ${inputBytes[i]}, got ${decodedBytes[i]}`;
      failures.push({ filename, pattern, error });
      resultsByPattern[pattern].failures.push({ filename, error });
      failureCount++;
      mismatch = true;
      break;
    }
  }

  if (!mismatch) {
    successCount++;
    successes.push({ filename, pattern });
    resultsByPattern[pattern].success++;
  }
}

console.log("RESULTS BY PATTERN:");
console.log("-".repeat(80));

const patterns = Object.keys(resultsByPattern).sort();
for (const pattern of patterns) {
  const { success, failures: patternFailures } = resultsByPattern[pattern];
  const total = success + patternFailures.length;
  const status = success === total ? "✅ ALL PASS" : patternFailures.length === total ? "❌ ALL FAIL" : "⚠️  MIXED";
  console.log(`  ${pattern.padEnd(10)}: ${status.padEnd(12)} (${success}/${total} passed)`);
}

console.log();
console.log("SUMMARY:");
console.log("-".repeat(80));
console.log(`Total tests: ${paddingInputFiles.length}`);
console.log(`Passed: ${successCount}`);
console.log(`Failed: ${failureCount}`);
console.log();

if (failures.length > 0) {
  console.log("FAILURES (first 20):");
  console.log("-".repeat(80));
  for (const { filename, pattern, error } of failures.slice(0, 20)) {
    console.log(`  [${pattern.padEnd(8)}] ${filename}`);
    console.log(`    ${error}`);
  }
  if (failures.length > 20) {
    console.log(`  ... and ${failures.length - 20} more failures`);
  }
  console.log();
}

console.log("EXPECTED BEHAVIOR:");
console.log("-".repeat(80));
console.log("✅ Pattern 'dots' should PASS (canonical padding with all dots)");
console.log("❌ Other patterns should FAIL (exposes decoder bug)");
console.log();
console.log("The decoder bug: It checks padding content (looks for '.' and '|')");
console.log("instead of skipping padding based on POSITION calculation only.");
console.log();

console.log("=".repeat(80));

// Exit with error code if there are unexpected results
// We EXPECT non-dots patterns to fail (that's the bug we're testing)
// But dots pattern should pass
const dotsResults = resultsByPattern["dots"];
if (dotsResults && dotsResults.failures.length > 0) {
  console.log("⚠️  WARNING: 'dots' pattern has failures (unexpected!)");
  Deno.exit(1);
}

console.log("✅ Results match expectations (dots pass, others fail due to known bug)");
