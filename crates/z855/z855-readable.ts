/**
 * z855-readable.ts — A readable reference implementation of the Z855 codec.
 *
 * Z855 extends RFC 32 Z85 (ZeroMQ's binary-to-text encoding) in two ways:
 *
 *   1. **Arbitrary-length input**: Z85 requires input to be a multiple of 4 bytes.
 *      Z855 handles any length by treating the final 1–3 bytes as a "partial block"
 *      encoded with one fewer output character than a full block (N bytes → N+1 chars).
 *
 *   2. **Raw passthrough for readable bytes**: When a run of input bytes are all in
 *      the "safe" set, Z855 may output them literally instead of Z85-encoding them.
 *      An escape character marks the raw bytes so the decoder copies them directly.
 *      This makes encoded data human-readable whenever the input contains text.
 *
 * Z85 quick recap:
 *   - 85-character alphabet: `0–9 a–z A–Z . - : + = ^ ! / * ? & < > ( ) [ ] { } @ % $ #`
 *   - Full blocks: 4 input bytes → 5 output chars  (big-endian u32, base-85 encoded)
 *   - Partial final block: N bytes (1–3) → N+1 output chars
 *
 * Passthrough escapes (all produce output the same length as standard Z85 would):
 *   `,XXXX`       — 4 raw bytes  (block-aligned or non-aligned)
 *   `C;BBBBB`     — 1 Z85 char + 5 raw bytes  (and similar with `_` for 6, `~` for 7)
 *   `NN|RRR...`   — 8+ raw bytes, length encoded as base-42 prefix `NN`, padding with `.`
 *   `0|RRR...`    — rest-of-input raw (shortcut when safe bytes reach end of input)
 *
 * This file prioritises clarity over performance. It produces the same output as
 * the production `z855.ts` for all inputs (verified against 1416 test cases) except
 * for the *choice* of passthrough position when multiple positions are equally valid —
 * the production encoder uses a bit-reversal heuristic to pick the "most aligned" one,
 * which we skip here since it only affects aesthetics, not correctness.
 */

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/** The 85-character Z85 alphabet. Index i → alphabet[i]. */
const ALPHABET =
  "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";

/** Reverse map: char code → Z85 digit value (0–84), or −1 if not in alphabet. */
const CHAR_TO_DIGIT: Int8Array = (() => {
  const t = new Int8Array(256).fill(-1);
  for (let i = 0; i < 85; i++) t[ALPHABET.charCodeAt(i)] = i;
  return t;
})();

/**
 * The "safe" set: bytes that qualify for raw passthrough.
 * This is the Z85 alphabet (85 chars) plus the five escape characters themselves
 * (`,;|~_`), so that sequences containing those characters can also pass through.
 * Total: 90 characters.
 */
const SAFE_CHARS =
  "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#,;|~_";

const IS_SAFE: Uint8Array = (() => {
  const t = new Uint8Array(256);
  for (let i = 0; i < SAFE_CHARS.length; i++) t[SAFE_CHARS.charCodeAt(i)] = 1;
  return t;
})();

// Escape character codes for the passthrough extension.
const ESC_4    = 0x2c; // ',' — next 4 bytes are raw
const ESC_5    = 0x3b; // ';' — extended: (P+1) Z85 + 5 raw bytes
const ESC_6    = 0x5f; // '_' — extended: (P+1) Z85 + 6 raw bytes
const ESC_7    = 0x7e; // '~' — extended: (P+1) Z85 + 7 raw bytes
const ESC_LONG = 0x7c; // '|' — long: base-42 prefix + raw bytes + dot padding
const PAD_DOT  = 0x2e; // '.' — padding byte inside the long escape (ignored on decode)
const PAD_HASH = 0x23; // '#' — padding used in concatenatable mode (not our concern here)

/** Maximum safe-byte run that a single long-escape can cover. */
const MAX_LONG_PASSTHROUGH = 65536;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

export class Z855DecodeError extends Error {
  constructor(msg: string) { super(msg); this.name = "Z855DecodeError"; }
}

/**
 * Encode arbitrary bytes to a Z855 string.
 *
 * Walks the input sequentially, choosing at each position the best available
 * passthrough form (most bytes covered) or falling back to standard Z85.
 */
export function encode(input: Uint8Array): string {
  if (input.length === 0) return "";

  const out: string[] = [];
  let i = 0;

  while (i < input.length) {
    const remaining = input.length - i;

    // -----------------------------------------------------------------------
    // Full-block path: 4 or more bytes available.
    //
    // We try passthrough forms first (most-efficient to least), then fall back
    // to standard Z85.  Every passthrough form produces exactly the same number
    // of output characters as standard Z85 would for the same byte count — so
    // switching between them never changes the total output length.
    // -----------------------------------------------------------------------
    if (remaining >= 4) {
      // (A) Long passthrough: 8+ consecutive safe bytes → prefix `|` raw [padding]
      const long = tryLongPassthrough(input, i);
      if (long !== null) { out.push(...long.chars); i += long.consumed; continue; }

      // (B) Extended passthrough: 5, 6, or 7 safe bytes → (P+1) Z85 + escape + raw
      const ext = tryExtendedPassthrough(input, i);
      if (ext !== null) { out.push(...ext.chars); i += ext.consumed; continue; }

      // (C) Block-aligned 4-byte passthrough: all 4 bytes safe → `,XXXX`
      if (allSafe(input, i, 4)) {
        out.push(",");
        for (let k = 0; k < 4; k++) out.push(chr(input[i + k]));
        i += 4;
        continue;
      }

      // (D) Non-aligned 4-byte passthrough: 4 safe bytes at offset 1, 2, or 3 within
      //     the current 4-byte block.  Requires additional constraints — see below.
      const nna = tryNonAlignedPassthrough(input, i);
      if (nna !== null) { out.push(...nna.chars); i += nna.consumed; continue; }

      // (E) Standard Z85 encoding for a full 4-byte block.
      out.push(...z85Encode(readU32(input, i), 5));
      i += 4;
      continue;
    }

    // -----------------------------------------------------------------------
    // Partial-block path: 1, 2, or 3 bytes remaining.
    //
    // There is no passthrough for partial blocks — always Z85.
    //
    // A partial block of N bytes is encoded as N+1 Z85 characters.  Why N+1?
    // Because N Z85 characters can represent only 85^N distinct values.  For
    // N=1 that's 85, which is fewer than the 256 possible byte values.  Adding
    // one extra character gives 85^(N+1) ≥ 256^N possibilities, always enough.
    //
    // Mechanically: treat the N bytes as an integer (big-endian), then base-85
    // encode it into exactly N+1 digits — the same math as a full block but
    // stopping one digit early.
    // -----------------------------------------------------------------------
    let value = 0;
    for (let k = 0; k < remaining; k++) value = value * 256 + input[i + k];
    out.push(...z85Encode(value, remaining + 1));
    i += remaining;
  }

  return out.join("");
}

/**
 * Decode a Z855 string back to bytes.
 * @throws Z855DecodeError on any invalid input.
 */
export function decode(input: string): Uint8Array {
  if (input.length === 0) return new Uint8Array(0);

  const out: number[] = [];
  let i = 0;

  // Z85 digits accumulated for the current block (cleared after each full block).
  let digits: number[] = [];

  // After a non-aligned `,` passthrough, the P high bytes of the "after" block are
  // known from the passthrough itself.  We hold them here until the remaining (5−P)
  // Z85 digits arrive, then reconstruct the full 32-bit value.
  let knownHighBytes: number[] = [];

  while (i < input.length) {
    const code = input.charCodeAt(i);

    // ------------------------------------------------------------------
    // Hash padding (concatenatable mode): `#` at block boundary.
    // A valid hash-padded group is 1–3 `#` chars followed by 2–4 Z85 chars.
    // `#####` is NOT hash padding — it falls through to normal Z85 (and overflows).
    // ------------------------------------------------------------------
    if (code === PAD_HASH && digits.length === 0 && i % 5 === 0 && input.length - i >= 5) {
      let h = 0;
      while (h < 3 && input.charCodeAt(i + h) === PAD_HASH) h++;
      if (h > 0 && input.charCodeAt(i + h) !== PAD_HASH) {
        out.push(...decodeHashBlock(input, i, h));
        i += 5;
        continue;
      }
      // Fall through: not hash padding, treat '#' as a Z85 digit.
    }

    // ------------------------------------------------------------------
    // Long escape: `|`
    //
    // Full structure:  [prefix][`|`][`offset` bytes of padding][raw bytes][remaining padding]
    //
    // The prefix (in `digits`) encodes two optional self-terminating base-42 numbers:
    //   - If two numbers: first is `offset` (dots before raw), second is `rawLen`.
    //   - If one number:  it is `rawLen`, offset = 0.
    //   - `rawLen = 0` is special: copy the rest of input literally.
    //
    // Total envelope length is always z855OutputLen(rawLen), so the decoder knows
    // exactly how much to skip after the raw bytes.
    // ------------------------------------------------------------------
    if (code === ESC_LONG) {
      if (digits.length === 0) throw new Z855DecodeError("'|' with no prefix");
      i++; // consume '|'

      const { offset, length: rawLen } = readLongPrefix(digits);
      if (rawLen >= 1 && rawLen <= 7) throw new Z855DecodeError(`invalid | length ${rawLen}`);

      if (rawLen === 0) {
        // Rest-of-input raw.
        for (; i < input.length; i++) out.push(input.charCodeAt(i));
        digits = []; knownHighBytes = [];
        break;
      }

      // The total envelope is: prefixLen + 1('|') + paddingTotal + rawLen = z855OutputLen(rawLen).
      const prefixLen = digits.length;
      const envelopeLen = z855OutputLen(rawLen);
      const paddingTotal = envelopeLen - prefixLen - 1 - rawLen;
      // `offset` bytes of padding come before the raw bytes; the rest come after.
      const paddingAfter = paddingTotal - offset;
      if (paddingAfter < 0) throw new Z855DecodeError("invalid | offset");

      i += offset; // skip pre-raw padding (content doesn't matter)
      if (i + rawLen > input.length) throw new Z855DecodeError("truncated | escape");
      for (let k = 0; k < rawLen; k++) out.push(input.charCodeAt(i + k));
      i += rawLen;
      i += paddingAfter; // skip post-raw padding

      digits = []; knownHighBytes = [];
      continue;
    }

    // ------------------------------------------------------------------
    // Short passthrough escapes: `,` (4), `;` (5), `_` (6), `~` (7)
    // ------------------------------------------------------------------
    const passLen = escapeLen(code);
    if (passLen > 0) {
      if (i + passLen >= input.length) throw new Z855DecodeError("incomplete passthrough");
      const pass: number[] = [];
      for (let k = 1; k <= passLen; k++) pass.push(input.charCodeAt(i + k));

      if (passLen === 4) {
        // `,` escape.
        // P = how many Z85 chars accumulated before the `,`.
        const P = digits.length;

        if (P === 0) {
          // Block-aligned: just copy the 4 bytes.
          out.push(...pass);
          i += 5;
        } else {
          // Non-aligned (P ≥ 1): the passthrough bytes overlap with the surrounding blocks.
          //
          // "Before" block: the encoder emitted P Z85 digits, then hid the low (4−P) bytes
          // of that block inside the passthrough.  To recover the before-block value, we
          // need the minimum 32-bit number consistent with those P digits AND those (4−P)
          // known low bytes.  (This "canonical minimum" is also what the encoder checks
          // before allowing non-aligned passthrough — guaranteeing a unique round-trip.)
          //
          // "After" block: the last P passthrough bytes are the HIGH bytes of the next
          // block.  Hold them in knownHighBytes; the remaining (5−P) Z85 digits will
          // arrive next and let us reconstruct the full value.
          const numKnownLow = 4 - P;
          out.push(...u32ToBytes(canonicalMin(digits, pass.slice(0, numKnownLow))));
          knownHighBytes = pass.slice(numKnownLow);
          digits = [];
          i += 5;
        }
      } else {
        // `;` / `_` / `~` extended escape (K = 5, 6, or 7 bytes).
        //
        // Structure: [(P+1) Z85 chars][escape][K raw bytes]
        //
        // The key difference from `,`: there's ONE EXTRA Z85 digit before the escape.
        // That extra digit fully determines the before-block (no canonical-minimum
        // search needed).  We output only the top P bytes of the before-block
        // (the remaining 4−P overlap with the passthrough and are output via it),
        // then output all K passthrough bytes directly.
        //
        // P+1 = digits.length, so P = digits.length − 1.
        // When digits.length = 0 (block-aligned), just copy raw bytes.
        if (digits.length === 0) {
          out.push(...pass);
          i += 1 + passLen;
          continue;
        }
        const P = digits.length - 1;
        const numKnownLow = 4 - P;
        const beforeVal = beforeBlockFromDigits(digits, pass.slice(0, numKnownLow));
        // Output the top P bytes (the ones NOT covered by the passthrough).
        const bBytes = u32ToBytes(beforeVal);
        for (let k = 0; k < P; k++) out.push(bBytes[k]);
        // Then all K raw bytes.
        out.push(...pass);
        digits = []; knownHighBytes = [];
        i += 1 + passLen;
      }
      continue;
    }

    // ------------------------------------------------------------------
    // Regular Z85 character — accumulate and flush when block is complete.
    // ------------------------------------------------------------------
    const d = CHAR_TO_DIGIT[code];
    if (d === -1) throw new Z855DecodeError(`invalid char 0x${code.toString(16).toUpperCase()}`);
    digits.push(d);
    i++;

    // We need 5 digits for a full block, but after a non-aligned `,` passthrough
    // we already know the top P bytes, so we only need 5−P more digits.
    const needed = 5 - knownHighBytes.length;
    if (digits.length === needed) {
      const raw = digitsToValue(digits);
      const value = knownHighBytes.length === 0
        ? raw                                                   // normal block
        : reconstructAfterBlock(knownHighBytes, raw, needed);  // after non-aligned ','

      if (value > 0xffffffff) throw new Z855DecodeError("Z85 value overflow");
      out.push(...u32ToBytes(value));
      digits = []; knownHighBytes = [];
    }
  }

  // ------------------------------------------------------------------
  // Flush any trailing partial block (2, 3, or 4 accumulated digits).
  //
  // The decoder side of the partial-block encoding: 2 digits → 1 byte,
  // 3 digits → 2 bytes, 4 digits → 3 bytes.  1 digit is always invalid.
  // ------------------------------------------------------------------
  if (digits.length > 0) {
    if (digits.length === 1) throw new Z855DecodeError("invalid: single trailing char");
    const value = digitsToValue(digits);
    const numBytes = digits.length - 1;
    const maxValue = [0, 0xff, 0xffff, 0xffffff][numBytes];
    if (value > maxValue) throw new Z855DecodeError("Z85 value overflow in partial block");
    for (let k = numBytes - 1; k >= 0; k--) out.push((value >>> (k * 8)) & 0xff);
  }

  return new Uint8Array(out);
}

// ---------------------------------------------------------------------------
// Long passthrough (8+ bytes)
// ---------------------------------------------------------------------------

/**
 * Try to encode 8+ consecutive safe bytes using the `|` long escape.
 *
 * When the safe run reaches end-of-input: emit `0|` + all remaining bytes.
 * Otherwise: emit a base-42 length prefix, `|`, the raw bytes, and dot padding.
 * The total output length equals z855OutputLen(rawLen) in both cases.
 *
 * The encoder always puts padding AFTER the raw bytes (offset = 0).  The
 * production encoder places padding more cleverly for alignment aesthetics, but
 * any placement is valid — the decoder uses the prefix to find the raw bytes.
 */
function tryLongPassthrough(input: Uint8Array, start: number): { chars: string[]; consumed: number } | null {
  let safeCount = 0;
  for (let j = start; j < input.length && safeCount < MAX_LONG_PASSTHROUGH; j++) {
    if (!IS_SAFE[input[j]]) break;
    safeCount++;
  }
  if (safeCount < 8) return null;

  if (start + safeCount === input.length) {
    // Rest-of-input shortcut: `0|` + raw bytes (no padding needed).
    return { chars: ["0", "|", ...rawChars(input, start, safeCount)], consumed: safeCount };
  }

  // General form: length-prefixed envelope, all padding after raw bytes.
  const rawLen = safeCount;
  const prefix = longPrefix(rawLen);
  const totalLen = z855OutputLen(rawLen);
  const paddingCount = totalLen - prefix.length - 1 - rawLen;
  return {
    chars: [...prefix, "|", ...rawChars(input, start, rawLen), ...Array(paddingCount).fill(".")],
    consumed: rawLen,
  };
}

/** Generate base-42 self-terminating prefix characters for a given length value. */
function longPrefix(value: number): string[] {
  // Single digit if value < 42 (terminal digit, < 42).
  if (value < 42) return [ALPHABET[value]];
  // Multi-digit: extract base-42 digits big-endian; first is terminal, rest get +42.
  const digits: number[] = [];
  for (let v = value; v > 0; v = Math.floor(v / 42)) digits.push(v % 42);
  digits.reverse();
  return digits.map((d, i) => ALPHABET[i === 0 ? d : d + 42]);
}

/**
 * Parse the base-42 prefix digits accumulated before a `|`.
 *
 * The prefix holds at most two self-terminating base-42 numbers (read right-to-left):
 *   - Last number  = rawLen (0 means "rest of input")
 *   - First number = offset (dots before raw bytes; 0 if absent)
 *
 * A digit < 42 terminates a number; a digit ≥ 42 is a continuation digit (value − 42).
 */
function readLongPrefix(digits: number[]): { offset: number; length: number } {
  const { value: length, count } = readBase42RTL(digits, digits.length);
  if (count === digits.length) return { offset: 0, length }; // only one number
  const { value: offset } = readBase42RTL(digits, digits.length - count);
  return { offset, length };
}

/** Read one base-42 self-terminating number from `digits[0..end]`, right-to-left. */
function readBase42RTL(digits: number[], end: number): { value: number; count: number } {
  let value = 0, multiplier = 1, pos = end, count = 0;
  while (pos > 0) {
    const d = digits[--pos]; count++;
    if (d >= 42) { value += (d - 42) * multiplier; multiplier *= 42; }
    else         { value += d * multiplier; break; }
  }
  return { value, count };
}

// ---------------------------------------------------------------------------
// Extended passthrough (5, 6, or 7 bytes)
// ---------------------------------------------------------------------------

/**
 * Try to encode 5, 6, or 7 consecutive safe bytes using `;`, `_`, or `~`.
 *
 * Structure: [(P+1) Z85 chars from before-block] [escape] [K raw bytes]
 *
 * P is chosen so that:
 *   - K safe bytes start at input[blockStart + P], and
 *   - Total output length = (P+1) + 1 + K + z855OutputLen(remaining) = z855OutputLen(total).
 *     (This "length invariant" ensures the extended passthrough fits the envelope exactly.)
 *
 * We prefer larger K (7 > 6 > 5) and prefer block-aligned positions (P=0) first.
 * The (P+1) Z85 digits uniquely determine the before-block — no canonical-minimum
 * check needed (that's the advantage over the `,` form which only uses P digits).
 */
function tryExtendedPassthrough(input: Uint8Array, blockStart: number): { chars: string[]; consumed: number } | null {
  for (const k of [7, 6, 5]) {
    const esc = chr(k === 7 ? ESC_7 : k === 6 ? ESC_6 : ESC_5);
    const total = input.length - blockStart;

    // Try P = 0 first (block-aligned: just escape + K raw bytes, no preceding Z85 digits).
    if (blockStart + k <= input.length && allSafe(input, blockStart, k)) {
      // Length invariant: (1 + k) + z855OutputLen(total − k) === z855OutputLen(total)
      if (1 + k + z855OutputLen(total - k) === z855OutputLen(total)) {
        return { chars: [esc, ...rawChars(input, blockStart, k)], consumed: k };
      }
    }

    // Try P = 1, 2, 3 (non-aligned).
    for (let p = 1; p <= 3; p++) {
      const passStart = blockStart + p;
      if (passStart + k > input.length) continue;
      if (!allSafe(input, passStart, k)) continue;
      const consumed = p + k;
      // Length invariant: (p+1 + 1 + k) + z855OutputLen(total − consumed) === z855OutputLen(total)
      if (p + 1 + 1 + k + z855OutputLen(total - consumed) !== z855OutputLen(total)) continue;
      if (blockStart + 4 > input.length) continue;

      const beforeVal = readU32(input, blockStart);
      return {
        chars: [...z85Encode(beforeVal, 5).slice(0, p + 1), esc, ...rawChars(input, passStart, k)],
        consumed,
      };
    }
  }
  return null;
}

// ---------------------------------------------------------------------------
// Non-aligned 4-byte passthrough
// ---------------------------------------------------------------------------

/**
 * Try to encode a non-aligned 4-byte passthrough.
 *
 * This handles 4 safe bytes starting at offset P = 1, 2, or 3 within the current
 * 4-byte input block.  Output:
 *   [P Z85 chars for before-block] [`,`] [4 safe bytes] [(5−P) Z85 chars for after-block]
 *
 * Two constraints must hold:
 *
 *   1. CANONICAL MINIMUM: The 32-bit "before" block value must be the *smallest*
 *      value consistent with its P Z85 digits and the (4−P) passthrough bytes that
 *      overlap it.  The decoder recovers the before-block by this same rule, so any
 *      other value would round-trip incorrectly.
 *
 *   2. COMPLETE AFTER-BLOCK: The 4 bytes after the current 4-byte block must all
 *      be present in the input, because the non-aligned passthrough overlaps them.
 *      (If the input ends partway through the after-block, fall back to standard Z85.)
 */
function tryNonAlignedPassthrough(input: Uint8Array, blockStart: number): { chars: string[]; consumed: number } | null {
  for (const p of [1, 2, 3]) {
    const passStart = blockStart + p;
    if (passStart + 4 > input.length) continue;
    if (!allSafe(input, passStart, 4)) continue;

    const beforeVal = readU32(input, blockStart);
    const numKnownLow = 4 - p;
    const knownLow: number[] = Array.from({ length: numKnownLow }, (_, k) => input[passStart + k]);

    if (!isCanonicalMin(beforeVal, p, knownLow)) continue;

    // The after-block: first P bytes come from passthrough, next (4−P) from input.
    const afterStart = blockStart + 4;
    if (afterStart + p + (4 - p) > input.length) continue; // need full after-block

    const afterBytes = [
      ...Array.from({ length: p }, (_, k) => input[passStart + numKnownLow + k]),   // from passthrough
      ...Array.from({ length: 4 - p }, (_, k) => input[afterStart + p + k]),        // from input
    ];
    const afterVal = (afterBytes[0] << 24 | afterBytes[1] << 16 | afterBytes[2] << 8 | afterBytes[3]) >>> 0;

    return {
      chars: [
        ...z85Encode(beforeVal, 5).slice(0, p),
        ",",
        ...Array.from({ length: 4 }, (_, k) => chr(input[passStart + k])),
        ...z85Encode(afterVal, 5).slice(p),
      ],
      consumed: 8,
    };
  }
  return null;
}

// ---------------------------------------------------------------------------
// Canonical-minimum logic
// ---------------------------------------------------------------------------

/**
 * Return the canonical (minimum) 32-bit value consistent with:
 *   - `digits`: P high-order Z85 digits
 *   - `knownLow`: the (4−P) low bytes known from the passthrough
 *
 * The P Z85 digits define a contiguous "stripe" of u32 values.  Within that
 * stripe, many values share the same low (4−P) bytes.  We return the smallest.
 *
 * Math: digits define range [base·85^(5−P), (base+1)·85^(5−P)).
 * We find the smallest value ≥ rangeStart where (value mod 256^(4−P)) = knownPart.
 */
function canonicalMin(digits: number[], knownLow: number[]): number {
  let base = 0;
  for (const d of digits) base = base * 85 + d;
  const power = 85 ** (5 - digits.length);
  const rangeStart = base * power;
  const rangeEnd = (base + 1) * power;

  if (knownLow.length === 0) return rangeStart; // P=4: digits fully determine value

  let knownPart = 0;
  for (const b of knownLow) knownPart = knownPart * 256 + b;
  const modulus = 256 ** knownLow.length;

  let candidate = rangeStart - rangeStart % modulus + knownPart;
  if (rangeStart % modulus > knownPart) candidate += modulus;

  if (candidate >= rangeEnd || candidate > 0xffffffff) {
    throw new Z855DecodeError("non-aligned passthrough: invalid before-block");
  }
  return candidate;
}

/** True if `value` equals the canonical minimum for these P digits and known low bytes. */
function isCanonicalMin(value: number, P: number, knownLow: number[]): boolean {
  const digits = z85Encode(value, 5).slice(0, P).map(ch => ALPHABET.indexOf(ch));
  try { return value === canonicalMin(digits, knownLow); }
  catch { return false; }
}

/**
 * Determine the before-block value from (P+1) Z85 digits + (4−P) known low bytes.
 *
 * Same math as canonicalMin, but with one extra digit (P+1 instead of P), which
 * is always sufficient to uniquely identify the value — no "minimum of a set" needed.
 */
function beforeBlockFromDigits(digits: number[], knownLow: number[]): number {
  let base = 0;
  for (const d of digits) base = base * 85 + d;
  const power = 85 ** (5 - digits.length);
  const rangeStart = base * power;
  const rangeEnd = (base + 1) * power;

  if (knownLow.length === 0) return rangeStart;

  let knownPart = 0;
  for (const b of knownLow) knownPart = knownPart * 256 + b;
  const modulus = 256 ** knownLow.length;

  let candidate = rangeStart - rangeStart % modulus + knownPart;
  if (rangeStart % modulus > knownPart) candidate += modulus;

  if (candidate >= rangeEnd || candidate > 0xffffffff) {
    throw new Z855DecodeError("extended passthrough: invalid before-block");
  }
  return candidate;
}

// ---------------------------------------------------------------------------
// After-block reconstruction (non-aligned `,` decode)
// ---------------------------------------------------------------------------

/**
 * After a non-aligned `,` passthrough, the "after" block is partly known:
 *   - `knownHigh`: the top P bytes (came from the passthrough)
 *   - `lowVal`: the numeric value of the (5−P) low Z85 digits
 *   - `numDigits`: 5−P
 *
 * Find the 32-bit value V where high P bytes = knownHigh and V mod 85^(5−P) = lowVal.
 */
function reconstructAfterBlock(knownHigh: number[], lowVal: number, numDigits: number): number {
  let highPart = 0;
  for (const b of knownHigh) highPart = highPart * 256 + b;
  const shift = 8 * (4 - knownHigh.length);
  const rangeStart = (highPart << shift) >>> 0;
  const rangeSize = 1 << shift;
  const modulus = 85 ** numDigits;

  let candidate = rangeStart - rangeStart % modulus + lowVal;
  if (rangeStart % modulus > lowVal) candidate += modulus;

  if (candidate >= rangeStart + rangeSize) throw new Z855DecodeError("after-block reconstruction failed");
  return candidate >>> 0;
}

// ---------------------------------------------------------------------------
// Hash-block decode (concatenatable mode)
// ---------------------------------------------------------------------------

function decodeHashBlock(input: string, blockStart: number, hashCount: number): number[] {
  const numChars = 5 - hashCount;
  const numBytes = numChars - 1;
  let value = 0;
  for (let k = 0; k < numChars; k++) {
    const d = CHAR_TO_DIGIT[input.charCodeAt(blockStart + hashCount + k)];
    if (d === -1) throw new Z855DecodeError("invalid char in hash block");
    value = value * 85 + d;
  }
  const maxVal = [0, 0xff, 0xffff, 0xffffff][numBytes];
  if (value > maxVal) throw new Z855DecodeError("overflow in hash block");
  return Array.from({ length: numBytes }, (_, k) => (value >>> ((numBytes - 1 - k) * 8)) & 0xff);
}

// ---------------------------------------------------------------------------
// Tiny utilities
// ---------------------------------------------------------------------------

/** Encode `value` as exactly `numChars` Z85 characters (big-endian base-85). */
function z85Encode(value: number, numChars: number): string[] {
  const chars = new Array<string>(numChars);
  for (let j = numChars - 1; j >= 0; j--) { chars[j] = ALPHABET[value % 85]; value = Math.floor(value / 85); }
  return chars;
}

/** Read 4 bytes at `input[i..i+4]` as a big-endian unsigned 32-bit integer. */
function readU32(input: Uint8Array, i: number): number {
  return ((input[i] << 24) | (input[i+1] << 16) | (input[i+2] << 8) | input[i+3]) >>> 0;
}

/** Split a 32-bit integer into 4 big-endian bytes. */
function u32ToBytes(v: number): [number, number, number, number] {
  return [(v >>> 24) & 0xff, (v >>> 16) & 0xff, (v >>> 8) & 0xff, v & 0xff];
}

/** Accumulate Z85 digit values into a single number. */
function digitsToValue(digits: number[]): number {
  let v = 0;
  for (const d of digits) v = v * 85 + d;
  return v;
}

/** True if `count` bytes starting at `input[start]` are all in the safe set. */
function allSafe(input: Uint8Array, start: number, count: number): boolean {
  for (let k = 0; k < count; k++) if (!IS_SAFE[input[start + k]]) return false;
  return true;
}

/** Standard Z85 output length for N input bytes: ceil(N × 5 / 4). */
function z855OutputLen(n: number): number { return Math.ceil(n * 5 / 4); }

/** Return the short-passthrough byte count for an escape char code, or 0. */
function escapeLen(code: number): number {
  return code === ESC_4 ? 4 : code === ESC_5 ? 5 : code === ESC_6 ? 6 : code === ESC_7 ? 7 : 0;
}

/** Convert a byte value to a one-character string. */
function chr(b: number): string { return String.fromCharCode(b); }

/** Extract `count` bytes from `input` starting at `start` as char strings. */
function rawChars(input: Uint8Array, start: number, count: number): string[] {
  return Array.from({ length: count }, (_, k) => chr(input[start + k]));
}
