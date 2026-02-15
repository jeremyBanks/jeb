#!/usr/bin/env -S deno run --allow-read
import { decode } from "./z855.ts";

const testFile = "test-cases/padding-8-0-pipes.encoded";
const encoded = Deno.readTextFileSync(testFile);

console.log("Testing:", testFile);
console.log("Encoded content:", JSON.stringify(encoded));
console.log("Encoded (visual):", encoded);
console.log("Length:", encoded.length, "chars");
console.log();

console.log("Structure breakdown:");
console.log("  '8' = length prefix (8 bytes)");
console.log("  '|' = separator");
console.log("  Next 8 chars = raw data");
console.log("  Remaining chars = padding (should be 10 chars of pipes)");
console.log();

try {
  const decoded = decode(encoded);
  console.log("✅ Decode succeeded!");
  console.log("Decoded length:", decoded.length, "bytes");
  console.log("Decoded (hex):", Array.from(decoded).map(b => b.toString(16).padStart(2, '0')).join(' '));
} catch (e) {
  console.log("❌ Decode failed:", e.message);
  console.log();
  console.log("This is EXPECTED - the decoder incorrectly checks padding content!");
}
