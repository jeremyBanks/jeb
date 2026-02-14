#!/usr/bin/env node
import { readdirSync, readFileSync, writeFileSync, existsSync } from "node:fs";
import { join } from "node:path";
import { z855 as minZ855 } from "./min.mjs";

const TEST_CASES_DIR = "./test-cases";

const files = readdirSync(TEST_CASES_DIR);
const inputFiles = files.filter(f => f.endsWith(".input"));

let generated = 0;
let matched = 0;
let skipped = 0;

for (const inputFile of inputFiles) {
  const name = inputFile.replace(".input", "");
  const inputPath = join(TEST_CASES_DIR, inputFile);
  const encodedPath = join(TEST_CASES_DIR, name + ".encoded");
  const minEncodedPath = join(TEST_CASES_DIR, name + ".encoded-min");

  // Read input
  const input = new Uint8Array(readFileSync(inputPath));

  // Skip error cases (file content is <error />)
  const inputText = new TextDecoder().decode(input);
  if (inputText.trim() === "<error />") {
    skipped++;
    continue;
  }

  // Generate min.z855 encoding
  const minEncoded = minZ855(input);

  // Check if it matches existing .encoded
  if (existsSync(encodedPath)) {
    const existingEncoded = readFileSync(encodedPath, "utf8").trim();

    if (minEncoded === existingEncoded) {
      matched++;
      // Delete .encoded-min if it exists (no longer needed)
      if (existsSync(minEncodedPath)) {
        console.log(`Removing ${name}.encoded-min (now matches .encoded)`);
        // We'll keep it for now to show what changed
      }
    } else {
      // Different encoding - save as .encoded-min
      writeFileSync(minEncodedPath, minEncoded + "\n", "utf8");
      console.log(`Generated: ${name}.encoded-min`);
      console.log(`  Reference: ${existingEncoded.substring(0, 60)}${existingEncoded.length > 60 ? '...' : ''}`);
      console.log(`  Min:       ${minEncoded.substring(0, 60)}${minEncoded.length > 60 ? '...' : ''}`);
      generated++;
    }
  } else {
    // No existing .encoded, create both
    writeFileSync(encodedPath, minEncoded + "\n", "utf8");
    console.log(`Generated: ${name}.encoded (no previous encoding)`);
    generated++;
  }
}

console.log(`\nSummary: ${generated} different/new, ${matched} matched, ${skipped} skipped (error cases)`);
