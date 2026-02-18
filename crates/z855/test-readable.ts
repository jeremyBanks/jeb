/**
 * test-readable.ts — Test the readable z855 implementation against the test-cases directory.
 *
 * Run with: deno run --allow-read test-readable.ts
 */
import { encode, decode, Z855DecodeError } from "./z855-readable.ts";
import { join } from "https://deno.land/std@0.224.0/path/mod.ts";

const testCasesDir = "test-cases";
const entries = [...Deno.readDirSync(testCasesDir)]
  .filter((e) => e.name.endsWith(".input"))
  .map((e) => e.name.replace(".input", ""))
  .sort();

let passed = 0;
let failed = 0;
const failures: string[] = [];

for (const name of entries) {
  const inputPath = join(testCasesDir, `${name}.input`);
  const encodedPath = join(testCasesDir, `${name}.encoded`);

  const inputBytes = Deno.readFileSync(inputPath);
  let encodedRaw: Uint8Array;
  try {
    encodedRaw = Deno.readFileSync(encodedPath);
  } catch {
    continue; // Skip if no encoded file
  }

  const encodedStr = new TextDecoder("latin1").decode(encodedRaw).trim();

  // Error test cases
  if (new TextDecoder().decode(inputBytes).trim() === "<error />") {
    let threw = false;
    try { decode(encodedStr); } catch (e) { if (e instanceof Z855DecodeError) threw = true; }
    if (threw) {
      passed++;
    } else {
      failed++;
      failures.push(`${name}: expected decode error but got success`);
    }
    continue;
  }

  // Normal test: decode the .encoded file and check it matches .input
  let decoded: Uint8Array;
  try {
    decoded = decode(encodedStr);
  } catch (e) {
    failed++;
    failures.push(`${name}: decode threw: ${e}`);
    continue;
  }

  const inputArr = new Uint8Array(inputBytes);
  if (decoded.length !== inputArr.length || !decoded.every((b, i) => b === inputArr[i])) {
    failed++;
    failures.push(`${name}: decode mismatch (got ${decoded.length} bytes, want ${inputArr.length})`);
    continue;
  }

  // Also verify roundtrip: encode(input) should decode back to input.
  let reDecoded: Uint8Array;
  try {
    const reEncoded = encode(inputArr);
    reDecoded = decode(reEncoded);
  } catch (e) {
    failed++;
    failures.push(`${name}: encode/decode roundtrip threw: ${e}`);
    continue;
  }

  if (reDecoded.length !== inputArr.length || !reDecoded.every((b, i) => b === inputArr[i])) {
    failed++;
    failures.push(`${name}: roundtrip mismatch`);
    continue;
  }

  passed++;
}

console.log(`Passed: ${passed} / ${passed + failed}`);
if (failures.length > 0) {
  console.log("\nFailures:");
  for (const f of failures.slice(0, 30)) console.log("  " + f);
  if (failures.length > 30) console.log(`  ... and ${failures.length - 30} more`);
  Deno.exit(1);
}
