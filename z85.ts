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

  // Validate input length
  // Valid lengths: 0, 2, 3, 4, 5, 7, 8, 9, 10, 12, ...
  // Invalid: 1, 6, 11, 16, ... (i.e., 1 mod 5)
  const trailingChars = input.length % 5;
  if (trailingChars === 1) {
    throw new Z85DecodeError("invalid Z85 input length");
  }

  // Calculate output size
  const fullBlocks = Math.floor(input.length / 5);
  const trailingBytes = trailingChars > 0 ? trailingChars - 1 : 0;
  const outputLen = fullBlocks * 4 + trailingBytes;

  const output = new Uint8Array(outputLen);
  let outIdx = 0;
  let inIdx = 0;

  // Process full 5-character blocks
  while (inIdx + 5 <= input.length) {
    // Check for raw passthrough escape at block boundary.
    // When `,` is the first character of a 5-char block, the next 4 bytes
    // are taken as literal output (no Z85 decoding, no content validation).
    if (input.charCodeAt(inIdx) === RAW_ESCAPE) {
      // Raw passthrough: copy the 4 bytes after `,` directly to output
      output[outIdx] = input.charCodeAt(inIdx + 1);
      output[outIdx + 1] = input.charCodeAt(inIdx + 2);
      output[outIdx + 2] = input.charCodeAt(inIdx + 3);
      output[outIdx + 3] = input.charCodeAt(inIdx + 4);
    } else {
      // Standard Z85 decoding
      const value = decodeBlock(input, inIdx);

      // Convert u32 to 4 big-endian bytes
      output[outIdx] = (value >>> 24) & 0xff;
      output[outIdx + 1] = (value >>> 16) & 0xff;
      output[outIdx + 2] = (value >>> 8) & 0xff;
      output[outIdx + 3] = value & 0xff;
    }

    inIdx += 5;
    outIdx += 4;
  }

  // Process trailing characters (2, 3, or 4 chars)
  // Note: Raw passthrough does NOT apply to trailing blocks - only full 5-char blocks
  if (trailingChars > 0) {
    const numBytes = trailingChars - 1;

    // Decode the partial block (standard Z85 only)
    const value = decodePartialBlock(input, inIdx, trailingChars);

    // Extract the appropriate number of bytes (most significant first)
    // The value represents a number that should fit in numBytes bytes
    if (numBytes === 1) {
      if (value > 0xff) {
        throw new Z85DecodeError("Z85 value overflow");
      }
      output[outIdx] = value;
    } else if (numBytes === 2) {
      if (value > 0xffff) {
        throw new Z85DecodeError("Z85 value overflow");
      }
      output[outIdx] = (value >>> 8) & 0xff;
      output[outIdx + 1] = value & 0xff;
    } else {
      // 3 bytes
      if (value > 0xffffff) {
        throw new Z85DecodeError("Z85 value overflow");
      }
      output[outIdx] = (value >>> 16) & 0xff;
      output[outIdx + 1] = (value >>> 8) & 0xff;
      output[outIdx + 2] = value & 0xff;
    }
  }

  return output;
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
