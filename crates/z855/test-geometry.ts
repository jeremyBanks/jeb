import { encode, decode } from "./z855.ts";

// Test various sizes to see what the encoder actually produces

console.log("=== Testing 5, 6, 7 byte inputs (all safe chars) ===\n");

for (let len = 5; len <= 7; len++) {
  const input = new Uint8Array(len);
  for (let i = 0; i < len; i++) {
    input[i] = 0x61 + i; // 'a', 'b', 'c', ...
  }
  
  const encoded = z855(input);
  const decoded = decode(encoded);
  
  console.log(`${len} bytes: ${Array.from(input).map(b => String.fromCharCode(b)).join('')}`);
  console.log(`  Encoded: "${encoded}"`);
  console.log(`  Length: ${encoded.length} chars`);
  console.log(`  Decoded: ${Array.from(decoded).map(b => String.fromCharCode(b)).join('')}`);
  console.log(`  Match: ${Array.from(input).every((b, i) => b === decoded[i])}`);
  console.log();
}

console.log("=== Testing 4-byte at different positions ===\n");

// Test 4-byte passthrough at different positions by controlling the input
const testCases = [
  { desc: "4 safe bytes aligned", input: [0x61, 0x62, 0x63, 0x64] }, // "abcd"
  { desc: "1 unsafe + 4 safe", input: [0x00, 0x61, 0x62, 0x63, 0x64] },
  { desc: "2 unsafe + 4 safe", input: [0x00, 0x01, 0x61, 0x62, 0x63, 0x64] },
  { desc: "3 unsafe + 4 safe", input: [0x00, 0x01, 0x02, 0x61, 0x62, 0x63, 0x64] },
];

for (const { desc, input } of testCases) {
  const inputBytes = new Uint8Array(input);
  const encoded = z855(inputBytes);
  const decoded = decode(encoded);
  
  console.log(desc);
  console.log(`  Encoded: "${encoded}"`);
  console.log(`  Roundtrip: ${Array.from(inputBytes).every((b, i) => b === decoded[i])}`);
  console.log();
}
