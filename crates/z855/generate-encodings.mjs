#!/usr/bin/env node
import { readdirSync, readFileSync, writeFileSync, existsSync } from "node:fs";
import { join } from "node:path";
import { z855 } from "./min.mjs";

const TEST_CASES_DIR = "./test-cases";

// Get all test case input files
const files = readdirSync(TEST_CASES_DIR);
const inputFiles = files.filter(f => f.endsWith(".input"));

let generated = 0;
let skipped = 0;

for (const inputFile of inputFiles) {
  const name = inputFile.replace(".input", "");
  const inputPath = join(TEST_CASES_DIR, inputFile);
  const encodedPath = join(TEST_CASES_DIR, name + ".encoded");

  // Skip error cases
  const isError = name.includes("invalid") || name.includes("overflow") ||
                  name.includes("incomplete") || name.includes("error");

  if (isError) {
    skipped++;
    continue;
  }

  // Skip if .encoded already exists
  if (existsSync(encodedPath)) {
    skipped++;
    continue;
  }

  // Generate encoding using min.z855
  const input = new Uint8Array(readFileSync(inputPath));
  const encoded = z855(input);

  writeFileSync(encodedPath, encoded, "utf8");
  console.log(`Generated: ${name}.encoded (${encoded.length} chars)`);
  generated++;
}

console.log(`\nSummary: ${generated} generated, ${skipped} skipped, ${inputFiles.length} total`);
