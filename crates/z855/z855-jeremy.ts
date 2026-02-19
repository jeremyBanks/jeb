#!/usr/bin/env -S deno run --allow-read --allow-write
/**
 * Z855: Extended Z85 Binary-to-Text Encoding with Safe-String Passthrough
 *
 * Z855 is a superset of Z85 that allows human-readable strings to pass through
 * unencoded using escape sequences, while maintaining perfect alignment with
 * standard Z85 for non-escaped blocks. The decoder handles all encoder options
 * automatically without configuration, and can decode standard Z85.
 *
 * ══════════════════════════════════════════════════════════════════════════════
 * § 1. WHAT IS Z85?
 * ══════════════════════════════════════════════════════════════════════════════
 *
 * Z85 (RFC: https://rfc.zeromq.org/spec/32/) is a base-85 encoding that maps
 * 4 bytes → 5 ASCII characters, using an alphabet chosen to avoid escaping in
 * source code and config files:
 *
 *   0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#
 *
 * Advantages over base64:
 *   • More compact (5 chars for 4 bytes vs. 4 chars for 3 bytes)
 *   • 4-byte blocks align with real-world data structures (u32, IPv4, etc.)
 *   • 64 bytes fit in 80-character lines
 *   • Alphabet starts with '0', making zero values and small integers visible
 *
 * Disadvantages:
 *   • Larger alphabet (less portable)
 *   • Slower (requires division/modulo instead of bit shifts)
 *
 * ══════════════════════════════════════════════════════════════════════════════
 * § 2. WHY Z855?
 * ══════════════════════════════════════════════════════════════════════════════
 *
 * Z855 adds 5 escape characters (_, , ~ ; |) to allow "safe" byte sequences
 * (those composed of printable ASCII in the Z85+escape alphabet) to pass through
 * RAW and UNENCODED. This dramatically improves readability when binary data
 * contains readable strings like URLs, code snippets, or configuration values.
 *
 * Example: encoding "Hello, world!" in Z855 might produce ",Hello_worl_d!00"
 * where the readable portions are preserved verbatim.
 *
 * Key properties:
 *   • Non-escaped Z85 blocks appear at IDENTICAL positions as in standard Z85
 *   • Output length matches standard Z85 (except for 0| rest-of-input escape)
 *   • Deterministic decoding without encoder configuration
 *
 * ══════════════════════════════════════════════════════════════════════════════
 * § 3. ESCAPE SYSTEM
 * ══════════════════════════════════════════════════════════════════════════════
 *
 * Escape prefixes indicate that following bytes should be passed through raw:
 *
 *   _      4-byte passthrough
 *   ,      5-byte passthrough
 *   ~      6-byte passthrough
 *   ;      7-byte passthrough
 *   N|     long passthrough (8+ bytes), where N encodes length and offset
 *   0|     rest-of-input passthrough (non-concatenatable mode only)
 *
 * Short escapes (4-7 bytes) have two modes:
 *
 *   • BLOCK-ALIGNED: Escape appears at start of a Z85 block boundary.
 *     Format: [escape][K raw bytes]
 *     Example: "_test" = 4 raw bytes "test"
 *
 *   • NON-ALIGNED: Escape appears within a block (position P = 1, 2, or 3).
 *     Format: [(P or P+1) Z85 chars][escape][K raw bytes]
 *     The Z85 chars encode the "before" block that the raw bytes interrupt.
 *
 * Long escapes (8+ bytes) use base-42 encoding in the prefix before |:
 *
 *   Format: [offset-base42?][length-base42]|[padding-before][raw-bytes][padding-after]
 *
 *   • Length is ALWAYS encoded in base-42
 *   • Offset is encoded if length > 15 AND offset > 0
 *   • Padding (typically '.') fills space to maintain Z85 output length
 *   • Special case "0|" means "all remaining bytes are raw" (no padding)
 *
 * ══════════════════════════════════════════════════════════════════════════════
 * § 4. BLOCK ALIGNMENT & THE CANONICAL MINIMUM RULE
 * ══════════════════════════════════════════════════════════════════════════════
 *
 * Z85 encodes 4 bytes as 5 characters. When a 4-byte passthrough interrupts a
 * block at position P (1-3), the encoder must emit P Z85 characters representing
 * the "before" block's high-order digits, followed by _, followed by 4 raw bytes.
 *
 * The decoder needs to reconstruct the full before-block 32-bit value from:
 *   • P high-order Z85 digit indices
 *   • (4-P) known low bytes (the first bytes of the passthrough)
 *
 * To ensure UNIQUE decoding, the before-block value must be the CANONICAL MINIMUM:
 * the smallest 32-bit value whose Z85 encoding starts with those P digits and
 * whose low (4-P) bytes match the passthrough bytes.
 *
 * Mathematical formula (see canonicalMin() for implementation):
 *   1. Compute the range [rangeStart, rangeEnd) of values matching P high digits
 *   2. Find the smallest value in that range whose low bytes match
 *   3. Use modular arithmetic to solve efficiently without enumeration
 *
 * Example: If P=2 (two high Z85 chars "ab"), and the passthrough starts with
 * bytes [0x12, 0x34], we find the minimum value V where:
 *   • V's Z85 encoding starts with "ab"
 *   • V's low 2 bytes are [0x12, 0x34]
 *
 * ══════════════════════════════════════════════════════════════════════════════
 * § 5. EXTENDED PASSTHROUGH (5/6/7 BYTES)
 * ══════════════════════════════════════════════════════════════════════════════
 *
 * For 5, 6, and 7-byte passthroughs, the encoding uses (P+1) Z85 characters
 * instead of P for the before-block. This UNIQUELY determines the before-block
 * value without needing the canonical-minimum constraint.
 *
 * Why this works:
 *   • P+1 high-order digits + (4-P) known low bytes = 5 constraints total
 *   • 5 constraints fully specify a unique 32-bit value
 *   • No ambiguity → no need for canonical-min check
 *
 * This is handled by extendedBlockValue(), which returns the unique value or -1.
 *
 * Format:
 *   • Block-aligned (P=0): [escape][K bytes]
 *   • Non-aligned (P≥1): [(P+1) Z85 chars][escape][K bytes]
 *
 * The encoder tries all valid positions P and picks the one with the best
 * alignment score (see § 7).
 *
 * ══════════════════════════════════════════════════════════════════════════════
 * § 6. LONG ESCAPE (|) FORMAT & BASE-42 ENCODING
 * ══════════════════════════════════════════════════════════════════════════════
 *
 * For 8+ byte passthroughs, the prefix format is:
 *
 *   [offset-digits?][length-digits]|[padding-before][raw-bytes][padding-after]
 *
 * Both length and offset are encoded in BASE-42 using Z85 alphabet chars:
 *
 *   • Digits 0-41 (Z85 chars '0'-'g'): represent themselves, STOP
 *   • Digits 42-84 (Z85 chars 'h'-'#'): represent (digit - 42), CONTINUE
 *
 * This continuation scheme allows arbitrarily large values. When decoding,
 * scan RIGHT-TO-LEFT from the | character:
 *
 *   1. Read length digits (stop when digit < 42)
 *   2. If length > 15, read offset digits (same scheme)
 *   3. If length ≤ 15, offset is implicitly 0
 *
 * The offset specifies how many padding bytes appear BEFORE the raw data.
 * Total padding = (Z85 output length for N bytes) - prefix_length - 1 - N
 *
 * Special case: "0|" means "rest of input is raw, no padding" (shorter output).
 *
 * Why base-42? It's exactly half of 84 (the number of Z85 chars usable as
 * digits, excluding |). This makes the continuation bit scheme clean.
 *
 * ══════════════════════════════════════════════════════════════════════════════
 * § 7. CONCATENATABLE MODE & HASH PADDING
 * ══════════════════════════════════════════════════════════════════════════════
 *
 * Standard Z85 requires input length to be a multiple of 4 bytes. Z855 supports
 * variable-length input by encoding partial blocks (1-3 trailing bytes) as
 * 2-4 output characters.
 *
 * Problem: This breaks concatenation. If you concatenate two Z855 outputs with
 * partial blocks, the decoder can't tell where one ends and the next begins.
 *
 * Solution: CONCATENATABLE MODE pads output to a multiple of 5 characters using
 * the '#' character (hash padding). The '#' character is chosen because:
 *   • It's a valid Z85 character
 *   • It NEVER appears at the start of any Z85 block (full or partial)
 *   • Thus it can be unambiguously detected and stripped by the decoder
 *
 * Hash padding appears at the BEGINNING of the output (not the end like base64).
 *
 * Example: encoding 1 byte (0x07):
 *   • Canonical mode: "07" (2 chars)
 *   • Concatenatable mode: "###07" (5 chars, padded to block boundary)
 *
 * The encoder inserts hash padding BEFORE the final partial block to achieve
 * 5-alignment without interrupting raw passthroughs.
 *
 * ══════════════════════════════════════════════════════════════════════════════
 * § 8. BIT-REVERSAL ALIGNMENT OPTIMIZATION
 * ══════════════════════════════════════════════════════════════════════════════
 *
 * When multiple valid passthrough positions exist, the encoder picks the one
 * with the BEST ALIGNMENT SCORE to minimize fragmentation and improve output
 * aesthetics.
 *
 * Alignment scoring uses BIT-REVERSAL: positions aligned to power-of-2 boundaries
 * (0, 4, 8, 16, ...) have trailing zeros in binary. Bit-reversal turns these
 * into LEADING zeros, so they sort FIRST lexicographically.
 *
 * For a span [start, end], the sort key is a 4-tuple of bigints:
 *   [ min(bitReverse(start), bitReverse(end)),
 *     max(bitReverse(start), bitReverse(end)),
 *     min(bitReverse(outputStart), bitReverse(outputEnd)),
 *     max(bitReverse(outputStart), bitReverse(outputEnd)) ]
 *
 * This ensures that:
 *   • Input-aligned passthroughs are preferred
 *   • Output-aligned passthroughs are preferred
 *   • Among equally-aligned positions, lower positions win
 *
 * See findBestOffset() for the full implementation.
 *
 * ══════════════════════════════════════════════════════════════════════════════
 * § 9. ENCODER OPTION SETS
 * ══════════════════════════════════════════════════════════════════════════════
 *
 * Two standard option sets are defined:
 *
 *   CANONICAL_ENCODING (default):
 *     • concatenatable: false (allows variable-length output)
 *     • extraSafeBytes: "_,~;|" (all escape characters)
 *     • maxRawLength: 64KB (practical limit for passthrough blocks)
 *
 *   CONCATENATABLE_ENCODING:
 *     • concatenatable: true (pads to 5-char boundaries with #)
 *     • extraSafeBytes: "_,~;|"
 *     • maxRawLength: 64KB
 *
 * Encoders may customize these options, but decoders handle all variants
 * automatically without configuration.
 */

import { assert } from "jsr:@std/assert";
import { parseArgs } from "jsr:@std/cli/parse-args";
import { readAll } from "jsr:@std/io";

// ─── Constants ───

const z85 = "" +
  "0123456789abcdefghijk" +
  "lmnopqrstuvwxyzABCDEF" +
  "GHIJKLMNOPQRSTUVWXYZ." +
  "-:+=^!/*?&<>()[]{}@%$" +
  "#";
const Z85_DIGITS = [...z85];
const Z85_DIGIT_BYTES = new TextEncoder().encode(z85);
const Z85_VALUES = new Map(
  Z85_DIGITS.map((digit, index) => [digit, index]),
);
const Z85_VALUES_BYTES = new Map(
  Z85_DIGIT_BYTES.entries().map(([index, byte]) => [byte, index]),
);

const BLOCK_SIZE_ORIGINAL = 4;

const ESCAPE_4 = "_";  // Jeremy's new assignment (was `,` in production)
const ESCAPE_5 = ",";  // Jeremy's new assignment (was `;` in production)
const ESCAPE_6 = "~";  // Jeremy's new assignment (was `_` in production)
const ESCAPE_7 = ";";  // Jeremy's new assignment (was `~` in production)
const ESCAPE_MANY = "|";
const PAD_HASH = 0x23;  // '#' — padding for concatenatable mode

/** Options for encoding using Z855. (Decoders support all options without requiring any configuration.) */
export interface EncodeOptions {
  /** Whether to add padding to support concatenating multiple encoded values together. */
  concatenatable?: boolean;
  /** Bytes that are considered "safe" and will not be escaped beyond the Z85 alphabet. */
  extraSafeBytes?: Iterable<string | number>;
  /** Sequences of bytes that are considered "unsafe" and will not be included in escapes. */
  unsafeSequences?: Iterable<string | Iterable<number>>;
  /** The maximum size in bytes of a raw escape block. */
  maxRawLength?: number;
}

/** Canonical Z855 encoding. */
export const CANONICAL_ENCODING: Required<EncodeOptions> = {
  concatenatable: false,
  extraSafeBytes: "_,~;|",
  maxRawLength: 64 * 1024,
  unsafeSequences: [],
};

/** Concatenatable Z855 encoding. */
export const CONCATENATABLE_ENCODING: Required<EncodeOptions> = {
  concatenatable: true,
  extraSafeBytes: "_,~;|",
  maxRawLength: 64 * 1024,
  unsafeSequences: [],
};

/**
 * Printable ASCII Z855 encoding with terminal-size raw blocks and markdown
 * code fence escaping. Looks nice when split into 80 character lines, if you
 * strip all newlines before decoding.
 */
export const PRINTABLE_ASCII_ENCODING: Required<EncodeOptions> = {
  concatenatable: true,
  extraSafeBytes: `_,~;| "'\`\\`,
  maxRawLength: 64 * 24,
  unsafeSequences: ["```"],
};

// ─── Alignment helpers ───

/**
 * Bit-reversal of n treated as a 64-bit integer.
 *
 * Positions aligned to power-of-2 boundaries have trailing zeros in binary.
 * Bit-reversal puts those zeros at the front, so power-of-2 aligned positions
 * sort FIRST when comparing bit-reversed values lexicographically.
 *
 * This is used to pick the "best aligned" passthrough position when multiple
 * options exist. For example, position 0 (perfectly aligned) has the most
 * trailing zeros, so it sorts first after bit-reversal.
 */
function bitReverse(n: number): bigint {
  let x = BigInt(n);
  // Standard bit-reversal algorithm using divide-and-conquer swaps.
  x = ((x & 0x5555555555555555n) << 1n) | ((x >> 1n) & 0x5555555555555555n);
  x = ((x & 0x3333333333333333n) << 2n) | ((x >> 2n) & 0x3333333333333333n);
  x = ((x & 0x0f0f0f0f0f0f0f0fn) << 4n) | ((x >> 4n) & 0x0f0f0f0f0f0f0f0fn);
  x = ((x & 0x00ff00ff00ff00ffn) << 8n) | ((x >> 8n) & 0x00ff00ff00ff00ffn);
  x = ((x & 0x0000ffff0000ffffn) << 16n) | ((x >> 16n) & 0x0000ffff0000ffffn);
  x = (x << 32n) | (x >> 32n);
  return x;
}

/** Lexicographic compare of two 2-tuples of bigints. */
function cmp2(a: [bigint, bigint], b: [bigint, bigint]): number {
  if (a[0] !== b[0]) return a[0] < b[0] ? -1 : 1;
  if (a[1] !== b[1]) return a[1] < b[1] ? -1 : 1;
  return 0;
}

/** Lexicographic compare of two 4-tuples of bigints. */
function cmp4(a: [bigint, bigint, bigint, bigint], b: [bigint, bigint, bigint, bigint]): number {
  for (let i = 0; i < 4; i++) {
    if (a[i] < b[i]) return -1;
    if (a[i] > b[i]) return 1;
  }
  return 0;
}

/** Expected Z85 output length for N input bytes. */
function z855OutputLen(n: number): number {
  return Math.ceil(n * 5 / 4);
}

// ─── Canonical minimum (closed-form) ───

/**
 * Compute the canonical minimum 32-bit value for a non-aligned 4-byte passthrough.
 *
 * Given:
 *   • P high-order Z85 digit indices (P = 1, 2, or 3)
 *   • (4-P) known low bytes from the passthrough
 *
 * Returns: The MINIMUM 32-bit value V where:
 *   • V's Z85 encoding starts with the given P high-order digits
 *   • V's low (4-P) bytes match the known bytes
 *   • Returns -1 if no such value exists
 *
 * Algorithm:
 *   1. Compute the range [rangeStart, rangeEnd) of all 32-bit values whose
 *      Z85 encoding starts with the P high-order digits.
 *   2. Use modular arithmetic to find the smallest value in that range
 *      whose low bytes match the constraint.
 *
 * Example: P=2, highDigits=[10, 20], knownLowBytes=[0xAB, 0xCD]
 *   • rangeStart = (10*85 + 20) * 85³ = value whose Z85 starts "..."
 *   • We want rangeStart ≤ V < rangeEnd and V % 0x10000 == 0xABCD
 *   • Solve using modular arithmetic to avoid enumeration
 */
function canonicalMin(highDigits: number[], knownLowBytes: number[]): number {
  const P = highDigits.length;
  const numKnown = knownLowBytes.length; // = 4 - P

  // Compute the base value from high digits (accumulated in base 85).
  let base = 0;
  for (const d of highDigits) base = base * 85 + d;

  // Range of 32-bit values matching the P high digits.
  const power = Math.pow(85, 5 - P);
  const rangeStart = base * power;
  const rangeEnd   = (base + 1) * power;

  if (numKnown === 0) {
    // No low-byte constraints; just return rangeStart if valid.
    return rangeStart > 0xffffffff ? -1 : rangeStart;
  }

  // Pack known low bytes into a single integer.
  let knownPart = 0;
  for (const b of knownLowBytes) knownPart = (knownPart << 8 | b) >>> 0;

  // Find the minimum value in [rangeStart, rangeEnd) whose low bytes match knownPart.
  const modulus = Math.pow(2, numKnown * 8);
  const rem = rangeStart % modulus;
  let candidate = rem <= knownPart
    ? rangeStart - rem + knownPart
    : rangeStart - rem + modulus + knownPart;

  if (candidate >= rangeEnd || candidate > 0xffffffff) return -1;
  return candidate;
}

/**
 * Compute the unique 32-bit value for an extended passthrough (5/6/7 bytes).
 *
 * Given:
 *   • (P+1) high-order Z85 digit indices
 *   • (4-P) known low bytes from the passthrough
 *
 * Returns: The UNIQUE 32-bit value V satisfying both constraints, or -1 if none.
 *
 * Unlike canonicalMin(), this function has P+1 high-order digits instead of P.
 * Since we have 5 total constraints (P+1 digits + 4-P bytes), the value is
 * UNIQUELY determined (not just a minimum). This eliminates the need for a
 * canonical-minimum check during encoding.
 *
 * The algorithm is identical to canonicalMin() but with different parameters.
 */
function extendedBlockValue(highDigits: number[], knownLowBytes: number[]): number {
  const numDigits = highDigits.length; // P+1
  const numKnown  = knownLowBytes.length; // 4-P

  let base = 0;
  for (const d of highDigits) base = base * 85 + d;

  const power = Math.pow(85, 5 - numDigits);
  const rangeStart = base * power;
  const rangeEnd   = (base + 1) * power;

  if (numKnown === 0) {
    // P=4 → numDigits=5: full block, unique value
    return rangeStart > 0xffffffff ? -1 : rangeStart;
  }

  let knownPart = 0;
  for (const b of knownLowBytes) knownPart = (knownPart << 8 | b) >>> 0;

  const modulus = Math.pow(2, numKnown * 8);
  const rem = rangeStart % modulus;
  let candidate = rem <= knownPart
    ? rangeStart - rem + knownPart
    : rangeStart - rem + modulus + knownPart;

  if (candidate >= rangeEnd || candidate > 0xffffffff) return -1;
  return candidate;
}

/**
 * Extract the first P Z85 digit indices from a 32-bit value.
 *
 * Z85 represents a 32-bit value as 5 base-85 digits (MSB first).
 * This returns just the first P digits as an array of indices.
 */
function highDigits(v: number, P: number): number[] {
  const all: number[] = new Array(5);
  let x = v;
  for (let i = 4; i >= 0; i--) { all[i] = x % 85; x = Math.floor(x / 85); }
  return all.slice(0, P);
}

/**
 * Check if a value is the canonical minimum for the given position and passthrough bytes.
 *
 * Used during encoding to verify that a non-aligned 4-byte passthrough satisfies
 * the canonical-minimum constraint required for unique decoding.
 */
function isCanonMin(value: number, P: number, passBytesLow: number[]): boolean {
  return canonicalMin(highDigits(value, P), passBytesLow) === value;
}

// ─── Long-escape prefix encoding ───

/**
 * Encode a non-negative integer as base-42 Z85 chars with continuation bits.
 *
 * Base-42 encoding uses Z85_DIGITS[0..83] with a continuation scheme:
 *   • Digits 0-41: represent themselves, signal STOP (last digit)
 *   • Digits 42-84: represent (digit - 42), signal CONTINUE (more digits follow)
 *
 * Returns: Array of Z85 character strings, MSB first.
 *
 * Example: encodeBase42(100) might return ["h", "n"] where:
 *   • 100 = 2*42 + 16
 *   • First digit: 2 + 42 = 44 (continue bit set) → 'h'
 *   • Second digit: 16 (stop bit) → 'G'
 */
function encodeBase42(n: number): string[] {
  if (n < 42) return [Z85_DIGITS[n]];
  const digits: number[] = [];
  let v = n;
  while (v > 0) { digits.push(v % 42); v = Math.floor(v / 42); }
  digits.reverse();
  // First digit doesn't need continuation bit; remaining digits do.
  return digits.map((d, i) => Z85_DIGITS[i === 0 ? d : d + 42]);
}

/**
 * Find the best offset for a long-escape block to maximize alignment quality.
 *
 * When encoding a long passthrough (8+ bytes), we have flexibility in where to
 * place padding bytes (before vs. after the raw data). The offset parameter
 * controls how many padding bytes appear BEFORE the raw data.
 *
 * This function enumerates all valid offsets and picks the one with the best
 * alignment score using bit-reversal sort keys (see § 8).
 *
 * @param currentOutputLen - Current position in output buffer
 * @param lengthPrefixLen - Number of base-42 digits encoding the length
 * @param rawLen - Number of raw bytes to pass through
 * @param paddingNeeded - Total padding bytes needed
 * @returns Best offset (number of padding bytes before raw data)
 */
function findBestOffset(
  currentOutputLen: number,
  lengthPrefixLen: number,
  rawLen: number,
  paddingNeeded: number,
): number {
  let bestOffset = 0;
  let bestKey: [bigint, bigint, bigint, bigint] | null = null;

  for (let offset = 0; offset <= paddingNeeded; offset++) {
    // Offset prefix length (0 if offset is 0).
    const offsetPrefixLen = offset > 0 ? encodeBase42(offset).length : 0;
    // Check if this offset fits within available padding space.
    if (offsetPrefixLen + offset > paddingNeeded) continue;

    // Compute input and output positions of the raw data span.
    const outputStart = currentOutputLen + offsetPrefixLen + lengthPrefixLen + 1 + offset;
    const outputEnd   = outputStart + rawLen - 1;
    const inputStart  = Math.floor(outputStart * 4 / 5);
    const inputEnd    = Math.floor(outputEnd   * 4 / 5);

    // Bit-reversed alignment sort key: [min(inputRev), max(inputRev), min(outputRev), max(outputRev)]
    const riS = bitReverse(inputStart), riE = bitReverse(inputEnd);
    const roS = bitReverse(outputStart), roE = bitReverse(outputEnd);
    const key: [bigint, bigint, bigint, bigint] = [
      riS < riE ? riS : riE, riS > riE ? riS : riE,
      roS < roE ? roS : roE, roS > roE ? roS : roE,
    ];

    if (bestKey === null || cmp4(key, bestKey) < 0) { bestKey = key; bestOffset = offset; }
  }
  return bestOffset;
}

// ─── Safe-byte helpers ───

/** Check if K consecutive bytes starting at `start` are all safe. */
function kBytesSafe(input: Uint8Array, start: number, k: number, safe: boolean[]): boolean {
  if (start + k > input.length) return false;
  for (let i = 0; i < k; i++) if (!safe[input[start + i]]) return false;
  return true;
}

// ─── Encoder ───

/**
 * Encode a Uint8Array to a Uint8Array using Z855.
 *
 * The encoder attempts to identify "safe" byte sequences (readable ASCII strings)
 * and pass them through unencoded using escape prefixes. Non-safe bytes are
 * encoded using standard Z85.
 *
 * Passthrough cases (tried in order):
 *   (A) Long passthrough (8+ bytes, block-aligned only)
 *       • Includes special case: 0| for "rest of input" (non-concatenatable mode)
 *   (B) Extended passthrough (5, 6, or 7 bytes, any alignment)
 *       • Uses P+1 high-order digits for unique reconstruction
 *   (C) Block-aligned 4-byte passthrough
 *       • Simple case: _[4 bytes]
 *   (D) Non-aligned 4-byte passthrough
 *       • Requires canonical-minimum check
 *       • Consumes 8 bytes total (before + after blocks)
 *   (E) Standard Z85 encoding (fallback)
 *
 * @param original - Input bytes to encode
 * @param opts - Encoding options (see EncodeOptions interface)
 * @returns Encoded output as Uint8Array
 */
export function encode(
  original: Uint8Array,
  opts: EncodeOptions = CANONICAL_ENCODING,
): Uint8Array {
  const { concatenatable, extraSafeBytes, maxRawLength } = Object.assign(
    {},
    CANONICAL_ENCODING,
    opts,
  );

  // Build safe-byte lookup table (all Z85 chars + extra safe bytes).
  const safeBytes = new Array(256).fill(false);
  for (let byte of extraSafeBytes ?? []) {
    if (typeof byte === "string") {
      assert(byte.length === 1, "extra safe byte must be a single character");
      byte = byte.charCodeAt(0);
    }
    assert(Number.isInteger(byte) && Number.isFinite(byte) && byte >= 0 && byte <= 255,
      "extra safe byte must be a byte value");
    safeBytes[byte] = true;
  }
  for (const byte of Z85_VALUES_BYTES.keys()) safeBytes[byte] = true;

  // Determine which escape characters are enabled.
  const hasEscape4    = safeBytes[ESCAPE_4.charCodeAt(0)]    && maxRawLength >= 4;
  const hasEscape5    = safeBytes[ESCAPE_5.charCodeAt(0)]    && maxRawLength >= 5;
  const hasEscape6    = safeBytes[ESCAPE_6.charCodeAt(0)]    && maxRawLength >= 6;
  const hasEscape7    = safeBytes[ESCAPE_7.charCodeAt(0)]    && maxRawLength >= 7;
  const hasLongEscape = safeBytes[ESCAPE_MANY.charCodeAt(0)] && maxRawLength >= 8;

  // Allocate output buffer (may grow for 0| escape).
  let buf = new Uint8Array(Math.ceil(original.length / 4) * 5 + 16);
  let outOff = 0;   // bytes written so far into buf
  let inOff  = 0;   // bytes consumed from original

  function emit(b: number) {
    if (outOff >= buf.length) { const nb = new Uint8Array(buf.length * 2); nb.set(buf); buf = nb; }
    buf[outOff++] = b;
  }
  function emitStr(s: string)  { for (let i = 0; i < s.length; i++) emit(s.charCodeAt(i)); }
  function emitBytes(src: Uint8Array, start: number, len: number) {
    while (outOff + len > buf.length) { const nb = new Uint8Array(buf.length * 2); nb.set(buf); buf = nb; }
    buf.set(src.subarray(start, start + len), outOff);
    outOff += len;
  }

  // In concatenatable mode, reserve trailing 1-3 bytes to be encoded AFTER hash padding.
  // We also reserve one full 4-byte block ("safeZone") before the tail to ensure the
  // main loop always ends at a 5-aligned output boundary, even when passthrough escapes
  // shift the output length. The safeZone block is always encoded as plain Z85, so the
  // last 5 chars before the hash+tail block are guaranteed Z85.
  let stopAt = original.length;
  let reservedTail = 0;
  if (concatenatable) {
    const hypLen = z855OutputLen(original.length);
    const rem = hypLen % 5;
    if (rem > 0 && rem !== 1) {
      reservedTail = rem - 1;
      stopAt = original.length - reservedTail;
    }
  }

  mainLoop:
  while (inOff < stopAt) {
    const remaining = stopAt - inOff;
    const isFullBlock = remaining >= BLOCK_SIZE_ORIGINAL;

    if (!isFullBlock) {
      // Trailing partial block (1–3 bytes), no passthrough possible.
      const numBytes = remaining;
      let pv = 0;
      for (let k = 0; k < numBytes; k++) pv = pv * 256 + original[inOff + k];
      const pd = encodePartial(pv, numBytes + 1);
      for (let k = 0; k < pd.length; k++) emit(pd[k]);
      inOff += numBytes;
      break;
    }

    // Read current 4-byte block and compute Z85 encoding.
    const blockValue = bytesToValue([
      original[inOff], original[inOff+1], original[inOff+2], original[inOff+3],
    ]);
    const blockDigits = valueToDigits(blockValue);

    // Quick rejection: if last byte of block isn't safe, skip passthrough logic.
    // Also skip passthrough for the last 2 blocks before stopAt in concatenatable
    // mode: this ensures the last chars before the reserved-tail splice are always
    // Z85 digits, making the splice safe (no escape chars in the spliced tail).
    if (!safeBytes[original[inOff + 3]] || (concatenatable && stopAt - inOff <= 8)) {
      for (let k = 0; k < 5; k++) emit(blockDigits[k]);
      inOff += 4;
      continue;
    }

    // Count safe bytes at end of current block (scanning backward).
    let safeBytesAtEnd = 0;
    for (let j = 3; j >= 0; j--) {
      if (safeBytes[original[inOff + j]]) safeBytesAtEnd++;
      else break;
    }

    // Count safe bytes immediately following this block.
    const afterBlock = inOff + 4;
    const afterBlockEnd = Math.min(original.length, inOff + maxRawLength);
    let safeBytesFollowing = 0;
    for (let j = afterBlock; j < afterBlockEnd && safeBytesFollowing + safeBytesAtEnd < maxRawLength; j++) {
      if (safeBytes[original[j]]) safeBytesFollowing++;
      else break;
    }

    const safeLen = safeBytesAtEnd + safeBytesFollowing;
    const remainingAfterSafe = original.length - afterBlock - safeBytesFollowing;

    // ── (A) Long passthrough: 8+ bytes ──────────────────────────────────────
    //
    // Only applies to block-aligned runs (safeBytesAtEnd === 4).
    // Encodes using |: [offset-base42?][length-base42]|[padding][raw-bytes][padding]
    //
    // Special case: 0| (rest-of-input, non-concatenatable only).
    //
    // In concatenatable mode, cap the raw run at stopAt to avoid consuming
    // reserved-tail bytes. The reserved tail must be encoded as a separate
    // partial block (hash-padded) after the main loop.
    if (hasLongEscape && safeLen >= 8 && safeBytesAtEnd === 4) {
      const safeStart = inOff; // block-aligned
      const rawLen = concatenatable ? Math.min(safeLen, stopAt - inOff) : safeLen;

      // 0| rest-of-input escape: only when safe run reaches end of input.
      if (remainingAfterSafe === 0 && !concatenatable) {
        emitStr(Z85_DIGITS[0]); // '0' prefix digit
        emitStr(ESCAPE_MANY);
        emitBytes(original, safeStart, rawLen);
        inOff = original.length;
        return buf.subarray(0, outOff);
      }

      // Normal long escape with padding.
      const lenPrefix = encodeBase42(rawLen);
      const envelopeLen = z855OutputLen(rawLen);
      const ourLenNoPad = lenPrefix.length + 1 + rawLen;
      if (ourLenNoPad <= envelopeLen) {
        const paddingNeeded = envelopeLen - ourLenNoPad;

        // Find best alignment offset via bit-reversal sort key.
        // Offset is only encodable (and thus decodable) when rawLen > 15,
        // because the decoder distinguishes offset prefix from length prefix
        // only when the length requires multiple base-42 digits.
        const rawOffset = paddingNeeded >= 2
          ? findBestOffset(outOff, lenPrefix.length, rawLen, paddingNeeded)
          : 0;
        // Cap: total prefix (offset + length) must be < 5 digits, otherwise the
        // decoder triggers a Z85 block decode before seeing `|` (spurious block).
        const rawOffsetPrefix = (rawLen > 15 && rawOffset > 0) ? encodeBase42(rawOffset) : [];
        const offsetPrefix = (rawOffsetPrefix.length + lenPrefix.length < 5) ? rawOffsetPrefix : [];
        const offset = offsetPrefix.length > 0 ? rawOffset : 0;

        if (offsetPrefix.length + offset <= paddingNeeded) {
          const paddingAfter = paddingNeeded - offsetPrefix.length - offset;

          for (const c of offsetPrefix) emitStr(c);
          for (const c of lenPrefix)    emitStr(c);
          emitStr(ESCAPE_MANY);
          for (let k = 0; k < offset; k++) emit(0x2e);       // '.' padding before
          emitBytes(original, safeStart, rawLen);              // raw bytes
          for (let k = 0; k < paddingAfter; k++) emit(0x2e); // '.' padding after



          inOff = safeStart + rawLen;
          continue mainLoop;
        }
      }
    }

    // ── (B) Extended passthrough: 5, 6, or 7 bytes ──────────────────────────
    //
    // For each length K ∈ {7, 6, 5} (larger preferred):
    //   Try all positions p ∈ {0, 1, 2, 3}:
    //     p=0: block-aligned, emit [escape][K bytes]
    //     p≥1: non-aligned, emit [(p+1) Z85 chars][escape][K bytes]
    //   Pick candidate with best bit-reversal alignment score.
    //
    // No canonical-min check needed (P+1 digits uniquely determine the value).
    {
      let handledB = false;
      for (const K of [7, 6, 5]) {
        const escEnabled = (K === 7 && hasEscape7) || (K === 6 && hasEscape6) || (K === 5 && hasEscape5);
        if (!escEnabled || maxRawLength < K) continue;

        const escChar = K === 7 ? ESCAPE_7 : K === 6 ? ESCAPE_6 : ESCAPE_5;
        const totalRemaining = original.length - inOff;

        // Collect valid candidates (p, sortKey).
        const candidates: Array<{ p: number; key: [bigint, bigint] }> = [];
        for (let p = 0; p <= 3; p++) {
          const bytesConsumed = p + K;
          if (inOff + bytesConsumed > original.length) continue;
          // Length invariant: passthrough output + remaining output = total output.
          const passthroughOutputChars = (p === 0 ? 1 : p + 2); // escape-only or (p+1)+escape
          const remaining2 = totalRemaining - bytesConsumed;
          if (passthroughOutputChars + z855OutputLen(remaining2) !== z855OutputLen(totalRemaining)) continue;
          // Check K consecutive bytes are safe.
          if (!kBytesSafe(original, inOff + p, K, safeBytes)) continue;

          const start = inOff + p;
          const end   = start + K - 1;
          const rS = bitReverse(start), rE = bitReverse(end);
          const key: [bigint, bigint] = [rS < rE ? rS : rE, rS > rE ? rS : rE];
          candidates.push({ p, key });
        }
        if (candidates.length === 0) continue;
        candidates.sort((a, b) => cmp2(a.key, b.key));

        for (const { p } of candidates) {
          const passStart = inOff + p;
          if (p === 0) {
            // Block-aligned: just escape + K raw bytes.
            emitStr(escChar);
            emitBytes(original, passStart, K);
            inOff = passStart + K;
          } else {
            // Non-aligned: emit (p+1) before-block Z85 chars, escape, K raw bytes.
            for (let k = 0; k <= p; k++) emit(blockDigits[k]);
            emitStr(escChar);
            emitBytes(original, passStart, K);
            inOff = passStart + K;
          }
          handledB = true;
          break;
        }
        if (handledB) continue mainLoop;
      }
    }

    // ── (C) Block-aligned 4-byte passthrough ────────────────────────────────
    //
    // Simplest case: all 4 bytes of current block are safe.
    // Emit: _[4 raw bytes]
    if (hasEscape4 && safeBytesAtEnd === 4) {
      emitStr(ESCAPE_4);
      emitBytes(original, inOff, 4);
      inOff += 4;
      continue;
    }

    // ── (D) Non-aligned 4-byte passthrough ──────────────────────────────────
    //
    // The escape appears at position P ∈ {1, 2, 3} within the output block.
    // Emit: [P Z85 chars][_][4 raw bytes][remaining after-block chars]
    //
    // Constraints:
    //   • Before-block value must satisfy canonical-minimum rule
    //   • After-block value is reconstructed from P known high bytes + (5-P) digits
    //   • Requires full after-block (8 bytes total consumed)
    if (hasEscape4) {
      const totalRemaining = original.length - inOff;
      const candidates: Array<{ p: number; key: [bigint, bigint] }> = [];
      for (let p = 1; p <= 3; p++) {
        const passStart = inOff + p;
        if (passStart + 4 > original.length) continue;
        if (!kBytesSafe(original, passStart, 4, safeBytes)) continue;
        // Need full after-block (8 bytes total).
        if (inOff + 8 > original.length) continue;
        const start = passStart, end = passStart + 3;
        const rS = bitReverse(start), rE = bitReverse(end);
        const key: [bigint, bigint] = [rS < rE ? rS : rE, rS > rE ? rS : rE];
        candidates.push({ p, key });
      }
      candidates.sort((a, b) => cmp2(a.key, b.key));

      for (const { p } of candidates) {
        const passStart = inOff + p;
        const numKnownLow = 4 - p;
        const knownLow = Array.from(original.subarray(passStart, passStart + numKnownLow));

        // Check canonical minimum constraint.
        if (!isCanonMin(blockValue, p, knownLow)) continue;

        // Emit P before-block Z85 chars.
        for (let k = 0; k < p; k++) emit(blockDigits[k]);
        // Emit escape + 4 raw bytes.
        emitStr(ESCAPE_4);
        emitBytes(original, passStart, 4);

        // Reconstruct after-block value and emit its last (5-P) Z85 chars.
        const afterBytes: number[] = [];
        for (let k = 0; k < p; k++) afterBytes.push(original[passStart + numKnownLow + k]);
        for (let k = 0; k < 4 - p; k++) afterBytes.push(original[inOff + 4 + p + k]);
        const afterVal = bytesToValue(afterBytes);
        const afterDig = valueToDigits(afterVal);
        for (let k = p; k < 5; k++) emit(afterDig[k]);

        inOff += 8;
        continue mainLoop;
      }
    }

    // ── (E) Standard Z85 ────────────────────────────────────────────────────
    // No passthrough possible; encode as standard Z85.
    for (let k = 0; k < 5; k++) emit(blockDigits[k]);
    inOff += 4;
  }

  // Concatenatable mode: insert hash padding before any reserved tail bytes.
  if (concatenatable && reservedTail > 0) {
    // Partial-block encoding: emit [###...][partial Z85 chars] in a 5-char block.
    // If the loop left output at a non-5-aligned offset, first splice hashes before
    // the dangling bytes to complete that block (same as the else-if branch below),
    // then emit the reserved-tail partial block.
    const remBefore = outOff % 5;
    if (remBefore > 0) {
      const hashCount = 5 - remBefore;
      const tail = buf.slice(outOff - remBefore, outOff);
      outOff -= remBefore;
      for (let k = 0; k < hashCount; k++) emit(PAD_HASH);
      for (let k = 0; k < tail.length; k++) emit(tail[k]);
    }
    const partialChars = reservedTail + 1;
    const hashCount = 5 - partialChars;
    for (let k = 0; k < hashCount; k++) emit(PAD_HASH);
    let pv = 0;
    for (let k = 0; k < reservedTail; k++) pv = pv * 256 + original[stopAt + k];
    const pd = encodePartial(pv, partialChars);
    for (let k = 0; k < pd.length; k++) emit(pd[k]);
  } else if (concatenatable) {
    // No reserved tail but output might not be 5-aligned: insert hash padding.
    const rem = outOff % 5;
    if (rem > 0) {
      const hashCount = 5 - rem;
      // Splice hash padding before the last `rem` bytes of output.
      const tail = buf.slice(outOff - rem, outOff);
      outOff -= rem;
      for (let k = 0; k < hashCount; k++) emit(PAD_HASH);
      for (let k = 0; k < tail.length; k++) emit(tail[k]);
    }
  }

  return buf.subarray(0, outOff);
}

// ─── Decoder helpers ───

/**
 * Read one base-42 self-terminating number from a digit array, scanning right-to-left.
 *
 * Base-42 digits use continuation bits:
 *   • digit < 42: represents itself, STOP
 *   • digit ≥ 42: represents (digit - 42), CONTINUE to next digit
 *
 * @param digits - Array of Z85 digit indices
 * @param end - Position to start scanning from (exclusive, scan backwards)
 * @returns { value, count } where count = number of digits consumed
 */
function readBase42RTL(digits: number[], end: number): { value: number; count: number } {
  let value = 0, multiplier = 1, pos = end, count = 0;
  while (pos > 0) {
    const d = digits[--pos];
    count++;
    if (d >= 42) {
      // Continuation bit set: add (d - 42) and continue.
      value += (d - 42) * multiplier;
      multiplier *= 42;
    } else {
      // Stop bit: add d and stop.
      value += d * multiplier;
      break;
    }
  }
  return { value, count };
}

/**
 * Decode base-42 prefix digits before a '|' character.
 *
 * Scans right-to-left from end of digit array:
 *   1. Read length (always present)
 *   2. If digits remain, read offset (for length > 15)
 *
 * @param digits - Z85 digit indices before '|'
 * @returns { offset, length }
 */
function decodeLongPrefix(digits: number[]): { offset: number; length: number } {
  const { value: length, count } = readBase42RTL(digits, digits.length);
  if (count === digits.length) return { offset: 0, length };
  const { value: offset } = readBase42RTL(digits, digits.length - count);
  return { offset, length };
}

/**
 * Reconstruct an after-block 32-bit value from known high bytes and low Z85 digits.
 *
 * After a non-aligned 4-byte passthrough, we know:
 *   • P high bytes of the after-block (from the passthrough)
 *   • (5-P) low Z85 digit indices (from following chars)
 *
 * This function solves for the unique 32-bit value satisfying both constraints.
 *
 * Algorithm:
 *   1. Pack P known high bytes into highPart
 *   2. Compute range of values with those high bytes: [rangeStart, ...)
 *   3. Use modular arithmetic to find value in range matching low digits
 *
 * @param knownHighBytes - P known high bytes (big-endian)
 * @param lowDigitsVal - Value accumulated from (5-P) low Z85 digits
 * @param numLowDigits - Number of low digits (5-P)
 * @returns 32-bit value (unsigned)
 */
function reconstructAfterBlock(knownHighBytes: number[], lowDigitsVal: number, numLowDigits: number): number {
  const P = knownHighBytes.length;
  let highPart = 0;
  for (const b of knownHighBytes) highPart = highPart * 256 + b;
  const shift = 8 * (4 - P);
  const rangeStart = (highPart << shift) >>> 0;
  const modulus = Math.pow(85, numLowDigits);
  const rem = rangeStart % modulus;
  let candidate = rem <= lowDigitsVal
    ? rangeStart - rem + lowDigitsVal
    : rangeStart - rem + modulus + lowDigitsVal;
  return candidate >>> 0;
}

/**
 * Decode a Uint8Array to a Uint8Array using Z855.
 *
 * The decoder handles all escape types automatically:
 *   • Hash padding (#, concatenatable mode)
 *   • Long escape (|)
 *   • Short escapes (_, ,, ~, ;)
 *   • Standard Z85 characters
 *
 * Decoding is stateful:
 *   • `digits` accumulates Z85 digit indices for the current block
 *   • `knownHighBytes` tracks known high bytes after a non-aligned 4-byte passthrough
 *
 * @param encoded - Z855-encoded input
 * @returns Decoded bytes
 */
export function decode(encoded: Uint8Array): Uint8Array {
  if (encoded.length === 0) return new Uint8Array(0);

  const out: number[] = [];
  let i = 0;

  // State: accumulated Z85 digit indices for current block.
  let digits: number[] = [];
  // State: after non-aligned 4-byte passthrough, P high bytes of after-block are known.
  let knownHighBytes: number[] = [];

  while (i < encoded.length) {
    const code = encoded[i];

    // ── Hash padding (concatenatable mode) ────────────────────────────────────
    // Format: ###[2-4 Z85 chars] (total 5 chars, aligned to block boundary)
    if (code === PAD_HASH && digits.length === 0 && i % 5 === 0 && encoded.length - i >= 5) {
      let h = 0;
      while (h < 3 && encoded[i + h] === PAD_HASH) h++;
      if (h > 0 && encoded[i + h] !== PAD_HASH) {
        const numChars = 5 - h;
        const numBytes = numChars - 1;
        let value = 0;
        for (let k = 0; k < numChars; k++) {
          const d = Z85_VALUES_BYTES.get(encoded[i + h + k]);
          if (d === undefined) throw new Error(`invalid char in hash block`);
          value = value * 85 + d;
        }
        const maxVal = [0, 0xff, 0xffff, 0xffffff][numBytes];
        if (value > maxVal) throw new Error("overflow in hash block");
        for (let k = numBytes - 1; k >= 0; k--) out.push((value >>> (k * 8)) & 0xff);
        i += 5;
        digits = []; knownHighBytes = [];
        continue;
      }
    }

    // ── Long escape '|' ───────────────────────────────────────────────────────
    // Format: [prefix-digits]|[padding][raw-bytes][padding]
    // Special case: 0| = rest of input is raw
    if (code === ESCAPE_MANY.charCodeAt(0)) {
      if (digits.length === 0) throw new Error("'|' with no prefix digits");
      i++; // consume '|'

      const { offset, length: rawLen } = decodeLongPrefix(digits);
      if (rawLen >= 1 && rawLen <= 7) throw new Error(`invalid | length ${rawLen}`);

      if (rawLen === 0) {
        // 0| — rest of input is raw
        for (; i < encoded.length; i++) out.push(encoded[i]);
        digits = []; knownHighBytes = [];
        break;
      }

      // length >= 8: rawLen bytes follow, with padding around them.
      const prefixLen = digits.length;
      const envelopeLen = z855OutputLen(rawLen);
      const paddingTotal = envelopeLen - prefixLen - 1 - rawLen;
      const paddingAfter = paddingTotal - offset;
      if (paddingAfter < 0) throw new Error("invalid | offset");

      i += offset; // skip padding-before (content irrelevant)
      if (i + rawLen > encoded.length) throw new Error("truncated | escape");
      for (let k = 0; k < rawLen; k++) out.push(encoded[i + k]);
      i += rawLen;
      i += paddingAfter; // skip padding-after

      // Consume any '#' alignment padding following a long escape in non-concatenatable
      // output (none expected in concatenatable mode since long escape is disabled there).
      while (i < encoded.length && encoded[i] === PAD_HASH) i++;

      digits = []; knownHighBytes = [];
      continue;
    }

    // ── Short passthrough escapes (4, 5, 6, 7 bytes) ─────────────────────────
    const escCode = code;
    let passLen = 0;
    if (escCode === ESCAPE_4.charCodeAt(0)) passLen = 4;
    else if (escCode === ESCAPE_5.charCodeAt(0)) passLen = 5;
    else if (escCode === ESCAPE_6.charCodeAt(0)) passLen = 6;
    else if (escCode === ESCAPE_7.charCodeAt(0)) passLen = 7;

    if (passLen > 0) {
      if (i + passLen >= encoded.length) throw new Error("incomplete passthrough");
      const pass: number[] = [];
      for (let k = 1; k <= passLen; k++) pass.push(encoded[i + k]);

      if (passLen === 4) {
        // ── ESCAPE_4: 4-byte passthrough ──
        const P = digits.length; // 0 = block-aligned, 1-3 = non-aligned

        if (P === 0) {
          // Block-aligned: just output the 4 bytes.
          out.push(...pass);
          i += 5; // escape + 4 bytes
        } else {
          // Non-aligned: P high-order Z85 digits + (4-P) known low bytes.
          const numKnownLow = 4 - P;
          const knownLow = pass.slice(0, numKnownLow);
          const beforeVal = canonicalMin(digits, knownLow);
          if (beforeVal < 0) throw new Error("non-aligned passthrough: invalid before-block");

          out.push(...valueToBytes(beforeVal));
          knownHighBytes = pass.slice(numKnownLow); // last P bytes become known high for after-block
          digits = [];
          i += 5; // escape + 4 bytes
        }
      } else {
        // ── ESCAPE_5/6/7: extended passthrough ──
        // Structure: [(P+1) Z85 chars][escape][K bytes] or [escape][K bytes] (block-aligned)

        if (digits.length === 0) {
          // Block-aligned: no preceding chars, just output the K bytes.
          out.push(...pass);
          i += 1 + passLen;
          continue;
        }

        // Non-aligned: digits.length == P+1
        const numDigits = digits.length; // P+1
        const P = numDigits - 1;
        const numKnownLow = 4 - P;
        const knownLow = pass.slice(0, numKnownLow);

        // Use extendedBlockValue (P+1 digits fully determine value).
        const beforeVal = extendedBlockValue(digits, knownLow);
        if (beforeVal < 0) throw new Error("extended passthrough: invalid before-block");

        // Output first P bytes of before-block (last byte is in passthrough).
        const bBytes = valueToBytes(beforeVal);
        for (let k = 0; k < P; k++) out.push(bBytes[k]);
        // Output all K passthrough bytes.
        out.push(...pass);

        digits = []; knownHighBytes = [];
        i += 1 + passLen;
      }
      continue;
    }

    // ── Regular Z85 character ─────────────────────────────────────────────────
    const d = Z85_VALUES_BYTES.get(code);
    if (d === undefined) throw new Error(`invalid Z85 char 0x${code.toString(16).toUpperCase()}`);
    digits.push(d);
    i++;

    // Check if we've completed a block.
    // Normal: need 5 digits. After non-aligned 4-byte passthrough: need (5-P) digits.
    const needed = 5 - knownHighBytes.length;
    if (digits.length === needed) {
      const raw = digitsToValue(digits);
      if (raw === null) throw new Error("Z85 value overflow");

      let value: number;
      if (knownHighBytes.length === 0) {
        // Normal Z85 block.
        value = raw;
      } else {
        // After non-aligned 4-byte passthrough: reconstruct from P known high bytes + (5-P) digits.
        value = reconstructAfterBlock(knownHighBytes, raw, needed);
      }

      if (value > 0xffffffff) throw new Error("Z85 value overflow");
      out.push(...valueToBytes(value));
      digits = []; knownHighBytes = [];
    }
  }

  // ── Flush trailing partial block ──────────────────────────────────────────
  if (digits.length > 0) {
    if (digits.length === 1) throw new Error("invalid: single trailing Z85 char");
    const value = digitsToValue(digits);
    if (value === null) throw new Error("Z85 value overflow in partial block");
    const numBytes = digits.length - 1;
    const maxValue = [0, 0xff, 0xffff, 0xffffff][numBytes];
    if (value > maxValue) throw new Error("Z85 value overflow in partial block");
    for (let k = numBytes - 1; k >= 0; k--) out.push((value >>> (k * 8)) & 0xff);
  }

  return new Uint8Array(out);
}

/** Entry point for the command-line interface. */
export async function main() {
  // TODO: add encode-lines which uses PRINTABLE_ASCII_ENCODING and splits into
  // 80-character line-delimited blocks, and decode-lines which strips newlines
  // when decoding.
  if (Deno.args[0] === "encode") {
    const args = parseArgs(Deno.args.slice(1), {
      boolean: ["concatenatable"],
      negatable: ["concatenatable"],
      string: ["extra-safe-characters", "max-raw-length"],
      default: {
        concatenatable: CANONICAL_ENCODING.concatenatable,
        "extra-safe-characters": CANONICAL_ENCODING.extraSafeBytes,
        "max-raw-length": CANONICAL_ENCODING.maxRawLength.toString(),
      },
    });
    const opts = {
      concatenatable: args.concatenatable,
      extraSafeCharacters: args["extra-safe-characters"],
      maxRawLength: Number(args["max-raw-length"]),
    };
    assert(
      Number.isInteger(opts.maxRawLength),
      "max-raw-length must be an integer",
    );
    assert(
      Number.isFinite(opts.maxRawLength),
      "max-raw-length must be finite",
    );
    assert(
      opts.maxRawLength <= Number.MAX_SAFE_INTEGER,
      "max-raw-length must be less than or equal to 2^53 - 1",
    );
    assert(opts.maxRawLength > 0, "max-raw-length must be greater than 0");
    const stdin = await readAll(Deno.stdin);
    await Deno.stdout.write(encode(stdin, {
      concatenatable: args.concatenatable,
    }));
  } else if (Deno.args[0] === "decode") {
    const stdin = await readAll(Deno.stdin);
    await Deno.stdout.write(decode(stdin));
  } else if (Deno.args[0] === "shebang") {
    const stdin = await readAll(Deno.stdin);
    const encoded = textEncode(stdin, CANONICAL_ENCODING);
    const shebangScript = createShebangScript(encoded);
    await Deno.stdout.write(new TextEncoder().encode(shebangScript));
  } else if (Deno.args[0] === "shebang-decode") {
    const stdin = await readAll(Deno.stdin);
    const text = new TextDecoder().decode(stdin);

    // Find and extract the encoded data from the template literal
    const match = text.match(/const encoded = `([^`]+)`/);
    if (!match) {
      throw new Error("Could not find encoded data in shebang file");
    }

    const encodedData = match[1];
    const decoded = textDecode(encodedData);
    await Deno.stdout.write(decoded);
  } else {
    await Deno.stderr.write(new TextEncoder().encode(
      "Usage: z855 encode|decode|shebang|shebang-decode < input > output\n",
    ));
    return 2;
  }
}

// ─── Text encode/decode helpers ───

/** Encode a Uint8Array to a string using Z855. */
export function textEncode(
  original: Uint8Array,
  opts: EncodeOptions = CANONICAL_ENCODING,
): string {
  return new TextDecoder().decode(encode(original, opts));
}

/** Decode a string to a Uint8Array using Z855. */
export function textDecode(encoded: string): Uint8Array {
  return decode(new TextEncoder().encode(encoded));
}

// ─── Z85 conversion primitives ───

/**
 * Convert a 32-bit unsigned integer to 5 Z85 character bytes.
 *
 * Returns a Uint8Array of 5 bytes, each containing the ASCII code of a Z85 character.
 * The encoding is big-endian (most significant digit first).
 */
function valueToDigits(v: number): Uint8Array {
  const digits = new Uint8Array(5);
  for (let i = 4; i >= 0; i--) {
    digits[i] = Z85_DIGIT_BYTES[v % 85];
    v = Math.floor(v / 85);
  }
  return digits;
}

/**
 * Encode a partial-block value into exactly `numChars` Z85 character bytes.
 *
 * Used for encoding trailing 1-3 bytes as 2-4 Z85 characters.
 */
function encodePartial(v: number, numChars: number): Uint8Array {
  const out = new Uint8Array(numChars);
  for (let i = numChars - 1; i >= 0; i--) {
    out[i] = Z85_DIGIT_BYTES[v % 85];
    v = Math.floor(v / 85);
  }
  return out;
}

/**
 * Convert Z85 digit indices (2-5 of them) to a 32-bit value.
 *
 * Returns the value if it fits in 32 bits, or null if overflow.
 */
function digitsToValue(digits: number[]) {
  let v = 0;
  for (let i = 0; i < digits.length; i++) {
    v = v * 85 + digits[i];
  }
  return v <= 0xFFFFFFFF ? v : null;
}

/** Convert a 32-bit unsigned integer to 4 bytes (big-endian). */
function valueToBytes(v: number) {
  return [
    (v >>> 24) & 0xFF,
    (v >>> 16) & 0xFF,
    (v >>> 8) & 0xFF,
    v & 0xFF,
  ];
}

/** Convert 4 bytes (big-endian) to a 32-bit unsigned integer. */
function bytesToValue(b: number[]) {
  return ((b[0] << 24) | (b[1] << 16) | (b[2] << 8) | b[3]) >>> 0;
}

if (import.meta.main) {
  Deno.exit(await main());
}
