#!/usr/bin/env -S deno run --allow-read
import { encode, decode } from "./z855.ts";
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

const TEST_CASES_DIR = "./test-cases";
const allFiles = readdirSync(TEST_CASES_DIR);
const gapInputFiles = allFiles
  .filter((f) => f.startsWith("gap-") && f.endsWith(".input"))
  .sort();

console.log("VERIFYING ROUND-TRIP CORRECTNESS");
console.log("=".repeat(80));
console.log();

let successCount = 0;
let failureCount = 0;
const failures = [];

for (const filename of gapInputFiles) {
  const inputPath = join(TEST_CASES_DIR, filename);
  const encodedPath = inputPath.replace(/\.input$/, ".encoded");
  
  const inputBytes = new Uint8Array(readFileSync(inputPath));
  const encodedText = readFileSync(encodedPath, "utf8");
  
  // Decode the encoded text
  let decodedBytes;
  try {
    decodedBytes = decode(encodedText);
  } catch (e) {
    failures.push({ filename, error: `Decode failed: ${e.message}` });
    failureCount++;
    continue;
  }
  
  // Compare decoded bytes with original input
  if (decodedBytes.length !== inputBytes.length) {
    failures.push({
      filename,
      error: `Length mismatch: expected ${inputBytes.length}, got ${decodedBytes.length}`
    });
    failureCount++;
    continue;
  }
  
  let mismatch = false;
  for (let i = 0; i < inputBytes.length; i++) {
    if (decodedBytes[i] !== inputBytes[i]) {
      failures.push({
        filename,
        error: `Byte mismatch at position ${i}: expected ${inputBytes[i]}, got ${decodedBytes[i]}`
      });
      failureCount++;
      mismatch = true;
      break;
    }
  }
  
  if (!mismatch) {
    successCount++;
  }
}

console.log(`Total files tested: ${gapInputFiles.length}`);
console.log(`Successful round-trips: ${successCount}`);
console.log(`Failed round-trips: ${failureCount}`);
console.log();

if (failures.length > 0) {
  console.log("FAILURES:");
  console.log("-".repeat(40));
  for (const { filename, error } of failures) {
    console.log(`  ${filename}: ${error}`);
  }
} else {
  console.log("SUCCESS: All gap test files round-trip correctly!");
}

console.log();
console.log("=".repeat(80));
