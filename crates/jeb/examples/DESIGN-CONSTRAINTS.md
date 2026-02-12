# Extended Z85 — Design Constraints & Analysis

*Living document. Captures the invariants, constraints, and analytical findings
that will inform the final design. Not a specification — a foundation for one.*

## 0. Assumptions

These are stated up front to avoid burying them in later sections.

**Motivation.** Z85 encodes arbitrary binary data as printable ASCII, expanding
4 bytes into 5 characters (~25% overhead). When the input already contains
printable ASCII (text strings, human-readable headers, structured data), that
overhead is pure waste — the bytes are already printable. This design extends
Z85 with escape sequences that let the encoder pass those bytes through raw,
eliminating the overhead for text-heavy regions.

**Encoder/decoder asymmetry.** The encoder is smart; the decoder is simple. The
encoder decides opportunistically when raw passthrough is beneficial. The decoder
just follows unambiguous rules to reconstruct the original bytes. The format
doesn't need to be optimal for all inputs — just unambiguous for any valid
encoding.

**Self-signaling.** The presence of non-Z85 characters (escape chars) in the
output self-signals that this is extended Z85, not standard Z85. A standard Z85
decoder will reject the escape characters as invalid. This is acceptable —
extended Z85 is a superset, not a drop-in replacement.

**Data distribution.** Stability percentages in §5 and §6 assume uniformly
random byte values. Real data (small integers, ASCII-adjacent values, structured
headers) will often have higher stability rates. The uniform-random case
represents a conservative baseline, not a worst case — an adversarial encoder
could always avoid unstable cuts. The percentages describe "what fraction of
byte values allow a stable cut at this position."

**Block alignment.** "Block-aligned" always means aligned to Z85's 4-byte /
5-character block boundaries relative to the **start of the Z85 stream**, not
relative to the raw section or any other reference point.

**Raw byte order.** Bytes within raw sections appear in their original
sequential order, matching the input stream. Endianness discussion (§6) applies
only to how partial Z85 blocks encode boundary bytes, not to raw data ordering.

## 1. Mental Model

This is a **standard Z85 stream** with opportunistic escape sequences that allow
raw (unencoded) byte passthrough. Not a new format — Z85 with extensions.

## 2. Priority Order

### P1: Correctness & The Position Invariant

All Z85-encoded portions of the output must appear at the **exact same character
positions** as they would in a standard Z85 encoding of the same input data.
The position invariant applies to Z85 blocks that remain Z85-encoded — raw
sections replace Z85 blocks entirely and occupy the same character positions
those blocks would have used.

This means:
- Output length is always **≤ standard Z85 length** for the same input (the
  savings come from raw sections using N characters instead of `⌈N×5/4⌉`)
- Escape sequences + raw bytes + overhead must consume exactly the character
  positions that standard Z85 would have used for those input bytes
- Visual diff between our output and standard Z85 output shows identical Z85
  blocks in identical positions — escape sequences *replace* Z85 blocks, they
  don't shift them

This is a **constraint the encoder must satisfy**. Any encoding that violates
position invariance is invalid, regardless of whether the decoder could
reconstruct the bytes.

**Exception — mid-block transitions:** When we cut a Z85 block partway through
to begin or end a raw section, the partial block's Z85 characters might differ
from what standard Z85 would produce (because we're encoding fewer bytes with a
different convention). This is acceptable as long as the decoder can
unambiguously reconstruct the original bytes.

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

A non-Z85 escape character signals the transition from standard Z85 decoding to
raw passthrough. The escape character must appear within the character positions
of the Z85 block(s) being replaced — practically, within ~5 characters of the
transition point. (This is a design target, not yet a hard specification. The
exact bound depends on layout decisions in §10 Q4.)

The decoder must know the raw section length **before reading raw bytes** (see
§9). No scanning or sentinel detection.

Raw sections can be **any byte length** (not constrained to multiples of 4 bytes
or 5 characters). The encoding is opportunistic — if a given length at a given
alignment would cause problems (insufficient budget, unstable boundary), the
encoder simply doesn't use it.

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

| Context | Z85 already breaks? | Additional break characters |
|---------|---------------------|---------------------------|
| JSON double-quoted strings | **NO** | `"` `\` |
| Shell single-quoted strings | **NO** | `'` |
| CSV unquoted fields | **NO** | `,` `"` (`;` `|` in variants) |
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

The previous design (IDEATION.md) used all 6 characters from Tiers 1-2:
`_` `~` `` ` `` `|` `,` `;`. This preserves JSON string compatibility — likely
the most important context for jeb. This constraints document leaves the escape
character count as an open design decision (§10 Q1). The number of escape
characters and how their information capacity is allocated is central to the
design (see §8).

## 5. Mid-Block Stability (Entry Boundaries)

This section analyzes **entry boundaries** — where the encoder cuts a Z85 block
to begin a raw section, emitting partial Z85 characters for the bytes before
the cut. Exit boundaries are analyzed separately in §6.

When cutting a Z85 block after K known bytes, the K+1 leading Z85 characters
may or may not be "stable" (uniquely determined regardless of the unknown
bytes' values).

**Definition:** Stability = the fraction of byte values (for the known bytes)
where the leading Z85 characters are the same for ALL possible values of the
remaining unknown bytes. In other words: positions where the encoder can emit
partial Z85 characters without needing disambiguation.

| Cut after K bytes | Characters emitted | Stable (%) | Disambiguation bits when not |
|-------------------|-------------------|------------|------------------------------|
| 1 | 1 leading character | 68.0% | ~2 bits (3-4 candidates) |
| 2 | 2 leading characters | 89.3% | ~4 bits (~9-10 candidates) |
| 3 | 3 leading characters | 96.4% | ~5 bits (~27-28 candidates) |

### Why These Numbers

Z85 encodes 4-byte blocks as big-endian u32 (`V`), then extracts base-85 digits
most-significant first. The leading digit `c0 = V / 85^4` depends primarily on
the first byte. Stability fails when the unknown low bytes could push V across
a `85^4` boundary (where `85^4 = 52,200,625`).

For a 1-byte entry cut, the first byte `b0` maps to a range
`[b0 × 2^24, (b0+1) × 2^24 - 1]` of width `2^24 = 16,777,216`. This range
spans at most one `85^4` boundary. When it doesn't cross a boundary, the
leading character is stable.

Of the 85 possible leading-digit values (0–84), the number that each `b0` maps
to determines stability:
- If `b0` maps to exactly 1 leading digit: stable (68% of byte values)
- If `b0` straddles a boundary → 2 possible leading digits: unstable, need
  ~2 bits to disambiguate which one

For 2-byte and 3-byte cuts, the same logic applies at finer granularity, with
stability improving because more bits are known.

## 6. Entry vs Exit Boundary Symmetry

### The Key Insight

Entry and exit boundaries have the **same cost structure** (~2 bits of
disambiguation per boundary byte) but **different stability profiles** due to
the position of known bytes within the Z85 block. This motivates asymmetric
default conventions.

- **Entry boundary:** Known bytes are at the START of the block (high-order in
  Z85's big-endian convention). Information lives in the **leading** Z85
  characters.
- **Exit boundary:** Known bytes are at the END of the block (low-order).
  Information lives in the **trailing** Z85 characters.

### Stability by Position

**Entry (prefix bytes → leading characters):**

| Bytes known | Characters emitted | Disambiguation needed | Naturally stable |
|------------|-------------------|----------------------|-----------------|
| 1 | 1 leading (char 0) | ~2 bits | 68% |
| 2 | 2 leading (chars 0-1) | ~4 bits | 89% |
| 3 | 3 leading (chars 0-2) | ~5 bits | 96% |

**Exit (suffix bytes → trailing characters):**

| Bytes known | Characters emitted | Disambiguation needed | Naturally stable |
|------------|-------------------|----------------------|-----------------|
| 1 | 1 trailing (char 4) | ~7 bits | 0% |
| 2 | 2 trailing (chars 3-4) | ~4 bits | (analysis pending) |
| 3 | 3 trailing (chars 2-4) | ~2 bits | (analysis pending) |

### ⚠️ Correction: Exit Trailing Char Is NOT Stable

An earlier version of this document claimed 100% stability for the trailing
character based on `0xFFFFFF00 mod 85 = 0`. **This was wrong.** The reasoning
error: `0xFFFFFF00` is one specific value of the unknown bytes (all `0xFF`),
not a proof that `V mod 85` is independent of the unknown bytes.

The actual math: since `2^8 ≡ 2^16 ≡ 2^24 ≡ 1 (mod 85)`, we get
`V mod 85 = (b0 + b1 + b2 + b3) mod 85`. The trailing Z85 digit depends on
the **sum of all four bytes mod 85**, not just the low byte. As the unknown
bytes vary, this sum takes all 85 possible residues → **0% stability**.

This means exit boundaries are **worse** than entry boundaries for partial
Z85 characters, not better. The entry leading character has 68% stability
because the quotient `V / 85^4` is dominated by the high byte. The exit
trailing character has 0% stability because the modulus `V mod 85` mixes
all bytes equally.

However, the disambiguation cost structure is still symmetric (~2 bits per
boundary byte). The difference is that exit boundaries **always** need
disambiguation for the trailing character, while entry boundaries need it
only 32% of the time for the leading character.

### Recommended Defaults

When the encoding has no budget to signal which convention is in use (e.g.,
tight budget with only 1 escape character), these defaults apply:

**Entry: BE (leading characters)** — The byte just before a raw section tends to
be a small value (length field, type tag, null terminator). Small values have
zeros in high bits, placing them far from `85^4` boundaries → higher stability.
68% baseline, likely higher for real data.

**Exit: Also BE (leading characters)** — With the trailing character 100%
stability disproven, the exit boundary has no mathematical advantage from using
trailing characters. BE leading characters give 68% natural stability (the
same as entry), which is better than 0% from trailing characters.

**Implication:** The asymmetric entry-BE/exit-LE convention may no longer be
justified on stability grounds. Both boundaries may benefit from the same BE
convention, simplifying the design. However, this needs further analysis —
the exit boundary's known bytes are at the END of the block, so "leading
characters" for exit means emitting characters that encode the high (unknown)
bytes, which is different from the entry case. The full implications of using
BE at exit boundaries need to be worked through.

When budget allows, the escape character choice can override these defaults
for specific data (see §8).

### Rejected Alternative: Direct Byte Encoding

Instead of using Z85's natural leading/trailing characters for boundary bytes,
we could encode them directly: map K bytes → K+1 Z85 characters via a bijection
independent of the block's other bytes. This gives 100% stability (no
disambiguation needed) at the cost of 1 extra character per boundary.

| Approach | Characters used | Disambiguation cost | Total cost |
|----------|----------------|--------------------|-----------| 
| Natural (leading/trailing) | K | ~2 bits/byte from escape char info | K characters + borrowed info bits |
| Direct encoding | K+1 | 0 | K+1 characters |

**Natural wins for tight budgets.** At budget=2 (8 raw bytes), natural uses 1
character + 1 escape = 2 characters of budget, with disambiguation packed into
the escape character's information bits. Direct encoding would need 2 characters
+ 1 escape = 3 characters — doesn't fit.

The key insight: each escape character choice provides `log2(N)` bits of free
information (where N is the number of distinct escape characters). Those bits
can carry disambiguation more efficiently than spending a full extra Z85
character (~6.4 bits of capacity) on it.

## 7. Padding Budget

**Budget** = `⌈N × 5/4⌉ - N`: the number of Z85 characters that standard
encoding would use for N bytes, minus the N characters consumed by raw
passthrough. This budget must cover: escape character(s), length encoding,
disambiguation bits, and any padding.

| Raw bytes | Standard Z85 characters | Budget | Notes |
|-----------|------------------------|--------|-------|
| 4 | 5 | 1 | Escape only. No mid-block cuts. |
| 5-8 | 7-10 | 2 | Escape + 1 character |
| 9-12 | 12-15 | 3 | Comfortable for most cases |
| 13-16 | 17-20 | 4 | Comfortable for all cases |
| N (large) | ~5N/4 | ~N/4 | Abundant |

### Break-Even Analysis

At 4 raw bytes: standard Z85 = 5 characters, raw passthrough = 4 bytes + 1
escape = 5 characters. **Zero net savings.** The benefit is transparency (raw
bytes are readable), not compactness. Net character savings begin at 5+ raw
bytes (budget ≥ 2, with at least 1 character saved after escape overhead).

For larger raw sections, savings grow linearly: N raw bytes save approximately
`N/4 - overhead` characters compared to standard Z85.

### Budget Requirements for Mid-Block Transitions

| Configuration | Minimum budget needed |
|--------------|---------------------|
| Block-aligned, no explicit length | 1 (escape only) |
| Block-aligned, with length | 2 (escape + length character) |
| Mid-block entry OR exit | 2 (escape + disambiguation) |
| Mid-block both + length | 4 (escape + length + 2× disambiguation) |

Note: "block-aligned, no explicit length" at budget=1 requires the escape
character alone to imply the raw section length. This is only possible if a
specific escape character is dedicated to a specific length (spending one of the
available escape characters on this case).

### Critical Thresholds

- **4 bytes (budget=1):** Block-aligned raw only. No mid-block cuts. Length
  must be implied by escape character choice.
- **5-8 bytes (budget=2):** Mid-block cut at ONE boundary possible. Tight.
- **9-12 bytes (budget=3):** Mid-block at both boundaries feasible.
- **13+ bytes (budget=4+):** Everything works comfortably.

## 8. The Escape Character Budget

Each escape character beyond Z85's 85 is a precious resource. It costs context
compatibility (§4) and provides information at critical decision points.

The choice of which escape character to use at a given point provides
`log2(N)` bits of information, where N is the total number of escape characters.
These bits can be **multiplexed** across different purposes — they aren't
limited to one use per character. For example, with 4 escape characters (2 bits):
the high bit could signal endianness while the low bit signals a length class.

**What escape character info bits could encode** (can be combined):
- Endianness of boundary blocks (BE vs LE)
- Length class (short vs long raw sections)
- Entry/exit boundary convention
- Part of disambiguation information for mid-block cuts

| Escape characters | Info bits (`log2(N)`) | Example allocation |
|------------------|----------------------|-------------------|
| 1 | 0 | Fixed conventions, length implicit |
| 2 | 1 | BE/LE, OR 2 length classes |
| 3 | ~1.6 | 3-way split (e.g., short/long/to-end) |
| 4 | 2 | Endianness × length class |
| 6 | ~2.6 | Full IDEATION.md design |

The number and allocation of escape characters is **undecided**. The analysis
above maps the constraint space; the choice depends on which capabilities matter
most for real-world data.

### Information Flow Constraint

For escape character info bits to carry disambiguation, the decoder must
encounter the escape character **before** it needs to decode the boundary block.
This constrains the layout: the escape character must precede (or be part of)
the prefix that appears before raw data. This is compatible with the length-
before-data constraint (§9) — both require a prefix-oriented layout.

## 9. Resolved Design Decisions

These were open questions; they've been answered:

- **Minimum raw section length: 4 bytes.** The format supports raw sections as
  short as 4 bytes. At this length, budget=1, which only allows block-aligned
  sections with length implied by escape character choice. There are no net
  character savings (5 characters either way), but transparency (readable raw
  bytes) is still valuable. The encoder is opportunistic — it uses raw sections
  wherever they fit and provide value.

- **Mid-block cuts: support both entry AND exit.** We are being opportunistic;
  if a mid-block cut is possible and beneficial, do it. Both boundaries should
  be supported.

- **Complexity budget: high.** This project accepts high complexity in exchange
  for value. Whether that manifests as asymmetric conventions or a unified
  approach, the design will be carefully specified. (The earlier decision for
  asymmetric entry-BE/exit-LE was based on the now-disproven 100% exit trailing
  char stability. The question of optimal exit convention is reopened.)

- **Length before data; no sentinels.** The decoder must know the raw section
  length before reading raw bytes. Length is encoded either implicitly (in the
  escape character choice) or explicitly (in prefix characters between the
  escape and the raw data). Sentinels are rejected. Special case: a "raw to end
  of input" escape where no termination is needed.

## 10. Open Design Questions

These require discussion and decision before a specification can be written.
Grouped by dependency:

### Design Logic

1. **How many escape characters?** Tiers 1-2 give up to 6 without breaking
   JSON. What's the right number? Does the layout complexity scale with the
   number of escape characters, or is there a unified layout that works for any?

2. **What information does each escape character encode?** The `log2(N)` bits
   can be multiplexed across endianness, length classes, disambiguation, and
   boundary conventions. What's the optimal allocation?

### Layout & Encoding

3. **How are disambiguation bits laid out?** Where in the output stream do they
   go? Are they encoded as part of the escape character choice, as padding
   characters, or as part of a length prefix? How does this interact with the
   information flow constraint (§8)?

4. **Raw block internal layout:** What is the syntax and ordering of non-raw
   components? Specifically:
   - Where does the escape character go relative to the raw data?
   - Where does length information go? (Implied by escape? Explicit prefix?)
   - Where do padding characters go?
   - Where do disambiguation bits for entry/exit boundary blocks go?
   - Is the layout fixed, or does it vary based on escape character choice or
     raw section length?

### Edge Cases

5. **Stream boundaries:** Does the entry/exit analysis change at the very start
   or end of the encoded stream (no preceding/following Z85 block)?

6. **Consecutive raw sections:** Can two raw sections be adjacent with no Z85
   blocks between them? If so, what separates them? If not, what's the minimum
   Z85 gap?

## 11. Related Design Theme

This encoding shares a design philosophy with zipng: making binary data more
transparent. zipng embeds ZIP archives in PNG images (binary data visually
transparent via an image format). Extended Z85 embeds raw text in Z85 encoding
(binary data textually transparent via a text encoding). Both optimize for
legibility within the constraints of a binary-safe container format.
