#!/usr/bin/env -S deno run --allow-read --allow-write
import { encode } from "./z855.ts";
import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const TEST_CASES_DIR = "./test-cases";

// Get all test case input files
const files = readdirSync(TEST_CASES_DIR);
const inputFiles = files.filter(f => f.endsWith(".input"));

let generated = 0;

for (const inputFile of inputFiles) {
  const name = inputFile.replace(".input", "");
  const inputPath = join(TEST_CASES_DIR, inputFile);
  const encodedPath = join(TEST_CASES_DIR, name + ".encoded");

  // Skip error cases
  const isError = name.includes("invalid") || name.includes("overflow") ||
                  name.includes("incomplete") || name.includes("error");

  if (isError) {
    continue;
  }

  // Generate encoding using z855.ts
  const input = new Uint8Array(readFileSync(inputPath));
  const encoded = encode(input);

  writeFileSync(encodedPath, encoded + "\n", "utf8");
  console.log(`Generated: ${name}.encoded (${encoded.length} chars)`);
  generated++;
}

console.log(`\nTotal: ${generated} files regenerated`);
