/**
 * z855-readable.ts — A readable reference implementation of the Z855 codec.
 *
 * Z855 extends RFC 32 Z85 (ZeroMQ's binary-to-text encoding) in two ways:
 *
 *   1. **Arbitrary-length input**: Z85 requires input to be a multiple of 4 bytes.
 *      Z855 handles any length by treating the final 1–3 bytes as a partial block
 *      (encoded with one fewer output character than a full block would use).
 *
 *   2. **Raw passthrough for readable bytes**: When a run of input bytes happens to
 *      be printable, Z855 may output them literally instead of Z85-encoding them.
 *      This improves human-readability of encoded data that contains readable text
 *      or identifiers. The passthrough is marked with an escape character so the
 *      decoder knows to copy the following bytes directly without decoding.
 *
 * Z85 quick recap:
 *   - 85-character alphabet: `0123456789abcdefghijklmnopqrstuvwxyz
 *                               ABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#`
 *   - Full blocks: 4 input bytes → 5 output chars (big-endian u32 in base 85)
 *   - Partial blocks: N bytes (1–3) → N+1 output chars
 *
 * This file prioritises clarity over performance. All edge cases are handled by
 * the production implementation in `z855.ts`; the two files share the same format.
 */

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/** The 85-character Z85 alphabet. Index i maps to alphabet[i]. */
const ALPHABET =
  "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";

/** Reverse lookup: char code → Z85 digit value (0–84), or −1 if invalid. */
const CHAR_TO_DIGIT: Int8Array = (() => {
  const table = new Int8Array(256).fill(-1);
  for (let i = 0; i < 85; i++) table[ALPHABET.charCodeAt(i)] = i;
  return table;
})();

/**
 * "Safe" characters: bytes that are allowed through the passthrough extension.
 * This is the Z85 alphabet (85 chars) plus five additional escape-marker characters
 * that are needed by the various passthrough escape forms: `,;|~_`.
 * Total: 90 characters.
 */
const SAFE_CHARS =
  "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#,;|~_";

/** Fast lookup table for safe-character detection. */
const IS_SAFE: Uint8Array = (() => {
  const table = new Uint8Array(256);
  for (let i = 0; i < SAFE_CHARS.length; i++) table[SAFE_CHARS.charCodeAt(i)] = 1;
  return table;
})();

// Escape character codes used by the passthrough extension.
const ESC_4    = 0x2c; // ',' — the next 4 bytes are literal (block-aligned or non-aligned)
const ESC_5    = 0x3b; // ';' — extended: (P+1) Z85 chars, then 5 literal bytes
const ESC_6    = 0x5f; // '_' — extended: (P+1) Z85 chars, then 6 literal bytes
const ESC_7    = 0x7e; // '~' — extended: (P+1) Z85 chars, then 7 literal bytes
const ESC_LONG = 0x7c; // '|' — long:   [prefix][|][raw bytes][padding]
const PAD_DOT  = 0x2e; // '.' — padding byte for the long escape (ignored on decode)
const PAD_HASH = 0x23; // '#' — padding used in concatenatable mode for partial blocks

/** Maximum length for a single long-passthrough segment (matches the Rust impl). */
const MAX_LONG_PASSTHROUGH = 65536;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

export class Z855DecodeError extends Error {
  constructor(msg: string) {
    super(msg);
    this.name = "Z855DecodeError";
  }
}

/**
 * Encode arbitrary bytes to a Z855 string.
 *
 * The output is a printable ASCII string that is slightly longer than the input
 * (at most 25% overhead for pure binary data; often much shorter if the input
 * contains readable text that triggers the passthrough extension).
 */
export function encode(input: Uint8Array): string {
  if (input.length === 0) return "";

  const out: string[] = [];
  let i = 0; // byte index into input

  while (i < input.length) {
    const remaining = input.length - i;

    // -----------------------------------------------------------------------
    // Full-block path (4 or more bytes remaining)
    // -----------------------------------------------------------------------
    if (remaining >= 4) {
      // Try each passthrough form, most-efficient-first.  Fall through to
      // standard Z85 encoding if none applies.

      // (A) Long passthrough: 8+ consecutive safe bytes → [prefix]|[raw bytes]
      const long = tryLongPassthrough(input, i);
      if (long !== null) {
        out.push(...long.chars);
        i += long.consumed;
        continue;
      }

      // (B) Extended passthrough: 5, 6, or 7 safe bytes → [(P+1) Z85][ESC][raw]
      const ext = tryExtendedPassthrough(input, i);
      if (ext !== null) {
        out.push(...ext.chars);
        i += ext.consumed;
        continue;
      }

      // (C) Block-aligned 4-byte passthrough: all 4 bytes are safe → `,XXXX`
      if (allSafe(input, i, 4)) {
        out.push(",");
        for (let k = 0; k < 4; k++) out.push(String.fromCharCode(input[i + k]));
        i += 4;
        continue;
      }

      // (D) Non-aligned 4-byte passthrough: 4 safe bytes starting at offset 1–3
      //     within this block.  Only valid when the surrounding block values
      //     satisfy a canonical-minimum constraint (see encodeNonAligned).
      const nna = tryNonAlignedPassthrough(input, i);
      if (nna !== null) {
        out.push(...nna.chars);
        i += nna.consumed;
        continue;
      }

      // (E) Standard Z85 encoding for a full 4-byte block.
      out.push(...encodeFullBlock(readU32(input, i)));
      i += 4;
      continue;
    }

    // -----------------------------------------------------------------------
    // Partial-block path (1, 2, or 3 bytes remaining) — always Z85, no passthrough
    //
    // A partial block of N bytes is treated as if it were the high-order bytes of
    // a 4-byte value (with the missing low bytes implicitly zero).  We then emit
    // N+1 Z85 characters — one more than the number of bytes, analogous to how a
    // full block always uses 5 characters for 4 bytes.
    //
    // Why N+1 instead of N?  Because N Z85 characters can only distinguish 85^N
    // values, which (for N=1) is only 85 — not enough to represent all 256 possible
    // single-byte values.  N+1 characters give 85^(N+1) possibilities, which is
    // always sufficient.
    //
    // The decoder recognises the partial block by length: any sequence that is not
    // a multiple of 5 characters (and not explained by an escape) must end with a
    // partial block of 2, 3, or 4 trailing characters.  (A single trailing character
    // is always invalid because 85^1 < 256.)
    // -----------------------------------------------------------------------
    const numChars = remaining + 1; // 1 byte → 2 chars, 2 bytes → 3 chars, 3 bytes → 4 chars

    // Pack the N bytes into a number as if they were the high-order bytes of a u32.
    let value = 0;
    for (let k = 0; k < remaining; k++) value = (value * 256 + input[i + k]) | 0;
    // (No left-shift needed: we emit exactly numChars digits, so the base-85 math
    //  works on the value directly.)

    out.push(...encodeValue(value, numChars));
    i += remaining;
  }

  return out.join("");
}

/**
 * Decode a Z855 string back to bytes.
 *
 * @throws Z855DecodeError on any invalid input.
 */
export function decode(input: string): Uint8Array {
  if (input.length === 0) return new Uint8Array(0);

  const out: number[] = [];
  let i = 0; // character index into input

  // Accumulated Z85 digits for the current block.
  let blockDigits: number[] = [];
  // After a non-aligned 4-byte passthrough, the decoder knows the high P bytes
  // of the "after" block from the passthrough itself.  These are held here so
  // that when the remaining (5−P) Z85 digits arrive, we can reconstruct the
  // full 32-bit value.
  let knownHighBytes: number[] = [];

  while (i < input.length) {
    const code = input.charCodeAt(i);

    // ------------------------------------------------------------------
    // Hash padding — used in concatenatable mode; skip it.
    // ------------------------------------------------------------------
    if (code === PAD_HASH && blockDigits.length === 0 && (i % 5) === 0) {
      const partial = decodeHashPaddedBlock(input, i);
      out.push(...partial.bytes);
      i += 5;
      continue;
    }

    // ------------------------------------------------------------------
    // Long escape: `|`
    //
    // Structure: [prefix digits] `|` [optional dot padding] [raw bytes] [optional dot padding]
    //
    // The prefix digits (accumulated in blockDigits) encode the raw-byte count
    // in a self-terminating base-42 scheme.  A prefix of `0` is the special
    // "rest of input is raw" form.
    // ------------------------------------------------------------------
    if (code === ESC_LONG) {
      if (blockDigits.length === 0) throw new Z855DecodeError("'|' without preceding prefix digits");
      const { offset, length } = readLongEscapePrefix(blockDigits);

      if (length >= 1 && length <= 7) {
        throw new Z855DecodeError(`invalid | escape length ${length} (use ,;_~ for 4–7 bytes)`);
      }

      i++; // consume `|`

      if (length === 0) {
        // Special form: copy all remaining input bytes literally.
        for (; i < input.length; i++) out.push(input.charCodeAt(i));
        blockDigits = [];
        knownHighBytes = [];
        break;
      }

      // General form: copy exactly `length` raw bytes (with padding around them).
      const rawLen = length;
      const prefixCharCount = blockDigits.length - (offset > 0 ? generateLongPrefix(offset).length : 0);
      const totalEnvelopeLen = z855OutputLen(rawLen);
      const paddingNeeded = totalEnvelopeLen - (prefixCharCount + 1 + rawLen);
      const paddingAfter = paddingNeeded - (offset > 0 ? generateLongPrefix(offset).length : 0) - offset;

      // Skip `offset` padding bytes before the raw data (content is irrelevant).
      i += offset;
      if (i + rawLen > input.length) throw new Z855DecodeError("truncated | escape");

      for (let k = 0; k < rawLen; k++) out.push(input.charCodeAt(i + k));
      i += rawLen;
      i += paddingAfter; // skip trailing padding

      blockDigits = [];
      knownHighBytes = [];
      continue;
    }

    // ------------------------------------------------------------------
    // Short passthrough escapes: `,` (4 bytes), `;` (5), `_` (6), `~` (7)
    // ------------------------------------------------------------------
    const passLen = shortPassthroughLength(code);
    if (passLen > 0) {
      if (i + passLen >= input.length) throw new Z855DecodeError("incomplete passthrough sequence");
      const passBytes: number[] = [];
      for (let k = 1; k <= passLen; k++) passBytes.push(input.charCodeAt(i + k));

      if (passLen === 4) {
        // `,` escape.
        // Position P = blockDigits.length (how many Z85 chars were before the `,`).
        const P = blockDigits.length;

        if (P === 0) {
          // Block-aligned (P=0): the 4 passthrough bytes are the full output.
          out.push(...passBytes);
          i += 5; // `,` + 4 bytes
        } else {
          // Non-aligned (P ≥ 1): the passthrough bytes OVERLAP the surrounding blocks.
          //
          // The "before" block: the encoder output P Z85 digits, then hid the low
          // (4−P) bytes of that block inside the passthrough.  The decoder picks the
          // *minimum* 32-bit value consistent with those P digits and those (4−P)
          // known bytes (called the "canonical minimum").
          //
          // The "after" block: the last P bytes of the passthrough are the *high*
          // bytes of the next block.  The decoder holds them in `knownHighBytes`
          // and waits for the remaining (5−P) Z85 digits.
          const numKnownLow = 4 - P;
          const knownLow = passBytes.slice(0, numKnownLow);
          const beforeValue = canonicalMinimum(blockDigits, knownLow);
          out.push(...u32ToBytes(beforeValue));

          knownHighBytes = passBytes.slice(numKnownLow); // last P bytes
          blockDigits = [];
          i += 5; // `,` + 4 bytes
        }
      } else {
        // `;` / `_` / `~` escape (5, 6, or 7 bytes).
        // Structure: [(P+1) Z85 chars before escape] [ESC] [K raw bytes]
        //
        // Unlike the `,` case, the extra digit (P+1 instead of P) provides enough
        // information to fully determine the before-block without the canonical-
        // minimum trick.  The decoder computes the before-block value directly,
        // outputs only the *top P* bytes (those not covered by the passthrough),
        // then outputs all K passthrough bytes verbatim.
        const numDigits = blockDigits.length; // = P+1

        if (numDigits === 0) {
          // Block-aligned extended passthrough (P+1 = 0 means no preceding digits);
          // just copy the raw bytes.
          out.push(...passBytes);
          i += 1 + passLen;
          continue;
        }

        const P = numDigits - 1;
        const numKnownLow = 4 - P;
        const knownLow = passBytes.slice(0, numKnownLow);

        // Determine the full before-block value from (P+1) digits + known low bytes.
        const beforeValue = beforeBlockFromExtendedDigits(blockDigits, knownLow);

        // Emit the top P bytes (the rest are covered by the passthrough).
        const beforeBytes = u32ToBytes(beforeValue);
        for (let k = 0; k < P; k++) out.push(beforeBytes[k]);

        // Then emit all K passthrough bytes directly.
        out.push(...passBytes);

        blockDigits = [];
        knownHighBytes = [];
        i += 1 + passLen;
      }
      continue;
    }

    // ------------------------------------------------------------------
    // Regular Z85 character — accumulate digit and flush full blocks.
    // ------------------------------------------------------------------
    const digit = CHAR_TO_DIGIT[code];
    if (digit === -1) {
      throw new Z855DecodeError(
        `invalid character 0x${code.toString(16).padStart(2, "0").toUpperCase()}`
      );
    }
    blockDigits.push(digit);
    i++;

    // How many digits do we need before we can output bytes?
    // - Normally: 5 (full Z85 block → 4 bytes)
    // - After a non-aligned `,` passthrough: 5 − P digits remain for the after-block,
    //   where P = knownHighBytes.length.
    const needed = 5 - knownHighBytes.length;

    if (blockDigits.length === needed) {
      // Decode the accumulated digits.
      const lowDigitsValue = digitsToValue(blockDigits);

      let value: number;
      if (knownHighBytes.length === 0) {
        // Normal full-block decode.
        value = lowDigitsValue;
        if (value > 0xffffffff) throw new Z855DecodeError("Z85 value overflow");
      } else {
        // After-block reconstruct: we know the top P bytes (from the passthrough),
        // and the Z85 digits encode the remaining low-order bits.
        value = reconstructAfterBlock(knownHighBytes, lowDigitsValue, needed);
        knownHighBytes = [];
      }

      out.push(...u32ToBytes(value));
      blockDigits = [];
    }
  }

  // ------------------------------------------------------------------
  // Flush any remaining partial block (2, 3, or 4 accumulated digits).
  //
  // This is the decoder side of the partial-block handling described in the
  // encoder.  Trailing digits that don't make a full 5-digit group are
  // interpreted as a shortened encoding of 1–3 bytes.
  // ------------------------------------------------------------------
  if (blockDigits.length > 0) {
    const numChars = blockDigits.length;
    if (numChars === 1) throw new Z855DecodeError("invalid length: single trailing character");

    const value = digitsToValue(blockDigits);
    const numBytes = numChars - 1; // 2 chars → 1 byte, 3 → 2, 4 → 3

    const maxValue = [0, 0xff, 0xffff, 0xffffff][numBytes];
    if (value > maxValue) throw new Z855DecodeError("Z85 value overflow in partial block");

    // Extract the bytes from the value (big-endian).
    for (let k = numBytes - 1; k >= 0; k--) {
      out.push((value >>> (k * 8)) & 0xff);
    }
  }

  return new Uint8Array(out);
}

// ---------------------------------------------------------------------------
// Encoding helpers
// ---------------------------------------------------------------------------

/** Encode a 32-bit value into exactly `numChars` Z85 characters. */
function encodeValue(value: number, numChars: number): string[] {
  const chars: string[] = new Array(numChars);
  let v = value;
  for (let i = numChars - 1; i >= 0; i--) {
    chars[i] = ALPHABET[v % 85];
    v = Math.floor(v / 85);
  }
  return chars;
}

/** Encode a full 4-byte block (always exactly 5 chars). */
function encodeFullBlock(value: number): string[] {
  return encodeValue(value, 5);
}

/** Read 4 bytes from `input[i..i+4]` as a big-endian unsigned 32-bit integer. */
function readU32(input: Uint8Array, i: number): number {
  return (
    ((input[i] << 24) | (input[i + 1] << 16) | (input[i + 2] << 8) | input[i + 3]) >>> 0
  );
}

/** Convert a 32-bit integer to 4 big-endian bytes. */
function u32ToBytes(value: number): [number, number, number, number] {
  return [
    (value >>> 24) & 0xff,
    (value >>> 16) & 0xff,
    (value >>> 8) & 0xff,
    value & 0xff,
  ];
}

/** True if all `count` bytes starting at `input[start]` are in the safe-char set. */
function allSafe(input: Uint8Array, start: number, count: number): boolean {
  for (let k = 0; k < count; k++) {
    if (!IS_SAFE[input[start + k]]) return false;
  }
  return true;
}

/** The standard Z85 output length for `n` input bytes: ceil(n × 5 / 4). */
function z855OutputLen(n: number): number {
  return Math.ceil((n * 5) / 4);
}

// ---------------------------------------------------------------------------
// Passthrough encoding — long (8+ bytes)
// ---------------------------------------------------------------------------

/**
 * Try to encode 8 or more consecutive safe bytes using the `|` long escape.
 *
 * Structure of the output:
 *   [prefix][`|`][optional dot padding][raw bytes][optional dot padding]
 *
 * The prefix encodes the number of raw bytes in base-42 (self-terminating).
 * The total envelope length equals `z855OutputLen(rawLen)` so the encoded
 * length is the same as if the bytes were Z85-encoded normally.
 *
 * Special case: when the safe run reaches the end of input, emit `0|` followed
 * by all remaining bytes — this is shorter than the general form.
 */
function tryLongPassthrough(
  input: Uint8Array,
  start: number
): { chars: string[]; consumed: number } | null {
  // Count consecutive safe bytes.
  let safeCount = 0;
  for (let j = start; j < input.length && safeCount < MAX_LONG_PASSTHROUGH; j++) {
    if (!IS_SAFE[input[j]]) break;
    safeCount++;
  }
  if (safeCount < 8) return null;

  // "Rest of input is raw" shortcut.
  if (start + safeCount === input.length) {
    const chars: string[] = ["0", "|"];
    for (let j = start; j < input.length; j++) chars.push(String.fromCharCode(input[j]));
    return { chars, consumed: safeCount };
  }

  // General form: fixed-length envelope.
  const rawLen = safeCount;
  const prefix = generateLongPrefix(rawLen);
  const totalLen = z855OutputLen(rawLen);
  const paddingNeeded = totalLen - prefix.length - 1 - rawLen; // padding = envelope − prefix − `|` − raw

  const chars: string[] = [...prefix, "|"];
  for (let j = 0; j < paddingNeeded; j++) chars.push(".");         // padding before raw bytes
  for (let j = 0; j < rawLen; j++) chars.push(String.fromCharCode(input[start + j]));
  // Note: we put all padding before the raw bytes for simplicity (offset = paddingNeeded).
  // The production encoder uses a smarter offset for alignment; the decoder ignores padding content.

  return { chars, consumed: rawLen };
}

/**
 * Generate the base-42 length prefix for the `|` escape.
 *
 * Base-42 self-terminating encoding:
 *   - Split `value` into base-42 digits (big-endian).
 *   - The most-significant digit is the *terminal* digit (value < 42, output as-is).
 *   - All other digits are *continuation* digits (value += 42 before emitting).
 *
 * The Z85 alphabet is used to represent digit values (digit 0 → `'0'`, etc.).
 */
function generateLongPrefix(value: number): string[] {
  if (value < 42) return [ALPHABET[value]];

  const digits: number[] = [];
  let v = value;
  while (v > 0) {
    digits.push(v % 42);
    v = Math.floor(v / 42);
  }
  digits.reverse(); // big-endian

  return digits.map((d, idx) => (idx === 0 ? ALPHABET[d] : ALPHABET[d + 42]));
}

/**
 * Parse the prefix digits accumulated before a `|` and return
 * `{ offset, length }` where `length` is the raw byte count
 * (0 = rest-of-input) and `offset` is the dot padding before raw data.
 *
 * The prefix may contain two self-terminating base-42 numbers:
 *   [optional offset number][length number]
 * Both are read right-to-left to find their boundaries.
 */
function readLongEscapePrefix(digits: number[]): { offset: number; length: number } {
  // Read the rightmost number first — that is the length.
  const { value: length, count: lengthCount } = readBase42Backwards(digits, digits.length);
  if (lengthCount === digits.length) {
    // Only one number: it is the length, offset defaults to 0.
    return { offset: 0, length };
  }
  // There is a second number before the length: that is the offset.
  const { value: offset } = readBase42Backwards(digits, digits.length - lengthCount);
  return { offset, length };
}

/** Read a single base-42 self-terminating number backwards from `digits[0..end]`. */
function readBase42Backwards(
  digits: number[],
  end: number
): { value: number; count: number } {
  let value = 0;
  let multiplier = 1;
  let pos = end;
  let count = 0;

  while (pos > 0) {
    pos--;
    count++;
    const d = digits[pos];
    if (d >= 42) {
      // Continuation digit.
      value += (d - 42) * multiplier;
      multiplier *= 42;
    } else {
      // Terminal digit — stops the number.
      value += d * multiplier;
      break;
    }
  }

  return { value, count };
}

// ---------------------------------------------------------------------------
// Passthrough encoding — extended (5, 6, or 7 bytes)
// ---------------------------------------------------------------------------

/**
 * Try to encode 5, 6, or 7 consecutive safe bytes using the extended escapes
 * (`;`, `_`, `~`).
 *
 * Structure: [(P+1) Z85 chars] [ESC] [K raw bytes]
 *
 * P is chosen so that:
 *   - `input[blockStart + P .. blockStart + P + K]` are all safe, and
 *   - The total output length (P+1 + 1 + K) + z855OutputLen(remaining after) equals
 *     z855OutputLen(total remaining), preserving the length invariant.
 *
 * Using P+1 chars (instead of P as with the `,` escape) fully determines the
 * before-block, eliminating the need for a canonical-minimum check.
 *
 * Longer K is preferred (7 > 6 > 5) and block-aligned forms (P=0) are tried first.
 */
function tryExtendedPassthrough(
  input: Uint8Array,
  blockStart: number
): { chars: string[]; consumed: number } | null {
  for (const k of [7, 6, 5]) {
    // Choose the escape character for this K.
    const esc = k === 7 ? ESC_7 : k === 6 ? ESC_6 : ESC_5;

    // Try block-aligned (P=0): just [ESC][K raw bytes], no preceding Z85 digits.
    if (blockStart + k <= input.length && allSafe(input, blockStart, k)) {
      const totalRemaining = input.length - blockStart;
      const remaining = totalRemaining - k;
      // Check length invariant: (1 + k) + z855OutputLen(remaining) === z855OutputLen(totalRemaining)
      if (1 + k + z855OutputLen(remaining) === z855OutputLen(totalRemaining)) {
        const chars: string[] = [String.fromCharCode(esc)];
        for (let j = 0; j < k; j++) chars.push(String.fromCharCode(input[blockStart + j]));
        return { chars, consumed: k };
      }
    }

    // Try non-aligned positions P = 0 through 3 (P=0 was already tried above).
    for (let p = 1; p <= 3; p++) {
      const passStart = blockStart + p;
      if (passStart + k > input.length) continue;
      if (!allSafe(input, passStart, k)) continue;

      const totalRemaining = input.length - blockStart;
      const bytesConsumed = p + k;
      const remaining = totalRemaining - bytesConsumed;
      const outputChars = (p + 1) + 1 + k; // (P+1) Z85 + ESC + K raw
      if (outputChars + z855OutputLen(remaining) !== z855OutputLen(totalRemaining)) continue;

      // Read the before-block and emit (P+1) high-order Z85 digits.
      if (blockStart + 4 > input.length) continue;
      const beforeValue = readU32(input, blockStart);
      const chars: string[] = [...topNDigits(beforeValue, p + 1), String.fromCharCode(esc)];
      for (let j = 0; j < k; j++) chars.push(String.fromCharCode(input[passStart + j]));

      return { chars, consumed: bytesConsumed };
    }
  }
  return null;
}

// ---------------------------------------------------------------------------
// Passthrough encoding — non-aligned 4-byte
// ---------------------------------------------------------------------------

/**
 * Try to encode a non-aligned 4-byte passthrough.
 *
 * This form handles the case where 4 safe bytes appear at byte offset P (1–3)
 * within the current 4-byte input block.  The encoder emits:
 *   [P Z85 chars for before-block][`,`][4 safe bytes][(5−P) Z85 chars for after-block]
 *
 * For round-tripping to work, the 32-bit "before" block value must equal the
 * canonical minimum for its P Z85 digits and the (4−P) known low bytes
 * (the decoder uses that same rule to recover the before-block).  We check
 * this before committing.
 *
 * The "after" block also needs at least 4 bytes of input; if not enough remain,
 * we skip this form.
 */
function tryNonAlignedPassthrough(
  input: Uint8Array,
  blockStart: number
): { chars: string[]; consumed: number } | null {
  for (let p = 1; p <= 3; p++) {
    const passStart = blockStart + p;
    if (passStart + 4 > input.length) continue;
    if (!allSafe(input, passStart, 4)) continue;

    const beforeValue = readU32(input, blockStart);
    const numKnownLow = 4 - p;
    const knownLow: number[] = [];
    for (let k = 0; k < numKnownLow; k++) knownLow.push(input[passStart + k]);

    // Verify before-block is the canonical minimum (required for correct decoding).
    if (!isCanonicalMinimum(beforeValue, p, knownLow)) continue;

    // We also need enough input to form the after-block.
    const afterStart = blockStart + 4;
    const afterRemaining = input.length - (afterStart + p);
    if (afterRemaining < 4 - p) continue;

    // Assemble the full after-block value.
    const afterBytes: number[] = [];
    for (let k = 0; k < p; k++) afterBytes.push(input[passStart + numKnownLow + k]); // from passthrough
    for (let k = 0; k < 4 - p; k++) afterBytes.push(input[afterStart + p + k]);       // from input
    const afterValue =
      ((afterBytes[0] << 24) | (afterBytes[1] << 16) | (afterBytes[2] << 8) | afterBytes[3]) >>> 0;

    // Build output.
    const chars: string[] = [
      ...topNDigits(beforeValue, p),
      ",",
      ...Array.from({ length: 4 }, (_, k) => String.fromCharCode(input[passStart + k])),
      ...bottomNDigits(afterValue, 5 - p),
    ];

    return { chars, consumed: 8 }; // before block (4) + after block (4)
  }
  return null;
}

// ---------------------------------------------------------------------------
// Canonical-minimum logic (shared by encoder and decoder)
// ---------------------------------------------------------------------------

/**
 * Return the canonical (minimum) 32-bit value consistent with:
 *   - `highDigits`: the P high-order Z85 digits that were emitted
 *   - `knownLowBytes`: the (4−P) low-order bytes known from the passthrough
 *
 * The P Z85 digits define a contiguous range of u32 values (a "stripe").
 * Within that stripe, many values share the same low (4−P) bytes (mod 2^(8*(4−P))).
 * We return the *smallest* such value — that is what the encoder requires and
 * what the decoder will produce.
 */
function canonicalMinimum(highDigits: number[], knownLowBytes: number[]): number {
  const P = highDigits.length;

  // Compute the start of the range defined by these P Z85 digits.
  let base = 0;
  for (const d of highDigits) base = base * 85 + d;
  const power = Math.pow(85, 5 - P);       // 85^(5−P)
  const rangeStart = base * power;
  const rangeEnd = (base + 1) * power;

  if (knownLowBytes.length === 0) {
    // P = 4: the 5 Z85 digits uniquely determine the value.
    return rangeStart;
  }

  // Pack the known low bytes into a number.
  let knownPart = 0;
  for (const b of knownLowBytes) knownPart = (knownPart << 8) | b;

  const modulus = Math.pow(2, knownLowBytes.length * 8);
  const startRemainder = rangeStart % modulus;

  // Find the smallest value ≥ rangeStart congruent to knownPart (mod modulus).
  let candidate = rangeStart - startRemainder + knownPart;
  if (startRemainder > knownPart) candidate += modulus;

  if (candidate >= rangeEnd || candidate > 0xffffffff) {
    throw new Z855DecodeError("non-aligned passthrough: no valid before-block value");
  }
  return candidate;
}

/** Return true if `value` equals the canonical minimum for these digits + known bytes. */
function isCanonicalMinimum(value: number, P: number, knownLowBytes: number[]): boolean {
  const highDigits = topNDigits(value, P).map((ch) => ALPHABET.indexOf(ch));
  try {
    return value === canonicalMinimum(highDigits, knownLowBytes);
  } catch {
    return false;
  }
}

/**
 * Determine the before-block value from (P+1) Z85 digits and (4−P) known low bytes.
 *
 * Unlike the `,` case (which needs a canonical-minimum search), the extended
 * escapes (`;`/`_`/`~`) emit *one extra* Z85 digit.  That extra digit is always
 * enough information to uniquely identify the value — no search needed.
 */
function beforeBlockFromExtendedDigits(highDigits: number[], knownLowBytes: number[]): number {
  // Same range computation as in canonicalMinimum, but here we expect exactly one answer.
  let base = 0;
  for (const d of highDigits) base = base * 85 + d;
  const power = Math.pow(85, 5 - highDigits.length);
  const rangeStart = base * power;
  const rangeEnd = (base + 1) * power;

  if (knownLowBytes.length === 0) {
    if (rangeStart > 0xffffffff) throw new Z855DecodeError("overflow in extended passthrough decode");
    return rangeStart;
  }

  let knownPart = 0;
  for (const b of knownLowBytes) knownPart = (knownPart << 8) | b;

  const modulus = Math.pow(2, knownLowBytes.length * 8);
  const startRemainder = rangeStart % modulus;

  let candidate = rangeStart - startRemainder + knownPart;
  if (startRemainder > knownPart) candidate += modulus;

  if (candidate >= rangeEnd || candidate > 0xffffffff) {
    throw new Z855DecodeError("extended passthrough: no valid before-block value");
  }
  return candidate;
}

// ---------------------------------------------------------------------------
// After-block reconstruction (for non-aligned `,` decode)
// ---------------------------------------------------------------------------

/**
 * After a non-aligned `,` passthrough, reconstruct the 32-bit "after" block from:
 *   - `knownHigh`: the P known high bytes (from the passthrough)
 *   - `lowDigitsValue`: the numeric value of the (5−P) low Z85 digits
 *   - `numDigits`: how many low digits there were (5−P)
 */
function reconstructAfterBlock(
  knownHigh: number[],
  lowDigitsValue: number,
  numDigits: number
): number {
  const P = knownHigh.length;

  let highPart = 0;
  for (const b of knownHigh) highPart = (highPart << 8) | b;

  const shift = 8 * (4 - P);
  const rangeStart = (highPart << shift) >>> 0;
  const rangeSize = 1 << shift;
  const modulus = Math.pow(85, numDigits);

  const startRemainder = rangeStart % modulus;
  let candidate = rangeStart - startRemainder + lowDigitsValue;
  if (startRemainder > lowDigitsValue) candidate += modulus;

  if (candidate >= rangeStart + rangeSize) {
    throw new Z855DecodeError("after-block reconstruction failed");
  }
  return candidate >>> 0;
}

// ---------------------------------------------------------------------------
// Hash-padding decode (concatenatable mode)
// ---------------------------------------------------------------------------

function decodeHashPaddedBlock(input: string, blockStart: number): { bytes: number[] } {
  let hashRun = 0;
  while (hashRun < 3 && input.charCodeAt(blockStart + hashRun) === PAD_HASH) hashRun++;

  const numChars = 5 - hashRun;
  const numBytes = numChars - 1;

  let value = 0;
  for (let k = 0; k < numChars; k++) {
    const code = input.charCodeAt(blockStart + hashRun + k);
    const digit = CHAR_TO_DIGIT[code];
    if (digit === -1) throw new Z855DecodeError("invalid character in hash-padded block");
    value = value * 85 + digit;
  }

  const bytes: number[] = [];
  for (let k = numBytes - 1; k >= 0; k--) bytes.unshift((value >>> (k * 8)) & 0xff);
  return { bytes };
}

// ---------------------------------------------------------------------------
// Z85 digit utilities
// ---------------------------------------------------------------------------

/** Convert an array of Z85 digit values to a numeric value. */
function digitsToValue(digits: number[]): number {
  let v = 0;
  for (const d of digits) v = v * 85 + d;
  return v;
}

/** Return the top `n` Z85 characters of `value` (as character strings). */
function topNDigits(value: number, n: number): string[] {
  const all = encodeValue(value, 5);
  return all.slice(0, n);
}

/** Return the bottom `n` Z85 characters of `value` (as character strings). */
function bottomNDigits(value: number, n: number): string[] {
  const all = encodeValue(value, 5);
  return all.slice(5 - n);
}

/** Return the short passthrough length (4, 5, 6, or 7) for an escape char, or 0. */
function shortPassthroughLength(code: number): number {
  if (code === ESC_4) return 4;
  if (code === ESC_5) return 5;
  if (code === ESC_6) return 6;
  if (code === ESC_7) return 7;
  return 0;
}
