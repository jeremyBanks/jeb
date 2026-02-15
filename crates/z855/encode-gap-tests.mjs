#!/usr/bin/env -S deno run --allow-read --allow-write
// Generate canonical encodings for all gap test input files

import { encode } from "./z855.ts";
import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const TEST_CASES_DIR = "./test-cases";

// Find all gap-*.input files
const allFiles = readdirSync(TEST_CASES_DIR);
const gapInputFiles = allFiles
  .filter((f) => f.startsWith("gap-") && f.endsWith(".input"))
  .sort();

console.log(`Found ${gapInputFiles.length} gap test input files\n`);

// Track statistics
let totalEncoded = 0;
let rawPassthroughCount = 0;
let nonRawCount = 0;
const unexpectedPatterns = [];
const samplesByGapSize = new Map();

// Process each file
for (const filename of gapInputFiles) {
  const inputPath = join(TEST_CASES_DIR, filename);
  const outputPath = inputPath.replace(/\.input$/, ".encoded");

  // Read binary content
  const inputBytes = new Uint8Array(readFileSync(inputPath));

  // Encode using canonical encoder
  const encoded = encode(inputBytes);

  // Save to .encoded file
  writeFileSync(outputPath, encoded, "utf8");

  totalEncoded++;

  // Parse filename to extract gap size: gap-GAP-LEN1-LEN2.input
  const match = filename.match(/^gap-(\d+)-(\d+)-(\d+)\.input$/);
  if (match) {
    const [, gapSize, len1, len2] = match;
    const key = `gap-${gapSize}`;

    if (!samplesByGapSize.has(key)) {
      samplesByGapSize.set(key, []);
    }

    const samples = samplesByGapSize.get(key);
    if (samples.length < 3) {
      // Keep up to 3 samples per gap size
      samples.push({
        filename,
        gapSize: parseInt(gapSize),
        len1: parseInt(len1),
        len2: parseInt(len2),
        inputBytes,
        encoded,
      });
    }
  }

  // Verify that both runs (AAAA... and BBBB...) appear as raw bytes in encoding
  // Raw passthrough uses escape chars: , ; _ ~ |
  // Check if AAAA and BBBB appear literally in the encoded output
  const hasRawAAAA = encoded.includes("AAAA") || encoded.includes("AAA");
  const hasRawBBBB = encoded.includes("BBBB") || encoded.includes("BBB");

  if (hasRawAAAA || hasRawBBBB) {
    rawPassthroughCount++;
  } else {
    nonRawCount++;
    // This might be expected for very short runs or certain gap sizes
    // Only report if both runs are missing
    if (!hasRawAAAA && !hasRawBBBB) {
      unexpectedPatterns.push({
        filename,
        encoded,
        inputBytes,
      });
    }
  }
}

// Print summary
console.log("=".repeat(80));
console.log("SUMMARY");
console.log("=".repeat(80));
console.log(`Total files encoded: ${totalEncoded}`);
console.log(
  `Files with raw passthrough (AAAA/BBBB visible): ${rawPassthroughCount}`
);
console.log(`Files without raw passthrough: ${nonRawCount}`);
console.log();

if (unexpectedPatterns.length > 0) {
  console.log("UNEXPECTED ENCODING PATTERNS (neither AAAA nor BBBB visible):");
  console.log("-".repeat(80));
  for (const { filename, encoded, inputBytes } of unexpectedPatterns.slice(
    0,
    10
  )) {
    console.log(`File: ${filename}`);
    console.log(`  Input (hex): ${Array.from(inputBytes, (b) =>
      b.toString(16).padStart(2, "0")
    ).join(" ")}`);
    console.log(`  Encoded: ${encoded}`);
    console.log();
  }
  if (unexpectedPatterns.length > 10) {
    console.log(`... and ${unexpectedPatterns.length - 10} more`);
  }
  console.log();
}

// Print sample encodings for each gap size
console.log("SAMPLE ENCODINGS BY GAP SIZE:");
console.log("-".repeat(80));

const sortedGapSizes = Array.from(samplesByGapSize.keys()).sort((a, b) => {
  const numA = parseInt(a.split("-")[1]);
  const numB = parseInt(b.split("-")[1]);
  return numA - numB;
});

for (const key of sortedGapSizes) {
  const samples = samplesByGapSize.get(key);
  const gapSize = samples[0].gapSize;

  console.log(`\nGap size: ${gapSize} byte${gapSize !== 1 ? "s" : ""}`);
  console.log("-".repeat(40));

  for (const { filename, len1, len2, inputBytes, encoded } of samples) {
    // Show input structure
    const inputHex = Array.from(inputBytes, (b) =>
      b.toString(16).padStart(2, "0")
    ).join(" ");
    console.log(`  ${filename}`);
    console.log(`    Runs: ${len1} A's, ${gapSize} gap, ${len2} B's`);
    console.log(`    Input (hex): ${inputHex}`);
    console.log(`    Encoded: ${encoded}`);

    // Analyze encoding pattern
    const hasComma = encoded.includes(",");
    const hasSemicolon = encoded.includes(";");
    const hasUnderscore = encoded.includes("_");
    const hasTilde = encoded.includes("~");
    const hasPipe = encoded.includes("|");

    const escapes = [];
    if (hasComma) escapes.push(",");
    if (hasSemicolon) escapes.push(";");
    if (hasUnderscore) escapes.push("_");
    if (hasTilde) escapes.push("~");
    if (hasPipe) escapes.push("|");

    if (escapes.length > 0) {
      console.log(`    Escapes used: ${escapes.join(" ")}`);
    } else {
      console.log(`    Escapes used: none (standard Z85)`);
    }
    console.log();
  }
}

console.log("=".repeat(80));
console.log("Done! All .encoded files have been created.");
