#!/usr/bin/env -S deno run --allow-write
import { encodeZ85 } from "./pure-z85.mjs";
import { z855 } from "./min.mjs";

const chunks = [];

// Moving "A" through "Z" across null bytes
const letters = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
for (let len = 1; len <= letters.length; len++) {
  const str = letters.substring(0, len);
  console.log(`Generating moving '${str}' pattern...`);

  for (let pos = 0; pos <= 64 - len; pos++) {
    const chunk = new Uint8Array(64);
    // chunk is already all zeros by default
    for (let i = 0; i < len; i++) {
      chunk[pos + i] = str.charCodeAt(i);
    }
    chunks.push(chunk);
  }
}

console.log(`\nTotal chunks: ${chunks.length}`);
console.log(`Total bytes: ${chunks.length * 64}`);

// Generate comparison output - just interleaved lines
const lines = [];

for (const chunk of chunks) {
  // Encode with both methods
  const z85 = encodeZ85(chunk);
  const z855encoded = z855(chunk);

  // Split into 80-char lines
  const z85Lines = [];
  const z855Lines = [];

  for (let i = 0; i < z85.length; i += 80) {
    z85Lines.push(z85.slice(i, i + 80));
  }
  for (let i = 0; i < z855encoded.length; i += 80) {
    z855Lines.push(z855encoded.slice(i, i + 80));
  }

  // Interleave Z85 and Z855 lines
  const maxLines = Math.max(z85Lines.length, z855Lines.length);
  for (let i = 0; i < maxLines; i++) {
    if (i < z85Lines.length) {
      lines.push(z85Lines[i]);
    }
    if (i < z855Lines.length) {
      lines.push(z855Lines[i]);
    }
  }

  // Add blank line after each chunk
  lines.push("");
}

// Write to target directory
await Deno.mkdir("target", { recursive: true });
const outputPath = "target/z85-vs-z855-comparison.txt";
await Deno.writeTextFile(outputPath, lines.join("\n"));

console.log(`\nWrote comparison to: ${outputPath}`);
console.log(`Total lines: ${lines.length}`);
