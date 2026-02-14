import { encode, decode } from "./z855.ts";

// Detailed analysis of 5-byte case

const input = new Uint8Array([0x61, 0x62, 0x63, 0x64, 0x65]); // "abcde"
console.log("Input:", Array.from(input).map(b => `0x${b.toString(16)}`).join(', '));
console.log("Input as string:", Array.from(input).map(b => String.fromCharCode(b)).join(''));

const encoded = encode(input);
console.log("\nEncoded:", encoded);
console.log("Encoded chars:", Array.from(encoded).map((c, i) => `[${i}]='${c}' (0x${c.charCodeAt(0).toString(16)})`).join(', '));

// Standard Z85 encoding for comparison
const Z85_ALPHABET = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";

// First 4 bytes as a 32-bit value (big-endian)
const val4 = (input[0] << 24) | (input[1] << 16) | (input[2] << 8) | input[3];
console.log(`\nFirst 4 bytes as u32 (BE): 0x${val4.toString(16)} = ${val4}`);

// What would standard Z85 encode this as?
let stdZ85 = "";
let v = val4;
for (let i = 0; i < 5; i++) {
  stdZ85 = Z85_ALPHABET[v % 85] + stdZ85;
  v = Math.floor(v / 85);
}
console.log("Standard Z85 (4 bytes):", stdZ85);

// What about just the first byte (position 0)?
const val1 = input[0];
console.log(`\nFirst byte: 0x${val1.toString(16)} = ${val1}`);
// 1 byte → 2 Z85 chars
const z85_1byte = Z85_ALPHABET[Math.floor(val1 / 85)] + Z85_ALPHABET[val1 % 85];
console.log("Standard Z85 (1 byte):", z85_1byte);

// So if the encoder is using 1 char before the escape, what is that char encoding?
console.log("\nActual leading char:", encoded[0], "=", Z85_ALPHABET.indexOf(encoded[0]));

// Let's decode and see what happens
const decoded = decode(encoded);
console.log("\nDecoded:", Array.from(decoded).map(b => `0x${b.toString(16)}`).join(', '));
console.log("Match:", Array.from(input).every((b, i) => b === decoded[i]));
