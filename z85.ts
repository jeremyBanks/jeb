// Z85 Encoding/Decoding Implementation
// =====================================
//
// Z85 is a binary-to-text encoding scheme defined by ZeroMQ (RFC 32).
// It encodes binary data into printable ASCII characters, similar to Base64
// but with a different alphabet and encoding ratio.
//
// Key Properties:
// - Alphabet: 85 printable ASCII characters (excluding characters that might
//   cause issues in various contexts like quotes, backslash, etc.)
// - Encoding ratio: 4 bytes -> 5 characters (80% efficiency vs 75% for Base64)
// - Big-endian byte order for the 4-byte blocks
//
// This implementation extends standard Z85 to support arbitrary-length input
// (not just multiples of 4 bytes) using a scheme similar to unpadded Base64:
// - 1 byte  -> 2 characters
// - 2 bytes -> 3 characters
// - 3 bytes -> 4 characters
// - 4 bytes -> 5 characters
//
// =============================================================================
// Raw Passthrough Extension (`,` escape)
// =============================================================================
//
// This implementation includes an extension to Z85 that allows raw passthrough
// of 4-byte blocks when ALL bytes are "safe" printable characters.
//
// ENCODING:
// - When a 4-byte block consists entirely of "safe" characters, the encoder
//   MAY output `,` followed by the 4 raw bytes (5 chars total) instead of
//   standard Z85 encoding.
// - The `,` character acts as an escape marker indicating raw passthrough.
// - Safe characters for encoding decisions: Z85 alphabet plus `,;|~_`
//   (total: 0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#,;|~_)
// - Standard Z85 encoding is always valid; the encoder SHOULD use `,` passthrough
//   when possible for better readability.
// - `,` can ONLY appear at position 0 of a 5-character block (block-aligned).
//
// DECODING:
// - When `,` is encountered at a block boundary (position 0 mod 5), the next
//   4 bytes are taken as literal output (raw passthrough).
// - The decoder does NOT validate that raw bytes are "safe" - it trusts the input.
// - Otherwise, standard Z85 decoding is applied.
//
// This extension is backward-compatible: any standard Z85 input decodes correctly,
// and extended output can be decoded by extended decoders.

/**
 * The Z85 alphabet: 85 printable ASCII characters in a specific order.
 * Characters are chosen to be safe in most contexts (no quotes, backslash, etc.)
 * Index 0 = '0', Index 84 = '#'
 */
const Z85_ALPHABET =
  "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";

/**
 * The raw passthrough escape character.
 * When this appears at position 0 of a 5-character block during decoding,
 * the following 4 characters are taken as literal bytes (no Z85 decoding).
 */
const RAW_ESCAPE = ",".charCodeAt(0); // 0x2C

/**
 * Extended safe characters for raw passthrough encoding decisions.
 * These are the Z85 alphabet (85 chars) plus 5 additional safe characters: `,;|~_`
 * Total: 90 characters that are considered "safe" for raw passthrough.
 *
 * A 4-byte block qualifies for raw passthrough encoding (`,XXXX` format)
 * only if ALL 4 bytes are in this safe set.
 */
const SAFE_CHARS =
  "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#,;|~_";

/**
 * Lookup table for safe character detection during encoding.
 * For each byte 0-255, stores true if the byte is a safe character for raw passthrough.
 */
const SAFE_CHAR_TABLE: boolean[] = buildSafeCharTable();

/**
 * Build the safe character lookup table.
 */
function buildSafeCharTable(): boolean[] {
  const table: boolean[] = new Array(256).fill(false);
  for (let i = 0; i < SAFE_CHARS.length; i++) {
    table[SAFE_CHARS.charCodeAt(i)] = true;
  }
  return table;
}

/**
 * Lookup table for decoding: maps ASCII character code -> Z85 digit value (0-84)
 * Invalid characters are marked with -1
 */
const Z85_DECODE_TABLE: number[] = buildDecodeTable();

/**
 * Build the decode lookup table.
 * For each ASCII code 0-255, stores either the Z85 digit value (0-84) or -1 if invalid.
 */
function buildDecodeTable(): number[] {
  const table: number[] = new Array(256).fill(-1);
  for (let i = 0; i < 85; i++) {
    table[Z85_ALPHABET.charCodeAt(i)] = i;
  }
  return table;
}

/**
 * Error class for Z85 decoding failures
 */
export class Z85DecodeError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "Z85DecodeError";
  }
}

/**
 * Encode arbitrary bytes into a Z85 string.
 *
 * # Algorithm
 *
 * For each 4-byte block:
 * 1. Check if all 4 bytes are safe for raw passthrough
 *    - If so, output `,` followed by the 4 raw bytes (5 chars total)
 * 2. Otherwise, apply standard Z85 encoding:
 *    - Interpret the 4 bytes as a big-endian u32
 *    - Convert to base-85 by repeatedly dividing by 85
 *    - Map each base-85 digit to the corresponding alphabet character
 *
 * The division produces digits in reverse order (least significant first),
 * so we fill the output buffer from right to left.
 *
 * For trailing bytes (1-3 bytes), we:
 * 1. Pad conceptually with zeros on the right to form a partial block
 * 2. Encode only the significant portion (standard Z85, no passthrough):
 *    - 1 byte  (8 bits)  -> 2 chars
 *    - 2 bytes (16 bits) -> 3 chars
 *    - 3 bytes (24 bits) -> 4 chars
 *
 * @param input - The bytes to encode (Uint8Array)
 * @returns The Z85 encoded string
 */
export function encode(input: Uint8Array): string {
  // Handle empty input
  if (input.length === 0) {
    return "";
  }

  // Calculate output size:
  // - Full 4-byte blocks: each produces 5 characters (either Z85 or `,` + 4 raw)
  // - Trailing n bytes (1-3): produces n+1 characters (always Z85, no passthrough)
  const fullBlocks = Math.floor(input.length / 4);
  const trailing = input.length % 4;
  const trailingChars = trailing > 0 ? trailing + 1 : 0;
  const outputLen = fullBlocks * 5 + trailingChars;

  // Pre-allocate output array
  const output: string[] = new Array(outputLen);
  let outIdx = 0;

  // Process full 4-byte blocks
  let inIdx = 0;
  while (inIdx + 4 <= input.length) {
    // Check if all 4 bytes are safe for raw passthrough.
    // If so, use `,XXXX` format for better readability.
    // Otherwise, use standard Z85 encoding.
    if (isBlockSafeForPassthrough(input, inIdx)) {
      // Raw passthrough: output `,` followed by the 4 raw bytes
      output[outIdx] = ",";
      output[outIdx + 1] = String.fromCharCode(input[inIdx]);
      output[outIdx + 2] = String.fromCharCode(input[inIdx + 1]);
      output[outIdx + 3] = String.fromCharCode(input[inIdx + 2]);
      output[outIdx + 4] = String.fromCharCode(input[inIdx + 3]);
    } else {
      // Standard Z85 encoding
      // Convert 4 bytes to big-endian u32
      // JavaScript bitwise operations work on 32-bit signed integers,
      // so we use >>> 0 to convert to unsigned
      const value =
        ((input[inIdx] << 24) |
          (input[inIdx + 1] << 16) |
          (input[inIdx + 2] << 8) |
          input[inIdx + 3]) >>>
        0;

      // Convert to base-85, filling 5 characters from right to left
      encodeBlockToArray(value, output, outIdx);
    }
    outIdx += 5;
    inIdx += 4;
  }

  // Handle trailing bytes (1, 2, or 3 bytes)
  // Note: Raw passthrough is NOT used for trailing bytes - only full 4-byte blocks
  if (trailing > 0) {
    const numBytes = trailing;
    const numChars = numBytes + 1;

    // Construct the value from available bytes (big-endian, left-aligned)
    let value: number;
    if (numBytes === 1) {
      value = input[inIdx];
    } else if (numBytes === 2) {
      value = (input[inIdx] << 8) | input[inIdx + 1];
    } else {
      // 3 bytes: construct 24-bit value
      value = (input[inIdx] << 16) | (input[inIdx + 1] << 8) | input[inIdx + 2];
    }

    // Encode the partial block
    encodePartialToArray(value, numChars, output, outIdx);
  }

  return output.join("");
}

/**
 * Check if a 4-byte block consists entirely of safe characters for raw passthrough.
 *
 * A block qualifies for raw passthrough if ALL 4 bytes are in the SAFE_CHARS set
 * (Z85 alphabet plus `,;|~_`). This allows the encoder to output `,XXXX` format
 * instead of standard Z85 encoding, which can improve readability for text-like data.
 */
function isBlockSafeForPassthrough(input: Uint8Array, startIdx: number): boolean {
  return (
    SAFE_CHAR_TABLE[input[startIdx]] &&
    SAFE_CHAR_TABLE[input[startIdx + 1]] &&
    SAFE_CHAR_TABLE[input[startIdx + 2]] &&
    SAFE_CHAR_TABLE[input[startIdx + 3]]
  );
}

/**
 * Encode a full 32-bit value into exactly 5 Z85 characters.
 * Fills the array from right to left with base-85 digits.
 */
function encodeBlockToArray(
  value: number,
  output: string[],
  startIdx: number
): void {
  // Fill from right to left (least significant digit first)
  for (let i = 4; i >= 0; i--) {
    output[startIdx + i] = Z85_ALPHABET[value % 85];
    value = Math.floor(value / 85);
  }
}

/**
 * Encode a partial value (from 1-3 bytes) into the appropriate number of Z85 characters.
 *
 * The math: for n input bytes, we need n+1 output characters.
 * We encode as if the value represents the most significant bits of a larger number.
 */
function encodePartialToArray(
  value: number,
  numChars: number,
  output: string[],
  startIdx: number
): void {
  // Fill from right to left
  for (let i = numChars - 1; i >= 0; i--) {
    output[startIdx + i] = Z85_ALPHABET[value % 85];
    value = Math.floor(value / 85);
  }
}

/**
 * Decode a Z85 string back into bytes.
 *
 * # Algorithm
 *
 * For each 5-character block:
 * 1. Check if the first character is `,` (raw passthrough escape)
 *    - If so, take the next 4 characters as literal bytes (no Z85 decoding)
 *    - The decoder does NOT validate that raw bytes are "safe" - it trusts the input
 * 2. Otherwise, apply standard Z85 decoding:
 *    - Map each character to its base-85 digit value (0-84)
 *    - Accumulate: value = d0*85^4 + d1*85^3 + d2*85^2 + d3*85 + d4
 *    - Convert the u32 value to 4 big-endian bytes
 *
 * # Non-Aligned Passthrough (Advanced)
 *
 * When `,` appears at position P (1-4) within a 5-char block, it interrupts
 * the Z85 encoding of surrounding blocks:
 * - First P chars are partial Z85 of the "before" block
 * - `,` + next 4 chars are raw passthrough bytes
 * - The "before" block is ambiguous: we compute all possible 32-bit values
 *   and output the MINIMUM (canonical) value (big-endian interpretation)
 *
 * For trailing characters (2-4 chars), we:
 * 1. Decode to get the partial value (standard Z85, no passthrough for partials)
 * 2. Extract only the appropriate number of bytes:
 *    - 2 chars -> 1 byte
 *    - 3 chars -> 2 bytes
 *    - 4 chars -> 3 bytes
 *
 * @param input - The Z85 encoded string
 * @returns The decoded bytes as Uint8Array
 * @throws Z85DecodeError on invalid input
 */
export function decode(input: string): Uint8Array {
  // Handle empty input
  if (input.length === 0) {
    return new Uint8Array(0);
  }

  // With non-aligned passthrough, we can't pre-calculate output size easily
  // because commas can appear anywhere and shift the alignment.
  // We'll build output incrementally.
  const outputChunks: number[] = [];

  let inIdx = 0;

  // We track our "logical" position within the Z85 block structure.
  // blockPos is 0-4, representing position within current 5-char Z85 block.
  // When we encounter a comma at blockPos P (0-4), we handle it specially.
  let blockPos = 0;
  // Accumulated Z85 digits for the current block (0-5 digits)
  const currentBlockDigits: number[] = [];

  while (inIdx < input.length) {
    const charCode = input.charCodeAt(inIdx);

    if (charCode === RAW_ESCAPE) {
      // Found a comma - this is a passthrough marker
      const P = blockPos; // Position within the 5-char block (0-4)

      // Ensure we have at least 4 more characters for the passthrough bytes
      if (inIdx + 4 >= input.length) {
        throw new Z85DecodeError("incomplete passthrough sequence");
      }

      // Extract the 4 passthrough bytes
      const passBytes = [
        input.charCodeAt(inIdx + 1),
        input.charCodeAt(inIdx + 2),
        input.charCodeAt(inIdx + 3),
        input.charCodeAt(inIdx + 4),
      ];

      if (P === 0) {
        // Block-aligned passthrough: simple case, just output the 4 bytes
        outputChunks.push(...passBytes);
        inIdx += 5; // Skip comma + 4 bytes
        // blockPos stays at 0, currentBlockDigits stays empty
      } else {
        // Non-aligned passthrough at position P (1-4)
        // We have P high-order Z85 digits in currentBlockDigits
        // The passthrough bytes provide (4-P) known low-order bytes for the "before" block

        // The first (4-P) passthrough bytes are the known low bytes of the "before" block
        const numKnownBytes = 4 - P;
        const knownLowBytes = passBytes.slice(0, numKnownBytes);

        // Compute the canonical minimum for the "before" block
        const beforeValue = computeCanonicalMinimum(currentBlockDigits, knownLowBytes);

        // Output the 4 bytes of the "before" block
        outputChunks.push((beforeValue >>> 24) & 0xff);
        outputChunks.push((beforeValue >>> 16) & 0xff);
        outputChunks.push((beforeValue >>> 8) & 0xff);
        outputChunks.push(beforeValue & 0xff);

        // Output the 4 passthrough bytes
        outputChunks.push(...passBytes);

        // Reset block state
        currentBlockDigits.length = 0;
        blockPos = 0;

        // Move past comma + 4 passthrough bytes
        inIdx += 5;
      }
    } else {
      // Regular Z85 character
      const digit = Z85_DECODE_TABLE[charCode];
      if (digit === -1) {
        throw new Z85DecodeError(
          `invalid character in Z85 input: 0x${charCode.toString(16).padStart(2, "0").toUpperCase()}`
        );
      }

      currentBlockDigits.push(digit);
      blockPos++;
      inIdx++;

      // If we've completed a 5-character block, decode it
      if (blockPos === 5) {
        // Decode the full block
        let value = 0;
        for (const d of currentBlockDigits) {
          value = value * 85 + d;
        }

        // Check for overflow
        if (value > 0xffffffff) {
          throw new Z85DecodeError("Z85 value overflow");
        }

        // Output 4 bytes
        outputChunks.push((value >>> 24) & 0xff);
        outputChunks.push((value >>> 16) & 0xff);
        outputChunks.push((value >>> 8) & 0xff);
        outputChunks.push(value & 0xff);

        // Reset for next block
        currentBlockDigits.length = 0;
        blockPos = 0;
      }
    }
  }

  // Handle trailing partial block (if any)
  if (currentBlockDigits.length > 0) {
    const numChars = currentBlockDigits.length;

    // Invalid: 1 character doesn't map to a valid byte count
    if (numChars === 1) {
      throw new Z85DecodeError("invalid Z85 input length");
    }

    // Decode partial block: 2 chars -> 1 byte, 3 chars -> 2 bytes, 4 chars -> 3 bytes
    let value = 0;
    for (const d of currentBlockDigits) {
      value = value * 85 + d;
    }

    const numBytes = numChars - 1;

    // Check overflow based on expected byte count
    if (numBytes === 1 && value > 0xff) {
      throw new Z85DecodeError("Z85 value overflow");
    } else if (numBytes === 2 && value > 0xffff) {
      throw new Z85DecodeError("Z85 value overflow");
    } else if (numBytes === 3 && value > 0xffffff) {
      throw new Z85DecodeError("Z85 value overflow");
    }

    // Output the appropriate number of bytes
    if (numBytes === 1) {
      outputChunks.push(value);
    } else if (numBytes === 2) {
      outputChunks.push((value >>> 8) & 0xff);
      outputChunks.push(value & 0xff);
    } else {
      // 3 bytes
      outputChunks.push((value >>> 16) & 0xff);
      outputChunks.push((value >>> 8) & 0xff);
      outputChunks.push(value & 0xff);
    }
  }

  return new Uint8Array(outputChunks);
}

/**
 * Decode a full 5-character block into a number.
 * Throws Z85DecodeError if any character is invalid or if the value overflows u32.
 */
function decodeBlock(input: string, startIdx: number): number {
  let value = 0;

  // Accumulate: value = d0*85^4 + d1*85^3 + d2*85^2 + d3*85 + d4
  // We need to check for overflow since JavaScript numbers are 64-bit floats
  // but we want to represent a u32
  for (let i = 0; i < 5; i++) {
    const charCode = input.charCodeAt(startIdx + i);
    const digit = Z85_DECODE_TABLE[charCode];

    if (digit === -1) {
      throw new Z85DecodeError(
        `invalid character in Z85 input: 0x${charCode.toString(16).padStart(2, "0").toUpperCase()}`
      );
    }

    value = value * 85 + digit;
  }

  // Check for overflow (max valid Z85 5-char value is 85^5 - 1 = 4,437,053,124)
  // But we need it to fit in u32 (max 4,294,967,295 = 0xFFFFFFFF)
  if (value > 0xffffffff) {
    throw new Z85DecodeError("Z85 value overflow");
  }

  return value;
}

/**
 * Decode a partial block (2, 3, or 4 characters) into a number.
 * The value represents a partial number (1, 2, or 3 bytes worth).
 */
function decodePartialBlock(
  input: string,
  startIdx: number,
  numChars: number
): number {
  let value = 0;

  for (let i = 0; i < numChars; i++) {
    const charCode = input.charCodeAt(startIdx + i);
    const digit = Z85_DECODE_TABLE[charCode];

    if (digit === -1) {
      throw new Z85DecodeError(
        `invalid character in Z85 input: 0x${charCode.toString(16).padStart(2, "0").toUpperCase()}`
      );
    }

    value = value * 85 + digit;
  }

  // No overflow check here - we check in the caller based on expected byte count
  return value;
}

// =============================================================================
// Non-Aligned Passthrough Support
// =============================================================================
//
// When `,` appears at position P (1-4) within a 5-char block, the Z85 encoding
// is interrupted. The "before" block becomes ambiguous because we only have
// P high-order Z85 digits and the raw passthrough bytes provide (4-P) known
// low-order input bytes.
//
// To resolve ambiguity deterministically, we compute ALL possible 32-bit values
// that could have produced the observed partial Z85 + known bytes, then select
// the MINIMUM value (canonical). This ensures:
// - Decoding is deterministic and unambiguous
// - Encoding can check if actual value equals canonical minimum before using passthrough

/**
 * Compute the canonical (minimum) 32-bit value for an ambiguous "before" block.
 *
 * Given P high-order Z85 digits and (4-P) known low-order input bytes (from passthrough),
 * find the minimum 32-bit value V such that:
 * 1. V's Z85 encoding starts with the given P digits
 * 2. V's big-endian bytes end with the given (4-P) known bytes
 *
 * Algorithm:
 * - P Z85 digits define a range: [base, base + 85^(5-P)) where base = digits * 85^(5-P)
 * - Within this range, find values where low bytes match the known passthrough bytes
 * - Return the minimum such value
 *
 * @param highDigits - Array of P Z85 digit values (0-84)
 * @param knownLowBytes - Array of (4-P) known low-order bytes from passthrough
 * @returns The canonical minimum 32-bit value
 * @throws Z85DecodeError if no valid value exists (should not happen with valid input)
 */
function computeCanonicalMinimum(
  highDigits: number[],
  knownLowBytes: number[]
): number {
  const P = highDigits.length;
  const numKnownBytes = knownLowBytes.length; // Should be 4 - P

  // Compute the base value from high Z85 digits
  // base = d0 * 85^(5-1) + d1 * 85^(5-2) + ... + d(P-1) * 85^(5-P)
  // This is equivalent to: digits interpreted as base-85 number, then multiplied by 85^(5-P)
  let base = 0;
  for (let i = 0; i < P; i++) {
    base = base * 85 + highDigits[i];
  }

  // The range of possible values is [base * 85^(5-P), (base+1) * 85^(5-P))
  // But we need to express this in terms of the actual 32-bit value range
  const power = Math.pow(85, 5 - P);
  const rangeStart = base * power;
  const rangeEnd = (base + 1) * power;

  // Now we need to find values in [rangeStart, rangeEnd) whose big-endian bytes
  // end with knownLowBytes
  //
  // The knownLowBytes constrain the low (4-P) bytes of the 32-bit value.
  // So we construct the constraint from the known bytes.
  let knownPart = 0;
  for (let i = 0; i < numKnownBytes; i++) {
    knownPart = (knownPart << 8) | knownLowBytes[i];
  }

  // The mask for the known bytes (low numKnownBytes bytes)
  const mask = (1 << (numKnownBytes * 8)) - 1;
  // If numKnownBytes is 0, mask is 0 and any value works

  // Find the minimum value in [rangeStart, rangeEnd) where (value & mask) === knownPart
  //
  // We need to find the smallest V >= rangeStart such that V % (mask+1) === knownPart
  // and V < rangeEnd

  if (numKnownBytes === 0) {
    // P = 4: No constraint from known bytes, just return rangeStart
    // But we need to check it fits in u32
    if (rangeStart > 0xffffffff) {
      throw new Z85DecodeError("Z85 value overflow in non-aligned decode");
    }
    return rangeStart;
  }

  // Find smallest V >= rangeStart where V ends with knownPart
  const modulus = mask + 1; // 2^(numKnownBytes * 8)
  const startRemainder = rangeStart % modulus;

  let candidate: number;
  if (startRemainder <= knownPart) {
    candidate = rangeStart - startRemainder + knownPart;
  } else {
    candidate = rangeStart - startRemainder + modulus + knownPart;
  }

  // Verify candidate is in range and fits in u32
  if (candidate >= rangeEnd) {
    throw new Z85DecodeError("no valid value for non-aligned passthrough decode");
  }
  if (candidate > 0xffffffff) {
    throw new Z85DecodeError("Z85 value overflow in non-aligned decode");
  }

  return candidate;
}

/**
 * Find the position of `,` in the input string, if any.
 * Returns -1 if no comma found.
 */
function findCommaPosition(input: string, startIdx: number, endIdx: number): number {
  for (let i = startIdx; i < endIdx; i++) {
    if (input.charCodeAt(i) === RAW_ESCAPE) {
      return i;
    }
  }
  return -1;
}
