#!/usr/bin/env -S deno run --allow-read
import { decode } from "./z855.ts";
import { readFileSync } from "node:fs";

// Test the pipes pattern with offset 0 (should fail)
const encodedPath = "test-cases/padding-8-0-pipes.encoded";
const inputPath = "test-cases/padding-8-0-pipes.input";

const encodedText = readFileSync(encodedPath, "utf8");
const expectedBytes = new Uint8Array(readFileSync(inputPath));

console.log("Testing:", encodedPath);
console.log("Encoded text:", encodedText);
console.log("Expected bytes:", Array.from(expectedBytes).map(b => `0x${b.toString(16).padStart(2, '0')}`).join(' '));
console.log();

try {
  const decodedBytes = decode(encodedText);
  console.log("✅ Decode succeeded!");
  console.log("Decoded bytes:", Array.from(decodedBytes).map(b => `0x${b.toString(16).padStart(2, '0')}`).join(' '));

  if (decodedBytes.length === expectedBytes.length) {
    let match = true;
    for (let i = 0; i < expectedBytes.length; i++) {
      if (decodedBytes[i] !== expectedBytes[i]) {
        match = false;
        break;
      }
    }
    if (match) {
      console.log("✅ Bytes match!");
    } else {
      console.log("❌ Bytes mismatch!");
    }
  } else {
    console.log(`❌ Length mismatch: expected ${expectedBytes.length}, got ${decodedBytes.length}`);
  }
} catch (e) {
  console.log("❌ Decode failed:", e.message);
  console.log();
  console.log("This is EXPECTED - the decoder has a bug!");
  console.log("The decoder checks padding content (looks for '.' and '|')");
  console.log("instead of skipping padding based on POSITION only.");
}
