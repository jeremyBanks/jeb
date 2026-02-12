# Extended Z85 — Design Constraints & Analysis

*Living document. Captures the invariants, constraints, and analytical findings
that will inform the final design. Not a specification — a foundation for one.*

## 1. Mental Model

This is a **standard Z85 stream** with opportunistic escape sequences that allow
raw (unencoded) byte passthrough. Not a new format — Z85 with extensions.

## 2. Priority Order

### P1: Correctness & The Position Invariant

All Z85-encoded portions of the output must appear at the **exact same character
positions** as they would in a standard Z85 encoding of the same input data.

This means:
- Output length is always **equal to or slightly shorter than** standard Z85
  (never longer)
- Escape sequences + raw bytes + padding must consume exactly the character
  positions that standard Z85 would have used for those bytes
- Visual diff between our output and standard Z85 output shows identical Z85
  blocks in identical positions — escape sequences *replace* Z85 blocks, they
  don't shift them

**Exception — mid-block transitions:** When we cut a Z85 block partway through
to begin or end a raw section, the partial block's Z85 characters might differ
from what standard Z85 would produce (because we're encoding fewer bytes with a
different convention). This is acceptable as long as it's unambiguous and the
characters are valid Z85.

### P2: Context Compatibility

The characters we use (Z85 alphabet + escape characters) determine where
encoded data can be used without additional escaping. This isn't just "minimize
character count" — some contexts break with *any* character from a group, so
using one more from that group costs nothing.

See §4 for the full analysis.

### P3: Transparency / Readability

How much of the original data is visible in the encoded output. Quality matters
more than quantity:
- **Aligned data is more useful** — bytes at structural boundaries (word
  boundaries, field starts) are more informative than random mid-structure bytes
- **Alignment preference for tiebreaking:** When choosing between escape
  placement options, prefer the one that produces more structurally aligned raw
  data (more trailing zero bits → more likely to be at a meaningful boundary)
- This is subjective and heuristic — a tiebreaker, not a hard rule

## 3. Escape Detection & Decoder Constraints

The escape mechanism can span multiple 5-char blocks. The decoder should never
need more than ~1 block of lookahead beyond the block it's currently decoding
(guideline, not hard limit). A non-Z85 escape character must appear within that
window so the decoder knows to enter escape interpretation mode.

Raw sections can be **any byte length** (not constrained to multiples of 5
characters). The encoding is opportunistic — if a given length at a given
alignment would cause problems, the encoder simply doesn't use it.

## 4. Character Compatibility Analysis

### Z85 Alphabet (85 characters)

```
0123456789abcdefghijklmnopqrstuvwxyz
ABCDEFGHIJKLMNOPQRSTUVWXYZ
.-:+=^!/*?&<>()[]{}@%$#
```

### The 10 Non-Z85 Printable ASCII Characters

Ranked by the cost of using them as escape characters:

**Tier 1 — Completely free** (break no context Z85 doesn't already break):
- `_` (underscore)
- `~` (tilde)

**Tier 1.5 — Breaks markdown inline code only:**
- `` ` `` (backtick) — Z85's `$` already breaks shell double-quote (both do
  command substitution), but backtick also terminates `` `code` `` spans in
  markdown. Workaround exists (double-backtick delimiters) but it's a real
  compatibility regression.

**Tier 2 — Break CSV variants:**
- `|` (pipe) — breaks `|`-delimited files
- `,` (comma) — breaks standard CSV (RFC 4180)
- `;` (semicolon) — breaks European CSV (`;` as separator)

**Tier 3 — Break important contexts:**
- `'` — breaks shell single-quote strings, SQL single-quote strings
- `"` — breaks JSON strings, TOML basic strings, CSV quoting
- `\` — breaks JSON strings, TOML basic strings
- ` ` (space) — breaks word splitting everywhere, visual alignment. Unusable.

### Context Compatibility Summary

| Context | Z85 already breaks? | Additional break chars |
|---------|---------------------|----------------------|
| JSON double-quoted strings | **NO** | `"` `\` |
| Shell single-quoted strings | **NO** | `'` |
| CSV unquoted fields | **NO** | `,` `"` (`;` `\|` in variants) |
| TOML basic strings | **NO** | `"` `\` |
| SQL single-quoted strings | **NO** | `'` (`\` in some dialects) |
| URL components | YES | (free) |
| HTML/XML text | YES | (free) |
| Shell unquoted args | YES | (free) |
| Shell double-quoted strings | YES | (free — `$` already breaks) |
| YAML unquoted scalars | YES | (free) |
| Regex | YES | (free) |
| Markdown | YES | (free — but `` ` `` adds inline code) |
| Filenames | YES | (free) |

### Key Insight

The previous design (IDEATION.md) used all 6 chars from Tiers 1-2:
`_` `~` `` ` `` `|` `,` `;`. This preserves JSON string compatibility — likely
the most important context for jeb. The number of escape characters and how
their information capacity is allocated is a central design decision (see §7).

## 5. Mid-Block Stability

When cutting a Z85 block after K bytes to start a raw section, the emitted
partial Z85 characters may or may not be "stable" (determined regardless of the
unknown bytes' values).

| Cut position | Chars emitted | Stable (%) | Disambiguation bits when not |
|-------------|--------------|------------|------------------------------|
| After 1 byte | 1 leading char | 68.0% | 2 bits (3-4 candidates) |
| After 2 bytes | 2 leading chars | 89.3% | 4 bits (~9-10 candidates) |
| After 3 bytes | 3 leading chars | 96.4% | 5 bits (~27-28 candidates) |

These percentages assume uniformly random byte values. For realistic data
(small integers, ASCII-adjacent values), stability rates may be higher.

### Why These Numbers

Z85 encodes 4-byte blocks as big-endian u32, then extracts base-85 digits from
most-significant to least-significant. The leading Z85 digit `c0 = V / 85^4`
depends primarily on the high byte. Stability fails when the unknown low bytes
could push V across an `85^4` boundary.

Each `c0` value maps to 3 or 4 byte values:
- 72 out of 82 `c0` values → 3 byte candidates (need 2 bits)
- 10 out of 82 `c0` values → 4 byte candidates (need 2 bits)

The pattern: `256 / 85 ≈ 3.01`, so most digits cover exactly 3 byte values,
with every ~8th covering 4.

## 6. Entry vs Exit Boundary Symmetry

### The Key Insight

Entry and exit boundaries have **roughly the same disambiguation cost** (~2 bits
per boundary byte). The asymmetry isn't about one being "free" and the other
"impossible" — it's about **where in the 5-char Z85 block** the stable
information lives:

- **Entry boundary:** Known bytes are at the START of the block (high-order in
  Z85's big-endian convention). Information lives in the **leading** Z85 chars.
- **Exit boundary:** Known bytes are at the END of the block (low-order).
  Information lives in the **trailing** Z85 chars.

### Stability by Position

**Entry (prefix bytes → leading chars):**

| Bytes known | Bits captured | Disambiguation | Naturally stable |
|------------|--------------|----------------|-----------------|
| 1 | ~6.4 (char 0) | ~2 bits | 68% |
| 2 | ~12.8 (chars 0-1) | ~4 bits | 89% |
| 3 | ~19.2 (chars 0-2) | ~5 bits | 96% |

**Exit (suffix bytes → trailing chars):**

| Bytes known | Bits captured | Disambiguation | Trailing char stable |
|------------|--------------|----------------|---------------------|
| 1 | ~6.4 (char 4) | ~2 bits | **100%** |
| 2 | ~12.8 (chars 3-4) | ~4 bits | — |
| 3 | ~19.2 (chars 2-4) | ~5 bits | — |

The 100% stability for single-byte exit comes from a mathematical property:
`0xFFFFFF00 mod 85 = 0`, meaning the last Z85 digit depends only on the low
byte regardless of the high bytes.

### Recommended Defaults (When No Budget to Signal Endianness)

**Entry: BE (leading chars)** — The byte just before a raw section tends to be a
small value (length field, type tag, null terminator). Small values have zeros
in high bits, placing them far from `85^4` boundaries → higher stability. 68%
baseline, likely higher for real data.

**Exit: Trailing chars (effectively LE)** — The trailing Z85 digit is 100%
stable regardless of byte value. Since the first byte after a raw section is
less predictable, guaranteed stability wins.

This asymmetric default is natural, not arbitrary: entry and exit use different
ends of the Z85 character block because the known bytes occupy different
positions within the block.

### Rejected Alternative: Direct Byte Encoding

Instead of using Z85's natural leading/trailing chars for boundary bytes, we
could encode them directly: map K bytes → K+1 Z85 chars via a bijection
independent of the block's other bytes. This gives 100% stability (no
disambiguation needed) at the cost of 1 extra character per boundary.

| Approach | Chars used | Disambiguation | Total cost |
|----------|-----------|----------------|-----------|
| Natural (leading/trailing) | K | ~2 bits/byte | K chars + ~2K info bits |
| Direct encoding | K+1 | 0 | K+1 chars |

**Natural wins for tight budgets.** At budget=2 (8 raw bytes), natural uses 1
char + 1 escape = 2 chars of budget, with disambiguation packed into the escape
char's information bits. Direct encoding would need 2 chars + 1 escape = 3 chars
of budget — doesn't fit.

The key insight: each escape char choice provides ~1-2.6 bits of free
information (depending on how many escape chars exist). Those bits can carry
disambiguation more efficiently than spending a full extra Z85 character
(~6.4 bits) on it. The "different level" optimization is in how escape char
info bits are spent, not in the boundary byte encoding itself.

## 7. Padding Budget

For N raw bytes, standard Z85 uses `⌈N × 5/4⌉` characters. Raw passthrough
uses N characters. The difference is the budget for escape characters, length
encoding, disambiguation bits, and padding.

| Raw bytes | Z85 chars | Budget | Notes |
|-----------|-----------|--------|-------|
| 1-4 | 2-5 | 1 | Escape char only. No disambiguation. |
| 5-8 | 7-10 | 2 | Escape + 1 char for length/disambig |
| 9-12 | 12-15 | 3 | Escape + length + disambig (comfortable) |
| 13-16 | 17-20 | 4 | Comfortable for all cases |
| N (large) | ~5N/4 | ~N/4 | Abundant budget |

### Budget Requirements for Mid-Block Transitions

| Configuration | Minimum budget needed |
|--------------|---------------------|
| Block-aligned, no length | 1 (escape only) |
| Block-aligned, with length | 2 (escape + length char) |
| Mid-block entry OR exit | 2 (escape + disambig) |
| Mid-block both + length | 4 (escape + length + 2 disambig) |

### Critical Thresholds

- **4 bytes (budget=1):** Block-aligned raw only. No mid-block cuts. Length
  must be implied by escape char choice.
- **5-8 bytes (budget=2):** Mid-block cut at ONE boundary possible. Tight.
- **9-12 bytes (budget=3):** Mid-block at both boundaries feasible.
- **13+ bytes (budget=4+):** Everything works comfortably.

## 8. The Escape Character Budget

Each escape character beyond Z85's 85 is a precious resource. It costs context
compatibility (§4) and provides information at a critical decision point.

**What those bits could encode** (mutually exclusive uses):
- Endianness of boundary blocks (BE vs LE)
- Different raw sequence lengths
- Entry vs exit boundary conventions
- Disambiguation information for mid-block cuts

| Escape chars | Info bits | Example allocation |
|-------------|----------|-------------------|
| 1 | 0 | Fixed conventions, one raw length |
| 2 | 1 | BE/LE choice, OR 2 length classes |
| 3 | ~1.6 | Above + 1 more option |
| 4 | 2 | BE/LE × 2 length classes |
| 6 | ~2.6 | Full IDEATION.md design |

The number and allocation of escape characters is **undecided**. The analysis
above maps the constraint space; the choice depends on which capabilities matter
most for real-world data.

## 9. Resolved Design Decisions

These were open questions; they've been answered:

- **Minimum raw section length: 4 bytes.** There is no reason the minimum
  would be higher. Even with budget=1 (escape char only), a 4-byte
  block-aligned raw section works. The encoder is opportunistic — it uses
  whatever works at each point.

- **Mid-block cuts: support both entry AND exit.** We are being opportunistic;
  if a mid-block cut is possible and beneficial, do it. Both boundaries should
  be supported.

- **Asymmetric entry-BE/exit-LE: yes, worth the complexity.** This project
  accepts high complexity in exchange for value. The asymmetric convention is
  well-motivated by the mathematics (§6) and will be carefully specified.

## 10. Open Design Questions

These require discussion and decision before a specification can be written:

1. **How many escape characters?** Tiers 1-2 give us up to 6 without breaking
   JSON. What's the right number?

2. **What information does each escape char encode?** Endianness? Length?
   Disambiguation? Position-dependent meaning?

3. **How are disambiguation bits laid out?** Where in the output stream do they
   go? What encodes them (Z85 chars? escape char choice? position within block?)

4. **Raw block internal layout:** Beyond the raw bytes themselves, what is the
   syntax and ordering of the non-raw components? Specifically:
   - Where does the escape char go relative to the raw data? (Before? After?
     Multiple positions?)
   - Where does length information go? (Implicit in escape choice? Explicit
     prefix? Explicit suffix? Run until next escape/Z85?)
   - Where do padding chars go? (Between escape and raw? After raw? Split?)
   - Where do disambiguation bits for entry/exit boundary blocks go?
   - Is the layout fixed, or does it vary based on escape char choice or
     raw section length?

5. **Termination signaling:** How does the decoder know when a raw section
   ends? Options include:
   - Length prefix (explicit byte count before raw data)
   - Sentinel/escape at the end (scan until non-raw char)
   - Implicit from block alignment (raw ends at next Z85 block boundary)
   - Hybrid (length for short, sentinel for long)

## 11. Related Design Theme

This encoding shares a design philosophy with
[zipng](https://github.com/nickel-org/zipng): making binary data more
transparent. zipng embeds ZIP archives in PNG images (binary data visually
transparent via an image format). Extended Z85 embeds raw text in Z85 encoding
(binary data textually transparent via a text encoding). Both optimize for
legibility within the constraints of a binary-safe container format.
