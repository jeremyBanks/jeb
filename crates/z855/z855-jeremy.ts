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
  const bufferSize = Math.ceil(original.length / 4) * 5;
  const buffer = new Uint8Array(bufferSize);

  let inputOffset = 0;
  let encodedOffset = 0;

  const safeBytes = new Array(256).fill(false);
  for (let byte of extraSafeBytes ?? []) {
    if (typeof byte === "string") {
      assert(byte.length === 1, "extra safe byte must be a single character");
      byte = byte.charCodeAt(0);
    }
    assert(Number.isInteger(byte), "extra safe byte must be an integer");
    assert(Number.isFinite(byte), "extra safe byte must be a finite number");
    assert(
      byte >= 0 && byte <= 255,
      "extra safe byte must be between 0 and 255",
    );
    safeBytes[byte] = true;
  }
  for (const byte of Z85_VALUES_BYTES.keys()) {
    safeBytes[byte] = true;
  }

  while (inputOffset < original.length) {
    const nextBlock = original.subarray(
      inputOffset,
      inputOffset + BLOCK_SIZE_ORIGINAL,
    );

    let blockDigits = valueToDigits(bytesToValue([...nextBlock]));

    if (nextBlock.length < BLOCK_SIZE_ORIGINAL) {
      // incomplete block at end of input, if this hasn't already been
      // merged into a previous raw escape it's too late and we know it
      // must be encoded with Z85, then handled as appropriate depending
      // on whether we're in concatenatable mode or not.
      const requiredDigits = nextBlock.length + 1;
      if (concatenatable) {
        // Hash padding goes at the FRONT, not overwriting the digits
        const paddingByte = PAD_HASH;
        const paddingCount = 5 - requiredDigits;
        const output = new Uint8Array(5);
        for (let i = 0; i < paddingCount; i++) {
          output[i] = paddingByte;
        }
        for (let i = 0; i < requiredDigits; i++) {
          output[paddingCount + i] = blockDigits[5 - requiredDigits + i];
        }
        buffer.set(output, encodedOffset);
        encodedOffset += 5;
      } else {
        const partialDigits = blockDigits.subarray(5 - requiredDigits);
        buffer.set(partialDigits, encodedOffset);
        encodedOffset += partialDigits.length;
      }

      inputOffset += nextBlock.length;
    } else if (!safeBytes[nextBlock.at(-1)!]) {
      // If the last byte of this block is not safe, the entire block must be Z85-encoded.
      buffer.set(blockDigits, encodedOffset);
      encodedOffset += blockDigits.length;
      inputOffset += BLOCK_SIZE_ORIGINAL;
    } else {
      // this is where the fun begins

      // First, we need to know how many bytes in this block which are contiguous with the
      // end of the block are safe (may be between 1 and 4), then how many contiguous bytes
      // following this block are safe (but this is clamped so that the sum of the safe lengths
      // is at most maxRawLength).

      let safeBytesAtEnd = 0;
      for (let i = BLOCK_SIZE_ORIGINAL - 1; i >= 0; i--) {
        if (safeBytes[nextBlock[i]]) {
          safeBytesAtEnd++;
        } else {
          break;
        }
      }

      const remainingAfterBlock = original.length - inputOffset -
        BLOCK_SIZE_ORIGINAL;

      let safeBytesFollowing = 0;
      for (
        let i = 0;
        i < remainingAfterBlock &&
        safeBytesFollowing + safeBytesAtEnd < maxRawLength;
        i++
      ) {
        if (safeBytes[original[inputOffset + BLOCK_SIZE_ORIGINAL + i]]) {
          safeBytesFollowing++;
        } else {
          break;
        }
      }

      const safeLength = safeBytesAtEnd + safeBytesFollowing;
      const remainingAfterSafe = remainingAfterBlock - safeBytesFollowing;

      // Now we have to determine all of the possible ways that we could encode this block, then we filter
      // the candidates to determine which ones round-trip correctly, then sort them based on length
      // and alignment.

      if (remainingAfterSafe == 0 && !concatenatable) {
        // If we don't need to be concatenatable, we can use the `0|` "until end of stream"
        // special raw escape prefix. This is the ONLY case where the output can have a different length
        // than the standard Z85 encoding.
        const prefix = new TextEncoder().encode(`0${ESCAPE_MANY}`);
        buffer.set(prefix, encodedOffset);
        encodedOffset += prefix.length;
        // The safe bytes start at (inputOffset + BLOCK_SIZE_ORIGINAL - safeBytesAtEnd)
        const safeStart = inputOffset + BLOCK_SIZE_ORIGINAL - safeBytesAtEnd;
        buffer.set(original.subarray(safeStart, original.length), encodedOffset);
        encodedOffset += safeLength;
        inputOffset = original.length;
        break;
      }

      throw new Error("not implemented");
    }
  }

  return buffer.subarray(0, encodedOffset);
}

/** Decode a Uint8Array to a Uint8Array using Z855. */
export function decode(encoded: Uint8Array): Uint8Array {
  const buffer = new Uint8Array(encoded.length);
  throw new Error("not implemented");
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
  } else {
    await Deno.stderr.write(new TextEncoder().encode(
      "Usage: z855 encode|decode < input > output",
    ));
    return 2;
  }
}

// Returns a string that can be used to rank how-aligned a given number is, implicitly
// tiebreaking in favor of lower numbers, with 0 being the maximally-aligned value.
function alignmentKey(n: number): string {
  if (n === 0) return "";
  let key = "";
  let r = 0;
  let prevDist = 0;
  for (let k = 0;; k++) {
    r |= ((n >> k) & 1) << k;
    const pow = 1 << (k + 1);
    const dist = r < pow - r ? r : pow - r;
    key += dist > prevDist ? "1" : "0";
    prevDist = dist;
    if (r === n && dist === n) break;
  }
  return key;
}

function alignmentRank(n: number): string {
  return (
    parseInt([...(n | 0).toString(2).padStart(32, "0")].reverse().join(""), 2)
      .toString(16)
  ).padStart(8, "0");
}

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
  Z85_DIGIT_BYTES.entries().map(([byte, index]) => [byte, index]),
);

const BLOCK_SIZE_ORIGINAL = 4;
const BLOCK_SIZE_ENCODED = 5;

const ESCAPE_4 = "_";  // Jeremy's new assignment (was `,` in production)
const ESCAPE_5 = ",";  // Jeremy's new assignment (was `;` in production)
const ESCAPE_6 = "~";  // Jeremy's new assignment (was `_` in production)
const ESCAPE_7 = ";";  // Jeremy's new assignment (was `~` in production)
const ESCAPE_MANY = "|";
const PAD_HASH = 0x23;  // '#' — padding for concatenatable mode
const Z855_ESCAPE_CHARACTERS = [
  ESCAPE_4,
  ESCAPE_5,
  ESCAPE_6,
  ESCAPE_7,
  ESCAPE_MANY,
].join("");

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

/** Convert a 32-bit unsigned integer to 5 Z85 digit bytes. */
function valueToDigits(v: number): Uint8Array {
  const digits = new Uint8Array(5);
  for (let i = 4; i >= 0; i--) {
    digits[i] = Z85_DIGIT_BYTES[v % 85];
    v = Math.floor(v / 85);
  }
  return digits;
}

/** Convert 5 Z85 digit indices to a 32-bit unsigned integer, or null if out of range. */
function digitsToValue(digits: number[]) {
  let v = 0;
  for (let i = 0; i < 5; i++) {
    v = v * 85 + digits[i];
  }
  return v <= 0xFFFFFFFF ? v : null;
}

/** Convert a 32-bit unsigned integer to a 5-character Z85 string. */
function valueToChars(v: number) {
  return valueToDigits(v).map((d) => Z85_DIGITS[d]).join("");
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

// ─── Core solver ───

/**
 * Find all 32-bit values consistent with the given constraints.
 *
 * @param {Array<string|null>} chars - Length-5 array.
 *   Each element is a Z85 character (known) or null (unknown).
 * @param {Array<number|null>} bytes - Length-4 array.
 *   Each element is a byte value 0–255 (known) or null (unknown).
 *
 * @returns {number[]} Sorted ascending array of all valid 32-bit values.
 */
function findAllValues(chars: (string | null)[], bytes: (number | null)[]) {
  if (chars.length !== 5) throw new Error("chars must have length 5");
  if (bytes.length !== 4) throw new Error("bytes must have length 4");

  // Count known values
  const knownCharCount = chars.filter((c) => c !== null).length;
  const knownByteCount = bytes.filter((b) => b !== null).length;
  const totalKnown = knownCharCount + knownByteCount;

  if (totalKnown < 3) {
    throw new Error(
      `At least 3 known values required (got ${knownCharCount} chars + ${knownByteCount} bytes = ${totalKnown}).`,
    );
  }

  // Validate known chars
  for (let i = 0; i < 5; i++) {
    if (chars[i] !== null && !Object.hasOwn(Z85_VALUES, chars[i]!)) {
      throw new Error(`Invalid Z85 char at position ${i}: '${chars[i]}'`);
    }
  }

  // Validate known bytes
  for (let j = 0; j < 4; j++) {
    if (bytes[j] !== null && (bytes[j]! < 0 || bytes[j]! > 255)) {
      throw new Error(`Invalid byte value at position ${j}: ${bytes[j]}`);
    }
  }

  // Enumerate whichever side has fewer unknowns.
  const unknownChars = 5 - knownCharCount;
  const unknownBytes = 4 - knownByteCount;

  // Optimization note: a smarter version could use range + modular arithmetic
  // to avoid enumeration entirely. For now we just pick the smaller search space.
  if (unknownBytes <= unknownChars) {
    return enumerateBytes(chars, bytes);
  } else {
    return enumerateChars(chars, bytes);
  }
}

/**
 * Enumerate all possible byte combinations, filter by char constraints.
 */
function enumerateBytes(chars: (string | null)[], bytes: (number | null)[]) {
  const unknownPositions = [];
  const template = [...bytes];
  for (let j = 0; j < 4; j++) {
    if (bytes[j] === null) {
      unknownPositions.push(j);
      template[j] = 0;
    }
  }

  const numUnknown = unknownPositions.length;
  const totalCombinations = Math.pow(256, numUnknown);
  const results = [];

  for (let combo = 0; combo < totalCombinations; combo++) {
    // Fill in unknown byte positions from combo (treated as base-256 digits).
    let remaining = combo;
    for (let u = numUnknown - 1; u >= 0; u--) {
      template[unknownPositions[u]] = remaining & 0xFF;
      remaining = remaining >>> 8;
    }

    const v = bytesToValue(template.filter((b) => b !== null) as number[]);

    // Check against known characters.
    const digits = valueToDigits(v);
    let match = true;
    for (let i = 0; i < 5; i++) {
      if (chars[i] !== null && digits[i] !== Z85_VALUES.get(chars[i]!)) {
        match = false;
        break;
      }
    }

    if (match) results.push(v);
  }

  results.sort((a, b) => a - b);
  return results;
}

/**
 * Enumerate all possible char digit combinations, filter by byte constraints.
 */
function enumerateChars(chars: (string | null)[], bytes: (number | null)[]) {
  const unknownPositions = [];
  const template = chars.map((c) => c !== null ? Z85_VALUES.get(c)! : 0);
  for (let i = 0; i < 5; i++) {
    if (chars[i] === null) {
      unknownPositions.push(i);
    }
  }

  const numUnknown = unknownPositions.length;
  const totalCombinations = Math.pow(85, numUnknown);
  const results = [];

  for (let combo = 0; combo < totalCombinations; combo++) {
    // Fill in unknown char positions from combo (treated as base-85 digits).
    let remaining = combo;
    for (let u = numUnknown - 1; u >= 0; u--) {
      template[unknownPositions[u]] = remaining % 85;
      remaining = Math.floor(remaining / 85);
    }

    const v = digitsToValue(template);

    // Skip values that overflow 32 bits.
    if (v === null) continue;

    // Check against known bytes.
    const vBytes = valueToBytes(v);
    let match = true;
    for (let j = 0; j < 4; j++) {
      if (bytes[j] !== null && vBytes[j] !== bytes[j]!) {
        match = false;
        break;
      }
    }

    if (match) results.push(v);
  }

  results.sort((a, b) => a - b);
  return results;
}

/**
 * Convenience: find just the minimum valid value.
 * Returns null if no valid value exists.
 */
function findMinValue(chars: (string | null)[], bytes: (number | null)[]) {
  const all = findAllValues(chars, bytes);
  return all.length > 0 ? all[0] : null;
}

if (import.meta.main) {
  Deno.exit(await main());
}
