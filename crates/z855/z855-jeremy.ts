/**
Z855 is a superset of [standard Z85]. The encoder supports several different
options, but they are all described in-band; the decoder SHOULD handle decoding
values encoded with any encoder options without accepting any options itself,
unless otherwise required to enforce a contextual determinism requirement. This
implicitly includes support for decoding standard Z85. Two standard sets of
options are defined, and other Z855 implementation SHOULD attempt to provide
support for them for consistency, but this is not required for interoperability
in most contexts. These sets of options are referred to as "Canonical", the
typical default, and "Concatenatable", which is slightly messier but allows
multiple encoded values to be concatenated together and produce valid results.

```
0123456 789abcd efghijk      01234 56789 abcde fghij
lmnopqr stuvwxy zABCDEF      klmno pqrst uvwxy zABCD
GHIJKLM NOPQRST UVWXYZ.      EFGHI JKLMN OPQRS TUVWX
-:+=^!/ *?&<>() []{}@%$      YZ.-: +=^!/ *?&<> ()[]{
#                 _,~;|      }@%$#             _,~;|
```

Z85 is a base-85 encoding scheme with an alphabet chosen avoid the need for
escaping in as many source code or configuration contexts as possible given
the large alphabet size. It encodes 4 bytes input blocks into 5 characters each,
compared to base64 encoding 3 bytes input blocks into 4 characters. This has
the advantages over base64 that 4 bytes (32 bits) lines up with real-world data
structures much more often than 3 bytes (24 bits), that it's more compact, and
that 64 bytes of data (another nicely-aligned value) fits exactly into standard
80-character lines, and that it's easier to see zero values and some
single-digit integers due to the alphabet starting with `0` instead of `A`.
It has the disadvantages of requiring a larger less-compatible alphabet, and
being much less efficient to encode and decode, requiring division and overflow
checks where base64 can use fast and infallible bit shifts.

[Z85]: https://rfc.zeromq.org/spec/32/

Z855 adds a lot of complexity, sacrifices a lot of performance, and expands
the alphabet with up to 5 more characters (`,`, `;`, `_`, `~`, and `|`),
for the benefit of allowing "safe" values (at minimum: most strings of length 4
or greater which are made up of the 85 + 5 characters used by Z855) to be
escaped and passed-through raw, to improve the human- and agent-readability
of the encoded data, and making it possible for many significant strings to
show up in searches without handling (or even being aware of) the encoding.
The encoding scheme is chosen to ensure that any normal Z85 blocks, which are
not escaped and passed-through raw, will always appear in the exact same
location in the output data as they would with standard Z85. Maintaining
alignment consistent also helps to improve the readability of diffs of Z855-
encoded data (at least when the changes are clean enough for a naive
text-oriented diff of binary data to possibly be useful at all).

Standard Z85 only supports input which is a multiple of 4 bytes. Z855 also
supports the common variable-length extension where we emit an output that is
not a multiple of 5 bytes. However, that breaks the ability to concatenate
multiple encoded values together, so with Concatenatable options Z855 will
instead pad out the encoded final block to a multiple of 5 characters, to
disambiguate the start position of the next encoded value. Instead of padding
the end of the string with a special character (such as `=` in base64), we
instead pad the beginning of the string with `#`, which is a valid Z85 character
but will never appear at the beginning an encoded Z85 block (not full 4-byte
blocks nor truncated 1-, 2-, or 3-byte blocks) so it can be cleanly detected and
removed without expanding the alphabet. For example, a single byte with the
value 0x07 can be encoded under Canonical options as `07` or under
Concatenatable options as `###07`. This capability doesn't have anything to
do with readability of the encoded data, it's just an affordance to allow this
encoding to be used in more contexts.

Raw/passthrough values require the use of our 5 new escape characters (which
are all enabled in both standard option sets, but encoders may support
enabling or disabling depending on requirements). Each escape character
indicates that the next N bytes of input data will be included in the output
data as-is, without any encoding but with a prefix and in some cases padding.
(Decoders do not care about the values of padding bytes, only their locations,
so the specific padding bytes described below are just what's used by our
canonical and concatenatable option sets.) The exact use and interpretation of
these sequences in different locations will be defined after, but the escape
prefixes are:

- `_`: escape 4 bytes
- `,`: escape 5 bytes
- `~`: escape 6 bytes
- `;`: escape 7 bytes
- `8|`...`A|`: escape 8...11 bytes
- `9|`...`C|`: escape 12...15 bytes with a single padding byte (typically `.`)
   at the end.
- `0D|`...: escape 16 or more bytes. The prefix before the `|` indicates both
   the length of the raw data and the location of the raw data within the
   available space for the escape block (the rest is padding, typically `.`).
   The length and offset are determined by scanning backwards from a `|`
   character. The first thing you encounter will be a digit of the length. These
   use the first 84 digits of the Z85 alphabet, but represent a value in base
   42 (0–41): a digit with a value in the range 0-41 represents itself, while a
   digit with a value in the range 42-84 represents the value of the digit minus
   42, but also signals that this value continues into the next digit, allowing
   for arbitrarily-large escapes (with the only limit imposed by the format
   being that the length must be not be greater than JavaScript's
   `MAX_SAFE_INTEGER`, and the offset (where `0` is the byte after `|`) must fit
   within the available space for for the escape (which is determined based on
   its size). Once the end of the length is found, if the length is greater than
   15 then we continue back and repeat the process to get the offset which is
   encoded in the same way. (For sizes between 8 and 15 there's no space to have
   an offset we must not scan for one. In that case, the effective offset is 0.)
- `0|`: escape all remaining bytes (no padding is used, this is the only case
  where the output string may be shorter than the standard Z85 encoding). This
  must not be used in Concatenatable mode.



Encoders may choose to allow mark specific *sequences of bytes* to be declared
as "unsafe", and avoid generating any raw escapes with them, even if the
characters are safe in other contexts. For example, the three-backtick sequence
could be marked unsafe to ensure that the escaped content is safe to embed in a
markdown code block.

This is straightforward if the start and end align with block boundaries, but
can be tricky if they don't!

with the raw value included at a location indicated
  by the first prefix byte (scheme described below)

a padding byte at the location specified
   at the beginning of the prefix (explained below).

-


If the file ends non-blocked-aligned, with a raw escape at the end, in
Concatenatable mode, the padding will appear _after_ the raw escape, not
interrupt it.

Maybe we could support use as a shebang which extracts to a temporary file
and runs it, and use that as a pseudo-binary build target? In Rust we could
have our target/debug/foo.855 next to target/debug/foo or wherever the
binaries go. We could check if the existing file in that path matches the
data we decode, and if so we don't even attempt to open it for writing, we
just attempt to execute it (setting the executable bit if we need to).
Maybe we could even have some literate option where ``` code blocks are
interwoven with encoded data, but for that to be the case, the first
non-empty non-shebang line of the file must start with one or more `#`
followed by a space character (a markdown header, which can't appear
exactly like that in real Z855 data), in which case we ignore everything
outside of pairs of lines that start with the same three-or-more number of
backtick characters (fenced code blocks).

Or we could code golf and Terser down the decoder (only the decoder) into
a self-extracting Deno shim which does effectively the same thing, but
all as a standalone header to the Z855 executable data.
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
 * Positions aligned to power-of-2 boundaries have trailing zeros;
 * bit-reversal puts those as leading zeros so they sort first.
 */
function bitReverse(n: number): bigint {
  let x = BigInt(n);
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
 * Given P high-order Z85 digit indices and (4-P) known low bytes,
 * return the minimum 32-bit value whose Z85 encoding starts with those
 * digits and whose low bytes match.  Returns -1 if none exists.
 */
function canonicalMin(highDigits: number[], knownLowBytes: number[]): number {
  const P = highDigits.length;
  const numKnown = knownLowBytes.length; // = 4 - P

  let base = 0;
  for (const d of highDigits) base = base * 85 + d;

  const power = Math.pow(85, 5 - P);
  const rangeStart = base * power;
  const rangeEnd   = (base + 1) * power;

  if (numKnown === 0) {
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
 * Given (P+1) high-order Z85 digit indices and (4-P) known low bytes,
 * return the unique 32-bit value that satisfies both constraints.
 * Returns -1 if none exists.
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

/** Return the first P Z85 digit indices for a 32-bit value. */
function highDigits(v: number, P: number): number[] {
  const all: number[] = new Array(5);
  let x = v;
  for (let i = 4; i >= 0; i--) { all[i] = x % 85; x = Math.floor(x / 85); }
  return all.slice(0, P);
}

/** Check if value is the canonical minimum for position P and the given passthrough bytes. */
function isCanonMin(value: number, P: number, passBytesLow: number[]): boolean {
  return canonicalMin(highDigits(value, P), passBytesLow) === value;
}

// ─── Long-escape prefix encoding ───

/** Encode a single non-negative integer as base-42 Z85 chars (MSB first, continuation bits). */
function encodeBase42(n: number): string[] {
  if (n < 42) return [Z85_DIGITS[n]];
  const digits: number[] = [];
  let v = n;
  while (v > 0) { digits.push(v % 42); v = Math.floor(v / 42); }
  digits.reverse();
  return digits.map((d, i) => Z85_DIGITS[i === 0 ? d : d + 42]);
}

/**
 * Find the best offset for a long-escape block using the bit-reversal sort key.
 * We pick the offset whose sort key (4-tuple of bigints) is lexicographically smallest.
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
    const offsetPrefixLen = offset > 0 ? encodeBase42(offset).length : 0;
    if (offsetPrefixLen + offset > paddingNeeded) continue;

    const outputStart = currentOutputLen + offsetPrefixLen + lengthPrefixLen + 1 + offset;
    const outputEnd   = outputStart + rawLen - 1;
    const inputStart  = Math.floor(outputStart * 4 / 5);
    const inputEnd    = Math.floor(outputEnd   * 4 / 5);

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

function kBytesSafe(input: Uint8Array, start: number, k: number, safe: boolean[]): boolean {
  if (start + k > input.length) return false;
  for (let i = 0; i < k; i++) if (!safe[input[start + i]]) return false;
  return true;
}

// ─── Encoder ───

/** Encode a Uint8Array to a Uint8Array using Z855. */
export function encode(
  original: Uint8Array,
  opts: EncodeOptions = CANONICAL_ENCODING,
): Uint8Array {
  const { concatenatable, extraSafeBytes, maxRawLength } = Object.assign(
    {},
    CANONICAL_ENCODING,
    opts,
  );

  // Derive feature flags
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

  const hasEscape4    = safeBytes[ESCAPE_4.charCodeAt(0)]    && maxRawLength >= 4;
  const hasEscape5    = safeBytes[ESCAPE_5.charCodeAt(0)]    && maxRawLength >= 5;
  const hasEscape6    = safeBytes[ESCAPE_6.charCodeAt(0)]    && maxRawLength >= 6;
  const hasEscape7    = safeBytes[ESCAPE_7.charCodeAt(0)]    && maxRawLength >= 7;
  const hasLongEscape = safeBytes[ESCAPE_MANY.charCodeAt(0)] && maxRawLength >= 8;

  // Allocate output buffer; may be grown for the 0| escape.
  let buf = new Uint8Array(Math.ceil(original.length / BLOCK_SIZE_ORIGINAL) * 5 + 16);
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

  // In concatenatable mode we stop before the last 1-3 bytes so we can
  // insert hash padding right before the partial block.
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
      // Trailing partial block (1–3 bytes), no passthrough.
      const numBytes = remaining;
      let pv = 0;
      for (let k = 0; k < numBytes; k++) pv = pv * 256 + original[inOff + k];
      const pd = encodePartial(pv, numBytes + 1);
      for (let k = 0; k < pd.length; k++) emit(pd[k]);
      inOff += numBytes;
      break;
    }

    const blockValue = bytesToValue([
      original[inOff], original[inOff+1], original[inOff+2], original[inOff+3],
    ]);
    const blockDigits = valueToDigits(blockValue);

    // Skip passthrough logic if last byte of block isn't safe.
    if (!safeBytes[original[inOff + 3]]) {
      for (let k = 0; k < 5; k++) emit(blockDigits[k]);
      inOff += BLOCK_SIZE_ORIGINAL;
      continue;
    }

    // Count safe bytes at end of current block (1–4).
    let safeBytesAtEnd = 0;
    for (let j = 3; j >= 0; j--) {
      if (safeBytes[original[inOff + j]]) safeBytesAtEnd++;
      else break;
    }

    // Count safe bytes immediately following this block (capped at maxRawLength).
    const afterBlock = inOff + BLOCK_SIZE_ORIGINAL;
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
    // Only block-aligned (safeBytesAtEnd === BLOCK_SIZE_ORIGINAL): all bytes of this block are
    // safe, so the raw run starts at inOff.  Non-aligned cases fall through to (B).
    //
    // Special case: 0| (rest-of-input, non-concatenatable only).
    if (hasLongEscape && safeLen >= 8 && safeBytesAtEnd === BLOCK_SIZE_ORIGINAL) {
      const safeStart = inOff; // block-aligned
      const rawLen = safeLen;

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
        const offset = paddingNeeded >= 2
          ? findBestOffset(outOff, lenPrefix.length, rawLen, paddingNeeded)
          : 0;
        const offsetPrefix = (rawLen > 15 && offset > 0) ? encodeBase42(offset) : [];

        if (offsetPrefix.length + offset <= paddingNeeded) {
          const paddingAfter = paddingNeeded - offsetPrefix.length - offset;

          for (const c of offsetPrefix) emitStr(c);
          for (const c of lenPrefix)    emitStr(c);
          emitStr(ESCAPE_MANY);
          for (let k = 0; k < offset; k++) emit(0x2e);       // padding before
          emitBytes(original, safeStart, rawLen);              // raw bytes
          for (let k = 0; k < paddingAfter; k++) emit(0x2e); // padding after

          inOff = safeStart + rawLen;
          continue mainLoop;
        }
      }
    }

    // ── (B) Extended passthrough: 5, 6, or 7 bytes ──────────────────────────
    //
    // For each length K (7 preferred, then 6, then 5):
    //   Try all positions p=0..3 (where p=0 is block-aligned).
    //   For p=0: just emit escape + K raw bytes (no before-block chars needed,
    //            but must satisfy length invariant).
    //   For p=1..3: emit (p+1) before-block Z85 chars, escape, K raw bytes.
    //               No canonical-min check needed (P+1 chars fully disambiguate).
    //
    // Pick the candidate with the best bit-reversal sort key.
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
          // Length invariant: (p+1 + 1 + K) + z855Len(remaining) == z855Len(total)
          const passthroughOutputChars = (p === 0 ? 1 : p + 2); // escape only (p=0) or (p+1)+escape
          const remaining2 = totalRemaining - bytesConsumed;
          if (passthroughOutputChars + z855OutputLen(remaining2) !== z855OutputLen(totalRemaining)) continue;
          // Check K bytes are safe.
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
            // Non-aligned: emit (p+1) before-block high-order Z85 chars, escape, K raw bytes.
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
    if (hasEscape4 && safeBytesAtEnd === BLOCK_SIZE_ORIGINAL) {
      emitStr(ESCAPE_4);
      emitBytes(original, inOff, BLOCK_SIZE_ORIGINAL);
      inOff += BLOCK_SIZE_ORIGINAL;
      continue;
    }

    // ── (D) Non-aligned 4-byte passthrough ──────────────────────────────────
    //
    // The comma appears at position P (1–3) within the 5-char output block.
    // The block value must be the canonical minimum for the given P digits +
    // (4-P) known low bytes, so the decoder can recover it unambiguously.
    // After the passthrough, the "after" block's first P bytes are known;
    // we emit the remaining (5-P) Z85 digits of the after block.
    // Requires a complete after block (8 bytes total consumed).
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

        // Reconstruct after-block value and emit its last (5-p) Z85 chars.
        const afterBytes: number[] = [];
        for (let k = 0; k < p; k++) afterBytes.push(original[passStart + numKnownLow + k]);
        for (let k = 0; k < BLOCK_SIZE_ORIGINAL - p; k++) afterBytes.push(original[inOff + BLOCK_SIZE_ORIGINAL + p + k]);
        const afterVal = bytesToValue(afterBytes);
        const afterDig = valueToDigits(afterVal);
        for (let k = p; k < 5; k++) emit(afterDig[k]);

        inOff += 8;
        continue mainLoop;
      }
    }

    // ── (E) Standard Z85 ────────────────────────────────────────────────────
    for (let k = 0; k < 5; k++) emit(blockDigits[k]);
    inOff += BLOCK_SIZE_ORIGINAL;
  }

  // Concatenatable mode: insert hash padding before any reserved tail bytes.
  if (concatenatable && reservedTail > 0) {
    const remBefore = outOff % 5;
    const hashCount = remBefore === 0 ? 5 - (reservedTail + 1) : 0;
    for (let k = 0; k < hashCount; k++) emit(PAD_HASH);
    let pv = 0;
    for (let k = 0; k < reservedTail; k++) pv = pv * 256 + original[stopAt + k];
    const pd = encodePartial(pv, reservedTail + 1);
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

/** Read one base-42 self-terminating number from digits[0..end], right-to-left. */
function readBase42RTL(digits: number[], end: number): { value: number; count: number } {
  let value = 0, multiplier = 1, pos = end, count = 0;
  while (pos > 0) {
    const d = digits[--pos];
    count++;
    if (d >= 42) {
      value += (d - 42) * multiplier;
      multiplier *= 42;
    } else {
      value += d * multiplier;
      break;
    }
  }
  return { value, count };
}

/** Decode base-42 prefix digits before a '|': returns { offset, length }. */
function decodeLongPrefix(digits: number[]): { offset: number; length: number } {
  const { value: length, count } = readBase42RTL(digits, digits.length);
  if (count === digits.length) return { offset: 0, length };
  const { value: offset } = readBase42RTL(digits, digits.length - count);
  return { offset, length };
}

/**
 * Reconstruct an after-block value from P known high bytes and (5-P) low Z85 digits.
 * knownHighBytes: the P bytes; lowDigitsVal: accumulated value of (5-P) low digits.
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

/** Decode a Uint8Array to a Uint8Array using Z855. */
export function decode(encoded: Uint8Array): Uint8Array {
  if (encoded.length === 0) return new Uint8Array(0);

  const out: number[] = [];
  let i = 0;

  // Accumulated Z85 digit indices for the current block.
  let digits: number[] = [];
  // After a non-aligned 4-byte passthrough, the P high bytes of the "after" block are known.
  let knownHighBytes: number[] = [];

  while (i < encoded.length) {
    const code = encoded[i];

    // ── Hash padding (concatenatable mode) ────────────────────────────────────
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

      i += offset; // skip padding-before (content irrelevant, typically '.')
      if (i + rawLen > encoded.length) throw new Error("truncated | escape");
      for (let k = 0; k < rawLen; k++) out.push(encoded[i + k]);
      i += rawLen;
      i += paddingAfter; // skip padding-after

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
        const P = digits.length; // 0 = block-aligned

        if (P === 0) {
          // Block-aligned: just output the 4 bytes.
          out.push(...pass);
          i += 5; // escape + 4 bytes
        } else {
          // Non-aligned: P high-order Z85 digits + (4-P) known low bytes.
          // Use closed-form canonical minimum (same as production).
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
        // Structure: [(P+1) Z85 chars] [escape] [K bytes]
        // digits.length = P+1 for non-aligned (P >= 1), or 0 for block-aligned.

        if (digits.length === 0) {
          // Block-aligned: no preceding chars, just output the K bytes.
          out.push(...pass);
          i += 1 + passLen;
          continue;
        }

        // Non-aligned: digits.length == P+1, giving P+1 high-order digit indices.
        // The first (4-P) = (4-(digits.length-1)) = (5-digits.length) passthrough bytes
        // are the known low bytes of the before-block.
        const numDigits = digits.length; // P+1
        const P = numDigits - 1;
        const numKnownLow = 4 - P;       // = 5 - numDigits
        const knownLow = pass.slice(0, numKnownLow);

        // Use closed-form extendedBlockValue (P+1 digits fully determine the value).
        const beforeVal = extendedBlockValue(digits, knownLow);
        if (beforeVal < 0) throw new Error("extended passthrough: invalid before-block");

        // Output only the first P bytes of the before-block (not in passthrough).
        const bBytes = valueToBytes(beforeVal);
        for (let k = 0; k < P; k++) out.push(bBytes[k]);
        // Then output all K passthrough bytes.
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

    // How many digits do we need to complete this block?
    // Normal: 5. After a non-aligned 4-byte passthrough: 5-P (P = knownHighBytes.length).
    const needed = 5 - knownHighBytes.length;
    if (digits.length === needed) {
      const raw = digitsToValue(digits);
      if (raw === null) throw new Error("Z85 value overflow");

      let value: number;
      if (knownHighBytes.length === 0) {
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
      extraSafeBytes: args["extra-safe-characters"],
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
    await Deno.stdout.write(encode(stdin, opts));
  } else if (Deno.args[0] === "encode-lines") {
    const stdin = await readAll(Deno.stdin);
    const encoded = textEncode(stdin, PRINTABLE_ASCII_ENCODING);
    // Split into 80-character lines
    const lines: string[] = [];
    for (let i = 0; i < encoded.length; i += 80) {
      lines.push(encoded.slice(i, i + 80));
    }
    await Deno.stdout.write(new TextEncoder().encode(lines.join('\n') + '\n'));
  } else if (Deno.args[0] === "decode") {
    const stdin = await readAll(Deno.stdin);
    await Deno.stdout.write(decode(stdin));
  } else if (Deno.args[0] === "decode-lines") {
    const stdin = await readAll(Deno.stdin);
    const text = new TextDecoder().decode(stdin);
    // Strip all newlines before decoding
    const stripped = text.replace(/\n/g, '');
    const decoded = textDecode(stripped);
    await Deno.stdout.write(decoded);
  } else {
    await Deno.stderr.write(new TextEncoder().encode(
      "Usage: z855 encode|decode|encode-lines|decode-lines < input > output\n",
    ));
    return 2;
  }
}

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

/** Convert a 32-bit unsigned integer to 5 Z85 char bytes (as a Uint8Array). */
function valueToDigits(v: number): Uint8Array {
  const digits = new Uint8Array(5);
  for (let i = 4; i >= 0; i--) {
    digits[i] = Z85_DIGIT_BYTES[v % 85];
    v = Math.floor(v / 85);
  }
  return digits;
}

/** Encode a partial-block value into exactly `numChars` Z85 char bytes. */
function encodePartial(v: number, numChars: number): Uint8Array {
  const out = new Uint8Array(numChars);
  for (let i = numChars - 1; i >= 0; i--) {
    out[i] = Z85_DIGIT_BYTES[v % 85];
    v = Math.floor(v / 85);
  }
  return out;
}

/** Convert Z85 digit indices (2–5 of them) to a number, or null if out of range. */
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
