// Z855 Encoding/Decoding Implementation
// ======================================
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
// - `,` can appear at position 0 (block-aligned) or position P (1-4) for non-aligned.
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
 * The 4-byte raw passthrough escape character.
 * When this appears at position 0 of a 5-character block during decoding,
 * the following 4 characters are taken as literal bytes (no Z85 decoding).
 */
const RAW_ESCAPE_4 = ",".charCodeAt(0); // 0x2C

/**
 * The 5-byte raw passthrough escape character.
 * When this appears at position P+1 of a 5-character block during decoding,
 * the following 5 characters are taken as literal bytes (no Z85 decoding).
 * Unlike 4-byte passthrough, this outputs P+1 chars before the escape (not P),
 * so no canonical minimum constraint is needed - the extra char fully disambiguates.
 */
const RAW_ESCAPE_5 = ";".charCodeAt(0); // 0x3B

/**
 * The 6-byte raw passthrough escape character.
 * When this appears at position P+1 of a 5-character block during decoding,
 * the following 6 characters are taken as literal bytes (no Z85 decoding).
 * Unlike 4-byte passthrough, this outputs P+1 chars before the escape (not P),
 * so no canonical minimum constraint is needed - the extra char fully disambiguates.
 */
const RAW_ESCAPE_6 = "_".charCodeAt(0); // 0x5F

/**
 * The 7-byte raw passthrough escape character.
 * When this appears at position P+1 of a 5-character block during decoding,
 * the following 7 characters are taken as literal bytes (no Z85 decoding).
 * Unlike 4-byte passthrough, this outputs P+1 chars before the escape (not P),
 * so no canonical minimum constraint is needed - the extra char fully disambiguates.
 */
const RAW_ESCAPE_7 = "~".charCodeAt(0); // 0x7E

/**
 * The 8+ byte raw passthrough escape character (long escape).
 * Structure: [prefix digits][|][raw bytes][padding]
 * The prefix encodes the raw byte count using base-42 with continuation bits.
 * Values 0-41 are terminal digits, 42-83 are continuation digits (+42).
 * Special cases:
 * - 0: rest of input is raw (can be shorter than standard Z85)
 * - 1-7: decoding error (use ,;_~ escapes for these)
 * - 8+: that many raw bytes follow
 */
const RAW_ESCAPE_LONG = "|".charCodeAt(0); // 0x7C

/**
 * Padding character for the long escape (aesthetic, ignored by decoder)
 */
const RAW_ESCAPE_PADDING = ".".charCodeAt(0); // 0x2E

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
 * Check if a byte is an escape character and return the passthrough length.
 * Returns 0 if not an escape, or the length (4, 5, 6, or 7).
 */
function getPassthroughLength(charCode: number): number {
  switch (charCode) {
    case RAW_ESCAPE_4:
      return 4;
    case RAW_ESCAPE_5:
      return 5;
    case RAW_ESCAPE_6:
      return 6;
    case RAW_ESCAPE_7:
      return 7;
    default:
      return 0;
  }
}

/**
 * Check if a char code is the long escape character `|`.
 */
function isLongEscape(charCode: number): boolean {
  return charCode === RAW_ESCAPE_LONG;
}

/**
 * Read a single base-42 self-terminating number from an array of Z85 digit values,
 * reading backwards from the given end position.
 *
 * The format is: [terminal digit][continuation digits...]
 * When reading backwards, we see continuation digits first (>= 42), then the terminal.
 *
 * Returns { value, digitsConsumed }.
 */
function readSingleBase42NumberBackwards(
  digits: number[],
  end: number
): { value: number; digitsConsumed: number } {
  if (end === 0 || end > digits.length) {
    throw new Z855DecodeError("invalid prefix position");
  }

  let value = 0;
  let multiplier = 1;
  let pos = end;
  let count = 0;

  // Read backwards: continuation digits first, then terminal
  while (pos > 0) {
    pos -= 1;
    count += 1;
    const digit = digits[pos];

    if (digit > 83) {
      throw new Z855DecodeError(`invalid prefix digit value: ${digit}`);
    }

    if (digit >= 42) {
      // Continuation digit
      const baseValue = digit - 42;
      value += baseValue * multiplier;
      multiplier *= 42;
      // Check for overflow
      if (value > Number.MAX_SAFE_INTEGER || multiplier > Number.MAX_SAFE_INTEGER) {
        throw new Z855DecodeError("prefix value overflow");
      }
    } else {
      // Terminal digit - this completes the number
      value += digit * multiplier;
      break;
    }
  }

  // Verify we ended on a terminal digit
  if (count === 0 || digits[pos] >= 42) {
    throw new Z855DecodeError("invalid prefix structure");
  }

  return { value, digitsConsumed: count };
}

/**
 * Read offset and length from prefix digits for the `|` escape.
 *
 * Reading backwards from the `|`:
 * 1. Read length (first number encountered going backwards)
 * 2. If there are more digits, read offset (second number)
 *
 * Returns { offset, length }.
 */
function readOffsetAndLengthFromPrefix(prefixDigits: number[]): { offset: number; length: number; offsetDigitsUsed: number } {
  if (prefixDigits.length === 0) {
    throw new Z855DecodeError("no prefix digits");
  }

  // Read length first (backwards from end)
  const { value: length, digitsConsumed: lengthConsumed } = readSingleBase42NumberBackwards(
    prefixDigits,
    prefixDigits.length
  );

  if (lengthConsumed === prefixDigits.length) {
    // Only one number - it's the length, offset = 0
    return { offset: 0, length, offsetDigitsUsed: 0 };
  }

  // There are more digits - read offset (backwards from where length started)
  const offsetEnd = prefixDigits.length - lengthConsumed;
  const { value: offset, digitsConsumed: offsetConsumed } = readSingleBase42NumberBackwards(
    prefixDigits,
    offsetEnd
  );

  // Verify we consumed all digits
  if (lengthConsumed + offsetConsumed !== prefixDigits.length) {
    throw new Z855DecodeError("invalid prefix structure");
  }

  return { offset, length, offsetDigitsUsed: offsetConsumed };
}

/**
 * Generate prefix characters for encoding a length value with the `|` escape.
 *
 * Uses base-42 with continuation bits:
 * - Most significant digit is output as-is (terminal, 0-41)
 * - Remaining digits are output with +42 (continuation, 42-83)
 *
 * Returns an array of Z85 CHARACTERS (not digit values).
 */
function generateLongEscapePrefix(length: number): string[] {
  if (length < 42) {
    // Single digit: just the length value as a Z85 character
    return [Z85_ALPHABET[length]];
  }

  // Multiple digits: extract base-42 digits
  const digits: number[] = [];
  let remaining = length;

  while (remaining > 0) {
    digits.push(remaining % 42);
    remaining = Math.floor(remaining / 42);
  }

  // digits is now in reverse order (least significant first)
  // We need to output: most significant as terminal (0-41), rest as continuation (+42)
  const output: string[] = [];

  // Reverse to get big-endian order
  digits.reverse();

  for (let i = 0; i < digits.length; i++) {
    const d = digits[i];
    if (i === 0) {
      // Most significant digit: terminal (as-is)
      output.push(Z85_ALPHABET[d]);
    } else {
      // Continuation digit: add 42
      output.push(Z85_ALPHABET[d + 42]);
    }
  }

  return output;
}

/**
 * Calculate the standard Z85 output length for a given input byte count.
 */
function z855OutputLength(inputBytes: number): number {
  // ceil(inputBytes * 5 / 4)
  return Math.ceil((inputBytes * 5) / 4);
}

/**
 * Calculate padding needed for a long escape of given raw length.
 * This is the space budget available for padding characters and prefix digits.
 */
function calculatePaddingNeeded(rawLen: number): number {
  if (rawLen < 8) {
    return 0;
  }
  // Available space = z855OutputLength(rawLen) - z855OutputLength(rawLen - 8)
  // This is the space saved by using the long escape instead of standard Z85
  return z855OutputLength(rawLen) - z855OutputLength(rawLen - 8);
}

/**
 * Reverse the bits of a number, treating it as a 64-bit integer.
 *
 * Positions aligned to power-of-2 boundaries have trailing zeros.
 * Bit reversal turns trailing zeros into leading zeros, so aligned
 * positions sort first naturally when comparing reversed values.
 *
 * Since JavaScript doesn't have native 64-bit integers, we use BigInt
 * for the reversal and return the result as a bigint.
 */
function bitReverse(n: number): bigint {
  let x = BigInt(n);
  // Reverse bits of a 64-bit integer
  x = ((x & 0x5555555555555555n) << 1n) | ((x >> 1n) & 0x5555555555555555n);
  x = ((x & 0x3333333333333333n) << 2n) | ((x >> 2n) & 0x3333333333333333n);
  x = ((x & 0x0f0f0f0f0f0f0f0fn) << 4n) | ((x >> 4n) & 0x0f0f0f0f0f0f0f0fn);
  x = ((x & 0x00ff00ff00ff00ffn) << 8n) | ((x >> 8n) & 0x00ff00ff00ff00ffn);
  x = ((x & 0x0000ffff0000ffffn) << 16n) | ((x >> 16n) & 0x0000ffff0000ffffn);
  x = (x << 32n) | (x >> 32n);
  return x;
}

/** Maximum length for a single long passthrough segment (64 KiB implementation limit) */
const MAX_LONG_PASSTHROUGH_LENGTH = 65536;

/**
 * Error class for Z85 decoding failures
 */
export class Z855DecodeError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "Z855DecodeError";
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
 * # Raw Passthrough Extension
 *
 * When a 4-byte block consists entirely of "safe" characters, the encoder
 * MAY use raw passthrough (`,XXXX` format) instead of standard Z85 encoding.
 * This includes non-aligned passthrough where the 4 safe bytes don't align
 * with block boundaries.
 *
 * For non-aligned passthrough at position P (1-4):
 * - The "before" block has P high-order Z85 chars
 * - The passthrough bytes overlap with before/after blocks
 * - We check if the before block value is canonical (minimum)
 * - We try both P and P+1 positions to maximize success rate
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

  // With non-aligned passthrough, we build output incrementally because
  // the alignment can shift based on where we place passthrough sections.
  const outputChars: string[] = [];

  let inIdx = 0;

  while (inIdx < input.length) {
    const bytesRemaining = input.length - inIdx;

    // First, check if we have at least 4 bytes for a potential passthrough
    if (bytesRemaining >= 4) {
      // HIGHEST PRIORITY: Try 8+ byte passthrough (| escape)
      // This is most efficient for long runs of safe bytes
      const longResult = tryLongPassthrough(input, inIdx, outputChars.length);
      if (longResult !== null) {
        outputChars.push(...longResult.output);
        inIdx += longResult.bytesConsumed;
        continue;
      }

      // Try extended passthrough (5/6/7 bytes) - second preference
      // Prefer: 7-byte > 6-byte > 5-byte
      // These are more efficient than 4-byte passthrough (8 chars for 7 bytes vs 5 chars for 4 bytes)
      // and allow consecutive escapes with zero gap for long safe sequences.
      const extendedResult = tryExtendedPassthrough(input, inIdx);
      if (extendedResult !== null) {
        outputChars.push(...extendedResult.output);
        inIdx += extendedResult.bytesConsumed;
        continue;
      }

      // Check for block-aligned 4-byte passthrough (third preference)
      if (areBytesAllSafe(input, inIdx)) {
        // Block-aligned passthrough: just output , + 4 bytes
        outputChars.push(",");
        outputChars.push(String.fromCharCode(input[inIdx]));
        outputChars.push(String.fromCharCode(input[inIdx + 1]));
        outputChars.push(String.fromCharCode(input[inIdx + 2]));
        outputChars.push(String.fromCharCode(input[inIdx + 3]));
        inIdx += 4;
        continue;
      }

      // Try non-aligned 4-byte passthrough (lowest preference for passthrough)
      // Look for 4 consecutive safe bytes starting at positions 1, 2, or 3
      const nonAlignedResult = tryNonAlignedPassthrough(input, inIdx);
      if (nonAlignedResult !== null) {
        // Non-aligned passthrough succeeded
        outputChars.push(...nonAlignedResult.output);
        inIdx += nonAlignedResult.bytesConsumed;
        continue;
      }

      // No passthrough opportunity, use standard Z85 encoding
      const value =
        ((input[inIdx] << 24) |
          (input[inIdx + 1] << 16) |
          (input[inIdx + 2] << 8) |
          input[inIdx + 3]) >>>
        0;

      // Convert to base-85
      let v = value;
      const digits: string[] = new Array(5);
      for (let i = 4; i >= 0; i--) {
        digits[i] = Z85_ALPHABET[v % 85];
        v = Math.floor(v / 85);
      }
      outputChars.push(...digits);
      inIdx += 4;
    } else {
      // Trailing bytes (1-3): always use standard Z85 encoding
      const numBytes = bytesRemaining;
      const numChars = numBytes + 1;

      let value: number;
      if (numBytes === 1) {
        value = input[inIdx];
      } else if (numBytes === 2) {
        value = (input[inIdx] << 8) | input[inIdx + 1];
      } else {
        value = (input[inIdx] << 16) | (input[inIdx + 1] << 8) | input[inIdx + 2];
      }

      // Encode partial block
      const digits: string[] = new Array(numChars);
      for (let i = numChars - 1; i >= 0; i--) {
        digits[i] = Z85_ALPHABET[value % 85];
        value = Math.floor(value / 85);
      }
      outputChars.push(...digits);
      inIdx += numBytes;
    }
  }

  return outputChars.join("");
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
 * @throws Z855DecodeError on invalid input
 */
export function decode(input: string): Uint8Array {
  // Handle empty input
  if (input.length === 0) {
    return new Uint8Array(0);
  }

  // With non-aligned passthrough, we need to track additional state because
  // passthrough bytes overlap with the before/after blocks.
  const outputChunks: number[] = [];

  let inIdx = 0;

  // Track position within current 5-char Z85 block (0-4)
  let blockPos = 0;
  // Accumulated Z85 digits for the current block
  const currentBlockDigits: number[] = [];
  // Known high bytes for current block (from passthrough of previous block)
  // These are the first P bytes of "after" block when P>0 passthrough was used
  let knownHighBytes: number[] = [];

  while (inIdx < input.length) {
    const charCode = input.charCodeAt(inIdx);

    // Check for long escape (|) first
    if (isLongEscape(charCode)) {
      // The | escape for 8+ bytes
      // Structure: [offset prefix][length prefix][|][padding before][raw bytes][padding after]
      //
      // The prefix digits are in currentBlockDigits (the accumulated Z85 digits)
      // We read them to get offset (if present) and length.

      if (currentBlockDigits.length === 0) {
        // No prefix digits means invalid encoding
        throw new Z855DecodeError("no prefix digits before |");
      }

      // Try to read offset and length from prefix digits
      const { offset, length, offsetDigitsUsed } = readOffsetAndLengthFromPrefix(currentBlockDigits);

      // Handle length semantics
      if (length >= 1 && length <= 7) {
        // Invalid: should use ,;_~ escapes for 4-7 bytes
        throw new Z855DecodeError(`invalid length ${length} for | escape (use ,;_~ for 4-7 bytes)`);
      }

      if (length === 0) {
        // Special case: rest of input is raw
        // Skip the |
        inIdx += 1;
        // Output all remaining bytes as raw
        for (let i = inIdx; i < input.length; i++) {
          outputChunks.push(input.charCodeAt(i));
        }
        // Done decoding
        return new Uint8Array(outputChunks);
      }

      // length >= 8: that many raw bytes follow
      const rawLen = length;

      // Skip the |
      inIdx += 1;

      // Calculate padding positions (position-based, not content-based!)
      const paddingNeeded = calculatePaddingNeeded(rawLen);
      const paddingBefore = offset;
      const paddingAfter = paddingNeeded - offsetDigitsUsed - offset;

      if (paddingAfter < 0) {
        throw new Z855DecodeError(`invalid padding calculation: paddingNeeded=${paddingNeeded}, offsetDigits=${offsetDigitsUsed}, offset=${offset}`);
      }

      // Skip padding before (ANY content - do not check!)
      if (inIdx + paddingBefore > input.length) {
        throw new Z855DecodeError("insufficient input for padding before");
      }
      inIdx += paddingBefore;

      // Ensure we have enough input for the raw bytes
      if (inIdx + rawLen > input.length) {
        throw new Z855DecodeError(`insufficient bytes for | escape: need ${rawLen}, have ${input.length - inIdx}`);
      }

      // Output the raw bytes
      for (let i = 0; i < rawLen; i++) {
        outputChunks.push(input.charCodeAt(inIdx + i));
      }
      inIdx += rawLen;

      // Skip padding after (ANY content - do not check!)
      if (inIdx + paddingAfter > input.length) {
        throw new Z855DecodeError("insufficient input for padding after");
      }
      inIdx += paddingAfter;

      // Reset block state
      currentBlockDigits.length = 0;
      blockPos = 0;
      knownHighBytes = [];
      continue;
    }

    const passLen = getPassthroughLength(charCode);

    if (passLen > 0) {
      // Found an escape character - this is a passthrough marker
      // passLen is 4, 5, 6, or 7

      // Ensure we have enough characters for the passthrough bytes
      if (inIdx + passLen >= input.length) {
        throw new Z855DecodeError("incomplete passthrough sequence");
      }

      // Extract the passthrough bytes
      const passBytes: number[] = [];
      for (let i = 1; i <= passLen; i++) {
        passBytes.push(input.charCodeAt(inIdx + i));
      }

      if (passLen === 4) {
        // 4-byte passthrough (`,`)
        // Structure: [P chars] [,] [4 bytes] [(5-P) chars]
        const P = blockPos; // Position within the 5-char block (0-4)

        if (P === 0) {
          // Block-aligned passthrough: simple case, just output the 4 bytes
          outputChunks.push(...passBytes);
          inIdx += 5; // Skip comma + 4 bytes
          // blockPos stays at 0, currentBlockDigits stays empty, knownHighBytes stays empty
        } else {
          // Non-aligned 4-byte passthrough at position P (1-4)
          //
          // Structure:
          // - We have P high-order Z85 digits for "before" block
          // - passBytes[0..4-P] are the known low bytes of "before" block
          // - passBytes[4-P..4] are the known high bytes of "after" block
          //
          // The passthrough bytes OVERLAP with both blocks!

          const numKnownLowBytes = 4 - P;
          const knownLowBytes = passBytes.slice(0, numKnownLowBytes);

          // Compute the canonical minimum for the "before" block
          const beforeValue = computeCanonicalMinimum(currentBlockDigits, knownLowBytes);

          // Output the 4 bytes of the "before" block
          outputChunks.push((beforeValue >>> 24) & 0xff);
          outputChunks.push((beforeValue >>> 16) & 0xff);
          outputChunks.push((beforeValue >>> 8) & 0xff);
          outputChunks.push(beforeValue & 0xff);

          // DO NOT output passthrough bytes separately - they overlap with before/after blocks!
          // Instead, set the known high bytes for the "after" block
          knownHighBytes = passBytes.slice(numKnownLowBytes); // Last P bytes

          // Reset block state for "after" block
          currentBlockDigits.length = 0;
          blockPos = 0;

          // Move past comma + 4 passthrough bytes
          inIdx += 5;
        }
      } else {
        // 5/6/7-byte passthrough (`;`, `_`, `~`)
        // Structure: [(P+1) chars] [escape] [K bytes] [(5-P) chars]
        //
        // KEY DIFFERENCE from 4-byte: The escape appears at position P+1 (not P).
        // This means block_pos = P+1, so P = block_pos - 1.
        // The extra char fully disambiguates the before block.

        const numDigits = currentBlockDigits.length;

        if (numDigits === 0) {
          // Block-aligned extended passthrough: no preceding Z85 digits
          // This is the "zero gap" case for consecutive escapes
          // Just output the K passthrough bytes directly
          outputChunks.push(...passBytes);
          inIdx += 1 + passLen; // Skip escape + K bytes
          // blockPos stays at 0, currentBlockDigits stays empty
          continue;
        }

        const p = numDigits - 1; // P = (P+1) - 1

        // For 5/6/7-byte passthrough:
        // - First (4-P) passthrough bytes overlap with before block's low bytes
        // - We output only the HIGH P bytes of before block (not in passthrough)
        // - Then we output ALL passthrough bytes directly
        // - After portion continues as normal Z85

        const numKnownLowBytes = 4 - p;
        const knownLowBytes = passBytes.slice(0, numKnownLowBytes);

        // Compute the before block value from (P+1) Z85 digits + (4-P) known low bytes
        const beforeValue = computeBeforeBlockFromExtendedDigits(currentBlockDigits, knownLowBytes);

        // Output only the HIGH P bytes of the before block (the ones NOT in passthrough)
        const beforeBytes = [
          (beforeValue >>> 24) & 0xff,
          (beforeValue >>> 16) & 0xff,
          (beforeValue >>> 8) & 0xff,
          beforeValue & 0xff,
        ];
        for (let i = 0; i < p; i++) {
          outputChunks.push(beforeBytes[i]);
        }

        // Output ALL the passthrough bytes directly
        outputChunks.push(...passBytes);

        // Reset block state for after portion
        currentBlockDigits.length = 0;
        blockPos = 0;
        // No known_high_bytes for 5/6/7-byte passthrough
        knownHighBytes = [];

        // Move past escape + K passthrough bytes
        inIdx += 1 + passLen;
      }
    } else {
      // Regular Z85 character
      const digit = Z85_DECODE_TABLE[charCode];
      if (digit === -1) {
        throw new Z855DecodeError(
          `invalid character in Z85 input: 0x${charCode.toString(16).padStart(2, "0").toUpperCase()}`
        );
      }

      currentBlockDigits.push(digit);
      blockPos++;
      inIdx++;

      // Check if we have enough digits to complete the current block
      // Normal case: 5 digits
      // After non-aligned passthrough: (5-P) digits where P = knownHighBytes.length
      const neededDigits = 5 - knownHighBytes.length;

      if (currentBlockDigits.length === neededDigits) {
        let value: number;

        if (knownHighBytes.length === 0) {
          // Normal case: decode full 5-digit block
          value = 0;
          for (const d of currentBlockDigits) {
            value = value * 85 + d;
          }
        } else {
          // After non-aligned passthrough: reconstruct block from known high bytes + low digits
          // lowDigitsValue = accumulated Z85 digits
          let lowDigitsValue = 0;
          for (const d of currentBlockDigits) {
            lowDigitsValue = lowDigitsValue * 85 + d;
          }

          // Reconstruct the full value
          // The high P bytes are known, the low digits give us a modular constraint
          value = reconstructAfterBlockValue(knownHighBytes, lowDigitsValue, neededDigits);

          // Clear knownHighBytes for next block
          knownHighBytes = [];
        }

        // Check for overflow
        if (value > 0xffffffff) {
          throw new Z855DecodeError("Z85 value overflow");
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
      throw new Z855DecodeError("invalid Z85 input length");
    }

    // Decode partial block: 2 chars -> 1 byte, 3 chars -> 2 bytes, 4 chars -> 3 bytes
    let value = 0;
    for (const d of currentBlockDigits) {
      value = value * 85 + d;
    }

    const numBytes = numChars - 1;

    // Check overflow based on expected byte count
    if (numBytes === 1 && value > 0xff) {
      throw new Z855DecodeError("Z85 value overflow");
    } else if (numBytes === 2 && value > 0xffff) {
      throw new Z855DecodeError("Z85 value overflow");
    } else if (numBytes === 3 && value > 0xffffff) {
      throw new Z855DecodeError("Z85 value overflow");
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
 * Throws Z855DecodeError if any character is invalid or if the value overflows u32.
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
      throw new Z855DecodeError(
        `invalid character in Z85 input: 0x${charCode.toString(16).padStart(2, "0").toUpperCase()}`
      );
    }

    value = value * 85 + digit;
  }

  // Check for overflow (max valid Z85 5-char value is 85^5 - 1 = 4,437,053,124)
  // But we need it to fit in u32 (max 4,294,967,295 = 0xFFFFFFFF)
  if (value > 0xffffffff) {
    throw new Z855DecodeError("Z85 value overflow");
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
      throw new Z855DecodeError(
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
 * Compute the "before" block value from extended Z85 digits (P+1 digits) and known low bytes.
 *
 * For 5/6/7-byte passthrough, we have P+1 Z85 digits (one more than 4-byte passthrough).
 * Combined with the (4-P) known low bytes from the passthrough, this fully determines
 * the before block value - NO ambiguity, NO canonical minimum needed.
 *
 * @param highDigits - Array of (P+1) Z85 digit values (0-84)
 * @param knownLowBytes - Array of (4-P) known low-order bytes from passthrough
 * @returns The 32-bit before block value
 * @throws Z855DecodeError if no valid value exists
 */
function computeBeforeBlockFromExtendedDigits(
  highDigits: number[],
  knownLowBytes: number[]
): number {
  const numDigits = highDigits.length; // This is P+1
  const p = numDigits - 1;
  const numKnownBytes = knownLowBytes.length; // This should be 4-P

  // Compute the base value from high Z85 digits
  // These numDigits define a range [base * 85^(5-numDigits), (base+1) * 85^(5-numDigits))
  let base = 0;
  for (let i = 0; i < numDigits; i++) {
    base = base * 85 + highDigits[i];
  }

  // The range is [base * 85^(5-numDigits), (base+1) * 85^(5-numDigits))
  // With numDigits = P+1, that's [base * 85^(4-P), (base+1) * 85^(4-P))
  const power = Math.pow(85, 5 - numDigits);
  const rangeStart = base * power;
  const rangeEnd = (base + 1) * power;

  if (numKnownBytes === 0) {
    // P = 4, numDigits = 5: we have a full Z85 block, no additional constraint
    if (rangeStart > 0xffffffff) {
      throw new Z855DecodeError("Z85 value overflow in extended passthrough decode");
    }
    return rangeStart;
  }

  // Construct the constraint from known low bytes
  let knownPart = 0;
  for (let i = 0; i < numKnownBytes; i++) {
    knownPart = ((knownPart << 8) | knownLowBytes[i]) >>> 0;
  }

  // When numKnownBytes == 4, all 4 bytes are known, so there's only one possible value.
  // We just need to check if knownPart is in the range.
  if (numKnownBytes === 4) {
    if (knownPart >= rangeStart && knownPart < rangeEnd) {
      return knownPart;
    } else {
      throw new Z855DecodeError("no valid value for extended passthrough decode");
    }
  }

  // The mask for known bytes (low numKnownBytes bytes)
  // Use Math.pow to avoid JavaScript's 32-bit shift limitation
  const modulus = Math.pow(2, numKnownBytes * 8);

  // Find the unique value in [rangeStart, rangeEnd) where (value % modulus) === knownPart
  const startRemainder = rangeStart % modulus;

  let candidate: number;
  if (startRemainder <= knownPart) {
    candidate = rangeStart - startRemainder + knownPart;
  } else {
    candidate = rangeStart - startRemainder + modulus + knownPart;
  }

  // With P+1 digits, the range size is small enough that at most one value matches.
  // Verify the candidate is in range.
  if (candidate >= rangeEnd) {
    throw new Z855DecodeError("no valid value for extended passthrough decode");
  }
  if (candidate > 0xffffffff) {
    throw new Z855DecodeError("Z85 value overflow in extended passthrough decode");
  }

  return candidate;
}

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
 * @throws Z855DecodeError if no valid value exists (should not happen with valid input)
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
      throw new Z855DecodeError("Z85 value overflow in non-aligned decode");
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
    throw new Z855DecodeError("no valid value for non-aligned passthrough decode");
  }
  if (candidate > 0xffffffff) {
    throw new Z855DecodeError("Z85 value overflow in non-aligned decode");
  }

  return candidate;
}

// =============================================================================
// Non-Aligned Passthrough Encoding Support
// =============================================================================
//
// For encoding, we need to check if a block's actual value equals the canonical
// minimum for the partial Z85 + known bytes. If so, we can use non-aligned passthrough.

/**
 * Get the P high-order Z85 digits for a 32-bit value.
 *
 * The Z85 encoding of a 32-bit value produces 5 digits. This returns the first P digits.
 *
 * @param value - The 32-bit value
 * @param P - Number of high-order digits to return (1-4)
 * @returns Array of P Z85 digit values (0-84)
 */
function getHighOrderZ85Digits(value: number, P: number): number[] {
  // Full Z85 encoding: value = d0*85^4 + d1*85^3 + d2*85^2 + d3*85 + d4
  // We need d0, d1, ..., d(P-1)

  // First, convert to all 5 digits
  const digits: number[] = [];
  let v = value;
  for (let i = 0; i < 5; i++) {
    digits.unshift(v % 85);
    v = Math.floor(v / 85);
  }

  // Return first P digits
  return digits.slice(0, P);
}

/**
 * Check if a block value is the canonical minimum for given partial Z85 encoding.
 *
 * For non-aligned passthrough at position P, the encoder outputs P Z85 digits,
 * then comma + 4 passthrough bytes. For this to round-trip correctly, the
 * block's actual value must equal the canonical minimum that the decoder
 * would compute.
 *
 * @param blockValue - The actual 32-bit value of the block
 * @param P - Position of comma (1-4), meaning P high-order Z85 digits are used
 * @param knownLowBytes - The (4-P) known low-order bytes (from passthrough)
 * @returns true if blockValue equals the canonical minimum
 */
function isCanonicalMinimum(
  blockValue: number,
  P: number,
  knownLowBytes: number[]
): boolean {
  // Get the P high-order Z85 digits of the block value
  const highDigits = getHighOrderZ85Digits(blockValue, P);

  // Compute what the canonical minimum would be
  const canonicalMin = computeCanonicalMinimumForEncoding(highDigits, knownLowBytes);

  return blockValue === canonicalMin;
}

/**
 * Compute canonical minimum for encoding (doesn't throw, returns -1 on error).
 */
function computeCanonicalMinimumForEncoding(
  highDigits: number[],
  knownLowBytes: number[]
): number {
  const P = highDigits.length;
  const numKnownBytes = knownLowBytes.length;

  let base = 0;
  for (let i = 0; i < P; i++) {
    base = base * 85 + highDigits[i];
  }

  const power = Math.pow(85, 5 - P);
  const rangeStart = base * power;
  const rangeEnd = (base + 1) * power;

  if (numKnownBytes === 0) {
    if (rangeStart > 0xffffffff) return -1;
    return rangeStart;
  }

  let knownPart = 0;
  for (let i = 0; i < numKnownBytes; i++) {
    knownPart = (knownPart << 8) | knownLowBytes[i];
  }

  const modulus = 1 << (numKnownBytes * 8);
  const startRemainder = rangeStart % modulus;

  let candidate: number;
  if (startRemainder <= knownPart) {
    candidate = rangeStart - startRemainder + knownPart;
  } else {
    candidate = rangeStart - startRemainder + modulus + knownPart;
  }

  if (candidate >= rangeEnd || candidate > 0xffffffff) {
    return -1;
  }

  return candidate;
}

/**
 * Reconstruct the "after" block value from known high bytes and low Z85 digits.
 *
 * After a non-aligned passthrough at position P, the "after" block has:
 * - P known high bytes from the passthrough
 * - (5-P) low-order Z85 digits from the input
 *
 * This function finds the 32-bit value V such that:
 * - V's high P bytes equal knownHighBytes
 * - V mod 85^numDigits equals lowDigitsValue
 *
 * @param knownHighBytes - The P known high bytes
 * @param lowDigitsValue - The accumulated value from (5-P) low Z85 digits
 * @param numDigits - Number of low digits (5-P)
 * @returns The reconstructed 32-bit value
 */
function reconstructAfterBlockValue(
  knownHighBytes: number[],
  lowDigitsValue: number,
  numDigits: number
): number {
  const P = knownHighBytes.length;

  // Compute the known high value (big-endian)
  let knownHigh = 0;
  for (const b of knownHighBytes) {
    knownHigh = (knownHigh << 8) | b;
  }

  // The full value V must satisfy:
  // - (V >>> (8 * (4-P))) === knownHigh (high bytes match)
  // - V % 85^numDigits === lowDigitsValue (low digits match)

  // Since we know the high P bytes, V is in range:
  // [knownHigh * 2^(8*(4-P)), (knownHigh+1) * 2^(8*(4-P)))

  const shift = 8 * (4 - P);
  const rangeStart = knownHigh << shift;
  const rangeSize = 1 << shift; // 2^shift

  const modulus = Math.pow(85, numDigits);

  // Find smallest V >= rangeStart where V % modulus === lowDigitsValue
  const startRemainder = rangeStart % modulus;

  let candidate: number;
  if (startRemainder <= lowDigitsValue) {
    candidate = rangeStart - startRemainder + lowDigitsValue;
  } else {
    candidate = rangeStart - startRemainder + modulus + lowDigitsValue;
  }

  // Verify candidate is in range
  if (candidate >= rangeStart + rangeSize) {
    // This shouldn't happen with valid input, but fall back to rangeStart
    // (This could indicate malformed input)
    throw new Z855DecodeError("invalid after-block reconstruction");
  }

  return candidate >>> 0; // Ensure unsigned
}

/**
 * Check if 4 consecutive bytes are safe for passthrough.
 */
function areBytesAllSafe(input: Uint8Array, startIdx: number): boolean {
  return (
    SAFE_CHAR_TABLE[input[startIdx]] &&
    SAFE_CHAR_TABLE[input[startIdx + 1]] &&
    SAFE_CHAR_TABLE[input[startIdx + 2]] &&
    SAFE_CHAR_TABLE[input[startIdx + 3]]
  );
}

/**
 * Check if K consecutive bytes starting at the given index are all safe.
 */
function areKBytesSafe(input: Uint8Array, startIdx: number, k: number): boolean {
  if (startIdx + k > input.length) {
    return false;
  }
  for (let i = 0; i < k; i++) {
    if (!SAFE_CHAR_TABLE[input[startIdx + i]]) {
      return false;
    }
  }
  return true;
}

// =============================================================================
// Extended Passthrough Encoding (5/6/7 bytes)
// =============================================================================
//
// Extended passthrough (`;`, `_`, `~`) allows encoding 5, 6, or 7 consecutive
// safe bytes. Unlike 4-byte passthrough, these use P+1 chars before the escape
// (not P), which provides full disambiguation without canonical minimum constraint.
//
// Structure: [(P+1) Z85 chars] [escape] [K raw bytes]
// Then the main loop handles the remaining input normally.
//
// Preferences: 8+ byte (|) > 7-byte (~) > 6-byte (_) > 5-byte (;) > 4-byte (,)

// =============================================================================
// Long Passthrough Encoding (8+ bytes with | escape)
// =============================================================================
//
// Long passthrough allows encoding 8 or more consecutive safe bytes using the
// `|` escape character with a variable-length prefix.
//
// Structure: [prefix digits][|][raw bytes][padding]
//
// The prefix encodes the raw byte count using base-42 with continuation bits.
// Special case: 0| means "rest of input is raw" (can be shorter than standard Z85).

/**
 * Result of a successful long passthrough encoding attempt.
 */
interface LongPassthroughResult {
  /** The output characters for this passthrough sequence */
  output: string[];
  /** Number of input bytes consumed */
  bytesConsumed: number;
}

/**
 * Try to encode 8+ consecutive safe bytes using the `|` escape.
 *
 * This has highest priority among passthrough escapes because it's most efficient
 * for long runs of safe bytes.
 *
 * Two cases:
 * 1. At end of input: use `0|` (rest is raw) - shorter output
 * 2. Otherwise: use length-prefixed escape with optional offset for alignment
 *
 * When paddingNeeded >= 2, the encoder can choose where to position the raw data
 * within the padding space. An offset prefix is added to indicate how many padding
 * bytes precede the raw data.
 */
function tryLongPassthrough(
  input: Uint8Array,
  startIdx: number,
  currentOutputLen: number
): LongPassthroughResult | null {
  const bytesRemaining = input.length - startIdx;

  // Need at least 8 safe bytes for this escape
  if (bytesRemaining < 8) {
    return null;
  }

  // Count consecutive safe bytes starting at startIdx
  let safeCount = 0;
  for (let i = startIdx; i < input.length; i++) {
    if (SAFE_CHAR_TABLE[input[i]]) {
      safeCount++;
      // Cap at implementation limit
      if (safeCount >= MAX_LONG_PASSTHROUGH_LENGTH) {
        break;
      }
    } else {
      break;
    }
  }

  // Need at least 8 consecutive safe bytes
  if (safeCount < 8) {
    return null;
  }

  // Check if this safe run extends to end of input
  const atEndOfInput = startIdx + safeCount === input.length;

  if (atEndOfInput) {
    // Use 0| (rest of input is raw) - shorter output
    const output: string[] = [];
    output.push(Z85_ALPHABET[0]); // '0' prefix
    output.push(String.fromCharCode(RAW_ESCAPE_LONG)); // '|'
    for (let i = startIdx; i < input.length; i++) {
      output.push(String.fromCharCode(input[i]));
    }
    return {
      output,
      bytesConsumed: safeCount,
    };
  }

  // Not at end: use length-prefixed escape
  // Structure: [offset prefix][length prefix][|][padding before][raw bytes][padding after]
  //
  // We need to calculate padding to maintain length invariant.
  //
  // IMPORTANT: Z85 output length is NOT additive!
  // z855OutputLength(a + b) != z855OutputLength(a) + z855OutputLength(b) in general.
  //
  // We must ensure: escape_chars + z855OutputLength(remaining) <= z855OutputLength(total)
  // where total = bytesRemaining and remaining = bytesRemaining - rawLen.

  const rawLen = Math.min(safeCount, MAX_LONG_PASSTHROUGH_LENGTH);
  const lengthPrefix = generateLongEscapePrefix(rawLen);

  // Calculate the budget available for the escape sequence
  // Total standard Z85 length for all remaining bytes
  const totalStandardLen = z855OutputLength(bytesRemaining);
  // Standard Z85 length for bytes after the passthrough
  const afterLen = z855OutputLength(bytesRemaining - rawLen);
  // Available chars for our escape (must not exceed this to maintain invariant)
  const availableChars = totalStandardLen - afterLen;

  // Our encoding (without padding, without offset): lengthPrefix.length + 1 (|) + rawLen
  const ourLenNoPadding = lengthPrefix.length + 1 + rawLen;

  // If our escape is already too long, don't use it
  if (ourLenNoPadding > availableChars) {
    return null;
  }

  // Padding needed to reach the available budget (or 0 if exact fit)
  const paddingNeeded = availableChars - ourLenNoPadding;

  // When paddingNeeded >= 2, we can choose an offset for alignment
  // When paddingNeeded < 2, offset is implicitly 0 and not encoded
  // When bestOffset == 0, we also don't encode it (for backward compatibility)
  let offset: number;
  let offsetPrefix: string[];
  if (paddingNeeded >= 2) {
    // Find the best offset using bit-reversal sort key
    const bestOffset = findBestOffset(
      currentOutputLen,
      lengthPrefix.length,
      rawLen,
      paddingNeeded
    );
    // Only include offset prefix if offset > 0
    if (bestOffset > 0) {
      offsetPrefix = generateLongEscapePrefix(bestOffset);
      offset = bestOffset;
    } else {
      offsetPrefix = [];
      offset = 0;
    }
  } else {
    offsetPrefix = [];
    offset = 0;
  }

  // If we're encoding an offset, we need space for it in the padding
  // The offset prefix chars come from the padding budget
  if (offsetPrefix.length > paddingNeeded) {
    return null;
  }

  const output: string[] = [];

  // Output offset prefix (if any), then length prefix
  output.push(...offsetPrefix);
  output.push(...lengthPrefix);
  output.push(String.fromCharCode(RAW_ESCAPE_LONG));

  // Output padding before raw bytes (offset dots)
  for (let i = 0; i < offset; i++) {
    output.push(String.fromCharCode(RAW_ESCAPE_PADDING));
  }

  // Output raw bytes
  for (let i = 0; i < rawLen; i++) {
    output.push(String.fromCharCode(input[startIdx + i]));
  }

  // Calculate remaining padding after raw bytes
  // Total padding space = paddingNeeded - offsetPrefix.length (offset prefix chars)
  // We've used 'offset' chars as dots before raw bytes
  // Remaining = (paddingNeeded - offsetPrefix.length) - offset
  const paddingAfter = paddingNeeded - offsetPrefix.length - offset;

  // Add remaining padding: all dots, no final |
  for (let i = 0; i < paddingAfter; i++) {
    output.push(String.fromCharCode(RAW_ESCAPE_PADDING));
  }

  return {
    output,
    bytesConsumed: rawLen,
  };
}

/**
 * Find the best offset for padding alignment using bit-reversal sort key.
 *
 * The sort key is a 4-tuple:
 * (min(bitReverse(inputStart), bitReverse(inputEnd)),
 *  max(bitReverse(inputStart), bitReverse(inputEnd)),
 *  min(bitReverse(outputStart), bitReverse(outputEnd)),
 *  max(bitReverse(outputStart), bitReverse(outputEnd)))
 *
 * We pick the offset with the lexicographically smallest key.
 */
function findBestOffset(
  currentOutputLen: number,
  lengthPrefixLen: number,
  rawLen: number,
  paddingNeeded: number
): number {
  // Generate candidate offsets: 0 to maxOffset
  // The offset prefix takes space from the padding budget, so we need to account for that
  // We iterate over possible offsets and compute valid ones
  //
  // Note: offset=0 means no offset prefix is encoded (for backward compatibility)
  // offset>0 requires encoding the offset prefix, which takes space

  let bestOffset = 0;
  let bestKey: [bigint, bigint, bigint, bigint] | null = null;

  for (let offset = 0; offset <= paddingNeeded; offset++) {
    // When offset=0, no offset prefix is encoded
    // When offset>0, we need to encode the offset value
    const offsetPrefixLen = offset > 0 ? generateLongEscapePrefix(offset).length : 0;

    // Check if this offset is valid (fits in padding budget)
    // We need: offsetPrefixLen + offset (dots before) + paddingAfter (dots after) <= paddingNeeded
    // Actually: offsetPrefixLen + offset + paddingAfter = paddingNeeded
    // And paddingAfter must be >= 0
    if (offsetPrefixLen + offset > paddingNeeded) {
      continue;
    }

    // Compute output positions
    // Output structure: [offset_prefix][length_prefix][|][offset dots][raw bytes][remaining dots][|]
    const outputStart = currentOutputLen + offsetPrefixLen + lengthPrefixLen + 1 + offset;
    const outputEnd = outputStart + rawLen - 1;

    // Map output positions to input positions
    // inPos = floor(outPos * 4 / 5)
    const inputStart = Math.floor(outputStart * 4 / 5);
    const inputEnd = Math.floor(outputEnd * 4 / 5);

    // Compute bit-reversed values
    const revInStart = bitReverse(inputStart);
    const revInEnd = bitReverse(inputEnd);
    const revOutStart = bitReverse(outputStart);
    const revOutEnd = bitReverse(outputEnd);

    // Build sort key
    const key: [bigint, bigint, bigint, bigint] = [
      revInStart < revInEnd ? revInStart : revInEnd,
      revInStart > revInEnd ? revInStart : revInEnd,
      revOutStart < revOutEnd ? revOutStart : revOutEnd,
      revOutStart > revOutEnd ? revOutStart : revOutEnd,
    ];

    // Update best if this is better (or first candidate)
    if (bestKey === null || compareTuples(key, bestKey) < 0) {
      bestKey = key;
      bestOffset = offset;
    }
  }

  return bestOffset;
}

/**
 * Compare two 4-tuples lexicographically.
 * Returns negative if a < b, 0 if a == b, positive if a > b.
 */
function compareTuples(a: [bigint, bigint, bigint, bigint], b: [bigint, bigint, bigint, bigint]): number {
  for (let i = 0; i < 4; i++) {
    if (a[i] < b[i]) return -1;
    if (a[i] > b[i]) return 1;
  }
  return 0;
}

// =============================================================================
// Extended Passthrough Encoding (5/6/7 bytes)
// =============================================================================

/**
 * Result of a successful extended passthrough encoding attempt.
 */
interface ExtendedPassthroughResult {
  /** The output characters for this passthrough sequence */
  output: string[];
  /** Number of input bytes consumed */
  bytesConsumed: number;
}

/**
 * Try to find and encode an extended (5/6/7-byte) passthrough.
 *
 * Looks for K consecutive safe bytes (K = 5, 6, or 7) starting at various positions.
 * Returns the best option found, preferring longer passthrough (7 > 6 > 5).
 *
 * Two cases are handled:
 * 1. Block-aligned: [escape] [K raw bytes] - no preceding Z85 chars
 *    This enables consecutive escapes with zero gap (e.g., ~XXXXXXX~YYYYYYY)
 * 2. Non-aligned: [(P+1) Z85 chars] [escape] [K raw bytes]
 *    where P is the position within the input block (1-3).
 */
function tryExtendedPassthrough(
  input: Uint8Array,
  blockStart: number
): ExtendedPassthroughResult | null {
  // Try 7-byte first (highest preference), then 6, then 5
  for (const k of [7, 6, 5]) {
    // First try block-aligned extended passthrough (escape at position 0, no preceding chars)
    // This enables consecutive escapes with zero gap
    const blockAlignedResult = tryBlockAlignedExtendedPassthrough(input, blockStart, k);
    if (blockAlignedResult !== null) {
      return blockAlignedResult;
    }

    // Then try non-aligned positions
    const result = tryExtendedPassthroughOfLength(input, blockStart, k);
    if (result !== null) {
      return result;
    }
  }
  return null;
}

/**
 * Try block-aligned extended passthrough of length K.
 *
 * This outputs just [escape] [K raw bytes] with NO preceding Z85 chars.
 * This is possible when the K safe bytes start exactly at blockStart.
 */
function tryBlockAlignedExtendedPassthrough(
  input: Uint8Array,
  blockStart: number,
  k: number
): ExtendedPassthroughResult | null {
  // Check if we have enough bytes for the passthrough
  if (blockStart + k > input.length) {
    return null;
  }

  // Check the length invariant: passthrough_output + z855(remaining) == z855(total)
  // Block-aligned output is 1 (escape) + K (raw) = K+1 chars.
  const totalRemaining = input.length - blockStart;
  const remaining = totalRemaining - k;
  const passthroughOutputChars = k + 1;
  if (passthroughOutputChars + z855OutputLength(remaining) !== z855OutputLength(totalRemaining)) {
    return null;
  }

  // Check if all K bytes starting at blockStart are safe
  if (!areKBytesSafe(input, blockStart, k)) {
    return null;
  }

  // Build output: just escape + K raw bytes
  const output: string[] = [];

  let escape: string;
  switch (k) {
    case 5:
      escape = String.fromCharCode(RAW_ESCAPE_5);
      break;
    case 6:
      escape = String.fromCharCode(RAW_ESCAPE_6);
      break;
    case 7:
      escape = String.fromCharCode(RAW_ESCAPE_7);
      break;
    default:
      throw new Error("unreachable");
  }
  output.push(escape);

  // Add K passthrough bytes
  for (let i = 0; i < k; i++) {
    output.push(String.fromCharCode(input[blockStart + i]));
  }

  return {
    output,
    bytesConsumed: k,
  };
}

/**
 * Try extended passthrough of a specific length K (5, 6, or 7).
 */
function tryExtendedPassthroughOfLength(
  input: Uint8Array,
  blockStart: number,
  k: number
): ExtendedPassthroughResult | null {
  const totalRemaining = input.length - blockStart;

  // Generate all valid positions and compute sort keys using bit reversal.
  // Positions aligned to power-of-2 boundaries have trailing zeros.
  // Bit reversal turns trailing zeros into leading zeros, so aligned
  // positions sort first naturally.
  //
  // Sort key = (min(rev_start, rev_end), max(rev_start, rev_end))
  // where start = blockStart + p, end = start + k - 1
  const candidates: Array<{ sortKey0: bigint; sortKey1: bigint; p: number }> = [];

  for (let p = 0; p <= 3; p++) {
    const bytesConsumed = p + k;
    if (blockStart + bytesConsumed > input.length) {
      continue; // Not enough input
    }
    const remaining = totalRemaining - bytesConsumed;
    const passthroughOutputChars = bytesConsumed + 2; // (P+1) + 1 + K = P+K+2
    if (passthroughOutputChars + z855OutputLength(remaining) !== z855OutputLength(totalRemaining)) {
      continue; // Would violate length invariant
    }

    // Compute sort key using bit reversal
    const start = blockStart + p;
    const end = start + k - 1;
    const revStart = bitReverse(start);
    const revEnd = bitReverse(end);
    const sortKey0 = revStart < revEnd ? revStart : revEnd;
    const sortKey1 = revStart < revEnd ? revEnd : revStart;
    candidates.push({ sortKey0, sortKey1, p });
  }

  // Sort by sort key (lower is better)
  candidates.sort((a, b) => {
    if (a.sortKey0 < b.sortKey0) return -1;
    if (a.sortKey0 > b.sortKey0) return 1;
    if (a.sortKey1 < b.sortKey1) return -1;
    if (a.sortKey1 > b.sortKey1) return 1;
    return 0;
  });

  // Try candidates in sorted order
  for (const { p } of candidates) {
    const result = tryExtendedPassthroughAtPosition(input, blockStart, k, p);
    if (result !== null) {
      return result;
    }
  }
  return null;
}

/**
 * Try extended passthrough of length K at a specific position P.
 */
function tryExtendedPassthroughAtPosition(
  input: Uint8Array,
  blockStart: number,
  k: number,
  p: number
): ExtendedPassthroughResult | null {
  // Passthrough bytes start at blockStart + p and span K bytes
  const passStart = blockStart + p;

  // Check if we have enough input for the passthrough bytes
  if (passStart + k > input.length) {
    return null;
  }

  // Check if all K passthrough bytes are safe
  if (!areKBytesSafe(input, passStart, k)) {
    return null;
  }

  // Compute the before block (first 4 bytes of current block)
  const beforeValue =
    ((input[blockStart] << 24) |
      (input[blockStart + 1] << 16) |
      (input[blockStart + 2] << 8) |
      input[blockStart + 3]) >>>
    0;

  // For extended passthrough (5/6/7 bytes), we output:
  // 1. (P+1) Z85 chars - partial encoding of before block
  // 2. Escape character
  // 3. K passthrough bytes
  //
  // The remaining input after the passthrough is handled by the main encode loop.
  // Unlike 4-byte passthrough, there's NO canonical minimum constraint because
  // the extra char (P+1 instead of P) provides full disambiguation.

  const output: string[] = [];

  // 1. (P+1) Z85 chars for before block (partial encoding)
  const beforeChars = getHighOrderZ85Chars(beforeValue, p + 1);
  output.push(...beforeChars);

  // 2. Escape character
  let escape: string;
  switch (k) {
    case 5:
      escape = String.fromCharCode(RAW_ESCAPE_5);
      break;
    case 6:
      escape = String.fromCharCode(RAW_ESCAPE_6);
      break;
    case 7:
      escape = String.fromCharCode(RAW_ESCAPE_7);
      break;
    default:
      throw new Error("unreachable");
  }
  output.push(escape);

  // 3. K passthrough bytes
  for (let i = 0; i < k; i++) {
    output.push(String.fromCharCode(input[passStart + i]));
  }

  // Bytes consumed:
  // - P bytes from before block (input[blockStart..blockStart+P])
  // - K bytes of passthrough (input[blockStart+P..blockStart+P+K])
  // Total: P + K bytes
  const bytesConsumed = p + k;

  return {
    output,
    bytesConsumed,
  };
}

// =============================================================================
// Non-Aligned 4-byte Passthrough Encoding
// =============================================================================
//
// Non-aligned passthrough allows encoding 4 consecutive safe bytes that don't
// align with block boundaries. The `,` marker appears at position P (1-4) within
// a 5-character output block.
//
// For this to work correctly:
// 1. The "before" block value must be the canonical minimum
// 2. The "after" block must have enough remaining input to complete
// 3. We try both position P and P+1 to maximize success rate

/**
 * Result of a successful non-aligned passthrough encoding attempt.
 */
interface NonAlignedResult {
  /** The output characters for this passthrough sequence */
  output: string[];
  /** Number of input bytes consumed */
  bytesConsumed: number;
}

/**
 * Try to find and encode a non-aligned passthrough within the current block.
 *
 * Looks for 4 consecutive safe bytes starting at positions 1, 2, or 3 within
 * the current 4-byte block. For each candidate, checks if the "before" block
 * value is canonical (minimum) and if the "after" block can be properly encoded.
 *
 * Returns the result if successful, null otherwise.
 */
function tryNonAlignedPassthrough(
  input: Uint8Array,
  blockStart: number
): NonAlignedResult | null {
  // Generate candidates and sort by bit-reversal for consistent position preference.
  // This matches the approach used for extended passthrough (5/6/7 bytes).
  const candidates: Array<[bigint, bigint, number]> = [];

  for (let p = 1; p <= 3; p++) {
    const passStart = blockStart + p;
    if (passStart + 4 > input.length) {
      continue;
    }
    // Check if passthrough bytes are safe before adding as candidate
    if (!isBlockSafeForPassthrough(input, passStart)) {
      continue;
    }

    // Compute sort key using bit reversal
    const start = passStart;
    const end = start + 3; // 4 bytes, so end is start + 3
    const revStart = bitReverse(start);
    const revEnd = bitReverse(end);
    const sortKey: [bigint, bigint] = [
      revStart < revEnd ? revStart : revEnd,
      revStart < revEnd ? revEnd : revStart,
    ];
    candidates.push([sortKey[0], sortKey[1], p]);
  }

  // Sort by sort key (lower is better - more aligned positions first)
  candidates.sort((a, b) => {
    if (a[0] !== b[0]) return a[0] < b[0] ? -1 : 1;
    if (a[1] !== b[1]) return a[1] < b[1] ? -1 : 1;
    return 0;
  });

  // Try candidates in sorted order
  for (const [, , p] of candidates) {
    const result = tryNonAlignedAtPosition(input, blockStart, p);
    if (result !== null) {
      return result;
    }
  }
  return null;
}

/**
 * Try non-aligned passthrough at a specific position P within the block.
 *
 * Position P means:
 * - The "before" block has P high-order Z85 chars
 * - The passthrough bytes start at input[blockStart + P]
 * - The passthrough bytes overlap: first (4-P) bytes are end of "before", last P bytes are start of "after"
 */
function tryNonAlignedAtPosition(
  input: Uint8Array,
  blockStart: number,
  p: number
): NonAlignedResult | null {
  // Passthrough bytes start at blockStart + p and span 4 bytes
  const passStart = blockStart + p;

  // Check if we have enough input for the passthrough
  if (passStart + 4 > input.length) {
    return null;
  }

  // Check if the 4 passthrough bytes are all safe
  if (!areBytesAllSafe(input, passStart)) {
    return null;
  }

  // Extract the passthrough bytes
  const passBytes = [
    input[passStart],
    input[passStart + 1],
    input[passStart + 2],
    input[passStart + 3],
  ];

  // The "before" block is the 4 bytes starting at blockStart
  const beforeValue =
    ((input[blockStart] << 24) |
      (input[blockStart + 1] << 16) |
      (input[blockStart + 2] << 8) |
      input[blockStart + 3]) >>>
    0;

  // The known low bytes of the "before" block are the first (4-p) passthrough bytes
  const numKnownLowBytes = 4 - p;
  const knownLowBytes = passBytes.slice(0, numKnownLowBytes);

  // Check if the beforeValue is canonical for the given partial encoding
  if (!isCanonicalMinimumForEncoding(beforeValue, p, knownLowBytes)) {
    return null;
  }

  // The beforeValue is canonical! Now we need to handle the "after" block.
  //
  // The "after" block:
  // - Has P known high bytes from the passthrough (the last P bytes)
  // - Needs (5-P) Z85 chars for the low-order digits
  //
  // We need to check if there's enough input to form a complete after block.

  const knownHighBytes = passBytes.slice(numKnownLowBytes); // Last P bytes of passthrough

  // The after block starts at blockStart + 4
  // Its first P bytes are from the passthrough (knownHighBytes)
  // Its remaining (4-P) bytes come from input starting at blockStart + 4 + P

  const afterBlockDataStart = blockStart + 4;

  // Check if we have enough input for the full after block
  // The after block needs 4 bytes total, and its first P bytes are from passthrough
  // So we need (4-P) more bytes from input starting at afterBlockDataStart + P
  const afterRemainingStart = afterBlockDataStart + p;
  const afterRemainingNeeded = 4 - p;

  if (afterRemainingStart + afterRemainingNeeded > input.length) {
    // Not enough input for a complete after block.
    // Non-aligned passthrough requires a complete after block because:
    // - The passthrough bytes overlap with both before and after blocks
    // - The after block needs (5-P) Z85 chars to encode its low-order bytes
    // - Without a complete after block, we can't properly output those Z85 chars
    //
    // Fall back to standard Z85 encoding for this case.
    return null;
  }

  // We have enough input for a complete after block
  // Construct the full after block value
  const afterBytes = new Array(4);

  // First P bytes come from passthrough (knownHighBytes)
  for (let i = 0; i < p; i++) {
    afterBytes[i] = knownHighBytes[i];
  }

  // Remaining (4-P) bytes come from input
  for (let i = 0; i < afterRemainingNeeded; i++) {
    afterBytes[p + i] = input[afterRemainingStart + i];
  }

  const afterValue =
    ((afterBytes[0] << 24) |
      (afterBytes[1] << 16) |
      (afterBytes[2] << 8) |
      afterBytes[3]) >>>
    0;

  // Now construct the output:
  // 1. P high-order Z85 chars from beforeValue
  // 2. Comma
  // 3. 4 passthrough bytes
  // 4. (5-P) low-order Z85 chars from afterValue

  const output: string[] = [];

  // 1. P high-order Z85 chars
  const highChars = getHighOrderZ85Chars(beforeValue, p);
  output.push(...highChars);

  // 2. Comma
  output.push(",");

  // 3. 4 passthrough bytes
  for (const byte of passBytes) {
    output.push(String.fromCharCode(byte));
  }

  // 4. (5-P) low-order Z85 chars from afterValue
  const lowChars = getLowOrderZ85Chars(afterValue, 5 - p);
  output.push(...lowChars);

  // Bytes consumed: before block (4) + after block (4) = 8
  return {
    output,
    bytesConsumed: 8,
  };
}

/**
 * Get the first P Z85 characters (high-order digits) for a 32-bit value.
 */
function getHighOrderZ85Chars(value: number, p: number): string[] {
  // Full Z85 encoding produces 5 chars
  const chars: string[] = new Array(5);
  let v = value;
  for (let i = 4; i >= 0; i--) {
    chars[i] = Z85_ALPHABET[v % 85];
    v = Math.floor(v / 85);
  }
  // Return first P chars
  return chars.slice(0, p);
}

/**
 * Get the last numChars Z85 characters (low-order digits) for a 32-bit value.
 */
function getLowOrderZ85Chars(value: number, numChars: number): string[] {
  // Full Z85 encoding produces 5 chars
  const chars: string[] = new Array(5);
  let v = value;
  for (let i = 4; i >= 0; i--) {
    chars[i] = Z85_ALPHABET[v % 85];
    v = Math.floor(v / 85);
  }
  // Return last numChars
  return chars.slice(5 - numChars);
}

/**
 * Check if a block value is the canonical minimum for given partial Z85 encoding.
 *
 * For non-aligned passthrough at position P, the encoder outputs P high-order Z85
 * digits. For round-trip correctness, the block's actual value must equal the
 * canonical minimum that the decoder would compute given those P digits and
 * the known low bytes from the passthrough.
 */
function isCanonicalMinimumForEncoding(
  blockValue: number,
  p: number,
  knownLowBytes: number[]
): boolean {
  // Get the P high-order Z85 digits of the block value
  const highDigits = getHighOrderZ85Digits(blockValue, p);

  // Compute what the canonical minimum would be
  const canonicalMin = computeCanonicalMinimumForEncoding(highDigits, knownLowBytes);

  return blockValue === canonicalMin;
}

/**
 * Find the position of `,` in the input string, if any.
 * Returns -1 if no comma found.
 */
function findCommaPosition(input: string, startIdx: number, endIdx: number): number {
  for (let i = startIdx; i < endIdx; i++) {
    if (input.charCodeAt(i) === RAW_ESCAPE_4) {
      return i;
    }
  }
  return -1;
}
