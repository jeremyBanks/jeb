#!/usr/bin/env node
import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { z855 as minZ855, decode as minDecode } from "./min.mjs";
import { encode as z855Encode } from "./z855.ts";

const TEST_CASES_DIR = "./test-cases";
const files = readdirSync(TEST_CASES_DIR);
const inputFiles = files.filter(f => f.endsWith(".input"));

let same = 0;
let different = 0;
let errors = 0;

for (const inputFile of inputFiles) {
  const name = inputFile.replace(".input", "");
  const inputPath = join(TEST_CASES_DIR, inputFile);

  const isError = name.includes("invalid") || name.includes("overflow") ||
                  name.includes("incomplete") || name.includes("error");
  if (isError) continue;

  const input = new Uint8Array(readFileSync(inputPath));

  try {
    const minEncoded = minZ855(input);
    const z855Encoded = z855Encode(input);

    if (minEncoded === z855Encoded) {
      same++;
    } else {
      different++;
      console.log(`DIFF: ${name}`);
      console.log(`  min:  ${minEncoded.substring(0, 80)}${minEncoded.length > 80 ? '...' : ''}`);
      console.log(`  z855: ${z855Encoded.substring(0, 80)}${z855Encoded.length > 80 ? '...' : ''}`);

      // Verify both decode to same input
      const minDecoded = minDecode(minEncoded);
      const arraysEqual = input.every((v, i) => v === minDecoded[i]) && input.length === minDecoded.length;
      if (!arraysEqual) {
        console.log(`  ERROR: min encoding doesn't round-trip!`);
        errors++;
      }
    }
  } catch (e) {
    console.log(`ERROR in ${name}: ${e.message}`);
    errors++;
  }
}

console.log(`\nSummary: ${same} same, ${different} different, ${errors} errors`);
